/**
 * Hive Consensus - Exact Dioxus GUI Recreation
 * Layout: Left Sidebar | Center (with bottom Terminal) | Right Consensus Chat
 */

// Import CLI tool detector
import { cliToolDetector } from './utils/cli-tool-detector';

// CLI Tool Types
interface CliToolStatus {
  id: string;
  name: string;
  installed: boolean;
  version?: string;
  path?: string;
  memoryServiceConnected?: boolean;
}

interface CliToolCardInfo {
  id: string;
  name: string;
  description: string;
  status: CliToolStatus | null;
  docsUrl: string;
  badgeText?: string | null;
  badgeColor?: string;
}

// Extend electronAPI type
interface ExtendedElectronAPI {
  detectCliTool(toolId: string): Promise<CliToolStatus | null>;
  detectAllCliTools(): Promise<CliToolStatus[]>;
}

// DISABLE WEBPACK-DEV-SERVER ERROR OVERLAY
(function() {
  if (typeof window !== 'undefined') {
    // Remove overlay periodically
    const removeOverlay = () => {
      const overlay = document.getElementById('webpack-dev-server-client-overlay');
      if (overlay) overlay.remove();
      const overlayDiv = document.getElementById('webpack-dev-server-client-overlay-div');
      if (overlayDiv) overlayDiv.remove();
    };
    removeOverlay();
    setInterval(removeOverlay, 100);
  }
})();

import './index.css';
import './neural-consciousness.css';
import './analytics.css';
import './git.css';
import './file-explorer.css';
import './status-bar.css';
import './vscode-scm.css';
import hiveLogo from './Hive-Logo-small.jpg';
import aiRobotIcon from './assets/ai-robot.png';
import { SettingsModal } from './settings-modal';

// Import AI CLI tool icons
import claudeIcon from '../resources/ai-cli-icons/claude.svg';
import geminiIcon from '../resources/ai-cli-icons/gemini.svg';
import grokIcon from '../resources/ai-cli-icons/grok.svg';
import qwenIcon from '../resources/ai-cli-icons/qwen.svg';
import openaiIcon from '../resources/ai-cli-icons/openai.svg';
import clineIcon from '../resources/ai-cli-icons/cline.svg';
import { ConsensusWebSocket, formatTokens, formatCost, STAGE_DISPLAY_NAMES } from './consensus-websocket';
import { MemoryDashboard } from './components/memory-dashboard';
import { NeuralConsciousness } from './neural-consciousness';
import { analyticsDashboard } from './analytics';
import { GitUI } from './git-ui';
import { VSCodeSCMView } from './vscode-scm-view';
import './notification.css';
import { FileExplorer } from './file-explorer';
import { VSCodeFileExplorer } from './vs-file-explorer';
import { VSCodeExplorerExact } from './vscode-explorer-exact';
import { EditorTabs } from './editor-tabs';
import { StatusBar } from './status-bar';
import { ttydTerminalPanel } from './components/TTYDTerminalPanel';

// Current opened folder state
let currentOpenedFolder: string | null = null;

// Expose to window for TTYDTerminalPanel
(window as any).currentOpenedFolder = currentOpenedFolder;

// Add welcome view styles
const addWelcomeStyles = () => {
    if (!document.getElementById('welcome-view-styles')) {
        const style = document.createElement('style');
        style.id = 'welcome-view-styles';
        style.textContent = `
            .welcome-view {
                padding: 20px;
                height: 100%;
                display: flex;
                flex-direction: column;
            }
            .welcome-content {
                color: var(--vscode-foreground, #cccccc);
                font-size: 13px;
                line-height: 1.5;
            }
            .welcome-message {
                margin: 0 0 12px 0;
                color: var(--vscode-foreground, #cccccc);
            }
            .welcome-button {
                display: block;
                width: 100%;
                padding: 6px 14px;
                margin: 8px 0;
                background: var(--vscode-button-secondaryBackground, #3a3d41);
                color: var(--vscode-button-secondaryForeground, #cccccc);
                border: 1px solid var(--vscode-button-border, transparent);
                border-radius: 2px;
                cursor: pointer;
                font-size: 13px;
                text-align: center;
            }
            .welcome-button.primary {
                background: var(--vscode-button-background, #0e639c);
                color: var(--vscode-button-foreground, #ffffff);
            }
            .welcome-button:hover {
                background: var(--vscode-button-hoverBackground, #1177bb);
            }
            .welcome-hint {
                margin: 12px 0;
                font-size: 12px;
                color: var(--vscode-descriptionForeground, #969696);
            }
            .welcome-section {
                margin: 20px 0;
                padding-top: 20px;
                border-top: 1px solid var(--vscode-widget-border, #303031);
            }
            .welcome-text {
                margin: 0 0 8px 0;
                color: var(--vscode-foreground, #cccccc);
            }
            .welcome-docs {
                margin: 20px 0 0 0;
                font-size: 12px;
                color: var(--vscode-descriptionForeground, #969696);
            }
            .welcome-link {
                color: var(--vscode-textLink-foreground, #3794ff);
                text-decoration: none;
            }
            .welcome-link:hover {
                text-decoration: underline;
            }
        `;
        document.head.appendChild(style);
    }
};

// Clone repository function
(window as any).cloneRepository = async () => {
    console.log('Clone repository clicked');
    // TODO: Implement clone repository dialog
    alert('Clone repository feature coming soon!');
};

// Initialize welcome styles
addWelcomeStyles();

// Simple input dialog function for VS Code-style prompts
function showInputDialog(title: string, message: string, defaultValue: string = ''): Promise<string | null> {
    return new Promise((resolve) => {
        // Create overlay
        const overlay = document.createElement('div');
        overlay.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0, 0, 0, 0.5);
            z-index: 10000;
            display: flex;
            align-items: center;
            justify-content: center;
        `;
        
        // Create dialog
        const dialog = document.createElement('div');
        dialog.style.cssText = `
            background: #252526;
            border: 1px solid #007acc;
            border-radius: 4px;
            padding: 20px;
            min-width: 400px;
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
        `;
        
        // Title
        const titleEl = document.createElement('h3');
        titleEl.style.cssText = `
            margin: 0 0 10px 0;
            color: #cccccc;
            font-size: 14px;
            font-weight: normal;
        `;
        titleEl.textContent = title;
        dialog.appendChild(titleEl);
        
        // Message
        const messageEl = document.createElement('div');
        messageEl.style.cssText = `
            color: #969696;
            font-size: 13px;
            margin-bottom: 15px;
        `;
        messageEl.textContent = message;
        dialog.appendChild(messageEl);
        
        // Input
        const input = document.createElement('input');
        input.type = 'text';
        input.value = defaultValue;
        input.style.cssText = `
            width: 100%;
            padding: 6px 8px;
            background: #3c3c3c;
            border: 1px solid #3c3c3c;
            color: #cccccc;
            font-size: 13px;
            border-radius: 2px;
            outline: none;
            box-sizing: border-box;
        `;
        input.addEventListener('focus', () => {
            input.style.borderColor = '#007acc';
        });
        input.addEventListener('blur', () => {
            input.style.borderColor = '#3c3c3c';
        });
        dialog.appendChild(input);
        
        // Buttons
        const buttons = document.createElement('div');
        buttons.style.cssText = `
            display: flex;
            justify-content: flex-end;
            gap: 10px;
            margin-top: 20px;
        `;
        
        const cancelBtn = document.createElement('button');
        cancelBtn.textContent = 'Cancel';
        cancelBtn.style.cssText = `
            padding: 6px 14px;
            background: #3c3c3c;
            border: 1px solid #3c3c3c;
            color: #cccccc;
            border-radius: 2px;
            cursor: pointer;
            font-size: 13px;
        `;
        cancelBtn.onclick = () => {
            document.body.removeChild(overlay);
            resolve(null);
        };
        
        const okBtn = document.createElement('button');
        okBtn.textContent = 'OK';
        okBtn.style.cssText = `
            padding: 6px 14px;
            background: #007acc;
            border: 1px solid #007acc;
            color: white;
            border-radius: 2px;
            cursor: pointer;
            font-size: 13px;
        `;
        okBtn.onclick = () => {
            document.body.removeChild(overlay);
            resolve(input.value);
        };
        
        buttons.appendChild(cancelBtn);
        buttons.appendChild(okBtn);
        dialog.appendChild(buttons);
        
        // Handle Enter key
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                document.body.removeChild(overlay);
                resolve(input.value);
            } else if (e.key === 'Escape') {
                document.body.removeChild(overlay);
                resolve(null);
            }
        });
        
        overlay.appendChild(dialog);
        document.body.appendChild(overlay);
        
        // Focus input and select text
        setTimeout(() => {
            input.focus();
            input.select();
        }, 0);
    });
}

// Create the exact Hive Consensus GUI layout
// Add global error handler to catch errors before webpack-dev-server
window.addEventListener('error', (event) => {
  // Silently handle Event object errors (these are harmless)
  if (event.error && (event.error instanceof Event || event.error.constructor.name.includes('Event'))) {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
  
  // Silently handle errors with [object Event] in message
  if (event.message && event.message.includes('[object Event]')) {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
  
  // Only log actual errors, not Event objects
  console.error('Error:', event.message, event.error);
}, true);

// Also catch unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
  // Silently handle Event object rejections
  if (event.reason instanceof Event) {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
  
  // Silently handle [object Event] string rejections
  const reasonStr = Object.prototype.toString.call(event.reason);
  if (reasonStr.includes('Event') || (event.reason && event.reason.toString && event.reason.toString().includes('[object Event]'))) {
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
  
  // Only log actual promise rejections, not Event objects
  console.error('Unhandled Promise Rejection:', event.reason);
}, true);

document.body.innerHTML = `
<div class="hive-consensus-gui">
  <!-- Title Bar -->
  <div class="title-bar">
    <div class="title-bar-left">
    </div>
    <div class="title-bar-center">
      <div class="title-logo">
        <img src="${hiveLogo}" alt="HiveTechs Logo" style="width: 24px; height: 24px; object-fit: contain; border-radius: 4px;" />
      </div>
      <span class="title-text">Hive Consensus</span>
    </div>
    <div class="title-bar-right"></div>
  </div>

  <!-- Main Content Area - Exact Dioxus Layout -->
  <div class="main-content">
    <!-- Left Sidebar - VS Code style with activity bar + sidebar panel -->
    <div class="sidebar" id="left-sidebar">
      <!-- Activity buttons -->
      <div class="activity-bar-unified">
        <!-- File & Git Section -->
        <button class="activity-btn" data-view="explorer" aria-label="Explorer">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.5 0h-9L7 1.5V6H2.5L1 7.5v15.07L2.5 24h12.07L16 22.57V18h4.7l1.3-1.43v-14l-1.3-1.43L17.5 0zm0 2.12l2.38 2.38v12.38l-2.38 2.38H16v-10l-1.5-1.5H8V1.5l1.5-.08H17.5z"/>
          </svg>
          <span class="activity-tooltip">Explorer (Ctrl+Shift+E)</span>
        </button>
        <button class="activity-btn" data-view="git" aria-label="Source Control">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M21.007 8.222A3.738 3.738 0 0 0 15.045 5.2a3.737 3.737 0 0 0 1.156 6.583 2.988 2.988 0 0 1-2.668 1.67h-2.99a4.456 4.456 0 0 0-2.989 1.165V7.4a3.737 3.737 0 1 0-1.494 0v9.117a3.776 3.776 0 1 0 1.816.099 2.99 2.99 0 0 1 2.668-1.667h2.99a4.484 4.484 0 0 0 4.223-3.039 3.736 3.736 0 0 0 3.25-3.687zM4.565 3.738a2.242 2.242 0 1 1 4.484 0 2.242 2.242 0 0 1-4.484 0zm4.484 16.441a2.242 2.242 0 1 1-4.484 0 2.242 2.242 0 0 1 4.484 0zm8.221-9.715a2.242 2.242 0 1 1 0-4.485 2.242 2.242 0 0 1 0 4.485z"/>
          </svg>
          <span class="activity-tooltip">Source Control (Ctrl+Shift+G)</span>
        </button>
        
        <div class="sidebar-divider"></div>
        
        <!-- Analytics, Memory and CLI Tools Section -->
        <button class="activity-btn" data-view="analytics" aria-label="Analytics">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 13h2v8H3zm4-8h2v16H7zm4-2h2v18h-2zm4 4h2v14h-2zm4-2h2v16h-2z"/>
          </svg>
          <span class="activity-tooltip">Analytics</span>
        </button>
        <button class="activity-btn" data-view="memory" aria-label="Memory">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <!-- Database/Memory icon similar to VS Code's database extension icon -->
            <path d="M12 3C7.58 3 4 4.79 4 7s3.58 4 8 4 8-1.79 8-4-3.58-4-8-4zm0 6c-3.31 0-6-1.34-6-3s2.69-3 6-3 6 1.34 6 3-2.69 3-6 3z"/>
            <path d="M4 9v3c0 2.21 3.58 4 8 4s8-1.79 8-4V9c-1.69 1.24-4.66 2-8 2s-6.31-.76-8-2z"/>
            <path d="M4 14v3c0 2.21 3.58 4 8 4s8-1.79 8-4v-3c-1.69 1.24-4.66 2-8 2s-6.31-.76-8-2z"/>
            <!-- Neural network nodes overlaid to represent AI memory -->
            <circle cx="12" cy="7" r="1" opacity="0.8"/>
            <circle cx="9" cy="12" r="0.5" opacity="0.6"/>
            <circle cx="15" cy="12" r="0.5" opacity="0.6"/>
            <circle cx="12" cy="17" r="0.5" opacity="0.6"/>
          </svg>
          <span class="activity-tooltip">Memory Service</span>
        </button>
        <button class="activity-btn" data-view="cli-tools" aria-label="AI CLI Tools">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M22 14H21C21 10.13 17.87 7 14 7H13V5.73C13.6 5.39 14 4.74 14 4C14 2.9 13.11 2 12 2S10 2.9 10 4C10 4.74 10.4 5.39 11 5.73V7H10C6.13 7 3 10.13 3 14H2C1.45 14 1 14.45 1 15V18C1 18.55 1.45 19 2 19H3V20C3 21.1 3.9 22 5 22H19C20.1 22 21 21.1 21 20V19H22C22.55 19 23 18.55 23 18V15C23 14.45 22.55 14 22 14M8.5 13C9.33 13 10 13.67 10 14.5S9.33 16 8.5 16S7 15.33 7 14.5S7.67 13 8.5 13M15.5 13C16.33 13 17 13.67 17 14.5S16.33 16 15.5 16S14 15.33 14 14.5S14.67 13 15.5 13M8 19L10 17H14L16 19H8Z"/>
          </svg>
          <span class="activity-tooltip">AI CLI Tools</span>
        </button>
        
        <div class="sidebar-divider"></div>
        
        <!-- AI CLI Tool Quick Launch Icons -->
        <div class="ai-cli-icons-section">
          <button class="activity-btn cli-quick-launch claude-icon" data-tool="claude-code" aria-label="Claude Code">
            <img src="${claudeIcon}" width="24" height="24" alt="Claude" style="object-fit: contain;" />
            <span class="activity-tooltip">Claude Code</span>
          </button>
          <button class="activity-btn cli-quick-launch gemini-icon" data-tool="gemini-cli" aria-label="Gemini CLI">
            <img src="${geminiIcon}" width="24" height="24" alt="Gemini" style="object-fit: contain;" />
            <span class="activity-tooltip">Gemini CLI</span>
          </button>
          <button class="activity-btn cli-quick-launch" data-tool="grok" aria-label="Grok">
            <img src="${grokIcon}" width="24" height="24" alt="Grok" style="object-fit: contain;" />
            <span class="activity-tooltip">Grok CLI</span>
          </button>
          <button class="activity-btn cli-quick-launch" data-tool="qwen-code" aria-label="Qwen Code">
            <img src="${qwenIcon}" width="24" height="24" alt="Qwen" style="object-fit: contain;" />
            <span class="activity-tooltip">Qwen Code</span>
          </button>
          <button class="activity-btn cli-quick-launch" data-tool="openai-codex" aria-label="OpenAI Codex">
            <img src="${openaiIcon}" width="24" height="24" alt="OpenAI" style="object-fit: contain;" />
            <span class="activity-tooltip">OpenAI Codex</span>
          </button>
          <button class="activity-btn cli-quick-launch" data-tool="cline" aria-label="Cline">
            <img src="${clineIcon}" width="24" height="24" alt="Cline" style="object-fit: contain;" />
            <span class="activity-tooltip">Cline</span>
          </button>
        </div>
        
        <!-- Bottom section with fixed positioning -->
        <div class="sidebar-bottom-section" style="position: absolute; bottom: 0; width: 100%;">
          <div class="sidebar-divider"></div>
          <button class="activity-btn" data-view="settings" aria-label="Settings">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.8,11.69,4.8,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z"/>
            </svg>
            <span class="activity-tooltip">Settings</span>
          </button>
        </div>
      </div>
      
      <!-- Sidebar Panel Content (VS Code style) -->
      <div class="sidebar-panel" id="sidebar-panel">
        <!-- Explorer Panel -->
        <div class="sidebar-section" id="explorer-sidebar" style="display: none;">
          <div class="sidebar-header">
            <span class="sidebar-title">EXPLORER</span>
            <div class="sidebar-actions">
              <button class="sidebar-action" title="New File" aria-label="New File">
                <i class="codicon codicon-new-file"></i>
              </button>
              <button class="sidebar-action" title="New Folder" aria-label="New Folder">
                <i class="codicon codicon-new-folder"></i>
              </button>
              <button class="sidebar-action" title="Refresh Explorer" aria-label="Refresh Explorer">
                <i class="codicon codicon-refresh"></i>
              </button>
              <button class="sidebar-action" title="Collapse Folders in Explorer" aria-label="Collapse All">
                <i class="codicon codicon-collapse-all"></i>
              </button>
            </div>
          </div>
          <div class="sidebar-content" id="explorer-content">
            <!-- Explorer tree will be rendered here -->
          </div>
        </div>
        
        <!-- Git Panel -->
        <div class="sidebar-section" id="git-sidebar" style="display: none;">
          <div class="sidebar-header">
            <span class="sidebar-title">SOURCE CONTROL</span>
            <div class="sidebar-actions">
              <!-- Removed redundant commit and refresh buttons -->
            </div>
          </div>
          <div class="sidebar-content" id="git-content">
            <!-- Git UI will be rendered here -->
          </div>
        </div>
      </div>
    </div>

    <!-- Center Area (Editor + Terminal) -->
    <div class="center-area" id="center-area" style="position: relative;">
      <!-- Collapse button for center area (positioned on right to avoid tab arrows) -->
      <button class="panel-collapse-btn" id="toggle-center-area" title="Collapse Editor" style="right: 10px; left: auto;">−</button>
      
      <!-- Resize handle removed - using fixed flex layout instead -->
      
      <!-- Editor Area -->
      <div class="editor-area" id="editor-area">
        <!-- Editor tabs and content will be mounted here -->
      </div>
      
      <!-- Analytics Panel (Hidden by default) -->
      <div id="analytics-panel" class="panel-content" style="display: none;">
        <!-- Analytics content will be mounted here -->
      </div>

      <!-- Terminal Section (Bottom, resizable) - Hidden since we use System Log in TTYD panel -->
      <div class="terminal-section" id="terminal-section" style="height: 200px; display: none;">
        <div class="resize-handle horizontal-resize" id="terminal-resize"></div>
        <div class="terminal-header">
          <button class="collapse-btn" id="toggle-terminal" title="Toggle Terminal">−</button>
          <span class="terminal-title">TERMINAL</span>
          <div class="terminal-controls">
            <button class="terminal-btn" id="close-terminal" title="Close">×</button>
          </div>
        </div>
        <div class="terminal-content" id="terminal-content">
          <div id="terminal-output">
            <div class="terminal-line">[${new Date().toLocaleTimeString()}] Hive Consensus initialized</div>
            <div class="terminal-line" id="backend-server-line">[${new Date().toLocaleTimeString()}] Backend server: discovering port...</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Isolated Terminal Panel (modeled after consensus panel) -->
    <div class="isolated-terminal-panel" id="isolated-terminal-panel">
      <!-- Collapse button for entire panel -->
      <button class="panel-collapse-btn" id="toggle-isolated-terminal" title="Collapse Terminal Panel">−</button>
      
      <!-- Resize handle removed - using fixed flex layout instead -->
      
      <!-- Terminal tabs header -->
      <div class="isolated-terminal-header" style="height: 35px; background: #252526; display: flex; align-items: center; border-bottom: 1px solid #3c3c3c; padding-left: 30px;">
        <!-- Left arrow for tab navigation -->
        <button class="tab-nav-arrow" id="tab-nav-left" title="Scroll tabs left" style="display: none; padding: 0 8px; background: transparent; border: none; color: #969696; cursor: pointer; font-size: 14px; height: 100%;">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <path d="M8 2 L4 6 L8 10" stroke="currentColor" stroke-width="2" fill="none"/>
          </svg>
        </button>
        
        <!-- Tab container with overflow hidden -->
        <div class="isolated-terminal-tabs-wrapper" style="flex: 1; position: relative; overflow: hidden;">
          <div class="isolated-terminal-tabs" id="isolated-terminal-tabs" style="display: flex; align-items: center; transition: transform 0.3s ease; white-space: nowrap;">
            <!-- Tabs will be inserted here -->
          </div>
        </div>
        
        <!-- Right arrow for tab navigation -->
        <button class="tab-nav-arrow" id="tab-nav-right" title="Scroll tabs right" style="display: none; padding: 0 8px; background: transparent; border: none; color: #969696; cursor: pointer; font-size: 14px; height: 100%;">
          <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
            <path d="M4 2 L8 6 L4 10" stroke="currentColor" stroke-width="2" fill="none"/>
          </svg>
        </button>
        <!-- System Log toggle button -->
        <button class="isolated-terminal-system-log-toggle" id="isolated-terminal-system-log-toggle" title="Toggle System Log (📊)" style="padding: 0 8px; background: transparent; border: none; color: #cccccc; cursor: pointer; font-size: 14px; transition: all 0.2s ease;">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <!-- Terminal/log icon similar to VS Code's output panel -->
            <path d="M2 2v12h12V2H2zm11 11H3V3h10v10z"/>
            <path d="M4 5h8v1H4zm0 2h8v1H4zm0 2h6v1H4zm0 2h7v1H4z"/>
          </svg>
        </button>
        
        <button class="isolated-terminal-new-tab" id="isolated-terminal-new-tab" title="New Terminal" style="padding: 0 10px; background: transparent; border: none; color: #969696; cursor: pointer; font-size: 18px;">+</button>
      </div>
      
      <!-- Terminal content area -->
      <div class="isolated-terminal-content" id="isolated-terminal-content" style="flex: 1; position: relative; background: #1a1a1a;">
        <!-- Terminal instances will be inserted here -->
      </div>
    </div>

    <!-- Right Panel (Consensus Chat) -->
    <div class="consensus-chat-panel" id="consensus-chat">
      <!-- Collapse button for entire panel -->
      <button class="panel-collapse-btn" id="toggle-consensus-panel" title="Toggle Panel">−</button>
      
      <!-- Vertical resize handle for consensus panel -->
      <div class="resize-handle vertical-resize" id="consensus-resize"></div>
      
      <!-- Neural Consciousness at top right -->
      <div id="neural-consciousness-container" style="height: 200px; width: 100%; border-bottom: 1px solid var(--border-color);">
        <!-- Neural Consciousness will be mounted here -->
      </div>
      
      <!-- Progress Bars below Neural Consciousness -->
      <div class="progress-section" id="progress-section">
        <div class="progress-header">
          <button class="collapse-btn" id="toggle-progress" title="Toggle Progress">−</button>
          <img src="${hiveLogo}" alt="Hive" 
               style="width: 20px; height: 20px; object-fit: contain; border-radius: 3px;" />
          <span>Consensus Progress</span>
          <span id="consensus-type" style="margin-left: 10px; color: var(--accent-color); font-weight: 600;"></span>
        </div>
        <div class="progress-content" id="progress-content">
          <div class="profile-display" id="active-profile-display">
            <span class="profile-label">Profile:</span>
            <span class="profile-name" id="active-profile-name">Loading...</span>
          </div>
          <div class="pipeline-stages">
          <div class="stage" data-stage="generator">
            <div class="stage-progress">
              <div class="stage-label">
                <span>⚡ Generator</span>
                <span class="stage-status" id="generator-status">ready</span>
              </div>
              <div class="progress-bar"><div class="progress-fill" id="generator-progress"></div></div>
              <div class="stage-model" id="generator-model">--</div>
            </div>
          </div>
          <div class="stage" data-stage="refiner">
            <div class="stage-progress">
              <div class="stage-label">
                <span>✨ Refiner</span>
                <span class="stage-status" id="refiner-status">ready</span>
              </div>
              <div class="progress-bar"><div class="progress-fill" id="refiner-progress"></div></div>
              <div class="stage-model" id="refiner-model">--</div>
            </div>
          </div>
          <div class="stage" data-stage="validator">
            <div class="stage-progress">
              <div class="stage-label">
                <span>🔍 Validator</span>
                <span class="stage-status" id="validator-status">ready</span>
              </div>
              <div class="progress-bar"><div class="progress-fill" id="validator-progress"></div></div>
              <div class="stage-model" id="validator-model">--</div>
            </div>
          </div>
          <div class="stage" data-stage="curator">
            <div class="stage-progress">
              <div class="stage-label">
                <span>💎 Curator</span>
                <span class="stage-status" id="curator-status">ready</span>
              </div>
              <div class="progress-bar"><div class="progress-fill" id="curator-progress"></div></div>
              <div class="stage-model" id="curator-model">--</div>
            </div>
          </div>
        </div>
        <div class="consensus-stats">
          <div class="stat-item">
            <span class="stat-label">Tokens:</span>
            <span class="stat-value" id="token-count">0</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Cost:</span>
            <span class="stat-value" id="cost-count">$0.00</span>
          </div>
        </div>
        </div>
      </div>

      <!-- Chat Area -->
      <div class="chat-area">
        <div class="chat-header">
          <span>CONSENSUS CHAT</span>
          <button class="consensus-settings-btn" id="consensus-settings-btn" title="Consensus Settings">⚙️</button>
        </div>
        <div class="chat-content" id="chat-content">
          <div class="chat-message system">
            <div class="message-time">[${new Date().toLocaleTimeString()}]</div>
            <div class="message-content">Hive Consensus ready for queries</div>
          </div>
        </div>
        <div class="chat-input-area">
          <textarea id="chat-input" placeholder="Enter your question..." rows="2" style="resize: vertical; min-height: 40px; max-height: 200px; width: calc(100% - 200px); padding: 8px; font-family: inherit; font-size: 14px;"></textarea>
          <button id="send-chat" class="send-btn">Send</button>
          <button id="test-progress" class="send-btn" style="background: #FFC107; margin-left: 5px;">Test Progress</button>
        </div>
      </div>
    </div>
  </div>

  <!-- Status Bar -->
  <div class="status-bar">
    <div class="status-bar-left">
      <div class="status-item" id="status-git-branch" style="display: none;">
        <span class="status-icon">🌿</span>
        <span id="branch-name">main</span>
      </div>
      <div class="status-item" id="status-git-warnings" style="display: none;">
        <span class="status-icon">⚠️</span>
        <span id="warning-count">0</span>
      </div>
      <div class="status-item" id="status-git-errors" style="display: none;">
        <span class="status-icon">🚫</span>
        <span id="error-count">0</span>
      </div>
    </div>
    <div class="status-bar-center">
      <div class="status-item">
        <span id="status-user">Not logged in</span>
      </div>
      <div class="status-divider">|</div>
      <div class="status-item">
        <span id="status-plan">Free</span>
      </div>
      <div class="status-divider">|</div>
      <div class="status-item">
        <span id="status-conversations">-- remaining</span>
      </div>
    </div>
    <div class="status-bar-right">
      <div class="status-item">
        <span>Rust Backend: </span>
        <span id="backend-status">Connecting...</span>
      </div>
      <div class="status-item">
        <span class="status-icon">⚡</span>
        <span>IPC</span>
      </div>
    </div>
  </div>
</div>

<!-- Consensus Settings Modal -->
<div id="consensus-settings-modal" class="modal-overlay" style="display: none;">
  <div class="modal-content" style="width: 400px;">
    <div class="modal-header">
      <h3>Consensus Settings</h3>
      <button class="modal-close" id="close-consensus-settings">×</button>
    </div>
    <div class="modal-body">
      <div class="form-group">
        <label for="max-consensus-rounds">Maximum Consensus Rounds</label>
        <input 
          type="number" 
          id="max-consensus-rounds" 
          min="1" 
          max="10" 
          value="3"
          style="width: 100%; padding: 8px; margin-top: 8px;"
        />
        <small class="help-text" style="color: var(--text-secondary); display: block; margin-top: 8px;">
          Number of rounds the AI models will deliberate before reaching consensus (default: 3)
        </small>
      </div>
      <div class="form-group" style="margin-top: 20px;">
        <p style="color: var(--text-secondary); font-size: 12px;">
          <strong>Round 1-2:</strong> Requires unanimous agreement<br/>
          <strong>Final Round:</strong> Accepts majority vote or curator decision<br/>
          <strong>Higher values:</strong> More thorough but uses more tokens
        </p>
      </div>
    </div>
    <div class="modal-footer" style="display: flex; justify-content: flex-end; gap: 10px; padding: 16px;">
      <button class="btn btn-secondary" id="cancel-consensus-settings">Cancel</button>
      <button class="btn btn-primary" id="save-consensus-settings">Save</button>
    </div>
  </div>
</div>
`;

// State management
let currentView = 'consensus';
let isConnected = false;
let isProcessing = false;
let conversationStartTime = 0;
let settingsModal: SettingsModal | null = null;

// TEST: Add a direct API test on window load
window.addEventListener('DOMContentLoaded', () => {
  setTimeout(() => {
    console.log('🔴🔴🔴 TESTING DIRECT API CALL');
    
    // Create a test button
    const testBtn = document.createElement('button');
    testBtn.textContent = 'TEST API';
    testBtn.style.position = 'fixed';
    testBtn.style.top = '10px';
    testBtn.style.right = '10px';
    testBtn.style.zIndex = '9999';
    testBtn.style.backgroundColor = 'red';
    testBtn.style.color = 'white';
    testBtn.style.padding = '10px';
    
    testBtn.onclick = async () => {
      console.log('🔴🔴🔴 TEST BUTTON CLICKED');
      
      // Test 1: Simple sync test
      try {
        if ((window as any).testAPI) {
          console.log('🔴🔴🔴 TEST 1: Calling testAPI.test()');
          const syncResult = (window as any).testAPI.test();
          console.log('🔴🔴🔴 TEST 1 RESULT:', syncResult);
        }
      } catch (e) {
        console.log('🔴🔴🔴 TEST 1 ERROR:', e);
      }
      
      // Test 2: Async test via testAPI
      try {
        if ((window as any).testAPI && (window as any).testAPI.testAsync) {
          console.log('🔴🔴🔴 TEST 2: Calling testAPI.testAsync()');
          const asyncResult = await (window as any).testAPI.testAsync();
          console.log('🔴🔴🔴 TEST 2 RESULT:', asyncResult);
          
          if (asyncResult && asyncResult.response) {
            addChatMessage('TEST ASYNC SUCCESS', false);
            addChatMessage(asyncResult.response, true);
          }
        }
      } catch (e) {
        console.log('🔴🔴🔴 TEST 2 ERROR:', e);
      }
      
      // Test 3: Direct IPC via window.api
      try {
        if ((window as any).api && (window as any).api.invoke) {
          console.log('🔴🔴🔴 TEST 3: Direct api.invoke()');
          const directResult = await (window as any).api.invoke('backend-consensus-quick', {
            query: 'TEST: Direct IPC call',
            profile: 'Free Also'
          });
          console.log('🔴🔴🔴 TEST 3 RESULT:', directResult);
          
          if (directResult && directResult.response) {
            addChatMessage('TEST DIRECT SUCCESS', false);
            addChatMessage(directResult.response, true);
          }
        }
      } catch (e) {
        console.log('🔴🔴🔴 TEST 3 ERROR:', e);
      }
      
      // Test 4: Original backendAPI test
      try {
        if ((window as any).backendAPI && (window as any).backendAPI.runQuickConsensus) {
          console.log('🔴🔴🔴 TEST 4: backendAPI.runQuickConsensus EXISTS');
          const result = await (window as any).backendAPI.runQuickConsensus({
            query: 'TEST: What is 5 + 5?',
            profile: 'Free Also'
          });
          console.log('🔴🔴🔴 TEST 4 RESULT:', result);
          
          // Also add to chat
          if (result && result.response) {
            addChatMessage('TEST: What is 5 + 5?', false);
            addChatMessage(result.response, true);
          }
        } else {
          console.log('🔴🔴🔴 TEST 4: NO backendAPI.runQuickConsensus!');
          console.log('window.backendAPI:', (window as any).backendAPI);
        }
      } catch (error) {
        console.log('🔴🔴🔴 TEST 4 ERROR:', error);
        addChatMessage(`TEST ERROR: ${error}`, true);
      }
    };
    
    document.body.appendChild(testBtn);
    console.log('🔴🔴🔴 TEST BUTTON ADDED');
  }, 2000);
});

// Sidebar Panel Management
function toggleSidebarPanel(panelType: 'explorer' | 'git') {
    const sidebarPanel = document.getElementById('sidebar-panel');
    const explorerSidebar = document.getElementById('explorer-sidebar');
    const gitSidebar = document.getElementById('git-sidebar');
    
    if (!sidebarPanel || !explorerSidebar || !gitSidebar) return;
    
    // Check if this panel is already active
    const isCurrentlyActive = sidebarPanel.classList.contains('active') && 
                              document.getElementById(`${panelType}-sidebar`)?.style.display !== 'none';
    
    if (isCurrentlyActive) {
        // Hide sidebar panel
        sidebarPanel.classList.remove('active');
        explorerSidebar.style.display = 'none';
        gitSidebar.style.display = 'none';
    } else {
        // Show sidebar panel and activate the requested panel
        sidebarPanel.classList.add('active');
        
        // Hide all panels first
        explorerSidebar.style.display = 'none';
        gitSidebar.style.display = 'none';
        
        // Show the requested panel
        const targetPanel = document.getElementById(`${panelType}-sidebar`);
        if (targetPanel) {
            targetPanel.style.display = 'block';
            
            // Initialize the content if needed
            if (panelType === 'explorer') {
                const container = document.getElementById('explorer-content');
                if (container) {
                    console.log('Explorer panel activated, currentOpenedFolder:', currentOpenedFolder);
                    
                    // Ensure editor tabs exist first
                    if (!window.editorTabs) {
                        const editorArea = document.getElementById('editor-area');
                        if (editorArea) {
                            window.editorTabs = new EditorTabs(editorArea);
                            console.log('Created editorTabs instance');
                        }
                    }
                    
                    // Check if we need to initialize or update the explorer
                    if (currentOpenedFolder) {
                        console.log('[Explorer Activation] currentOpenedFolder value:', currentOpenedFolder);
                        console.log('[Explorer Activation] currentOpenedFolder type:', typeof currentOpenedFolder);
                        
                        // Check if explorer exists and is showing the wrong folder
                        const needsUpdate = window.fileExplorer && window.fileExplorer.getCurrentPath() !== currentOpenedFolder;
                        
                        // If explorer doesn't exist OR container is empty OR showing wrong folder, create/update it
                        if (!window.fileExplorer || !container.querySelector('.explorer-folders-view') || needsUpdate) {
                            if (needsUpdate) {
                                console.log('Explorer showing wrong folder. Current:', window.fileExplorer.getCurrentPath(), 'Should be:', currentOpenedFolder);
                            }
                            console.log('Creating/updating file explorer for:', currentOpenedFolder);
                            container.innerHTML = ''; // Clear any existing content
                            window.fileExplorer = new VSCodeExplorerExact(container);
                            window.fileExplorer.initialize(currentOpenedFolder);
                            
                            // Connect to editor tabs when files are selected
                            window.fileExplorer.onFileSelect((filePath: string) => {
                                console.log('File selected:', filePath);
                                if (window.editorTabs) {
                                    // Wrap in try-catch to prevent errors from bubbling to webpack
                                    try {
                                        window.editorTabs.openFile(filePath).catch((err: any) => {
                                            console.error('Error opening file:', err);
                                        });
                                    } catch (err) {
                                        console.error('Error calling openFile:', err);
                                    }
                                } else {
                                    console.error('editorTabs not found');
                                }
                            });
                        } else {
                            console.log('Explorer already showing correct folder:', currentOpenedFolder);
                        }
                    } else {
                        // Show VS Code-style welcome screen
                        container.innerHTML = `
                            <div class="welcome-view">
                                <div class="welcome-content">
                                    <p class="welcome-message">You have not yet opened a folder.</p>
                                    <button class="welcome-button primary" onclick="window.openFolder()">
                                        Open Folder
                                    </button>
                                    <p class="welcome-hint">
                                        Opening a folder will close all currently open editors. To keep them open, add a folder instead.
                                    </p>
                                    <div class="welcome-section">
                                        <p class="welcome-text">You can clone a repository locally.</p>
                                        <button class="welcome-button" onclick="window.cloneRepository()">
                                            Clone Repository
                                        </button>
                                    </div>
                                    <p class="welcome-docs">
                                        To learn more about how to use Git and source control in VS Code 
                                        <a href="https://code.visualstudio.com/docs/editor/versioncontrol" target="_blank" class="welcome-link">read our docs</a>.
                                    </p>
                                </div>
                            </div>
                        `;
                    }
                    
                    // Connect add file/folder buttons
                    const addFileBtn = document.querySelector('.sidebar-actions button[title="New File"]');
                    const addFolderBtn = document.querySelector('.sidebar-actions button[title="New Folder"]');
                    const refreshBtn = document.querySelector('.sidebar-actions button[title="Refresh Explorer"]');
                    const collapseBtn = document.querySelector('.sidebar-actions button[title="Collapse Folders in Explorer"]');
                    
                    addFileBtn?.addEventListener('click', async () => {
                        console.log('Add file clicked');
                        if (window.fileExplorer) {
                            // Create a simple input dialog
                            const fileName = await showInputDialog('New File', 'Enter file name:', 'untitled.txt');
                            if (fileName && fileName.trim()) {
                                console.log('Creating new file:', fileName);
                                await window.fileExplorer.createFile(fileName.trim());
                            }
                        }
                    });
                    
                    addFolderBtn?.addEventListener('click', async () => {
                        console.log('Add folder clicked');
                        if (window.fileExplorer) {
                            // Create a simple input dialog
                            const folderName = await showInputDialog('New Folder', 'Enter folder name:', 'New Folder');
                            if (folderName && folderName.trim()) {
                                console.log('Creating new folder:', folderName);
                                await window.fileExplorer.createFolder(folderName.trim());
                            }
                        }
                    });
                    
                    refreshBtn?.addEventListener('click', () => {
                        console.log('Refresh clicked');
                        if (window.fileExplorer) {
                            window.fileExplorer.refresh();
                        }
                    });
                    
                    collapseBtn?.addEventListener('click', () => {
                        console.log('Collapse all clicked');
                        if (window.fileExplorer && window.fileExplorer.collapseAll) {
                            window.fileExplorer.collapseAll();
                        }
                    });
                }
            } else if (panelType === 'git') {
                const container = document.getElementById('git-content');
                if (container) {
                    // If we don't have a git UI yet, or need to refresh it
                    if (!window.gitUI) {
                        console.log('Git panel activated, currentOpenedFolder:', currentOpenedFolder);
                        
                        // If a folder is open, make sure Git is set to that folder
                        if (currentOpenedFolder && window.gitAPI) {
                            console.log('Setting Git folder to:', currentOpenedFolder);
                            window.gitAPI.setFolder(currentOpenedFolder).then(() => {
                                // Add a delay to ensure Git status is fully ready
                                setTimeout(() => {
                                    try {
                                        // Create the Git UI after setting the folder
                                        console.log('Creating VSCodeSCMView for folder:', currentOpenedFolder);
                                        window.gitUI = new VSCodeSCMView(container);
                                        window.scmView = window.gitUI;
                                        console.log('VSCodeSCMView created successfully');
                                    } catch (error) {
                                        console.error('Failed to create VSCodeSCMView:', error);
                                        console.error('Error stack:', error.stack);
                                    }
                                }, 300);
                            }).catch(error => {
                                console.error('Failed to set Git folder:', error);
                            });
                        } else {
                            try {
                                // No folder open, create Git UI which will show welcome
                                console.log('Creating VSCodeSCMView (no folder)');
                                window.gitUI = new VSCodeSCMView(container);
                                window.scmView = window.gitUI;
                                console.log('VSCodeSCMView created successfully (no folder)');
                            } catch (error) {
                                console.error('Failed to create VSCodeSCMView (no folder):', error);
                                console.error('Error stack:', error.stack);
                            }
                        }
                    } else if (currentOpenedFolder && window.gitAPI) {
                        // Git UI exists but we need to ensure it's showing the right folder
                        console.log('Git UI exists, updating to folder:', currentOpenedFolder);
                        window.gitAPI.setFolder(currentOpenedFolder).then(() => {
                            // Add a delay to ensure Git status is fully ready
                            setTimeout(() => {
                                // Recreate the Git UI to show the updated folder
                                container.innerHTML = '';
                                window.gitUI = new VSCodeSCMView(container);
                                window.scmView = window.gitUI;
                            }, 300);
                        });
                    }
                }
            }
        }
    }
}
let dailyUsageCount = 0;
let dailyLimit = 5;
let totalTokens = 0;
let totalCost = 0;
let currentStageTokens = 0; // Track tokens for the current stage only
let activeProfile: any = null;
let consensusWebSocket: ConsensusWebSocket | null = null;
let neuralConsciousness: NeuralConsciousness | null = null;
let currentStreamContent: Map<string, string> = new Map();
// Track progress intervals to prevent overlapping animations
let progressIntervals: Map<string, NodeJS.Timeout> = new Map();

// Feature flag for Neural Consciousness (can be toggled without breaking app)
let ENABLE_NEURAL_CONSCIOUSNESS = true;

// Conversation context management (like Dioxus implementation)
let currentConversationId: string | null = null;
let conversationHistory: Array<{role: string, content: string}> = [];
let conversationMessages: Array<{question: string, answer: string}> = [];

// DOM elements - Updated for new layout
const terminalOutput = document.getElementById('terminal-output')!;
const backendStatus = document.getElementById('backend-status')!;
const chatContent = document.getElementById('chat-content')!;

// Utility functions
function generateConversationId(): string {
  // Generate a unique ID like the Rust implementation
  const timestamp = Date.now().toString(36);
  const randomPart = Math.random().toString(36).substring(2, 9);
  return `conv_${timestamp}_${randomPart}`;
}

function addLogEntry(message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') {
  const entry = document.createElement('div');
  entry.className = `terminal-line ${type}`;
  entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
  terminalOutput.appendChild(entry);
  terminalOutput.scrollTop = terminalOutput.scrollHeight;
}

// Declare runConsensusViaREST as a variable that will hold the function
let runConsensusViaREST: (query: string) => Promise<void>;

function addChatMessage(message: string, isSystem: boolean = false, messageType: 'info' | 'success' | 'warning' | 'error' | 'user' = 'user') {
  const messageDiv = document.createElement('div');
  
  // Determine the proper class based on message type
  let messageClass = 'assistant';
  if (!isSystem && messageType === 'user') {
    messageClass = 'user';
  } else if (messageType === 'error') {
    messageClass = 'error';
  } else if (messageType === 'warning') {
    messageClass = 'warning';
  } else if (messageType === 'success') {
    messageClass = 'success';
  } else if (messageType === 'info') {
    messageClass = 'info';
  } else if (isSystem) {
    messageClass = 'system';
  }
  
  messageDiv.className = `chat-message ${messageClass}`;
  
  const timestamp = new Date().toLocaleTimeString('en-US', { 
    hour: 'numeric', 
    minute: '2-digit',
    hour12: true 
  });
  
  messageDiv.innerHTML = `
    <div class="message-time">${timestamp}</div>
    <div class="message-content">${convertMarkdownToHTML(message)}</div>
  `;
  chatContent.appendChild(messageDiv);
  // Auto-scroll to bottom to show newest message
  requestAnimationFrame(() => {
    chatContent.scrollTop = chatContent.scrollHeight;
  });
}

// Helper function to auto-scroll chat to bottom
function autoScrollChat() {
  const chatContent = document.getElementById('chat-content');
  if (chatContent) {
    requestAnimationFrame(() => {
      chatContent.scrollTop = chatContent.scrollHeight;
    });
  }
}

// Enhanced markdown to HTML converter
function convertMarkdownToHTML(markdown: string): string {
  // First, remove any HTML artifacts that shouldn't be in the markdown
  let cleanedMarkdown = markdown
    .replace(/<h1 class="md-h1">/g, '# ')
    .replace(/<h2 class="md-h2">/g, '## ')
    .replace(/<h3 class="md-h3">/g, '### ')
    .replace(/<h4 class="md-h4">/g, '#### ')
    .replace(/<h5 class="md-h5">/g, '##### ')
    .replace(/<h6 class="md-h6">/g, '###### ')
    .replace(/<\/h[1-6]>/g, '');
  
  let html = cleanedMarkdown;
  
  // Headers (h1-h6)
  html = html.replace(/^###### (.*?)$/gm, '<h6 class="md-h6">$1</h6>');
  html = html.replace(/^##### (.*?)$/gm, '<h5 class="md-h5">$1</h5>');
  html = html.replace(/^#### (.*?)$/gm, '<h4 class="md-h4">$1</h4>');
  html = html.replace(/^### (.*?)$/gm, '<h3 class="md-h3">$1</h3>');
  html = html.replace(/^## (.*?)$/gm, '<h2 class="md-h2">$1</h2>');
  html = html.replace(/^# (.*?)$/gm, '<h1 class="md-h1">$1</h1>');
  
  // Code blocks with language support
  html = html.replace(/```([\w]*)?\n([\s\S]*?)```/g, (match, lang, code) => {
    const language = lang || 'plaintext';
    return `<pre class="code-block" data-lang="${language}"><code>${escapeHtml(code.trim())}</code></pre>`;
  });
  
  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code class="inline-code">$1</code>');
  
  // Bold and italic
  html = html.replace(/\*\*\*(.*?)\*\*\*/g, '<strong><em>$1</em></strong>'); // Bold + Italic
  html = html.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>'); // Bold
  html = html.replace(/\*(.*?)\*/g, '<em>$1</em>'); // Italic
  
  // Lists
  html = html.replace(/^\* (.*?)$/gm, '<li class="ul-item">$1</li>');
  html = html.replace(/^- (.*?)$/gm, '<li class="ul-item">$1</li>');
  html = html.replace(/^\d+\. (.*?)$/gm, '<li class="ol-item">$1</li>');
  
  // Wrap consecutive list items
  html = html.replace(/(<li class="ul-item">.*?<\/li>\s*)+/g, (match) => {
    return `<ul class="md-list">${match}</ul>`;
  });
  html = html.replace(/(<li class="ol-item">.*?<\/li>\s*)+/g, (match) => {
    return `<ol class="md-list">${match}</ol>`;
  });
  
  // Blockquotes
  html = html.replace(/^> (.*?)$/gm, '<blockquote class="md-blockquote">$1</blockquote>');
  
  // Horizontal rules
  html = html.replace(/^---$/gm, '<hr class="md-hr">');
  
  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="md-link" target="_blank">$1</a>');
  
  // Paragraphs (must be last)
  html = html.replace(/\n\n/g, '</p><p class="md-paragraph">');
  html = html.replace(/\n/g, '<br>');
  
  // Wrap in paragraph if not already wrapped
  if (!html.startsWith('<')) {
    html = `<p class="md-paragraph">${html}</p>`;
  }
  
  return html;
}

// Helper to escape HTML in code blocks
function escapeHtml(text: string): string {
  const map: { [key: string]: string } = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  };
  return text.replace(/[&<>"']/g, m => map[m]);
}

function updateStatus(text: string, className: string) {
  // Update status in terminal instead of old status indicator
  addLogEntry(`Status: ${text}`, className === 'error' ? 'error' : 'info');
}

function updateConnectionStatus(connected: boolean) {
  isConnected = connected;
  
  if (connected) {
    backendStatus.textContent = 'Connected';
    addLogEntry('✅ Backend connection established', 'info');
  } else {
    backendStatus.textContent = 'Connecting...';
    addLogEntry('🔄 Connecting to backend...', 'info');
  }
}

function updateStageStatus(stage: string, status: 'ready' | 'running' | 'completed' | 'error') {
  const stageElement = document.querySelector(`[data-stage="${stage}"]`);
  if (stageElement) {
    const progressFill = stageElement.querySelector('.progress-fill') as HTMLElement;
    const statusElement = document.getElementById(`${stage}-status`);
    
    // Update status text
    if (statusElement) {
      statusElement.textContent = status;
      statusElement.className = `stage-status ${status}`;
    }
    
    // Update progress bar
    switch (status) {
      case 'ready':
        progressFill.style.width = '0%';
        progressFill.classList.remove('running');
        progressFill.style.background = 'linear-gradient(90deg, var(--hive-yellow) 0%, var(--hive-blue) 100%)';
        addLogEntry(`🔄 ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage ready`, 'info');
        break;
      case 'running':
        progressFill.style.width = '50%';
        progressFill.classList.add('running');
        progressFill.style.background = 'linear-gradient(90deg, var(--terminal-warning) 0%, var(--terminal-info) 100%)';
        addLogEntry(`⚡ ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage running...`, 'info');
        break;
      case 'completed':
        progressFill.style.width = '100%';
        progressFill.classList.remove('running');
        progressFill.style.background = 'linear-gradient(90deg, var(--terminal-success) 0%, var(--terminal-info) 100%)';
        addLogEntry(`✅ ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage completed`, 'success');
        break;
      case 'error':
        progressFill.style.width = '0%';
        progressFill.classList.remove('running');
        progressFill.style.background = 'var(--terminal-error)';
        addLogEntry(`❌ ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage error`, 'error');
        break;
    }
  }
}

// Control panel button handlers (moved to after function definitions)

document.getElementById('settings-btn')?.addEventListener('click', () => {
  addLogEntry('⚙️ Opening settings...', 'info');
  openSettingsTab();
});

// Function to open settings as a full tab
function openSettingsTab() {
  console.log('[Settings] Opening settings as tab, editorTabs exists:', !!window.editorTabs);
  
  // Create the settings content as a tab
  const settingsContent = settingsModal.getSettingsTabContent();
  
  // Use the editor tabs system to open settings as a custom tab
  if (window.editorTabs) {
    console.log('[Settings] Calling openCustomTab');
    window.editorTabs.openCustomTab('settings', '⚙️ Settings', settingsContent, {
      isCloseable: true,
      onClose: () => {
        // Save any pending changes when tab is closed
        settingsModal.handleSave();
      }
    });
  } else {
    console.error('[Settings] EditorTabs not available, falling back to modal');
    // Fallback - show the modal if tabs aren't available
    // Just try to show it, initializeModal will be called if needed
    settingsModal.showModal();
  }
}

document.getElementById('memory-btn')?.addEventListener('click', () => {
  addLogEntry('Memory Service panel clicked', 'info');
  openMemoryDashboard();
});

// Button handlers (using IPC like before)
document.getElementById('test-connection-btn')?.addEventListener('click', async () => {
  if (isProcessing) return;
  
  isProcessing = true;
  updateStatus('Testing Connection...', 'processing');
  addLogEntry('🔗 Testing connection to Rust backend...', 'info');
  
  try {
    const result = await (window as any).backendAPI.testConnection();
    
    updateStatus('Connected!', 'success');
    updateConnectionStatus(true);
    addLogEntry(`✅ Connection successful! Echo: ${result.echo}`, 'success');
    addLogEntry(`⏱️  Response time: ${new Date(result.timestamp).toLocaleTimeString()}`, 'info');
    
  } catch (error) {
    updateStatus('Connection Failed', 'error');
    updateConnectionStatus(false);
    addLogEntry(`❌ Connection failed: ${error}`, 'error');
  } finally {
    isProcessing = false;
  }
});

document.getElementById('run-consensus-btn')?.addEventListener('click', async () => {
  if (isProcessing) {
    addLogEntry('⚠️ Consensus already in progress', 'warning');
    return;
  }
  
  if (!consensusWebSocket?.isConnected()) {
    addLogEntry('🔄 Connecting to consensus engine...', 'info');
    consensusWebSocket?.connect();
    setTimeout(() => {
      if (consensusWebSocket?.isConnected()) {
        document.getElementById('run-consensus-btn')?.click();
      } else {
        addLogEntry('❌ Failed to connect to consensus engine', 'error');
      }
    }, 1000);
    return;
  }
  
  isProcessing = true;
  conversationStartTime = performance.now();
  (window as any).consensusStartTime = Date.now();
  totalTokens = 0;
  totalCost = 0;
  currentStageTokens = 0;
  updateConsensusStats();
  resetStageStatus(); // This now properly resets ALL stages including curator
  updateStatus('Running Consensus...', 'processing');
  addLogEntry('🚀 Starting streaming consensus pipeline...', 'info');
  
  // Test query
  const testQuery = "What is the capital of France?";
  addChatMessage(testQuery, false);
  
  // Create new conversation if needed
  if (!currentConversationId) {
    currentConversationId = generateConversationId();
    addLogEntry(`📝 New conversation started: ${currentConversationId}`, 'info');
  }
  
  // Add to conversation history
  conversationHistory.push({ role: 'user', content: testQuery });
  
  // Get current profile from database (should always be loaded)
  const currentProfileName = activeProfile?.name || 'Balanced Performer';
  console.log('[Consensus] Using profile:', currentProfileName, 'activeProfile:', activeProfile);
  
  // Start consensus via WebSocket with conversation context
  consensusWebSocket.startConsensus(testQuery, currentProfileName, currentConversationId, conversationHistory);
});

// New conversation button handler
document.getElementById('new-conversation-btn')?.addEventListener('click', () => {
  currentConversationId = null;
  conversationHistory = [];
  conversationMessages = [];
  addLogEntry('🆕 Starting new conversation (context cleared)', 'info');
  addChatMessage('New conversation started. Previous context cleared.', true);
});

// Consensus Settings Modal Event Listeners
document.getElementById('consensus-settings-btn')?.addEventListener('click', async () => {
  const modal = document.getElementById('consensus-settings-modal');
  if (modal) {
    // Load current max rounds value from active profile
    try {
      const profileName = localStorage.getItem('activeProfile') || 'Free Also';
      const profile = await (window as any).settingsAPI.getProfile(profileName);
      const maxRoundsInput = document.getElementById('max-consensus-rounds') as HTMLInputElement;
      if (maxRoundsInput && profile) {
        maxRoundsInput.value = (profile.max_consensus_rounds || 3).toString();
      }
    } catch (error) {
      console.error('Error loading max rounds:', error);
    }
    modal.style.display = 'flex';
  }
});

document.getElementById('close-consensus-settings')?.addEventListener('click', () => {
  const modal = document.getElementById('consensus-settings-modal');
  if (modal) modal.style.display = 'none';
});

document.getElementById('cancel-consensus-settings')?.addEventListener('click', () => {
  const modal = document.getElementById('consensus-settings-modal');
  if (modal) modal.style.display = 'none';
});

document.getElementById('save-consensus-settings')?.addEventListener('click', async () => {
  const maxRoundsInput = document.getElementById('max-consensus-rounds') as HTMLInputElement;
  const maxRounds = parseInt(maxRoundsInput.value) || 3;
  
  // Save to active profile
  try {
    const profileName = localStorage.getItem('activeProfile') || 'Free Also';
    await (window as any).settingsAPI.updateProfileMaxRounds(profileName, maxRounds);
    
    // Close modal
    const modal = document.getElementById('consensus-settings-modal');
    if (modal) modal.style.display = 'none';
    
    // Show success message
    addChatMessage(`✅ Max consensus rounds updated to ${maxRounds}`, true);
  } catch (error) {
    console.error('Error saving max rounds:', error);
    addChatMessage('❌ Failed to update consensus settings', true);
  }
});

// TEST: Add IPC test before main consensus logic
document.getElementById('send-chat')?.addEventListener('click', async () => {
  const chatInput = document.getElementById('chat-input') as HTMLTextAreaElement;
  const query = chatInput.value.trim();
  
  if (!query || isProcessing) return;
  
  // TEST: Prove IPC mechanism works
  if (query.startsWith('TEST:')) {
    try {
      const testResult = await (window as any).backendAPI.testClaudeDebug(query);
      addChatMessage(`🟢 IPC TEST RESULT: ${testResult.result}`, true);
      return;
    } catch (error) {
      addChatMessage(`🔴 IPC TEST FAILED: ${error}`, true);
      return;
    }
  }
  
  chatInput.value = '';
  addChatMessage(query, false);
  
  // ALWAYS use DirectConsensusEngine, regardless of neural consciousness
  // Start neural consciousness + progress bars (straightforward approach)
  if (true) { // Force this path to always run for DirectConsensusEngine
    isProcessing = true;
    resetStageStatus(); // This now properly resets ALL stages
    
    // Only use neural consciousness if it exists
    if (neuralConsciousness) {
      await neuralConsciousness.show();
      // The actual stages (memory, synthesis, route/classification) will be controlled
      // by SimpleConsensusEngine based on real processing, not hardcoded timing
      // For now, just show the neural graphic is ready
      await neuralConsciousness.updatePhase('memory');
    }
    // SimpleConsensusEngine will send actual stage updates via IPC
    // The progress bars will be updated by the direct commands from DirectConsensusEngine
    
    // Set up real-time event listeners for deliberation feedback
    const eventHandler = (event: any, data: string) => {
      try {
        const message = JSON.parse(data);
        
        switch (message.type) {
          case 'llm_started':
            // Individual LLM activation
            updateStageStatus(message.llm, 'running');
            updateStageProgress(message.llm, 25);
            updateModelDisplay(message.llm, `Round ${message.round}: ${message.model.split('/').pop() || message.model}`);
            break;
            
          case 'token_update':
            // Real-time token updates (gas pump effect)
            totalTokens = message.tokens;
            totalCost = message.cost;
            updateConsensusStats();
            updateStageStatus(message.current_llm, 'completed');
            updateStageProgress(message.current_llm, 100);
            break;
            
          case 'streaming_token_update':
            // Live token updates during LLM streaming (gas pump effect)
            totalTokens += message.current_tokens;
            updateConsensusStats();
            break;
            
          case 'consensus_complete':
            // Final response display
            addChatMessage(message.result, true);
            break;
        }
      } catch (e) {
        console.error('Event handling error:', e);
      }
    };
    
    // Listen for deliberation events directly
    (window as any).api.receive('websocket-message', eventHandler);
    
    // Set up consensus event listeners for 4-stage pipeline
    const consensusAPI = (window as any).consensusAPI;
    if (consensusAPI) {
      console.log('📡 Setting up consensus event listeners');
      
      // Clean up any existing listeners
      consensusAPI.removeAllListeners();
      
      // ESC key handler for interrupting consensus (like Claude Code CLI)
      let escapeHandler: ((e: KeyboardEvent) => void) | null = null;
      
      // Consensus status handler (for tracking consensus type)
      consensusAPI.onConsensusStatus((data: any) => {
        console.log('📊 Consensus Status:', data);
        
        // Add escape handler during routing, conversing, stage_running, and curating states
        if ((data.consensusType === 'routing' || data.consensusType === 'conversing' || data.consensusType === 'stage_running' || data.consensusType === 'curating') && !escapeHandler) {
          escapeHandler = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
              console.log('🛑 ESC pressed - interrupting consensus');
              // Stop consensus and reset display
              window.consensusAPI?.interruptConsensus();
              // Reset consensus display
              const consensusTypeElement = document.getElementById('consensus-type');
              if (consensusTypeElement) {
                consensusTypeElement.textContent = '';
                consensusTypeElement.style.color = '';
                consensusTypeElement.style.animation = '';
                consensusTypeElement.style.fontWeight = '';
                consensusTypeElement.style.textShadow = '';
                consensusTypeElement.style.fontFamily = '';
                consensusTypeElement.style.whiteSpace = '';
              }
              // DON'T remove escape handler - keep it active for multiple uses
            }
          };
          document.addEventListener('keydown', escapeHandler);
        }
        
        // Remove escape handler when consensus complete
        if (data.achieved && escapeHandler) {
          document.removeEventListener('keydown', escapeHandler);
          escapeHandler = null;
        }
        
        // Update consensus type display with color coding
        const consensusTypeElement = document.getElementById('consensus-type');
        if (consensusTypeElement && data.consensusType) {
          let displayText = '';
          let color = '';
          
          switch(data.consensusType) {
            case 'unanimous':
              displayText = 'Unanimous';
              color = '#4CAF50'; // Green
              break;
            case 'majority':
              displayText = 'Majority';
              color = '#FFC107'; // Amber/Yellow
              break;
            case 'curator_override':
              displayText = 'Curator Decision';
              color = '#FF9800'; // Orange
              break;
            case 'pending':
              displayText = ''; // Don't show anything for pending state
              color = '';
              break;
            case 'conversing':
            case 'routing':
            case 'stage_running':
            case 'curating':
              const funPhrase = data.funPhrase || "processing";
              const animatedIcon = data.animatedIcon || "+";
              // Use fixed-width for symbol to prevent word bouncing
              const fixedSymbol = animatedIcon.padEnd(1, ' '); // Ensure consistent width
              displayText = `${fixedSymbol} ${funPhrase}...\n(esc to interrupt)`;
              color = '#FF9500'; // Orange like Claude Code CLI
              // Remove breathing, focus on fast symbol spinning
              consensusTypeElement.style.animation = '';
              consensusTypeElement.style.fontWeight = 'bold';
              consensusTypeElement.style.fontSize = '16px';
              consensusTypeElement.style.textShadow = '0 0 8px rgba(255, 149, 0, 0.8)';
              consensusTypeElement.style.fontFamily = 'monospace'; // Fixed-width font to prevent bouncing
              consensusTypeElement.style.whiteSpace = 'pre'; // Preserve spacing
              break;
            default:
              displayText = '';
              color = 'var(--accent-color)';
          }
          
          consensusTypeElement.textContent = displayText;
          consensusTypeElement.style.color = color;
          
          // Clear animations for non-conversing states
          if (data.consensusType !== 'conversing') {
            consensusTypeElement.style.animation = '';
            consensusTypeElement.style.fontWeight = '';
            consensusTypeElement.style.textShadow = '';
          }
        }
        
        // Clear on reset/new query
        if (data.reset) {
          const consensusTypeElement = document.getElementById('consensus-type');
          if (consensusTypeElement) {
            consensusTypeElement.textContent = '';
          }
        }
      });
      
      // SIMPLE visual update handler - one function to handle all updates
      consensusAPI.onVisualUpdate((data: any) => {
        console.log('🎨 RECEIVED Visual Update:', data.type, data);
        addLogEntry(`Visual Update: ${data.type} - ${JSON.stringify(data)}`, 'info');
        
        switch(data.type) {
          // Neural consciousness controls
          case 'neural-show':
            if (neuralConsciousness) neuralConsciousness.show();
            break;
            
          case 'neural-phase':
            if (neuralConsciousness) neuralConsciousness.updatePhase(data.phase);
            break;
            
          case 'neural-completion':
            if (neuralConsciousness) neuralConsciousness.showCompletion();
            break;
            
          case 'neural-hide':
            if (neuralConsciousness) neuralConsciousness.hide();
            break;
            
          // Stage progress bar controls
          case 'stage-start':
            // Update neural phase
            if (neuralConsciousness) neuralConsciousness.updatePhase(data.stage);
            // Update progress bar
            updateStageStatus(data.stage, 'running');
            updateStageProgress(data.stage, 10); // Start at 10%
            if (data.model) updateModelDisplay(data.stage, data.model);
            break;
            
          case 'stage-progress':
            // Update progress bar during API calls
            if (data.stage && data.progress !== undefined) {
              updateStageProgress(data.stage, data.progress);
            }
            break;
            
          case 'stage-complete':
            // Complete the stage progress bar
            updateStageStatus(data.stage, 'completed');
            updateStageProgress(data.stage, 100);
            // Update token display if provided
            if (data.tokens) {
              const tokenElement = document.querySelector(`.consensus-stage.${data.stage} .stage-tokens`);
              if (tokenElement) {
                tokenElement.textContent = `${data.tokens} tokens • $${(data.cost || 0).toFixed(4)}`;
              }
            }
            break;
            
          case 'final-stats':
            // Update total tokens and cost
            if (data.totalTokens !== undefined) totalTokens = data.totalTokens;
            if (data.totalCost !== undefined) totalCost = data.totalCost;
            updateConsensusStats();
            break;
        }
      });
      
      // Listen for consensus completion (final response)
      consensusAPI.onConsensusComplete((data: any) => {
        console.log('🎉 Consensus Complete:', data);
        
        // Show Neural Graphic completion
        if (neuralConsciousness) {
          neuralConsciousness.showCompletion();
          setTimeout(() => neuralConsciousness.hide(), 3000);
        }
        
        // Add the final response to chat
        if (data.response) {
          addChatMessage(data.response, true);
        }
        
        // Update final stats
        if (data.totalTokens) totalTokens = data.totalTokens;
        if (data.totalCost) totalCost = data.totalCost;
        updateConsensusStats();
      });
      
      // New iterative consensus event handlers
      consensusAPI.onRoundUpdate((data: any) => {
        console.log('🔄 Round Update:', data);
        addLogEntry(`Round ${data.round} started`, 'info');
        
        // Reset iterative stages (but NOT curator) for the new round
        if (data.round > 1) {
          ['generator', 'refiner', 'validator'].forEach(stage => {
            updateStageStatus(stage, 'ready');
            updateStageProgress(stage, 0);
          });
        }
        
        // Update round display in UI
        const roundIndicator = document.createElement('div');
        roundIndicator.className = 'consensus-round';
        roundIndicator.textContent = `Round ${data.round}`;
        roundIndicator.style.cssText = 'color: #3794ff; font-weight: bold; margin: 10px 0;';
        const chatMessages = document.getElementById('consensus-chat-messages');
        if (chatMessages) {
          chatMessages.appendChild(roundIndicator);
        }
      });
      
      consensusAPI.onStageUpdate((data: any) => {
        console.log('📊 Stage Update:', data);
        console.log('📊 Neural Graphic status:', neuralConsciousness ? 'exists' : 'missing');
        addLogEntry(`Stage ${data.stage}: ${data.status}`, 'info');
        
        // NEVER update curator visually unless it's really running
        if (data.stage === 'curator') {
          console.log('🎨 Curator stage update received:', data.status);
          // Only allow curator updates if status is 'running' or 'completed'
          if (data.status !== 'running' && data.status !== 'completed') {
            console.log('⚠️ Blocking premature curator visual update');
            return;
          }
        }
        
        // Update stage status using existing functions
        updateStageStatus(data.stage, data.status);
        
        // Show model name for this stage
        if (activeProfile && data.status === 'running') {
          const modelName = activeProfile[`${data.stage}`] || activeProfile[`${data.stage}_model`] || 'Model';
          updateModelDisplay(data.stage, modelName);
        }
        
        // CRITICAL: Update Neural Graphic to match the stage
        if (data.status === 'running') {
          console.log(`🧠 Updating Neural Graphic to phase: ${data.stage}`);
          if (neuralConsciousness) {
            // Map 'route' stage to 'classification' phase for neural graphic
            const neuralPhase = data.stage === 'route' ? 'classification' : data.stage;
            neuralConsciousness.updatePhase(neuralPhase as 'classification' | 'generator' | 'refiner' | 'validator' | 'curator');
          } else {
            console.warn('⚠️ Neural Graphic not initialized!');
          }
          
          // Clear any existing interval for this stage
          if (progressIntervals.has(data.stage)) {
            clearInterval(progressIntervals.get(data.stage)!);
            progressIntervals.delete(data.stage);
            console.log(`🔍 CLEARED existing interval for stage: ${data.stage}`);
          }
          
          // Start animating progress from 0 to 90
          console.log(`🔍 STARTING animation for stage: ${data.stage}`);
          updateStageProgress(data.stage, 0);
          let progress = 0;
          const interval = setInterval(() => {
            progress = Math.min(90, progress + 5);
            console.log(`🔍 Animating ${data.stage} progress to ${progress}%`);
            updateStageProgress(data.stage, progress);
            if (progress >= 90) {
              clearInterval(interval);
              progressIntervals.delete(data.stage);
            }
          }, 200);
          progressIntervals.set(data.stage, interval);
        } else if (data.status === 'completed') {
          console.log(`🔍 COMPLETING stage: ${data.stage}`);
          // Clear any existing interval for this stage
          if (progressIntervals.has(data.stage)) {
            clearInterval(progressIntervals.get(data.stage)!);
            progressIntervals.delete(data.stage);
          }
          updateStageProgress(data.stage, 100);
        }
      });
      
      consensusAPI.onConsensusStatus((data: any) => {
        console.log('🤔 Consensus Status:', data);
        addLogEntry(`Consensus check - Generator: ${data.generator}, Refiner: ${data.refiner}, Validator: ${data.validator}`, 'info');
        // Display consensus opinions in chat
        const consensusDiv = document.createElement('div');
        consensusDiv.className = 'consensus-check';
        consensusDiv.style.cssText = 'background: #1e1e1e; padding: 10px; margin: 10px 0; border-left: 3px solid #3794ff;';
        consensusDiv.innerHTML = `
          <div style="color: #969696; font-size: 12px; margin-bottom: 5px;">Can this be improved?</div>
          <div style="color: #cccccc;">
            🤖 Generator: <span style="color: ${data.generator === 'YES' ? '#f14c4c' : '#89d185'}">${data.generator}</span><br>
            🔧 Refiner: <span style="color: ${data.refiner === 'YES' ? '#f14c4c' : '#89d185'}">${data.refiner}</span><br>
            ✅ Validator: <span style="color: ${data.validator === 'YES' ? '#f14c4c' : '#89d185'}">${data.validator}</span><br>
            ${data.achieved ? '<span style="color: #89d185; font-weight: bold;">✨ Consensus Achieved!</span>' : '<span style="color: #f14c4c;">Continuing iteration...</span>'}
          </div>
        `;
        const chatMessages = document.getElementById('consensus-chat-messages');
        if (chatMessages) {
          chatMessages.appendChild(consensusDiv);
          chatMessages.scrollTop = chatMessages.scrollHeight;
        }
      });
    }
    
    // Call DirectConsensusEngine with animated progress bars
    try {
      console.log('🎯🎯🎯 Starting consensus with visual feedback');
      
      // Helper to animate a progress bar during an async operation
      const animateStage = async (stage: string, model: string, duration: number = 3000) => {
        updateStageStatus(stage, 'running');
        updateModelDisplay(stage, model);
        
        // Animate progress from 0 to 90 over the duration
        let progress = 0;
        const interval = 100; // Update every 100ms
        const increment = 90 / (duration / interval);
        
        const timer = setInterval(() => {
          progress = Math.min(90, progress + increment);
          updateStageProgress(stage, progress);
        }, interval);
        
        return timer;
      };
      
      // Get the models from the profile - NO FALLBACKS, must have actual profile
      console.log('[AnimateStages] activeProfile:', activeProfile);
      if (!activeProfile || !activeProfile.generator) {
        console.error('No active profile loaded! Cannot proceed with consensus.');
        addChatMessage('Error: No profile loaded. Please select a profile in settings.', false, 'error');
        return;
      }
      
      const models = {
        generator: activeProfile.generator,
        refiner: activeProfile.refiner,
        validator: activeProfile.validator,
        curator: activeProfile.curator
      };
      console.log('[AnimateStages] Using models:', models);
      
      // REMOVED FAKE ANIMATION - Let the real consensus engine control visuals
      // The SimpleConsensusEngine will send real-time updates via IPC events
      // which are handled by the consensusAPI event listeners above
      console.log('[AnimateStages] Waiting for real consensus engine updates...');
      
      // Make the actual API call - visual updates come from consensus engine
      const result = await (window as any).backendAPI.runQuickConsensus({
        query: query,
        profile: activeProfile?.name || 'Free Also'
      });
      
      // Don't force completion - let the consensus engine control this
      console.log('[AnimateStages] Consensus complete, received result');
      
      // Show neural consciousness completion
      if (neuralConsciousness) {
        neuralConsciousness.showCompletion();
        setTimeout(() => neuralConsciousness.hide(), 2000);
      }
      
      console.log('🎯🎯🎯 GOT RESULT FROM runQuickConsensus:', result);
      
      // Response is already displayed by consensus-complete event handler
      // Don't add it again here to avoid duplicates
      if (result && result.error) {
        console.log('🎯🎯🎯 GOT ERROR FROM API:', result.error);
        addChatMessage(`Error: ${result.error}`, true);
      }
      
      // Events handled all progress updates and final response display
      // Clean up event listeners
      (window as any).api.removeListener('websocket-message', eventHandler);
      
      // Clean up consensus event listeners after a longer delay (to ensure animations complete)
      if (consensusAPI) {
        setTimeout(() => consensusAPI.removeAllListeners(), 10000);
      }
      
      // Neural consciousness completion is now handled by the neural stage update event
      // with progress === 1.0, so we don't need to manually trigger it here
      
    } catch (error) {
      console.log('🎯🎯🎯 CAUGHT ERROR:', error);
      // Properly handle error display
      let errorMessage = 'An error occurred';
      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === 'string') {
        errorMessage = error;
      } else if (error && typeof error === 'object') {
        errorMessage = (error as any).message || (error as any).error || JSON.stringify(error);
      }
      addChatMessage(`Error: ${errorMessage}`, true);
      if (neuralConsciousness) {
        neuralConsciousness.hide();
      }
    } finally {
      console.log('🎯🎯🎯 FINALLY BLOCK - isProcessing = false');
      isProcessing = false;
    }
    
    return;
  }
  
  // Fallback: Check WebSocket connection
  if (!consensusWebSocket || !consensusWebSocket.isConnected()) {
    addLogEntry('WebSocket not connected, attempting to connect...', 'warning');
    
    // Initialize WebSocket if not already done
    if (!consensusWebSocket) {
      initializeWebSocket();
    } else {
      consensusWebSocket.connect();
    }
    
    // Wait a bit for connection
    setTimeout(() => {
      if (consensusWebSocket?.isConnected()) {
        addLogEntry('WebSocket connected!', 'success');
        // Retry the send
        document.getElementById('send-chat')?.click();
      } else {
        addLogEntry('WebSocket connection failed, using REST API fallback', 'warning');
        // Fallback to REST API
        runConsensusViaREST(query);
      }
    }, 1000);
    return;
  }
  
  isProcessing = true;
  conversationStartTime = performance.now();
  (window as any).consensusStartTime = Date.now();
  totalTokens = 0;
  totalCost = 0;
  currentStageTokens = 0;
  updateConsensusStats();
  resetStageStatus();
  
  // Show Neural Consciousness animation if enabled
  if (ENABLE_NEURAL_CONSCIOUSNESS && neuralConsciousness) {
    try {
      // Show the consciousness animation
      neuralConsciousness.show();
      
      // Phase 1: Memory Retrieval (0-2s)
      setTimeout(() => {
        neuralConsciousness?.updatePhase('memory');
      }, 500);
      
      // Phase 2: Context Synthesis (2-4s)
      setTimeout(() => {
        neuralConsciousness?.updatePhase('synthesis');
      }, 2500);
      
      // Phase 3: Classification (4-6s)
      setTimeout(() => {
        neuralConsciousness?.updatePhase('classification');
      }, 4500);
      
      // Animation will continue through all consensus stages
      // and only hide after the final completion animation
    } catch (error) {
      console.error('Error with Neural Consciousness animation:', error);
    }
  }
  
  addLogEntry(`🚀 Starting streaming consensus for: "${query}"`, 'info');
  
  // Create new conversation if needed
  if (!currentConversationId) {
    currentConversationId = generateConversationId();
    addLogEntry(`📝 New conversation started: ${currentConversationId}`, 'info');
  }
  
  // Add to conversation history
  conversationHistory.push({ role: 'user', content: query });
  
  // Get current profile from database (should always be loaded)
  const currentProfileName = activeProfile?.name || 'Balanced Performer';
  console.log('[Consensus] Using profile for user query:', currentProfileName, 'activeProfile:', activeProfile);
  
  // Start consensus via WebSocket with conversation context
  consensusWebSocket.startConsensus(query, currentProfileName, currentConversationId, conversationHistory);
});

// Fallback REST API function
runConsensusViaREST = async (query: string) => {
  isProcessing = true;
  conversationStartTime = performance.now();
  (window as any).consensusStartTime = Date.now();
  totalTokens = 0;
  totalCost = 0;
  currentStageTokens = 0;
  updateConsensusStats();
  resetStageStatus();
  
  addLogEntry(`🚀 Starting consensus via IPC/REST for: "${query}"`, 'info');
  
  // Trigger neural consciousness (this is the actual path being used!)
  if (neuralConsciousness) {
    await neuralConsciousness.show();
    await neuralConsciousness.updatePhase('memory');
    await new Promise(resolve => setTimeout(resolve, 500));
    await neuralConsciousness.updatePhase('synthesis');
    await new Promise(resolve => setTimeout(resolve, 500));
    await neuralConsciousness.updatePhase('classification');
    await new Promise(resolve => setTimeout(resolve, 500));
    await neuralConsciousness.updatePhase('generator');
  }
  
  try {
    // Use IPC to communicate with main process, which can make the HTTP request
    const result = await (window as any).backendAPI.runQuickConsensus({
      query: query,
      profile: activeProfile?.name || 'Free Also'
    });
    
    // Remove manual stage completion - let DirectConsensusEngine events handle progress
    // Stages will update via event system from DirectConsensusEngine
    
    // Update stats
    totalTokens = result.tokens_used || 1000;
    totalCost = result.cost || 0.01;
    updateConsensusStats();
    
    console.log('📡 CONSENSUS RESULT RECEIVED:', result);
    console.log('📡 RESULT.RESULT:', result.result);
    console.log('📡 RESULT.RESPONSE:', result.response);
    
    addLogEntry(`✅ Consensus completed in ${result.duration_ms}ms`, 'success');
    
    // Use result.response if result.result is undefined (handler returns both)
    const consensusResponse = result.result || result.response || 'No response received';
    console.log('📡 ADDING TO CHAT:', consensusResponse);
    console.log('📡 TYPE OF RESPONSE:', typeof consensusResponse);
    
    // Ensure we have a string to display
    if (typeof consensusResponse === 'string') {
      addChatMessage(consensusResponse, true);
    } else if (consensusResponse && typeof consensusResponse === 'object') {
      // If it's an object, try to extract the message
      const message = consensusResponse.content || consensusResponse.message || JSON.stringify(consensusResponse);
      console.log('📡 EXTRACTED MESSAGE:', message);
      addChatMessage(message, true);
    } else {
      console.error('📡 INVALID RESPONSE TYPE:', consensusResponse);
      addChatMessage('Error: Invalid response format', true);
    }
    
    // Update usage count
    dailyUsageCount++;
    updateConversationCount();
    
  } catch (error) {
    resetStageStatus();
    console.error('Full error details:', error);
    
    // Check if it's a network error
    if (error instanceof TypeError && error.message === 'Failed to fetch') {
      addLogEntry(`❌ Network error: Cannot connect to backend service`, 'error');
      addLogEntry(`💡 Make sure the backend server is running`, 'warning');
      addChatMessage(`Network Error: Cannot reach the backend server. Please ensure it's running.`, true);
    } else {
      const errorMessage = error?.message || error?.toString() || 'Unknown error occurred';
      console.error('📡 ERROR OBJECT:', error);
      addLogEntry(`❌ Consensus failed: ${errorMessage}`, 'error');
      
      // Ensure error message is a string
      if (typeof errorMessage === 'string') {
        addChatMessage(`Error: ${errorMessage}`, true);
      } else {
        addChatMessage(`Error: ${JSON.stringify(errorMessage)}`, true);
      }
    }
  } finally {
    isProcessing = false;
  }
};

// Enter key support for chat input
document.getElementById('chat-input')?.addEventListener('keypress', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    document.getElementById('send-chat')?.click();
  }
});

// Listen for direct IPC commands from DirectConsensusEngine
(window as any).consensusAPI?.on('updateStageStatus', (data: any) => {
  console.log('📡 Received updateStageStatus:', data);
  updateStageStatus(data.stage, data.status);
});

(window as any).consensusAPI?.on('updateStageProgress', (data: any) => {
  console.log('📡 Received updateStageProgress:', data);
  updateStageProgress(data.stage, data.progress);
});

(window as any).consensusAPI?.on('updateModelDisplay', (data: any) => {
  console.log('📡 Received updateModelDisplay:', data);
  updateModelDisplay(data.stage, data.model);
});

(window as any).consensusAPI?.on('neuralConsciousness.updatePhase', (data: any) => {
  console.log('📡 Received neuralConsciousness.updatePhase:', data);
  if (neuralConsciousness) {
    neuralConsciousness.updatePhase(data.phase);
  }
});

// Test Progress Bars button
document.getElementById('test-progress')?.addEventListener('click', async () => {
  console.log('🧪 Testing Progress Bars Sequentially');
  addChatMessage('🧪 Testing progress bars sequentially...', true);
  
  // Helper function to animate progress bar
  async function animateProgressBar(stage: string, modelName: string = 'test-model') {
    console.log(`Testing ${stage} progress bar`);
    
    // Set status to running
    updateStageStatus(stage, 'running');
    updateModelDisplay(stage, modelName);
    
    // Animate progress from 0 to 100
    for (let i = 0; i <= 100; i += 10) {
      updateStageProgress(stage, i);
      await new Promise(resolve => setTimeout(resolve, 200)); // 200ms between updates
    }
    
    // Set to completed
    updateStageStatus(stage, 'completed');
    updateStageProgress(stage, 100);
  }
  
  // Show neural consciousness
  if (neuralConsciousness) {
    neuralConsciousness.show();
    
    // Run through prep stages
    await new Promise(resolve => setTimeout(resolve, 500));
    neuralConsciousness.updatePhase('memory');
    await new Promise(resolve => setTimeout(resolve, 500));
    neuralConsciousness.updatePhase('synthesis');
    await new Promise(resolve => setTimeout(resolve, 500));
    neuralConsciousness.updatePhase('classification');
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  // Test each progress bar sequentially - use actual profile models only
  const stages: Array<{name: string, model: string, phase: 'generator' | 'refiner' | 'validator' | 'curator'}> = [
    { name: 'generator', model: activeProfile?.generator, phase: 'generator' },
    { name: 'refiner', model: activeProfile?.refiner, phase: 'refiner' },
    { name: 'validator', model: activeProfile?.validator, phase: 'validator' },
    { name: 'curator', model: activeProfile?.curator, phase: 'curator' }
  ];
  
  for (const stage of stages) {
    // Update neural consciousness phase
    if (neuralConsciousness) {
      neuralConsciousness.updatePhase(stage.phase);
    }
    
    // Animate the progress bar
    await animateProgressBar(stage.name, stage.model);
    
    // Brief pause between stages
    await new Promise(resolve => setTimeout(resolve, 500));
  }
  
  // Show completion
  if (neuralConsciousness) {
    neuralConsciousness.showCompletion();
    await new Promise(resolve => setTimeout(resolve, 2000));
    neuralConsciousness.hide();
  }
  
  addChatMessage('✅ Progress bar test complete!', true);
});

// Initialize WebSocket connection for streaming
async function initializeWebSocket() {
  // Prevent multiple initializations
  if (consensusWebSocket) {
    console.log('WebSocket already initialized');
    return;
  }
  
  // Get dynamic backend port from IPC - NO FALLBACK
  let backendPort: number;
  try {
    if ((window as any).backendAPI?.getBackendPort) {
      backendPort = await (window as any).backendAPI.getBackendPort();
      console.log('Got dynamic backend port:', backendPort);
      
      // Update the terminal display with actual port
      const backendLine = document.getElementById('backend-server-line');
      if (backendLine) {
        backendLine.textContent = `[${new Date().toLocaleTimeString()}] Backend server: http://localhost:${backendPort}`;
      }
    } else {
      throw new Error('Backend API not available');
    }
  } catch (error) {
    console.error('Failed to get backend port - cannot connect:', error);
    addLogEntry('❌ Backend service not available', 'error');
    return; // Don't attempt connection without port
  }
  
  const wsUrl = `ws://127.0.0.1:${backendPort}/ws`;
  
  console.log('Initializing WebSocket with URL:', wsUrl);
  consensusWebSocket = new ConsensusWebSocket(wsUrl, {
    onConnectionStateChange: (connected) => {
      if (connected) {
        addLogEntry('✅ WebSocket connected for streaming', 'success');
      } else {
        addLogEntry('⚠️ WebSocket disconnected', 'warning');
      }
    },
    
    onProfileLoaded: (name, models) => {
      activeProfile = { name, models };
      const profileDisplay = document.getElementById('active-profile-name');
      if (profileDisplay) {
        profileDisplay.textContent = name;
      }
      
      // Update model displays
      if (models.length >= 4) {
        updateModelDisplay('generator', models[0]);
        updateModelDisplay('refiner', models[1]);
        updateModelDisplay('validator', models[2]);
        updateModelDisplay('curator', models[3]);
      }
      
      addLogEntry(`📋 Profile loaded: ${name}`, 'info');
    },
    
    onStageStarted: (stage, model) => {
      const stageName = stage.toLowerCase();
      updateStageStatus(stageName, 'running');
      updateStageProgress(stageName, 25); // Start at 25% when stage begins
      updateModelDisplay(stageName, model);
      addLogEntry(`▶️ ${stage} started with ${model}`, 'info');
      currentStreamContent.set(stageName, '');
      // Reset current stage tokens when a new stage starts
      currentStageTokens = 0;
      
      // Update Neural Consciousness for each consensus stage
      if (neuralConsciousness) {
        switch(stageName) {
          case 'memory':
            neuralConsciousness.updatePhase('memory');
            break;
          case 'generator':
            neuralConsciousness.updatePhase('generator');
            break;
          case 'refiner':
            neuralConsciousness.updatePhase('refiner');
            break;
          case 'validator':
            neuralConsciousness.updatePhase('validator');
            break;
          case 'curator':
            neuralConsciousness.updatePhase('curator');
            break;
        }
      }
    },
    
    onStreamChunk: (stage, chunk) => {
      // Show streaming output immediately as it arrives
      const stageName = stage.toLowerCase();
      const chatContent = document.getElementById('chat-content');
      
      // Find or create message for this stage
      let stageMessage = chatContent?.querySelector(`.streaming-${stageName}`);
      
      if (!stageMessage) {
        // Create new message container for this stage with enhanced styling
        const message = document.createElement('div');
        message.className = `chat-message assistant streaming streaming-${stageName}`;
        
        const timestamp = new Date().toLocaleTimeString('en-US', { 
          hour: 'numeric', 
          minute: '2-digit',
          hour12: true 
        });
        
        message.innerHTML = `
          <div class="message-time">${timestamp} <span class="stage-badge">${stage}</span></div>
          <div class="message-content"></div>
        `;
        chatContent?.appendChild(message);
        stageMessage = message;
      }
      
      // Accumulate content first
      const currentContent = currentStreamContent.get(stageName) || '';
      const newContent = currentContent + chunk;
      currentStreamContent.set(stageName, newContent);
      
      // Update the entire content (replacing, not appending)
      const contentEl = stageMessage.querySelector('.message-content');
      if (contentEl) {
        // Enhanced markdown to HTML conversion
        let htmlContent = convertMarkdownToHTML(newContent);
        
        // Replace entire content (not append)
        contentEl.innerHTML = htmlContent;
      }
      
      // Always auto-scroll to show the newest content
      autoScrollChat();
    },
    
    onStageProgress: (stage, percentage, tokens) => {
      const stageName = stage.toLowerCase();
      // Ensure minimum 25% when running, cap at 95% until complete
      const adjustedPercentage = Math.max(25, Math.min(95, percentage));
      updateStageProgress(stageName, adjustedPercentage);
      
      // Track current stage tokens for display (these are cumulative within the stage)
      currentStageTokens = tokens;
      // Don't add to totalTokens here - will be added when stage completes
      // This prevents the exponential accumulation bug
      updateConsensusStats();
    },
    
    onStageCompleted: (stage, tokens, cost) => {
      const stageName = stage.toLowerCase();
      updateStageStatus(stageName, 'completed');
      updateStageProgress(stageName, 100);
      // Add this stage's tokens to the total (only once per stage)
      totalTokens += tokens;
      totalCost += cost;
      // Reset current stage tokens since this stage is done
      currentStageTokens = 0;
      updateConsensusStats();
      
      // Track stage metrics for analytics
      trackStageCompletion(stageName, tokens, cost);
      
      addLogEntry(`✅ ${stage} completed (${tokens} tokens, ${formatCost(cost)})`, 'success');
    },
    
    onConsensusComplete: (result, finalTokens, finalCost) => {
      // Prevent duplicate completion messages
      if (!isProcessing) {
        console.warn('Consensus complete called but not processing, ignoring');
        return;
      }
      
      totalTokens = finalTokens;
      totalCost = finalCost;
      updateConsensusStats();
      
      // Save analytics data for the dashboard
      saveConsensusAnalytics(finalTokens, finalCost);
      
      // Mark as no longer processing
      isProcessing = false;
      
      // Show final completion animation before hiding
      if (neuralConsciousness) {
        // Trigger completion phase with grand finale
        neuralConsciousness.showCompletion().then(() => {
          // Hide after completion animation finishes
          setTimeout(() => {
            neuralConsciousness.hide();
          }, 2000);
        });
      }
      
      // Remove all streaming indicators
      const chatContent = document.getElementById('chat-content');
      const streamingMessages = chatContent?.querySelectorAll('.streaming');
      streamingMessages?.forEach(msg => {
        msg.classList.remove('streaming');
        // Remove stage-specific streaming classes
        msg.className = msg.className.replace(/streaming-\w+/g, '').trim();
      });
      
      // Add final consensus result if provided (only once)
      if (result && result.trim()) {
        const finalMessage = document.createElement('div');
        finalMessage.className = 'chat-message assistant consensus-final';
        
        const timestamp = new Date().toLocaleTimeString('en-US', { 
          hour: 'numeric', 
          minute: '2-digit',
          hour12: true 
        });
        
        finalMessage.innerHTML = `
          <div class="message-time">${timestamp} <span class="stage-badge">🎆 Final Consensus</span></div>
          <div class="message-content">${convertMarkdownToHTML(result)}</div>
        `;
        chatContent?.appendChild(finalMessage);
        
        // Add assistant response to conversation history for context
        conversationHistory.push({ role: 'assistant', content: result });
        
        // Store the Q&A pair for this conversation
        if (conversationHistory.length >= 2) {
          const lastUserMessage = conversationHistory[conversationHistory.length - 2];
          if (lastUserMessage.role === 'user') {
            conversationMessages.push({
              question: lastUserMessage.content,
              answer: result
            });
          }
        }
        
        addLogEntry(`💾 Conversation context updated (${conversationHistory.length} messages)`, 'info');
      }
      
      // Auto-scroll to ensure the complete result is visible
      autoScrollChat();
      
      addLogEntry(`🎯 Consensus complete! Total: ${formatTokens(finalTokens)} tokens, ${formatCost(finalCost)}`, 'success');
      addLogEntry(`📝 Conversation ID: ${currentConversationId}`, 'info');
      
      // Update usage count
      dailyUsageCount++;
      updateConversationCount();
      
      // Reset processing state
      isProcessing = false;
      currentStreamContent.clear();
    },
    
    onError: (message) => {
      addLogEntry(`❌ Error: ${message}`, 'error');
      resetStageStatus();
      isProcessing = false;
      
      // Hide Neural Consciousness on error
      if (neuralConsciousness) {
        neuralConsciousness.hide();
      }
    },
    
    onAIHelperDecision: (directMode, reason) => {
      addLogEntry(`🤖 AI Helper: ${reason}`, 'info');
      // If Direct mode, mark other stages as skipped
      if (directMode) {
        ['refiner', 'validator', 'curator'].forEach(stage => {
          updateStageStatus(stage, 'completed');
          updateModelDisplay(stage, 'skipped (direct mode)');
        });
      }
    },
    
    // NEW: Individual LLM activation for deliberation rounds
    onLLMStarted: (round: number, llm: string, model: string) => {
      // Reset all progress bars for new round if it's a new round starting with generator
      if (llm === 'generator' && round > 1) {
        // Reset only the iterative stages for new round (NOT curator)
        ['generator', 'refiner', 'validator'].forEach(stage => {
          updateStageStatus(stage, 'ready');
          updateStageProgress(stage, 0);
        });
        // Curator stays dormant until consensus is achieved
      }
      
      // Activate current LLM progress bar
      updateStageStatus(llm, 'running');
      updateStageProgress(llm, 25);
      updateModelDisplay(llm, `Round ${round}: ${model.split('/').pop() || model}`);
      
      addLogEntry(`🤖 Round ${round}: ${llm.toUpperCase()} deliberating with ${model}`, 'info');
    },
    
    // NEW: Real-time token updates (gas pump effect)
    onTokenUpdate: (tokens: number, cost: number, currentLLM: string, round: number) => {
      // Update token counter in real-time like gas pump
      totalTokens = tokens;
      totalCost = cost;
      updateConsensusStats();
      
      // Complete current LLM progress bar
      updateStageStatus(currentLLM, 'completed');
      updateStageProgress(currentLLM, 100);
      
      addLogEntry(`📊 Round ${round}: ${currentLLM.toUpperCase()} added tokens (Total: ${tokens})`, 'info');
    }
  });
  
  consensusWebSocket.connect();
  console.log('WebSocket connect() called, waiting for connection...');
  
  // Force check connection status after a short delay
  setTimeout(() => {
    console.log('Checking WebSocket connection status...');
    if (consensusWebSocket.isConnected()) {
      console.log('✅ WebSocket is connected!');
      addLogEntry('✅ WebSocket connected and ready', 'success');
    } else {
      console.log('❌ WebSocket failed to connect');
      addLogEntry('⚠️ WebSocket not connected - messages will use REST API', 'warning');
    }
  }, 500);
}

function updateModelDisplay(stage: string, model: string) {
  const modelElement = document.getElementById(`${stage}-model`);
  if (modelElement) {
    // Truncate long model names for display
    const displayModel = model.length > 30 ? model.substring(0, 27) + '...' : model;
    modelElement.textContent = displayModel;
    modelElement.title = model; // Show full name on hover
  }
}

function updateStageProgress(stage: string, percentage: number) {
  const progressElement = document.getElementById(`${stage}-progress`) as HTMLElement;
  if (progressElement) {
    progressElement.style.width = `${percentage}%`;
    
    // Update status based on percentage
    const statusElement = document.getElementById(`${stage}-status`);
    if (statusElement) {
      if (percentage === 0) {
        statusElement.textContent = 'ready';
        statusElement.className = 'stage-status ready';
        progressElement.classList.remove('running');
      } else if (percentage > 0 && percentage < 100) {
        statusElement.textContent = 'running';
        statusElement.className = 'stage-status running';
        progressElement.classList.add('running');
      } else if (percentage === 100) {
        statusElement.textContent = 'completed';
        statusElement.className = 'stage-status completed';
        progressElement.classList.remove('running');
        progressElement.style.background = 'linear-gradient(90deg, var(--terminal-success) 0%, var(--terminal-info) 100%)';
      }
    }
  }
}

function resetStageStatus() {
  // Clear all progress intervals first
  progressIntervals.forEach((interval, stage) => {
    clearInterval(interval);
    console.log(`🔍 Cleared interval for stage: ${stage}`);
  });
  progressIntervals.clear();
  
  // Clear consensus type display
  const consensusTypeElement = document.getElementById('consensus-type');
  if (consensusTypeElement) {
    consensusTypeElement.textContent = '';
  }
  
  // Reset ALL stages to proper initial state
  // All stages should start as 'ready' when a new consensus begins
  ['generator', 'refiner', 'validator', 'curator'].forEach(stage => {
    updateStageStatus(stage, 'ready');
    updateStageProgress(stage, 0);
    
    // Reset model display
    const modelElement = document.getElementById(`${stage}-model`);
    if (modelElement) {
      modelElement.textContent = '--';
    }
  });
  
  // Clear any lingering running states
  document.querySelectorAll('.progress-fill').forEach(el => {
    el.classList.remove('running');
  });
}

// Add a test WebSocket button for debugging
const testWSBtn = document.getElementById('test-consensus');
if (testWSBtn) {
  const originalOnClick = testWSBtn.onclick;
  testWSBtn.onclick = null;
  testWSBtn.addEventListener('click', async () => {
    addLogEntry('🧪 Testing WebSocket connection directly...', 'info');
    
    // Get dynamic WebSocket port from ProcessManager
    let wsPort;
    try {
      wsPort = await (window as any).api.invoke('websocket-backend-port');
      if (!wsPort) {
        throw new Error('No WebSocket port allocated');
      }
      addLogEntry(`📡 Using WebSocket port: ${wsPort}`, 'info');
    } catch (error) {
      addLogEntry(`❌ Failed to get WebSocket port: ${error}`, 'error');
      return;
    }
    
    // Test with a simple WebSocket first
    try {
      const testWS = new WebSocket(`ws://localhost:${wsPort}/ws-test`);
      testWS.onopen = () => {
        addLogEntry('✅ Test WebSocket connected!', 'success');
        testWS.send('Hello from Electron');
      };
      testWS.onmessage = (event) => {
        addLogEntry(`📥 Test WS received: ${event.data}`, 'info');
      };
      testWS.onerror = (error) => {
        addLogEntry(`❌ Test WS error: ${error}`, 'error');
        console.error('Test WebSocket error:', error);
      };
      testWS.onclose = () => {
        addLogEntry('🔌 Test WebSocket closed', 'info');
      };
    } catch (error) {
      addLogEntry(`❌ Failed to create test WebSocket: ${error}`, 'error');
      console.error('WebSocket creation error:', error);
    }
  });
}

// Make WebSocket test function available globally for console debugging
(window as any).testWebSocket = async () => {
  console.log('Testing WebSocket connection...');
  let wsPort;
  try {
    wsPort = await (window as any).api.invoke('websocket-backend-port');
    if (!wsPort) {
      throw new Error('No WebSocket port allocated');
    }
    console.log(`Using WebSocket port: ${wsPort}`);
  } catch (error) {
    console.error(`Failed to get WebSocket port: ${error}`);
    return null;
  }
  const ws = new WebSocket(`ws://localhost:${wsPort}/ws-test`);
  ws.onopen = () => console.log('✅ WebSocket opened!');
  ws.onerror = (e) => console.error('❌ WebSocket error:', e);
  ws.onclose = (e) => console.log('WebSocket closed:', e.code, e.reason);
  ws.onmessage = (e) => console.log('WebSocket message:', e.data);
  return ws;
};

// Auto-test connection on startup
setTimeout(async () => {
  addLogEntry('🔄 Auto-testing backend connection...', 'info');
  
  // Initialize WebSocket for streaming
  initializeWebSocket();
  
  try {
    if ((window as any).backendAPI) {
      const health = await (window as any).backendAPI.healthCheck();
      updateConnectionStatus(true);
      updateStatus('Ready', 'ready');
      addLogEntry(`✅ Backend health check passed: ${health.service} v${health.version}`, 'success');
    } else {
      throw new Error('Backend API not available');
    }
  } catch (error) {
    updateConnectionStatus(false);
    updateStatus('Backend Unavailable', 'error');
    addLogEntry(`❌ Backend health check failed: ${error}`, 'error');
  }
}, 1000);

// Initialize the application
addLogEntry('⚡ Hive Consensus Day 0 Validation started', 'info');
addLogEntry('🔧 Click buttons above to test the Electron + Rust architecture', 'info');
addChatMessage('Welcome to Hive Consensus! Try asking me a question.', true);

// Function to setup menu event listeners
function setupMenuEventListeners() {
    // Listen for Open Folder menu event
    (window as any).electronAPI?.onMenuOpenFolder?.((folderPath: string) => {
        console.log('Opening folder:', folderPath);
        // Refresh the file explorer with the new folder
        if (window.fileExplorer && window.fileExplorer.refresh) {
            window.fileExplorer.refresh();
        }
        // Update status bar
        if ((window as any).statusBar) {
            (window as any).statusBar.render();
        }
    });
    
    // Listen for New File menu event
    (window as any).electronAPI?.onMenuNewFile?.(() => {
        console.log('New file requested');
        if (window.fileExplorer && window.fileExplorer.createFile) {
            showInputDialog('New File', 'Enter file name:').then(fileName => {
                if (fileName) {
                    window.fileExplorer.createFile(fileName);
                }
            });
        }
    });
    
    // Listen for Save File menu event
    (window as any).electronAPI?.onMenuSaveFile?.(() => {
        console.log('Save file requested');
        // Save current file in editor
        if (window.editorTabs && window.editorTabs.saveCurrentFile) {
            window.editorTabs.saveCurrentFile();
        }
    });
}

// Function to update Git branch display in status bar
async function updateGitStatusBar() {
    const branchElement = document.getElementById('status-git-branch');
    const branchNameElement = document.getElementById('branch-name');
    const warningsElement = document.getElementById('status-git-warnings');
    const errorsElement = document.getElementById('status-git-errors');
    
    if (currentOpenedFolder && window.gitAPI) {
        try {
            // Get Git status to show branch
            const status = await window.gitAPI.getStatus();
            if (status && status.isRepo) {
                // Show Git info
                if (branchElement) branchElement.style.display = 'flex';
                if (warningsElement) warningsElement.style.display = 'flex';
                if (errorsElement) errorsElement.style.display = 'flex';
                
                // Update branch name
                if (branchNameElement) {
                    branchNameElement.textContent = status.branch || 'main';
                }
            } else {
                // Not a Git repo, hide Git info
                if (branchElement) branchElement.style.display = 'none';
                if (warningsElement) warningsElement.style.display = 'none';
                if (errorsElement) errorsElement.style.display = 'none';
            }
        } catch (error) {
            console.error('Failed to get Git status:', error);
            // Hide on error
            if (branchElement) branchElement.style.display = 'none';
            if (warningsElement) warningsElement.style.display = 'none';
            if (errorsElement) errorsElement.style.display = 'none';
        }
    } else {
        // No folder open, hide Git info
        if (branchElement) branchElement.style.display = 'none';
        if (warningsElement) warningsElement.style.display = 'none';
        if (errorsElement) errorsElement.style.display = 'none';
    }
}

// Function to update status bar with license info
async function updateStatusBar() {
  try {
    const settings = await (window as any).settingsAPI.loadSettings();
    console.log('Settings loaded:', settings);
    
    const userElement = document.getElementById('status-user');
    const planElement = document.getElementById('status-plan');
    const conversationsElement = document.getElementById('status-conversations');
    
    if (settings.hiveKey) {
      // Test the key to get license info
      const result = await (window as any).settingsAPI.testKeys({
        hiveKey: settings.hiveKey
      });
      console.log('License info:', result.licenseInfo);
      
      if (result.hiveValid && result.licenseInfo) {
        // Update user display
        if (userElement) {
          const email = result.licenseInfo.email || 'Licensed User';
          // Store full email for responsive handling
          userElement.setAttribute('data-full-email', email);
          // Truncate email if too long for status bar
          const displayEmail = email.length > 20 ? email.substring(0, 17) + '...' : email;
          userElement.textContent = displayEmail;
          userElement.title = email; // Full email in tooltip
          console.log('Set user display to:', displayEmail);
        }
        
        // Update plan display
        if (planElement) {
          // Capitalize the tier name
          const tier = (result.licenseInfo.tier || 'Free').charAt(0).toUpperCase() + 
                       (result.licenseInfo.tier || 'Free').slice(1).toLowerCase();
          planElement.setAttribute('data-full-plan', tier);
          planElement.textContent = tier;
          console.log('Set plan display to:', tier);
        }
        
        // Don't update conversation count here - let updateConversationCount handle it from local DB
        // Just store the tier info for display
        if (result.licenseInfo.tier === 'unlimited') {
          dailyLimit = 999999;
        } else if (result.licenseInfo.dailyLimit) {
          dailyLimit = result.licenseInfo.dailyLimit;
        }
        
        // The actual count will be updated by updateConversationCount from local database
        console.log('D1 validation complete, will fetch actual usage from local DB');
      } else {
        // Invalid license
        if (userElement) userElement.textContent = 'Invalid license';
        if (planElement) planElement.textContent = 'Free';
        if (conversationsElement) conversationsElement.textContent = '-- remaining';
      }
    } else {
      // No license key configured
      if (userElement) userElement.textContent = 'Not logged in';
      if (planElement) planElement.textContent = 'Free';
      if (conversationsElement) conversationsElement.textContent = '-- remaining';
    }
  } catch (error) {
    console.error('Failed to update status bar:', error);
    // Set defaults on error - don't hardcode values
    const userElement = document.getElementById('status-user');
    const planElement = document.getElementById('status-plan');
    const conversationsElement = document.getElementById('status-conversations');
    
    if (userElement) userElement.textContent = 'Not logged in';
    if (planElement) planElement.textContent = 'Free';
    if (conversationsElement) conversationsElement.textContent = '-- remaining';
  }
}

// Initialize settings modal with callback to update status bar and profile
settingsModal = new SettingsModal(async () => {
  // Callback when settings are saved
  updateStatusBar();
  await loadActiveProfile(); // Reload profile from database
  
  // Log the profile switch after it's loaded
  if (activeProfile) {
    addLogEntry(`✅ Profile switched to: ${activeProfile.name}`, 'success');
    console.log('[Settings] Active profile updated:', activeProfile);
  }
});
// Don't initialize modal - we're using tabs instead
// settingsModal.initializeModal(document.body);

// Function to update just the conversation count from database
async function updateConversationCount() {
  try {
    // Fetch real usage from database
    const usage = await (window as any).electronAPI?.getUsageCount();
    if (usage) {
      console.log('Usage from database:', usage);
      dailyUsageCount = usage.used;
      dailyLimit = usage.limit;
      
      const conversationsElement = document.getElementById('status-conversations');
      if (conversationsElement) {
        let fullText: string;
        if (usage.limit === 999999) {
          fullText = `${usage.used} used / Unlimited`;
        } else {
          fullText = `${usage.used} used / ${usage.remaining} remaining`;
        }
        conversationsElement.setAttribute('data-full-text', fullText);
        conversationsElement.textContent = fullText;
        // Update responsive display
        updateStatusBarResponsive();
      }
    }
  } catch (error) {
    console.error('Failed to update conversation count:', error);
    // Fallback to local count
    const conversationsElement = document.getElementById('status-conversations');
    if (conversationsElement) {
      const remaining = Math.max(0, dailyLimit - dailyUsageCount);
      const fullText = `${dailyUsageCount} used / ${remaining} remaining`;
      conversationsElement.setAttribute('data-full-text', fullText);
      conversationsElement.textContent = fullText;
      // Update responsive display
      updateStatusBarResponsive();
    }
  }
}

// Function to update consensus stats (tokens and cost)
function updateConsensusStats() {
  const tokenElement = document.getElementById('token-count');
  const costElement = document.getElementById('cost-count');
  
  if (tokenElement) {
    // Show total + current stage tokens during streaming
    const displayTokens = totalTokens + currentStageTokens;
    tokenElement.textContent = displayTokens.toLocaleString();
    tokenElement.className = 'stat-value tokens';
  }
  
  if (costElement) {
    costElement.textContent = `$${totalCost.toFixed(4)}`;
    costElement.className = 'stat-value cost';
  }
}

// Function to load and display active profile
async function loadActiveProfile() {
  try {
    const settings = await (window as any).settingsAPI.loadSettings();
    
    if (settings.activeProfileId || settings.activeProfileName) {
      // Load all profiles
      const profiles = await (window as any).settingsAPI.loadProfiles();
      
      // Find the active profile
      const matchingProfile = profiles.find((p: any) => 
        p.id === settings.activeProfileId || 
        p.name === settings.activeProfileName
      );
      
      if (matchingProfile) {
        activeProfile = matchingProfile;
        
        // Update profile display
        const profileNameElement = document.getElementById('active-profile-name');
        if (profileNameElement) {
          profileNameElement.textContent = matchingProfile.name;
        }
        
        // Update model displays
        const generatorElement = document.getElementById('generator-model');
        const refinerElement = document.getElementById('refiner-model');
        const validatorElement = document.getElementById('validator-model');
        const curatorElement = document.getElementById('curator-model');
        
        if (generatorElement) generatorElement.textContent = matchingProfile.generator || '--';
        if (refinerElement) refinerElement.textContent = matchingProfile.refiner || '--';
        if (validatorElement) validatorElement.textContent = matchingProfile.validator || '--';
        if (curatorElement) curatorElement.textContent = matchingProfile.curator || '--';
        
        addLogEntry(`📋 Loaded profile: ${matchingProfile.name}`, 'info');
      } else {
        // Set default profile
        const profileNameElement = document.getElementById('active-profile-name');
        if (profileNameElement) {
          profileNameElement.textContent = 'Balanced Performer (Default)';
        }
        addLogEntry('📋 Using default profile: Balanced Performer', 'info');
      }
    }
  } catch (error) {
    console.error('Failed to load active profile:', error);
    const profileNameElement = document.getElementById('active-profile-name');
    if (profileNameElement) {
      profileNameElement.textContent = 'Error loading profile';
    }
  }
}

// Update status bar on startup
setTimeout(async () => {
  console.log('🔄 Updating status bar and loading profile...');
  await updateStatusBar();
  await loadActiveProfile();  // Await to ensure profile is loaded before continuing
  
  // ALWAYS update conversation count from local database (overrides D1)
  await updateConversationCount();
  console.log('Updated conversation count from local database');
  
  // Apply responsive text sizing after initial load
  updateStatusBarResponsive();
  
  // Initialize Neural Consciousness AFTER critical components
  // Neural Consciousness is initialized in the right panel, not here
}, 100);

// Function to handle responsive status bar text
function updateStatusBarResponsive() {
  const width = window.innerWidth;
  const userElement = document.getElementById('status-user');
  const planElement = document.getElementById('status-plan');
  const conversationsElement = document.getElementById('status-conversations');
  
  if (userElement) {
    const fullEmail = userElement.getAttribute('data-full-email') || userElement.textContent || '';
    if (width < 480) {
      // Ultra small - show only username part
      const username = fullEmail.split('@')[0];
      userElement.textContent = username.length > 10 ? username.substring(0, 7) + '...' : username;
    } else if (width < 768) {
      // Small - show abbreviated email
      const parts = fullEmail.split('@');
      if (parts.length === 2) {
        const username = parts[0].length > 8 ? parts[0].substring(0, 5) + '...' : parts[0];
        const domain = parts[1].length > 10 ? '@' + parts[1].substring(0, 7) + '...' : '@' + parts[1];
        userElement.textContent = username + domain;
      } else {
        userElement.textContent = fullEmail.length > 15 ? fullEmail.substring(0, 12) + '...' : fullEmail;
      }
    } else if (width < 1200) {
      // Medium - show most of email
      userElement.textContent = fullEmail.length > 20 ? fullEmail.substring(0, 17) + '...' : fullEmail;
    } else {
      // Large - show full email
      userElement.textContent = fullEmail;
    }
  }
  
  if (planElement) {
    const fullPlan = planElement.getAttribute('data-full-plan') || planElement.textContent || '';
    if (width < 768) {
      // Abbreviate plan names
      if (fullPlan.includes('Professional')) {
        planElement.textContent = 'Pro';
      } else if (fullPlan.includes('Enterprise')) {
        planElement.textContent = 'Ent';
      } else if (fullPlan.includes('Unlimited')) {
        planElement.textContent = 'Unl';
      }
    } else {
      planElement.textContent = fullPlan;
    }
  }
  
  if (conversationsElement) {
    const fullText = conversationsElement.getAttribute('data-full-text') || conversationsElement.textContent || '';
    if (width < 480) {
      // Ultra compact
      const match = fullText.match(/(\d+)\s*(?:used|\/)/);
      if (match) {
        conversationsElement.textContent = match[1];
      }
    } else if (width < 768) {
      // Compact format
      const usedMatch = fullText.match(/(\d+)\s*used/);
      const remainingMatch = fullText.match(/(\d+)\s*remaining/);
      if (usedMatch && remainingMatch) {
        conversationsElement.textContent = `${usedMatch[1]}/${remainingMatch[1]}`;
      } else if (fullText.includes('Unlimited')) {
        conversationsElement.textContent = `${usedMatch?.[1] || '0'}/∞`;
      }
    } else {
      conversationsElement.textContent = fullText;
    }
  }
}

// Add resize listener
window.addEventListener('resize', () => {
  requestAnimationFrame(updateStatusBarResponsive);
});

// Analytics Panel Management
let analyticsPanel: HTMLElement | null = null;

function showAnalyticsPanel(): void {
    // Get the analytics panel that's already in the DOM
    const analyticsPanel = document.getElementById('analytics-panel');
    const welcomeContent = document.getElementById('welcome-content');
    
    if (!analyticsPanel) {
        console.error('Analytics panel not found in DOM');
        return;
    }
    
    // Hide welcome content
    if (welcomeContent) {
        welcomeContent.style.display = 'none';
    }
    
    // Show analytics panel
    analyticsPanel.style.display = 'block';
    
    // Mount the analytics dashboard
    analyticsDashboard.mount(analyticsPanel);
    
    // Add Analytics tab if it doesn't exist
    const tabsContainer = document.querySelector('.editor-tabs');
    if (tabsContainer) {
        // Remove active class from all tabs
        tabsContainer.querySelectorAll('.tab').forEach(tab => {
            tab.classList.remove('active');
        });
        
        // Check if analytics tab already exists
        let analyticsTab = tabsContainer.querySelector('[data-tab="analytics"]');
        if (!analyticsTab) {
            // Create new tab for Analytics
            const newTab = document.createElement('div');
            newTab.className = 'tab active';
            newTab.setAttribute('data-tab', 'analytics');
            newTab.innerHTML = `
                <span class="tab-icon">📊</span>
                <span class="tab-name">Analytics Dashboard</span>
                <span class="tab-close">×</span>
            `;
            
            // Add click handler for tab
            newTab.addEventListener('click', (e) => {
                if ((e.target as HTMLElement).classList.contains('tab-close')) {
                    // Close tab
                    newTab.remove();
                    hideAnalyticsPanel();
                    // Show welcome content
                    if (welcomeContent) {
                        welcomeContent.style.display = 'block';
                    }
                    // Make Day 0 tab active
                    const day0Tab = tabsContainer.querySelector('[data-tab="day0"]');
                    if (day0Tab) {
                        day0Tab.classList.add('active');
                    }
                } else {
                    // Switch to this tab
                    tabsContainer.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                    newTab.classList.add('active');
                    if (welcomeContent) welcomeContent.style.display = 'none';
                    analyticsPanel.style.display = 'block';
                }
            });
            
            tabsContainer.appendChild(newTab);
        } else {
            analyticsTab.classList.add('active');
        }
    }
    
    // Update button states
    updateButtonStates('analytics');
}

function hideAnalyticsPanel(): void {
    if (analyticsPanel) {
        analyticsDashboard.unmount();
        analyticsPanel.style.display = 'none';
    }
}

// CLI Tools Panel Management
async function renderCliToolsPanel(forceRefresh: boolean = false) {
    const container = document.getElementById('cli-tools-container');
    if (container && (container.innerHTML.trim() === '' || forceRefresh)) {
        console.log('[CLI Tools] Rendering CLI Tools panel...');
        
        // Show loading state first
        container.innerHTML = `
            <div class="cli-tools-panel" style="padding: 24px; height: 100%; overflow-y: auto; background: linear-gradient(135deg, #0E1414 0%, #181E21 100%);">
                <div style="margin-bottom: 24px;">
                    <h2 style="
                        margin: 0 0 8px 0; 
                        color: #FFC107; 
                        font-size: 24px; 
                        font-weight: 700;
                        text-shadow: 0 2px 4px rgba(255, 193, 7, 0.2);
                    ">AI CLI Tools Management</h2>
                    <p style="
                        color: #9CA3AF; 
                        margin: 0;
                        font-size: 14px;
                        font-weight: 500;
                    ">Install and manage AI-powered coding assistants</p>
                </div>
                
                <!-- Batch Action Buttons -->
                <div style="display: flex; gap: 12px; margin-bottom: 24px; align-items: center;">
                    <button onclick="installAllCliTools()" style="
                        padding: 10px 20px;
                        background: linear-gradient(135deg, #FFC107 0%, #FFAD00 100%);
                        color: #0E1414;
                        border: none;
                        border-radius: 8px;
                        font-size: 13px;
                        font-weight: 600;
                        cursor: pointer;
                        transition: all 0.2s ease;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        box-shadow: 0 4px 6px rgba(255, 193, 7, 0.25), 0 1px 3px rgba(0, 0, 0, 0.3);
                        transform: translateY(0);
                    " onmouseover="this.style.background='linear-gradient(135deg, #FFD54F 0%, #FFC107 100%)'; this.style.transform='translateY(-2px) scale(1.02)'; this.style.boxShadow='0 6px 12px rgba(255, 193, 7, 0.35), 0 2px 4px rgba(0, 0, 0, 0.4)'" 
                       onmouseout="this.style.background='linear-gradient(135deg, #FFC107 0%, #FFAD00 100%)'; this.style.transform='translateY(0) scale(1)'; this.style.boxShadow='0 4px 6px rgba(255, 193, 7, 0.25), 0 1px 3px rgba(0, 0, 0, 0.3)'">
                        <span class="codicon codicon-cloud-download" style="font-size: 16px;"></span>
                        <span>Install All Tools</span>
                    </button>
                    <button onclick="updateAllCliTools()" style="
                        padding: 10px 20px;
                        background: linear-gradient(135deg, #007BFF 0%, #0056b3 100%);
                        color: #fff;
                        border: none;
                        border-radius: 8px;
                        font-size: 13px;
                        font-weight: 600;
                        cursor: pointer;
                        transition: all 0.2s ease;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        box-shadow: 0 4px 6px rgba(0, 123, 255, 0.25), 0 1px 3px rgba(0, 0, 0, 0.3);
                        transform: translateY(0);
                    " onmouseover="this.style.background='linear-gradient(135deg, #2489ce 0%, #007BFF 100%)'; this.style.transform='translateY(-2px) scale(1.02)'; this.style.boxShadow='0 6px 12px rgba(0, 123, 255, 0.35), 0 2px 4px rgba(0, 0, 0, 0.4)'" 
                       onmouseout="this.style.background='linear-gradient(135deg, #007BFF 0%, #0056b3 100%)'; this.style.transform='translateY(0) scale(1)'; this.style.boxShadow='0 4px 6px rgba(0, 123, 255, 0.25), 0 1px 3px rgba(0, 0, 0, 0.3)'">
                        <span class="codicon codicon-sync" style="font-size: 16px;"></span>
                        <span>Update All Tools</span>
                    </button>
                    <button onclick="uninstallAllCliTools()" style="
                        padding: 10px 20px;
                        background: #1E2427;
                        color: #9CA3AF;
                        border: 1px solid #2D3336;
                        border-radius: 8px;
                        font-size: 13px;
                        font-weight: 600;
                        cursor: pointer;
                        transition: all 0.2s ease;
                        display: flex;
                        align-items: center;
                        gap: 8px;
                        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
                        transform: translateY(0);
                    " onmouseover="this.style.background='#2D3336'; this.style.color='#ef4444'; this.style.borderColor='#ef4444'; this.style.transform='translateY(-1px)'; this.style.boxShadow='0 4px 6px rgba(239, 68, 68, 0.2)'" 
                       onmouseout="this.style.background='#1E2427'; this.style.color='#9CA3AF'; this.style.borderColor='#2D3336'; this.style.transform='translateY(0)'; this.style.boxShadow='0 2px 4px rgba(0, 0, 0, 0.2)'">
                        <span class="codicon codicon-trash" style="font-size: 16px;"></span>
                        <span>Uninstall All</span>
                    </button>
                    <div id="batch-status" style="
                        display: none;
                        align-items: center;
                        padding: 8px 12px;
                        background: #1e1e1e;
                        border: 1px solid #3e3e42;
                        border-radius: 3px;
                        color: #ccc;
                        font-size: 12px;
                        margin-left: auto;
                    "></div>
                </div>
                
                <div class="cli-tools-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 15px;">
                    <div style="grid-column: 1 / -1; text-align: center; padding: 20px; color: #888;">
                        Detecting installed CLI tools...
                    </div>
                </div>
            </div>
        `;
        
        try {
            // Detect Claude Code installation status
            const electronAPI = window.electronAPI as any as ExtendedElectronAPI;
            const claudeCodeStatus = await electronAPI.detectCliTool('claude-code');
            console.log('[CLI Tools] Claude Code status:', claudeCodeStatus);
            
            // Build the dynamic UI
            const gridContainer = container.querySelector('.cli-tools-grid');
            gridContainer.innerHTML = '';
            
            // Claude Code Card (dynamic)
            gridContainer.appendChild(createCliToolCard({
                id: 'claude-code',
                name: 'Claude Code',
                description: 'Anthropic\'s terminal-native AI agent',
                status: claudeCodeStatus,
                docsUrl: 'https://docs.anthropic.com/en/docs/claude-code',
                badgeText: claudeCodeStatus?.installed ? 'INSTALLED' : null,
                badgeColor: '#007acc'
            }));
            
            // Gemini CLI - now with full detection and functionality
            const geminiStatus = await electronAPI.detectCliTool('gemini-cli');
            gridContainer.appendChild(createCliToolCard({
                id: 'gemini-cli',
                name: 'Gemini CLI',
                description: 'Google\'s free AI coding assistant with 1M token context',
                status: geminiStatus,
                docsUrl: 'https://cloud.google.com/gemini/docs/codeassist/gemini-cli',
                badgeText: 'FREE',
                badgeColor: '#28a745'
            }));
            // Qwen Code - now with full detection and functionality
            const qwenStatus = await electronAPI.detectCliTool('qwen-code');
            gridContainer.appendChild(createCliToolCard({
                id: 'qwen-code',
                name: 'Qwen Code',
                description: 'AI-powered command-line workflow tool (2000 req/day free)',
                status: qwenStatus,
                docsUrl: 'https://github.com/QwenLM/qwen-code',
                badgeText: 'FREE 2K/DAY',
                badgeColor: '#FF6600'  // Orange color for Alibaba
            }));
            // OpenAI Codex - now with full detection and functionality
            const codexStatus = await electronAPI.detectCliTool('openai-codex');
            gridContainer.appendChild(createCliToolCard({
                id: 'openai-codex',
                name: 'OpenAI Codex',
                description: 'OpenAI\'s agentic coding CLI with GPT-5 and o-series models',
                status: codexStatus,
                docsUrl: 'https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started',
                badgeText: codexStatus?.installed ? 'INSTALLED' : null,
                badgeColor: '#007acc'
            }));
            
            // Grok CLI - xAI powered terminal agent with MCP support
            const grokStatus = await electronAPI.detectCliTool('grok');
            gridContainer.appendChild(createCliToolCard({
                id: 'grok',
                name: 'Grok CLI',
                description: 'xAI Grok-powered terminal agent with MCP support',
                status: grokStatus,
                docsUrl: 'https://github.com/superagent-ai/grok-cli',
                badgeText: 'MCP',
                badgeColor: '#ff6b6b'
            }));
            
            // Cline - now with full detection and functionality (moved to bottom)
            const clineStatus = await electronAPI.detectCliTool('cline');
            gridContainer.appendChild(createCliToolCard({
                id: 'cline',
                name: 'Cline',
                description: 'Task-based AI assistant with 47k+ GitHub stars',
                status: clineStatus,
                docsUrl: 'https://cline.bot',
                badgeText: '47K ⭐',
                badgeColor: '#28a745'
            }));
            
            console.log('[CLI Tools] Panel rendered successfully');
        } catch (error) {
            console.error('[CLI Tools] Error rendering panel:', error);
            // Show error state
            container.innerHTML = `
                <div class="cli-tools-panel" style="padding: 20px; height: 100%; overflow-y: auto; background: var(--vscode-editor-background);">
                    <h2 style="margin: 0 0 10px 0; color: #fff;">AI CLI Tools Management</h2>
                    <p style="color: #f44336; margin-bottom: 20px;">Error loading CLI tools. Please try again.</p>
                </div>
            `;
        }
    } else {
        console.log('[CLI Tools] Panel already rendered (use forceRefresh=true to refresh)');
    }
}

// Helper function to create dynamic CLI tool cards
function createCliToolCard(toolInfo: CliToolCardInfo): HTMLDivElement {
    const { id, name, description, status, docsUrl, badgeText, badgeColor } = toolInfo;
    const card = document.createElement('div');
    card.className = 'cli-tool-card';
    card.setAttribute('data-tool-id', id);  // Add data attribute for button handlers
    card.style.cssText = `
        background: linear-gradient(135deg, #1E2427 0%, #181E21 100%); 
        border: 1px solid #2D3336; 
        border-radius: 12px; 
        padding: 20px; 
        transition: all 0.3s ease;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05);
    `;
    
    // Add hover effect to card
    card.onmouseover = () => {
        card.style.transform = 'translateY(-2px)';
        card.style.boxShadow = '0 8px 16px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.08)';
        card.style.borderColor = '#3e444a';
    };
    card.onmouseout = () => {
        card.style.transform = 'translateY(0)';
        card.style.boxShadow = '0 4px 6px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05)';
        card.style.borderColor = '#2D3336';
    };
    
    const isInstalled = status?.installed || false;
    const version = status?.version;
    const memoryConnected = status?.memoryServiceConnected || false;
    
    // Status badge
    let statusBadge = '';
    if (badgeText) {
        statusBadge = `<span style="background: ${badgeColor}; color: #fff; padding: 2px 6px; border-radius: 3px; font-size: 10px; margin-left: 8px;">${badgeText}</span>`;
    }
    
    // Status details
    let statusDetails = '';
    if (isInstalled) {
        statusDetails = `
            <div><span style="color: #aaa;">Version:</span> <span data-version="${id}">${version || 'Unknown'}</span></div>
            <div><span style="color: #aaa;">Memory:</span> <span data-memory="${id}" style="color: ${memoryConnected ? '#4caf50' : '#f44336'};">${memoryConnected ? 'Connected ✓' : 'Not connected'}</span></div>
            <div><span style="color: #aaa;">Path:</span> ${status.path || 'Unknown'}</div>
        `;
    } else {
        statusDetails = `
            <div><span style="color: #aaa;">Status:</span> Not Installed</div>
            <div><span style="color: #aaa;">Installation:</span> npm install -g ${id}</div>
        `;
    }
    
    // Buttons based on status
    let buttons = '';
    if (isInstalled) {
        buttons = `
            <button onclick="launchCliTool('${id}')" style="
                flex: 1; 
                padding: 8px 12px; 
                background: linear-gradient(135deg, #FFC107 0%, #007BFF 100%); 
                color: #fff; 
                border: none; 
                border-radius: 8px; 
                font-size: 11px; 
                font-weight: 600;
                cursor: pointer; 
                transition: all 0.2s ease;
                box-shadow: 0 4px 6px rgba(255, 193, 7, 0.25);
            " onmouseover="this.style.transform='scale(1.05) translateY(-1px)'; this.style.boxShadow='0 6px 12px rgba(255, 193, 7, 0.35)'" 
               onmouseout="this.style.transform='scale(1) translateY(0)'; this.style.boxShadow='0 4px 6px rgba(255, 193, 7, 0.25)'" 
               title="Launch in current project">Launch</button>
            <button onclick="refreshCliToolDetails('${id}')" style="
                flex: 1; 
                padding: 8px 12px; 
                background: #1E2427; 
                color: #FFD54F; 
                border: 1px solid #2D3336; 
                border-radius: 8px; 
                font-size: 11px; 
                font-weight: 500;
                cursor: pointer; 
                transition: all 0.2s ease;
            " onmouseover="this.style.background='#2D3336'; this.style.borderColor='#FFC107'; this.style.transform='translateY(-1px)'" 
               onmouseout="this.style.background='#1E2427'; this.style.borderColor='#2D3336'; this.style.transform='translateY(0)'" 
               title="Refresh tool details">Details</button>
            <button onclick="updateCliTool('${id}')" style="
                flex: 1; 
                padding: 8px 12px; 
                background: linear-gradient(135deg, #8A2BE2 0%, #007BFF 100%); 
                color: #fff; 
                border: none; 
                border-radius: 8px; 
                font-size: 11px; 
                font-weight: 600;
                cursor: pointer; 
                transition: all 0.2s ease;
                box-shadow: 0 4px 6px rgba(138, 43, 226, 0.25);
            " onmouseover="this.style.transform='scale(1.05) translateY(-1px)'; this.style.boxShadow='0 6px 12px rgba(138, 43, 226, 0.35)'" 
               onmouseout="this.style.transform='scale(1) translateY(0)'; this.style.boxShadow='0 4px 6px rgba(138, 43, 226, 0.25)'">Update</button>
            <button onclick="uninstallCliTool('${id}')" style="
                flex: 1; 
                padding: 8px 12px; 
                background: #1E2427; 
                color: #FF6B6B; 
                border: 1px solid #2D3336; 
                border-radius: 8px; 
                font-size: 11px; 
                font-weight: 500;
                cursor: pointer; 
                transition: all 0.2s ease;
            " onmouseover="this.style.background='#CC3D3D'; this.style.color='white'; this.style.borderColor='#CC3D3D'; this.style.transform='translateY(-1px)'" 
               onmouseout="this.style.background='#1E2427'; this.style.color='#FF6B6B'; this.style.borderColor='#2D3336'; this.style.transform='translateY(0)'" 
               title="Uninstall this tool">Uninstall</button>
        `;
    } else {
        buttons = `
            <button onclick="installCliTool('${id}')" style="
                flex: 1; 
                padding: 8px 12px; 
                background: linear-gradient(135deg, #28A745 0%, #20C997 100%); 
                color: #fff; 
                border: none; 
                border-radius: 8px; 
                font-size: 11px; 
                font-weight: 600;
                cursor: pointer; 
                transition: all 0.2s ease;
                box-shadow: 0 4px 6px rgba(40, 167, 69, 0.25);
            " onmouseover="this.style.transform='scale(1.05) translateY(-1px)'; this.style.boxShadow='0 6px 12px rgba(40, 167, 69, 0.35)'" 
               onmouseout="this.style.transform='scale(1) translateY(0)'; this.style.boxShadow='0 4px 6px rgba(40, 167, 69, 0.25)'">Install</button>
        `;
    }
    buttons += `<button onclick="window.open('${docsUrl}', '_blank')" style="
        padding: 8px 12px; 
        background: #1E2427; 
        color: #FFD54F; 
        border: 1px solid #2D3336; 
        border-radius: 8px; 
        font-size: 11px; 
        font-weight: 500;
        cursor: pointer; 
        transition: all 0.2s ease;
    " onmouseover="this.style.background='#2D3336'; this.style.borderColor='#FFC107'; this.style.transform='translateY(-1px)'" 
       onmouseout="this.style.background='#1E2427'; this.style.borderColor='#2D3336'; this.style.transform='translateY(0)'" 
       title="View official documentation">Docs</button>`;
    
    card.innerHTML = `
        <h4 style="margin: 0 0 8px 0; color: #fff; font-size: 15px;">
            ${name}${statusBadge}
        </h4>
        <div style="color: #aaa; font-size: 12px; margin-bottom: 12px;">${description}</div>
        <div style="border-top: 1px solid #3e3e42; padding-top: 10px; margin-top: 10px;">
            <div class="tool-status" style="font-size: 11px; color: #888; line-height: 1.6;">
                ${statusDetails}
            </div>
        </div>
        <div style="margin-top: 12px; display: flex; gap: 8px;">
            ${buttons}
        </div>
    `;
    
    return card;
}

// Helper function for static tool cards (temporary for other tools)
function createStaticToolCard(id: string, name: string, description: string, badgeText: string | null, badgeColor: string | null, docsUrl: string): HTMLDivElement {
    const card = document.createElement('div');
    card.className = 'cli-tool-card';
    card.style.cssText = `
        background: linear-gradient(135deg, #1E2427 0%, #181E21 100%); 
        border: 1px solid #2D3336; 
        border-radius: 12px; 
        padding: 20px; 
        transition: all 0.3s ease;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05);
    `;
    
    // Add hover effect to card
    card.onmouseover = () => {
        card.style.transform = 'translateY(-2px)';
        card.style.boxShadow = '0 8px 16px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.08)';
        card.style.borderColor = '#3e444a';
    };
    card.onmouseout = () => {
        card.style.transform = 'translateY(0)';
        card.style.boxShadow = '0 4px 6px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.05)';
        card.style.borderColor = '#2D3336';
    };
    
    const statusBadge = badgeText ? `<span style="background: ${badgeColor}; color: #fff; padding: 2px 6px; border-radius: 3px; font-size: 10px; margin-left: 8px;">${badgeText}</span>` : '';
    
    card.innerHTML = `
        <h4 style="margin: 0 0 8px 0; color: #fff; font-size: 15px;">
            ${name}${statusBadge}
        </h4>
        <div style="color: #aaa; font-size: 12px; margin-bottom: 12px;">${description}</div>
        <div style="border-top: 1px solid #3e3e42; padding-top: 10px; margin-top: 10px;">
            <div style="font-size: 11px; color: #888; line-height: 1.6;">
                <div><span style="color: #aaa;">Status:</span> Not Installed</div>
                <div><span style="color: #aaa;">Detection:</span> Coming soon</div>
            </div>
        </div>
        <div style="margin-top: 12px; display: flex; gap: 8px;">
            <button onclick="alert('Installation coming soon')" style="flex: 1; padding: 6px; background: #6c757d; color: #fff; border: none; border-radius: 3px; font-size: 12px; cursor: pointer;" disabled>Install</button>
            <button onclick="window.open('${docsUrl}', '_blank')" style="padding: 6px 12px; background: #3e3e42; color: #ccc; border: none; border-radius: 3px; font-size: 11px; cursor: pointer;" title="View official documentation">Docs</button>
        </div>
    `;
    
    return card;
}

// CLI Tool Action Handlers
/**
 * Refresh a specific sidebar tool icon after installation/update
 */
async function refreshSidebarToolIcon(toolId: string): Promise<void> {
    console.log(`[Sidebar] Refreshing icon status for ${toolId}`);
    
    const btn = document.querySelector(`.cli-quick-launch[data-tool="${toolId}"]`) as HTMLElement;
    if (!btn) return;
    
    // Check the current installation status
    const electronAPI = window.electronAPI as any;
    const toolStatus = await electronAPI.detectCliTool(toolId);
    
    // Remove any existing status indicator
    const existingIndicator = btn.querySelector('.cli-tool-status-indicator');
    if (existingIndicator) {
        existingIndicator.remove();
    }
    
    if (toolStatus.installed) {
        // Tool is installed - remove the not-installed class and restore full opacity
        btn.classList.remove('not-installed');
        console.log(`[Sidebar] ${toolId} icon updated - tool is installed`);
    } else {
        // Tool is not installed - add indicator
        const indicator = document.createElement('span');
        indicator.className = 'cli-tool-status-indicator';
        indicator.innerHTML = '⬇';  // Download arrow icon
        indicator.title = 'Not installed - Click to install';
        btn.appendChild(indicator);
        btn.classList.add('not-installed');
        console.log(`[Sidebar] ${toolId} icon updated - tool is not installed`);
    }
}

async function installCliTool(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Install requested for ${toolId}`);
    
    // Show progress in the UI
    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
    if (card) {
        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
        if (statusDiv) {
            statusDiv.innerHTML = '⏳ Installing...';
            statusDiv.style.color = '#FFA500';
        }
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        const result = await electronAPI.installCliTool(toolId);
        
        if (result.success) {
            console.log(`[CLI Tools] ${toolId} installed successfully`);
            // Force refresh the entire panel to update status
            await renderCliToolsPanel(true);
            // Also refresh the sidebar icon for this tool
            await refreshSidebarToolIcon(toolId);
        } else {
            console.error(`[CLI Tools] Failed to install ${toolId}:`, result.error);
            alert(`Failed to install: ${result.error}`);
        }
    } catch (error) {
        console.error(`[CLI Tools] Error installing ${toolId}:`, error);
        alert(`Installation error: ${error}`);
    }
}

// DEPRECATED: Configuration is now done automatically during installation
// This function is kept for backward compatibility but should not be called
async function configureCliTool(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Configure requested for ${toolId} - DEPRECATED: Configuration is now automatic during installation`);
    
    // Since configuration is now automatic, just show a message
    alert(`${toolId} is already configured! Configuration is now done automatically during installation.`);
    return;
    
    /* OLD CODE - PRESERVED FOR REFERENCE
    // Show progress in the UI
    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
    if (card) {
        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
        if (statusDiv) {
            statusDiv.innerHTML = '⚙️ Configuring...';
            statusDiv.style.color = '#FFA500';
        }
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        const result = await electronAPI.configureCliTool(toolId);
        
        if (result && result.success) {
            console.log(`[CLI Tools] ${toolId} configured successfully`);
            
            // Update UI immediately to show success
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    statusDiv.innerHTML = '✅ Configured';
                    statusDiv.style.color = '#4ec9b0';
                }
            }
            
            // Show success message
            if (toolId === 'claude-code') {
                setTimeout(() => {
                    alert('Claude Code has been configured with Memory Service integration! You can now use Claude Code with enhanced memory capabilities.');
                }, 100);
            } else {
                setTimeout(() => {
                    alert(`${toolId} configured successfully!`);
                }, 100);
            }
            
            // Don't refresh the entire panel, just update the Memory status
            setTimeout(async () => {
                if (card) {
                    // Find and update the Memory status line using data attribute
                    const memorySpan = card.querySelector(`span[data-memory="${toolId}"]`) as HTMLElement;
                    if (memorySpan) {
                        memorySpan.textContent = 'Connected ✓';
                        memorySpan.style.color = '#4caf50';
                    }
                }
            }, 500);
        } else {
            console.error(`[CLI Tools] Failed to configure ${toolId}:`, result?.error || 'Unknown error');
            
            // Update UI to show error
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    statusDiv.innerHTML = '❌ Config failed';
                    statusDiv.style.color = '#f44747';
                }
            }
            
            if (result?.error) {
                alert(`Failed to configure: ${result.error}`);
            }
        }
    } catch (error) {
        console.error(`[CLI Tools] Error configuring ${toolId}:`, error);
        alert(`Configuration error: ${error}`);
    }
    */
}

async function updateCliTool(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Update requested for ${toolId}`);
    
    // Show progress in the UI
    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
    if (card) {
        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
        if (statusDiv) {
            statusDiv.innerHTML = '⬆️ Updating...';
            statusDiv.style.color = '#FFA500';
        }
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        const result = await electronAPI.updateCliTool(toolId);
        
        if (result && result.success) {
            console.log(`[CLI Tools] ${toolId} updated successfully`);
            
            // Update UI immediately to show success
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    statusDiv.innerHTML = '✅ Up to date';
                    statusDiv.style.color = '#4ec9b0';
                }
            }
            
            // Force refresh the panel to show updated version
            setTimeout(async () => {
                await renderCliToolsPanel(true);
                // Also refresh the sidebar icon
                await refreshSidebarToolIcon(toolId);
            }, 1000);
        } else {
            console.error(`[CLI Tools] Failed to update ${toolId}:`, result?.error || 'Unknown error');
            
            // Update UI to show error
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    statusDiv.innerHTML = '❌ Update failed';
                    statusDiv.style.color = '#f44747';
                }
            }
            
            if (result?.error) {
                alert(`Failed to update: ${result.error}`);
            }
        }
    } catch (error) {
        console.error(`[CLI Tools] Error updating ${toolId}:`, error);
        
        // Update UI to show error
        if (card) {
            const statusDiv = card.querySelector('.tool-status') as HTMLElement;
            if (statusDiv) {
                statusDiv.innerHTML = '❌ Error';
                statusDiv.style.color = '#f44747';
            }
        }
        
        alert(`Update error: ${error}`);
    }
}

/**
 * Install all AI CLI tools in sequence
 */
async function installAllCliTools(): Promise<void> {
    console.log('[CLI Tools] Installing all tools...');
    
    // List of tools to install (in order)
    const toolsToInstall = [
        'claude-code',
        'gemini-cli', 
        'qwen-code',
        'openai-codex',
        'grok',
        'cline'  // Moved to last as least likely to be used
    ];
    
    // Show status
    const statusDiv = document.getElementById('batch-status');
    if (statusDiv) {
        statusDiv.style.display = 'flex';
        statusDiv.innerHTML = '⏳ Checking tools...';
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        let installedCount = 0;
        let skippedCount = 0;
        let failedCount = 0;
        
        for (const toolId of toolsToInstall) {
            // Check if already installed
            const status = await electronAPI.detectCliTool(toolId);
            
            if (status?.installed) {
                console.log(`[CLI Tools] ${toolId} already installed, skipping`);
                skippedCount++;
                if (statusDiv) {
                    statusDiv.innerHTML = `⏳ ${toolId} already installed (${installedCount} installed, ${skippedCount} skipped)`;
                }
                continue;
            }
            
            // Install the tool
            if (statusDiv) {
                statusDiv.innerHTML = `📦 Installing ${toolId}... (${installedCount} installed so far)`;
            }
            
            try {
                const result = await electronAPI.installCliTool(toolId);
                
                if (result?.success) {
                    installedCount++;
                    console.log(`[CLI Tools] Successfully installed ${toolId}`);
                    
                    // Update the specific card UI
                    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
                    if (card) {
                        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                        if (statusDiv) {
                            statusDiv.innerHTML = '✅ Installed';
                            statusDiv.style.color = '#4ec9b0';
                        }
                    }
                } else {
                    failedCount++;
                    console.error(`[CLI Tools] Failed to install ${toolId}:`, result?.error);
                }
            } catch (error) {
                failedCount++;
                console.error(`[CLI Tools] Error installing ${toolId}:`, error);
            }
        }
        
        // Final status
        if (statusDiv) {
            if (failedCount === 0) {
                statusDiv.innerHTML = `✅ Complete! ${installedCount} installed, ${skippedCount} already installed`;
                statusDiv.style.color = '#4ec9b0';
            } else {
                statusDiv.innerHTML = `⚠️ Complete with errors: ${installedCount} installed, ${skippedCount} skipped, ${failedCount} failed`;
                statusDiv.style.color = '#ffa500';
            }
            
            // Hide status after 5 seconds
            setTimeout(() => {
                statusDiv.style.display = 'none';
            }, 5000);
        }
        
        // Refresh the entire panel to show updated statuses and sidebar icons
        setTimeout(async () => {
            await renderCliToolsPanel(true);
            // Refresh all sidebar icons
            const toolsToInstall = ['claude-code', 'gemini-cli', 'qwen-code', 'openai-codex', 'grok', 'cline'];
            for (const toolId of toolsToInstall) {
                await refreshSidebarToolIcon(toolId);
            }
        }, 1000);
        
    } catch (error) {
        console.error('[CLI Tools] Error in batch install:', error);
        if (statusDiv) {
            statusDiv.innerHTML = `❌ Error: ${error}`;
            statusDiv.style.color = '#f44747';
        }
    }
}

/**
 * Update all installed AI CLI tools in sequence
 */
async function updateAllCliTools(): Promise<void> {
    console.log('[CLI Tools] Updating all tools...');
    
    // List of tools to update (in order)
    const toolsToUpdate = [
        'claude-code',
        'gemini-cli',
        'qwen-code', 
        'openai-codex',
        'grok',
        'cline'  // Moved to last as least likely to be used
    ];
    
    // Show status
    const statusDiv = document.getElementById('batch-status');
    if (statusDiv) {
        statusDiv.style.display = 'flex';
        statusDiv.innerHTML = '⏳ Checking for updates...';
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        let updatedCount = 0;
        let skippedCount = 0;
        let failedCount = 0;
        
        for (const toolId of toolsToUpdate) {
            // Check if installed
            const status = await electronAPI.detectCliTool(toolId);
            
            if (!status?.installed) {
                console.log(`[CLI Tools] ${toolId} not installed, skipping update`);
                skippedCount++;
                continue;
            }
            
            // Update the tool
            if (statusDiv) {
                statusDiv.innerHTML = `🔄 Updating ${toolId}... (${updatedCount} updated so far)`;
            }
            
            try {
                const result = await electronAPI.updateCliTool(toolId);
                
                if (result?.success) {
                    updatedCount++;
                    console.log(`[CLI Tools] Successfully updated ${toolId} to version ${result.version}`);
                    
                    // Update the specific card UI
                    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
                    if (card) {
                        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                        if (statusDiv) {
                            statusDiv.innerHTML = '✅ Updated';
                            statusDiv.style.color = '#4ec9b0';
                        }
                        
                        // Update version display if present
                        const versionSpan = card.querySelector('span[style*="color: #4ec9b0"]') as HTMLElement;
                        if (versionSpan && result.version) {
                            versionSpan.textContent = `v${result.version}`;
                        }
                    }
                } else {
                    // Could be no update available, which is not a failure
                    if (result?.error?.includes('already up to date')) {
                        console.log(`[CLI Tools] ${toolId} already up to date`);
                    } else {
                        failedCount++;
                        console.error(`[CLI Tools] Failed to update ${toolId}:`, result?.error);
                    }
                }
            } catch (error) {
                failedCount++;
                console.error(`[CLI Tools] Error updating ${toolId}:`, error);
            }
        }
        
        // Final status
        if (statusDiv) {
            if (failedCount === 0) {
                statusDiv.innerHTML = `✅ Complete! ${updatedCount} updated, ${skippedCount} not installed`;
                statusDiv.style.color = '#4ec9b0';
            } else {
                statusDiv.innerHTML = `⚠️ Complete with errors: ${updatedCount} updated, ${skippedCount} skipped, ${failedCount} failed`;
                statusDiv.style.color = '#ffa500';
            }
            
            // Hide status after 5 seconds
            setTimeout(() => {
                statusDiv.style.display = 'none';
            }, 5000);
        }
        
        // Refresh the entire panel to show updated statuses and sidebar icons
        setTimeout(async () => {
            await renderCliToolsPanel(true);
            // Refresh all sidebar icons
            const toolsToUpdate = ['claude-code', 'gemini-cli', 'qwen-code', 'openai-codex', 'grok', 'cline'];
            for (const toolId of toolsToUpdate) {
                await refreshSidebarToolIcon(toolId);
            }
        }, 1000);
        
    } catch (error) {
        console.error('[CLI Tools] Error in batch update:', error);
        if (statusDiv) {
            statusDiv.innerHTML = `❌ Error: ${error}`;
            statusDiv.style.color = '#f44747';
        }
    }
}

/**
 * Uninstall a CLI tool
 */
async function uninstallCliTool(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Uninstall requested for ${toolId}`);
    
    // Confirm with user first
    const toolName = (window as any).CLI_TOOLS_REGISTRY?.[toolId]?.name || toolId;
    const confirmed = confirm(`Are you sure you want to uninstall ${toolName}?\n\nThis will remove the tool globally from your system.`);
    
    if (!confirmed) {
        console.log(`[CLI Tools] Uninstall canceled for ${toolId}`);
        return;
    }
    
    // Show progress in the UI
    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
    if (card) {
        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
        if (statusDiv) {
            statusDiv.innerHTML = '🗑️ Uninstalling...';
            statusDiv.style.color = '#FFA500';
        }
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        const result = await electronAPI.uninstallCliTool(toolId);
        
        if (result && result.success) {
            console.log(`[CLI Tools] ${toolId} uninstalled successfully`);
            
            // Update UI immediately to show not installed
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    statusDiv.innerHTML = '✅ Uninstalled';
                    statusDiv.style.color = '#4ec9b0';
                }
            }
            
            // Refresh the panel after a short delay to update buttons
            setTimeout(async () => {
                await renderCliToolsPanel(true);
                // Also refresh the sidebar icon to show it's uninstalled
                await refreshSidebarToolIcon(toolId);
            }, 1500);
            
        } else {
            console.error(`[CLI Tools] Failed to uninstall ${toolId}:`, result?.error || 'Unknown error');
            
            // Update UI to show error
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    statusDiv.innerHTML = '❌ Uninstall failed';
                    statusDiv.style.color = '#f44747';
                }
            }
            
            if (result?.error) {
                alert(`Failed to uninstall ${toolName}: ${result.error}`);
            }
        }
    } catch (error) {
        console.error(`[CLI Tools] Error uninstalling ${toolId}:`, error);
        
        // Update UI to show error
        if (card) {
            const statusDiv = card.querySelector('.tool-status') as HTMLElement;
            if (statusDiv) {
                statusDiv.innerHTML = '❌ Error';
                statusDiv.style.color = '#f44747';
            }
        }
        
        alert(`Uninstall error: ${error}`);
    }
}

/**
 * Uninstall all installed AI CLI tools in sequence
 */
async function uninstallAllCliTools(): Promise<void> {
    console.log('[CLI Tools] Uninstalling all tools...');
    
    // Confirm with user first
    const confirmed = confirm('Are you sure you want to uninstall ALL AI CLI tools?\n\nThis will remove all tools globally from your system.\nYour configurations will be preserved for potential reinstallation.');
    
    if (!confirmed) {
        console.log('[CLI Tools] Batch uninstall canceled');
        return;
    }
    
    // List of tools to uninstall (in order)
    const toolsToUninstall = [
        'claude-code',
        'gemini-cli',
        'qwen-code',
        'openai-codex',
        'grok',
        'cline'  // Moved to last as least likely to be used
    ];
    
    // Show status
    const statusDiv = document.getElementById('batch-status');
    if (statusDiv) {
        statusDiv.style.display = 'flex';
        statusDiv.innerHTML = '⏳ Checking installed tools...';
    }
    
    try {
        const electronAPI = window.electronAPI as any;
        let uninstalledCount = 0;
        let skippedCount = 0;
        let failedCount = 0;
        
        for (const toolId of toolsToUninstall) {
            // Check if installed
            const status = await electronAPI.detectCliTool(toolId);
            
            if (!status?.installed) {
                console.log(`[CLI Tools] ${toolId} not installed, skipping`);
                skippedCount++;
                continue;
            }
            
            // Uninstall the tool
            if (statusDiv) {
                statusDiv.innerHTML = `🗑️ Uninstalling ${toolId}... (${uninstalledCount} uninstalled so far)`;
            }
            
            try {
                const result = await electronAPI.uninstallCliTool(toolId);
                
                if (result?.success) {
                    uninstalledCount++;
                    console.log(`[CLI Tools] Successfully uninstalled ${toolId}`);
                    
                    // Update the specific card UI
                    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
                    if (card) {
                        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                        if (statusDiv) {
                            statusDiv.innerHTML = '✅ Uninstalled';
                            statusDiv.style.color = '#4ec9b0';
                        }
                    }
                } else {
                    failedCount++;
                    console.error(`[CLI Tools] Failed to uninstall ${toolId}:`, result?.error);
                }
            } catch (error) {
                failedCount++;
                console.error(`[CLI Tools] Error uninstalling ${toolId}:`, error);
            }
        }
        
        // Final status
        if (statusDiv) {
            if (failedCount === 0) {
                statusDiv.innerHTML = `✅ Complete! ${uninstalledCount} uninstalled, ${skippedCount} not installed`;
                statusDiv.style.color = '#4ec9b0';
            } else {
                statusDiv.innerHTML = `⚠️ Complete with errors: ${uninstalledCount} uninstalled, ${skippedCount} skipped, ${failedCount} failed`;
                statusDiv.style.color = '#ffa500';
            }
            
            // Hide status after 5 seconds
            setTimeout(() => {
                statusDiv.style.display = 'none';
            }, 5000);
        }
        
        // Refresh the entire panel to show updated statuses and sidebar icons
        setTimeout(async () => {
            await renderCliToolsPanel(true);
            // Refresh all sidebar icons to show uninstalled status
            const toolsToUninstall = ['claude-code', 'gemini-cli', 'qwen-code', 'openai-codex', 'grok', 'cline'];
            for (const toolId of toolsToUninstall) {
                await refreshSidebarToolIcon(toolId);
            }
        }, 1000);
        
    } catch (error) {
        console.error('[CLI Tools] Error in batch uninstall:', error);
        if (statusDiv) {
            statusDiv.innerHTML = `❌ Error: ${error}`;
            statusDiv.style.color = '#f44747';
        }
    }
}

// Make functions available globally
(window as any).installAllCliTools = installAllCliTools;
(window as any).updateAllCliTools = updateAllCliTools;
(window as any).uninstallAllCliTools = uninstallAllCliTools;
(window as any).uninstallCliTool = uninstallCliTool;

/**
 * Launch a CLI tool in the current project context
 */
async function launchCliTool(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Launch requested for ${toolId}`);
    
    // First, check if the tool is installed
    const electronAPI = window.electronAPI as any;
    const toolStatus = await electronAPI.detectCliTool(toolId);
    
    if (!toolStatus.installed) {
        console.log(`[CLI Tools] ${toolId} is not installed. Redirecting to CLI Tools panel...`);
        
        // Switch to CLI Tools panel
        const cliToolsButton = document.querySelector('[data-view="cli-tools"]') as HTMLElement;
        if (cliToolsButton) {
            cliToolsButton.click();
            
            // Wait for panel to render, then highlight the install button
            setTimeout(() => {
                const toolCard = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
                if (toolCard) {
                    // Add highlight effect
                    toolCard.style.animation = 'pulse-highlight 1s ease-in-out 3';
                    
                    // Scroll to the card
                    toolCard.scrollIntoView({ behavior: 'smooth', block: 'center' });
                    
                    // Show a notification in the card
                    const statusDiv = toolCard.querySelector('.tool-status') as HTMLElement;
                    if (statusDiv) {
                        statusDiv.innerHTML = '⚠️ Please install this tool first';
                        statusDiv.style.color = '#FFC107';
                    }
                }
            }, 300);
        }
        return;
    }
    
    // Tool is installed, proceed with launch
    // Show launching status  
    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
    if (card) {
        const statusDiv = card.querySelector('.tool-status') as HTMLElement;
        if (statusDiv) {
            statusDiv.innerHTML = '📂 Select a folder to launch in...';
            statusDiv.style.color = '#2196F3';
        }
    }
    
    try {
        // Call the IPC handler which will:
        // 1. Show folder selection dialog
        // 2. Check database for previous launches
        // 3. Determine command (claude vs claude --resume)
        // 4. Send event to create terminal
        const result = await electronAPI.launchCliTool(toolId);
        
        if (result.success) {
            console.log(`[CLI Tools] ${toolId} launched successfully with command: ${result.command}`);
            
            // Update status to show it's running
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    const resumeText = result.command?.includes('--resume') ? ' (resumed)' : '';
                    statusDiv.innerHTML = `✅ Launched in ${result.path}${resumeText}`;
                    statusDiv.style.color = '#4CAF50';
                    
                    // Reset status after 5 seconds
                    setTimeout(() => {
                        renderCliToolsPanel();
                    }, 5000);
                }
            }
        } else if (result.error) {
            console.log(`[CLI Tools] Launch cancelled or failed: ${result.error}`);
            
            // Reset status
            if (card) {
                const statusDiv = card.querySelector('.tool-status') as HTMLElement;
                if (statusDiv) {
                    if (result.error === 'Folder selection canceled') {
                        // User cancelled - just refresh the panel
                        renderCliToolsPanel();
                    } else {
                        statusDiv.innerHTML = `❌ ${result.error}`;
                        statusDiv.style.color = '#f44336';
                        setTimeout(() => {
                            renderCliToolsPanel();
                        }, 3000);
                    }
                }
            }
        }
    } catch (error) {
        console.error(`[CLI Tools] Error launching ${toolId}:`, error);
        
        if (card) {
            const statusDiv = card.querySelector('.tool-status') as HTMLElement;
            if (statusDiv) {
                statusDiv.innerHTML = `❌ Launch failed: ${error}`;
                statusDiv.style.color = '#f44336';
            }
        }
    }
}

/**
 * Refresh CLI tool details to show full status
 */
async function refreshCliToolDetails(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Refreshing details for ${toolId}`);
    
    const card = document.querySelector(`[data-tool-id="${toolId}"]`) as HTMLElement;
    if (!card) return;
    
    // Show loading state
    const statusDiv = card.querySelector('.tool-status') as HTMLElement;
    if (statusDiv) {
        statusDiv.innerHTML = '🔄 Loading details...';
        statusDiv.style.color = '#FFA500';
    }
    
    try {
        // Re-detect the tool to get fresh status
        const electronAPI = window.electronAPI as any;
        const status = await electronAPI.detectCliTool(toolId);
        
        if (status && status.installed) {
            // Rebuild the status details section
            const statusDetailsHtml = `
                <div><span style="color: #aaa;">Version:</span> <span data-version="${toolId}">${status.version || 'Unknown'}</span></div>
                <div><span style="color: #aaa;">Memory:</span> <span data-memory="${toolId}" style="color: ${status.memoryServiceConnected ? '#4caf50' : '#f44336'};">${status.memoryServiceConnected ? 'Connected ✓' : 'Not connected'}</span></div>
                <div><span style="color: #aaa;">Path:</span> ${status.path || 'Unknown'}</div>
            `;
            
            // Update the status div with the full details
            if (statusDiv) {
                statusDiv.innerHTML = statusDetailsHtml;
                statusDiv.style.color = '';
            }
            
            console.log(`[CLI Tools] Details refreshed for ${toolId}:`, status);
        } else {
            // Tool not installed or error
            if (statusDiv) {
                statusDiv.innerHTML = '❌ Tool not found';
                statusDiv.style.color = '#f44747';
            }
        }
    } catch (error) {
        console.error(`[CLI Tools] Error refreshing details for ${toolId}:`, error);
        if (statusDiv) {
            statusDiv.innerHTML = '❌ Error loading details';
            statusDiv.style.color = '#f44747';
        }
    }
}

// Expose CLI tool functions to window for onclick handlers
(window as any).installCliTool = installCliTool;
(window as any).configureCliTool = configureCliTool;
(window as any).updateCliTool = updateCliTool;
(window as any).refreshCliToolDetails = refreshCliToolDetails;
(window as any).launchCliTool = launchCliTool;

// Memory Dashboard Management
let memoryDashboardInstance: MemoryDashboard | null = null;

async function openMemoryDashboard(): Promise<void> {
    console.log('[Memory] Opening Memory Dashboard...');
    
    const container = document.getElementById('memory-container');
    if (!container) {
        console.error('[Memory] Memory container not found');
        return;
    }
    
    // Check if Memory Service is running, start if not
    const isRunning = await window.electronAPI.isMemoryServiceRunning();
    if (!isRunning) {
        console.log('[Memory] Starting Memory Service...');
        const started = await window.electronAPI.startMemoryService();
        if (!started) {
            console.error('[Memory] Failed to start Memory Service');
            container.innerHTML = `
                <div style="padding: 20px; color: #f48771;">
                    <h3>Memory Service Failed to Start</h3>
                    <p>Please check the console for errors.</p>
                </div>
            `;
            return;
        }
        
        // Wait a moment for service to initialize
        await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    // Check if dashboard already exists
    if (container.innerHTML.trim() === '') {
        console.log('[Memory] Creating new Memory Dashboard...');
        
        // Clean up old instance if exists
        if (memoryDashboardInstance) {
            memoryDashboardInstance.destroy();
            memoryDashboardInstance = null;
        }
        
        // Create new dashboard
        memoryDashboardInstance = new MemoryDashboard();
        const dashboardElement = await memoryDashboardInstance.create();
        container.innerHTML = '';
        container.appendChild(dashboardElement);
        
        console.log('[Memory] Memory Dashboard created successfully');
    } else {
        console.log('[Memory] Memory Dashboard already rendered');
    }
}

function hideAllPanels(): void {
    // Hide console output
    const consoleOutput = document.getElementById('console-output');
    if (consoleOutput) consoleOutput.style.display = 'none';
    
    // Hide analytics panel
    hideAnalyticsPanel();
    
    // Add more panels here as they are created
}

function updateButtonStates(activeButton: string): void {
    // Remove active class from all buttons
    document.querySelectorAll('.sidebar-button').forEach(btn => {
        btn.classList.remove('active');
    });
    
    // Add active class to the current button
    const activeBtn = document.getElementById(`${activeButton}-btn`);
    if (activeBtn) {
        activeBtn.classList.add('active');
    }
}

// Analytics data tracking
interface StageMetrics {
  stage: string;
  model: string;
  tokens: number;
  cost: number;
  timestamp: string;
  duration: number;
}

let sessionMetrics: {
  totalQueries: number;
  totalCost: number;
  successRate: number;
  avgResponseTime: number;
  modelUsage: { [model: string]: number };
  recentActivity: any[];
  hourlyStats: any[];
  costByModel: { [model: string]: number };
  tokenUsage: {
    total: number;
    input: number;
    output: number;
  };
  stageMetrics: StageMetrics[];
} = {
  totalQueries: 0,
  totalCost: 0,
  successRate: 100,
  avgResponseTime: 0,
  modelUsage: {},
  recentActivity: [],
  hourlyStats: [],
  costByModel: {},
  tokenUsage: {
    total: 0,
    input: 0,
    output: 0
  },
  stageMetrics: []
};

// Load existing metrics from storage
const loadSessionMetrics = () => {
  const stored = localStorage.getItem('consensusMetrics');
  if (stored) {
    try {
      sessionMetrics = JSON.parse(stored);
    } catch (e) {
      console.error('Failed to load session metrics:', e);
    }
  }
};

// Save consensus analytics
const saveConsensusAnalytics = async (totalTokens: number, totalCost: number) => {
  const timestamp = new Date().toISOString();
  
  // Generate a unique conversation ID
  const conversationId = `conv-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  
  // Get the current question from the chat
  const chatContent = document.getElementById('chat-content');
  const userMessages = chatContent?.querySelectorAll('.user-message');
  const lastUserMessage = userMessages?.[userMessages.length - 1]?.textContent || 'Consensus query';
  
  // Get the assistant's response
  const assistantMessages = chatContent?.querySelectorAll('.assistant-message');
  const lastAssistantMessage = assistantMessages?.[assistantMessages.length - 1]?.textContent || '';
  
  // Save to database via Electron API
  try {
    const electronAPI = (window as any).electronAPI;
    if (electronAPI && electronAPI.saveConversation) {
      // Log the values being saved
      console.log('💾 Saving conversation with:', {
        conversationId,
        totalCost,
        totalTokens,
        question: lastUserMessage.substring(0, 50) + '...'
      });
      
      const saved = await electronAPI.saveConversation({
        conversationId,
        question: lastUserMessage,
        answer: lastAssistantMessage,
        totalCost,
        totalTokens,
        inputTokens: Math.floor(totalTokens * 0.7),
        outputTokens: Math.floor(totalTokens * 0.3),
        duration: performance.now() - conversationStartTime,
        model: 'consensus-pipeline'
      });
      
      if (saved) {
        console.log('✅ Conversation saved to database with cost $' + totalCost.toFixed(4));
        // Update the conversation count after saving
        updateConversationCount();
      }
    }
  } catch (error) {
    console.error('Failed to save conversation:', error);
  }
  
  // Update session metrics
  sessionMetrics.totalQueries++;
  sessionMetrics.totalCost += totalCost;
  sessionMetrics.tokenUsage.total += totalTokens;
  
  // Estimate input/output split (70/30 typical)
  sessionMetrics.tokenUsage.input += Math.floor(totalTokens * 0.7);
  sessionMetrics.tokenUsage.output += Math.floor(totalTokens * 0.3);
  
  // Track model usage from stages
  const stageModels = ['claude-3-opus', 'claude-3-sonnet', 'gpt-4-turbo', 'gemini-pro'];
  stageModels.forEach((model, index) => {
    if (!sessionMetrics.modelUsage[model]) {
      sessionMetrics.modelUsage[model] = 0;
    }
    if (!sessionMetrics.costByModel[model]) {
      sessionMetrics.costByModel[model] = 0;
    }
    
    // Distribute cost across models used
    const modelCost = totalCost / 4; // Assuming 4 stages
    sessionMetrics.modelUsage[model]++;
    sessionMetrics.costByModel[model] += modelCost;
  });
  
  // Add to recent activity
  sessionMetrics.recentActivity.unshift({
    timestamp,
    model: 'consensus-pipeline',
    cost: totalCost,
    duration: sessionMetrics.avgResponseTime * 1000,
    status: 'completed',
    tokens: totalTokens
  });
  
  // Keep only last 10 activities
  sessionMetrics.recentActivity = sessionMetrics.recentActivity.slice(0, 10);
  
  // Update hourly stats
  const hour = new Date().toISOString().slice(11, 16);
  let hourStat = sessionMetrics.hourlyStats.find(s => s.hour === hour);
  if (!hourStat) {
    hourStat = { hour, queries: 0, cost: 0, avgTime: 0 };
    sessionMetrics.hourlyStats.push(hourStat);
  }
  hourStat.queries++;
  hourStat.cost += totalCost;
  hourStat.avgTime = sessionMetrics.avgResponseTime;
  
  // Save to localStorage
  localStorage.setItem('consensusMetrics', JSON.stringify(sessionMetrics));
  
  // Log analytics update
  addLogEntry(`📊 Analytics updated: Query #${sessionMetrics.totalQueries}, Total Cost: $${sessionMetrics.totalCost.toFixed(4)}`, 'info');
};

// Track stage completion for analytics
const trackStageCompletion = (stage: string, tokens: number, cost: number) => {
  const stageMetric: StageMetrics = {
    stage,
    model: getModelForStage(stage),
    tokens,
    cost,
    timestamp: new Date().toISOString(),
    duration: 0 // Would need to track start/end times for accurate duration
  };
  
  sessionMetrics.stageMetrics.push(stageMetric);
  
  // Keep only last 100 stage metrics
  if (sessionMetrics.stageMetrics.length > 100) {
    sessionMetrics.stageMetrics = sessionMetrics.stageMetrics.slice(-100);
  }
};

// Helper to get model for stage (from profile)
const getModelForStage = (stage: string): string => {
  // Read from the displayed model elements
  const modelElement = document.getElementById(`${stage}-model`);
  if (modelElement && modelElement.textContent) {
    return modelElement.textContent;
  }
  
  // Default models if not found
  const defaults: { [key: string]: string } = {
    'generator': 'claude-3-opus',
    'refiner': 'claude-3-sonnet',
    'validator': 'gpt-4-turbo',
    'curator': 'gemini-pro'
  };
  
  return defaults[stage.toLowerCase()] || 'unknown';
};

// Initialize on load
loadSessionMetrics();


// Set up Analytics button click handler after functions are defined
setTimeout(() => {
    document.getElementById('analytics-btn')?.addEventListener('click', () => {
        addLogEntry('📊 Opening Analytics Dashboard', 'info');
        showAnalyticsPanel();
    });
    
    // Add click handler for Day 0 Validation tab
    const day0Tab = document.querySelector('[data-tab="day0"]');
    if (day0Tab) {
        day0Tab.addEventListener('click', (e) => {
            if (!(e.target as HTMLElement).classList.contains('tab-close')) {
                // Switch to Day 0 tab
                document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
                day0Tab.classList.add('active');
                
                // Hide analytics panel
                const analyticsPanel = document.getElementById('analytics-panel');
                if (analyticsPanel) {
                    analyticsPanel.style.display = 'none';
                }
                
                // Show welcome content
                const welcomeContent = document.getElementById('welcome-content');
                if (welcomeContent) {
                    welcomeContent.style.display = 'block';
                }
            }
        });
    }
    
    // Don't initialize Git UI here - it will be created when Source Control tab is clicked
    // This ensures the welcome screen shows properly when no folder is open
    
    // Initialize enhanced Status Bar with Git integration
    // TEMPORARILY DISABLED: StatusBar class overwrites the user/plan/conversations center section
    // const statusBar = document.querySelector('.status-bar');
    // if (statusBar) {
    //     (window as any).statusBar = new StatusBar(statusBar as HTMLElement);
    // }
    
    // Initialize Editor Tabs immediately on startup (not just when explorer is clicked)
    if (!window.editorTabs) {
        const editorArea = document.getElementById('editor-area');
        if (editorArea) {
            window.editorTabs = new EditorTabs(editorArea);
            console.log('✅ Editor tabs initialized on startup');
        }
    }
    
    // Initialize Isolated Terminal Panel
    const isolatedTerminalPanelElement = document.getElementById('isolated-terminal-panel');
    if (isolatedTerminalPanelElement) {
        (window as any).isolatedTerminal = ttydTerminalPanel.initialize(isolatedTerminalPanelElement);
        console.log('✅ TTYD Terminal Panel initialized');
        
        // Listen for AI tool launch events from main process
        if (window.electronAPI.onLaunchAIToolTerminal) {
            console.log('✅ [Renderer] Setting up onLaunchAIToolTerminal listener');
            window.electronAPI.onLaunchAIToolTerminal((data: {
                toolId: string;
                toolName: string;
                command: string;
                cwd: string;
                env?: Record<string, string>;  // Optional environment variables
            }) => {
                console.log('📦 [Renderer] Received launch-ai-tool-terminal event:', data);
                console.log('📦 [Renderer] Tool ID:', data.toolId);
                console.log('📦 [Renderer] Command:', data.command);
                console.log('📦 [Renderer] CWD:', data.cwd);
                console.log('📦 [Renderer] Has env vars:', !!data.env);
                
                // Get the TTYDTerminalPanel instance and create a terminal
                const terminal = (window as any).isolatedTerminal;
                console.log('📦 [Renderer] Terminal panel instance exists:', !!terminal);
                
                if (terminal) {
                    console.log('📦 [Renderer] Current opened folder:', window.currentOpenedFolder);
                    console.log('📦 [Renderer] Calling createTerminalTab with:', {
                        toolId: data.toolId,
                        command: data.command,
                        env: data.env
                    });
                    
                    // Note: The global folder context is already updated by the main process
                    // via the 'menu-open-folder' event before this terminal launch event
                    // Call createTerminalTab with correct parameters: toolId, command, env
                    // The cwd is already set globally via window.currentOpenedFolder
                    terminal.createTerminalTab(data.toolId, data.command, data.env);
                } else {
                    console.error('❌ [Renderer] Terminal panel not initialized! isolatedTerminal is null');
                }
            });
        } else {
            console.warn('⚠️ [Renderer] onLaunchAIToolTerminal not available in electronAPI');
        }
        
        // REMOVED resize handler for isolated terminal panel - now using auto-flex layout
        // This prevents the terminal resize issues when dragging
        
        // Get elements for use in both handlers
        const centerArea = document.getElementById('center-area');
        const isolatedTerminalPanel = document.querySelector('.isolated-terminal-panel') as HTMLElement;
        
        // Isolated terminal panel collapse/expand (exactly like consensus panel)
        const toggleIsolatedTerminal = document.getElementById('toggle-isolated-terminal');
        
        if (toggleIsolatedTerminal && isolatedTerminalPanel) {
            toggleIsolatedTerminal.addEventListener('click', () => {
                const isCollapsed = isolatedTerminalPanel.classList.contains('collapsed');
                if (isCollapsed) {
                    // Expand TTYD panel
                    isolatedTerminalPanel.classList.remove('collapsed');
                    isolatedTerminalPanel.style.width = ''; // Let CSS handle width via flex
                    toggleIsolatedTerminal.textContent = '−';
                    toggleIsolatedTerminal.title = 'Collapse Terminal Panel';
                    
                    // Check if center area is collapsed, if so, expand TTYD to fill
                    if (centerArea && centerArea.classList.contains('collapsed')) {
                        isolatedTerminalPanel.classList.add('expanded');
                    }
                } else {
                    // Collapse TTYD panel
                    isolatedTerminalPanel.classList.add('collapsed');
                    isolatedTerminalPanel.classList.remove('expanded');
                    isolatedTerminalPanel.style.width = ''; // Let CSS handle width
                    toggleIsolatedTerminal.textContent = '+';
                    toggleIsolatedTerminal.title = 'Expand Terminal Panel';
                }
            });
        }
        
        // Center area collapse/expand (exactly like consensus panel)
        const toggleCenterArea = document.getElementById('toggle-center-area');
        
        if (toggleCenterArea && centerArea) {
            toggleCenterArea.addEventListener('click', () => {
                const isCollapsed = centerArea.classList.contains('collapsed');
                if (isCollapsed) {
                    // Expand center area
                    centerArea.classList.remove('collapsed');
                    toggleCenterArea.textContent = '−';
                    toggleCenterArea.title = 'Collapse Editor';
                    
                    // Remove expanded from TTYD panel
                    if (isolatedTerminalPanel) {
                        isolatedTerminalPanel.classList.remove('expanded');
                    }
                } else {
                    // Collapse center area
                    centerArea.classList.add('collapsed');
                    toggleCenterArea.textContent = '+';
                    toggleCenterArea.title = 'Expand Editor';
                    
                    // Add expanded to TTYD panel if it's not collapsed
                    if (isolatedTerminalPanel && !isolatedTerminalPanel.classList.contains('collapsed')) {
                        isolatedTerminalPanel.classList.add('expanded');
                    }
                }
            });
        }
        
        // REMOVED center area resize handler - now using auto-flex layout
        // Panels will automatically adjust to fill available space
    }
    
    // Listen for menu events from main process
    setupMenuEventListeners();
    
    // File Explorer and Editor Tabs are already initialized in showSidebarPanel
    // Just set up Git UI file click handler if not already done
    if (!document.querySelector('[data-git-handler]')) {
        const gitHandler = (e: Event) => {
            const target = e.target as HTMLElement;
            const fileNode = target.closest('.git-file');
            if (fileNode) {
                const filePath = fileNode.getAttribute('data-file');
                if (filePath && window.editorTabs) {
                    window.editorTabs.openFile(filePath);
                }
            }
        };
        document.addEventListener('click', gitHandler);
        document.body.setAttribute('data-git-handler', 'true');
    }
    
    // Initialize Neural Consciousness in the right panel
    const neuralContainer = document.getElementById('neural-consciousness-container');
    if (neuralContainer && ENABLE_NEURAL_CONSCIOUSNESS) {
        neuralConsciousness = new NeuralConsciousness();
        neuralContainer.appendChild(neuralConsciousness.getContainer());
        neuralConsciousness.enable(true);
        neuralConsciousness.animate();
        neuralConsciousness.startIdleAnimation();
        (window as any).neuralConsciousness = neuralConsciousness;
        console.log('✅ Neural Consciousness initialized in right panel');
    }
    
    // Set up activity bar toggle functionality for all buttons
    const activityButtons = document.querySelectorAll('.activity-btn');
    const editorArea = document.getElementById('editor-area');
    const analyticsPanel = document.getElementById('analytics-panel');
    let currentView: string | null = null;
    let panels: { [key: string]: HTMLElement } = {};
    
    // Create panels for each view
    const createPanel = (id: string, title: string, content: HTMLElement | string) => {
        const panel = document.createElement('div');
        panel.id = `${id}-panel`;
        panel.className = 'content-panel';
        console.log('Creating panel:', id, panel);
        panel.innerHTML = `
            <div class="panel-header">
                <span>${title}</span>
                <button class="panel-close" data-panel="${id}">×</button>
            </div>
            <div class="panel-body" id="${id}-content">
                ${typeof content === 'string' ? content : ''}
            </div>
        `;
        if (typeof content !== 'string') {
            panel.querySelector('.panel-body')?.appendChild(content);
        }
        panel.style.display = 'none';
        return panel;
    };
    
    // Initialize panels
    if (editorArea) {
        // File Explorer
        const explorerContent = document.createElement('div');
        explorerContent.id = 'file-explorer-container';
        panels.explorer = createPanel('explorer', 'EXPLORER', explorerContent);
        editorArea.appendChild(panels.explorer);
        
        // Source Control
        const gitContent = document.createElement('div');
        gitContent.id = 'git-ui-container';
        panels.git = createPanel('git', 'SOURCE CONTROL', gitContent);
        editorArea.appendChild(panels.git);
        
        // Settings - Create without panel header since SettingsModal has its own
        panels.settings = document.createElement('div');
        panels.settings.id = 'settings-panel';
        panels.settings.className = 'content-panel';
        panels.settings.innerHTML = '<div id="settings-container"></div>';
        panels.settings.style.display = 'none';
        editorArea.appendChild(panels.settings);
        
        // CLI Tools - Create without panel header for independent management
        panels['cli-tools'] = document.createElement('div');
        panels['cli-tools'].id = 'cli-tools-panel';
        panels['cli-tools'].className = 'content-panel';
        panels['cli-tools'].innerHTML = '<div id="cli-tools-container"></div>';
        panels['cli-tools'].style.display = 'none';
        editorArea.appendChild(panels['cli-tools']);
        
        // Memory - Create without panel header since Memory Dashboard has its own
        panels.memory = document.createElement('div');
        panels.memory.id = 'memory-panel';
        panels.memory.className = 'content-panel';
        panels.memory.innerHTML = '<div id="memory-container"></div>';
        panels.memory.style.display = 'none';
        editorArea.appendChild(panels.memory);
        
        // Analytics panel already exists, just track it
        if (analyticsPanel) {
            panels.analytics = analyticsPanel;
        }
    }
    
    activityButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const view = btn.getAttribute('data-view');
            if (!view) return;
            
            // Handle sidebar views (Explorer, Git) separately from center panels
            if (view === 'explorer' || view === 'git') {
                // For Explorer and Git, toggle the sidebar panel only
                if (currentView === view) {
                    // Already active, toggle off
                    toggleSidebarPanel(view as 'explorer' | 'git');
                    btn.classList.remove('active');
                    currentView = null;
                } else {
                    // Not active, toggle on
                    // Remove active from all buttons first
                    activityButtons.forEach(b => b.classList.remove('active'));
                    // Hide all center panels (don't interfere with sidebar)
                    Object.values(panels).forEach(p => {
                        if (p) p.style.display = 'none';
                    });
                    
                    toggleSidebarPanel(view as 'explorer' | 'git');
                    btn.classList.add('active');
                    currentView = view;
                }
                return; // Exit early for sidebar panels
            }
            
            // Handle center panels (Settings, Analytics, Memory)
            if (currentView === view) {
                const panel = panels[view];
                if (panel) {
                    if (panel.style.display === 'none') {
                        panel.style.display = 'block';
                        btn.classList.add('active');
                    } else {
                        panel.style.display = 'none';
                        btn.classList.remove('active');
                        currentView = null;
                        return;
                    }
                }
            } else {
                // Hide all center panels
                Object.values(panels).forEach(p => {
                    if (p) p.style.display = 'none';
                });
                
                // Remove active from all buttons
                activityButtons.forEach(b => b.classList.remove('active'));
                
                // Show selected center panel
                const panel = panels[view];
                if (panel) {
                    console.log('Showing center panel for view:', view, panel);
                    panel.style.display = 'block';
                    btn.classList.add('active');
                    currentView = view;
                    
                    // Handle special panel initialization
                    if (view === 'settings') {
                        console.log('Rendering settings panel...');
                        const container = document.getElementById('settings-container');
                        if (container && settingsModal) {
                            // Check if container is empty (not yet rendered)
                            if (container.innerHTML.trim() === '') {
                                console.log('Rendering settings in container:', container);
                                settingsModal.renderInContainer(container);
                            } else {
                                console.log('Settings already rendered in container');
                            }
                        } else if (!settingsModal) {
                            console.error('Settings modal not initialized');
                        } else {
                            console.error('Settings container not found');
                        }
                    } else if (view === 'analytics' && analyticsPanel) {
                        showAnalyticsPanel();
                    } else if (view === 'cli-tools') {
                        console.log('Opening CLI Tools panel...');
                        renderCliToolsPanel();
                    } else if (view === 'memory') {
                        console.log('Opening Memory Dashboard...');
                        openMemoryDashboard();
                    }
                }
            }
        });
    });
    
    // Close buttons for panels (inside same scope as panels variable)
    document.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        if (target.classList.contains('panel-close')) {
            const panelId = target.getAttribute('data-panel');
            if (panelId && panels[panelId]) {
                panels[panelId].style.display = 'none';
                // Remove active state from corresponding button
                const btn = document.querySelector(`.activity-btn[data-view="${panelId}"]`);
                if (btn) btn.classList.remove('active');
                if (currentView === panelId) currentView = null;
            }
        }
    });
    
    // Terminal controls
    const closeTerminal = document.getElementById('close-terminal');
    const terminalSectionElement = document.getElementById('terminal-section');
    
    if (closeTerminal && terminalSectionElement) {
        closeTerminal.addEventListener('click', () => {
            terminalSectionElement.style.display = 'none';
        });
    }
    
    // Add resize functionality
    setupResizeHandles();
    
    // Add click handlers for AI CLI quick launch buttons - use the same launchCliTool function as Launch buttons
    const cliQuickLaunchButtons = document.querySelectorAll('.cli-quick-launch');
    cliQuickLaunchButtons.forEach(async (btn) => {
        const toolId = btn.getAttribute('data-tool');
        if (!toolId) return;
        
        // Check installation status and add visual indicator
        const electronAPI = window.electronAPI as any;
        const toolStatus = await electronAPI.detectCliTool(toolId);
        
        if (!toolStatus.installed) {
            // Add a small indicator for uninstalled tools
            const indicator = document.createElement('span');
            indicator.className = 'cli-tool-status-indicator';
            indicator.innerHTML = '⬇';  // Download arrow icon
            indicator.title = 'Not installed - Click to install';
            btn.appendChild(indicator);
            
            // Add visual styling for uninstalled tools
            btn.classList.add('not-installed');
        }
        
        btn.addEventListener('click', async () => {
            // Use the exact same launchCliTool function that the Launch buttons use
            console.log(`[Sidebar] Launching ${toolId} via quick launch icon...`);
            await (window as any).launchCliTool(toolId);
        });
    });
    
    // Menu events are handled via window.addEventListener for messages from main process
    // This would need to be set up in preload script if we want menu events
    
}, 200);

// Resize functionality for panels
function setupResizeHandles() {
    // Terminal vertical resize
    const terminalResize = document.getElementById('terminal-resize');
    const terminalSection = document.getElementById('terminal-section');
    
    if (terminalResize && terminalSection) {
        let isResizing = false;
        let startY = 0;
        let startHeight = 0;
        
        terminalResize.addEventListener('mousedown', (e) => {
            isResizing = true;
            startY = e.clientY;
            startHeight = parseInt(window.getComputedStyle(terminalSection).height, 10);
            document.body.style.cursor = 'ns-resize';
            e.preventDefault();
        });
        
        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;
            
            const deltaY = startY - e.clientY;
            const newHeight = Math.min(Math.max(startHeight + deltaY, 100), 600);
            terminalSection.style.height = newHeight + 'px';
        });
        
        document.addEventListener('mouseup', () => {
            isResizing = false;
            document.body.style.cursor = '';
        });
    }
    
    // Consensus panel horizontal resize
    const consensusResize = document.getElementById('consensus-resize');
    const consensusPanel = document.getElementById('consensus-chat');
    
    if (consensusResize && consensusPanel) {
        let isResizing = false;
        let startX = 0;
        let startWidth = 0;
        
        consensusResize.addEventListener('mousedown', (e) => {
            isResizing = true;
            startX = e.clientX;
            startWidth = parseInt(window.getComputedStyle(consensusPanel).width, 10);
            document.body.style.cursor = 'ew-resize';
            e.preventDefault();
        });
        
        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;
            
            const deltaX = startX - e.clientX;
            const newWidth = Math.min(Math.max(startWidth + deltaX, 300), 800);
            
            // Ensure center area maintains minimum width of 400px
            const centerArea = document.getElementById('center-area');
            if (centerArea) {
                const windowWidth = window.innerWidth;
                const sidebarWidth = document.getElementById('left-sidebar')?.offsetWidth || 0;
                const terminalWidth = document.getElementById('isolated-terminal-panel')?.offsetWidth || 0;
                const remainingWidth = windowWidth - sidebarWidth - terminalWidth - newWidth;
                
                // Only apply the new width if center area would have at least 400px
                if (remainingWidth >= 400) {
                    consensusPanel.style.width = newWidth + 'px';
                }
            } else {
                consensusPanel.style.width = newWidth + 'px';
            }
        });
        
        document.addEventListener('mouseup', () => {
            isResizing = false;
            document.body.style.cursor = '';
        });
    }
    
    // Progress section collapse/expand
    const toggleProgress = document.getElementById('toggle-progress');
    const progressContent = document.getElementById('progress-content');
    
    if (toggleProgress && progressContent) {
        toggleProgress.addEventListener('click', () => {
            const isCollapsed = progressContent.style.display === 'none';
            progressContent.style.display = isCollapsed ? 'block' : 'none';
            toggleProgress.textContent = isCollapsed ? '−' : '+';
        });
    }
    
    // Terminal collapse/expand
    const toggleTerminal = document.getElementById('toggle-terminal');
    const terminalContent = document.getElementById('terminal-content');
    
    if (toggleTerminal && terminalContent && terminalSection) {
        toggleTerminal.addEventListener('click', () => {
            const isCollapsed = terminalContent.style.display === 'none';
            terminalContent.style.display = isCollapsed ? 'block' : 'none';
            terminalSection.style.height = isCollapsed ? '200px' : '35px';
            toggleTerminal.textContent = isCollapsed ? '−' : '+';
        });
    }
    
    // Consensus panel collapse/expand
    const toggleConsensusPanel = document.getElementById('toggle-consensus-panel');
    
    if (toggleConsensusPanel && consensusPanel) {
        toggleConsensusPanel.addEventListener('click', () => {
            const isCollapsed = consensusPanel.classList.contains('collapsed');
            if (isCollapsed) {
                consensusPanel.classList.remove('collapsed');
                consensusPanel.style.width = '400px';
                toggleConsensusPanel.textContent = '−';
                toggleConsensusPanel.title = 'Collapse Panel';
            } else {
                consensusPanel.classList.add('collapsed');
                consensusPanel.style.width = '40px';
                toggleConsensusPanel.textContent = '+';
                toggleConsensusPanel.title = 'Expand Panel';
            }
        });
    }
}

// Menu event handlers for opening files and folders
// These are triggered from the main process when File menu items are clicked

// Function to handle opening a folder
async function handleOpenFolder(folderPath: string) {
    try {
        console.log('[handleOpenFolder] Opening folder:', folderPath);
        console.log('[handleOpenFolder] Folder path type:', typeof folderPath);
        console.log('[handleOpenFolder] Folder path value:', JSON.stringify(folderPath));
        console.log('[handleOpenFolder] Previous folder:', currentOpenedFolder);
        
        // Update the current opened folder state
        currentOpenedFolder = folderPath;
        (window as any).currentOpenedFolder = currentOpenedFolder;
        console.log('[handleOpenFolder] Set currentOpenedFolder to:', currentOpenedFolder);
        
        // Update window title with folder name
        const folderName = folderPath.split('/').pop() || folderPath;
        document.title = `Hive Consensus - ${folderName}`;
        
        // Initialize Git manager with the new folder
        if (window.gitAPI) {
            await window.gitAPI.setFolder(folderPath);
        }
        
        // Update Git branch display in status bar
        updateGitStatusBar();
        
        // Refresh the file explorer with the new folder
        const explorerContainer = document.getElementById('explorer-content');
        if (explorerContainer) {
            // Check if explorer panel is currently visible
            const explorerPanel = document.getElementById('explorer-sidebar');
            const isExplorerVisible = explorerPanel && explorerPanel.style.display !== 'none';
            
            if (isExplorerVisible) {
                // Explorer is visible, update it immediately
                console.log('[handleOpenFolder] Explorer is visible, updating now');
                explorerContainer.innerHTML = '';
                window.fileExplorer = new VSCodeExplorerExact(explorerContainer);
                await window.fileExplorer.initialize(folderPath);
                
                // Reconnect file selection handler for the editor
                window.fileExplorer.onFileSelect((filePath: string) => {
                    console.log('File selected:', filePath);
                    if (window.editorTabs) {
                        // Wrap in try-catch to prevent errors from bubbling to webpack
                        try {
                            window.editorTabs.openFile(filePath).catch((err: any) => {
                                console.error('Error opening file:', err);
                            });
                        } catch (err) {
                            console.error('Error calling openFile:', err);
                        }
                    } else {
                        console.error('editorTabs not found');
                    }
                });
            } else {
                // Explorer is not visible, just clear the existing instance so it gets recreated with the new folder when activated
                console.log('[handleOpenFolder] Explorer is not visible, clearing instance for later recreation');
                if (window.fileExplorer) {
                    window.fileExplorer = null;
                    explorerContainer.innerHTML = '';
                }
            }
        }
        
        // Update Git manager with the new folder
        if (window.gitAPI) {
            await window.gitAPI.setFolder(folderPath);
            
            // Also refresh the Source Control view if it exists
            const gitContainer = document.getElementById('git-content');
            console.log('[Menu] Git container found:', !!gitContainer);
            if (gitContainer) {
                console.log('[Menu] Clearing and recreating SCM view...');
                // Clear existing Git UI and recreate it with the new folder
                window.gitUI = null;
                window.scmView = null;
                gitContainer.innerHTML = '';
                
                // Create new SCM view after a short delay to ensure Git status is ready
                setTimeout(async () => {
                    console.log('[Menu] Creating new VSCodeSCMView...');
                    window.gitUI = new VSCodeSCMView(gitContainer);
                    window.scmView = window.gitUI;
                    console.log('[Menu] Source Control view refreshed for folder:', folderPath);
                    
                    // Force a refresh to ensure it loads
                    if (window.scmView && window.scmView.refresh) {
                        console.log('[Menu] Forcing SCM refresh...');
                        await window.scmView.refresh();
                    }
                    
                    // Give extra time for Git graph to initialize
                    // The SCM view creates the graph after a delay
                    setTimeout(() => {
                        console.log('[Menu] Checking if Git graph needs refresh...');
                        if ((window as any).gitGraph && (window as any).gitGraph.refresh) {
                            console.log('[Menu] Refreshing Git graph...');
                            (window as any).gitGraph.refresh();
                        }
                    }, 1500);
                }, 500);
            } else {
                console.log('[Menu] Git container not found! SCM view may not be visible.');
            }
        }
        
        // Update status bar with folder info
        if ((window as any).statusBar) {
            (window as any).statusBar.setWorkspaceInfo({ 
                folder: folderName,
                path: folderPath
            });
            
            // Check if this folder is a Git repository and update Git info
            try {
                const gitStatus = await (window as any).gitAPI.getStatus();
                if (gitStatus && gitStatus.branch) {
                    (window as any).statusBar.setGitInfo({
                        branch: gitStatus.branch,
                        ahead: gitStatus.ahead || 0,
                        behind: gitStatus.behind || 0
                    });
                }
            } catch (error) {
                console.log('[Menu] No Git repo found in opened folder');
            }
        }
        
        addLogEntry(`📁 Opened folder: ${folderName}`, 'success');
        
    } catch (error) {
        console.error('[Menu] Failed to open folder:', error);
        addLogEntry('❌ Failed to open folder', 'error');
    }
}

// Function to handle opening a single file
async function handleOpenFile(filePath: string) {
    try {
        console.log('[Menu] Opening file:', filePath);
        
        // If no folder is currently opened, open the parent folder
        if (!currentOpenedFolder) {
            const parentFolder = filePath.substring(0, filePath.lastIndexOf('/'));
            await handleOpenFolder(parentFolder);
        }
        
        // Open the file in the editor
        if (window.editorTabs) {
            const content = await (window as any).fileAPI.readFile(filePath);
            window.editorTabs.openFile(filePath, content);
        }
        
        addLogEntry(`📄 Opened file: ${filePath.split('/').pop()}`, 'success');
        
    } catch (error) {
        console.error('[Menu] Failed to open file:', error);
        addLogEntry('❌ Failed to open file', 'error');
    }
}

// Listen for menu events from the main process
// These events are sent from the main process when File menu items are clicked
if (typeof window !== 'undefined' && (window as any).electronAPI) {
    // Listen for open folder event
    (window as any).electronAPI.onMenuOpenFolder((folderPath: string) => {
        handleOpenFolder(folderPath);
    });
    
    // Listen for reset state event (Cmd+R)
    (window as any).electronAPI.onMenuResetState(async () => {
        console.log('[Menu] Reset state requested before reload');
        
        // Clear Git folder state
        if (window.gitAPI) {
            await window.gitAPI.setFolder('');
        }
        
        // Clear current folder
        currentOpenedFolder = null;
        (window as any).currentOpenedFolder = currentOpenedFolder;
        
        // Reset localStorage if needed
        localStorage.removeItem('lastOpenedFolder');
        
        // The reload will happen after this
    });
    
    // Listen for close folder event
    (window as any).electronAPI.onMenuCloseFolder(async () => {
        console.log('[Menu] Close folder requested');
        // Reset the current opened folder
        currentOpenedFolder = null;
        (window as any).currentOpenedFolder = currentOpenedFolder;
        
        // Reset window title
        document.title = 'Hive Consensus';
        
        // Hide Git branch display in status bar
        updateGitStatusBar();
        
        // Clear and reinitialize the Explorer to show welcome message
        const explorerContainer = document.getElementById('explorer-content');
        if (explorerContainer) {
            explorerContainer.innerHTML = '';
            window.fileExplorer = new VSCodeExplorerExact(explorerContainer);
            await window.fileExplorer.initialize(); // Initialize without a folder - shows welcome
        }
        
        // Reset Git manager to no folder state
        if (window.gitAPI) {
            // Tell the backend to reset git manager without a folder
            await window.gitAPI.setFolder('');
        }
        
        // Reinitialize SCM view to show welcome message
        const scmContainer = document.getElementById('source-control-content');
        if (scmContainer) {
            // Destroy old view and create new one
            if (window.scmView) {
                window.scmView.destroy();
            }
            // Create new SCM view which will show welcome since no folder is open
            window.gitUI = new VSCodeSCMView(scmContainer);
            window.scmView = window.gitUI;
        }
        
        // Close all editor tabs
        if (window.editorTabs) {
            await window.editorTabs.closeAllTabs();
        }
        
        addLogEntry('📁 Closed folder', 'info');
    });
    
    // Listen for open file event
    (window as any).electronAPI.onMenuOpenFile((filePath: string) => {
        handleOpenFile(filePath);
    });
    
    // Listen for other menu events
    (window as any).electronAPI.onMenuNewFile(() => {
        console.log('[Menu] New file requested');
        // Create a new untitled file in the editor
        if (window.editorTabs) {
            window.editorTabs.openFile('untitled.txt', '');
        }
    });
    
    (window as any).electronAPI.onMenuSave(() => {
        console.log('[Menu] Save requested');
        // Save the current file
        if (window.editorTabs) {
            window.editorTabs.saveCurrentFile();
        }
    });
    
    (window as any).electronAPI.onMenuCloseTab(() => {
        console.log('[Menu] Close tab requested');
        // Close the current tab
        if (window.editorTabs) {
            window.editorTabs.closeCurrentTab();
        }
    });
    
    console.log('[Menu] Menu event listeners registered');
}

// Initialize CLI tool detector on startup
async function initializeCliToolDetector() {
    console.log('[CliToolDetector] Checking for installed AI CLI tools...');
    try {
        await cliToolDetector.checkAllTools();
        const tools = cliToolDetector.getAllTools();
        console.log('[CliToolDetector] Tool detection complete:', tools);
    } catch (error) {
        console.error('[CliToolDetector] Error during tool detection:', error);
    }
}

// Call on startup
initializeCliToolDetector();

// Define global functions for opening folder and cloning repository
window.openFolder = async (folderPath?: string) => {
    try {
        // If a folder path is provided, open it directly
        if (folderPath) {
            console.log('[OpenFolder] Opening provided folder:', folderPath);
            handleOpenFolder(folderPath);
            return;
        }
        
        // Otherwise, show the folder selection dialog
        console.log('[OpenFolder] Starting folder selection...');
        const result = await window.electronAPI.showOpenDialog({
            properties: ['openDirectory']
        });
        
        console.log('[OpenFolder] Dialog result:', result);
        
        if (!result.canceled && result.filePaths.length > 0) {
            const selectedFolder = result.filePaths[0];
            console.log('[OpenFolder] Selected folder:', selectedFolder);
            // Use the same handleOpenFolder function that File > Open Folder uses
            handleOpenFolder(selectedFolder);
        } else {
            console.log('[OpenFolder] Folder selection was canceled');
        }
    } catch (error) {
        console.error('Failed to open folder:', error);
    }
};

window.cloneRepository = async () => {
    try {
        // For now, show a prompt for the repository URL
        const repoUrl = await window.electronAPI.showInputDialog('Clone Repository', 'Enter repository URL:');
        
        if (repoUrl) {
            // Select destination folder
            const result = await window.electronAPI.showOpenDialog({
                properties: ['openDirectory', 'createDirectory'],
                title: 'Select destination folder for clone'
            });
            
            if (!result.canceled && result.filePaths.length > 0) {
                const destPath = result.filePaths[0];
                // TODO: Implement actual git clone functionality
                console.log('Would clone', repoUrl, 'to', destPath);
                alert(`Clone functionality coming soon!\nWould clone: ${repoUrl}\nTo: ${destPath}`);
            }
        }
    } catch (error) {
        console.error('Failed to clone repository:', error);
    }
};

// Testing Git modification indicator


// ========== HELP MODAL FUNCTIONS ==========
function showGettingStartedModal() {
    const modal = document.createElement('div');
    modal.className = 'help-modal-overlay';
    modal.innerHTML = `
        <div class="help-modal">
            <div class="help-modal-header">
                <h2>Getting Started with Hive Consensus</h2>
                <button class="help-modal-close">&times;</button>
            </div>
            <div class="help-modal-content">
                <section>
                    <h3>Welcome to Hive Consensus IDE</h3>
                    <p>Hive Consensus is an advanced AI-powered development environment that enhances your coding experience with multi-stage consensus processing and integrated AI tools.</p>
                </section>
                
                <section>
                    <h3>Key Features</h3>
                    <ul>
                        <li><strong>Multi-Stage Consensus:</strong> Leverage our 4-stage AI consensus engine (Generator → Refiner → Validator → Curator) for superior code quality</li>
                        <li><strong>AI CLI Tools:</strong> Integrated support for Claude Code, Gemini CLI, Qwen Code, and more</li>
                        <li><strong>Smart Memory Access:</strong> Your AI tools can directly query your development history and knowledge base</li>
                        <li><strong>Git Integration:</strong> Full version control with intelligent commit suggestions</li>
                        <li><strong>Real-time Analytics:</strong> Track your productivity and AI usage patterns</li>
                    </ul>
                </section>
                
                <section>
                    <h3>Quick Start</h3>
                    <ol>
                        <li><strong>Open a Project:</strong> Click "Open Folder" or use Cmd/Ctrl+O</li>
                        <li><strong>Launch AI Tools:</strong> Click any AI tool icon in the sidebar to start coding with AI assistance</li>
                        <li><strong>Use Consensus:</strong> Type in the consensus chat panel to get refined AI responses</li>
                        <li><strong>Access Memory:</strong> Your AI tools can query past solutions using simple SQL views</li>
                    </ol>
                </section>
                
                <section>
                    <h3>Keyboard Shortcuts</h3>
                    <ul>
                        <li><code>Cmd/Ctrl+O</code> - Open folder</li>
                        <li><code>Cmd/Ctrl+S</code> - Save file</li>
                        <li><code>Cmd/Ctrl+B</code> - Toggle file explorer</li>
                        <li><code>Cmd/Ctrl+\`</code> - Toggle terminal</li>
                        <li><code>Cmd/Ctrl+Shift+G</code> - Toggle source control</li>
                    </ul>
                </section>
                
                <div class="help-modal-footer">
                    <button class="help-modal-button primary help-modal-ok-btn">Got it!</button>
                </div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    
    // Close on click outside or close button
    const closeBtn = modal.querySelector('.help-modal-close');
    if (closeBtn) {
        closeBtn.addEventListener('click', () => modal.remove());
    }
    
    // OK button handler
    const okBtn = modal.querySelector('.help-modal-ok-btn');
    if (okBtn) {
        okBtn.addEventListener('click', () => modal.remove());
    }
    
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

function showMemoryGuideModal() {
    const modal = document.createElement('div');
    modal.className = 'help-modal-overlay';
    modal.innerHTML = `
        <div class="help-modal">
            <div class="help-modal-header">
                <h2>Smart Memory Access Guide</h2>
                <button class="help-modal-close">&times;</button>
            </div>
            <div class="help-modal-content">
                <section>
                    <h3>AI Tools Can Now Access Your Memory</h3>
                    <p>When you launch any AI CLI tool (Claude Code, Gemini CLI, etc.) from Hive, it automatically gains access to your development history through a local SQLite database symlink.</p>
                </section>
                
                <section>
                    <h3>How It Works</h3>
                    <ol>
                        <li>When you open a project folder, Hive creates a <code>.hive-ai.db</code> symlink in your project root</li>
                        <li>This symlink points to your unified knowledge base at <code>~/.hive/hive-ai.db</code></li>
                        <li>AI tools can query this database to understand your past work and solutions</li>
                    </ol>
                </section>
                
                <section>
                    <h3>Available Views for Querying</h3>
                    <div class="code-block">
                        <h4>Recent Work</h4>
                        <code>SELECT * FROM recent_work LIMIT 10;</code>
                        <p>Shows your most recent assistant responses</p>
                    </div>
                    
                    <div class="code-block">
                        <h4>Solutions</h4>
                        <code>SELECT * FROM solutions WHERE content LIKE '%authentication%';</code>
                        <p>Finds problems you've solved before</p>
                    </div>
                    
                    <div class="code-block">
                        <h4>Patterns</h4>
                        <code>SELECT * FROM patterns;</code>
                        <p>Shows learned patterns and best practices</p>
                    </div>
                    
                    <div class="code-block">
                        <h4>Debugging</h4>
                        <code>SELECT * FROM debugging WHERE content LIKE '%error%';</code>
                        <p>Searches through past debugging sessions</p>
                    </div>
                    
                    <div class="code-block">
                        <h4>Code Examples</h4>
                        <code>SELECT * FROM code_examples LIMIT 5;</code>
                        <p>Retrieves code snippets from your history</p>
                    </div>
                    
                    <div class="code-block">
                        <h4>Full-Text Search</h4>
                        <code>SELECT * FROM messages_fts WHERE content MATCH 'authentication';</code>
                        <p>Fast text search across all messages</p>
                    </div>
                </section>
                
                <section>
                    <h3>Example Usage in AI Tools</h3>
                    <p>When using Claude Code or other AI tools, you can ask questions like:</p>
                    <ul>
                        <li>"Query the local .hive-ai.db to see what we've been working on"</li>
                        <li>"Check our past solutions for authentication problems"</li>
                        <li>"Search the memory database for React patterns we've used"</li>
                        <li>"Find debugging sessions related to TypeScript errors"</li>
                    </ul>
                </section>
                
                <section>
                    <h3>Privacy & Security</h3>
                    <ul>
                        <li>The database is stored locally on your machine</li>
                        <li>Only accessible when you explicitly launch AI tools from Hive</li>
                        <li>Symlink is created per-project and removed when switching projects</li>
                        <li>No data is sent to external servers without your consent</li>
                    </ul>
                </section>
                
                <div class="help-modal-footer">
                    <button class="help-modal-button primary help-modal-ok-btn">Close</button>
                </div>
            </div>
        </div>
    `;
    
    document.body.appendChild(modal);
    
    // Close on click outside or close button
    const closeBtn = modal.querySelector('.help-modal-close');
    if (closeBtn) {
        closeBtn.addEventListener('click', () => modal.remove());
    }
    
    // OK button handler
    const okBtn = modal.querySelector('.help-modal-ok-btn');
    if (okBtn) {
        okBtn.addEventListener('click', () => modal.remove());
    }
    
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

// Add modal styles
const addHelpModalStyles = () => {
    if (!document.getElementById('help-modal-styles')) {
        const style = document.createElement('style');
        style.id = 'help-modal-styles';
        style.textContent = `
            .help-modal-overlay {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.5);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 10000;
            }
            
            .help-modal {
                background: var(--vscode-editor-background, #1e1e1e);
                border: 1px solid var(--vscode-widget-border, #464647);
                border-radius: 4px;
                width: 90%;
                max-width: 800px;
                max-height: 80vh;
                display: flex;
                flex-direction: column;
                box-shadow: 0 4px 16px rgba(0, 0, 0, 0.3);
            }
            
            .help-modal-header {
                padding: 16px 20px;
                border-bottom: 1px solid var(--vscode-widget-border, #464647);
                display: flex;
                justify-content: space-between;
                align-items: center;
            }
            
            .help-modal-header h2 {
                margin: 0;
                font-size: 18px;
                font-weight: 600;
                color: var(--vscode-foreground, #cccccc);
            }
            
            .help-modal-close {
                background: none;
                border: none;
                color: var(--vscode-foreground, #cccccc);
                font-size: 24px;
                cursor: pointer;
                padding: 0;
                width: 30px;
                height: 30px;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            
            .help-modal-close:hover {
                background: var(--vscode-toolbar-hoverBackground, #2a2d2e);
                border-radius: 3px;
            }
            
            .help-modal-content {
                padding: 20px;
                overflow-y: auto;
                flex: 1;
                color: var(--vscode-foreground, #cccccc);
                line-height: 1.6;
            }
            
            .help-modal-content section {
                margin-bottom: 24px;
            }
            
            .help-modal-content h3 {
                margin: 0 0 12px 0;
                font-size: 16px;
                font-weight: 600;
                color: var(--vscode-foreground, #cccccc);
            }
            
            .help-modal-content p {
                margin: 0 0 12px 0;
                color: var(--vscode-descriptionForeground, #969696);
            }
            
            .help-modal-content ul,
            .help-modal-content ol {
                margin: 0 0 12px 0;
                padding-left: 24px;
                color: var(--vscode-descriptionForeground, #969696);
            }
            
            .help-modal-content li {
                margin-bottom: 8px;
            }
            
            .help-modal-content strong {
                color: var(--vscode-foreground, #cccccc);
            }
            
            .help-modal-content code {
                background: var(--vscode-textBlockQuote-background, #222222);
                padding: 2px 6px;
                border-radius: 3px;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 13px;
                color: var(--vscode-textPreformat-foreground, #d7ba7d);
            }
            
            .code-block {
                background: var(--vscode-textBlockQuote-background, #222222);
                border: 1px solid var(--vscode-widget-border, #464647);
                border-radius: 3px;
                padding: 12px;
                margin-bottom: 16px;
            }
            
            .code-block h4 {
                margin: 0 0 8px 0;
                font-size: 14px;
                font-weight: 600;
                color: var(--vscode-foreground, #cccccc);
            }
            
            .code-block code {
                display: block;
                background: none;
                padding: 0;
                margin-bottom: 8px;
            }
            
            .code-block p {
                margin: 0;
                font-size: 12px;
            }
            
            .help-modal-footer {
                padding: 16px 20px;
                border-top: 1px solid var(--vscode-widget-border, #464647);
                display: flex;
                justify-content: flex-end;
            }
            
            .help-modal-button {
                padding: 6px 14px;
                border: 1px solid var(--vscode-button-border, transparent);
                border-radius: 2px;
                cursor: pointer;
                font-size: 13px;
                background: var(--vscode-button-secondaryBackground, #3a3d41);
                color: var(--vscode-button-secondaryForeground, #cccccc);
            }
            
            .help-modal-button.primary {
                background: var(--vscode-button-background, #0e639c);
                color: var(--vscode-button-foreground, #ffffff);
            }
            
            .help-modal-button:hover {
                background: var(--vscode-button-hoverBackground, #1177bb);
            }
        `;
        document.head.appendChild(style);
    }
};

// Initialize help modal styles
addHelpModalStyles();

// ========== FILE MENU EVENT HANDLERS ==========
// Listen for menu events from the main process
(function setupMenuHandlers() {
    // Auto-save toggle
    window.electronAPI.onMenuToggleAutoSave((enabled: boolean) => {
        if (window.editorTabs) {
            window.editorTabs.setAutoSave(enabled, 1000); // 1 second delay
            console.log('[Menu] Auto-save:', enabled ? 'enabled' : 'disabled');
        }
    });
    
    // Save current file
    window.electronAPI.onMenuSave(() => {
        if (window.editorTabs) {
            // Save the active tab
            const activeTab = window.editorTabs.getActiveTab();
            if (activeTab) {
                window.editorTabs.saveActiveTab();
            }
        }
    });
    
    // Save As
    window.electronAPI.onMenuSaveAs(async () => {
        if (window.editorTabs) {
            const activeTab = window.editorTabs.getActiveTab();
            if (activeTab) {
                // TODO: Implement save as dialog
                console.log('[Menu] Save As not yet implemented');
            }
        }
    });
    
    // Open file
    window.electronAPI.onMenuOpenFile((filePath: string) => {
        if (window.editorTabs && filePath) {
            window.editorTabs.openFile(filePath);
        }
    });
    
    // New file
    window.electronAPI.onMenuNewFile(() => {
        if (window.editorTabs) {
            // Create a new untitled file
            window.editorTabs.createNewFile();
        }
    });
    
    // Close tab
    window.electronAPI.onMenuCloseTab(() => {
        if (window.editorTabs) {
            const activeTab = window.editorTabs.getActiveTab();
            if (activeTab) {
                window.editorTabs.closeTab(activeTab.id);
            }
        }
    });
    
    // Close all tabs
    window.electronAPI.onMenuCloseAllTabs(() => {
        if (window.editorTabs) {
            window.editorTabs.closeAllTabs();
        }
    });
    
    // Open folder
    window.electronAPI.onMenuOpenFolder((folderPath: string) => {
        if (folderPath) {
            window.openFolder(folderPath);
        }
    });
    
    // Close folder
    window.electronAPI.onMenuCloseFolder(() => {
        window.closeFolder();
    });
    
    // Getting Started
    if (window.electronAPI.onMenuGettingStarted) {
        window.electronAPI.onMenuGettingStarted(() => {
            console.log('[Menu] Getting Started requested');
            showGettingStartedModal();
        });
    }
    
    // Memory Access Guide
    if (window.electronAPI.onMenuMemoryGuide) {
        window.electronAPI.onMenuMemoryGuide(() => {
            console.log('[Menu] Memory Guide requested');
            showMemoryGuideModal();
        });
    }
    
    // About dialog
    window.electronAPI.onMenuAbout(async () => {
        console.log('[Menu] About dialog requested');
        const version = await window.electronAPI.getVersion();
        alert(`Hive Consensus\nVersion: ${version}\n\nAdvanced AI-powered development environment\nwith Multi-Stage Consensus Processing\n\nCopyright © 2025 HiveTechs`);
    });
    
    console.log('[Menu] Event handlers registered');
})();

// Add consensus reset handlers
if (typeof window !== 'undefined' && (window as any).electronAPI) {
    // Listen for neural consciousness reset
    (window as any).electronAPI.on?.('reset-neural-consciousness', () => {
        console.log('🧠 Resetting Neural Consciousness');
        if ((window as any).neuralConsciousness) {
            (window as any).neuralConsciousness.reset();
        }
    });
    
    // Listen for progress reset  
    (window as any).electronAPI.on?.('reset-all-progress', () => {
        console.log('📊 Resetting all progress bars and stage status');
        // Reset all stage progress bars to 0 and status to ready
        const stages = ['generator', 'refiner', 'validator', 'curator'];
        stages.forEach(stage => {
            // Reset progress bar fill
            const progressBar = document.querySelector(`[data-stage="${stage}"] .stage-progress-fill`);
            if (progressBar) {
                (progressBar as HTMLElement).style.width = '0%';
            }
            
            // Reset stage status more aggressively - try multiple selectors
            const stageElement = document.querySelector(`[data-stage="${stage}"]`);
            if (stageElement) {
                stageElement.setAttribute('data-status', 'ready');
                stageElement.removeAttribute('data-progress');
                
                // Try multiple status element selectors
                const statusSelectors = ['.stage-status', '.status', '.stage-text', '.stage-label'];
                statusSelectors.forEach(selector => {
                    const statusElement = stageElement.querySelector(selector);
                    if (statusElement && statusElement.textContent !== 'ready') {
                        statusElement.textContent = 'ready';
                        console.log(`🔧 Reset ${stage} status via ${selector}`);
                    }
                });
                
                // Force CSS class updates for visual state
                stageElement.classList.remove('running', 'completed');
                stageElement.classList.add('ready');
            }
            
            console.log(`📊 Reset ${stage} progress and status`);
        });
    });
    
    // Listen for chat clearing (hide error messages from interruption)
    (window as any).electronAPI.on?.('clear-chat-window', () => {
        console.log('💬 Clearing chat window');
        // Small delay to ensure abort error appears first, then clear it
        setTimeout(() => {
            const chatContainer = document.querySelector('.chat-container');
            if (chatContainer) {
                chatContainer.innerHTML = '<div class="message bot-message">Welcome to Hive Consensus! Try asking me a question.</div>';
                console.log('✅ Chat window cleared - error messages hidden');
            }
        }, 100); // 100ms delay to catch abort errors
    });
}

// Close the current folder
window.closeFolder = () => {
    // Clear current folder
    window.currentOpenedFolder = null;
    
    // Clear file explorer
    if (window.fileExplorer) {
        const explorerContainer = document.querySelector('.file-explorer-container');
        if (explorerContainer) {
            explorerContainer.innerHTML = '';
        }
        window.fileExplorer = null;
    }
    
    // Clear git status
    if (window.scmView) {
        window.scmView = null;
        const gitContainer = document.querySelector('.scm-view');
        if (gitContainer && gitContainer.parentElement) {
            gitContainer.parentElement.innerHTML = '';
        }
    }
    
    // Close all editor tabs
    if (window.editorTabs) {
        window.editorTabs.closeAllTabs();
    }
    
    console.log('[CloseFolder] Folder closed');
};
