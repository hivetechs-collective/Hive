"use strict";
/**
 * CLI Tool Detection Utility
 * Safely detects installed AI CLI tools without modifying system
 */
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
exports.getCachedToolStatus = exports.detectAllCliTools = exports.detectClaudeCode = void 0;
const child_process_1 = require("child_process");
const util_1 = require("util");
const execAsync = (0, util_1.promisify)(child_process_1.exec);
/**
 * Detect if Claude Code CLI is installed
 */
function detectClaudeCode() {
    return __awaiter(this, void 0, void 0, function* () {
        const status = {
            id: 'claude-code',
            name: 'Claude Code',
            installed: false
        };
        try {
            // Try to get version - this is the safest detection method
            const { stdout } = yield execAsync('claude --version 2>/dev/null');
            if (stdout) {
                status.installed = true;
                // Extract version from output (format: "claude-code version X.X.X")
                const versionMatch = stdout.match(/(\d+\.\d+\.\d+)/);
                if (versionMatch) {
                    status.version = versionMatch[1];
                }
            }
        }
        catch (error) {
            // Command not found or other error - tool not installed
            console.log('[CLI Detector] Claude Code not found');
        }
        // Try to get the executable path
        if (status.installed) {
            try {
                const { stdout: pathOutput } = yield execAsync('which claude 2>/dev/null');
                if (pathOutput) {
                    status.path = pathOutput.trim();
                }
            }
            catch (_a) {
                // Path detection failed, but tool might still be installed
            }
        }
        // Check if Memory Service connection is configured
        if (status.installed) {
            status.memoryServiceConnected = yield checkMemoryServiceConfig('claude-code');
        }
        return status;
    });
}
exports.detectClaudeCode = detectClaudeCode;
/**
 * Check if a CLI tool has Memory Service configured
 */
function checkMemoryServiceConfig(toolId) {
    return __awaiter(this, void 0, void 0, function* () {
        // For now, just check if Memory Service is running
        // Later we can check actual tool configuration
        try {
            const response = yield fetch('http://localhost:3457/health');
            return response.ok;
        }
        catch (_a) {
            return false;
        }
    });
}
/**
 * Detect all supported CLI tools
 */
function detectAllCliTools() {
    return __awaiter(this, void 0, void 0, function* () {
        const tools = [];
        // Start with Claude Code only
        tools.push(yield detectClaudeCode());
        // TODO: Add other tools incrementally
        // tools.push(await detectGeminiCli());
        // tools.push(await detectQwenCode());
        // etc.
        return tools;
    });
}
exports.detectAllCliTools = detectAllCliTools;
/**
 * Cache detection results to avoid repeated checks
 */
const detectionCache = new Map();
const CACHE_TTL = 30000; // 30 seconds
function getCachedToolStatus(toolId) {
    return __awaiter(this, void 0, void 0, function* () {
        const cached = detectionCache.get(toolId);
        if (cached && Date.now() - cached.timestamp < CACHE_TTL) {
            return cached.status;
        }
        // Cache miss or expired
        let status = null;
        if (toolId === 'claude-code') {
            status = yield detectClaudeCode();
            detectionCache.set(toolId, { status, timestamp: Date.now() });
        }
        return status;
    });
}
exports.getCachedToolStatus = getCachedToolStatus;
//# sourceMappingURL=cli-tool-detector.js.map