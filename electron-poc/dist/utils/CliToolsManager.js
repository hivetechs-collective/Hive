"use strict";
/**
 * CLI Tools Manager
 * Handles installation, updates, and configuration of AI CLI tools
 * Integrates with Memory Service for seamless AI tool connectivity
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
const events_1 = require("events");
const child_process_1 = require("child_process");
const util_1 = require("util");
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
const os = __importStar(require("os"));
const SafeLogger_1 = require("./SafeLogger");
// Placeholder for detectClaudeCode - will use IPC instead
function detectClaudeCode() {
    return __awaiter(this, void 0, void 0, function* () {
        // This will be replaced with IPC call to main process
        return {
            id: 'claude-code',
            name: 'Claude Code',
            installed: false
        };
    });
}
const execAsync = (0, util_1.promisify)(child_process_1.exec);
const fsPromises = fs.promises;
class CliToolsManager extends events_1.EventEmitter {
    constructor(database) {
        super();
        this.tools = new Map();
        this.memoryServiceEndpoint = 'http://localhost:3457';
        this.db = database;
        this.configDir = path.join(os.homedir(), '.hive');
        this.configFile = path.join(this.configDir, 'cli-tools-config.json');
        this.initializeTools();
        this.loadConfig();
    }
    static getInstance(database) {
        if (!CliToolsManager.instance) {
            CliToolsManager.instance = new CliToolsManager(database);
        }
        return CliToolsManager.instance;
    }
    initializeTools() {
        // Claude Code
        this.tools.set('claude-code', {
            id: 'claude-code',
            name: 'Claude Code',
            description: 'Anthropic\'s terminal-native AI agent',
            installCommand: 'npm install -g @anthropic-ai/claude-code',
            updateCommand: 'npm update -g @anthropic-ai/claude-code',
            versionCommand: 'claude --version',
            checkCommand: 'which claude || where claude',
            docsUrl: 'https://docs.anthropic.com/en/docs/claude-code',
            requiresAuth: true,
            memoryServiceIntegration: true
        });
        // Gemini CLI
        this.tools.set('gemini-cli', {
            id: 'gemini-cli',
            name: 'Gemini CLI',
            description: 'Google\'s free-tier agentic assistant',
            installCommand: 'npm install -g @google/gemini-cli',
            updateCommand: 'npm update -g @google/gemini-cli',
            versionCommand: 'gemini --version',
            checkCommand: 'which gemini || where gemini',
            docsUrl: 'https://ai.google.dev/gemini/docs',
            requiresAuth: true,
            memoryServiceIntegration: false
        });
        // Qwen Code
        this.tools.set('qwen-code', {
            id: 'qwen-code',
            name: 'Qwen Code',
            description: 'Alibaba\'s open-source coding agent',
            installCommand: 'npm install -g @qwen-code/qwen-code',
            updateCommand: 'npm update -g @qwen-code/qwen-code',
            versionCommand: 'qwen --version',
            checkCommand: 'which qwen-code || where qwen-code',
            docsUrl: 'https://github.com/QwenLM/Qwen3-Coder',
            requiresAuth: false,
            memoryServiceIntegration: false
        });
        // Aider
        this.tools.set('aider', {
            id: 'aider',
            name: 'Aider',
            description: 'Git-integrated agentic editor',
            installCommand: 'pip install aider-chat',
            updateCommand: 'pip install --upgrade aider-chat',
            versionCommand: 'aider --version',
            checkCommand: 'which aider || where aider',
            docsUrl: 'https://aider.chat/docs/',
            requiresAuth: true,
            memoryServiceIntegration: false
        });
    }
    loadConfig() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                if (!fs.existsSync(this.configDir)) {
                    yield fsPromises.mkdir(this.configDir, { recursive: true });
                }
                if (fs.existsSync(this.configFile)) {
                    const data = yield fsPromises.readFile(this.configFile, 'utf-8');
                    const config = JSON.parse(data);
                    SafeLogger_1.logger.info(`[CliToolsManager] Loaded config from ${this.configFile}`);
                    return config;
                }
            }
            catch (error) {
                SafeLogger_1.logger.error('[CliToolsManager] Failed to load config:', error);
            }
            return {};
        });
    }
    saveConfig(config) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                yield fsPromises.writeFile(this.configFile, JSON.stringify(config, null, 2));
                SafeLogger_1.logger.info('[CliToolsManager] Config saved');
            }
            catch (error) {
                SafeLogger_1.logger.error('[CliToolsManager] Failed to save config:', error);
            }
        });
    }
    /**
     * Install a CLI tool
     */
    install(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool) {
                throw new Error(`Unknown tool: ${toolId}`);
            }
            SafeLogger_1.logger.info(`[CliToolsManager] Installing ${tool.name}...`);
            this.emit('install:progress', {
                toolId,
                stage: 'checking',
                message: 'Checking prerequisites...'
            });
            try {
                // Check if already installed
                const status = yield this.getToolStatus(toolId);
                if (status.installed) {
                    this.emit('install:progress', {
                        toolId,
                        stage: 'complete',
                        message: `${tool.name} is already installed (v${status.version})`
                    });
                    return;
                }
                // Check prerequisites
                if (tool.installCommand.includes('npm')) {
                    yield this.checkNodeVersion();
                }
                else if (tool.installCommand.includes('pip')) {
                    yield this.checkPythonVersion();
                }
                // Install the tool
                this.emit('install:progress', {
                    toolId,
                    stage: 'installing',
                    message: `Installing ${tool.name}...`,
                    progress: 30
                });
                yield this.executeCommand(tool.installCommand);
                // Verify installation
                const newStatus = yield this.getToolStatus(toolId);
                if (!newStatus.installed) {
                    throw new Error('Installation verification failed');
                }
                // Configure Memory Service integration if applicable
                if (tool.memoryServiceIntegration) {
                    this.emit('install:progress', {
                        toolId,
                        stage: 'configuring',
                        message: 'Configuring Memory Service integration...',
                        progress: 80
                    });
                    yield this.configureMemoryServiceIntegration(toolId);
                }
                // Save installation info
                const config = yield this.loadConfig();
                config[toolId] = {
                    installed: true,
                    version: newStatus.version,
                    installedAt: new Date().toISOString()
                };
                yield this.saveConfig(config);
                // Save to database if available
                if (this.db) {
                    yield this.saveToDatabase(toolId, newStatus);
                }
                this.emit('install:progress', {
                    toolId,
                    stage: 'complete',
                    message: `${tool.name} installed successfully!`,
                    progress: 100
                });
                SafeLogger_1.logger.info(`[CliToolsManager] ${tool.name} installed successfully`);
            }
            catch (error) {
                SafeLogger_1.logger.error(`[CliToolsManager] Installation failed for ${toolId}:`, error);
                this.emit('install:progress', {
                    toolId,
                    stage: 'error',
                    message: `Installation failed: ${error.message}`
                });
                throw error;
            }
        });
    }
    /**
     * Update a CLI tool
     */
    update(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool) {
                throw new Error(`Unknown tool: ${toolId}`);
            }
            SafeLogger_1.logger.info(`[CliToolsManager] Updating ${tool.name}...`);
            this.emit('update:progress', {
                toolId,
                stage: 'checking',
                message: 'Checking for updates...'
            });
            try {
                // Check if installed
                const status = yield this.getToolStatus(toolId);
                if (!status.installed) {
                    throw new Error(`${tool.name} is not installed`);
                }
                const currentVersion = status.version;
                // Run update command
                this.emit('update:progress', {
                    toolId,
                    stage: 'installing',
                    message: `Updating ${tool.name}...`,
                    progress: 50
                });
                yield this.executeCommand(tool.updateCommand);
                // Check new version
                const newStatus = yield this.getToolStatus(toolId);
                const newVersion = newStatus.version;
                if (currentVersion === newVersion) {
                    this.emit('update:progress', {
                        toolId,
                        stage: 'complete',
                        message: `${tool.name} is already up to date (v${newVersion})`
                    });
                }
                else {
                    // Update config
                    const config = yield this.loadConfig();
                    config[toolId] = Object.assign(Object.assign({}, config[toolId]), { version: newVersion, updatedAt: new Date().toISOString() });
                    yield this.saveConfig(config);
                    this.emit('update:progress', {
                        toolId,
                        stage: 'complete',
                        message: `${tool.name} updated to v${newVersion}`,
                        progress: 100
                    });
                }
                SafeLogger_1.logger.info(`[CliToolsManager] ${tool.name} updated successfully`);
            }
            catch (error) {
                SafeLogger_1.logger.error(`[CliToolsManager] Update failed for ${toolId}:`, error);
                this.emit('update:progress', {
                    toolId,
                    stage: 'error',
                    message: `Update failed: ${error.message}`
                });
                throw error;
            }
        });
    }
    /**
     * Configure a CLI tool (especially Memory Service integration)
     */
    configure(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool) {
                throw new Error(`Unknown tool: ${toolId}`);
            }
            SafeLogger_1.logger.info(`[CliToolsManager] Configuring ${tool.name}...`);
            try {
                // Check if installed
                const status = yield this.getToolStatus(toolId);
                if (!status.installed) {
                    throw new Error(`${tool.name} is not installed`);
                }
                if (tool.memoryServiceIntegration) {
                    yield this.configureMemoryServiceIntegration(toolId);
                }
                // Tool-specific configuration
                if (toolId === 'claude-code') {
                    yield this.configureClaudeCode();
                }
                SafeLogger_1.logger.info(`[CliToolsManager] ${tool.name} configured successfully`);
            }
            catch (error) {
                SafeLogger_1.logger.error(`[CliToolsManager] Configuration failed for ${toolId}:`, error);
                throw error;
            }
        });
    }
    /**
     * Configure Memory Service integration for a tool
     */
    configureMemoryServiceIntegration(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Register with Memory Service
                const response = yield fetch(`${this.memoryServiceEndpoint}/api/v1/memory/register`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({
                        toolName: toolId,
                        clientInfo: {
                            version: yield this.getToolVersion(toolId),
                            platform: process.platform,
                            nodeVersion: process.version
                        }
                    })
                });
                if (!response.ok) {
                    throw new Error(`Memory Service registration failed: ${response.statusText}`);
                }
                const { token, endpoint } = yield response.json();
                SafeLogger_1.logger.info(`[CliToolsManager] Registered ${toolId} with Memory Service`);
                // Save the token for the tool
                const config = yield this.loadConfig();
                config[toolId] = Object.assign(Object.assign({}, config[toolId]), { memoryService: {
                        endpoint,
                        token,
                        connectedAt: new Date().toISOString()
                    } });
                yield this.saveConfig(config);
                // Configure the tool to use Memory Service
                if (toolId === 'claude-code') {
                    yield this.updateClaudeCodeConfig({ memoryService: { endpoint, token, enabled: true } });
                }
            }
            catch (error) {
                SafeLogger_1.logger.error(`[CliToolsManager] Memory Service integration failed:`, error);
                throw error;
            }
        });
    }
    /**
     * Configure Claude Code specifically
     */
    configureClaudeCode() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const claudeConfigPath = path.join(os.homedir(), '.claude', 'config.json');
            try {
                // Ensure directory exists
                const claudeDir = path.dirname(claudeConfigPath);
                if (!fs.existsSync(claudeDir)) {
                    yield fsPromises.mkdir(claudeDir, { recursive: true });
                }
                // Load existing config or create new
                let claudeConfig = {};
                if (fs.existsSync(claudeConfigPath)) {
                    const data = yield fsPromises.readFile(claudeConfigPath, 'utf-8');
                    claudeConfig = JSON.parse(data);
                }
                // Get Memory Service config
                const config = yield this.loadConfig();
                const memoryConfig = (_a = config['claude-code']) === null || _a === void 0 ? void 0 : _a.memoryService;
                // Update Claude config with Memory Service
                if (memoryConfig) {
                    claudeConfig.memoryService = {
                        endpoint: memoryConfig.endpoint,
                        token: memoryConfig.token,
                        enabled: true
                    };
                }
                // Enable MCP if available
                claudeConfig.mcpServers = Object.assign(Object.assign({}, claudeConfig.mcpServers), { 'hive-memory': {
                        command: 'npx',
                        args: ['@hive/mcp-memory-server', '--port', '3457']
                    } });
                // Save Claude config
                yield fsPromises.writeFile(claudeConfigPath, JSON.stringify(claudeConfig, null, 2));
                SafeLogger_1.logger.info('[CliToolsManager] Claude Code configuration updated');
            }
            catch (error) {
                SafeLogger_1.logger.error('[CliToolsManager] Failed to configure Claude Code:', error);
                throw error;
            }
        });
    }
    /**
     * Update Claude Code configuration
     */
    updateClaudeCodeConfig(updates) {
        return __awaiter(this, void 0, void 0, function* () {
            const claudeConfigPath = path.join(os.homedir(), '.claude', 'config.json');
            const mcpConfigPath = path.join(os.homedir(), '.claude', '.mcp.json');
            try {
                // Update main config
                let config = {};
                if (fs.existsSync(claudeConfigPath)) {
                    const data = yield fsPromises.readFile(claudeConfigPath, 'utf-8');
                    config = JSON.parse(data);
                }
                // Merge updates
                config = Object.assign(Object.assign({}, config), updates);
                // Ensure directory exists
                const dir = path.dirname(claudeConfigPath);
                if (!fs.existsSync(dir)) {
                    yield fsPromises.mkdir(dir, { recursive: true });
                }
                yield fsPromises.writeFile(claudeConfigPath, JSON.stringify(config, null, 2));
                SafeLogger_1.logger.info('[CliToolsManager] Claude Code config updated');
                // Update MCP configuration if Memory Service is enabled
                if (updates.memoryService && updates.memoryService.enabled) {
                    let mcpConfig = { servers: {} };
                    if (fs.existsSync(mcpConfigPath)) {
                        const mcpData = yield fsPromises.readFile(mcpConfigPath, 'utf-8');
                        mcpConfig = JSON.parse(mcpData);
                    }
                    // Add Memory Service MCP server
                    mcpConfig.servers['hive-memory-service'] = {
                        command: 'node',
                        args: [
                            path.join(this.configDir, 'memory-service-mcp-wrapper.js')
                        ],
                        env: {
                            MEMORY_SERVICE_ENDPOINT: updates.memoryService.endpoint,
                            MEMORY_SERVICE_TOKEN: updates.memoryService.token
                        },
                        description: 'Hive Consensus Memory Service - AI memory and learning system'
                    };
                    yield fsPromises.writeFile(mcpConfigPath, JSON.stringify(mcpConfig, null, 2));
                    SafeLogger_1.logger.info('[CliToolsManager] MCP configuration updated with Memory Service');
                    // Create the MCP wrapper script
                    yield this.createMemoryServiceMCPWrapper(updates.memoryService);
                }
            }
            catch (error) {
                SafeLogger_1.logger.error('[CliToolsManager] Failed to update Claude Code config:', error);
            }
        });
    }
    /**
     * Create MCP wrapper script for Memory Service
     */
    createMemoryServiceMCPWrapper(memoryService) {
        return __awaiter(this, void 0, void 0, function* () {
            const wrapperPath = path.join(this.configDir, 'memory-service-mcp-wrapper.js');
            const wrapperScript = `#!/usr/bin/env node
/**
 * MCP Wrapper for Hive Memory Service
 * This script provides an MCP-compatible interface to the Memory Service
 */

const { Server } = require('@modelcontextprotocol/sdk');
const fetch = require('node-fetch');

const ENDPOINT = process.env.MEMORY_SERVICE_ENDPOINT || 'http://localhost:3457';
const TOKEN = process.env.MEMORY_SERVICE_TOKEN;

class MemoryServiceMCP extends Server {
  constructor() {
    super({
      name: 'hive-memory-service',
      version: '1.0.0',
      description: 'Hive Consensus Memory Service'
    });

    this.registerTool({
      name: 'query_memory',
      description: 'Query the AI memory system for relevant learnings',
      parameters: {
        query: { type: 'string', required: true },
        limit: { type: 'number', default: 5 }
      },
      handler: this.queryMemory.bind(this)
    });

    this.registerTool({
      name: 'contribute_learning',
      description: 'Contribute a new learning to the memory system',
      parameters: {
        type: { type: 'string', required: true },
        category: { type: 'string', required: true },
        content: { type: 'string', required: true },
        code: { type: 'string' }
      },
      handler: this.contributeLearning.bind(this)
    });
  }

  async queryMemory({ query, limit = 5 }) {
    const response = await fetch(\`\${ENDPOINT}/api/v1/memory/query\`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': \`Bearer \${TOKEN}\`,
        'X-Client-Name': 'claude-code-mcp'
      },
      body: JSON.stringify({
        client: 'claude-code',
        context: { file: process.cwd() },
        query,
        options: { limit }
      })
    });

    return await response.json();
  }

  async contributeLearning({ type, category, content, code }) {
    const response = await fetch(\`\${ENDPOINT}/api/v1/memory/contribute\`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': \`Bearer \${TOKEN}\`,
        'X-Client-Name': 'claude-code-mcp'
      },
      body: JSON.stringify({
        source: 'claude-code',
        learning: {
          type,
          category,
          content,
          code,
          context: { file: process.cwd(), success: true }
        }
      })
    });

    return await response.json();
  }
}

// Start the MCP server
const server = new MemoryServiceMCP();
server.start();
`;
            yield fsPromises.writeFile(wrapperPath, wrapperScript, { mode: 0o755 });
            SafeLogger_1.logger.info('[CliToolsManager] Created Memory Service MCP wrapper');
        });
    }
    /**
     * Get tool status
     */
    getToolStatus(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            if (toolId === 'claude-code') {
                return yield detectClaudeCode();
            }
            // Generic detection for other tools
            const tool = this.tools.get(toolId);
            if (!tool) {
                return {
                    id: toolId,
                    name: 'Unknown',
                    installed: false
                };
            }
            try {
                const { stdout } = yield execAsync(tool.versionCommand + ' 2>/dev/null');
                const versionMatch = stdout.match(/(\d+\.\d+\.\d+)/);
                return {
                    id: toolId,
                    name: tool.name,
                    installed: true,
                    version: versionMatch ? versionMatch[1] : 'unknown'
                };
            }
            catch (_a) {
                return {
                    id: toolId,
                    name: tool.name,
                    installed: false
                };
            }
        });
    }
    /**
     * Get tool version
     */
    getToolVersion(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const status = yield this.getToolStatus(toolId);
            return status.version || 'unknown';
        });
    }
    /**
     * Check Node.js version
     */
    checkNodeVersion() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const { stdout } = yield execAsync('node --version');
                const version = stdout.trim();
                const major = parseInt(version.split('.')[0].substring(1));
                if (major < 18) {
                    throw new Error(`Node.js 18+ required (current: ${version})`);
                }
                SafeLogger_1.logger.info(`[CliToolsManager] Node.js version OK: ${version}`);
            }
            catch (error) {
                throw new Error('Node.js 18+ is required for npm installations');
            }
        });
    }
    /**
     * Check Python version
     */
    checkPythonVersion() {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                const { stdout } = yield execAsync('python3 --version');
                const version = stdout.trim();
                SafeLogger_1.logger.info(`[CliToolsManager] Python version OK: ${version}`);
            }
            catch (_a) {
                throw new Error('Python 3 is required for pip installations');
            }
        });
    }
    /**
     * Execute command with progress tracking
     */
    executeCommand(command) {
        return new Promise((resolve, reject) => {
            SafeLogger_1.logger.info(`[CliToolsManager] Executing: ${command}`);
            (0, child_process_1.exec)(command, (error, stdout, stderr) => {
                if (error) {
                    SafeLogger_1.logger.error(`[CliToolsManager] Command failed: ${stderr}`);
                    reject(error);
                }
                else {
                    SafeLogger_1.logger.info(`[CliToolsManager] Command succeeded`);
                    resolve();
                }
            });
        });
    }
    /**
     * Save tool status to database
     */
    saveToDatabase(toolId, status) {
        return __awaiter(this, void 0, void 0, function* () {
            if (!this.db)
                return;
            const syncType = `${toolId}_cli_update`;
            const now = new Date().toISOString();
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
                        new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
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
                SafeLogger_1.logger.error('[CliToolsManager] Failed to save to database:', error);
            }
        });
    }
    /**
     * Check for updates for all installed tools
     */
    checkForUpdates() {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const updates = new Map();
            for (const [toolId, tool] of this.tools) {
                try {
                    const status = yield this.getToolStatus(toolId);
                    if (status.installed) {
                        // For npm packages, check latest version
                        if (tool.installCommand.includes('npm')) {
                            const packageName = (_a = tool.installCommand.match(/@[^@\s]+/)) === null || _a === void 0 ? void 0 : _a[0];
                            if (packageName) {
                                const { stdout } = yield execAsync(`npm view ${packageName} version 2>/dev/null`);
                                const latestVersion = stdout.trim();
                                updates.set(toolId, latestVersion !== status.version);
                            }
                        }
                    }
                }
                catch (error) {
                    SafeLogger_1.logger.error(`[CliToolsManager] Failed to check updates for ${toolId}:`, error);
                }
            }
            return updates;
        });
    }
    /**
     * Launch a CLI tool in a specific project directory
     */
    launch(toolId, projectPath) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool) {
                throw new Error(`Unknown tool: ${toolId}`);
            }
            SafeLogger_1.logger.info(`[CliToolsManager] Launching ${tool.name} in ${projectPath}...`);
            // Check if tool is installed
            const status = yield this.getToolStatus(toolId);
            if (!status.installed) {
                throw new Error(`${tool.name} is not installed. Please install it first.`);
            }
            // Special handling for Claude Code
            if (toolId === 'claude-code') {
                yield this.launchClaudeCode(projectPath);
            }
            else {
                // Generic launch for other tools
                throw new Error(`Launch not yet implemented for ${tool.name}`);
            }
        });
    }
    /**
     * Launch Claude Code in a specific project directory
     */
    launchClaudeCode(projectPath) {
        return __awaiter(this, void 0, void 0, function* () {
            try {
                // Open a new terminal window and launch Claude Code
                // This uses platform-specific commands
                const platform = process.platform;
                let command;
                if (platform === 'darwin') {
                    // macOS: Open Terminal and run claude
                    command = `osascript -e 'tell application "Terminal" to do script "cd \\"${projectPath}\\" && claude"'`;
                }
                else if (platform === 'win32') {
                    // Windows: Open Command Prompt and run claude
                    command = `start cmd /k "cd /d ${projectPath} && claude"`;
                }
                else {
                    // Linux: Try to open a terminal emulator
                    // This is a best-effort attempt as terminal emulators vary
                    command = `gnome-terminal -- bash -c "cd '${projectPath}' && claude; exec bash" || xterm -e "cd '${projectPath}' && claude; bash" || konsole -e "cd '${projectPath}' && claude"`;
                }
                SafeLogger_1.logger.info(`[CliToolsManager] Executing launch command: ${command}`);
                yield this.executeCommand(command);
                SafeLogger_1.logger.info(`[CliToolsManager] Claude Code launched successfully in ${projectPath}`);
            }
            catch (error) {
                SafeLogger_1.logger.error('[CliToolsManager] Failed to launch Claude Code:', error);
                throw new Error(`Failed to launch Claude Code: ${error}`);
            }
        });
    }
    /**
     * Get all tools
     */
    getAllTools() {
        return new Map(this.tools);
    }
}
exports.CliToolsManager = CliToolsManager;
exports.default = CliToolsManager;
//# sourceMappingURL=CliToolsManager.js.map