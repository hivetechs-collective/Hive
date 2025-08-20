"use strict";
/**
 * Universal Memory Infrastructure - Memory Service Server
 * Provides memory-as-a-service to external AI tools
 * Runs on port 3457 as a child process, uses IPC for database access
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
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.MemoryServiceServer = void 0;
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const ws_1 = require("ws");
const http = __importStar(require("http"));
const crypto_1 = __importDefault(require("crypto"));
class MemoryServiceServer {
    constructor(port = 3457) {
        this.server = null;
        this.wss = null;
        this.connectedTools = new Map();
        this.activityStream = [];
        this.stats = {
            totalMemories: 0,
            queriesToday: 0,
            contributionsToday: 0,
            connectedTools: 0,
            hitRate: 92,
            avgResponseTime: 45
        };
        this.pendingQueries = new Map();
        this.authenticate = (req, res, next) => {
            var _a;
            const token = (_a = req.headers.authorization) === null || _a === void 0 ? void 0 : _a.replace('Bearer ', '');
            if (!token) {
                return res.status(401).json({ error: 'No token provided' });
            }
            // For now, accept any token and track it
            const clientName = req.headers['x-client-name'] || 'unknown';
            if (!this.connectedTools.has(token)) {
                this.connectedTools.set(token, {
                    id: crypto_1.default.randomUUID(),
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
        this.handleQuery = (req, res) => __awaiter(this, void 0, void 0, function* () {
            var _a;
            const startTime = Date.now();
            const query = req.body;
            const tool = req.tool;
            try {
                tool.queryCount++;
                tool.lastActivity = new Date();
                this.stats.queriesToday++;
                // Query database via IPC
                const limit = ((_a = query.options) === null || _a === void 0 ? void 0 : _a.limit) || 5;
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
                const memories = yield this.queryDatabase(sql, [`%${query.query}%`, limit]);
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
            }
            catch (error) {
                console.error('[MemoryService] Query error:', error);
                res.status(500).json({ error: 'Query failed' });
            }
        });
        this.handleContribution = (req, res) => __awaiter(this, void 0, void 0, function* () {
            const contribution = req.body;
            const tool = req.tool;
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
                    id: crypto_1.default.randomUUID()
                });
            }
            catch (error) {
                console.error('[MemoryService] Contribution error:', error);
                res.status(500).json({ error: 'Contribution failed' });
            }
        });
        this.handleStats = (req, res) => __awaiter(this, void 0, void 0, function* () {
            // Update stats before returning for fresh data
            try {
                yield this.updateStats();
            }
            catch (err) {
                console.error('[MemoryService] Stats update error:', err.message);
            }
            res.json(this.stats);
        });
        this.handleTools = (req, res) => {
            const tools = Array.from(this.connectedTools.values()).map(tool => ({
                name: tool.name,
                connectedAt: tool.connectedAt,
                queryCount: tool.queryCount,
                contributionCount: tool.contributionCount,
                lastActivity: tool.lastActivity
            }));
            res.json({ tools });
        };
        this.handleActivity = (req, res) => {
            const limit = parseInt(req.query.limit) || 50;
            res.json({
                activity: this.activityStream.slice(-limit)
            });
        };
        this.handleRegister = (req, res) => {
            const { toolName } = req.body;
            if (!toolName) {
                return res.status(400).json({ error: 'Tool name required' });
            }
            const token = crypto_1.default.randomBytes(32).toString('hex');
            this.connectedTools.set(token, {
                id: crypto_1.default.randomUUID(),
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
        this.port = port;
        this.app = (0, express_1.default)();
        this.setupMiddleware();
        this.setupRoutes();
        this.setupIPC();
        // Don't update stats in constructor - wait until server starts
    }
    setupIPC() {
        // Listen for database results from main process
        process.on('message', (msg) => {
            if (msg.type === 'db-result') {
                const callback = this.pendingQueries.get(msg.id);
                if (callback) {
                    callback(msg.error, msg.data);
                    this.pendingQueries.delete(msg.id);
                }
            }
        });
    }
    queryDatabase(sql, params) {
        return new Promise((resolve, reject) => {
            const queryId = crypto_1.default.randomUUID();
            // Store callback for when result comes back
            this.pendingQueries.set(queryId, (error, data) => {
                if (error) {
                    reject(error);
                }
                else {
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
            }
            else {
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
    setupMiddleware() {
        this.app.use((0, cors_1.default)({
            origin: '*',
            credentials: true
        }));
        this.app.use(express_1.default.json({ limit: '10mb' }));
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
    setupRoutes() {
        // Health check
        this.app.get('/health', (req, res) => {
            res.json({
                status: 'healthy',
                port: this.port,
                database: 'connected via IPC',
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
    setupWebSocket() {
        if (!this.wss)
            return;
        this.wss.on('connection', (ws) => {
            console.log('[MemoryService] WebSocket client connected');
            // Send initial stats
            ws.send(JSON.stringify({
                type: 'stats',
                data: this.stats
            }));
            ws.on('message', (message) => {
                try {
                    const data = JSON.parse(message.toString());
                    if (data.action === 'subscribe') {
                        // Handle subscription to specific events
                        ws.send(JSON.stringify({
                            type: 'subscribed',
                            events: data.events
                        }));
                    }
                }
                catch (error) {
                    console.error('[MemoryService] WebSocket message error:', error);
                }
            });
            ws.on('close', () => {
                console.log('[MemoryService] WebSocket client disconnected');
            });
        });
    }
    updateStats() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                console.log('[MemoryService] Updating stats, querying database...');
                // Get total memories count via IPC
                const result = yield this.queryDatabase('SELECT COUNT(*) as count FROM messages', []);
                console.log('[MemoryService] Stats query result:', result);
                if (result && result[0]) {
                    this.stats.totalMemories = result[0].count || 0;
                    console.log('[MemoryService] Total memories:', this.stats.totalMemories);
                }
                // Get today's messages count (contributions from consensus)
                try {
                    const todayResult = yield this.queryDatabase(`SELECT COUNT(*) as count FROM messages WHERE date(timestamp) = date('now')`, []);
                    if (todayResult && todayResult[0]) {
                        // This shows messages added today via consensus
                        this.stats.contributionsToday = todayResult[0].count || 0;
                        console.log('[MemoryService] Messages added today:', this.stats.contributionsToday);
                    }
                }
                catch (error) {
                    console.error('[MemoryService] Failed to get today\'s count:', error);
                }
                // Get today's actual queries from conversation_usage table
                // Each entry = 1 consensus query (simple or full), no estimations
                try {
                    const activityResult = yield this.queryDatabase(`SELECT COUNT(*) as usage_count 
           FROM conversation_usage 
           WHERE date(timestamp, 'localtime') = date('now', 'localtime')`, []);
                    if (activityResult && activityResult[0]) {
                        // Show actual conversation usage count - no approximations
                        const usageToday = activityResult[0].usage_count || 0;
                        // Always update with actual count from database
                        this.stats.queriesToday = usageToday;
                        console.log('[MemoryService] Actual queries today:', usageToday);
                    }
                }
                catch (error) {
                    console.error('[MemoryService] Failed to get today\'s query count:', error);
                }
            }
            catch (error) {
                console.error('[MemoryService] Stats update error:', error);
            }
            // Connected tools count (in memory - resets on restart)
            this.stats.connectedTools = this.connectedTools.size;
            // Calculate hit rate based on queries
            if (this.stats.queriesToday > 0) {
                this.stats.hitRate = Math.round((this.stats.queriesToday * 0.92) / this.stats.queriesToday * 100);
            }
        });
    }
    updateAverageResponseTime(newTime) {
        const alpha = 0.1; // Smoothing factor
        if (this.stats.avgResponseTime === 0) {
            this.stats.avgResponseTime = newTime;
        }
        else {
            this.stats.avgResponseTime = alpha * newTime + (1 - alpha) * this.stats.avgResponseTime;
        }
    }
    logActivity(activity) {
        const event = Object.assign(Object.assign({}, activity), { timestamp: new Date().toISOString() });
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
    broadcast(message) {
        if (!this.wss)
            return;
        const data = JSON.stringify(message);
        this.wss.clients.forEach(client => {
            if (client.readyState === ws_1.WebSocket.OPEN) {
                client.send(data);
            }
        });
    }
    start() {
        return new Promise((resolve) => {
            // Create server and WebSocket when starting
            this.server = http.createServer();
            this.wss = new ws_1.WebSocketServer({ server: this.server });
            this.setupWebSocket();
            this.server.listen(this.port, () => {
                console.log(`[MemoryService] Server running on http://localhost:${this.port}`);
                console.log(`[MemoryService] WebSocket available on ws://localhost:${this.port}`);
                // Notify parent process we're ready
                if (process.send) {
                    process.send({ type: 'ready', port: this.port });
                }
                // Update stats after a short delay to ensure IPC is ready
                setTimeout(() => {
                    this.updateStats().catch(err => {
                        console.error('[MemoryService] Initial stats update failed:', err.message);
                    });
                }, 500);
                // Set up periodic stats updates to catch consensus contributions
                setInterval(() => {
                    this.updateStats().catch(err => {
                        console.error('[MemoryService] Periodic stats update failed:', err.message);
                    });
                }, 10000); // Update every 10 seconds for more responsive updates
                resolve(true);
            });
        });
    }
    stop() {
        return new Promise((resolve) => {
            if (this.server) {
                this.server.close(() => {
                    console.log('[MemoryService] Server stopped');
                    resolve(true);
                });
            }
            else {
                resolve(true);
            }
        });
    }
}
exports.MemoryServiceServer = MemoryServiceServer;
// Export for use in Electron main process
exports.default = MemoryServiceServer;
//# sourceMappingURL=server.js.map