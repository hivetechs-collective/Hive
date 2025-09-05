/**
 * AI Tools Database Service
 * Manages tracking of AI tool launches per repository for intelligent resume detection
 */

import { logger } from '../utils/SafeLogger';

export interface AIToolLaunch {
  id?: number;
  tool_id: string;
  repository_path: string;
  launch_count: number;
  first_launched_at: string;
  last_launched_at: string;
  status: 'active' | 'closed' | 'crashed';
  session_metadata?: string;
  user_id: string;
  tool_version?: string;
  launch_context?: string;
}

export class AIToolsDatabase {
  private db: any; // Will be the better-sqlite3 database instance from main process
  private static instance: AIToolsDatabase | null = null;

  private constructor(database: any) {
    // Use the existing database connection from main process
    this.db = database;
    this.initializeSchema();
  }

  /**
   * Get singleton instance with database connection
   */
  public static getInstance(database?: any): AIToolsDatabase {
    if (!database && !AIToolsDatabase.instance) {
      throw new Error('AIToolsDatabase requires a database connection on first initialization');
    }
    if (!AIToolsDatabase.instance && database) {
      AIToolsDatabase.instance = new AIToolsDatabase(database);
    }
    return AIToolsDatabase.instance as AIToolsDatabase;
  }

  /**
   * Initialize the AI tools tracking schema
   */
  private initializeSchema(): void {
    try {
      // Create the main table
      this.db.exec(`
        CREATE TABLE IF NOT EXISTS ai_tool_launches (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          tool_id TEXT NOT NULL,
          repository_path TEXT NOT NULL,
          launch_count INTEGER DEFAULT 1,
          first_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
          last_launched_at TEXT DEFAULT CURRENT_TIMESTAMP,
          status TEXT DEFAULT 'active',
          session_metadata TEXT,
          user_id TEXT DEFAULT 'default',
          tool_version TEXT,
          launch_context TEXT,
          UNIQUE(tool_id, repository_path, user_id),
          FOREIGN KEY (user_id) REFERENCES users(id)
        )
      `);

      // Create indexes for performance
      this.db.exec(`
        CREATE INDEX IF NOT EXISTS idx_ai_tool_launches_lookup 
        ON ai_tool_launches(tool_id, repository_path);
        
        CREATE INDEX IF NOT EXISTS idx_ai_tool_launches_recent 
        ON ai_tool_launches(last_launched_at DESC);
        
        CREATE INDEX IF NOT EXISTS idx_ai_tool_launches_active 
        ON ai_tool_launches(status) WHERE status = 'active';
      `);

      logger.info('[AIToolsDatabase] Schema initialized successfully');
    } catch (error) {
      logger.error('[AIToolsDatabase] Failed to initialize schema:', error);
      throw error;
    }
  }

  /**
   * Check if a tool has been launched in a specific repository before
   */
  public hasBeenLaunchedBefore(toolId: string, repositoryPath: string): boolean {
    try {
      const stmt = this.db.prepare(`
        SELECT COUNT(*) as count 
        FROM ai_tool_launches 
        WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
      `);
      
      const result = stmt.get(toolId, repositoryPath) as { count: number };
      return result.count > 0;
    } catch (error) {
      logger.error('[AIToolsDatabase] Error checking launch history:', error);
      return false;
    }
  }

  /**
   * Get launch information for a tool in a specific repository
   */
  public getLaunchInfo(toolId: string, repositoryPath: string): AIToolLaunch | null {
    try {
      const stmt = this.db.prepare(`
        SELECT * FROM ai_tool_launches 
        WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
      `);
      
      const result = stmt.get(toolId, repositoryPath) as AIToolLaunch | undefined;
      return result || null;
    } catch (error) {
      logger.error('[AIToolsDatabase] Error getting launch info:', error);
      return null;
    }
  }

  /**
   * Record a new tool launch or update existing launch record
   */
  public recordLaunch(
    toolId: string, 
    repositoryPath: string,
    metadata?: {
      version?: string;
      context?: any;
      sessionData?: any;
    }
  ): boolean {
    try {
      const existing = this.getLaunchInfo(toolId, repositoryPath);
      
      if (existing) {
        // Update existing launch record
        const stmt = this.db.prepare(`
          UPDATE ai_tool_launches 
          SET 
            launch_count = launch_count + 1,
            last_launched_at = CURRENT_TIMESTAMP,
            status = 'active',
            tool_version = COALESCE(?, tool_version),
            launch_context = COALESCE(?, launch_context),
            session_metadata = COALESCE(?, session_metadata)
          WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
        `);
        
        stmt.run(
          metadata?.version,
          metadata?.context ? JSON.stringify(metadata.context) : null,
          metadata?.sessionData ? JSON.stringify(metadata.sessionData) : null,
          toolId,
          repositoryPath
        );
        
        logger.info(`[AIToolsDatabase] Updated launch record for ${toolId} in ${repositoryPath}`);
      } else {
        // Insert new launch record
        const stmt = this.db.prepare(`
          INSERT INTO ai_tool_launches (
            tool_id, 
            repository_path, 
            tool_version, 
            launch_context,
            session_metadata,
            user_id
          ) VALUES (?, ?, ?, ?, ?, 'default')
        `);
        
        stmt.run(
          toolId,
          repositoryPath,
          metadata?.version,
          metadata?.context ? JSON.stringify(metadata.context) : null,
          metadata?.sessionData ? JSON.stringify(metadata.sessionData) : null
        );
        
        logger.info(`[AIToolsDatabase] Created new launch record for ${toolId} in ${repositoryPath}`);
      }
      
      return true;
    } catch (error) {
      logger.error('[AIToolsDatabase] Error recording launch:', error);
      return false;
    }
  }

  /**
   * Mark a tool session as closed
   */
  public closeSession(toolId: string, repositoryPath: string): boolean {
    try {
      const stmt = this.db.prepare(`
        UPDATE ai_tool_launches 
        SET status = 'closed'
        WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
      `);
      
      stmt.run(toolId, repositoryPath);
      logger.info(`[AIToolsDatabase] Closed session for ${toolId} in ${repositoryPath}`);
      return true;
    } catch (error) {
      logger.error('[AIToolsDatabase] Error closing session:', error);
      return false;
    }
  }

  /**
   * Get all launch records for a specific repository
   */
  public getRepositoryLaunches(repositoryPath: string): AIToolLaunch[] {
    try {
      const stmt = this.db.prepare(`
        SELECT * FROM ai_tool_launches 
        WHERE repository_path = ? 
        ORDER BY last_launched_at DESC
      `);
      
      return stmt.all(repositoryPath) as AIToolLaunch[];
    } catch (error) {
      logger.error('[AIToolsDatabase] Error getting repository launches:', error);
      return [];
    }
  }

  /**
   * Get recent launches across all repositories
   */
  public getRecentLaunches(limit: number = 10): AIToolLaunch[] {
    try {
      const stmt = this.db.prepare(`
        SELECT * FROM ai_tool_launches 
        ORDER BY last_launched_at DESC 
        LIMIT ?
      `);
      
      return stmt.all(limit) as AIToolLaunch[];
    } catch (error) {
      logger.error('[AIToolsDatabase] Error getting recent launches:', error);
      return [];
    }
  }

  /**
   * Get statistics about tool usage
   */
  public getUsageStats(): {
    totalLaunches: number;
    uniqueRepositories: number;
    mostUsedTool: string | null;
    activeSessions: number;
  } {
    try {
      const totalStmt = this.db.prepare(`
        SELECT SUM(launch_count) as total FROM ai_tool_launches
      `);
      const total = (totalStmt.get() as { total: number })?.total || 0;

      const reposStmt = this.db.prepare(`
        SELECT COUNT(DISTINCT repository_path) as count FROM ai_tool_launches
      `);
      const repos = (reposStmt.get() as { count: number })?.count || 0;

      const mostUsedStmt = this.db.prepare(`
        SELECT tool_id, SUM(launch_count) as total 
        FROM ai_tool_launches 
        GROUP BY tool_id 
        ORDER BY total DESC 
        LIMIT 1
      `);
      const mostUsed = (mostUsedStmt.get() as { tool_id: string })?.tool_id || null;

      const activeStmt = this.db.prepare(`
        SELECT COUNT(*) as count FROM ai_tool_launches WHERE status = 'active'
      `);
      const active = (activeStmt.get() as { count: number })?.count || 0;

      return {
        totalLaunches: total,
        uniqueRepositories: repos,
        mostUsedTool: mostUsed,
        activeSessions: active
      };
    } catch (error) {
      logger.error('[AIToolsDatabase] Error getting usage stats:', error);
      return {
        totalLaunches: 0,
        uniqueRepositories: 0,
        mostUsedTool: null,
        activeSessions: 0
      };
    }
  }

  /**
   * Clean up old/stale launch records (optional maintenance)
   */
  public cleanupOldRecords(daysOld: number = 90): number {
    try {
      const stmt = this.db.prepare(`
        DELETE FROM ai_tool_launches 
        WHERE 
          status = 'closed' AND 
          julianday('now') - julianday(last_launched_at) > ?
      `);
      
      const result = stmt.run(daysOld);
      logger.info(`[AIToolsDatabase] Cleaned up ${result.changes} old records`);
      return result.changes;
    } catch (error) {
      logger.error('[AIToolsDatabase] Error cleaning up old records:', error);
      return 0;
    }
  }
}