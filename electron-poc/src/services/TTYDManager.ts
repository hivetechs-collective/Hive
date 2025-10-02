/**
 * TTYDManager - Manages ttyd terminal server instances for tabbed terminals
 * Works with ProcessManager for port allocation and process lifecycle
 * Updated: 12:49 PM - Added debug logging
 */

import { ChildProcess, spawn } from 'child_process';
import { EventEmitter } from 'events';
import { PortManager } from '../utils/PortManager';
import { PidTracker } from '../utils/PidTracker';
import { logger } from '../utils/SafeLogger';
import ProcessManager from '../utils/ProcessManager';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { ProductionPaths } from '../utils/ProductionPaths';

export interface TTYDConfig {
  id: string;
  title: string;
  toolId?: string;
  cwd?: string;
  env?: NodeJS.ProcessEnv;
  command?: string;  // Command to auto-execute (e.g., 'claude')
  shell?: string;    // Shell to use (default: /bin/zsh)
}

export interface TTYDInstance {
  id: string;
  title: string;
  toolId?: string;
  port: number;
  url: string;
  process: ChildProcess;
  status: 'starting' | 'running' | 'stopping' | 'stopped';
  createdAt: Date;
  config: TTYDConfig;
  terminalNumber?: number;  // Track terminal number for reuse
}

export class TTYDManager extends EventEmitter {
  private instances: Map<string, TTYDInstance> = new Map();
  private processManager: ProcessManager;
  private ttydBinaryPath: string = process.env.TTYD_BINARY_PATH || '';
  
  constructor(processManager: ProcessManager) {
    super();
    this.processManager = processManager;
    // Resolve ttyd path up front so we favor the bundled binary in production
    this.resolveTTYDPath();
    this.verifyTTYDInstalled();
  }
  
  /**
   * Determine the most appropriate ttyd binary path, preferring the bundled copy
   */
  private resolveTTYDPath(): void {
    const candidates: string[] = [];
    try {
      // 1) Bundled binary inside packaged app (preferred)
      const bundled = ProductionPaths.getBinaryPath('ttyd');
      candidates.push(bundled);
    } catch {}
    // 2) Explicit override via env
    if (process.env.TTYD_BINARY_PATH) candidates.push(process.env.TTYD_BINARY_PATH);
    // 3) Common system locations
    candidates.push(
      '/opt/homebrew/bin/ttyd',
      '/usr/local/bin/ttyd',
      '/usr/bin/ttyd'
    );

    for (const p of candidates) {
      try {
        if (fs.existsSync(p)) {
          fs.accessSync(p, fs.constants.X_OK);
          this.ttydBinaryPath = p;
          logger.info(`[TTYDManager] Using ttyd at: ${this.ttydBinaryPath}`);
          return;
        }
      } catch {
        // try next
      }
    }

    // If nothing found, leave blank; verifyTTYDInstalled() will warn
    this.ttydBinaryPath = this.ttydBinaryPath || '/opt/homebrew/bin/ttyd';
  }

  /**
   * Verify ttyd is installed
   */
  private async verifyTTYDInstalled(): Promise<boolean> {
    try {
      const fs = require('fs');
      // Check if ttyd binary exists at the specified path
      if (fs.existsSync(this.ttydBinaryPath)) {
        logger.info(`[TTYDManager] ttyd binary found at: ${this.ttydBinaryPath}`);
        return true;
      } else {
        logger.error(`[TTYDManager] ttyd not found. Last tried path: ${this.ttydBinaryPath}`);
        logger.error('[TTYDManager] Please install: brew install ttyd');
        // Don't emit error - just warn and continue without terminal functionality
        logger.warn('[TTYDManager] Terminal functionality will be disabled');
        return false;
      }
    } catch (error) {
      logger.error(`[TTYDManager] Error checking ttyd: ${error.message}`);
      logger.warn('[TTYDManager] Terminal functionality will be disabled');
      return false;
    }
  }
  
  /**
   * Create a new ttyd terminal instance
   */
  async createTerminal(config: TTYDConfig): Promise<TTYDInstance> {
    logger.info(`[TTYDManager] Creating terminal: ${config.title}`);

    // Compute enhanced PATH: prefer packaged binaries, then Hive-managed bins, then system
    const buildEnhancedPath = (): string => {
      let packagedBinDir: string | null = null;
      try {
        const packagedNpm = ProductionPaths.getBinaryPath('npm');
        if (fs.existsSync(packagedNpm)) packagedBinDir = path.dirname(packagedNpm);
      } catch {}
      const home = os.homedir();
      const additions = [
        ...(packagedBinDir ? [packagedBinDir] : []),
        path.join(home, '.hive', 'npm-global', 'bin'),
        path.join(home, '.hive', 'cli-bin'),
        path.join(home, '.local', 'bin'),
        '/opt/homebrew/bin',
        '/usr/local/bin',
        '/usr/bin',
        '/bin',
      ];
      const current = (config.env?.PATH || process.env.PATH || '').split(path.delimiter);
      const all = [...new Set([...additions, ...current])].filter(Boolean);
      return all.join(path.delimiter);
    };
    
    // Allocate port through ProcessManager's PortManager
    // No hardcoded ports - PortManager handles allocation from configured pools
    const port = await PortManager.allocatePortForService(`ttyd-${config.id}`);
    
    if (!port) {
      throw new Error(`No ports available for terminal ${config.id}`);
    }
    
    logger.info(`[TTYDManager] Allocated port ${port} for ${config.title}`);
    
    // Prepare ttyd arguments
    const ttydArgs = [
      '--port', port.toString(),
      '--interface', '127.0.0.1',  // Bind to localhost only for security
      '--writable',              // Allow input
      // Note: --check-origin is a flag, not a key-value option
      // '--base-path', `/terminal/${config.id}`,  // Commented out - may interfere with routing
      // '--title', config.title  // Title doesn't exist as an option in ttyd
    ];
    
    // Add authentication if needed (for security in production)
    if (process.env.NODE_ENV === 'production') {
      // In production, we might want to add basic auth
      // ttydArgs.push('--credential', 'user:pass');
    }
    
    // Determine shell and initial command
    const shell = config.shell || '/bin/zsh';
    const shellArgs: string[] = [];
    
    // If we have a command to auto-execute (like 'claude' or 'copilot'), prepare it
    if (config.command) {
      // For commands with spaces, we need to handle them carefully
      // Check if the command contains spaces that need special handling
      if (config.command.includes(' ') && !config.command.startsWith('cd ')) {
        // For multi-word commands, execute directly then drop to interactive shell
        // This avoids shell quoting issues
        ttydArgs.push('--', shell, '-c', `${config.command}; exec ${shell} -i`);
      } else {
        // For single-word commands, use the sleep wrapper for better UX
        const initCommand = `sleep 0.5 && ${config.command}`;
        ttydArgs.push('--', shell, '-c', `${initCommand}; exec ${shell} -i`);
      }
    } else {
      // Just start the shell normally
      ttydArgs.push('--', shell);
    }
    
    // Log the full command for debugging
    logger.info(`[TTYDManager] Spawning ttyd: ${this.ttydBinaryPath} ${ttydArgs.join(' ')}`);
    console.log(`[TTYDManager] Spawning ttyd: ${this.ttydBinaryPath} ${ttydArgs.join(' ')}`);
    
    // Spawn ttyd process with proper terminal dimensions
    const ttydProcess = spawn(this.ttydBinaryPath, ttydArgs, {
      cwd: config.cwd || process.env.HOME,
      env: {
        ...process.env,
        ...config.env,
        PATH: buildEnhancedPath(),
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor',
        // Force proper terminal dimensions to prevent 9-row issue
        LINES: '50',
        COLUMNS: '150',
        // Disable zsh's % marker for cleaner output
        PROMPT_EOL_MARK: ''
      },
      detached: false,
      // Set PTY size if available
      windowsHide: true
    });
    
    // Track the PID for cleanup
    if (ttydProcess.pid) {
      PidTracker.addPid(ttydProcess.pid, `ttyd-${config.id}`);
    }
    
    // Create instance object
    const instance: TTYDInstance = {
      id: config.id,
      title: config.title,
      toolId: config.toolId,
      port,
      url: `http://localhost:${port}`,
      process: ttydProcess,
      status: 'starting',
      createdAt: new Date(),
      config
    };
    
    // Store instance
    this.instances.set(config.id, instance);
    
    // Set up process event handlers
    ttydProcess.on('error', (error) => {
      logger.error(`[TTYDManager] Terminal ${config.title} error:`, error);
      instance.status = 'stopped';
      this.emit('terminal:error', config.id, error);
      this.cleanupTerminal(config.id);
    });
    
    ttydProcess.on('exit', (code, signal) => {
      logger.info(`[TTYDManager] Terminal ${config.title} exited (code: ${code}, signal: ${signal})`);
      instance.status = 'stopped';
      this.emit('terminal:closed', config.id);
      this.cleanupTerminal(config.id);
    });
    
    // Capture ttyd output for debugging
    if (ttydProcess.stdout) {
      ttydProcess.stdout.on('data', (data) => {
        logger.debug(`[TTYDManager] ${config.title} stdout:`, data.toString());
      });
    }
    
    if (ttydProcess.stderr) {
      ttydProcess.stderr.on('data', (data) => {
        // ttyd logs to stderr, but most of it is just info
        const message = data.toString();
        // Always log stderr for debugging ttyd startup issues
        logger.error(`[TTYDManager] ${config.title} stderr:`, message);
        console.error(`[TTYDManager] ${config.title} stderr:`, message);
        if (message.includes('error') || message.includes('Error')) {
          logger.error(`[TTYDManager] ${config.title} stderr:`, message);
        } else {
          logger.debug(`[TTYDManager] ${config.title} info:`, message);
        }
      });
    }
    
    // Wait for ttyd to be ready (port to be listening)
    const isReady = await this.waitForTerminalReady(port, config.title);
    
    if (isReady) {
      instance.status = 'running';
      this.emit('terminal:ready', config.id, instance);
      logger.info(`[TTYDManager] Terminal ${config.title} is ready at ${instance.url}`);
      
      // If we have a command to execute, send it after a short delay
      if (config.command) {
        setTimeout(() => {
          this.executeCommand(config.id, config.command!);
        }, 500);
      }
    } else {
      logger.error(`[TTYDManager] Terminal ${config.title} failed to start`);
      await this.closeTerminal(config.id);
      throw new Error(`Failed to start terminal ${config.title}`);
    }
    
    return instance;
  }
  
  /**
   * Wait for ttyd to be ready on the specified port
   */
  private async waitForTerminalReady(port: number, title: string): Promise<boolean> {
    const maxAttempts = 30;  // 3 seconds total
    const checkInterval = 100;  // Check every 100ms
    
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      const isListening = await PortManager.waitForService(port, checkInterval);
      
      if (isListening) {
        logger.info(`[TTYDManager] Port ${port} is ready for ${title}`);
        return true;
      }
      
      if (attempt === maxAttempts / 2) {
        logger.info(`[TTYDManager] Still waiting for ${title} on port ${port}...`);
      }
    }
    
    logger.error(`[TTYDManager] Timeout waiting for ${title} on port ${port}`);
    return false;
  }
  
  /**
   * Execute a command in a terminal (via JavaScript injection)
   * Note: This requires the webview to call this after it's loaded
   */
  executeCommand(terminalId: string, command: string): void {
    const instance = this.instances.get(terminalId);
    if (!instance) {
      logger.error(`[TTYDManager] Terminal ${terminalId} not found`);
      return;
    }
    
    // Emit event for the renderer to handle
    // The renderer's webview will execute JavaScript to send the command
    this.emit('terminal:execute', terminalId, command);
  }
  
  /**
   * Close a terminal instance
   */
  async closeTerminal(id: string): Promise<boolean> {
    const instance = this.instances.get(id);
    if (!instance) {
      return true;
    }
    
    logger.info(`[TTYDManager] Closing terminal: ${instance.title}`);
    instance.status = 'stopping';
    
    // Kill the ttyd process
    if (instance.process && !instance.process.killed) {
      instance.process.kill('SIGTERM');
      
      // Give it time to shutdown gracefully
      await new Promise(resolve => setTimeout(resolve, 500));
      
      // Force kill if still running
      if (!instance.process.killed) {
        instance.process.kill('SIGKILL');
      }
    }
    
    // Cleanup
    await this.cleanupTerminal(id);
    
    return true;
  }
  
  /**
   * Clean up terminal resources
   */
  private async cleanupTerminal(id: string): Promise<void> {
    const instance = this.instances.get(id);
    if (!instance) {
      return;
    }
    
    // Remove PID tracking
    if (instance.process && instance.process.pid) {
      PidTracker.removePid(instance.process.pid);
    }
    
    // Release the port
    if (instance.port) {
      PortManager.releasePort(`ttyd-${id}`);
      logger.info(`[TTYDManager] Released port ${instance.port} for ${instance.title}`);
    }
    
    // Remove from instances map
    this.instances.delete(id);
    
    this.emit('terminal:cleaned', id);
  }
  
  /**
   * Get a terminal instance
   */
  getTerminal(id: string): TTYDInstance | undefined {
    return this.instances.get(id);
  }
  
  /**
   * Get all terminal instances
   */
  getAllTerminals(): TTYDInstance[] {
    return Array.from(this.instances.values());
  }
  
  /**
   * Check if a terminal is running
   */
  isTerminalRunning(id: string): boolean {
    const instance = this.instances.get(id);
    return instance?.status === 'running';
  }
  
  /**
   * Get terminal by tool ID
   */
  getTerminalByToolId(toolId: string): TTYDInstance | undefined {
    return Array.from(this.instances.values()).find(
      instance => instance.toolId === toolId
    );
  }
  
  /**
   * Restart a terminal
   */
  async restartTerminal(id: string): Promise<TTYDInstance | null> {
    const instance = this.instances.get(id);
    if (!instance) {
      return null;
    }
    
    const config = instance.config;
    await this.closeTerminal(id);
    
    // Wait a bit before restarting
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    return this.createTerminal(config);
  }
  
  /**
   * Clean up all terminals
   */
  async cleanup(): Promise<void> {
    logger.info('[TTYDManager] Cleaning up all terminals...');
    
    const closePromises = Array.from(this.instances.keys()).map(id =>
      this.closeTerminal(id)
    );
    
    await Promise.all(closePromises);
    
    this.instances.clear();
    logger.info('[TTYDManager] All terminals cleaned up');
  }
  
  /**
   * Get status of all terminals
   */
  getStatus(): {
    total: number;
    running: number;
    stopped: number;
    terminals: Array<{
      id: string;
      title: string;
      toolId?: string;
      port: number;
      status: string;
      url: string;
      uptime: number;
    }>;
  } {
    const terminals = Array.from(this.instances.values()).map(instance => ({
      id: instance.id,
      title: instance.title,
      toolId: instance.toolId,
      port: instance.port,
      status: instance.status,
      url: instance.url,
      uptime: Date.now() - instance.createdAt.getTime()
    }));
    
    return {
      total: terminals.length,
      running: terminals.filter(t => t.status === 'running').length,
      stopped: terminals.filter(t => t.status === 'stopped').length,
      terminals
    };
  }
}

export default TTYDManager;
