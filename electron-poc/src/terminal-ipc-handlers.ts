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

/**
 * Register all terminal-related IPC handlers
 */
export function registerTerminalHandlers(mainWindow: Electron.BrowserWindow): void {
  logger.info('[Terminal] Registering terminal IPC handlers');

  // Create a new terminal process
  ipcMain.handle('create-terminal-process', async (event: IpcMainInvokeEvent, options: {
    terminalId: string;
    command?: string;
    args?: string[];
    cwd?: string;
    env?: Record<string, string>;
  }) => {
    try {
      logger.info(`[Terminal] Creating terminal process ${options.terminalId} with command: ${options.command}`);
      
      // Check if terminal already exists
      if (terminalProcesses.has(options.terminalId)) {
        logger.warn(`[Terminal] Terminal ${options.terminalId} already exists`);
        return { success: false, error: 'Terminal already exists' };
      }

      // Determine shell and command to use
      let shell: string;
      let args: string[];
      const cwd = options.cwd || process.cwd();
      
      // Setup enhanced PATH first
      const pathAdditions = [
        '/opt/homebrew/bin',
        '/usr/local/bin', 
        '/usr/bin',
        '/bin',
        '/usr/sbin',
        '/sbin'
      ];
      
      if (options.command && options.command !== 'bash' && options.command !== 'zsh' && options.command !== 'sh') {
        // If a specific command is provided (like 'claude'), run it within a shell
        shell = process.platform === 'win32' ? 'powershell.exe' : process.env.SHELL || '/bin/bash';
        
        // For known CLI tools, use their full paths
        let commandToRun = options.command;
        if (options.command === 'claude') {
          // Claude Code is typically at /opt/homebrew/bin/claude on M1 Macs
          commandToRun = '/opt/homebrew/bin/claude';
          logger.info(`[Terminal] Using full path for Claude Code: ${commandToRun}`);
        }
        
        // Run the command in an interactive shell
        args = ['-i', '-c', `${commandToRun} ${(options.args || []).join(' ')}`];
        logger.info(`[Terminal] Running command in interactive shell: ${commandToRun}`);
      } else {
        // Otherwise use the shell directly  
        shell = options.command || (process.platform === 'win32' ? 'powershell.exe' : process.env.SHELL || '/bin/bash');
        args = options.args || [];
      }
      
      // Merge environment variables - reuse the pathAdditions from above
      const currentPath = process.env.PATH || '';
      const enhancedPath = [...new Set([...pathAdditions, ...currentPath.split(':')])].join(':');
      
      const env = {
        ...process.env,
        PATH: enhancedPath,
        ...options.env,
        TERM: 'xterm-256color',
        COLORTERM: 'truecolor'
      };

      logger.info(`[Terminal] Spawning: ${shell} ${args.join(' ')} in ${cwd}`);

      // Create PTY process
      const ptyProcess = pty.spawn(shell, args, {
        name: 'xterm-256color',
        cols: 80,
        rows: 30,
        cwd,
        env
      });

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
      return { success: true, pid: ptyProcess.pid };

    } catch (error: any) {
      logger.error(`[Terminal] Failed to create terminal ${options.terminalId}:`, error.message || error);
      logger.error(`[Terminal] Error details:`, error.stack || error);
      return { success: false, error: error.message || String(error) };
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