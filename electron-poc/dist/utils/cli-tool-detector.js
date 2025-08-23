"use strict";
/**
 * CLI Tool Detector
 * Checks for installed AI CLI tools and provides information about them
 * Note: This runs in the renderer process, actual detection happens in main process via IPC
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
exports.cliToolDetector = exports.CliToolDetector = exports.detectClaudeCode = void 0;
// Placeholder function for Claude Code detection
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
exports.detectClaudeCode = detectClaudeCode;
class CliToolDetector {
    constructor() {
        this.tools = new Map();
        this.initializeTools();
    }
    /**
     * Initialize the list of known tools
     */
    initializeTools() {
        const toolDefinitions = [
            { id: 'claude-code', name: 'Claude Code', command: 'claude' },
            { id: 'aider', name: 'Aider', command: 'aider' },
            { id: 'cursor', name: 'Cursor', command: 'cursor' },
            { id: 'continue', name: 'Continue', command: 'continue' },
            { id: 'codewhisperer', name: 'Amazon Q', command: 'aws' },
            { id: 'cody', name: 'Cody', command: 'cody' },
            { id: 'qwen-code', name: 'Qwen Code', command: 'qwen' },
            { id: 'gemini-cli', name: 'Gemini CLI', command: 'gemini' }
        ];
        toolDefinitions.forEach(tool => {
            this.tools.set(tool.id, Object.assign(Object.assign({}, tool), { installed: false }));
        });
    }
    /**
     * Check if a specific tool is installed (via IPC to main process)
     */
    checkTool(toolId) {
        return __awaiter(this, void 0, void 0, function* () {
            const tool = this.tools.get(toolId);
            if (!tool)
                return null;
            // This would normally call IPC, but for now just return the tool info
            // The actual detection happens in the AI CLI Tools panel
            return tool;
        });
    }
    /**
     * Check all known tools
     */
    checkAllTools() {
        return __awaiter(this, void 0, void 0, function* () {
            // For now, just return the tools map
            // Actual detection happens in the AI CLI Tools panel via IPC
            return this.tools;
        });
    }
    /**
     * Get info for a specific tool
     */
    getToolInfo(toolId) {
        return this.tools.get(toolId);
    }
    /**
     * Get all tools
     */
    getAllTools() {
        return this.tools;
    }
    /**
     * Quick check if tool exists (synchronous, from cache)
     */
    isToolInstalled(toolId) {
        const tool = this.tools.get(toolId);
        return tool ? tool.installed : false;
    }
}
exports.CliToolDetector = CliToolDetector;
// Export singleton instance for renderer process
exports.cliToolDetector = new CliToolDetector();
// Make it available globally in renderer
if (typeof window !== 'undefined') {
    window.cliToolDetector = exports.cliToolDetector;
}
//# sourceMappingURL=cli-tool-detector.js.map