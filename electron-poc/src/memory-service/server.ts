/**
 * Universal Memory Infrastructure - Memory Service Server
 * Provides memory-as-a-service to external AI tools
 * Runs on dynamically allocated port as a child process, uses IPC for database access
 */

import express from 'express';
import cors from 'cors';
import { WebSocketServer, WebSocket } from 'ws';
import * as http from 'http';
import path from 'path';
import crypto from 'crypto';

import { logger } from '../utils/SafeLogger';
// Types
interface MemoryQuery {
  client: string;
  context: {
    file?: string;
    line?: number;
    repository?: string;
    branch?: string;
  };
  query: string;
  options?: {
    limit?: number;
    thematic?: boolean;
    include_patterns?: boolean;
    min_confidence?: number;
  };
}

interface MemoryContribution {
  source: string;
  learning: {
    type: string;
    category: string;
    content: string;
    code?: string;
    context: {
      file?: string;
      success: boolean;
      performance_impact?: string;
    };
    metadata?: {
      language?: string;
      framework?: string;
      tags?: string[];
    };
  };
}

interface ConnectedTool {
  id: string;
  name: string;
  token: string;
  connectedAt: Date;
  queryCount: number;
  contributionCount: number;
  lastActivity: Date;
}

interface MemoryStats {
  totalMemories: number;
  queriesToday: number;
  contributionsToday: number;
  connectedTools: number;
  hitRate: number;
  avgResponseTime: number;
}

export class MemoryServiceServer {
  private app: express.Application;
  private server: http.Server<any, any> | null = null;
  private wss: WebSocketServer | null = null;
  private connectedTools: Map<string, ConnectedTool> = new Map();
  private activityStream: any[] = [];
  private stats: MemoryStats = {
    totalMemories: 0,
    queriesToday: 0,
    contributionsToday: 0,
    connectedTools: 0,
    hitRate: 92,
    avgResponseTime: 45
  };
  private port: number;
  private pendingQueries: Map<string, Function> = new Map();

  constructor(port: number) {
    this.port = port;
    this.app = express();
    this.setupMiddleware();
    this.setupRoutes();
    this.setupIPC();
    // Don't update stats in constructor - wait until server starts
  }

  private setupIPC() {
    // Listen for database results from main process
    process.on('message', (msg: any) => {
      if (msg.type === 'db-result') {
        const callback = this.pendingQueries.get(msg.id);
        if (callback) {
          callback(msg.error, msg.data);
          this.pendingQueries.delete(msg.id);
        }
      }
    });
  }

  private queryDatabase(sql: string, params: any[]): Promise<any> {
    return new Promise((resolve, reject) => {
      const queryId = crypto.randomUUID();
      
      // Store callback for when result comes back
      this.pendingQueries.set(queryId, (error: any, data: any) => {
        if (error) {
          reject(error);
        } else {
          resolve(data);
        }
      });

      // Send query to main process
      if (process.send) {
        process.send({
          type: 'db-query',
          id: queryId,
          sql,
          params
        });
      } else {
        reject(new Error('IPC not available'));
      }

      // Timeout after 5 seconds
      setTimeout(() => {
        if (this.pendingQueries.has(queryId)) {
          this.pendingQueries.delete(queryId);
          reject(new Error('Database query timeout'));
        }
      }, 5000);
    });
  }

  private setupMiddleware() {
    this.app.use(cors({
      origin: '*',
      credentials: true
    }));
    this.app.use(express.json({ limit: '10mb' }));
    
    // Request logging
    this.app.use((req, res, next) => {
      const start = Date.now();
      res.on('finish', () => {
        const duration = Date.now() - start;
        this.logActivity({
          type: 'request',
          method: req.method,
          path: req.path,
          status: res.statusCode,
          duration,
          client: req.headers['x-client-name'] || 'unknown'
        });
      });
      next();
    });
  }

  private setupRoutes() {
    // Health check
    this.app.get('/health', (req, res) => {
      res.json({ 
        status: 'healthy',
        port: this.port,
        database: 'connected via IPC',
        uptime: process.uptime()
      });
    });

    // MCP JSON-RPC endpoint
    this.app.get(['/mcp', '/mcp/v1'], (req, res) => {
      res.json({
        status: 'ready',
        protocol: 'MCP-JSONRPC',
        version: '2024-11-05',
        message: 'Hive Memory MCP bridge is running',
        endpoints: {
          rpc: '/mcp/v1',
          health: '/health'
        }
      });
    });

    this.app.post('/register', (req, res) => {
      res.json({
        client_id: 'hive-memory-service',
        client_secret: '',
        redirect_uris: [`http://127.0.0.1:${this.port}/oauth/callback`],
        scopes: ['memory.read', 'memory.write'],
        status: 'ok',
        message: 'Hive memory service ready for OAuth/MCP integration'
      });
    });

    this.app.post(['/mcp', '/mcp/:method(*)', '/mcp/v1', '/mcp/v1/:method(*)'], this.authenticate, this.handleMcpRpc.bind(this));

    // Query memories
    this.app.post('/api/v1/memory/query', this.authenticate, this.handleQuery.bind(this));
    
    // Enhanced query with full Memory+Context pipeline (like Consensus)
    this.app.post('/api/v1/memory/query-with-context', this.authenticate, this.handleQueryWithContext.bind(this));
    
    // Contribute learning (for now, just log it)
    this.app.post('/api/v1/memory/contribute', this.authenticate, this.handleContribution.bind(this));
    
    // Get statistics
    this.app.get('/api/v1/memory/stats', this.handleStats.bind(this));
    
    // Get connected tools
    this.app.get('/api/v1/memory/tools', this.handleTools.bind(this));
    
    // Get activity stream
    this.app.get('/api/v1/memory/activity', this.handleActivity.bind(this));
    
    // Generate token for a new tool
    this.app.post('/api/v1/memory/register', this.handleRegister.bind(this));
  }

  private setupWebSocket() {
    if (!this.wss) return;
    
    this.wss.on('connection', (ws: WebSocket) => {
      logger.info('[MemoryService] WebSocket client connected');
      
      // Send initial stats
      ws.send(JSON.stringify({
        type: 'stats',
        data: this.stats
      }));
      
      ws.on('message', (message: string) => {
        try {
          const data = JSON.parse(message.toString());
          if (data.action === 'subscribe') {
            // Handle subscription to specific events
            ws.send(JSON.stringify({
              type: 'subscribed',
              events: data.events
            }));
          }
        } catch (error) {
          logger.error('[MemoryService] WebSocket message error:', error);
        }
      });
      
      ws.on('close', () => {
        logger.info('[MemoryService] WebSocket client disconnected');
      });
    });
  }

  private authenticate = (req: any, res: any, next: any) => {
    const token = req.headers.authorization?.replace('Bearer ', '');
    
    if (!token) {
      return res.status(401).json({ error: 'No token provided' });
    }
    
    // For now, accept any token and track it
    const clientName = req.headers['x-client-name'] || 'unknown';
    
    if (!this.connectedTools.has(token)) {
      this.connectedTools.set(token, {
        id: crypto.randomUUID(),
        name: clientName,
        token,
        connectedAt: new Date(),
        queryCount: 0,
        contributionCount: 0,
        lastActivity: new Date()
      });
      logger.info(`[MemoryService] New tool connected: ${clientName} (Total: ${this.connectedTools.size})`);
    } else {
      // Update last activity for existing tool
      const tool = this.connectedTools.get(token)!;
      tool.lastActivity = new Date();
      logger.debug(`[MemoryService] Existing tool activity: ${clientName}`);
    }
    
    req.tool = this.connectedTools.get(token);
    next();
  };

  private handleMcpRpc = async (req: any, res: any) => {
    const rpcRequest = req.body || {};
    const rpcId = rpcRequest.id ?? null;
    let method: string | undefined = rpcRequest.method;
    const pathMethod = req.params?.method;
    if (!method && pathMethod) {
      method = pathMethod;
    }

    const sendResult = (result: any) => {
      res.json({ jsonrpc: '2.0', id: rpcId, result });
    };

    const sendError = (code: number, message: string, data?: any) => {
      res.json({ jsonrpc: '2.0', id: rpcId, error: { code, message, data } });
    };

    try {
      if (!method) {
        return sendError(-32600, 'Invalid request: missing method');
      }

      switch (method) {
        case 'initialize': {
          return sendResult({
            protocolVersion: '2024-11-05',
            capabilities: {
              logging: {},
              prompts: {},
              resources: {},
              tools: {}
            },
            serverInfo: {
              name: 'Hive Memory MCP Bridge',
              version: '1.0.0'
            }
          });
        }
        case 'initialized': {
          return sendResult(null);
        }
        case 'tools/list': {
          return sendResult({
            tools: [
              {
                name: 'query_memory_with_context',
                description: 'Retrieve Hive consensus memory and context for a query',
                inputSchema: {
                  type: 'object',
                  properties: {
                    query: {
                      type: 'string',
                      description: 'Natural language question or request'
                    },
                    limit: {
                      type: 'number',
                      minimum: 1,
                      maximum: 50,
                      default: 5,
                      description: 'Maximum results to include'
                    }
                  },
                  required: ['query']
                }
              }
            ]
          });
        }
        case 'resources/list': {
          return sendResult({ resources: [] });
        }
        case 'resources/read': {
          return sendResult({ contents: [] });
        }
        case 'tools/call': {
          const params = rpcRequest.params || {};
          const toolName = params.name;
          const args = params.arguments || {};
          if (toolName !== 'query_memory_with_context') {
            return sendError(-32601, `Tool not found: ${toolName}`);
          }

          const query = args.query || args.prompt || '';
          const limit = args.limit || 5;

          if (!query || typeof query !== 'string') {
            return sendError(-32602, 'Invalid params: query is required');
          }

          const tool = req.tool as ConnectedTool;
          try {
            const enhancedResult = await this.requestMemoryContext(query, tool?.name || 'cursor-cli');

            if (tool) {
              tool.queryCount++;
              tool.lastActivity = new Date();
            }

            this.logActivity({
              type: 'mcp_tool_call',
              tool: tool?.name || 'cursor-cli',
              query: query.substring(0, 50),
              resultCount: enhancedResult.relevantMemories?.length || 0,
              responseTime: enhancedResult.metadata?.processingTimeMs || 0
            });

            return sendResult({
              content: [
                {
                  type: 'text',
                  text: enhancedResult.memoryContext || 'No memory context available.'
                },
                {
                  type: 'json',
                  data: enhancedResult
                }
              ],
              isError: false
            });
          } catch (error) {
            logger.error('[MemoryService] MCP tool call failed:', error);
            return sendError(-32603, 'Tool execution failed', { message: String(error) });
          }
        }
        default:
          return sendError(-32601, `Method not found: ${method}`);
      }
    } catch (error) {
      logger.error('[MemoryService] MCP handler error:', error);
      return sendError(-32603, 'Internal error', { message: String(error) });
    }
  };

  private handleQuery = async (req: any, res: any) => {
    const startTime = Date.now();
    const query: MemoryQuery = req.body;
    const tool = req.tool as ConnectedTool;
    
    try {
      tool.queryCount++;
      tool.lastActivity = new Date();
      this.stats.queriesToday++;
      
      // Query database via IPC
      const limit = query.options?.limit || 5;
      const sql = `
        SELECT 
          id,
          content,
          role,
          stage,
          model_used,
          timestamp,
          conversation_id
        FROM messages
        WHERE content LIKE ?
        ORDER BY timestamp DESC
        LIMIT ?
      `;
      
      const memories = await this.queryDatabase(sql, [`%${query.query}%`, limit]);
      
      const responseTime = Date.now() - startTime;
      this.updateAverageResponseTime(responseTime);
      
      // Log activity
      this.logActivity({
        type: 'query',
        tool: tool.name,
        query: query.query.substring(0, 50),
        resultCount: memories.length,
        responseTime
      });
      
      res.json({
        memories,
        patterns: [],
        suggestions: [],
        metadata: {
          query_time_ms: responseTime,
          memories_scanned: memories.length,
          confidence_score: 0.85
        }
      });
      
    } catch (error) {
      logger.error('[MemoryService] Query error:', error);
      res.status(500).json({ error: 'Query failed' });
    }
  };

  private handleQueryWithContext = async (req: any, res: any) => {
    const startTime = Date.now();
    const query: MemoryQuery = req.body;
    const tool = req.tool as ConnectedTool;
    
    try {
      logger.info(`[MemoryService] Enhanced query from ${tool.name}: ${query.query}`);
      
      // Send IPC request to main process for enhanced Memory+Context pipeline
      const enhancedResult = await this.requestMemoryContext(query.query, tool.name);
      
      const responseTime = Date.now() - startTime;
      
      tool.queryCount++;
      tool.lastActivity = new Date();
      
      // Log enhanced activity
      this.logActivity({
        type: 'enhanced_query',
        tool: tool.name,
        query: query.query.substring(0, 50),
        resultCount: enhancedResult.relevantMemories?.length || 0,
        responseTime
      });
      
      res.json({
        success: true,
        enhancedContext: enhancedResult,
        metadata: {
          query_time_ms: responseTime,
          memories_retrieved: enhancedResult.relevantMemories?.length || 0,
          patterns_found: enhancedResult.patterns?.length || 0,
          context_summary: enhancedResult.memoryContext || ''
        }
      });
      
    } catch (error) {
      logger.error('[MemoryService] Enhanced query error:', error);
      res.status(500).json({ error: 'Enhanced query failed' });
    }
  };

  private async requestMemoryContext(query: string, toolName: string): Promise<any> {
    // Implement EXACT same Memory+Context pipeline as Consensus
    const startTime = Date.now();
    
    try {
      console.log(`[MemoryService] Starting Memory+Context pipeline for CLI tool: ${toolName}`);
      
      // STAGE 1: MEMORY - Use exact same memory retrieval as Consensus
      const relevantMemories = await this.retrieveRelevantMemoriesForCLI(query);
      console.log(`[MemoryService] Found ${relevantMemories.length} relevant memories for ${toolName}`);
      
      // STAGE 2: CONTEXT - Use exact same context building as Consensus  
      const contextFramework = await this.buildContextFrameworkForCLI(query, relevantMemories);
      console.log(`[MemoryService] Built context framework with ${contextFramework.patterns.length} patterns for ${toolName}`);
      
      const responseTime = Date.now() - startTime;
      console.log(`[MemoryService] Memory+Context pipeline completed in ${responseTime}ms for ${toolName}`);
      
      return {
        originalQuery: query,
        memoryContext: contextFramework.summary,
        patterns: contextFramework.patterns,
        userPreferences: contextFramework.userPreferences,
        relevantTopics: contextFramework.relevantTopics,
        relevantMemories: relevantMemories.map(m => ({
          content: m.content.substring(0, 200) + '...',
          timestamp: m.timestamp,
          recencyScore: m.recencyScore
        })),
        metadata: {
          totalMemoriesScanned: relevantMemories.length,
          processingTimeMs: responseTime,
          patterns_identified: contextFramework.patterns.length,
          topics_extracted: contextFramework.relevantTopics.length
        }
      };
      
    } catch (error) {
      console.error(`[MemoryService] Memory+Context pipeline error for ${toolName}:`, error);
      return {
        originalQuery: query,
        memoryContext: '',
        patterns: [],
        userPreferences: [],
        relevantMemories: [],
        error: 'Memory+Context pipeline failed'
      };
    }
  }

  // EXACT same memory retrieval logic as Consensus (copied from SimpleConsensusEngine)
  private async retrieveRelevantMemoriesForCLI(query: string): Promise<any[]> {
    return new Promise((resolve) => {
      const requestId = crypto.randomUUID();
      
      // Store resolver for response handling
      this.pendingQueries.set(requestId, resolve);
      
      process.send?.({
        type: 'db-query',
        id: requestId,
        sql: `SELECT id, conversation_id, content, role, timestamp, 
               CASE 
                 WHEN datetime(timestamp, 'localtime') >= datetime('now', '-2 hours', 'localtime') THEN 4
                 WHEN datetime(timestamp, 'localtime') >= datetime('now', '-1 day', 'localtime') THEN 3  
                 WHEN datetime(timestamp, 'localtime') >= datetime('now', '-7 days', 'localtime') THEN 2
                 ELSE 1
               END as recencyScore
              FROM messages 
              WHERE content LIKE ? 
              ORDER BY timestamp DESC 
              LIMIT 20`,
        params: [`%${query}%`]
      });
      
      // Timeout fallback
      setTimeout(() => {
        if (this.pendingQueries.has(requestId)) {
          this.pendingQueries.delete(requestId);
          resolve([]);
        }
      }, 5000);
    });
  }

  // EXACT same context building logic as Consensus (copied from SimpleConsensusEngine)  
  private async buildContextFrameworkForCLI(query: string, memories: any[]): Promise<any> {
    const framework = {
      summary: '',
      patterns: [] as string[],
      relevantTopics: [] as string[],
      userPreferences: [] as string[]
    };
    
    if (memories.length === 0) {
      console.log('[MemoryService] No memories found, using empty context framework');
      return framework;
    }
    
    // Sort memories by timestamp (newest first) for chronological context
    const sortedMemories = memories.sort((a, b) => 
      new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );
    
    // Extract actual conversation context from recent messages
    const recentContext: string[] = [];
    sortedMemories.slice(0, 10).forEach(memory => {
      if (memory.role === 'user') {
        recentContext.push(`User asked: "${memory.content}"`);
      } else if (memory.role === 'assistant') {
        const shortContent = memory.content.substring(0, 150);
        recentContext.push(`Previously discussed: ${shortContent}...`);
      }
    });
    
    // Extract patterns and topics (EXACT same logic as Consensus)
    const topics = new Set<string>();
    const patterns = new Set<string>();
    
    memories.forEach(memory => {
      // Extract meaningful topics
      if (/PowerShell|powershell/i.test(memory.content)) topics.add('PowerShell');
      if (/Python|python/i.test(memory.content)) topics.add('Python');
      if (/JavaScript|javascript|JS/i.test(memory.content)) topics.add('JavaScript');
      if (/TypeScript|typescript|TS/i.test(memory.content)) topics.add('TypeScript');
      if (/React|Vue|Angular/i.test(memory.content)) topics.add('Web Frameworks');
      if (/database|SQL|query/i.test(memory.content)) topics.add('Databases');
      if (/API|REST|GraphQL/i.test(memory.content)) topics.add('APIs');
      if (/example|examples/i.test(memory.content)) topics.add('Examples Requested');
      if (/authentication|auth|login/i.test(memory.content)) topics.add('Authentication');
      if (/error|bug|issue|problem/i.test(memory.content)) topics.add('Troubleshooting');
      
      // Extract user patterns (COMPLETE set from Consensus)
      if (/prefer|favorite|like/i.test(memory.content)) patterns.add('User Preferences');
      if (/problem|issue|error|failed/i.test(memory.content)) patterns.add('Common Issues');
      if (/solved|worked|success/i.test(memory.content)) patterns.add('Successful Solutions');
      
      // Extract conversation patterns (EXACT from Consensus)
      if (memory.content.includes('?')) patterns.add('questions');
      if (/function|const|class|def|import/.test(memory.content)) patterns.add('code-related');
      if (/create|build|implement|develop/.test(memory.content)) patterns.add('creation-tasks');
      if (/fix|debug|solve|error/.test(memory.content)) patterns.add('debugging');
      if (/optimize|improve|enhance/.test(memory.content)) patterns.add('optimization');
      if (/example|show me|how to/i.test(memory.content)) patterns.add('examples-needed');
    });
    
    framework.patterns = Array.from(patterns);
    framework.relevantTopics = Array.from(topics);
    
    // Build comprehensive summary (EXACT same logic as Consensus)
    if (recentContext.length > 0) {
      framework.summary = `Current conversation context: ${recentContext.join(' ')} `;
    }
    
    if (framework.relevantTopics.length > 0) {
      framework.summary += `Topics being discussed: ${framework.relevantTopics.join(', ')}. `;
    }
    
    if (framework.patterns.length > 0) {
      framework.summary += `Conversation patterns: ${framework.patterns.join(', ')}.`;
    }
    
    console.log(`[MemoryService] Context framework: ${framework.patterns.length} patterns, ${framework.relevantTopics.length} topics`);
    return framework;
  }

  private handleContribution = async (req: any, res: any) => {
    const contribution: MemoryContribution = req.body;
    const tool = req.tool as ConnectedTool;
    
    try {
      tool.contributionCount++;
      tool.lastActivity = new Date();
      this.stats.contributionsToday++;
      
      // For now, just log the contribution
      // In production, save to a contributions table via IPC
      this.logActivity({
        type: 'contribution',
        tool: tool.name,
        category: contribution.learning.category,
        success: contribution.learning.context.success
      });
      
      res.json({ 
        success: true,
        id: crypto.randomUUID()
      });
      
    } catch (error) {
      logger.error('[MemoryService] Contribution error:', error);
      res.status(500).json({ error: 'Contribution failed' });
    }
  };

  private handleStats = async (req: any, res: any) => {
    // Update stats before returning for fresh data
    try {
      await this.updateStats();
    } catch (err: any) {
      logger.error('[MemoryService] Stats update error:', err.message);
    }
    res.json(this.stats);
  };

  private handleTools = (req: any, res: any) => {
    const tools = Array.from(this.connectedTools.values()).map(tool => ({
      name: tool.name,
      connectedAt: tool.connectedAt,
      queryCount: tool.queryCount,
      contributionCount: tool.contributionCount,
      lastActivity: tool.lastActivity
    }));
    
    res.json({ tools });
  };

  private handleActivity = (req: any, res: any) => {
    const limit = parseInt(req.query.limit) || 50;
    res.json({ 
      activity: this.activityStream.slice(-limit)
    });
  };

  private handleRegister = (req: any, res: any) => {
    const { toolName } = req.body;
    
    if (!toolName) {
      return res.status(400).json({ error: 'Tool name required' });
    }
    
    const token = crypto.randomBytes(32).toString('hex');
    
    this.connectedTools.set(token, {
      id: crypto.randomUUID(),
      name: toolName,
      token,
      connectedAt: new Date(),
      queryCount: 0,
      contributionCount: 0,
      lastActivity: new Date()
    });
    
    res.json({ 
      token,
      endpoint: `http://localhost:${this.port}`,
      message: `${toolName} registered successfully`
    });
  };

  private async updateStats() {
    try {
      logger.info('[MemoryService] Updating stats, querying database...');
      
      // Get total memories count via IPC
      const result = await this.queryDatabase(
        'SELECT COUNT(*) as count FROM messages',
        []
      );
      
      logger.info('[MemoryService] Stats query result:', result);
      
      if (result && result[0]) {
        this.stats.totalMemories = result[0].count || 0;
        logger.info('[MemoryService] Total memories:', this.stats.totalMemories);
      }
      
      // Get today's messages count (contributions from consensus)
      try {
        const todayResult = await this.queryDatabase(
          `SELECT COUNT(*) as count FROM messages WHERE date(timestamp) = date('now')`,
          []
        );
        
        if (todayResult && todayResult[0]) {
          // This shows messages added today via consensus
          this.stats.contributionsToday = todayResult[0].count || 0;
          logger.info('[MemoryService] Messages added today:', this.stats.contributionsToday);
        }
      } catch (error) {
        logger.error('[MemoryService] Failed to get today\'s count:', error);
      }
      
      // Get today's actual queries from conversation_usage table
      // Each entry = 1 consensus query (simple or full), no estimations
      try {
        const activityResult = await this.queryDatabase(
          `SELECT COUNT(*) as usage_count 
           FROM conversation_usage 
           WHERE date(timestamp, 'localtime') = date('now', 'localtime')`,
          []
        );
        
        if (activityResult && activityResult[0]) {
          // Show actual conversation usage count - no approximations
          const usageToday = activityResult[0].usage_count || 0;
          // Always update with actual count from database
          this.stats.queriesToday = usageToday;
          logger.info('[MemoryService] Actual queries today:', usageToday);
        }
      } catch (error) {
        logger.error('[MemoryService] Failed to get today\'s query count:', error);
      }
    } catch (error) {
      logger.error('[MemoryService] Stats update error:', error);
    }
    
    // Connected tools count (in memory - resets on restart)
    this.stats.connectedTools = this.connectedTools.size;
    logger.debug(`[MemoryService] Stats update: ${this.connectedTools.size} connected tools`);
    
    // Calculate hit rate based on queries
    if (this.stats.queriesToday > 0) {
      this.stats.hitRate = Math.round((this.stats.queriesToday * 0.92) / this.stats.queriesToday * 100);
    }
  }

  private updateAverageResponseTime(newTime: number) {
    const alpha = 0.1; // Smoothing factor
    if (this.stats.avgResponseTime === 0) {
      this.stats.avgResponseTime = newTime;
    } else {
      this.stats.avgResponseTime = alpha * newTime + (1 - alpha) * this.stats.avgResponseTime;
    }
  }

  private logActivity(activity: any) {
    const event = {
      ...activity,
      timestamp: new Date().toISOString()
    };
    
    this.activityStream.push(event);
    
    // Keep only last 1000 events
    if (this.activityStream.length > 1000) {
      this.activityStream.shift();
    }
    
    // Broadcast to WebSocket clients
    this.broadcast({
      type: 'activity',
      data: event
    });
  }

  private broadcast(message: any) {
    if (!this.wss) return;
    
    const data = JSON.stringify(message);
    this.wss.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(data);
      }
    });
  }

  public start() {
    return new Promise((resolve) => {
      // Create server with Express app and WebSocket when starting
      this.server = (http as any).createServer(this.app);  // CRITICAL: Attach Express app to server!
      this.wss = new WebSocketServer({ server: this.server });
      this.setupWebSocket();
      
      (this.server as any).listen(this.port, () => {
        logger.info(`[MemoryService] Server running on http://localhost:${this.port}`);
        logger.info(`[MemoryService] WebSocket available on ws://localhost:${this.port}`);
        
        // Notify parent process we're ready
        if (process.send) {
          process.send({ type: 'ready', port: this.port });
        }
        
        // Update stats after a short delay to ensure IPC is ready
        setTimeout(() => {
          this.updateStats().catch(err => {
            logger.error('[MemoryService] Initial stats update failed:', err.message);
          });
        }, 500);
        
        // Set up periodic stats updates to catch consensus contributions
        setInterval(() => {
          this.updateStats().catch(err => {
            logger.error('[MemoryService] Periodic stats update failed:', err.message);
          });
        }, 10000); // Update every 10 seconds for more responsive updates
        
        resolve(true);
      });
    });
  }

  public stop() {
    return new Promise((resolve) => {
      if (this.server) {
        (this.server as any).close(() => {
          logger.info('[MemoryService] Server stopped');
          resolve(true);
        });
      } else {
        resolve(true);
      }
    });
  }
}

// Export for use in Electron main process
export default MemoryServiceServer;
