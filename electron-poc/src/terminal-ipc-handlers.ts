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
      logger.info(`[Terminal] Creating terminal process ${options.terminalId}`);
      
      // Check if terminal already exists
      if (terminalProcesses.has(options.terminalId)) {
        logger.warn(`[Terminal] Terminal ${options.terminalId} already exists`);
        return { success: false, error: 'Terminal already exists' };
      }

      // Determine shell to use
      const shell = options.command || (process.platform === 'win32' ? 'powershell.exe' : process.env.SHELL || '/bin/bash');
      const args = options.args || [];
      const cwd = options.cwd || process.cwd();
      
      // Merge environment variables
      const env = {
        ...process.env,
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
      logger.error(`[Terminal] Failed to create terminal ${options.terminalId}:`, error);
      return { success: false, error: error.message };
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