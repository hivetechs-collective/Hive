"use strict";
/**
 * CLI Tools Detector
 * Main process module for detecting installed CLI tools
 * Enterprise-grade implementation with proper error handling
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
exports.cliToolsDetector = exports.CliToolsDetector = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const cli_tools_1 = require("../../shared/types/cli-tools");
const SafeLogger_1 = require("../../utils/SafeLogger");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
/**
 * Detector class for CLI tools
 * Handles detection of installed tools with caching
 */
class CliToolsDetector {
    constructor() {
        this.cache = new Map();
        this.cacheTimeout = 30000; // 30 seconds cache
        this.lastCacheUpdate = new Map();
    }
    /**
     * Get enhanced PATH with common installation directories
     */
    getEnhancedPath() {
        const pathAdditions = [
            '/opt/homebrew/bin',
            '/usr/local/bin',
            '/usr/bin',
            '/bin',
            '/usr/sbin',
            '/sbin',
            path.join(process.env.HOME || '', '.local', 'bin'),
            path.join(process.env.HOME || '', '.cargo', 'bin'),
            path.join(process.env.HOME || '', 'go', 'bin'), // Go binaries
        ];
        const currentPath = process.env.PATH || '';
        const allPaths = [...new Set([...pathAdditions, ...currentPath.split(path.delimiter)])];
        return allPaths.join(path.delimiter);
    }
    /**
     * Detect a single CLI tool
     */
    detectTool(toolId, forceRefresh = false) {
        return __awaiter(this, void 0, void 0, function* () {
            // Check cache first
            if (!forceRefresh && this.isCacheValid(toolId)) {
                SafeLogger_1.logger.info(`[CliToolsDetector] Using cached result for ${toolId}`);
                return this.cache.get(toolId) || null;
            }
            const config = cli_tools_1.CLI_TOOLS_REGISTRY[toolId];
            if (!config) {
                SafeLogger_1.logger.warn(`[CliToolsDetector] Unknown tool ID: ${toolId}`);
                return null;
            }
            SafeLogger_1.logger.info(`[CliToolsDetector] Detecting ${config.name}...`);
            const toolInfo = {
                id: toolId,
                name: config.name,
                description: config.description,
                command: config.command,
                installed: false,
                status: cli_tools_1.CliToolStatus.CHECKING,
                lastChecked: new Date()
            };
            try {
                // Check if command exists
                const enhancedPath = this.getEnhancedPath();
                const env = Object.assign(Object.assign({}, process.env), { PATH: enhancedPath });
                const { stdout: whichOutput } = yield execAsync(`which ${config.command}`, { env });
                const toolPath = whichOutput.trim();
                if (!toolPath) {
                    toolInfo.status = cli_tools_1.CliToolStatus.NOT_INSTALLED;
                    this.updateCache(toolId, toolInfo);
                    return toolInfo;
                }
                toolInfo.path = toolPath;
                toolInfo.installed = true;
                toolInfo.status = cli_tools_1.CliToolStatus.INSTALLED;
                // Try to get version if command is provided
                if (config.versionCommand) {
                    try {
                        const { stdout: versionOutput } = yield execAsync(config.versionCommand, {
                            env,
                            timeout: 5000 // 5 second timeout for version check
                        });
                        toolInfo.version = this.extractVersion(versionOutput, config);
                        SafeLogger_1.logger.info(`[CliToolsDetector] ${config.name} version: ${toolInfo.version}`);
                    }
                    catch (versionError) {
                        SafeLogger_1.logger.warn(`[CliToolsDetector] Could not get version for ${config.name}:`, versionError);
                        toolInfo.version = 'unknown';
                    }
                }
                // Check for memory service connection (for supported tools)
                if (toolId === 'claude-code' || toolId === 'gemini-cli' || toolId === 'qwen-code' || toolId === 'openai-codex' || toolId === 'cline' || toolId === 'grok') {
                    toolInfo.memoryServiceConnected = yield this.checkMemoryServiceConnection(toolId);
                }
            }
            catch (error) {
                SafeLogger_1.logger.info(`[CliToolsDetector] ${config.name} not found in PATH`);
                toolInfo.status = cli_tools_1.CliToolStatus.NOT_INSTALLED;
            }
            this.updateCache(toolId, toolInfo);
            return toolInfo;
        });
    }
    /**
     * Detect all registered CLI tools
     */
    detectAllTools(forceRefresh = false) {
        return __awaiter(this, void 0, void 0, function* () {
            const toolIds = Object.keys(cli_tools_1.CLI_TOOLS_REGISTRY);
            const detectionPromises = toolIds.map(id => this.detectTool(id, forceRefresh));
            const results = yield Promise.all(detectionPromises);
            return results.filter(tool => tool !== null);
        });
    }
    /**
     * Extract version from command output
     */
    extractVersion(output, config) {
        if (!output)
            return 'unknown';
        // Clean the output
        const cleanOutput = output.trim();
        // Special handling for Claude Code
        if (config.id === 'claude-code') {
            // Claude Code outputs just the version number followed by (Claude Code)
            const match = cleanOutput.match(/^([\d.]+)/);
            if (match)
                return match[1];
        }
        // Special handling for Gemini CLI
        if (config.id === 'gemini-cli') {
            // Gemini CLI outputs format like "gemini-cli/0.1.18" or just "0.1.18"
            const match = cleanOutput.match(/(?:gemini-cli\/)?(\d+\.\d+\.\d+)/);
            if (match)
                return match[1];
        }
        // Use provided regex if available
        if (config.versionRegex) {
            const regex = typeof config.versionRegex === 'string'
                ? new RegExp(config.versionRegex)
                : config.versionRegex;
            const match = cleanOutput.match(regex);
            if (match && match[1])
                return match[1];
        }
        // Generic version extraction
        const genericMatch = cleanOutput.match(/(\d+\.\d+\.\d+(?:\.\d+)?)/);
        if (genericMatch)
            return genericMatch[1];
        // If no version found, return first line of output (truncated)
        const firstLine = cleanOutput.split('\n')[0];
        return firstLine.substring(0, 50);
    }
    /**
     * Check if tool is connected to memory service
     */
    checkMemoryServiceConnection(toolId) {
        var _a, _b, _c, _d, _e, _f, _g, _h;
        return __awaiter(this, void 0, void 0, function* () {
            try {
                let token;
                let endpoint;
                // Grok is unique - it uses its own MCP config file
                if (toolId === 'grok') {
                    const grokMcpPath = path.join(os.homedir(), '.grok', 'mcp-config.json');
                    if (fs.existsSync(grokMcpPath)) {
                        try {
                            const grokMcp = JSON.parse(fs.readFileSync(grokMcpPath, 'utf-8'));
                            const memoryServer = (_a = grokMcp.servers) === null || _a === void 0 ? void 0 : _a['hive-memory-service'];
                            if (memoryServer === null || memoryServer === void 0 ? void 0 : memoryServer.env) {
                                token = memoryServer.env.MEMORY_SERVICE_TOKEN;
                                endpoint = memoryServer.env.MEMORY_SERVICE_ENDPOINT;
                            }
                        }
                        catch (e) {
                            SafeLogger_1.logger.debug(`[CliToolsDetector] Failed to parse Grok MCP config:`, e);
                        }
                    }
                    if (!token) {
                        // Fallback to checking cli-tools-config.json for Grok
                        const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
                        if (fs.existsSync(configPath)) {
                            const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
                            token = (_c = (_b = config[toolId]) === null || _b === void 0 ? void 0 : _b.memoryService) === null || _c === void 0 ? void 0 : _c.token;
                            endpoint = (_e = (_d = config[toolId]) === null || _d === void 0 ? void 0 : _d.memoryService) === null || _e === void 0 ? void 0 : _e.endpoint;
                        }
                    }
                }
                else {
                    // Other tools use the standard cli-tools-config.json
                    const configPath = path.join(os.homedir(), '.hive', 'cli-tools-config.json');
                    if (!fs.existsSync(configPath)) {
                        SafeLogger_1.logger.debug(`[CliToolsDetector] No config file found for memory service check`);
                        return false;
                    }
                    const config = JSON.parse(fs.readFileSync(configPath, 'utf-8'));
                    const toolConfig = config[toolId];
                    token = (_f = toolConfig === null || toolConfig === void 0 ? void 0 : toolConfig.memoryService) === null || _f === void 0 ? void 0 : _f.token;
                    endpoint = (_g = toolConfig === null || toolConfig === void 0 ? void 0 : toolConfig.memoryService) === null || _g === void 0 ? void 0 : _g.endpoint;
                }
                if (!token) {
                    SafeLogger_1.logger.debug(`[CliToolsDetector] No memory service token found for ${toolId}`);
                    return false;
                }
                // Check if the token is valid by querying the Memory Service
                const memoryServicePort = ((_h = endpoint === null || endpoint === void 0 ? void 0 : endpoint.match(/:(\d+)/)) === null || _h === void 0 ? void 0 : _h[1]) || '3457';
                // Use node's http module instead of fetch for compatibility
                const http = require('http');
                return new Promise((resolve) => {
                    const options = {
                        hostname: 'localhost',
                        port: memoryServicePort,
                        path: '/api/v1/memory/stats',
                        method: 'GET',
                        headers: {
                            'Authorization': `Bearer ${token}`,
                            'X-Client-Name': toolId
                        },
                        timeout: 2000
                    };
                    const req = http.request(options, (res) => {
                        // If we get any response (even 401), the service is running
                        // We just care if the service is accessible
                        resolve(res.statusCode === 200 || res.statusCode === 401);
                    });
                    req.on('error', () => {
                        SafeLogger_1.logger.debug(`[CliToolsDetector] Memory service not accessible for ${toolId}`);
                        resolve(false);
                    });
                    req.on('timeout', () => {
                        req.destroy();
                        resolve(false);
                    });
                    req.end();
                });
            }
            catch (error) {
                SafeLogger_1.logger.debug(`[CliToolsDetector] Failed to check memory service connection for ${toolId}:`, error);
                return false;
            }
        });
    }
    /**
     * Check if cache is valid for a tool
     */
    isCacheValid(toolId) {
        const lastUpdate = this.lastCacheUpdate.get(toolId);
        if (!lastUpdate)
            return false;
        const now = Date.now();
        return (now - lastUpdate) < this.cacheTimeout;
    }
    /**
     * Update cache for a tool
     */
    updateCache(toolId, toolInfo) {
        this.cache.set(toolId, toolInfo);
        this.lastCacheUpdate.set(toolId, Date.now());
    }
    /**
     * Clear cache for a specific tool or all tools
     */
    clearCache(toolId) {
        if (toolId) {
            this.cache.delete(toolId);
            this.lastCacheUpdate.delete(toolId);
        }
        else {
            this.cache.clear();
            this.lastCacheUpdate.clear();
        }
    }
    /**
     * Get cached tool info without detection
     */
    getCachedTool(toolId) {
        return this.cache.get(toolId) || null;
    }
}
exports.CliToolsDetector = CliToolsDetector;
// Export singleton instance
exports.cliToolsDetector = new CliToolsDetector();
//# sourceMappingURL=detector.js.map