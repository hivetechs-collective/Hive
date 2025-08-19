/**
 * Universal Memory Infrastructure - Memory Service Server
 * Provides memory-as-a-service to external AI tools
 * Runs on port 3457, separate from consensus (3456)
 */

import express from 'express';
import cors from 'cors';
import { WebSocketServer, WebSocket } from 'ws';
import Database from 'better-sqlite3';
import { createServer } from 'http';
import path from 'path';
import crypto from 'crypto';
import fs from 'fs';

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
  private server: any;
  private wss: WebSocketServer;
  private db: Database.Database | null = null;
  private connectedTools: Map<string, ConnectedTool> = new Map();
  private activityStream: any[] = [];
  private stats: MemoryStats = {
    totalMemories: 0,
    queriesToday: 0,
    contributionsToday: 0,
    connectedTools: 0,
    hitRate: 0,
    avgResponseTime: 0
  };
  private port: number;
  private dbPath: string;

  constructor(port: number = 3457, dbPath?: string) {
    this.port = port;
    this.dbPath = dbPath || path.join(
      process.env.HOME || '',
      '.hive',
      'data',
      'conversations.db'
    );
    
    this.app = express();
    this.setupMiddleware();
    this.setupDatabase();
    this.setupRoutes();
    this.server = createServer(this.app);
    this.wss = new WebSocketServer({ server: this.server });
    this.setupWebSocket();
  }

  private setupMiddleware() {
    this.app.use(cors({
      origin: ['http://localhost:*', 'file://*'],
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

  private setupDatabase() {
    try {
      // Open database in read-only mode for safety
      if (fs.existsSync(this.dbPath)) {
        this.db = new Database(this.dbPath, { 
          readonly: true,
          fileMustExist: true 
        });
        
        // Get initial stats
        this.updateStats();
        
        console.log('[MemoryService] Connected to database:', this.dbPath);
      } else {
        console.warn('[MemoryService] Database not found, running in demo mode');
      }
    } catch (error) {
      console.error('[MemoryService] Database connection failed:', error);
      // Continue without database for demo purposes
    }
  }

  private setupRoutes() {
    // Health check
    this.app.get('/health', (req, res) => {
      res.json({ 
        status: 'healthy',
        port: this.port,
        database: this.db ? 'connected' : 'disconnected',
        uptime: process.uptime()
      });
    });

    // Query memories
    this.app.post('/api/v1/memory/query', this.authenticate, this.handleQuery.bind(this));
    
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
    this.wss.on('connection', (ws: WebSocket) => {
      console.log('[MemoryService] WebSocket client connected');
      
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
          console.error('[MemoryService] WebSocket message error:', error);
        }
      });
      
      ws.on('close', () => {
        console.log('[MemoryService] WebSocket client disconnected');
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
    }
    
    req.tool = this.connectedTools.get(token);
    next();
  };

  private handleQuery = async (req: any, res: any) => {
    const startTime = Date.now();
    const query: MemoryQuery = req.body;
    const tool = req.tool as ConnectedTool;
    
    try {
      tool.queryCount++;
      tool.lastActivity = new Date();
      this.stats.queriesToday++;
      
      let memories = [];
      
      if (this.db) {
        // Query the database for relevant memories
        const limit = query.options?.limit || 5;
        
        // Simple query - in production, use vector similarity
        const stmt = this.db.prepare(`
          SELECT 
            id,
            user_message as content,
            assistant_message as response,
            timestamp,
            conversation_id
          FROM messages
          WHERE user_message LIKE ?
          ORDER BY timestamp DESC
          LIMIT ?
        `);
        
        memories = stmt.all(`%${query.query}%`, limit);
      } else {
        // Demo data when no database
        memories = [
          {
            id: 'demo_1',
            content: 'Example memory about ' + query.query,
            response: 'This is a demo response',
            confidence: 0.85,
            timestamp: Date.now()
          }
        ];
      }
      
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
      console.error('[MemoryService] Query error:', error);
      res.status(500).json({ error: 'Query failed' });
    }
  };

  private handleContribution = async (req: any, res: any) => {
    const contribution: MemoryContribution = req.body;
    const tool = req.tool as ConnectedTool;
    
    try {
      tool.contributionCount++;
      tool.lastActivity = new Date();
      this.stats.contributionsToday++;
      
      // For now, just log the contribution
      // In production, save to a contributions table
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
      console.error('[MemoryService] Contribution error:', error);
      res.status(500).json({ error: 'Contribution failed' });
    }
  };

  private handleStats = (req: any, res: any) => {
    this.updateStats();
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

  private updateStats() {
    if (this.db) {
      try {
        const totalMemories = this.db.prepare(
          'SELECT COUNT(*) as count FROM messages'
        ).get() as any;
        
        this.stats.totalMemories = totalMemories?.count || 0;
      } catch (error) {
        console.error('[MemoryService] Stats update error:', error);
      }
    }
    
    this.stats.connectedTools = this.connectedTools.size;
    
    // Calculate hit rate (mock for now)
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
    const data = JSON.stringify(message);
    this.wss.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(data);
      }
    });
  }

  public start() {
    return new Promise((resolve) => {
      this.server.listen(this.port, () => {
        console.log(`[MemoryService] Server running on http://localhost:${this.port}`);
        console.log(`[MemoryService] WebSocket available on ws://localhost:${this.port}`);
        resolve(true);
      });
    });
  }

  public stop() {
    return new Promise((resolve) => {
      this.server.close(() => {
        if (this.db) {
          this.db.close();
        }
        console.log('[MemoryService] Server stopped');
        resolve(true);
      });
    });
  }
}

// Export for use in Electron main process
export default MemoryServiceServer;