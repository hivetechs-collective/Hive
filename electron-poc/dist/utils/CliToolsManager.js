"use strict";
/**
 * CLI Tools Manager
 * Handles installation, updates, and maintenance of AI CLI tools
 * Primary focus: Claude Code CLI integration with Memory Service
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
Object.defineProperty(exports, "__esModule", { value: true });
exports.CliToolsManager = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
// Remove node-fetch to avoid punycode deprecation warning
const events_1 = require("events");
const exec = (0, util_1.promisify)(child_process_1.exec);
class CliToolsManager extends events_1.EventEmitter {
    constructor(database) {
        super();
        this.db = database;
        this.toolsDir = path.join(os.homedir(), '.hive', 'tools');
        this.configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
        this.tools = new Map();
        this.status = new Map();
        this.updateCheckTimers = new Map();
        this.initializeTools();
        this.loadStatus();
    }
    /**
     * Initialize tool configurations
     */
    initializeTools() {
        // Only include agentic coding CLIs that work like Claude Code
        // These tools can understand codebases, make multi-file changes, and execute commands
        // Claude Code CLI - Anthropic's agentic coding assistant
        this.tools.set('claude', {
            id: 'claude',
            name: 'Claude Code CLI',
            command: 'claude',
            npmPackage: '@anthropic-ai/claude-code',
            checkCommand: 'which claude || where claude',
            versionCommand: 'claude --version',
            authCheckCommand: 'claude status',
            installCommand: 'npm install -g @anthropic-ai/claude-code',
            updateCheckInterval: 24,
            memoryServiceIntegration: true
        });
        // Gemini CLI - Google's agentic coding assistant (free tier)
        this.tools.set('gemini', {
            id: 'gemini',
            name: 'Gemini CLI',
            command: 'gemini',
            npmPackage: '@google/gemini-cli',
            checkCommand: 'which gemini || where gemini',
            versionCommand: 'gemini --version',
            authCheckCommand: 'gemini auth status',
            installCommand: 'npm install -g @google/gemini-cli',
            updateCheckInterval: 24,
            memoryServiceIntegration: true
        });
        // Qwen Code - Alibaba's agentic coding assistant (forked from Gemini CLI)
        this.tools.set('qwen', {
            id: 'qwen',
            name: 'Qwen Code',
            command: 'qwen-code',
            npmPackage: '@qwen-code/qwen-code',
            checkCommand: 'which qwen-code || where qwen-code',
            versionCommand: 'qwen-code --version',
            authCheckCommand: 'qwen-code auth status',
            installCommand: 'npm install -g @qwen-code/qwen-code',
            updateCheckInterval: 24,
            memoryServiceIntegration: true
        });
        // OpenAI Codex CLI - OpenAI's agentic coding assistant
        this.tools.set('openai', {
            id: 'openai',
            name: 'OpenAI Codex CLI',
            command: 'codex',
            npmPackage: '@openai/codex-cli',
            checkCommand: 'which codex || where codex',
            versionCommand: 'codex --version',
            authCheckCommand: 'codex auth status',
            installCommand: 'npm install -g @openai/codex-cli',
            updateCheckInterval: 24,
            memoryServiceIntegration: true
        });
        // Aider - Git-aware agentic coding assistant
        this.tools.set('aider', {
            id: 'aider',
            name: 'Aider',
            command: 'aider',
            checkCommand: 'which aider || where aider',
            versionCommand: 'aider --version',
            installCommand: 'pip install aider-chat',
            updateCheckInterval: 24,
            memoryServiceIntegration: false
        });
        // Cline - Lightweight agentic coding assistant
        this.tools.set('cline', {
            id: 'cline',
            name: 'Cline',
            command: 'cline',
            checkCommand: 'which cline || where cline',
            versionCommand: 'cline --version',
            installCommand: 'npm install -g @cline/cli',
            updateCheckInterval: 24,
            memoryServiceIntegration: false
        });
    }
    /**
     * Load saved status from disk
     */
    loadStatus() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                if (fs.existsSync(this.configPath)) {
                    const data = fs.readFileSync(this.configPath, 'utf-8');
                    const saved = JSON.parse(data);
                    for (const [toolId, status] of Object.entries(saved)) {
                        this.status.set(toolId, status);
                    }
                }
            }
            catch (error) {
                console.error('[CliToolsManager] Failed to load status:', error);
            }
        });
    }
    /**
     * Save status to disk
     */
    saveStatus() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const data = Object.fromEntries(this.status);
                fs.mkdirSync(path.dirname(this.configPath), { recursive: true });
                fs.writeFileSync(this.configPath, JSON.stringify(data, null, 2));
            }
            catch (error) {
                console.error('[CliToolsManager] Failed to save status:', error);
            }
        });
    }
    /**
     * Check if a tool is installed
     */
    checkInstalled(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                return false;
            try {
                yield exec(tool.checkCommand);
                return true;
            }
            catch (_a) {
                return false;
            }
        });
    }
    /**
     * Get tool version
     */
    getVersion(toolId) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                return undefined;
            try {
                const { stdout } = yield exec(tool.versionCommand);
                const version = (_a = stdout.trim().match(/\d+\.\d+\.\d+/)) === null || _a === void 0 ? void 0 : _a[0];
                return version;
            }
            catch (_b) {
                return undefined;
            }
        });
    }
    /**
     * Check if tool is authenticated
     */
    checkAuthenticated(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool || !tool.authCheckCommand)
                return true;
            try {
                yield exec(tool.authCheckCommand);
                return true;
            }
            catch (_a) {
                return false;
            }
        });
    }
    /**
     * Install a tool
     */
    install(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                throw new Error(`Unknown tool: ${toolId}`);
            this.emit('install-progress', {
                tool: toolId,
                status: 'checking',
                message: `Checking ${tool.name} installation...`
            });
            // Check if already installed
            const installed = yield this.checkInstalled(toolId);
            if (installed) {
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'complete',
                    message: `${tool.name} is already installed`
                });
                yield this.updateStatus(toolId);
                return;
            }
            // Check dependencies
            if (toolId === 'gh-copilot') {
                // Check if gh CLI is installed first
                try {
                    yield exec('gh --version');
                }
                catch (_a) {
                    throw new Error('GitHub CLI (gh) must be installed first. Run: brew install gh');
                }
            }
            this.emit('install-progress', {
                tool: toolId,
                status: 'installing',
                message: `Installing ${tool.name}...`
            });
            try {
                // Run installation command
                yield exec(tool.installCommand);
                // Verify installation
                const nowInstalled = yield this.checkInstalled(toolId);
                if (!nowInstalled) {
                    throw new Error('Installation verification failed');
                }
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'complete',
                    message: `${tool.name} installed successfully`
                });
                yield this.updateStatus(toolId);
                // If Claude CLI, configure Memory Service integration
                if (toolId === 'claude' && tool.memoryServiceIntegration) {
                    yield this.configureMemoryServiceIntegration(toolId);
                }
            }
            catch (error) {
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'error',
                    message: `Failed to install ${tool.name}`,
                    error: error
                });
                throw error;
            }
        });
    }
    /**
     * Configure Memory Service integration for Claude CLI
     */
    configureMemoryServiceIntegration(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            if (toolId !== 'claude')
                return;
            this.emit('install-progress', {
                tool: toolId,
                status: 'configuring',
                message: 'Configuring Memory Service integration...'
            });
            try {
                // Register Claude CLI with Memory Service
                const response = yield fetch('http://localhost:3457/api/v1/memory/register', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ toolName: 'Claude Code CLI' })
                });
                if (response.ok) {
                    const { token } = yield response.json();
                    // Save token to Claude CLI config
                    const claudeConfigPath = path.join(os.homedir(), '.claude', 'config.json');
                    let claudeConfig = {};
                    if (fs.existsSync(claudeConfigPath)) {
                        claudeConfig = JSON.parse(fs.readFileSync(claudeConfigPath, 'utf-8'));
                    }
                    claudeConfig.memoryService = {
                        enabled: true,
                        endpoint: 'http://localhost:3457',
                        token: token,
                        autoSync: true
                    };
                    fs.mkdirSync(path.dirname(claudeConfigPath), { recursive: true });
                    fs.writeFileSync(claudeConfigPath, JSON.stringify(claudeConfig, null, 2));
                    console.log('[CliToolsManager] Claude CLI registered with Memory Service');
                }
            }
            catch (error) {
                console.error('[CliToolsManager] Failed to configure Memory Service:', error);
            }
        });
    }
    /**
     * Update tool status
     */
    updateStatus(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                return;
            const status = {
                installed: yield this.checkInstalled(toolId),
                version: yield this.getVersion(toolId),
                lastChecked: new Date(),
                authenticated: yield this.checkAuthenticated(toolId)
            };
            // Find tool path
            try {
                const { stdout } = yield exec(`which ${tool.command} || where ${tool.command}`);
                status.path = stdout.trim().split('\n')[0];
            }
            catch (_a) {
                // Path not found
            }
            this.status.set(toolId, status);
            yield this.saveStatus();
            // Save to database for tracking
            yield this.saveToDatabase(toolId, status);
        });
    }
    /**
     * Save tool status to database
     */
    saveToDatabase(toolId, status) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool || !this.db)
                return;
            const syncType = `${toolId}_cli_update`;
            const now = new Date().toISOString();
            const nextCheck = new Date(Date.now() + tool.updateCheckInterval * 60 * 60 * 1000).toISOString();
            try {
                yield new Promise((resolve, reject) => {
                    this.db.run(`
          INSERT OR REPLACE INTO sync_metadata (
            id, sync_type, started_at, completed_at, status, 
            intelligence_version, next_sync_due, created_at
          ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        `, [
                        `${toolId}_${Date.now()}`,
                        syncType,
                        now,
                        now,
                        status.installed ? 'completed' : 'pending',
                        status.version || 'unknown',
                        nextCheck,
                        now
                    ], (err) => {
                        if (err)
                            reject(err);
                        else
                            resolve(true);
                    });
                });
            }
            catch (error) {
                console.error('[CliToolsManager] Failed to save to database:', error);
            }
        });
    }
    /**
     * Check for updates for a tool
     */
    checkForUpdates(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                return false;
            try {
                if (tool.npmPackage) {
                    // Check npm for latest version
                    const { stdout } = yield exec(`npm view ${tool.npmPackage} version`);
                    const latestVersion = stdout.trim();
                    const currentVersion = yield this.getVersion(toolId);
                    if (currentVersion && latestVersion !== currentVersion) {
                        const status = this.status.get(toolId) || {};
                        status.updateAvailable = true;
                        this.status.set(toolId, status);
                        yield this.saveStatus();
                        return true;
                    }
                }
            }
            catch (error) {
                console.error(`[CliToolsManager] Failed to check updates for ${toolId}:`, error);
            }
            return false;
        });
    }
    /**
     * Update a tool
     */
    update(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                throw new Error(`Unknown tool: ${toolId}`);
            this.emit('install-progress', {
                tool: toolId,
                status: 'downloading',
                message: `Updating ${tool.name}...`
            });
            try {
                if (tool.npmPackage) {
                    yield exec(`npm update -g ${tool.npmPackage}`);
                }
                yield this.updateStatus(toolId);
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'complete',
                    message: `${tool.name} updated successfully`
                });
            }
            catch (error) {
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'error',
                    message: `Failed to update ${tool.name}`,
                    error: error
                });
                throw error;
            }
        });
    }
    /**
     * Start automatic update checking
     */
    startAutoUpdateCheck() {
        for (const [toolId, tool] of this.tools) {
            // Clear existing timer
            const existingTimer = this.updateCheckTimers.get(toolId);
            if (existingTimer) {
                clearInterval(existingTimer);
            }
            // Set up new timer
            const timer = setInterval(() => __awaiter(this, void 0, void 0, function* () {
                const hasUpdate = yield this.checkForUpdates(toolId);
                if (hasUpdate) {
                    this.emit('update-available', { toolId, tool: tool.name });
                }
            }), tool.updateCheckInterval * 60 * 60 * 1000);
            this.updateCheckTimers.set(toolId, timer);
            // Also check immediately
            this.checkForUpdates(toolId);
        }
    }
    /**
     * Stop automatic update checking
     */
    stopAutoUpdateCheck() {
        for (const timer of this.updateCheckTimers.values()) {
            clearInterval(timer);
        }
        this.updateCheckTimers.clear();
    }
    /**
     * Get all tool statuses
     */
    getAllStatuses() {
        return __awaiter(this, void 0, void 0, function* () {
            const statuses = new Map();
            for (const toolId of this.tools.keys()) {
                yield this.updateStatus(toolId);
                const status = this.status.get(toolId);
                if (status) {
                    statuses.set(toolId, status);
                }
            }
            return statuses;
        });
    }
    /**
     * Get tool configuration
     */
    getTool(toolId) {
        return this.tools.get(toolId);
    }
    /**
     * Get all tools
     */
    getAllTools() {
        return new Map(this.tools);
    }
    /**
     * Uninstall a tool
     */
    uninstall(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                throw new Error(`Unknown tool: ${toolId}`);
            this.emit('install-progress', {
                tool: toolId,
                status: 'uninstalling',
                message: `Uninstalling ${tool.name}...`
            });
            try {
                // Check if tool is installed
                const installed = yield this.checkInstalled(toolId);
                if (!installed) {
                    throw new Error(`${tool.name} is not installed`);
                }
                // Uninstall based on package manager
                if (tool.npmPackage) {
                    yield exec(`npm uninstall -g ${tool.npmPackage}`);
                }
                else if (tool.command === 'aider') {
                    yield exec('pip uninstall -y aider-chat');
                }
                else {
                    // For other tools, remove from local installation
                    const toolPath = path.join(this.toolsDir, toolId);
                    if (fs.existsSync(toolPath)) {
                        fs.rmSync(toolPath, { recursive: true, force: true });
                    }
                }
                // Remove from status
                const status = this.status.get(toolId);
                if (status) {
                    status.installed = false;
                    status.version = undefined;
                    status.path = undefined;
                    status.authenticated = false;
                    this.status.set(toolId, status);
                    yield this.saveStatus();
                }
                // Remove from database
                yield this.removeFromDatabase(toolId);
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'complete',
                    message: `${tool.name} uninstalled successfully`
                });
            }
            catch (error) {
                this.emit('install-progress', {
                    tool: toolId,
                    status: 'error',
                    message: `Failed to uninstall ${tool.name}`,
                    error: error
                });
                throw error;
            }
        });
    }
    /**
     * Remove tool from database
     */
    removeFromDatabase(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool || !this.db)
                return;
            try {
                yield new Promise((resolve, reject) => {
                    this.db.run(`
          DELETE FROM sync_metadata 
          WHERE sync_type = ?
        `, [`${toolId}_cli_update`], (err) => {
                        if (err)
                            reject(err);
                        else
                            resolve(true);
                    });
                });
            }
            catch (error) {
                console.error('[CliToolsManager] Failed to remove from database:', error);
            }
        });
    }
    /**
     * Cancel an installation
     */
    cancelInstallation(toolId) {
        // TODO: Implement cancellation logic
        console.log(`[CliToolsManager] Cancelling installation of ${toolId}`);
        const status = this.status.get(toolId);
        if (status) {
            // Clear any temporary installation state
            this.status.set(toolId, status);
        }
        this.emit('install-progress', {
            tool: toolId,
            status: 'cancelled',
            message: 'Installation cancelled'
        });
    }
    /**
     * Get installation logs
     */
    getInstallationLogs(toolId) {
        // TODO: Implement log storage and retrieval
        return [`No logs available for ${toolId}`];
    }
    /**
     * Configure tool authentication
     */
    configureTool(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                throw new Error(`Unknown tool: ${toolId}`);
            // Special handling for Claude CLI
            if (toolId === 'claude' && tool.memoryServiceIntegration) {
                yield this.configureMemoryServiceIntegration(toolId);
            }
            // TODO: Add configuration for other tools
        });
    }
    /**
     * Check all tools for updates
     */
    checkAllUpdates() {
        return __awaiter(this, void 0, void 0, function* () {
            const updates = new Map();
            for (const toolId of this.tools.keys()) {
                const hasUpdate = yield this.checkForUpdates(toolId);
                updates.set(toolId, hasUpdate);
            }
            return updates;
        });
    }
    /**
     * Update advanced settings
     */
    updateSettings(settings) {
        // Update internal settings
        // This would typically update configuration that affects installation behavior
        console.log('[CliToolsManager] Settings updated:', settings);
    }
}
exports.CliToolsManager = CliToolsManager;
exports.default = CliToolsManager;
//# sourceMappingURL=CliToolsManager.js.map