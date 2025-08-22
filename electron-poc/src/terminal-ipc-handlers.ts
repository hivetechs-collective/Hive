/**
 * Terminal IPC Handlers for TTYD-based terminals
 * Manages ttyd terminal server instances via IPC communication
 */

import { ipcMain, IpcMainInvokeEvent } from 'electron';
import TTYDManager from './services/TTYDManager';
import ProcessManager from './utils/ProcessManager';
import { logger } from './utils/SafeLogger';

// Initialize managers
const processManager = new ProcessManager();
const ttydManager = new TTYDManager(processManager);

// Terminal counter for generic terminals
let terminalCounter = 1;

// Track if handlers are already registered
let handlersRegistered = false;

// Store reference to main window for events
let mainWindowRef: Electron.BrowserWindow | null = null;

/**
 * Register all terminal-related IPC handlers
 */
export function registerTerminalHandlers(mainWindow: Electron.BrowserWindow): void {
  console.log('[TerminalIPC] Registering TTYD terminal handlers');
  logger.info('[TerminalIPC] Registering TTYD terminal handlers');
  
  // Skip if already registered
  if (handlersRegistered) {
    console.log('[TerminalIPC] Terminal IPC handlers already registered, skipping');
    logger.info('[TerminalIPC] Terminal IPC handlers already registered, skipping');
    return;
  }
  handlersRegistered = true;
  mainWindowRef = mainWindow;
  console.log('[TerminalIPC] Handlers registered, mainWindow set');

  // Create a new terminal
  ipcMain.handle('create-terminal-process', async (event: IpcMainInvokeEvent, options: {
    terminalId: string;
    command?: string;
    args?: string[];
    cwd?: string;
    env?: Record<string, string>;
    toolId?: string;
  }) => {
    logger.info('[TerminalIPC] create-terminal-process called with options:', options);
    
    try {
      // Generate ID if not provided
      const id = options.terminalId || `terminal-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
      
      // Determine title
      let title: string;
      if (options.toolId) {
        // Use tool name as title
        title = getToolDisplayName(options.toolId);
      } else {
        // Generic terminal
        title = `Terminal ${terminalCounter++}`;
      }
      
      // Create terminal via TTYDManager
      const terminal = await ttydManager.createTerminal({
        id,
        title,
        toolId: options.toolId,
        cwd: options.cwd || process.env.HOME || '/Users/veronelazio',
        command: options.command,
        env: options.env
      });
      
      logger.info(`[TerminalIPC] Created terminal: ${title} (${id}) on port ${terminal.port}`);
      
      // Notify renderer about the new terminal
      if (mainWindowRef && !mainWindowRef.isDestroyed()) {
        mainWindowRef.webContents.send('terminal-created', {
          id: terminal.id,
          title: terminal.title,
          url: terminal.url,
          port: terminal.port,
          toolId: terminal.toolId
        });
      }
      
      return {
        success: true,
        terminal: {
          id: terminal.id,
          title: terminal.title,
          url: terminal.url,
          port: terminal.port,
          toolId: terminal.toolId
        }
      };

    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to create terminal:`, error);
      return {
        success: false,
        error: error.message || 'Failed to create terminal'
      };
    }
  });

  // Write data to terminal - NOT NEEDED WITH TTYD (webview handles input)
  // Keeping for compatibility but will be handled by webview
  ipcMain.handle('write-to-terminal', async (event: IpcMainInvokeEvent, terminalId: string, data: string) => {
    // With ttyd, input is handled directly by the webview
    // This handler can be used to execute commands programmatically
    ttydManager.executeCommand(terminalId, data);
    return { success: true };
  });

  // Resize terminal - NOT NEEDED WITH TTYD (webview auto-resizes)
  ipcMain.handle('resize-terminal', async (event: IpcMainInvokeEvent, terminalId: string, cols: number, rows: number) => {
    // With ttyd, resize is handled automatically by the webview
    logger.info(`[TerminalIPC] Resize not needed for ttyd terminals (auto-handled)`);
    return { success: true };
  });

  // Kill terminal process
  ipcMain.handle('kill-terminal-process', async (event: IpcMainInvokeEvent, terminalId: string) => {
    try {
      const success = await ttydManager.closeTerminal(terminalId);
      logger.info(`[TerminalIPC] Closed terminal: ${terminalId}`);
      return { success };
    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to close terminal:`, error);
      return { success: false, error: error.message };
    }
  });

  // Get terminal status
  ipcMain.handle('get-terminal-status', async (event: IpcMainInvokeEvent, terminalId: string) => {
    const terminal = ttydManager.getTerminal(terminalId);
    if (terminal) {
      return {
        exists: true,
        id: terminal.id,
        title: terminal.title,
        url: terminal.url,
        port: terminal.port,
        status: terminal.status,
        toolId: terminal.toolId
      };
    } else {
      return { exists: false };
    }
  });

  // List all terminals
  ipcMain.handle('list-terminals', async () => {
    const terminals = ttydManager.getAllTerminals();
    return terminals.map(t => ({
      terminalId: t.id,
      title: t.title,
      url: t.url,
      port: t.port,
      status: t.status,
      toolId: t.toolId
    }));
  });
  
  // Set up TTYDManager event forwarding
  ttydManager.on('terminal:ready', (terminalId, instance) => {
    if (mainWindowRef && !mainWindowRef.isDestroyed()) {
      mainWindowRef.webContents.send('terminal-ready', terminalId, instance.url);
    }
  });
  
  ttydManager.on('terminal:closed', (terminalId) => {
    if (mainWindowRef && !mainWindowRef.isDestroyed()) {
      mainWindowRef.webContents.send('terminal-exit', terminalId);
    }
  });
  
  ttydManager.on('terminal:error', (terminalId, error) => {
    if (mainWindowRef && !mainWindowRef.isDestroyed()) {
      mainWindowRef.webContents.send('terminal-error', terminalId, error.message);
    }
  });

  console.log('[TerminalIPC] TTYD terminal handlers registered successfully');
  logger.info('[TerminalIPC] TTYD terminal handlers registered');
}

/**
 * Helper function to get display name for tools
 */
function getToolDisplayName(toolId: string): string {
  const toolNames: Record<string, string> = {
    'claude-code': 'Claude',
    'gemini': 'Gemini',
    'qwen-coder': 'Qwen',
    'codex': 'Codex',
    'aider': 'Aider',
    'cline': 'Cline'
  };
  
  return toolNames[toolId] || toolId;
}

/**
 * Clean up all terminal processes on app quit
 */
export async function cleanupTerminals(): Promise<void> {
  logger.info('[TerminalIPC] Cleaning up all terminals...');
  await ttydManager.cleanup();
  logger.info('[TerminalIPC] All terminals cleaned up');
}