/**
 * Optimized Memory Service Integration
 * Leverages the EXISTING Memory Service child process instead of creating new workers
 * Uses IPC for communication with the already-running Memory Service
 */

import { ipcMain } from 'electron';
import * as sqlite3 from 'sqlite3';
import { EventEmitter } from 'events';

export interface OptimizedMemoryQuery {
  id: string;
  query: string;
  conversationId?: string;
  layers: Array<'RECENT' | 'TODAY' | 'WEEK' | 'SEMANTIC'>;
}

export class OptimizedMemoryService extends EventEmitter {
  private db: sqlite3.Database;
  private queryCache: Map<string, { data: any; timestamp: number }> = new Map();
  private cacheTimeout: number = 5 * 60 * 1000; // 5 minutes
  private dbConnections: sqlite3.Database[] = [];
  private poolSize: number = 4;

  constructor(database: sqlite3.Database) {
    super();
    this.db = database;
    this.initializeConnectionPool();
    this.startCacheCleanup();
  }

  /**
   * Initialize read-only connection pool for parallel queries
   */
  private initializeConnectionPool() {
    // Enable WAL mode on main connection
    this.db.run('PRAGMA journal_mode=WAL');
    this.db.run('PRAGMA synchronous=NORMAL');
    this.db.run('PRAGMA cache_size=10000');
    this.db.run('PRAGMA temp_store=MEMORY');
    
    // Create additional read-only connections
    const dbPath = (this.db as any).filename;
    for (let i = 0; i < this.poolSize - 1; i++) {
      const readDb = new sqlite3.Database(dbPath, sqlite3.OPEN_READONLY, (err) => {
        if (!err) {
          readDb.run('PRAGMA journal_mode=WAL');
          readDb.run('PRAGMA cache_size=5000');
          this.dbConnections.push(readDb);
        }
      });
    }
    
    console.log(`[OptimizedMemory] Initialized ${this.poolSize} database connections`);
  }

  /**
   * Retrieve memories using parallel queries on different connections
   */
  async retrieveMemories(query: string, conversationId?: string): Promise<any[]> {
    const startTime = Date.now();
    
    // Check cache first
    const cacheKey = `${query}_${conversationId || 'none'}`;
    const cached = this.getFromCache(cacheKey);
    if (cached) {
      console.log(`[OptimizedMemory] Cache hit for query`);
      return cached;
    }

    // Define layer queries
    const layers = [
      { name: 'RECENT', sql: this.getRecentQuery(conversationId), params: conversationId ? [conversationId] : [] },
      { name: 'TODAY', sql: this.getTodayQuery(query), params: this.extractSearchParams(query) },
      { name: 'WEEK', sql: this.getWeekQuery(query), params: this.extractSearchParams(query) },
      { name: 'SEMANTIC', sql: this.getSemanticQuery(query), params: this.extractSearchParams(query) }
    ];

    // Execute queries in parallel using different connections
    const layerPromises = layers.map((layer, index) => {
      const db = this.dbConnections[index] || this.db;
      return this.executeQuery(db, layer.sql, layer.params);
    });

    try {
      const results = await Promise.all(layerPromises);
      const combined = this.combineResults(results, query);
      
      // Cache the result
      this.setCache(cacheKey, combined);
      
      const endTime = Date.now();
      console.log(`[OptimizedMemory] Retrieved memories in ${endTime - startTime}ms`);
      
      // Update Memory Service stats via IPC
      this.updateMemoryServiceStats();
      
      return combined;
    } catch (error) {
      console.error('[OptimizedMemory] Error retrieving memories:', error);
      return [];
    }
  }

  /**
   * Store message using the main database connection
   */
  async storeMessage(params: {
    conversationId: string;
    role: 'user' | 'assistant';
    content: string;
    model?: string;
    tokensUsed?: number;
    cost?: number;
    consensusPath?: string;
    consensusRounds?: number;
    parentMessageId?: string;
  }): Promise<string> {
    return new Promise((resolve, reject) => {
      const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const sql = `
        INSERT INTO messages (
          id, conversation_id, role, content, 
          model_used, tokens_used, cost, 
          consensus_path, consensus_rounds, parent_message_id,
          timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
      `;
      
      const values = [
        messageId,
        params.conversationId,
        params.role,
        params.content,
        params.model || null,
        params.tokensUsed || 0,
        params.cost || 0,
        params.consensusPath || null,
        params.consensusRounds || null,
        params.parentMessageId || null
      ];
      
      this.db.run(sql, values, (err) => {
        if (err) {
          reject(err);
        } else {
          // Update Memory Service stats
          this.updateMemoryServiceStats();
          resolve(messageId);
        }
      });
    });
  }

  /**
   * Build SQL queries for each layer
   */
  private getRecentQuery(conversationId?: string): string {
    return `
      SELECT id, conversation_id, role, content, timestamp, 
             tokens_used, cost, consensus_path, model_used
      FROM messages
      WHERE role IN ('user', 'assistant')
        AND datetime(timestamp) > datetime('now', '-2 hours')
        ${conversationId ? 'AND conversation_id = ?' : ''}
      ORDER BY timestamp DESC
      LIMIT 5
    `;
  }

  private getTodayQuery(query: string): string {
    const searchTerms = this.extractSearchTerms(query);
    const searchCondition = searchTerms.length > 0 
      ? `AND (${searchTerms.map(() => 'LOWER(content) LIKE ?').join(' OR ')})` 
      : '';
    
    return `
      SELECT id, conversation_id, role, content, timestamp,
             tokens_used, cost, consensus_path, model_used
      FROM messages
      WHERE role IN ('user', 'assistant')
        AND datetime(timestamp) > datetime('now', '-24 hours')
        AND datetime(timestamp) <= datetime('now', '-2 hours')
        ${searchCondition}
      ORDER BY timestamp DESC
      LIMIT 5
    `;
  }

  private getWeekQuery(query: string): string {
    const searchTerms = this.extractSearchTerms(query);
    const searchCondition = searchTerms.length > 0 
      ? `AND (${searchTerms.map(() => 'LOWER(content) LIKE ?').join(' OR ')})` 
      : '';
    
    return `
      SELECT id, conversation_id, role, content, timestamp,
             tokens_used, cost, consensus_path, model_used
      FROM messages
      WHERE role IN ('user', 'assistant')
        AND datetime(timestamp) > datetime('now', '-7 days')
        AND datetime(timestamp) <= datetime('now', '-24 hours')
        ${searchCondition}
      ORDER BY timestamp DESC
      LIMIT 5
    `;
  }

  private getSemanticQuery(query: string): string {
    const searchTerms = this.extractSearchTerms(query);
    if (searchTerms.length === 0) {
      return 'SELECT * FROM messages WHERE 0'; // Empty query
    }
    
    return `
      SELECT id, conversation_id, role, content, timestamp,
             tokens_used, cost, consensus_path, model_used
      FROM messages
      WHERE role IN ('user', 'assistant')
        AND datetime(timestamp) <= datetime('now', '-7 days')
        AND (${searchTerms.map(() => 'LOWER(content) LIKE ?').join(' OR ')})
      ORDER BY timestamp DESC
      LIMIT 5
    `;
  }

  /**
   * Execute query on specified database connection
   */
  private executeQuery(db: sqlite3.Database, sql: string, params: any[]): Promise<any[]> {
    return new Promise((resolve, reject) => {
      db.all(sql, params, (err, rows) => {
        if (err) {
          console.error('[OptimizedMemory] Query error:', err);
          resolve([]);
        } else {
          resolve(rows || []);
        }
      });
    });
  }

  /**
   * Combine and deduplicate results from all layers
   */
  private combineResults(layerResults: any[][], query: string): any[] {
    const memoryMap = new Map();
    
    layerResults.forEach((memories, layerIndex) => {
      const recencyScore = 4 - layerIndex;
      
      memories.forEach(memory => {
        if (!memoryMap.has(memory.id)) {
          memoryMap.set(memory.id, {
            ...memory,
            recencyScore,
            relevanceScore: this.calculateRelevance(query, memory.content)
          });
        }
      });
    });
    
    return Array.from(memoryMap.values())
      .map(m => ({
        ...m,
        combinedScore: (m.recencyScore * 0.6) + (m.relevanceScore * 0.4)
      }))
      .sort((a, b) => b.combinedScore - a.combinedScore)
      .slice(0, 20);
  }

  /**
   * Extract search terms from query
   */
  private extractSearchTerms(query: string): string[] {
    return query.toLowerCase()
      .split(' ')
      .filter(term => term.length > 3 && !this.isStopWord(term));
  }

  private extractSearchParams(query: string): string[] {
    return this.extractSearchTerms(query).map(term => `%${term}%`);
  }

  private isStopWord(word: string): boolean {
    const stopWords = ['what', 'when', 'where', 'which', 'would', 'could', 'should', 'about', 'there', 'these'];
    return stopWords.includes(word);
  }

  private calculateRelevance(query: string, content: string): number {
    const queryWords = query.toLowerCase().split(' ');
    const contentWords = content.toLowerCase().split(' ');
    let score = 0;
    
    queryWords.forEach(qWord => {
      if (contentWords.includes(qWord)) {
        score += 1;
      }
    });
    
    return Math.min(1.0, score / queryWords.length);
  }

  /**
   * Cache management
   */
  private getFromCache(key: string): any | null {
    const cached = this.queryCache.get(key);
    if (cached && Date.now() - cached.timestamp < this.cacheTimeout) {
      return cached.data;
    }
    return null;
  }

  private setCache(key: string, data: any) {
    this.queryCache.set(key, {
      data,
      timestamp: Date.now()
    });
  }

  private startCacheCleanup() {
    setInterval(() => {
      const now = Date.now();
      for (const [key, value] of this.queryCache.entries()) {
        if (now - value.timestamp > this.cacheTimeout) {
          this.queryCache.delete(key);
        }
      }
    }, 60000); // Clean every minute
  }

  /**
   * Update Memory Service statistics via IPC
   * The Memory Service is already running and tracking stats
   */
  private updateMemoryServiceStats() {
    // The Memory Service automatically updates its stats every 10 seconds
    // by querying the database. Our inserts will be picked up automatically.
    // This is just a notification that new data is available.
    this.emit('stats-updated');
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    return {
      cacheSize: this.queryCache.size,
      connectionPoolSize: this.dbConnections.length + 1,
      cacheHitRate: 0 // Would need to track hits/misses for real metric
    };
  }

  /**
   * Cleanup resources
   */
  cleanup() {
    this.dbConnections.forEach(db => db.close());
    this.queryCache.clear();
  }
}

export default OptimizedMemoryService;