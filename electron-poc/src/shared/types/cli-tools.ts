/**
 * CLI Tools Type Definitions
 * Shared between main and renderer processes
 * Enterprise-grade type safety for 2025
 */

/**
 * Represents the installation status of a CLI tool
 */
export enum CliToolStatus {
  NOT_INSTALLED = 'not_installed',
  INSTALLED = 'installed',
  INSTALLING = 'installing',
  UPDATING = 'updating',
  ERROR = 'error',
  CHECKING = 'checking'
}

/**
 * Complete information about a CLI tool
 */
export interface CliToolInfo {
  id: string;
  name: string;
  description: string;
  command: string;
  installed: boolean;
  status: CliToolStatus;
  version?: string;
  path?: string;
  lastChecked?: Date;
  memoryServiceConnected?: boolean;
  updateAvailable?: boolean;
  latestVersion?: string;
}

/**
 * Configuration for a CLI tool
 */
export interface CliToolConfig {
  id: string;
  name: string;
  description: string;
  command: string;
  installCommand?: string;
  updateCommand?: string;
  versionCommand?: string;
  versionRegex?: RegExp | string;
  docsUrl?: string;
  icon?: string;
  requiresNode?: boolean;
  requiresPython?: boolean;
  wrapperPath?: string;  // Optional path to wrapper script for terminal compatibility
}

/**
 * Registry of all supported CLI tools
 */
export const CLI_TOOLS_REGISTRY: Record<string, CliToolConfig> = {
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
  'grok': {
    id: 'grok',
    name: 'Grok CLI (Community)',
    description: '⚠️ Known issue: Module resolution error in v0.0.29. Community-built terminal agent for xAI Grok',
    command: 'grok',
    installCommand: 'npm install -g @vibe-kit/grok-cli@0.0.28', // Pinned to v0.0.28 to avoid v0.0.29 ESM bug
    updateCommand: 'npm install -g @vibe-kit/grok-cli@0.0.28', // Force v0.0.28 on update to avoid broken v0.0.29
    versionCommand: 'grok --version',
    versionRegex: /(\d+\.\d+\.\d+)/,
    docsUrl: 'https://github.com/superagent-ai/grok-cli',
    icon: '⚠️',
    requiresNode: true
  }
};

/**
 * IPC channel names for CLI tools communication
 */
export enum CliToolsIpcChannels {
  // Detection
  DETECT_TOOL = 'cli-tool:detect',
  DETECT_ALL = 'cli-tool:detect-all',
  
  // Installation
  INSTALL = 'cli-tool:install',
  UPDATE = 'cli-tool:update',
  UNINSTALL = 'cli-tool:uninstall',
  
  // Launching
  LAUNCH = 'cli-tool:launch',
  CONFIGURE = 'cli-tool:configure',
  
  // Events
  STATUS_CHANGED = 'cli-tool:status-changed',
  INSTALL_PROGRESS = 'cli-tool:install-progress',
  UPDATE_AVAILABLE = 'cli-tool:update-available',
  ERROR = 'cli-tool:error'
}

/**
 * Launch options for CLI tools
 */
export interface CliToolLaunchOptions {
  toolId: string;
  workingDirectory: string;
  env?: Record<string, string>;
  args?: string[];
}

/**
 * Progress event for installations/updates
 */
export interface CliToolProgress {
  toolId: string;
  status: CliToolStatus;
  progress: number; // 0-100
  message?: string;
  error?: string;
}