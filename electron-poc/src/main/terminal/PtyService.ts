/**
 * PtyService - Production-ready PTY process management service
 * Manages node-pty terminal processes with comprehensive event handling
 *
 * This service provides a clean abstraction over node-pty for spawning and managing
 * shell processes in Electron's main process. It handles terminal lifecycle,
 * data flow, resizing, and cleanup with proper error handling.
 */

import { IPty, spawn } from 'node-pty';
import { EventEmitter } from 'events';
import * as os from 'os';
import * as path from 'path';

/**
 * Options for spawning a new PTY terminal
 */
export interface PtySpawnOptions {
  /** Terminal ID (generated if not provided) */
  id?: string;
  /** Terminal title */
  title?: string;
  /** Shell command to execute (e.g., 'bash', 'zsh', 'powershell.exe') */
  shell?: string;
  /** Command-line arguments for the shell */
  args?: string[];
  /** Environment variables to pass to the shell */
  env?: Record<string, string>;
  /** Working directory for the shell process */
  cwd?: string;
  /** Initial terminal dimensions */
  cols?: number;
  rows?: number;
  /** Tool ID for AI CLI tools */
  toolId?: string;
  /** Terminal number for display */
  terminalNumber?: number;
}

/**
 * Terminal instance data stored by PtyService
 */
export interface PtyTerminal {
  /** Unique terminal identifier */
  id: string;
  /** Terminal title */
  title: string;
  /** node-pty IPty interface */
  pty: IPty;
  /** Shell command being executed */
  shell: string;
  /** Current working directory */
  cwd: string;
  /** Terminal columns */
  cols: number;
  /** Terminal rows */
  rows: number;
  /** Creation timestamp */
  createdAt: Date;
  /** Tool ID for AI CLI tools */
  toolId?: string;
  /** Terminal number for display */
  terminalNumber?: number;
}

/**
 * Terminal info returned by listTerminals (without PTY instance)
 */
export interface PtyTerminalInfo {
  terminalId: string;
  title: string;
  shell: string;
  cwd: string;
  cols: number;
  rows: number;
  createdAt: Date;
  toolId?: string;
  terminalNumber?: number;
}

/**
 * Event types emitted by PtyService
 */
export interface PtyServiceEvents {
  /** Emitted when data is received from a terminal */
  data: (terminalId: string, data: string) => void;
  /** Emitted when a terminal process exits */
  exit: (terminalId: string, exitCode: number, signal?: number) => void;
  /** Emitted when an error occurs */
  error: (terminalId: string, error: Error) => void;
}

/**
 * PtyService - Manages PTY terminal processes
 *
 * This service provides a centralized way to spawn, manage, and communicate with
 * PTY-based terminal processes. It handles the lifecycle of terminals including
 * creation, data flow, resizing, and cleanup.
 *
 * @example
 * ```typescript
 * const ptyService = new PtyService();
 *
 * // Spawn a new terminal
 * const terminalId = ptyService.spawn({
 *   shell: '/bin/zsh',
 *   cwd: '/Users/username/projects',
 *   env: { TERM: 'xterm-256color' },
 *   cols: 80,
 *   rows: 24
 * });
 *
 * // Listen for data
 * ptyService.onData(terminalId, (data) => {
 *   console.log('Terminal output:', data);
 * });
 *
 * // Write to terminal
 * ptyService.write(terminalId, 'ls -la\r');
 *
 * // Resize terminal
 * ptyService.resize(terminalId, 120, 30);
 *
 * // Kill terminal
 * ptyService.kill(terminalId);
 * ```
 */
export class PtyService extends EventEmitter {
  /** Map of terminal ID to PtyTerminal instance */
  private terminals: Map<string, PtyTerminal> = new Map();

  /** Logger function (can be customized) */
  private log: (message: string, ...args: any[]) => void;

  /**
   * Initialize PtyService
   * @param logger - Optional custom logger function
   */
  constructor(logger?: (message: string, ...args: any[]) => void) {
    super();
    this.log = logger || console.log;
    this.log('[PtyService] Initialized');
  }

  /**
   * Spawn a new PTY terminal process
   *
   * @param options - Configuration options for the terminal
   * @returns Terminal instance
   * @throws Error if spawn fails or invalid parameters provided
   *
   * @example
   * ```typescript
   * const terminal = await ptyService.spawn({
   *   id: 'my-terminal',
   *   title: 'My Terminal',
   *   shell: '/bin/bash',
   *   args: ['--login'],
   *   cwd: process.env.HOME,
   *   env: { TERM: 'xterm-256color' },
   *   cols: 80,
   *   rows: 24
   * });
   * ```
   */
  async spawn(options: PtySpawnOptions = {}): Promise<PtyTerminal> {
    try {
      // Use provided ID or generate unique terminal ID
      const terminalId = options.id || this.generateTerminalId();

      // Terminal title
      const title = options.title || `Terminal ${terminalId}`;

      // Determine shell (with platform-specific defaults)
      const shell = options.shell || this.getDefaultShell();

      // Prepare environment variables
      const env = {
        ...process.env,
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor',
        ...options.env
      };

      // Determine working directory
      const cwd = options.cwd || process.env.HOME || process.cwd();

      // Terminal dimensions (defaults to 80x24)
      const cols = options.cols || 80;
      const rows = options.rows || 24;

      // Spawn PTY process using node-pty
      this.log(`[PtyService] Spawning terminal ${terminalId}:`, {
        title,
        shell,
        args: options.args,
        cwd,
        cols,
        rows
      });

      const pty = spawn(shell, options.args || [], {
        name: 'xterm-256color',
        cols,
        rows,
        cwd,
        env: env as { [key: string]: string },
        encoding: 'utf8'
      });

      // Store terminal instance
      const terminal: PtyTerminal = {
        id: terminalId,
        title,
        pty,
        shell,
        cwd,
        cols,
        rows,
        createdAt: new Date(),
        toolId: options.toolId,
        terminalNumber: options.terminalNumber
      };

      this.terminals.set(terminalId, terminal);

      // Set up event handlers
      this.setupPtyEventHandlers(terminalId, pty);

      this.log(`[PtyService] Terminal ${terminalId} spawned successfully (PID: ${pty.pid})`);

      return terminal;

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.log(`[PtyService] Failed to spawn terminal:`, errorMessage);
      throw new Error(`Failed to spawn terminal: ${errorMessage}`);
    }
  }

  /**
   * Write data to a terminal
   *
   * @param terminalId - ID of the terminal to write to
   * @param data - Data to write (typically user input)
   * @throws Error if terminal not found
   *
   * @example
   * ```typescript
   * // Send a command
   * await ptyService.write(terminalId, 'ls -la\r');
   *
   * // Send Ctrl+C
   * await ptyService.write(terminalId, '\x03');
   * ```
   */
  async write(terminalId: string, data: string): Promise<void> {
    const terminal = this.terminals.get(terminalId);

    if (!terminal) {
      const error = new Error(`Terminal ${terminalId} not found`);
      this.log(`[PtyService] Write failed:`, error.message);
      throw error;
    }

    try {
      terminal.pty.write(data);
      // Debug logging can be verbose, only log on errors or important events
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.log(`[PtyService] Write error for terminal ${terminalId}:`, errorMessage);
      this.emit('error', terminalId, error instanceof Error ? error : new Error(errorMessage));
      throw error;
    }
  }

  /**
   * Resize a terminal
   *
   * @param terminalId - ID of the terminal to resize
   * @param cols - New column count
   * @param rows - New row count
   * @throws Error if terminal not found or invalid dimensions
   *
   * @example
   * ```typescript
   * await ptyService.resize(terminalId, 120, 30);
   * ```
   */
  async resize(terminalId: string, cols: number, rows: number): Promise<void> {
    const terminal = this.terminals.get(terminalId);

    if (!terminal) {
      const error = new Error(`Terminal ${terminalId} not found`);
      this.log(`[PtyService] Resize failed:`, error.message);
      throw error;
    }

    // Validate dimensions
    if (cols < 1 || rows < 1) {
      const error = new Error(`Invalid terminal dimensions: ${cols}x${rows}`);
      this.log(`[PtyService] Resize failed:`, error.message);
      throw error;
    }

    try {
      terminal.pty.resize(cols, rows);
      terminal.cols = cols;
      terminal.rows = rows;
      this.log(`[PtyService] Terminal ${terminalId} resized to ${cols}x${rows}`);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.log(`[PtyService] Resize error for terminal ${terminalId}:`, errorMessage);
      this.emit('error', terminalId, error instanceof Error ? error : new Error(errorMessage));
      throw error;
    }
  }

  /**
   * Kill a terminal process
   *
   * @param terminalId - ID of the terminal to kill
   * @param signal - Optional signal to send (default: SIGHUP)
   * @throws Error if terminal not found
   *
   * @example
   * ```typescript
   * // Graceful shutdown
   * await ptyService.kill(terminalId);
   *
   * // Force kill
   * await ptyService.kill(terminalId, 'SIGKILL');
   * ```
   */
  async kill(terminalId: string, signal?: string): Promise<void> {
    const terminal = this.terminals.get(terminalId);

    if (!terminal) {
      const error = new Error(`Terminal ${terminalId} not found`);
      this.log(`[PtyService] Kill failed:`, error.message);
      throw error;
    }

    try {
      this.log(`[PtyService] Killing terminal ${terminalId} with signal ${signal || 'SIGHUP'}`);

      // Kill the PTY process
      if (signal) {
        terminal.pty.kill(signal);
      } else {
        terminal.pty.kill();
      }

      // Remove from terminals map
      this.terminals.delete(terminalId);

      this.log(`[PtyService] Terminal ${terminalId} killed successfully`);

    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.log(`[PtyService] Kill error for terminal ${terminalId}:`, errorMessage);
      this.emit('error', terminalId, error instanceof Error ? error : new Error(errorMessage));
      throw error;
    }
  }

  /**
   * Register a global data event handler (receives data from all terminals)
   *
   * @param callback - Function to call when data is received (includes terminalId)
   * @returns Function to remove the listener
   *
   * @example
   * ```typescript
   * const unsubscribe = ptyService.onData((terminalId, data) => {
   *   console.log(`Terminal ${terminalId} received:`, data);
   * });
   *
   * // Later, remove listener
   * unsubscribe();
   * ```
   */
  onData(callback: (terminalId: string, data: string) => void): () => void {
    this.on('data', callback);

    // Return unsubscribe function
    return () => this.off('data', callback);
  }

  /**
   * Register a global exit event handler (receives exit events from all terminals)
   *
   * @param callback - Function to call when terminal exits (includes terminalId)
   * @returns Function to remove the listener
   *
   * @example
   * ```typescript
   * const unsubscribe = ptyService.onExit((terminalId, exitCode, signal) => {
   *   console.log(`Terminal ${terminalId} exited with code ${exitCode}`);
   * });
   *
   * // Later, remove listener
   * unsubscribe();
   * ```
   */
  onExit(callback: (terminalId: string, exitCode: number, signal?: number) => void): () => void {
    this.on('exit', callback);

    // Return unsubscribe function
    return () => this.off('exit', callback);
  }

  /**
   * List all active terminals
   *
   * @returns Array of terminal information (without PTY instances)
   *
   * @example
   * ```typescript
   * const terminals = ptyService.listTerminals();
   * terminals.forEach(t => {
   *   console.log(`Terminal ${t.terminalId}: ${t.title}`);
   * });
   * ```
   */
  listTerminals(): PtyTerminalInfo[] {
    return Array.from(this.terminals.values()).map(terminal => ({
      terminalId: terminal.id,
      title: terminal.title,
      shell: terminal.shell,
      cwd: terminal.cwd,
      cols: terminal.cols,
      rows: terminal.rows,
      createdAt: terminal.createdAt,
      toolId: terminal.toolId,
      terminalNumber: terminal.terminalNumber
    }));
  }

  /**
   * Get terminal instance by ID
   *
   * @param terminalId - ID of the terminal
   * @returns Terminal instance or undefined if not found
   */
  getTerminal(terminalId: string): PtyTerminal | undefined {
    return this.terminals.get(terminalId);
  }

  /**
   * Get all active terminals
   *
   * @returns Array of terminal instances
   */
  getAllTerminals(): PtyTerminal[] {
    return Array.from(this.terminals.values());
  }

  /**
   * Check if a terminal exists
   *
   * @param terminalId - ID of the terminal
   * @returns True if terminal exists
   */
  hasTerminal(terminalId: string): boolean {
    return this.terminals.has(terminalId);
  }

  /**
   * Kill all terminals and cleanup
   *
   * This should be called when the application is shutting down
   */
  async cleanup(): Promise<void> {
    this.log(`[PtyService] Cleaning up ${this.terminals.size} terminals`);

    const terminals = Array.from(this.terminals.entries());
    for (const [terminalId, terminal] of terminals) {
      try {
        terminal.pty.kill();
        this.log(`[PtyService] Cleaned up terminal ${terminalId}`);
      } catch (error) {
        this.log(`[PtyService] Error cleaning up terminal ${terminalId}:`, error);
      }
    }

    this.terminals.clear();
    this.removeAllListeners();

    this.log('[PtyService] Cleanup complete');
  }

  /**
   * Set up event handlers for a PTY instance
   *
   * @private
   * @param terminalId - ID of the terminal
   * @param pty - PTY instance
   */
  private setupPtyEventHandlers(terminalId: string, pty: IPty): void {
    // Handle data from the PTY
    pty.onData((data: string) => {
      this.emit('data', terminalId, data);
    });

    // Handle PTY exit
    pty.onExit((event: { exitCode: number; signal?: number }) => {
      this.log(`[PtyService] Terminal ${terminalId} exited with code ${event.exitCode}`);
      this.emit('exit', terminalId, event.exitCode, event.signal);

      // Clean up terminal from map
      this.terminals.delete(terminalId);
    });
  }

  /**
   * Generate a unique terminal ID
   *
   * @private
   * @returns Unique terminal identifier
   */
  private generateTerminalId(): string {
    const timestamp = Date.now();
    const random = Math.random().toString(36).substring(2, 11);
    return `terminal-${timestamp}-${random}`;
  }

  /**
   * Get the default shell for the current platform
   *
   * @private
   * @returns Default shell path
   */
  private getDefaultShell(): string {
    const platform = os.platform();

    switch (platform) {
      case 'win32':
        // Windows: prefer PowerShell, fallback to cmd.exe
        return process.env.SHELL || 'powershell.exe';

      case 'darwin':
        // macOS: prefer zsh (default since macOS Catalina), fallback to bash
        return process.env.SHELL || '/bin/zsh';

      case 'linux':
        // Linux: use user's shell from environment, fallback to bash
        return process.env.SHELL || '/bin/bash';

      default:
        // Fallback for other Unix-like systems
        return process.env.SHELL || '/bin/sh';
    }
  }
}

/**
 * Singleton instance of PtyService (optional pattern)
 *
 * @example
 * ```typescript
 * import { ptyService } from './PtyService';
 *
 * const terminalId = ptyService.spawn({ shell: '/bin/bash' });
 * ```
 */
export const ptyService = new PtyService();
