/**
 * Terminal IPC Handlers for Main Process
 * Manages terminal processes using node-pty and communicates with renderer
 */

import { ipcMain, IpcMainInvokeEvent } from 'electron';
import * as pty from 'node-pty';
import { IPty } from 'node-pty';
import { logger } from './utils/SafeLogger';

interface TerminalProcess {
  terminalId: string;
  pty: IPty;
  command: string;
  workingDirectory: string;
  toolId?: string;
}

// Store active terminal processes
const terminalProcesses = new Map<string, TerminalProcess>();

// Track if handlers are already registered
let handlersRegistered = false;

/**
 * Register all terminal-related IPC handlers
 */
export function registerTerminalHandlers(mainWindow: Electron.BrowserWindow): void {
  logger.info('[Terminal] Registering terminal IPC handlers');
  
  // Skip if already registered
  if (handlersRegistered) {
    logger.info('[Terminal] Terminal IPC handlers already registered, skipping');
    return;
  }
  handlersRegistered = true;

  // Create a new terminal process
  ipcMain.handle('create-terminal-process', async (event: IpcMainInvokeEvent, options: {
    terminalId: string;
    command?: string;
    args?: string[];
    cwd?: string;
    env?: Record<string, string>;
  }) => {
    // Define these outside try block so they're available in catch
    // Use a shell that definitely exists on macOS
    let shell = '/bin/bash';  // Default to bash which exists on all Unix systems
    
    if (process.platform === 'win32') {
      shell = 'powershell.exe';
    } else if (process.platform === 'darwin') {
      // On macOS, prefer zsh (default since Catalina) but fall back to bash
      const fs = require('fs');
      if (fs.existsSync('/bin/zsh')) {
        shell = '/bin/zsh';
      } else if (fs.existsSync('/bin/bash')) {
        shell = '/bin/bash';
      } else if (fs.existsSync('/bin/sh')) {
        shell = '/bin/sh';
      }
    } else {
      // Linux - check for common shells
      const fs = require('fs');
      if (process.env.SHELL && fs.existsSync(process.env.SHELL)) {
        shell = process.env.SHELL;
      } else if (fs.existsSync('/bin/bash')) {
        shell = '/bin/bash';
      } else if (fs.existsSync('/bin/sh')) {
        shell = '/bin/sh';
      }
    }
    
    // Use login shell to ensure proper PATH setup for tools like Claude Code
    // Try without -l flag first to see if that's the issue
    const args: string[] = [];
    // Use HOME directory as default, never use root /
    const cwd = options.cwd || process.env.HOME || '/Users/veronelazio';
    
    // Enhanced PATH for finding tools like Claude Code
    const pathAdditions = [
      '/opt/homebrew/bin',
      '/usr/local/bin', 
      '/usr/bin',
      '/bin',
      '/usr/sbin',
      '/sbin'
    ];
    
    const currentPath = process.env.PATH || '';
    const enhancedPath = [...new Set([...pathAdditions, ...currentPath.split(':')])].join(':');
    
    const env = {
      ...process.env,
      PATH: enhancedPath,
      ...options.env,
      TERM: 'xterm-256color',
      COLORTERM: 'truecolor'
    };

    logger.info(`[Terminal] Creating shell: ${shell} in ${cwd}`);
    logger.info(`[Terminal] Shell exists: ${require('fs').existsSync(shell)}`);
    logger.info(`[Terminal] Shell is executable: ${shell} - checking...`);
    
    // Check if shell is executable
    try {
      const fs = require('fs');
      const stats = fs.statSync(shell);
      logger.info(`[Terminal] Shell file mode: ${stats.mode.toString(8)}`);
      logger.info(`[Terminal] Shell is file: ${stats.isFile()}`);
    } catch (e) {
      logger.error(`[Terminal] Cannot stat shell: ${e}`);
    }
    
    try {
      logger.info(`[Terminal] Creating terminal process ${options.terminalId} with command: ${options.command}`);
      
      // Check if terminal already exists
      if (terminalProcesses.has(options.terminalId)) {
        logger.warn(`[Terminal] Terminal ${options.terminalId} already exists`);
        return { success: false, error: 'Terminal already exists' };
      }

      logger.info(`[Terminal] About to spawn PTY with shell=${shell}, args=${JSON.stringify(args)}, cwd=${cwd}`);
      
      // Additional check - ensure cwd is valid
      const fs = require('fs');
      if (!fs.existsSync(cwd)) {
        logger.error(`[Terminal] Working directory does not exist: ${cwd}`);
        return { success: false, error: `Working directory does not exist: ${cwd}` };
      }
      
      // Check if we can access the directory
      try {
        fs.accessSync(cwd, fs.constants.R_OK);
      } catch (err) {
        logger.error(`[Terminal] Cannot access working directory: ${cwd}`, err);
        return { success: false, error: `Cannot access working directory: ${cwd}` };
      }
      
      // Create PTY process - wrap this specifically to catch the error
      let ptyProcess;
      try {
        ptyProcess = pty.spawn(shell, args, {
          name: 'xterm-256color',
          cols: 80,
          rows: 30,
          cwd,
          env
        });
        logger.info(`[Terminal] PTY spawn successful, PID: ${ptyProcess.pid}`);
      } catch (spawnError: any) {
        logger.error(`[Terminal] PTY spawn failed immediately:`, spawnError);
        logger.error(`[Terminal] Spawn error toString:`, spawnError?.toString());
        logger.error(`[Terminal] Spawn error message:`, spawnError?.message || 'No message');
        logger.error(`[Terminal] Spawn error code:`, spawnError?.code || 'No code');
        logger.error(`[Terminal] Spawn error errno:`, spawnError?.errno || 'No errno');
        logger.error(`[Terminal] Spawn error syscall:`, spawnError?.syscall || 'No syscall');
        logger.error(`[Terminal] Full error:`, JSON.stringify(spawnError, Object.getOwnPropertyNames(spawnError)));
        throw spawnError;
      }

      // Store the process
      const terminalProcess: TerminalProcess = {
        terminalId: options.terminalId,
        pty: ptyProcess,
        command: shell,
        workingDirectory: cwd
      };
      terminalProcesses.set(options.terminalId, terminalProcess);

      // Handle PTY output
      ptyProcess.onData((data: string) => {
        // Send data to renderer
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('terminal-data', options.terminalId, data);
        }
      });

      // Handle PTY exit
      ptyProcess.onExit((exitCode: { exitCode: number; signal?: number }) => {
        logger.info(`[Terminal] Process ${options.terminalId} exited with code ${exitCode.exitCode}`);
        
        // Remove from map
        terminalProcesses.delete(options.terminalId);
        
        // Notify renderer
        if (mainWindow && !mainWindow.isDestroyed()) {
          mainWindow.webContents.send('terminal-exit', options.terminalId, exitCode.exitCode);
        }
      });

      logger.info(`[Terminal] Successfully created terminal ${options.terminalId}, PID: ${ptyProcess.pid}`);
      
      // Don't handle commands here - let the renderer send them
      // This keeps it simple: we just create terminals, the UI types commands
      
      return { success: true, pid: ptyProcess.pid };

    } catch (error: any) {
      logger.error(`[Terminal] Failed to create terminal ${options.terminalId}:`, error);
      
      // Better error logging for debugging
      if (error) {
        logger.error(`[Terminal] Error type:`, typeof error);
        logger.error(`[Terminal] Error message:`, error.message || 'No message');
        logger.error(`[Terminal] Error stack:`, error.stack || 'No stack trace');
        logger.error(`[Terminal] Error code:`, error.code || 'No error code');
        logger.error(`[Terminal] Full error object:`, JSON.stringify(error, null, 2));
      } else {
        logger.error(`[Terminal] Error is null or undefined`);
      }
      
      logger.error(`[Terminal] Shell path:`, shell);
      logger.error(`[Terminal] Working directory:`, cwd);
      logger.error(`[Terminal] PATH env:`, env.PATH);
      
      // Return a meaningful error message
      const errorMessage = error?.message || error?.toString() || 'Unknown error creating terminal';
      return { success: false, error: errorMessage };
    }
  });

  // Write data to terminal (from user input)
  ipcMain.handle('write-to-terminal', async (event: IpcMainInvokeEvent, terminalId: string, data: string) => {
    const terminalProcess = terminalProcesses.get(terminalId);
    if (terminalProcess) {
      terminalProcess.pty.write(data);
      return { success: true };
    } else {
      logger.warn(`[Terminal] Terminal ${terminalId} not found for write`);
      return { success: false, error: 'Terminal not found' };
    }
  });

  // Resize terminal
  ipcMain.handle('resize-terminal', async (event: IpcMainInvokeEvent, terminalId: string, cols: number, rows: number) => {
    const terminalProcess = terminalProcesses.get(terminalId);
    if (terminalProcess) {
      terminalProcess.pty.resize(cols, rows);
      return { success: true };
    } else {
      logger.warn(`[Terminal] Terminal ${terminalId} not found for resize`);
      return { success: false, error: 'Terminal not found' };
    }
  });

  // Kill terminal process
  ipcMain.handle('kill-terminal-process', async (event: IpcMainInvokeEvent, terminalId: string) => {
    const terminalProcess = terminalProcesses.get(terminalId);
    if (terminalProcess) {
      logger.info(`[Terminal] Killing terminal ${terminalId}`);
      terminalProcess.pty.kill();
      terminalProcesses.delete(terminalId);
      return { success: true };
    } else {
      logger.warn(`[Terminal] Terminal ${terminalId} not found for kill`);
      return { success: false, error: 'Terminal not found' };
    }
  });

  // Get terminal status
  ipcMain.handle('get-terminal-status', async (event: IpcMainInvokeEvent, terminalId: string) => {
    const terminalProcess = terminalProcesses.get(terminalId);
    if (terminalProcess) {
      return {
        exists: true,
        pid: terminalProcess.pty.pid,
        command: terminalProcess.command,
        workingDirectory: terminalProcess.workingDirectory
      };
    } else {
      return { exists: false };
    }
  });

  // List all terminals
  ipcMain.handle('list-terminals', async () => {
    const terminals = Array.from(terminalProcesses.entries()).map(([id, process]) => ({
      terminalId: id,
      pid: process.pty.pid,
      command: process.command,
      workingDirectory: process.workingDirectory
    }));
    return terminals;
  });

  logger.info('[Terminal] Terminal IPC handlers registered');
}

/**
 * Clean up all terminal processes on app quit
 */
export function cleanupTerminals(): void {
  logger.info('[Terminal] Cleaning up all terminal processes');
  
  terminalProcesses.forEach((process, id) => {
    try {
      logger.info(`[Terminal] Killing terminal ${id}`);
      process.pty.kill();
    } catch (error) {
      logger.error(`[Terminal] Error killing terminal ${id}:`, error);
    }
  });
  
  terminalProcesses.clear();
}