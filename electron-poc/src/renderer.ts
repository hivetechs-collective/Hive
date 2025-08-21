/**
 * Hive Consensus - Exact Dioxus GUI Recreation
 * Layout: Left Sidebar | Center (with bottom Terminal) | Right Consensus Chat
 */

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

// Current opened folder state
let currentOpenedFolder: string | null = null;

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
  console.error('[Global Error Handler] Caught error event');
  console.error('[Global Error Handler] Event type:', event.constructor.name);
  console.error('[Global Error Handler] Error object:', event.error);
  console.error('[Global Error Handler] Error type:', event.error?.constructor?.name);
  console.error('[Global Error Handler] Error message:', event.message);
  console.error('[Global Error Handler] Stack:', event.error?.stack);
  
  // Check if the error itself is an Event
  if (event.error && (event.error instanceof Event || event.error.constructor.name.includes('Event'))) {
    console.error('[Global Error Handler] ERROR IS AN EVENT OBJECT! Preventing propagation');
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
  
  // Check if the error message contains [object Event]
  if (event.message && event.message.includes('[object Event]')) {
    console.error('[Global Error Handler] Error message contains [object Event], preventing');
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
}, true);

// Also catch unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
  console.error('[Unhandled Rejection] Caught rejection');
  console.error('[Unhandled Rejection] Event type:', event.constructor.name);
  console.error('[Unhandled Rejection] Reason:', event.reason);
  console.error('[Unhandled Rejection] Reason type:', event.reason?.constructor?.name);
  
  // Check if the reason is an Event object
  if (event.reason instanceof Event) {
    console.error('[Unhandled Rejection] REASON IS AN EVENT OBJECT! Preventing propagation');
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
  
  // Check for [object Event] string
  const reasonStr = Object.prototype.toString.call(event.reason);
  if (reasonStr.includes('Event') || (event.reason && event.reason.toString && event.reason.toString().includes('[object Event]'))) {
    console.error('[Unhandled Rejection] Reason contains Event, preventing:', reasonStr);
    event.preventDefault();
    event.stopPropagation();
    event.stopImmediatePropagation();
    return false;
  }
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
        
        <!-- Analytics, Settings, Memory Section -->
        <button class="activity-btn" data-view="analytics" aria-label="Analytics">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 13h2v8H3zm4-8h2v16H7zm4-2h2v18h-2zm4 4h2v14h-2zm4-2h2v16h-2z"/>
          </svg>
          <span class="activity-tooltip">Analytics</span>
        </button>
        <button class="activity-btn" data-view="settings" aria-label="Settings">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19.14,12.94c0.04-0.3,0.06-0.61,0.06-0.94c0-0.32-0.02-0.64-0.07-0.94l2.03-1.58c0.18-0.14,0.23-0.41,0.12-0.61 l-1.92-3.32c-0.12-0.22-0.37-0.29-0.59-0.22l-2.39,0.96c-0.5-0.38-1.03-0.7-1.62-0.94L14.4,2.81c-0.04-0.24-0.24-0.41-0.48-0.41 h-3.84c-0.24,0-0.43,0.17-0.47,0.41L9.25,5.35C8.66,5.59,8.12,5.92,7.63,6.29L5.24,5.33c-0.22-0.08-0.47,0-0.59,0.22L2.74,8.87 C2.62,9.08,2.66,9.34,2.86,9.48l2.03,1.58C4.84,11.36,4.8,11.69,4.8,12s0.02,0.64,0.07,0.94l-2.03,1.58 c-0.18,0.14-0.23,0.41-0.12,0.61l1.92,3.32c0.12,0.22,0.37,0.29,0.59,0.22l2.39-0.96c0.5,0.38,1.03,0.7,1.62,0.94l0.36,2.54 c0.05,0.24,0.24,0.41,0.48,0.41h3.84c0.24,0,0.44-0.17,0.47-0.41l0.36-2.54c0.59-0.24,1.13-0.56,1.62-0.94l2.39,0.96 c0.22,0.08,0.47,0,0.59-0.22l1.92-3.32c0.12-0.22,0.07-0.47-0.12-0.61L19.14,12.94z M12,15.6c-1.98,0-3.6-1.62-3.6-3.6 s1.62-3.6,3.6-3.6s3.6,1.62,3.6,3.6S13.98,15.6,12,15.6z"/>
          </svg>
          <span class="activity-tooltip">Settings</span>
        </button>
        <button class="activity-btn" data-view="cli-tools" aria-label="AI CLI Tools">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
            <path d="M22 14H21C21 10.13 17.87 7 14 7H13V5.73C13.6 5.39 14 4.74 14 4C14 2.9 13.11 2 12 2S10 2.9 10 4C10 4.74 10.4 5.39 11 5.73V7H10C6.13 7 3 10.13 3 14H2C1.45 14 1 14.45 1 15V18C1 18.55 1.45 19 2 19H3V20C3 21.1 3.9 22 5 22H19C20.1 22 21 21.1 21 20V19H22C22.55 19 23 18.55 23 18V15C23 14.45 22.55 14 22 14M8.5 13C9.33 13 10 13.67 10 14.5S9.33 16 8.5 16S7 15.33 7 14.5S7.67 13 8.5 13M15.5 13C16.33 13 17 13.67 17 14.5S16.33 16 15.5 16S14 15.33 14 14.5S14.67 13 15.5 13M8 19L10 17H14L16 19H8Z"/>
          </svg>
          <span class="activity-tooltip">AI CLI Tools</span>
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
    <div class="center-area" id="center-area">
      <!-- Editor Area -->
      <div class="editor-area" id="editor-area">
        <!-- Editor tabs and content will be mounted here -->
      </div>
      
      <!-- Analytics Panel (Hidden by default) -->
      <div id="analytics-panel" class="panel-content" style="display: none;">
        <!-- Analytics content will be mounted here -->
      </div>

      <!-- Terminal Section (Bottom, resizable) -->
      <div class="terminal-section" id="terminal-section" style="height: 200px;">
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
        <div class="chat-header">CONSENSUS CHAT</div>
        <div class="chat-content" id="chat-content">
          <div class="chat-message system">
            <div class="message-time">[${new Date().toLocaleTimeString()}]</div>
            <div class="message-content">Hive Consensus ready for queries</div>
          </div>
        </div>
        <div class="chat-input-area">
          <input type="text" id="chat-input" placeholder="Enter your question..." />
          <button id="send-chat" class="send-btn">Send</button>
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
`;

// State management
let currentView = 'consensus';
let isConnected = false;
let isProcessing = false;
let conversationStartTime = 0;
let settingsModal: SettingsModal | null = null;

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
                        // If explorer doesn't exist, create it
                        if (!window.fileExplorer) {
                            console.log('Creating new file explorer for:', currentOpenedFolder);
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
                            // Explorer already exists, but we need to ensure it's showing the correct folder
                            console.log('Explorer exists, reinitializing with:', currentOpenedFolder);
                            window.fileExplorer.initialize(currentOpenedFolder);
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
                                    // Create the Git UI after setting the folder
                                    window.gitUI = new VSCodeSCMView(container);
                                    window.scmView = window.gitUI;
                                }, 300);
                            });
                        } else {
                            // No folder open, create Git UI which will show welcome
                            window.gitUI = new VSCodeSCMView(container);
                            window.scmView = window.gitUI;
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
  resetStageStatus();
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

// Chat input handler - Now uses WebSocket streaming
document.getElementById('send-chat')?.addEventListener('click', async () => {
  const chatInput = document.getElementById('chat-input') as HTMLInputElement;
  const query = chatInput.value.trim();
  
  if (!query || isProcessing) return;
  
  chatInput.value = '';
  addChatMessage(query, false);
  
  // Check WebSocket connection
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
  
  try {
    // Use IPC to communicate with main process, which can make the HTTP request
    const result = await (window as any).backendAPI.runQuickConsensus({
      query: query,
      profile: activeProfile?.name || 'Free Also'
    });
    
    // Update all stages as complete
    ['generator', 'refiner', 'validator', 'curator'].forEach(stage => {
      updateStageStatus(stage, 'completed');
      updateStageProgress(stage, 100);
    });
    
    // Update stats
    totalTokens = result.tokens_used || 1000;
    totalCost = result.cost || 0.01;
    updateConsensusStats();
    
    addLogEntry(`✅ Consensus completed in ${result.duration_ms}ms`, 'success');
    addChatMessage(result.result, true);
    
    // Update usage count
    dailyUsageCount++;
    updateConversationCount();
    
  } catch (error) {
    resetStageStatus();
    console.error('Full error details:', error);
    
    // Check if it's a network error
    if (error instanceof TypeError && error.message === 'Failed to fetch') {
      addLogEntry(`❌ Network error: Cannot connect to backend at http://127.0.0.1:8765`, 'error');
      addLogEntry(`💡 Make sure the backend server is running`, 'warning');
      addChatMessage(`Network Error: Cannot reach the backend server. Please ensure it's running on port 8765.`, true);
    } else {
      addLogEntry(`❌ Consensus failed: ${error}`, 'error');
      addChatMessage(`Error: ${error}`, true);
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

// Initialize WebSocket connection for streaming
async function initializeWebSocket() {
  // Prevent multiple initializations
  if (consensusWebSocket) {
    console.log('WebSocket already initialized');
    return;
  }
  
  // Get dynamic backend port from IPC
  let backendPort = 8765; // default fallback
  try {
    if ((window as any).backendAPI?.getBackendPort) {
      backendPort = await (window as any).backendAPI.getBackendPort();
      console.log('Got dynamic backend port:', backendPort);
      
      // Update the terminal display with actual port
      const backendLine = document.getElementById('backend-server-line');
      if (backendLine) {
        backendLine.textContent = `[${new Date().toLocaleTimeString()}] Backend server: http://localhost:${backendPort}`;
      }
    }
  } catch (error) {
    console.warn('Failed to get dynamic backend port, using default:', error);
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
    
    // Test with a simple WebSocket first
    try {
      const testWS = new WebSocket('ws://localhost:8765/ws-test');
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
(window as any).testWebSocket = () => {
  console.log('Testing WebSocket connection...');
  const ws = new WebSocket('ws://localhost:8765/ws-test');
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
async function renderCliToolsPanel() {
    const container = document.getElementById('cli-tools-container');
    if (container && container.innerHTML.trim() === '') {
        console.log('[CLI Tools] Rendering CLI Tools panel...');
        
        // Show loading state first
        container.innerHTML = `
            <div class="cli-tools-panel" style="padding: 20px; height: 100%; overflow-y: auto; background: var(--vscode-editor-background);">
                <h2 style="margin: 0 0 10px 0; color: #fff;">AI CLI Tools Management</h2>
                <p style="color: #aaa; margin-bottom: 20px;">Install and manage AI-powered coding assistants</p>
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
            
            // Other tools (static for now - will implement detection incrementally)
            gridContainer.appendChild(createStaticToolCard('gemini-cli', 'Gemini CLI', 'Google\'s free AI coding assistant (1000 req/day)', 'FREE', '#28a745', 'https://cloud.google.com/gemini/docs/codeassist/gemini-cli'));
            gridContainer.appendChild(createStaticToolCard('qwen-code', 'Qwen Code', 'Alibaba\'s open-source coding agent', 'OPEN SOURCE', '#6c757d', 'https://github.com/QwenLM/Qwen3-Coder'));
            gridContainer.appendChild(createStaticToolCard('openai-codex', 'OpenAI Codex', 'GPT-powered terminal assistant', null, null, 'https://help.openai.com/en/articles/11096431-openai-codex-cli-getting-started'));
            gridContainer.appendChild(createStaticToolCard('aider', 'Aider', 'Git-integrated inline editing', null, null, 'https://aider.chat/docs/'));
            gridContainer.appendChild(createStaticToolCard('cline', 'Cline', 'Lightweight conversational agent', null, null, 'https://cline.bot/'));
            
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
        console.log('[CLI Tools] Panel already rendered');
    }
}

// Helper function to create dynamic CLI tool cards
function createCliToolCard(toolInfo: CliToolCardInfo): HTMLDivElement {
    const { id, name, description, status, docsUrl, badgeText, badgeColor } = toolInfo;
    const card = document.createElement('div');
    card.className = 'cli-tool-card';
    card.setAttribute('data-tool-id', id);  // Add data attribute for button handlers
    card.style.cssText = 'background: #2d2d30; border: 1px solid #3e3e42; border-radius: 6px; padding: 15px; cursor: pointer; transition: all 0.2s;';
    
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
            <button onclick="refreshCliToolDetails('${id}')" style="flex: 1; padding: 6px; background: #2d7d2d; color: #fff; border: none; border-radius: 3px; font-size: 12px; cursor: pointer;" title="Refresh tool details">Details</button>
            <button onclick="configureCliTool('${id}')" style="flex: 1; padding: 6px; background: #3e3e42; color: #ccc; border: none; border-radius: 3px; font-size: 12px; cursor: pointer;">Configure</button>
            <button onclick="updateCliTool('${id}')" style="flex: 1; padding: 6px; background: #3e3e42; color: #ccc; border: none; border-radius: 3px; font-size: 12px; cursor: pointer;">Update</button>
        `;
    } else {
        buttons = `
            <button onclick="installCliTool('${id}')" style="flex: 1; padding: 6px; background: #007acc; color: #fff; border: none; border-radius: 3px; font-size: 12px; cursor: pointer;">Install</button>
        `;
    }
    buttons += `<button onclick="window.open('${docsUrl}', '_blank')" style="padding: 6px 12px; background: #3e3e42; color: #ccc; border: none; border-radius: 3px; font-size: 11px; cursor: pointer;" title="View official documentation">Docs</button>`;
    
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
    card.style.cssText = 'background: #2d2d30; border: 1px solid #3e3e42; border-radius: 6px; padding: 15px; cursor: pointer; transition: all 0.2s;';
    
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
            // Refresh the tool status
            await renderCliToolsPanel();
        } else {
            console.error(`[CLI Tools] Failed to install ${toolId}:`, result.error);
            alert(`Failed to install: ${result.error}`);
        }
    } catch (error) {
        console.error(`[CLI Tools] Error installing ${toolId}:`, error);
        alert(`Installation error: ${error}`);
    }
}

async function configureCliTool(toolId: string): Promise<void> {
    console.log(`[CLI Tools] Configure requested for ${toolId}`);
    
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
            
            // Don't refresh the entire panel, just update the version if needed
            setTimeout(async () => {
                // Re-detect to get updated version
                const electronAPI = window.electronAPI as any;
                const updatedStatus = await electronAPI.detectCliTool(toolId);
                if (updatedStatus && updatedStatus.version && card) {
                    const versionSpan = card.querySelector(`span[data-version="${toolId}"]`) as HTMLElement;
                    if (versionSpan) {
                        versionSpan.textContent = updatedStatus.version;
                    }
                }
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
            const newWidth = Math.min(Math.max(startWidth + deltaX, 300), 600);
            consensusPanel.style.width = newWidth + 'px';
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
        console.log('[Menu] Opening folder:', folderPath);
        
        // Update the current opened folder state
        currentOpenedFolder = folderPath;
        
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
            // Clear existing explorer and create a new one with the opened folder
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
        
        // Reset localStorage if needed
        localStorage.removeItem('lastOpenedFolder');
        
        // The reload will happen after this
    });
    
    // Listen for close folder event
    (window as any).electronAPI.onMenuCloseFolder(async () => {
        console.log('[Menu] Close folder requested');
        // Reset the current opened folder
        currentOpenedFolder = null;
        
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
            window.scmView = new VSCodeSCMView(scmContainer);
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

// Define global functions for opening folder and cloning repository
window.openFolder = async () => {
    try {
        console.log('[OpenFolder] Starting folder selection...');
        const result = await window.electronAPI.showOpenDialog({
            properties: ['openDirectory']
        });
        
        console.log('[OpenFolder] Dialog result:', result);
        
        if (!result.canceled && result.filePaths.length > 0) {
            const folderPath = result.filePaths[0];
            console.log('[OpenFolder] Selected folder:', folderPath);
            // Use the same handleOpenFolder function that File > Open Folder uses
            handleOpenFolder(folderPath);
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
