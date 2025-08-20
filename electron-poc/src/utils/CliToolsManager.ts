/**
 * CLI Tools Manager
 * Handles installation, updates, and maintenance of AI CLI tools
 * Primary focus: Claude Code CLI integration with Memory Service
 */

import { exec as execCallback } from 'child_process';
import { promisify } from 'util';
import * as fs from 'fs';
import * as path from 'path';
import * as os from 'os';
import fetch from 'node-fetch';
import { EventEmitter } from 'events';

const exec = promisify(execCallback);

// Tool status tracking
export interface ToolStatus {
  installed: boolean;
  version?: string;
  path?: string;
  lastChecked?: Date;
  updateAvailable?: boolean;
  authenticated?: boolean;
}

// Tool configuration
export interface CliToolConfig {
  id: string;
  name: string;
  command: string;
  npmPackage?: string;
  checkCommand: string;
  versionCommand: string;
  authCheckCommand?: string;
  installCommand: string;
  updateCheckInterval: number; // hours
  memoryServiceIntegration?: boolean;
}

// Installation progress events
export interface InstallProgress {
  tool: string;
  status: 'checking' | 'downloading' | 'installing' | 'configuring' | 'complete' | 'error';
  progress?: number;
  message?: string;
  error?: Error;
}

export class CliToolsManager extends EventEmitter {
  private toolsDir: string;
  private configPath: string;
  private tools: Map<string, CliToolConfig>;
  private status: Map<string, ToolStatus>;
  private updateCheckTimers: Map<string, NodeJS.Timer>;
  private db: any; // SQLite database connection

  constructor(database: any) {
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
  private initializeTools() {
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
  private async loadStatus() {
    try {
      if (fs.existsSync(this.configPath)) {
        const data = fs.readFileSync(this.configPath, 'utf-8');
        const saved = JSON.parse(data);
        
        for (const [toolId, status] of Object.entries(saved)) {
          this.status.set(toolId, status as ToolStatus);
        }
      }
    } catch (error) {
      console.error('[CliToolsManager] Failed to load status:', error);
    }
  }

  /**
   * Save status to disk
   */
  private async saveStatus() {
    try {
      const data = Object.fromEntries(this.status);
      fs.mkdirSync(path.dirname(this.configPath), { recursive: true });
      fs.writeFileSync(this.configPath, JSON.stringify(data, null, 2));
    } catch (error) {
      console.error('[CliToolsManager] Failed to save status:', error);
    }
  }

  /**
   * Check if a tool is installed
   */
  public async checkInstalled(toolId: string): Promise<boolean> {
    const tool = this.tools.get(toolId);
    if (!tool) return false;

    try {
      await exec(tool.checkCommand);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get tool version
   */
  public async getVersion(toolId: string): Promise<string | undefined> {
    const tool = this.tools.get(toolId);
    if (!tool) return undefined;

    try {
      const { stdout } = await exec(tool.versionCommand);
      const version = stdout.trim().match(/\d+\.\d+\.\d+/)?.[0];
      return version;
    } catch {
      return undefined;
    }
  }

  /**
   * Check if tool is authenticated
   */
  public async checkAuthenticated(toolId: string): Promise<boolean> {
    const tool = this.tools.get(toolId);
    if (!tool || !tool.authCheckCommand) return true;

    try {
      await exec(tool.authCheckCommand);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Install a tool
   */
  public async install(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) throw new Error(`Unknown tool: ${toolId}`);

    this.emit('install-progress', {
      tool: toolId,
      status: 'checking',
      message: `Checking ${tool.name} installation...`
    } as InstallProgress);

    // Check if already installed
    const installed = await this.checkInstalled(toolId);
    if (installed) {
      this.emit('install-progress', {
        tool: toolId,
        status: 'complete',
        message: `${tool.name} is already installed`
      } as InstallProgress);
      
      await this.updateStatus(toolId);
      return;
    }

    // Check dependencies
    if (toolId === 'gh-copilot') {
      // Check if gh CLI is installed first
      try {
        await exec('gh --version');
      } catch {
        throw new Error('GitHub CLI (gh) must be installed first. Run: brew install gh');
      }
    }

    this.emit('install-progress', {
      tool: toolId,
      status: 'installing',
      message: `Installing ${tool.name}...`
    } as InstallProgress);

    try {
      // Run installation command
      await exec(tool.installCommand);
      
      // Verify installation
      const nowInstalled = await this.checkInstalled(toolId);
      if (!nowInstalled) {
        throw new Error('Installation verification failed');
      }

      this.emit('install-progress', {
        tool: toolId,
        status: 'complete',
        message: `${tool.name} installed successfully`
      } as InstallProgress);

      await this.updateStatus(toolId);
      
      // If Claude CLI, configure Memory Service integration
      if (toolId === 'claude' && tool.memoryServiceIntegration) {
        await this.configureMemoryServiceIntegration(toolId);
      }
      
    } catch (error) {
      this.emit('install-progress', {
        tool: toolId,
        status: 'error',
        message: `Failed to install ${tool.name}`,
        error: error as Error
      } as InstallProgress);
      throw error;
    }
  }

  /**
   * Configure Memory Service integration for Claude CLI
   */
  private async configureMemoryServiceIntegration(toolId: string): Promise<void> {
    if (toolId !== 'claude') return;

    this.emit('install-progress', {
      tool: toolId,
      status: 'configuring',
      message: 'Configuring Memory Service integration...'
    } as InstallProgress);

    try {
      // Register Claude CLI with Memory Service
      const response = await fetch('http://localhost:3457/api/v1/memory/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ toolName: 'Claude Code CLI' })
      });

      if (response.ok) {
        const { token } = await response.json();
        
        // Save token to Claude CLI config
        const claudeConfigPath = path.join(os.homedir(), '.claude', 'config.json');
        let claudeConfig: any = {};
        
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
    } catch (error) {
      console.error('[CliToolsManager] Failed to configure Memory Service:', error);
    }
  }

  /**
   * Update tool status
   */
  private async updateStatus(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) return;

    const status: ToolStatus = {
      installed: await this.checkInstalled(toolId),
      version: await this.getVersion(toolId),
      lastChecked: new Date(),
      authenticated: await this.checkAuthenticated(toolId)
    };

    // Find tool path
    try {
      const { stdout } = await exec(`which ${tool.command} || where ${tool.command}`);
      status.path = stdout.trim().split('\n')[0];
    } catch {
      // Path not found
    }

    this.status.set(toolId, status);
    await this.saveStatus();
    
    // Save to database for tracking
    await this.saveToDatabase(toolId, status);
  }

  /**
   * Save tool status to database
   */
  private async saveToDatabase(toolId: string, status: ToolStatus): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool || !this.db) return;

    const syncType = `${toolId}_cli_update`;
    const now = new Date().toISOString();
    const nextCheck = new Date(Date.now() + tool.updateCheckInterval * 60 * 60 * 1000).toISOString();

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
          nextCheck,
          now
        ], (err: any) => {
          if (err) reject(err);
          else resolve(true);
        });
      });
    } catch (error) {
      console.error('[CliToolsManager] Failed to save to database:', error);
    }
  }

  /**
   * Check for updates for a tool
   */
  public async checkForUpdates(toolId: string): Promise<boolean> {
    const tool = this.tools.get(toolId);
    if (!tool) return false;

    try {
      if (tool.npmPackage) {
        // Check npm for latest version
        const { stdout } = await exec(`npm view ${tool.npmPackage} version`);
        const latestVersion = stdout.trim();
        const currentVersion = await this.getVersion(toolId);
        
        if (currentVersion && latestVersion !== currentVersion) {
          const status = this.status.get(toolId) || {} as ToolStatus;
          status.updateAvailable = true;
          this.status.set(toolId, status);
          await this.saveStatus();
          return true;
        }
      }
    } catch (error) {
      console.error(`[CliToolsManager] Failed to check updates for ${toolId}:`, error);
    }

    return false;
  }

  /**
   * Update a tool
   */
  public async update(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) throw new Error(`Unknown tool: ${toolId}`);

    this.emit('install-progress', {
      tool: toolId,
      status: 'downloading',
      message: `Updating ${tool.name}...`
    } as InstallProgress);

    try {
      if (tool.npmPackage) {
        await exec(`npm update -g ${tool.npmPackage}`);
      }
      
      await this.updateStatus(toolId);
      
      this.emit('install-progress', {
        tool: toolId,
        status: 'complete',
        message: `${tool.name} updated successfully`
      } as InstallProgress);
    } catch (error) {
      this.emit('install-progress', {
        tool: toolId,
        status: 'error',
        message: `Failed to update ${tool.name}`,
        error: error as Error
      } as InstallProgress);
      throw error;
    }
  }

  /**
   * Start automatic update checking
   */
  public startAutoUpdateCheck(): void {
    for (const [toolId, tool] of this.tools) {
      // Clear existing timer
      const existingTimer = this.updateCheckTimers.get(toolId);
      if (existingTimer) {
        clearInterval(existingTimer);
      }

      // Set up new timer
      const timer = setInterval(async () => {
        const hasUpdate = await this.checkForUpdates(toolId);
        if (hasUpdate) {
          this.emit('update-available', { toolId, tool: tool.name });
        }
      }, tool.updateCheckInterval * 60 * 60 * 1000);

      this.updateCheckTimers.set(toolId, timer);
      
      // Also check immediately
      this.checkForUpdates(toolId);
    }
  }

  /**
   * Stop automatic update checking
   */
  public stopAutoUpdateCheck(): void {
    for (const timer of this.updateCheckTimers.values()) {
      clearInterval(timer);
    }
    this.updateCheckTimers.clear();
  }

  /**
   * Get all tool statuses
   */
  public async getAllStatuses(): Promise<Map<string, ToolStatus>> {
    const statuses = new Map<string, ToolStatus>();
    
    for (const toolId of this.tools.keys()) {
      await this.updateStatus(toolId);
      const status = this.status.get(toolId);
      if (status) {
        statuses.set(toolId, status);
      }
    }
    
    return statuses;
  }

  /**
   * Get tool configuration
   */
  public getTool(toolId: string): CliToolConfig | undefined {
    return this.tools.get(toolId);
  }

  /**
   * Get all tools
   */
  public getAllTools(): Map<string, CliToolConfig> {
    return new Map(this.tools);
  }

  /**
   * Uninstall a tool
   */
  public async uninstall(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) throw new Error(`Unknown tool: ${toolId}`);

    this.emit('install-progress', {
      tool: toolId,
      status: 'uninstalling',
      message: `Uninstalling ${tool.name}...`
    } as InstallProgress);

    try {
      // Check if tool is installed
      const installed = await this.checkInstalled(toolId);
      if (!installed) {
        throw new Error(`${tool.name} is not installed`);
      }

      // Uninstall based on package manager
      if (tool.npmPackage) {
        await exec(`npm uninstall -g ${tool.npmPackage}`);
      } else if (tool.command === 'aider') {
        await exec('pip uninstall -y aider-chat');
      } else {
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
        await this.saveStatus();
      }

      // Remove from database
      await this.removeFromDatabase(toolId);

      this.emit('install-progress', {
        tool: toolId,
        status: 'complete',
        message: `${tool.name} uninstalled successfully`
      } as InstallProgress);

    } catch (error) {
      this.emit('install-progress', {
        tool: toolId,
        status: 'error',
        message: `Failed to uninstall ${tool.name}`,
        error: error as Error
      } as InstallProgress);
      throw error;
    }
  }

  /**
   * Remove tool from database
   */
  private async removeFromDatabase(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool || !this.db) return;

    try {
      await new Promise((resolve, reject) => {
        this.db.run(`
          DELETE FROM sync_metadata 
          WHERE sync_type = ?
        `, [tool.syncType || `${toolId}_cli_update`], (err: any) => {
          if (err) reject(err);
          else resolve(true);
        });
      });
    } catch (error) {
      console.error('[CliToolsManager] Failed to remove from database:', error);
    }
  }

  /**
   * Cancel an installation
   */
  public cancelInstallation(toolId: string): void {
    // TODO: Implement cancellation logic
    console.log(`[CliToolsManager] Cancelling installation of ${toolId}`);
    
    const status = this.status.get(toolId);
    if (status) {
      delete status.installProgress;
      delete status.installMessage;
      this.status.set(toolId, status);
    }

    this.emit('install-progress', {
      tool: toolId,
      status: 'cancelled',
      message: 'Installation cancelled'
    } as InstallProgress);
  }

  /**
   * Get installation logs
   */
  public getInstallationLogs(toolId: string): string[] {
    // TODO: Implement log storage and retrieval
    return [`No logs available for ${toolId}`];
  }

  /**
   * Configure tool authentication
   */
  public async configureTool(toolId: string): Promise<void> {
    const tool = this.tools.get(toolId);
    if (!tool) throw new Error(`Unknown tool: ${toolId}`);

    // Special handling for Claude CLI
    if (toolId === 'claude' && tool.memoryServiceIntegration) {
      await this.configureMemoryServiceIntegration(toolId);
    }

    // TODO: Add configuration for other tools
  }

  /**
   * Check all tools for updates
   */
  public async checkAllUpdates(): Promise<Map<string, boolean>> {
    const updates = new Map<string, boolean>();
    
    for (const toolId of this.tools.keys()) {
      const hasUpdate = await this.checkForUpdates(toolId);
      updates.set(toolId, hasUpdate);
    }
    
    return updates;
  }

  /**
   * Update advanced settings
   */
  public updateSettings(settings: any): void {
    // Update internal settings
    // This would typically update configuration that affects installation behavior
    console.log('[CliToolsManager] Settings updated:', settings);
  }
}

export default CliToolsManager;