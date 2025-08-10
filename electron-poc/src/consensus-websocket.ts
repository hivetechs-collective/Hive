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
  
  constructor(url: string, callbacks: ConsensusStreamCallbacks) {
    this.url = url;
    this.callbacks = callbacks;
  }
  
  private async connectViaIPC(): Promise<void> {
    try {
      // Set up message listeners first
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
  
  async startConsensus(query: string, profile?: string): Promise<void> {
    if (!this.connected) {
      this.callbacks.onError?.('WebSocket not connected');
      return;
    }
    
    const message = {
      type: 'start_consensus',
      query,
      profile
    };
    
    try {
      await (window as any).websocketAPI.send(JSON.stringify(message));
      console.log('Sent consensus request via IPC');
    } catch (error) {
      console.error('Failed to send consensus request:', error);
      this.callbacks.onError?.('Failed to send message');
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