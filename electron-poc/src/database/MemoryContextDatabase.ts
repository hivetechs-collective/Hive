import * as sqlite3 from 'sqlite3';

export interface Memory {
  id: string;
  conversation_id: string;
  role: string;
  content: string;
  timestamp: string;
  tokens_used?: number;
  cost?: number;
  consensus_path?: string;
  recency_score: number;
  relevance_score: number;
}

export interface ContextFramework {
  summary: string;
  patterns: string[];
  relevantTopics: string[];
  userPreferences: string[];
}

export interface MemoryContextLog {
  log_id: string;
  request_id: string;
  conversation_id?: string;
  memories_retrieved: {
    recent: number;
    today: number;
    week: number;
    semantic: number;
  };
  context_summary?: string;
  patterns_identified?: string[];
  topics_extracted?: string[];
  routing_decision?: string;
  performance_ms: {
    memory: number;
    context: number;
  };
}

export class MemoryContextDatabase {
  private db: sqlite3.Database;

  constructor(db: sqlite3.Database) {
    this.db = db;
  }

  /**
   * Retrieve memories using layered approach:
   * 1. Most recent (2 hours) - for ongoing conversations
   * 2. Today (24 hours) - for recent context  
   * 3. This week (7 days) - for patterns
   * 4. Semantic search - for thematic relevance
   */
  async retrieveLayeredMemories(query: string, conversationId?: string): Promise<Memory[]> {
    return new Promise((resolve, reject) => {
      const memories: Memory[] = [];
      const searchTerms = this.extractSearchTerms(query);
      
      // Layer 1: Most Recent (Last 2 hours) - Highest priority for conversation continuity
      const recentSql = `
        SELECT 
          m.*,
          4 as recency_score,
          1.0 as relevance_score
        FROM messages m
        WHERE m.role IN ('user', 'assistant')
          AND datetime(m.timestamp) > datetime('now', '-2 hours')
          ${conversationId ? 'AND m.conversation_id = ?' : ''}
        ORDER BY m.timestamp DESC
        LIMIT 5
      `;
      
      // Layer 2: Today (Last 24 hours) - Recent context
      const todaySql = `
        SELECT 
          m.*,
          3 as recency_score,
          0.8 as relevance_score
        FROM messages m
        WHERE m.role IN ('user', 'assistant')
          AND datetime(m.timestamp) > datetime('now', '-24 hours')
          AND datetime(m.timestamp) <= datetime('now', '-2 hours')
          ${searchTerms.length > 0 ? `AND (${searchTerms.map(() => 'LOWER(m.content) LIKE ?').join(' OR ')})` : ''}
        ORDER BY m.timestamp DESC
        LIMIT 5
      `;
      
      // Layer 3: This Week (Last 7 days) - Patterns
      const weekSql = `
        SELECT 
          m.*,
          2 as recency_score,
          0.6 as relevance_score
        FROM messages m
        WHERE m.role IN ('user', 'assistant')
          AND datetime(m.timestamp) > datetime('now', '-7 days')
          AND datetime(m.timestamp) <= datetime('now', '-24 hours')
          ${searchTerms.length > 0 ? `AND (${searchTerms.map(() => 'LOWER(m.content) LIKE ?').join(' OR ')})` : ''}
        ORDER BY m.timestamp DESC
        LIMIT 5
      `;
      
      // Layer 4: Semantic Search (All time) - Thematic relevance
      const semanticSql = `
        SELECT 
          m.*,
          1 as recency_score,
          0.4 as relevance_score
        FROM messages m
        WHERE m.role IN ('user', 'assistant')
          AND datetime(m.timestamp) <= datetime('now', '-7 days')
          ${searchTerms.length > 0 ? `AND (${searchTerms.map(() => 'LOWER(m.content) LIKE ?').join(' OR ')})` : ''}
        ORDER BY m.timestamp DESC
        LIMIT 5
      `;

      // Execute all layers in parallel
      let completedLayers = 0;
      const totalLayers = 4;
      const allMemories: Map<string, Memory> = new Map();
      
      const checkComplete = () => {
        completedLayers++;
        if (completedLayers === totalLayers) {
          // Sort by combined score (recency * 0.6 + relevance * 0.4)
          const sortedMemories = Array.from(allMemories.values())
            .map(m => ({
              ...m,
              combinedScore: (m.recency_score * 0.6) + (m.relevance_score * 0.4)
            }))
            .sort((a, b) => b.combinedScore - a.combinedScore)
            .slice(0, 20); // Return top 20
          
          resolve(sortedMemories);
        }
      };

      // Layer 1: Recent
      const recentParams = conversationId ? [conversationId] : [];
      this.db.all(recentSql, recentParams, (err, rows: any[]) => {
        if (!err && rows) {
          rows.forEach(row => {
            if (!allMemories.has(row.id)) {
              allMemories.set(row.id, { ...row, relevance_score: 1.0 });
            }
          });
        }
        checkComplete();
      });

      // Layer 2: Today
      const todayParams = searchTerms.map(term => `%${term}%`);
      this.db.all(todaySql, todayParams, (err, rows: any[]) => {
        if (!err && rows) {
          rows.forEach(row => {
            if (!allMemories.has(row.id)) {
              const relevance = this.calculateRelevance(query, row.content);
              allMemories.set(row.id, { ...row, relevance_score: relevance * 0.8 });
            }
          });
        }
        checkComplete();
      });

      // Layer 3: Week
      const weekParams = searchTerms.map(term => `%${term}%`);
      this.db.all(weekSql, weekParams, (err, rows: any[]) => {
        if (!err && rows) {
          rows.forEach(row => {
            if (!allMemories.has(row.id)) {
              const relevance = this.calculateRelevance(query, row.content);
              allMemories.set(row.id, { ...row, relevance_score: relevance * 0.6 });
            }
          });
        }
        checkComplete();
      });

      // Layer 4: Semantic
      const semanticParams = searchTerms.map(term => `%${term}%`);
      this.db.all(semanticSql, semanticParams, (err, rows: any[]) => {
        if (!err && rows) {
          rows.forEach(row => {
            if (!allMemories.has(row.id)) {
              const relevance = this.calculateRelevance(query, row.content);
              allMemories.set(row.id, { ...row, relevance_score: relevance * 0.4 });
            }
          });
        }
        checkComplete();
      });
    });
  }

  /**
   * Store the consensus response as a message for future retrieval
   * This also updates Memory Service statistics
   */
  async storeConsensusResponse(
    conversationId: string,
    content: string,
    model: string,
    consensusPath: 'SIMPLE' | 'COMPLEX',
    consensusRounds: number,
    tokensUsed: number,
    cost: number,
    parentMessageId?: string
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const sql = `
        INSERT INTO messages (
          id, conversation_id, role, content, stage, model_used,
          tokens_used, cost, consensus_path, consensus_rounds,
          parent_message_id, timestamp
        ) VALUES (?, ?, 'assistant', ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
      `;
      
      const params = [
        messageId,
        conversationId,
        content,
        consensusPath === 'SIMPLE' ? 'generator' : 'curator',
        model,
        tokensUsed,
        cost,
        consensusPath,
        consensusRounds,
        parentMessageId
      ];
      
      this.db.run(sql, params, (err) => {
        if (err) {
          console.error('❌ Error storing consensus response:', err);
          reject(err);
        } else {
          console.log(`✅ Stored consensus response: ${messageId}`);
          resolve(messageId);
        }
      });
    });
  }

  /**
   * Log Memory and Context stage operations
   */
  async logMemoryContextOperation(log: MemoryContextLog): Promise<void> {
    return new Promise((resolve, reject) => {
      const sql = `
        INSERT INTO memory_context_logs (
          log_id, request_id, conversation_id,
          memories_retrieved_recent, memories_retrieved_today,
          memories_retrieved_week, memories_retrieved_semantic,
          context_summary, patterns_identified, topics_extracted,
          routing_decision, memory_stage_duration_ms, context_stage_duration_ms
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `;
      
      const params = [
        log.log_id,
        log.request_id,
        log.conversation_id,
        log.memories_retrieved.recent,
        log.memories_retrieved.today,
        log.memories_retrieved.week,
        log.memories_retrieved.semantic,
        log.context_summary,
        JSON.stringify(log.patterns_identified),
        JSON.stringify(log.topics_extracted),
        log.routing_decision,
        log.performance_ms.memory,
        log.performance_ms.context
      ];
      
      this.db.run(sql, params, (err) => {
        if (err) {
          console.error('❌ Error logging memory-context operation:', err);
          reject(err);
        } else {
          resolve();
        }
      });
    });
  }

  /**
   * Store user question as a message
   */
  async storeUserMessage(
    conversationId: string,
    content: string
  ): Promise<string> {
    return new Promise((resolve, reject) => {
      const messageId = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      
      const sql = `
        INSERT INTO messages (
          id, conversation_id, role, content, timestamp
        ) VALUES (?, ?, 'user', ?, datetime('now'))
      `;
      
      this.db.run(sql, [messageId, conversationId, content], (err) => {
        if (err) {
          console.error('❌ Error storing user message:', err);
          reject(err);
        } else {
          console.log(`✅ Stored user message: ${messageId}`);
          resolve(messageId);
        }
      });
    });
  }

  private extractSearchTerms(query: string): string[] {
    return query.toLowerCase()
      .split(' ')
      .filter(term => term.length > 3 && !['what', 'when', 'where', 'which', 'would', 'could', 'should', 'about'].includes(term));
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
   * Get Memory Service statistics for display
   */
  async getMemoryStats(): Promise<{
    totalMemories: number;
    queriesToday: number;
    contributionsToday: number;
  }> {
    return new Promise((resolve, reject) => {
      const stats = {
        totalMemories: 0,
        queriesToday: 0,
        contributionsToday: 0
      };

      // Count total memories
      this.db.get(
        "SELECT COUNT(*) as count FROM messages WHERE role IN ('user', 'assistant')",
        (err: any, row: any) => {
          if (!err && row) {
            stats.totalMemories = row.count;
          }
          
          // Count queries today (user messages)
          this.db.get(
            "SELECT COUNT(*) as count FROM messages WHERE role = 'user' AND date(timestamp) = date('now')",
            (err2: any, row2: any) => {
              if (!err2 && row2) {
                stats.queriesToday = row2.count;
              }
              
              // Count contributions today (assistant responses)
              this.db.get(
                "SELECT COUNT(*) as count FROM messages WHERE role = 'assistant' AND date(timestamp) = date('now')",
                (err3: any, row3: any) => {
                  if (!err3 && row3) {
                    stats.contributionsToday = row3.count;
                  }
                  
                  console.log('📊 Memory Service Stats:', stats);
                  resolve(stats);
                }
              );
            }
          );
        }
      );
    });
  }
}