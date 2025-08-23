"use strict";
/**
 * AI Tools Database Service
 * Manages tracking of AI tool launches per repository for intelligent resume detection
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.AIToolsDatabase = void 0;
const better_sqlite3_1 = __importDefault(require("better-sqlite3"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const SafeLogger_1 = require("../utils/SafeLogger");
class AIToolsDatabase {
    constructor() {
        // Use the shared database at ~/.hive/hive-ai.db
        const dbPath = path.join(os.homedir(), '.hive', 'hive-ai.db');
        this.db = new better_sqlite3_1.default(dbPath);
        this.initializeSchema();
    }
    /**
     * Get singleton instance
     */
    static getInstance() {
        if (!AIToolsDatabase.instance) {
            AIToolsDatabase.instance = new AIToolsDatabase();
        }
        return AIToolsDatabase.instance;
    }
    /**
     * Initialize the AI tools tracking schema
     */
    initializeSchema() {
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
            SafeLogger_1.logger.info('[AIToolsDatabase] Schema initialized successfully');
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Failed to initialize schema:', error);
            throw error;
        }
    }
    /**
     * Check if a tool has been launched in a specific repository before
     */
    hasBeenLaunchedBefore(toolId, repositoryPath) {
        try {
            const stmt = this.db.prepare(`
        SELECT COUNT(*) as count 
        FROM ai_tool_launches 
        WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
      `);
            const result = stmt.get(toolId, repositoryPath);
            return result.count > 0;
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error checking launch history:', error);
            return false;
        }
    }
    /**
     * Get launch information for a tool in a specific repository
     */
    getLaunchInfo(toolId, repositoryPath) {
        try {
            const stmt = this.db.prepare(`
        SELECT * FROM ai_tool_launches 
        WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
      `);
            const result = stmt.get(toolId, repositoryPath);
            return result || null;
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error getting launch info:', error);
            return null;
        }
    }
    /**
     * Record a new tool launch or update existing launch record
     */
    recordLaunch(toolId, repositoryPath, metadata) {
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
                stmt.run(metadata === null || metadata === void 0 ? void 0 : metadata.version, (metadata === null || metadata === void 0 ? void 0 : metadata.context) ? JSON.stringify(metadata.context) : null, (metadata === null || metadata === void 0 ? void 0 : metadata.sessionData) ? JSON.stringify(metadata.sessionData) : null, toolId, repositoryPath);
                SafeLogger_1.logger.info(`[AIToolsDatabase] Updated launch record for ${toolId} in ${repositoryPath}`);
            }
            else {
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
                stmt.run(toolId, repositoryPath, metadata === null || metadata === void 0 ? void 0 : metadata.version, (metadata === null || metadata === void 0 ? void 0 : metadata.context) ? JSON.stringify(metadata.context) : null, (metadata === null || metadata === void 0 ? void 0 : metadata.sessionData) ? JSON.stringify(metadata.sessionData) : null);
                SafeLogger_1.logger.info(`[AIToolsDatabase] Created new launch record for ${toolId} in ${repositoryPath}`);
            }
            return true;
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error recording launch:', error);
            return false;
        }
    }
    /**
     * Mark a tool session as closed
     */
    closeSession(toolId, repositoryPath) {
        try {
            const stmt = this.db.prepare(`
        UPDATE ai_tool_launches 
        SET status = 'closed'
        WHERE tool_id = ? AND repository_path = ? AND user_id = 'default'
      `);
            stmt.run(toolId, repositoryPath);
            SafeLogger_1.logger.info(`[AIToolsDatabase] Closed session for ${toolId} in ${repositoryPath}`);
            return true;
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error closing session:', error);
            return false;
        }
    }
    /**
     * Get all launch records for a specific repository
     */
    getRepositoryLaunches(repositoryPath) {
        try {
            const stmt = this.db.prepare(`
        SELECT * FROM ai_tool_launches 
        WHERE repository_path = ? 
        ORDER BY last_launched_at DESC
      `);
            return stmt.all(repositoryPath);
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error getting repository launches:', error);
            return [];
        }
    }
    /**
     * Get recent launches across all repositories
     */
    getRecentLaunches(limit = 10) {
        try {
            const stmt = this.db.prepare(`
        SELECT * FROM ai_tool_launches 
        ORDER BY last_launched_at DESC 
        LIMIT ?
      `);
            return stmt.all(limit);
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error getting recent launches:', error);
            return [];
        }
    }
    /**
     * Get statistics about tool usage
     */
    getUsageStats() {
        var _a, _b, _c, _d;
        try {
            const totalStmt = this.db.prepare(`
        SELECT SUM(launch_count) as total FROM ai_tool_launches
      `);
            const total = ((_a = totalStmt.get()) === null || _a === void 0 ? void 0 : _a.total) || 0;
            const reposStmt = this.db.prepare(`
        SELECT COUNT(DISTINCT repository_path) as count FROM ai_tool_launches
      `);
            const repos = ((_b = reposStmt.get()) === null || _b === void 0 ? void 0 : _b.count) || 0;
            const mostUsedStmt = this.db.prepare(`
        SELECT tool_id, SUM(launch_count) as total 
        FROM ai_tool_launches 
        GROUP BY tool_id 
        ORDER BY total DESC 
        LIMIT 1
      `);
            const mostUsed = ((_c = mostUsedStmt.get()) === null || _c === void 0 ? void 0 : _c.tool_id) || null;
            const activeStmt = this.db.prepare(`
        SELECT COUNT(*) as count FROM ai_tool_launches WHERE status = 'active'
      `);
            const active = ((_d = activeStmt.get()) === null || _d === void 0 ? void 0 : _d.count) || 0;
            return {
                totalLaunches: total,
                uniqueRepositories: repos,
                mostUsedTool: mostUsed,
                activeSessions: active
            };
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error getting usage stats:', error);
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
    cleanupOldRecords(daysOld = 90) {
        try {
            const stmt = this.db.prepare(`
        DELETE FROM ai_tool_launches 
        WHERE 
          status = 'closed' AND 
          julianday('now') - julianday(last_launched_at) > ?
      `);
            const result = stmt.run(daysOld);
            SafeLogger_1.logger.info(`[AIToolsDatabase] Cleaned up ${result.changes} old records`);
            return result.changes;
        }
        catch (error) {
            SafeLogger_1.logger.error('[AIToolsDatabase] Error cleaning up old records:', error);
            return 0;
        }
    }
}
exports.AIToolsDatabase = AIToolsDatabase;
AIToolsDatabase.instance = null;
//# sourceMappingURL=AIToolsDatabase.js.map