/**
 * Terminal IPC Handlers for PTY-based terminals
 * Manages terminal instances via native PTY integration (node-pty or hive_pty)
 */

import { ipcMain, IpcMainInvokeEvent } from 'electron';
import { PtyService } from './main/terminal/PtyService';
import ProcessManager from './utils/ProcessManager';
import { logger } from './utils/SafeLogger';
import * as fs from 'fs';
import * as os from 'os';
import * as path from 'path';

// Managers will be initialized when registerTerminalHandlers is called
let processManager: ProcessManager;
let ptyService: PtyService;

// Track active terminal numbers to reuse closed ones
const activeTerminalNumbers = new Set<number>();

// Track if handlers are already registered
let handlersRegistered = false;

// Store reference to main window for events
let mainWindowRef: Electron.BrowserWindow | null = null;

/**
 * Get the next available terminal number
 */
function getNextTerminalNumber(): number {
  let num = 1;
  while (activeTerminalNumbers.has(num)) {
    num++;
  }
  return num;
}

/**
 * Register all terminal-related IPC handlers
 * @param mainWindow - The main browser window
 * @param processManagerInstance - The ProcessManager instance from the main process
 */
export function registerTerminalHandlers(mainWindow: Electron.BrowserWindow, processManagerInstance: ProcessManager): void {
  console.log('[TerminalIPC] Registering PTY terminal handlers');
  logger.info('[TerminalIPC] Registering PTY terminal handlers');

  // Skip if already registered
  if (handlersRegistered) {
    console.log('[TerminalIPC] Terminal IPC handlers already registered, skipping');
    logger.info('[TerminalIPC] Terminal IPC handlers already registered, skipping');
    return;
  }

  // Initialize managers with the shared ProcessManager instance
  processManager = processManagerInstance;
  ptyService = new PtyService();

  handlersRegistered = true;
  mainWindowRef = mainWindow;
  console.log('[TerminalIPC] Handlers registered, mainWindow set');

  // Set up PTY event forwarding to renderer
  ptyService.onData((terminalId: string, data: string) => {
    if (mainWindowRef && !mainWindowRef.isDestroyed()) {
      mainWindowRef.webContents.send('terminal-data', terminalId, data);
    }
  });

  ptyService.onExit((terminalId: string, exitCode: number) => {
    // Clean up terminal number when terminal closes
    const terminals = ptyService.listTerminals();
    const terminal = terminals.find(t => t.terminalId === terminalId);
    if (terminal && terminal.terminalNumber) {
      activeTerminalNumbers.delete(terminal.terminalNumber);
    }

    if (mainWindowRef && !mainWindowRef.isDestroyed()) {
      mainWindowRef.webContents.send('terminal-exit', terminalId, exitCode);
    }
  });

  // Clean up terminals when the main window actually navigates/reloads
  // Only cleanup if navigating away from the app, not when webviews load
  mainWindow.webContents.on('will-navigate', async (event, url) => {
    // Only cleanup if it's the main window navigation, not webview navigation
    if (!url.includes('localhost:7') && !url.includes('localhost:8')) {
      logger.info('[TerminalIPC] Main window navigating, cleaning up terminals...');
      await ptyService.cleanup();
      // Clear all active terminal numbers on reload
      activeTerminalNumbers.clear();
    }
  });

  // Create a new terminal - CRITICAL: Keep exact same IPC channel name
  ipcMain.handle('create-terminal-process', async (event: IpcMainInvokeEvent, options: {
    terminalId: string;
    command?: string;
    args?: string[];
    cwd?: string;
    env?: Record<string, string>;
    toolId?: string;
    // Optional: inline script content to execute before interactive shell
    scriptContent?: string;
  }) => {
    logger.info('[TerminalIPC] create-terminal-process called with options:', options);

    try {
      // Generate ID if not provided
      const id = options.terminalId || `terminal-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

      // Determine title
      let title: string;
      let terminalNumber: number | undefined;
      if (options.toolId) {
        // Use tool name as title
        title = getToolDisplayName(options.toolId);
      } else {
        // Generic terminal - get next available number
        terminalNumber = getNextTerminalNumber();
        activeTerminalNumbers.add(terminalNumber);
        title = `Terminal ${terminalNumber}`;
      }

      // Handle special Grok setup wizard
      let actualCommand = options.command;
      let actualArgs = options.args;

      // If scriptContent is provided, write it to a temp file and run it
      if (options.scriptContent && options.scriptContent.trim().length > 0) {
        try {
          const scriptPath = path.join(os.tmpdir(), `hive-spec-wizard-${Date.now()}-${Math.random().toString(36).slice(2)}.sh`);
          fs.writeFileSync(scriptPath, options.scriptContent, { encoding: 'utf-8' });
          fs.chmodSync(scriptPath, '755');
          // Run the script, then shell will continue
          actualCommand = 'bash';
          actualArgs = [scriptPath];
          logger.info(`[TerminalIPC] Created temp script for terminal at ${scriptPath}`);
        } catch (e: any) {
          logger.error('[TerminalIPC] Failed to prepare temp script:', e?.message || e);
        }
      }

      if (options.command === 'grok:setup') {
        // Create an interactive setup script for Grok
        logger.info('[TerminalIPC] Launching Grok setup wizard');

        // Create a multi-line bash script that guides the user through setup
        const scriptContent = `#!/bin/bash
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "                 🚀 Grok CLI Setup Wizard"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Welcome to Grok CLI! Let's get you set up."
echo ""
echo "To use Grok, you need an API key from X.AI"
echo ""
echo "📝 Steps to get your API key:"
echo "   1. Visit https://console.x.ai/team/default/api-keys"
echo "   2. Sign in or create an account"
echo "   3. Click 'Create API key'"
echo "   4. Copy your new API key"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
read -p "Would you like to set up your API key now? (y/n): " response
echo ""

if [[ "$response" =~ ^[Yy]$ ]]; then
  echo "Please enter your Grok API key (it will be visible for verification):"
  echo ""
  echo "📝 Paste your API key below and press Enter:"
  read api_key
  echo ""

  if [ -n "$api_key" ]; then
    # Show the key for verification (masked partially for security)
    key_length=\${#api_key}
    if [ \$key_length -gt 10 ]; then
      # Show first 7 chars and last 4 chars with asterisks in between
      first_part=\${api_key:0:7}
      last_part=\${api_key: -4}
      masked_middle=\$(printf '*%.0s' {1..8})
      echo "🔑 API Key to be saved: \${first_part}\${masked_middle}\${last_part}"
    else
      echo "🔑 API Key to be saved: [key too short, might be invalid]"
    fi
    echo ""

    # Confirm before saving
    read -p "Is this correct? (y/n): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
      echo "❌ Setup cancelled. Please run the setup wizard again."
      exit 0
    fi

    echo ""
    echo "💾 Saving API key..."

    # Create the .grok directory if it doesn't exist
    mkdir -p ~/.grok

    # Check if user-settings.json exists and has content
    if [ -f ~/.grok/user-settings.json ]; then
      # Backup existing file
      cp ~/.grok/user-settings.json ~/.grok/user-settings.json.bak

      # Read existing settings and add apiKey using a more reliable method
      if command -v python3 >/dev/null 2>&1; then
        python3 -c "
import json
import sys

try:
    with open('$HOME/.grok/user-settings.json', 'r') as f:
        settings = json.load(f)
except:
    settings = {}

settings['apiKey'] = '$api_key'
if 'baseURL' not in settings:
    settings['baseURL'] = 'https://api.x.ai/v1'
if 'defaultModel' not in settings:
    settings['defaultModel'] = 'grok-4-latest'

with open('$HOME/.grok/user-settings.json', 'w') as f:
    json.dump(settings, f, indent=2)
print('✅ API key added to existing configuration')
"
      else
        # Fallback: create new config with just the essentials
        cat > ~/.grok/user-settings.json << EOF
{
  "apiKey": "$api_key",
  "baseURL": "https://api.x.ai/v1",
  "defaultModel": "grok-4-latest"
}
EOF
        echo "✅ API key saved to ~/.grok/user-settings.json"
      fi
    else
      # Create new config file
      cat > ~/.grok/user-settings.json << EOF
{
  "apiKey": "$api_key",
  "baseURL": "https://api.x.ai/v1",
  "defaultModel": "grok-4-latest"
}
EOF
      echo "✅ API key saved to ~/.grok/user-settings.json"
    fi

    echo ""
    echo "🎉 Setup complete! Your API key has been saved."
    echo ""
    echo "Launching Grok CLI in 2 seconds..."
    echo ""
    sleep 2
    exec grok
  else
    echo "❌ No API key entered. Please run the setup wizard again."
    echo ""
    echo "You can also set up your API key manually by:"
    echo "1. Running: grok config set apiKey YOUR_KEY"
    echo "2. Editing: ~/.grok/user-settings.json"
    echo "3. Setting: export GROK_API_KEY='your_key'"
  fi
else
  echo "You can set up your API key later using any of these methods:"
  echo ""
  echo "1. Run: grok config set apiKey YOUR_KEY"
  echo "2. Edit: ~/.grok/user-settings.json"
  echo "3. Set environment: export GROK_API_KEY='your_key'"
  echo "4. Use flag: grok --api-key YOUR_KEY"
  echo ""
  echo "To get your API key, visit:"
  echo "https://console.x.ai/team/default/api-keys"
fi
`;

        // Write the script to a temporary file
        const scriptPath = path.join(os.tmpdir(), `grok-setup-${Date.now()}.sh`);
        fs.writeFileSync(scriptPath, scriptContent);
        fs.chmodSync(scriptPath, '755');

        // Run the script
        actualCommand = 'bash';
        actualArgs = [scriptPath];
      }

      // Determine shell and args
      const shell = actualCommand || process.env.SHELL || '/bin/bash';
      const shellArgs = actualArgs || [];

      // Create terminal via PtyService
      const terminal = await ptyService.spawn({
        id,
        title,
        shell,
        args: shellArgs,
        cwd: options.cwd || process.env.HOME || process.cwd(),
        env: {
          ...process.env,
          ...options.env,
          TERM: 'xterm-256color',
        },
        toolId: options.toolId,
        terminalNumber,
      });

      logger.info(`[TerminalIPC] Created terminal: ${title} (${id})`);

      // Notify renderer about the new terminal
      if (mainWindowRef && !mainWindowRef.isDestroyed()) {
        mainWindowRef.webContents.send('terminal-created', {
          id: terminal.id,
          title: terminal.title,
          toolId: terminal.toolId,
        });
      }

      return {
        success: true,
        pid: terminal.pty.pid,
        terminal: {
          id: terminal.id,
          title: terminal.title,
          toolId: terminal.toolId,
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

  // Write data to terminal - CRITICAL: Keep exact same IPC channel name
  ipcMain.handle('write-to-terminal', async (event: IpcMainInvokeEvent, terminalId: string, data: string) => {
    try {
      await ptyService.write(terminalId, data);
      return { success: true };
    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to write to terminal ${terminalId}:`, error);
      return {
        success: false,
        error: error.message || 'Failed to write to terminal'
      };
    }
  });

  // Resize terminal - CRITICAL: Keep exact same IPC channel name
  ipcMain.handle('resize-terminal', async (event: IpcMainInvokeEvent, terminalId: string, cols: number, rows: number) => {
    try {
      await ptyService.resize(terminalId, cols, rows);
      logger.info(`[TerminalIPC] Resized terminal ${terminalId} to ${cols}x${rows}`);
      return { success: true };
    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to resize terminal ${terminalId}:`, error);
      return {
        success: false,
        error: error.message || 'Failed to resize terminal'
      };
    }
  });

  // Kill terminal process - CRITICAL: Keep exact same IPC channel name
  ipcMain.handle('kill-terminal-process', async (event: IpcMainInvokeEvent, terminalId: string) => {
    try {
      // Clean up terminal number before killing
      const terminals = ptyService.listTerminals();
      const terminal = terminals.find(t => t.terminalId === terminalId);
      if (terminal && terminal.terminalNumber) {
        activeTerminalNumbers.delete(terminal.terminalNumber);
      }

      await ptyService.kill(terminalId);
      logger.info(`[TerminalIPC] Killed terminal: ${terminalId}`);
      return { success: true };
    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to kill terminal:`, error);
      return { success: false, error: error.message };
    }
  });

  // Get terminal status - CRITICAL: Keep exact same IPC channel name
  ipcMain.handle('get-terminal-status', async (event: IpcMainInvokeEvent, terminalId: string) => {
    try {
      const terminals = ptyService.listTerminals();
      const terminal = terminals.find(t => t.terminalId === terminalId);

      if (terminal) {
        return {
          exists: true,
          id: terminal.terminalId,
          title: terminal.title,
          status: 'running', // PTY terminals are always running until they exit
          toolId: terminal.toolId,
        };
      } else {
        return { exists: false };
      }
    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to get terminal status:`, error);
      return { exists: false };
    }
  });

  // List all terminals - CRITICAL: Keep exact same IPC channel name
  ipcMain.handle('list-terminals', async () => {
    try {
      const terminals = ptyService.listTerminals();
      return terminals.map(t => ({
        terminalId: t.terminalId,
        title: t.title,
        status: 'running',
        toolId: t.toolId,
      }));
    } catch (error: any) {
      logger.error(`[TerminalIPC] Failed to list terminals:`, error);
      return [];
    }
  });

  console.log('[TerminalIPC] PTY terminal handlers registered successfully');
  logger.info('[TerminalIPC] PTY terminal handlers registered');
}

/**
 * Helper function to get display name for tools
 */
function getToolDisplayName(toolId: string): string {
  const toolNames: Record<string, string> = {
    'claude-code': 'Claude',
    'gemini-cli': 'Gemini',
    'gemini': 'Gemini',
    'qwen-code': 'Qwen',
    'qwen-coder': 'Qwen',
    'openai-codex': 'Codex',
    'codex': 'Codex',
    'cline': 'Cline',
    'cline-cli': 'Cline',
    'grok': 'Grok'
  };

  return toolNames[toolId] || toolId;
}

/**
 * Clean up all terminal processes on app quit
 */
export async function cleanupTerminals(): Promise<void> {
  logger.info('[TerminalIPC] Cleaning up all terminals...');
  if (ptyService) {
    await ptyService.cleanup();
  }
  logger.info('[TerminalIPC] All terminals cleaned up');
}
