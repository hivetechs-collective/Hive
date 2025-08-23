"use strict";
/**
 * CLI Tools Type Definitions
 * Shared between main and renderer processes
 * Enterprise-grade type safety for 2025
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.CliToolsIpcChannels = exports.CLI_TOOLS_REGISTRY = exports.CliToolStatus = void 0;
/**
 * Represents the installation status of a CLI tool
 */
var CliToolStatus;
(function (CliToolStatus) {
    CliToolStatus["NOT_INSTALLED"] = "not_installed";
    CliToolStatus["INSTALLED"] = "installed";
    CliToolStatus["INSTALLING"] = "installing";
    CliToolStatus["UPDATING"] = "updating";
    CliToolStatus["ERROR"] = "error";
    CliToolStatus["CHECKING"] = "checking";
})(CliToolStatus = exports.CliToolStatus || (exports.CliToolStatus = {}));
/**
 * Registry of all supported CLI tools
 */
exports.CLI_TOOLS_REGISTRY = {
    'claude-code': {
        id: 'claude-code',
        name: 'Claude Code',
        description: 'Anthropic\'s terminal-native AI agent',
        command: 'claude',
        installCommand: 'npm install -g @anthropic-ai/claude-code',
        updateCommand: 'npm update -g @anthropic-ai/claude-code',
        versionCommand: 'claude --version',
        versionRegex: /(\d+\.\d+\.\d+)/,
        docsUrl: 'https://docs.anthropic.com/en/docs/claude-code',
        icon: '🤖',
        requiresNode: true
    },
    'cursor': {
        id: 'cursor',
        name: 'Cursor',
        description: 'AI-powered code editor',
        command: 'cursor',
        versionCommand: 'cursor --version',
        docsUrl: 'https://cursor.sh',
        icon: '⚡'
    },
    'continue': {
        id: 'continue',
        name: 'Continue',
        description: 'Open-source AI code assistant',
        command: 'continue',
        versionCommand: 'continue --version',
        docsUrl: 'https://continue.dev',
        icon: '🔄'
    },
    'codewhisperer': {
        id: 'codewhisperer',
        name: 'Amazon Q',
        description: 'AWS AI coding companion',
        command: 'aws',
        versionCommand: 'aws --version',
        docsUrl: 'https://aws.amazon.com/q/',
        icon: '🌟'
    },
    'cody': {
        id: 'cody',
        name: 'Cody',
        description: 'Sourcegraph AI coding assistant',
        command: 'cody',
        versionCommand: 'cody --version',
        docsUrl: 'https://sourcegraph.com/cody',
        icon: '🦊'
    },
    'qwen-code': {
        id: 'qwen-code',
        name: 'Qwen Code',
        description: 'AI-powered command-line workflow tool (2000 req/day free)',
        command: 'qwen',
        installCommand: 'npm install -g @qwen-code/qwen-code@latest',
        updateCommand: 'npm update -g @qwen-code/qwen-code',
        versionCommand: 'qwen --version',
        versionRegex: /(?:qwen\/|v?)(\d+\.\d+\.\d+)/,
        docsUrl: 'https://github.com/QwenLM/qwen-code',
        icon: '🐉',
        requiresNode: true
    },
    'gemini-cli': {
        id: 'gemini-cli',
        name: 'Gemini CLI',
        description: 'Google\'s free AI coding assistant with 1M token context',
        command: 'gemini',
        installCommand: 'npm install -g @google/gemini-cli',
        updateCommand: 'npm update -g @google/gemini-cli',
        versionCommand: 'gemini --version',
        versionRegex: /(?:gemini-cli\/|v?)(\d+\.\d+\.\d+)/,
        docsUrl: 'https://cloud.google.com/gemini/docs/codeassist/gemini-cli',
        icon: '✨',
        requiresNode: true
    },
    'openai-codex': {
        id: 'openai-codex',
        name: 'OpenAI Codex',
        description: 'OpenAI\'s agentic coding CLI with GPT-5 and o-series models',
        command: 'codex',
        installCommand: 'npm install -g @openai/codex',
        updateCommand: 'npm update -g @openai/codex',
        versionCommand: 'codex --version',
        versionRegex: /codex-cli (\d+\.\d+\.\d+)/,
        docsUrl: 'https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started',
        icon: '🧠',
        requiresNode: true
    },
    'cline': {
        id: 'cline',
        name: 'Cline',
        description: 'Task-based AI assistant with 47k+ GitHub stars',
        command: 'cline-cli',
        installCommand: 'npm install -g @yaegaki/cline-cli',
        updateCommand: 'npm update -g @yaegaki/cline-cli',
        versionCommand: 'cline-cli --version',
        versionRegex: /(\d+\.\d+\.\d+)/,
        docsUrl: 'https://cline.bot',
        icon: '🤖',
        requiresNode: true
    },
    'grok': {
        id: 'grok',
        name: 'Grok CLI',
        description: 'xAI Grok-powered terminal agent with MCP support',
        command: 'grok',
        installCommand: 'npm install -g @vibe-kit/grok-cli',
        updateCommand: 'npm update -g @vibe-kit/grok-cli',
        versionCommand: 'grok --version',
        versionRegex: /(\d+\.\d+\.\d+)/,
        docsUrl: 'https://github.com/superagent-ai/grok-cli',
        icon: '🚀',
        requiresNode: true
    }
};
/**
 * IPC channel names for CLI tools communication
 */
var CliToolsIpcChannels;
(function (CliToolsIpcChannels) {
    // Detection
    CliToolsIpcChannels["DETECT_TOOL"] = "cli-tool:detect";
    CliToolsIpcChannels["DETECT_ALL"] = "cli-tool:detect-all";
    // Installation
    CliToolsIpcChannels["INSTALL"] = "cli-tool:install";
    CliToolsIpcChannels["UPDATE"] = "cli-tool:update";
    CliToolsIpcChannels["UNINSTALL"] = "cli-tool:uninstall";
    // Launching
    CliToolsIpcChannels["LAUNCH"] = "cli-tool:launch";
    CliToolsIpcChannels["CONFIGURE"] = "cli-tool:configure";
    // Events
    CliToolsIpcChannels["STATUS_CHANGED"] = "cli-tool:status-changed";
    CliToolsIpcChannels["INSTALL_PROGRESS"] = "cli-tool:install-progress";
    CliToolsIpcChannels["UPDATE_AVAILABLE"] = "cli-tool:update-available";
    CliToolsIpcChannels["ERROR"] = "cli-tool:error";
})(CliToolsIpcChannels = exports.CliToolsIpcChannels || (exports.CliToolsIpcChannels = {}));
//# sourceMappingURL=cli-tools.js.map