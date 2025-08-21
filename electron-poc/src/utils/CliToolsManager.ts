/**
 * CLI Tools Manager
 * Handles installation, updates, and configuration of AI CLI tools
 * Integrates with Memory Service for seamless AI tool connectivity
 */

import { EventEmitter } from 'events';
import { exec as execCallback, spawn } from 'child_process';
import { promisify } from 'util';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import { logger } from './SafeLogger';
import { detectClaudeCode, CliToolStatus } from './cli-tool-detector';

const execAsync = promisify(execCallback);
const fsPromises = fs.promises;

export interface CliToolConfig {
  id: string;
  name: string;
  description: string;
  installCommand: string;
  updateCommand: string;
  versionCommand: string;
  checkCommand: string;
  docsUrl: string;
  requiresAuth: boolean;
  memoryServiceIntegration: boolean;
}

export interface InstallProgress {
  toolId: string;
  stage: 'checking' | 'installing' | 'configuring' | 'complete' | 'error';
  message: string;
  progress?: number;
}

export class CliToolsManager extends EventEmitter {
  private static instance: CliToolsManager;
  private tools: Map<string, CliToolConfig> = new Map();
  private configDir: string;
  private configFile: string;
  private memoryServiceEndpoint: string = 'http://localhost:3457';
  private db: any; // SQLite database connection

  private constructor(database?: any) {
    super();
    this.db = database;
    this.configDir = path.join(os.homedir(), '.hive');
    this.configFile = path.join(this.configDir, 'cli-tools-config.json');
    this.initializeTools();
    this.loadConfig();
  }

  public static getInstance(database?: any): CliToolsManager {
    if (!CliToolsManager.instance) {
      CliToolsManager.instance = new CliToolsManager(database);
    }
    return CliToolsManager.instance;
  }

  private initializeTools() {
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

  private async loadConfig() {
    try {
      if (!fs.existsSync(this.configDir)) {
        await fsPromises.mkdir(this.configDir, { recursive: true });
      }

      if (fs.existsSync(this.configFile)) {
        const data = await fsPromises.readFile(this.configFile, 'utf-8');
        const config = JSON.parse(data);
        logger.info(`[CliToolsManager] Loaded config from ${this.configFile}`);
        return config;
      }
    } catch (error) {
      logger.error('[CliToolsManager] Failed to load config:', error);
    }
    return {};
  }

  private async saveConfig(config: any) {
    try {
      await fsPromises.writeFile(this.configFile, JSON.stringify(config, null, 2));
      logger.info('[CliToolsManager] Config saved');
    } catch (error) {
      logger.error('[CliToolsManager] Failed to save config:', error);
    }
  }

  /**
   * Install a CLI tool
   */
  public async install(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) {
      throw new Error(`Unknown tool: ${toolId}`);
    }

    logger.info(`[CliToolsManager] Installing ${tool.name}...`);
    this.emit('install:progress', {
      toolId,
      stage: 'checking',
      message: 'Checking prerequisites...'
    } as InstallProgress);

    try {
      // Check if already installed
      const status = await this.getToolStatus(toolId);
      if (status.installed) {
        this.emit('install:progress', {
          toolId,
          stage: 'complete',
          message: `${tool.name} is already installed (v${status.version})`
        } as InstallProgress);
        return;
      }

      // Check prerequisites
      if (tool.installCommand.includes('npm')) {
        await this.checkNodeVersion();
      } else if (tool.installCommand.includes('pip')) {
        await this.checkPythonVersion();
      }

      // Install the tool
      this.emit('install:progress', {
        toolId,
        stage: 'installing',
        message: `Installing ${tool.name}...`,
        progress: 30
      } as InstallProgress);

      await this.executeCommand(tool.installCommand);

      // Verify installation
      const newStatus = await this.getToolStatus(toolId);
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
        } as InstallProgress);

        await this.configureMemoryServiceIntegration(toolId);
      }

      // Save installation info
      const config = await this.loadConfig();
      config[toolId] = {
        installed: true,
        version: newStatus.version,
        installedAt: new Date().toISOString()
      };
      await this.saveConfig(config);

      // Save to database if available
      if (this.db) {
        await this.saveToDatabase(toolId, newStatus);
      }

      this.emit('install:progress', {
        toolId,
        stage: 'complete',
        message: `${tool.name} installed successfully!`,
        progress: 100
      } as InstallProgress);

      logger.info(`[CliToolsManager] ${tool.name} installed successfully`);
    } catch (error) {
      logger.error(`[CliToolsManager] Installation failed for ${toolId}:`, error);
      this.emit('install:progress', {
        toolId,
        stage: 'error',
        message: `Installation failed: ${error.message}`
      } as InstallProgress);
      throw error;
    }
  }

  /**
   * Update a CLI tool
   */
  public async update(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) {
      throw new Error(`Unknown tool: ${toolId}`);
    }

    logger.info(`[CliToolsManager] Updating ${tool.name}...`);
    this.emit('update:progress', {
      toolId,
      stage: 'checking',
      message: 'Checking for updates...'
    });

    try {
      // Check if installed
      const status = await this.getToolStatus(toolId);
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

      await this.executeCommand(tool.updateCommand);

      // Check new version
      const newStatus = await this.getToolStatus(toolId);
      const newVersion = newStatus.version;

      if (currentVersion === newVersion) {
        this.emit('update:progress', {
          toolId,
          stage: 'complete',
          message: `${tool.name} is already up to date (v${newVersion})`
        });
      } else {
        // Update config
        const config = await this.loadConfig();
        config[toolId] = {
          ...config[toolId],
          version: newVersion,
          updatedAt: new Date().toISOString()
        };
        await this.saveConfig(config);

        this.emit('update:progress', {
          toolId,
          stage: 'complete',
          message: `${tool.name} updated to v${newVersion}`,
          progress: 100
        });
      }

      logger.info(`[CliToolsManager] ${tool.name} updated successfully`);
    } catch (error) {
      logger.error(`[CliToolsManager] Update failed for ${toolId}:`, error);
      this.emit('update:progress', {
        toolId,
        stage: 'error',
        message: `Update failed: ${error.message}`
      });
      throw error;
    }
  }

  /**
   * Configure a CLI tool (especially Memory Service integration)
   */
  public async configure(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) {
      throw new Error(`Unknown tool: ${toolId}`);
    }

    logger.info(`[CliToolsManager] Configuring ${tool.name}...`);

    try {
      // Check if installed
      const status = await this.getToolStatus(toolId);
      if (!status.installed) {
        throw new Error(`${tool.name} is not installed`);
      }

      if (tool.memoryServiceIntegration) {
        await this.configureMemoryServiceIntegration(toolId);
      }

      // Tool-specific configuration
      if (toolId === 'claude-code') {
        await this.configureClaudeCode();
      }

      logger.info(`[CliToolsManager] ${tool.name} configured successfully`);
    } catch (error) {
      logger.error(`[CliToolsManager] Configuration failed for ${toolId}:`, error);
      throw error;
    }
  }

  /**
   * Configure Memory Service integration for a tool
   */
  private async configureMemoryServiceIntegration(toolId: string) {
    try {
      // Register with Memory Service
      const response = await fetch(`${this.memoryServiceEndpoint}/api/v1/memory/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          toolName: toolId,
          clientInfo: {
            version: await this.getToolVersion(toolId),
            platform: process.platform,
            nodeVersion: process.version
          }
        })
      });

      if (!response.ok) {
        throw new Error(`Memory Service registration failed: ${response.statusText}`);
      }

      const { token, endpoint } = await response.json();
      logger.info(`[CliToolsManager] Registered ${toolId} with Memory Service`);

      // Save the token for the tool
      const config = await this.loadConfig();
      config[toolId] = {
        ...config[toolId],
        memoryService: {
          endpoint,
          token,
          connectedAt: new Date().toISOString()
        }
      };
      await this.saveConfig(config);

      // Configure the tool to use Memory Service
      if (toolId === 'claude-code') {
        await this.updateClaudeCodeConfig({ memoryService: { endpoint, token, enabled: true } });
      }
    } catch (error) {
      logger.error(`[CliToolsManager] Memory Service integration failed:`, error);
      throw error;
    }
  }

  /**
   * Configure Claude Code specifically
   */
  private async configureClaudeCode() {
    const claudeConfigPath = path.join(os.homedir(), '.claude', 'config.json');
    
    try {
      // Ensure directory exists
      const claudeDir = path.dirname(claudeConfigPath);
      if (!fs.existsSync(claudeDir)) {
        await fsPromises.mkdir(claudeDir, { recursive: true });
      }

      // Load existing config or create new
      let claudeConfig: any = {};
      if (fs.existsSync(claudeConfigPath)) {
        const data = await fsPromises.readFile(claudeConfigPath, 'utf-8');
        claudeConfig = JSON.parse(data);
      }

      // Get Memory Service config
      const config = await this.loadConfig();
      const memoryConfig = config['claude-code']?.memoryService;

      // Update Claude config with Memory Service
      if (memoryConfig) {
        claudeConfig.memoryService = {
          endpoint: memoryConfig.endpoint,
          token: memoryConfig.token,
          enabled: true
        };
      }

      // Enable MCP if available
      claudeConfig.mcpServers = {
        ...claudeConfig.mcpServers,
        'hive-memory': {
          command: 'npx',
          args: ['@hive/mcp-memory-server', '--port', '3457']
        }
      };

      // Save Claude config
      await fsPromises.writeFile(claudeConfigPath, JSON.stringify(claudeConfig, null, 2));
      logger.info('[CliToolsManager] Claude Code configuration updated');
    } catch (error) {
      logger.error('[CliToolsManager] Failed to configure Claude Code:', error);
      throw error;
    }
  }

  /**
   * Update Claude Code configuration
   */
  private async updateClaudeCodeConfig(updates: any) {
    const claudeConfigPath = path.join(os.homedir(), '.claude', 'config.json');
    const mcpConfigPath = path.join(os.homedir(), '.claude', '.mcp.json');
    
    try {
      // Update main config
      let config = {};
      if (fs.existsSync(claudeConfigPath)) {
        const data = await fsPromises.readFile(claudeConfigPath, 'utf-8');
        config = JSON.parse(data);
      }

      // Merge updates
      config = { ...config, ...updates };

      // Ensure directory exists
      const dir = path.dirname(claudeConfigPath);
      if (!fs.existsSync(dir)) {
        await fsPromises.mkdir(dir, { recursive: true });
      }

      await fsPromises.writeFile(claudeConfigPath, JSON.stringify(config, null, 2));
      logger.info('[CliToolsManager] Claude Code config updated');

      // Update MCP configuration if Memory Service is enabled
      if (updates.memoryService && updates.memoryService.enabled) {
        let mcpConfig: any = { servers: {} };
        if (fs.existsSync(mcpConfigPath)) {
          const mcpData = await fsPromises.readFile(mcpConfigPath, 'utf-8');
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

        await fsPromises.writeFile(mcpConfigPath, JSON.stringify(mcpConfig, null, 2));
        logger.info('[CliToolsManager] MCP configuration updated with Memory Service');

        // Create the MCP wrapper script
        await this.createMemoryServiceMCPWrapper(updates.memoryService);
      }
    } catch (error) {
      logger.error('[CliToolsManager] Failed to update Claude Code config:', error);
    }
  }

  /**
   * Create MCP wrapper script for Memory Service
   */
  private async createMemoryServiceMCPWrapper(memoryService: any) {
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

    await fsPromises.writeFile(wrapperPath, wrapperScript, { mode: 0o755 });
    logger.info('[CliToolsManager] Created Memory Service MCP wrapper');
  }

  /**
   * Get tool status
   */
  public async getToolStatus(toolId: string): Promise<CliToolStatus> {
    if (toolId === 'claude-code') {
      return await detectClaudeCode();
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
      const { stdout } = await execAsync(tool.versionCommand + ' 2>/dev/null');
      const versionMatch = stdout.match(/(\d+\.\d+\.\d+)/);
      return {
        id: toolId,
        name: tool.name,
        installed: true,
        version: versionMatch ? versionMatch[1] : 'unknown'
      };
    } catch {
      return {
        id: toolId,
        name: tool.name,
        installed: false
      };
    }
  }

  /**
   * Get tool version
   */
  private async getToolVersion(toolId: string): Promise<string> {
    const status = await this.getToolStatus(toolId);
    return status.version || 'unknown';
  }

  /**
   * Check Node.js version
   */
  private async checkNodeVersion() {
    try {
      const { stdout } = await execAsync('node --version');
      const version = stdout.trim();
      const major = parseInt(version.split('.')[0].substring(1));
      if (major < 18) {
        throw new Error(`Node.js 18+ required (current: ${version})`);
      }
      logger.info(`[CliToolsManager] Node.js version OK: ${version}`);
    } catch (error) {
      throw new Error('Node.js 18+ is required for npm installations');
    }
  }

  /**
   * Check Python version
   */
  private async checkPythonVersion() {
    try {
      const { stdout } = await execAsync('python3 --version');
      const version = stdout.trim();
      logger.info(`[CliToolsManager] Python version OK: ${version}`);
    } catch {
      throw new Error('Python 3 is required for pip installations');
    }
  }

  /**
   * Execute command with progress tracking
   */
  private executeCommand(command: string): Promise<void> {
    return new Promise((resolve, reject) => {
      logger.info(`[CliToolsManager] Executing: ${command}`);
      
      execCallback(command, (error, stdout, stderr) => {
        if (error) {
          logger.error(`[CliToolsManager] Command failed: ${stderr}`);
          reject(error);
        } else {
          logger.info(`[CliToolsManager] Command succeeded`);
          resolve();
        }
      });
    });
  }

  /**
   * Save tool status to database
   */
  private async saveToDatabase(toolId: string, status: CliToolStatus): Promise<void> {
    if (!this.db) return;

    const syncType = `${toolId}_cli_update`;
    const now = new Date().toISOString();

    try {
      await new Promise((resolve, reject) => {
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
        ], (err: any) => {
          if (err) reject(err);
          else resolve(true);
        });
      });
    } catch (error) {
      logger.error('[CliToolsManager] Failed to save to database:', error);
    }
  }

  /**
   * Check for updates for all installed tools
   */
  public async checkForUpdates(): Promise<Map<string, boolean>> {
    const updates = new Map<string, boolean>();
    
    for (const [toolId, tool] of this.tools) {
      try {
        const status = await this.getToolStatus(toolId);
        if (status.installed) {
          // For npm packages, check latest version
          if (tool.installCommand.includes('npm')) {
            const packageName = tool.installCommand.match(/@[^@\s]+/)?.[0];
            if (packageName) {
              const { stdout } = await execAsync(`npm view ${packageName} version 2>/dev/null`);
              const latestVersion = stdout.trim();
              updates.set(toolId, latestVersion !== status.version);
            }
          }
        }
      } catch (error) {
        logger.error(`[CliToolsManager] Failed to check updates for ${toolId}:`, error);
      }
    }

    return updates;
  }

  /**
   * Launch a CLI tool in a specific project directory
   */
  public async launch(toolId: string, projectPath: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) {
      throw new Error(`Unknown tool: ${toolId}`);
    }

    logger.info(`[CliToolsManager] Launching ${tool.name} in ${projectPath}...`);

    // Check if tool is installed
    const status = await this.getToolStatus(toolId);
    if (!status.installed) {
      throw new Error(`${tool.name} is not installed. Please install it first.`);
    }

    // Special handling for Claude Code
    if (toolId === 'claude-code') {
      await this.launchClaudeCode(projectPath);
    } else {
      // Generic launch for other tools
      throw new Error(`Launch not yet implemented for ${tool.name}`);
    }
  }

  /**
   * Launch Claude Code in a specific project directory
   */
  private async launchClaudeCode(projectPath: string): Promise<void> {
    try {
      // Open a new terminal window and launch Claude Code
      // This uses platform-specific commands
      const platform = process.platform;
      let command: string;

      if (platform === 'darwin') {
        // macOS: Open Terminal and run claude
        command = `osascript -e 'tell application "Terminal" to do script "cd \\"${projectPath}\\" && claude"'`;
      } else if (platform === 'win32') {
        // Windows: Open Command Prompt and run claude
        command = `start cmd /k "cd /d ${projectPath} && claude"`;
      } else {
        // Linux: Try to open a terminal emulator
        // This is a best-effort attempt as terminal emulators vary
        command = `gnome-terminal -- bash -c "cd '${projectPath}' && claude; exec bash" || xterm -e "cd '${projectPath}' && claude; bash" || konsole -e "cd '${projectPath}' && claude"`;
      }

      logger.info(`[CliToolsManager] Executing launch command: ${command}`);
      await this.executeCommand(command);
      logger.info(`[CliToolsManager] Claude Code launched successfully in ${projectPath}`);
    } catch (error) {
      logger.error('[CliToolsManager] Failed to launch Claude Code:', error);
      throw new Error(`Failed to launch Claude Code: ${error}`);
    }
  }

  /**
   * Get all tools
   */
  public getAllTools(): Map<string, CliToolConfig> {
    return new Map(this.tools);
  }
}

export default CliToolsManager;