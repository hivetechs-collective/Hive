/**
 * WebSocket streaming client for Hive Consensus
 * Handles real-time consensus pipeline updates
 */

export interface ConsensusStreamCallbacks {
  onProfileLoaded?: (name: string, models: string[]) => void;
  onStageStarted?: (stage: string, model: string) => void;
  onStreamChunk?: (stage: string, chunk: string) => void;
  onStageProgress?: (stage: string, percentage: number, tokens: number) => void;
  onStageCompleted?: (stage: string, tokens: number, cost: number) => void;
  onConsensusComplete?: (result: string, totalTokens: number, totalCost: number) => void;
  onError?: (message: string) => void;
  onAIHelperDecision?: (directMode: boolean, reason: string) => void;
  onConnectionStateChange?: (connected: boolean) => void;
  // NEW: Iterative deliberation callbacks
  onLLMStarted?: (round: number, llm: string, model: string) => void;
  onTokenUpdate?: (tokens: number, cost: number, currentLLM: string, round: number) => void;
}

export class ConsensusWebSocket {
  private ws: WebSocket | null = null;
  private callbacks: ConsensusStreamCallbacks;
  private url: string;
  private reconnectTimer: number | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private isManuallyDisconnected = false;
  private pingInterval: number | null = null;
  private connected = false;
  private listenersSetup = false;
  
  constructor(url: string, callbacks: ConsensusStreamCallbacks) {
    this.url = url;
    this.callbacks = callbacks;
    this.setupIPCListeners();
  }
  
  private setupIPCListeners(): void {
    // Only set up listeners once to avoid duplication
    if (this.listenersSetup) {
      return;
    }
    
    this.listenersSetup = true;
    
    // Set up message listeners once
    (window as any).websocketAPI.onMessage((data: string) => {
      try {
        const message = JSON.parse(data);
        this.handleMessage(message);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    });
    
    (window as any).websocketAPI.onError((error: string) => {
      console.error('WebSocket error via IPC:', error);
      this.callbacks.onError?.('WebSocket connection error');
      this.connected = false;
      this.callbacks.onConnectionStateChange?.(false);
    });
    
    (window as any).websocketAPI.onClose(() => {
      console.log('WebSocket closed via IPC');
      this.connected = false;
      this.callbacks.onConnectionStateChange?.(false);
      this.scheduleReconnect();
    });
  }
  
  private async connectViaIPC(): Promise<void> {
    try {
      
      // Connect via IPC
      const result = await (window as any).websocketAPI.connect(this.url);
      if (result.connected) {
        console.log('WebSocket connected successfully via IPC');
        this.connected = true;
        this.reconnectAttempts = 0;
        this.callbacks.onConnectionStateChange?.(true);
        this.startPingInterval();
      }
    } catch (error) {
      console.error('Failed to connect WebSocket via IPC:', error);
      this.callbacks.onError?.('WebSocket connection failed');
      this.connected = false;
    }
  }
  
  connect(): void {
    if (this.connected) {
      console.log('WebSocket already connected');
      return;
    }
    
    this.isManuallyDisconnected = false;
    console.log(`Connecting via IPC WebSocket proxy: ${this.url}`);
    
    // Use IPC-based WebSocket instead of direct connection
    this.connectViaIPC();
  }
  
  private connectDirectly(): void {
    // Fallback to direct connection (kept for reference but not used)
    try {
      this.ws = new WebSocket(this.url);
      console.log('WebSocket created, state:', this.ws.readyState);
      
      this.ws.onopen = () => {
        console.log('WebSocket connected successfully');
        this.reconnectAttempts = 0;
        this.callbacks.onConnectionStateChange?.(true);
        
        // Start ping interval to keep connection alive
        this.startPingInterval();
      };
      
      this.ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };
      
      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.callbacks.onError?.('WebSocket connection error');
      };
      
      this.ws.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason);
        this.callbacks.onConnectionStateChange?.(false);
        this.stopPingInterval();
        
        // Attempt reconnection if not manually disconnected
        if (!this.isManuallyDisconnected && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.scheduleReconnect();
        }
      };
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      this.callbacks.onError?.(`Failed to connect: ${error}`);
    }
  }
  
  private handleMessage(message: any): void {
    switch (message.type) {
      case 'profile_loaded':
        this.callbacks.onProfileLoaded?.(message.name, message.models);
        break;
        
      case 'stage_started':
        this.callbacks.onStageStarted?.(message.stage, message.model);
        break;
        
      case 'stream_chunk':
        this.callbacks.onStreamChunk?.(message.stage, message.chunk);
        break;
        
      case 'stage_progress':
        this.callbacks.onStageProgress?.(message.stage, message.percentage, message.tokens);
        break;
        
      case 'stage_completed':
        this.callbacks.onStageCompleted?.(message.stage, message.tokens, message.cost);
        break;
        
      case 'consensus_complete':
        this.callbacks.onConsensusComplete?.(message.result, message.total_tokens, message.total_cost);
        break;
        
      case 'error':
        this.callbacks.onError?.(message.message);
        break;
        
      case 'llm_started':
        this.callbacks.onLLMStarted?.(message.round, message.llm, message.model);
        break;
        
      case 'token_update':
        this.callbacks.onTokenUpdate?.(message.tokens, message.cost, message.current_llm, message.round);
        break;
        
      case 'ai_helper_decision':
        this.callbacks.onAIHelperDecision?.(message.direct_mode, message.reason);
        break;
        
      default:
        console.warn('Unknown message type:', message.type);
    }
  }
  
  private scheduleReconnect(): void {
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);
    this.reconnectAttempts++;
    
    console.log(`Scheduling reconnect attempt ${this.reconnectAttempts} in ${delay}ms`);
    
    this.reconnectTimer = window.setTimeout(() => {
      this.connect();
    }, delay);
  }
  
  private startPingInterval(): void {
    // Send ping every 30 seconds to keep connection alive
    this.pingInterval = window.setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'ping' }));
      }
    }, 30000);
  }
  
  private stopPingInterval(): void {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }
  
  async startConsensus(
    query: string, 
    profile?: string, 
    conversationId?: string,
    context?: Array<{role: string, content: string}>
  ): Promise<void> {
    // Reproduce exact 7-stage streaming consensus from previous version
    try {
      console.log('Starting 7-stage consensus pipeline:', query);
      
      // Direct neural consciousness control (documented API)
      const neural = (window as any).neuralConsciousness;
      if (neural) {
        // Start processing mode
        await neural.show();
        
        // Stage 1: Memory
        await neural.updatePhase('memory');
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Stage 2: Context (synthesis)
        await neural.updatePhase('synthesis');
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Stage 3: Routing (classification)  
        await neural.updatePhase('classification');
        await new Promise(resolve => setTimeout(resolve, 500));
        
        // Stage 4: Generator (will animate during actual LLM processing)
        await neural.updatePhase('generator');
      }
      
      // DirectConsensusEngine will emit all stage events in real-time (no artificial timeouts)
      console.log('🔴 ConsensusWebSocket: About to call backendAPI.runQuickConsensus with query:', query);
      const result = await (window as any).backendAPI.runQuickConsensus({
        query: query,
        profile: profile
      });
      console.log('🔴 ConsensusWebSocket: Result from runQuickConsensus:', result);
      
      if (result && result.result) {
        // Complete remaining neural graphics stages after LLM finishes
        if (neural) {
          await neural.updatePhase('refiner');
          await new Promise(resolve => setTimeout(resolve, 300));
          await neural.updatePhase('validator');
          await new Promise(resolve => setTimeout(resolve, 300));
          await neural.updatePhase('curator');
          await new Promise(resolve => setTimeout(resolve, 300));
          await neural.showCompletion();
          setTimeout(() => neural.hide(), 2000);
        }
        
        this.callbacks.onConsensusComplete?.(result.result, 0, 0);
        console.log('7-stage consensus completed successfully');
      } else {
        throw new Error('No result from consensus engine');
      }
    } catch (error) {
      console.error('7-stage consensus failed:', error);
      this.callbacks.onError?.(`Consensus failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }
  
  cancelConsensus(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type: 'cancel_consensus' }));
    }
  }
  
  disconnect(): void {
    this.isManuallyDisconnected = true;
    
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    
    this.stopPingInterval();
    
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
  
  isConnected(): boolean {
    return this.connected;
  }
}

// Stage display mapping
export const STAGE_DISPLAY_NAMES: Record<string, string> = {
  'Generator': 'generator',
  'Refiner': 'refiner',
  'Validator': 'validator',
  'Curator': 'curator'
};

// Utility to format tokens and cost
export function formatTokens(tokens: number): string {
  if (tokens >= 1000000) {
    return `${(tokens / 1000000).toFixed(2)}M`;
  } else if (tokens >= 1000) {
    return `${(tokens / 1000).toFixed(1)}k`;
  }
  return tokens.toString();
}

export function formatCost(cost: number): string {
  if (cost < 0.01) {
    return `$${(cost * 100).toFixed(3)}¢`;
  }
  return `$${cost.toFixed(3)}`;
}