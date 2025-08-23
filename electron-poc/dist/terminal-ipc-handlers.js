"use strict";
/**
 * Terminal IPC Handlers for TTYD-based terminals
 * Manages ttyd terminal server instances via IPC communication
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    Object.defineProperty(o, k2, { enumerable: true, get: function() { return m[k]; } });
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.cleanupTerminals = exports.registerTerminalHandlers = void 0;
const electron_1 = require("electron");
const TTYDManager_1 = __importDefault(require("./services/TTYDManager"));
const ProcessManager_1 = __importDefault(require("./utils/ProcessManager"));
const SafeLogger_1 = require("./utils/SafeLogger");
const fs = __importStar(require("fs"));
const os = __importStar(require("os"));
const path = __importStar(require("path"));
// Initialize managers
const processManager = new ProcessManager_1.default();
const ttydManager = new TTYDManager_1.default(processManager);
// Track active terminal numbers to reuse closed ones
const activeTerminalNumbers = new Set();
// Track if handlers are already registered
let handlersRegistered = false;
// Store reference to main window for events
let mainWindowRef = null;
/**
 * Get the next available terminal number
 */
function getNextTerminalNumber() {
    let num = 1;
    while (activeTerminalNumbers.has(num)) {
        num++;
    }
    return num;
}
/**
 * Register all terminal-related IPC handlers
 */
function registerTerminalHandlers(mainWindow) {
    console.log('[TerminalIPC] Registering TTYD terminal handlers');
    SafeLogger_1.logger.info('[TerminalIPC] Registering TTYD terminal handlers');
    // Skip if already registered
    if (handlersRegistered) {
        console.log('[TerminalIPC] Terminal IPC handlers already registered, skipping');
        SafeLogger_1.logger.info('[TerminalIPC] Terminal IPC handlers already registered, skipping');
        return;
    }
    handlersRegistered = true;
    mainWindowRef = mainWindow;
    console.log('[TerminalIPC] Handlers registered, mainWindow set');
    // Clean up terminals when the main window actually navigates/reloads
    // Only cleanup if navigating away from the app, not when webviews load
    mainWindow.webContents.on('will-navigate', (event, url) => __awaiter(this, void 0, void 0, function* () {
        // Only cleanup if it's the main window navigation, not webview navigation
        if (!url.includes('localhost:7') && !url.includes('localhost:8')) {
            SafeLogger_1.logger.info('[TerminalIPC] Main window navigating, cleaning up terminals...');
            yield ttydManager.cleanup();
            // Clear all active terminal numbers on reload
            activeTerminalNumbers.clear();
        }
    }));
    // Create a new terminal
    electron_1.ipcMain.handle('create-terminal-process', (event, options) => __awaiter(this, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[TerminalIPC] create-terminal-process called with options:', options);
        try {
            // Generate ID if not provided
            const id = options.terminalId || `terminal-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
            // Determine title
            let title;
            let terminalNumber;
            if (options.toolId) {
                // Use tool name as title
                title = getToolDisplayName(options.toolId);
            }
            else {
                // Generic terminal - get next available number
                terminalNumber = getNextTerminalNumber();
                activeTerminalNumbers.add(terminalNumber);
                title = `Terminal ${terminalNumber}`;
            }
            // Handle special Grok setup wizard
            let actualCommand = options.command;
            if (options.command === 'grok:setup') {
                // Create an interactive setup script for Grok
                SafeLogger_1.logger.info('[TerminalIPC] Launching Grok setup wizard');
                // Create a multi-line bash script that guides the user through setup
                // Create a temporary script file for better handling of the setup wizard
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
                actualCommand = `bash ${scriptPath}; rm -f ${scriptPath}`;
            }
            // Create terminal via TTYDManager
            const terminal = yield ttydManager.createTerminal({
                id,
                title,
                toolId: options.toolId,
                cwd: options.cwd || process.env.HOME || '/Users/veronelazio',
                command: actualCommand,
                env: options.env
            });
            SafeLogger_1.logger.info(`[TerminalIPC] Created terminal: ${title} (${id}) on port ${terminal.port}`);
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
            // Store terminal number in the terminal object for cleanup later
            if (terminalNumber !== undefined) {
                terminal.terminalNumber = terminalNumber;
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
        }
        catch (error) {
            SafeLogger_1.logger.error(`[TerminalIPC] Failed to create terminal:`, error);
            return {
                success: false,
                error: error.message || 'Failed to create terminal'
            };
        }
    }));
    // Write data to terminal - NOT NEEDED WITH TTYD (webview handles input)
    // Keeping for compatibility but will be handled by webview
    electron_1.ipcMain.handle('write-to-terminal', (event, terminalId, data) => __awaiter(this, void 0, void 0, function* () {
        // With ttyd, input is handled directly by the webview
        // This handler can be used to execute commands programmatically
        ttydManager.executeCommand(terminalId, data);
        return { success: true };
    }));
    // Resize terminal - NOT NEEDED WITH TTYD (webview auto-resizes)
    electron_1.ipcMain.handle('resize-terminal', (event, terminalId, cols, rows) => __awaiter(this, void 0, void 0, function* () {
        // With ttyd, resize is handled automatically by the webview
        SafeLogger_1.logger.info(`[TerminalIPC] Resize not needed for ttyd terminals (auto-handled)`);
        return { success: true };
    }));
    // Kill terminal process
    electron_1.ipcMain.handle('kill-terminal-process', (event, terminalId) => __awaiter(this, void 0, void 0, function* () {
        try {
            // Get terminal info before closing to extract terminal number
            const terminal = ttydManager.getTerminal(terminalId);
            if (terminal && terminal.terminalNumber) {
                activeTerminalNumbers.delete(terminal.terminalNumber);
            }
            const success = yield ttydManager.closeTerminal(terminalId);
            SafeLogger_1.logger.info(`[TerminalIPC] Closed terminal: ${terminalId}`);
            return { success };
        }
        catch (error) {
            SafeLogger_1.logger.error(`[TerminalIPC] Failed to close terminal:`, error);
            return { success: false, error: error.message };
        }
    }));
    // Get terminal status
    electron_1.ipcMain.handle('get-terminal-status', (event, terminalId) => __awaiter(this, void 0, void 0, function* () {
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
        }
        else {
            return { exists: false };
        }
    }));
    // List all terminals
    electron_1.ipcMain.handle('list-terminals', () => __awaiter(this, void 0, void 0, function* () {
        const terminals = ttydManager.getAllTerminals();
        return terminals.map(t => ({
            terminalId: t.id,
            title: t.title,
            url: t.url,
            port: t.port,
            status: t.status,
            toolId: t.toolId
        }));
    }));
    // Set up TTYDManager event forwarding
    ttydManager.on('terminal:ready', (terminalId, instance) => {
        if (mainWindowRef && !mainWindowRef.isDestroyed()) {
            mainWindowRef.webContents.send('terminal-ready', terminalId, instance.url);
        }
    });
    ttydManager.on('terminal:closed', (terminalId) => {
        // Clean up terminal number when terminal closes on its own
        const terminal = ttydManager.getTerminal(terminalId);
        if (terminal && terminal.terminalNumber) {
            activeTerminalNumbers.delete(terminal.terminalNumber);
        }
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
    SafeLogger_1.logger.info('[TerminalIPC] TTYD terminal handlers registered');
}
exports.registerTerminalHandlers = registerTerminalHandlers;
/**
 * Helper function to get display name for tools
 */
function getToolDisplayName(toolId) {
    const toolNames = {
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
function cleanupTerminals() {
    return __awaiter(this, void 0, void 0, function* () {
        SafeLogger_1.logger.info('[TerminalIPC] Cleaning up all terminals...');
        yield ttydManager.cleanup();
        SafeLogger_1.logger.info('[TerminalIPC] All terminals cleaned up');
    });
}
exports.cleanupTerminals = cleanupTerminals;
//# sourceMappingURL=terminal-ipc-handlers.js.map