/**
 * Hive Consensus - Exact Dioxus GUI Recreation
 * Layout: Left Sidebar | LazyGit Panel | Center (with bottom Terminal) | Right Consensus Chat
 */

import './index.css';
import hiveLogo from './Hive-Logo-small.jpg';
import { SettingsModal } from './settings-modal';

// Create the exact Hive Consensus GUI layout
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
      <span class="title-text">Hive Consensus - Day 0 Validation</span>
    </div>
    <div class="title-bar-right"></div>
  </div>

  <!-- Main Content Area - Exact Dioxus Layout -->
  <div class="main-content">
    <!-- Left Sidebar with Logo and Control Buttons -->
    <div class="sidebar" id="left-sidebar">
      <!-- Logo Section at Top -->
      <div class="logo-section">
        <div class="gradient-line"></div>
        <div class="logo-container">
          <img src="${hiveLogo}" alt="HiveTechs Logo" 
               style="width: 64px; height: 64px; object-fit: contain; border-radius: 8px;" 
               class="hive-logo" />
          <div class="logo-version">2.0.0</div>
        </div>
      </div>

      <!-- Git Panel -->
      <div class="git-panel">
        <div class="panel-header">SOURCE CONTROL</div>
        <div class="git-info">
          <div class="branch-info">
            <span class="branch-icon">🔀</span>
            <span class="branch-name">main</span>
          </div>
        </div>
        <div class="git-buttons">
          <button class="git-btn" title="Pull">↓</button>
          <button class="git-btn" title="Push">↑</button>
          <button class="git-btn" title="Sync">🔄</button>
        </div>
      </div>

      <!-- Control Buttons Panel -->
      <div class="control-panel">
        <button class="control-btn" id="analytics-btn" data-panel="analytics">
          <span class="control-icon">📊</span>
          <span class="control-label">Analytics</span>
        </button>
        <button class="control-btn" id="settings-btn" data-panel="settings">
          <span class="control-icon">⚙️</span>
          <span class="control-label">Settings</span>
        </button>
        <button class="control-btn" id="memory-btn" data-panel="memory">
          <span class="control-icon">🧠</span>
          <span class="control-label">Memory</span>
        </button>
      </div>
    </div>

    <!-- LazyGit Panel (Middle Left) -->
    <div class="lazygit-panel" id="lazygit-panel">
      <div class="panel-header">LAZYGIT</div>
      <div class="lazygit-content">
        <div class="lazygit-placeholder">
          <div class="lazygit-status">
            <div class="status-item">📁 /hive</div>
            <div class="status-item">🌿 main</div>
            <div class="status-item">✅ Clean</div>
          </div>
          <div class="lazygit-files">
            <div class="file-status">
              <span class="file-icon">📄</span>
              <span class="file-name">electron-poc/src/renderer.ts</span>
              <span class="file-status-indicator modified">M</span>
            </div>
            <div class="file-status">
              <span class="file-icon">📄</span>
              <span class="file-name">electron-poc/src/index.css</span>
              <span class="file-status-indicator modified">M</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Center Area (Tabs + Main Content + Terminal) -->
    <div class="center-area" id="center-area">
      <!-- Tabs -->
      <div class="editor-tabs">
        <div class="tab active">
          <span class="tab-icon">🧠</span>
          <span class="tab-name">Day 0 Validation</span>
          <span class="tab-close">×</span>
        </div>
      </div>
      
      <!-- Main Content Area (Above Terminal) -->
      <div class="main-editor-area">
        <div class="welcome-content">
          <h2>Hive Consensus - Day 0 Validation</h2>
          <div class="validation-status">
            <div class="status-item">
              <span class="status-icon">✅</span>
              <span>Electron App Running</span>
            </div>
            <div class="status-item">
              <span class="status-icon">✅</span>
              <span>Rust Backend Connected</span>
            </div>
            <div class="status-item">
              <span class="status-icon">✅</span>
              <span>IPC Communication Working</span>
            </div>
          </div>
          <div class="test-buttons">
            <button id="test-connection-btn" class="action-button primary">
              <span class="button-icon">🔗</span>
              Test Connection
            </button>
            <button id="run-consensus-btn" class="action-button primary">
              <span class="button-icon">🚀</span>
              Run Consensus
            </button>
          </div>
        </div>
      </div>

      <!-- Terminal Section (Bottom, toggleable) -->
      <div class="terminal-section" id="terminal-section">
        <div class="terminal-header">
          <span class="terminal-title">TERMINAL</span>
          <div class="terminal-controls">
            <button class="terminal-btn" id="toggle-terminal">−</button>
          </div>
        </div>
        <div class="terminal-content" id="terminal-content">
          <div id="terminal-output">
            <div class="terminal-line">[${new Date().toLocaleTimeString()}] Hive Consensus initialized</div>
            <div class="terminal-line">[${new Date().toLocaleTimeString()}] Backend server: http://localhost:8765</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Panel (Consensus Chat) -->
    <div class="consensus-chat-panel" id="consensus-chat">
      <!-- Progress Bars at Top -->
      <div class="progress-section">
        <div class="progress-header">
          <img src="${hiveLogo}" alt="Hive" 
               style="width: 20px; height: 20px; object-fit: contain; border-radius: 3px;" />
          <span>Consensus Progress</span>
        </div>
        <div class="pipeline-stages">
          <div class="stage" data-stage="generator">
            <div class="stage-progress">
              <div class="stage-label">Generator</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
            </div>
          </div>
          <div class="stage" data-stage="refiner">
            <div class="stage-progress">
              <div class="stage-label">Refiner</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
            </div>
          </div>
          <div class="stage" data-stage="validator">
            <div class="stage-progress">
              <div class="stage-label">Validator</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
            </div>
          </div>
          <div class="stage" data-stage="curator">
            <div class="stage-progress">
              <div class="stage-label">Curator</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
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
      <div class="status-item">
        <span class="status-icon">👤</span>
        <span id="status-user">Not logged in</span>
      </div>
      <div class="status-item">
        <span class="status-icon">📦</span>
        <span id="status-plan">Free</span>
      </div>
      <div class="status-item">
        <span class="status-icon">💬</span>
        <span id="status-conversations">-- remaining</span>
      </div>
      <div class="status-divider">|</div>
      <div class="status-item">
        <span class="status-icon">🌿</span>
        <span>main</span>
      </div>
      <div class="status-item">
        <span class="status-icon">⚠️</span>
        <span>0</span>
      </div>
      <div class="status-item">
        <span class="status-icon">🚫</span>
        <span>0</span>
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
let settingsModal: SettingsModal | null = null;

// DOM elements - Updated for new layout
const terminalOutput = document.getElementById('terminal-output')!;
const backendStatus = document.getElementById('backend-status')!;
const chatContent = document.getElementById('chat-content')!;

// Utility functions
function addLogEntry(message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') {
  const entry = document.createElement('div');
  entry.className = `terminal-line ${type}`;
  entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
  terminalOutput.appendChild(entry);
  terminalOutput.scrollTop = terminalOutput.scrollHeight;
}

function addChatMessage(message: string, isSystem: boolean = false) {
  const messageDiv = document.createElement('div');
  messageDiv.className = `chat-message ${isSystem ? 'system' : 'user'}`;
  messageDiv.innerHTML = `
    <div class="message-time">[${new Date().toLocaleTimeString()}]</div>
    <div class="message-content">${message}</div>
  `;
  chatContent.appendChild(messageDiv);
  chatContent.scrollTop = chatContent.scrollHeight;
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
    const stageLabel = stageElement.querySelector('.stage-label')!;
    
    switch (status) {
      case 'ready':
        progressFill.style.width = '0%';
        addLogEntry(`🔄 ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage ready`, 'info');
        break;
      case 'running':
        progressFill.style.width = '50%';
        addLogEntry(`⚡ ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage running...`, 'info');
        break;
      case 'completed':
        progressFill.style.width = '100%';
        addLogEntry(`✅ ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage completed`, 'info');
        break;
      case 'error':
        progressFill.style.width = '0%';
        progressFill.style.background = '#F44336';
        addLogEntry(`❌ ${stage.charAt(0).toUpperCase() + stage.slice(1)} stage error`, 'error');
        break;
    }
  }
}

// Control panel button handlers
document.getElementById('analytics-btn')?.addEventListener('click', () => {
  addLogEntry('📊 Analytics panel clicked', 'info');
  addChatMessage('Analytics functionality coming soon...', true);
});

document.getElementById('settings-btn')?.addEventListener('click', () => {
  addLogEntry('⚙️ Opening settings...', 'info');
  if (settingsModal) {
    settingsModal.showModal();
  }
});

document.getElementById('memory-btn')?.addEventListener('click', () => {
  addLogEntry('🧠 Memory panel clicked', 'info');
  addChatMessage('Memory management coming soon...', true);
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
  if (isProcessing) return;
  
  isProcessing = true;
  updateStatus('Running Consensus...', 'processing');
  addLogEntry('🚀 Starting 4-stage consensus pipeline...', 'info');
  
  // Animate stages
  updateStageStatus('generator', 'running');
  
  setTimeout(() => {
    updateStageStatus('generator', 'completed');
    updateStageStatus('refiner', 'running');
  }, 500);
  
  setTimeout(() => {
    updateStageStatus('refiner', 'completed');
    updateStageStatus('validator', 'running');
  }, 1000);
  
  setTimeout(() => {
    updateStageStatus('validator', 'completed');
    updateStageStatus('curator', 'running');
  }, 1500);
  
  // Add query to chat
  addChatMessage("What is the capital of France?", false);
  
  try {
    const result = await (window as any).backendAPI.runConsensus("What is the capital of France?");
    
    setTimeout(() => {
      updateStageStatus('curator', 'completed');
      updateStatus('Consensus Complete!', 'success');
      addLogEntry(`🎯 Consensus completed in ${result.duration_ms}ms`, 'success');
      addLogEntry(`📝 Model: ${result.model_used}`, 'info');
      addLogEntry(`💬 Result: ${result.result.substring(0, 200)}${result.result.length > 200 ? '...' : ''}`, 'success');
      
      // Also show result in chat
      addChatMessage(result.result, true);
    }, 2000);
    
  } catch (error) {
    updateStageStatus('generator', 'error');
    updateStageStatus('refiner', 'ready');
    updateStageStatus('validator', 'ready');
    updateStageStatus('curator', 'ready');
    updateStatus('Consensus Failed', 'error');
    addLogEntry(`❌ Consensus failed: ${error}`, 'error');
  } finally {
    setTimeout(() => {
      isProcessing = false;
    }, 2500);
  }
});

// Chat input handler
document.getElementById('send-chat')?.addEventListener('click', async () => {
  const chatInput = document.getElementById('chat-input') as HTMLInputElement;
  const query = chatInput.value.trim();
  
  if (!query || isProcessing) return;
  
  chatInput.value = '';
  addChatMessage(query, false);
  
  if (!isConnected) {
    addChatMessage('Please connect to backend first', true);
    return;
  }
  
  isProcessing = true;
  addLogEntry(`🚀 Running consensus for: "${query}"`, 'info');
  
  // Animate stages
  updateStageStatus('generator', 'running');
  setTimeout(() => {
    updateStageStatus('generator', 'completed');
    updateStageStatus('refiner', 'running');
  }, 500);
  setTimeout(() => {
    updateStageStatus('refiner', 'completed');
    updateStageStatus('validator', 'running');
  }, 1000);
  setTimeout(() => {
    updateStageStatus('validator', 'completed');
    updateStageStatus('curator', 'running');
  }, 1500);
  
  try {
    const result = await (window as any).backendAPI.runConsensus(query);
    
    setTimeout(() => {
      updateStageStatus('curator', 'completed');
      addLogEntry(`✅ Consensus completed for: "${query}"`, 'success');
      addChatMessage(result.result, true);
    }, 2000);
    
  } catch (error) {
    updateStageStatus('generator', 'error');
    updateStageStatus('refiner', 'ready');
    updateStageStatus('validator', 'ready');
    updateStageStatus('curator', 'ready');
    addLogEntry(`❌ Consensus failed: ${error}`, 'error');
    addChatMessage(`Error: ${error}`, true);
  } finally {
    setTimeout(() => {
      isProcessing = false;
    }, 2500);
  }
});

// Enter key support for chat input
document.getElementById('chat-input')?.addEventListener('keypress', (e) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    document.getElementById('send-chat')?.click();
  }
});

// Auto-test connection on startup
setTimeout(async () => {
  addLogEntry('🔄 Auto-testing backend connection...', 'info');
  
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

// Function to update status bar with license info
async function updateStatusBar() {
  try {
    const settings = await (window as any).settingsAPI.load();
    
    const userElement = document.getElementById('status-user');
    const planElement = document.getElementById('status-plan');
    const conversationsElement = document.getElementById('status-conversations');
    
    if (settings.hiveKey) {
      // Test the key to get license info
      const result = await (window as any).settingsAPI.testKeys({
        hiveKey: settings.hiveKey
      });
      
      if (result.hiveValid && result.licenseInfo) {
        // Update user display
        if (userElement) {
          const email = result.licenseInfo.email || 'Licensed User';
          // Truncate email if too long for status bar
          const displayEmail = email.length > 20 ? email.substring(0, 17) + '...' : email;
          userElement.textContent = displayEmail;
          userElement.title = email; // Full email in tooltip
        }
        
        // Update plan display
        if (planElement) {
          planElement.textContent = result.licenseInfo.tier || 'Free';
        }
        
        // Update conversations display
        if (conversationsElement) {
          if (result.licenseInfo.remaining !== undefined) {
            if (result.licenseInfo.remaining === 'unlimited' || result.licenseInfo.remaining === 2147483647 || result.licenseInfo.remaining === 4294967295) {
              conversationsElement.textContent = 'Unlimited';
            } else {
              conversationsElement.textContent = `${result.licenseInfo.remaining} remaining`;
            }
          } else if (result.licenseInfo.dailyUsed !== undefined && result.licenseInfo.dailyLimit) {
            const remaining = result.licenseInfo.dailyLimit - result.licenseInfo.dailyUsed;
            conversationsElement.textContent = `${remaining} remaining`;
          } else if (result.licenseInfo.dailyLimit) {
            conversationsElement.textContent = `${result.licenseInfo.dailyLimit} daily`;
          } else {
            conversationsElement.textContent = '-- remaining';
          }
        }
      } else {
        // Invalid license
        if (userElement) userElement.textContent = 'Invalid license';
        if (planElement) planElement.textContent = 'Free';
        if (conversationsElement) conversationsElement.textContent = '-- remaining';
      }
    } else {
      // No license key
      if (userElement) userElement.textContent = 'Not logged in';
      if (planElement) planElement.textContent = 'Free';
      if (conversationsElement) conversationsElement.textContent = '-- remaining';
    }
  } catch (error) {
    console.error('Failed to update status bar:', error);
    // Set defaults on error
    const userElement = document.getElementById('status-user');
    const planElement = document.getElementById('status-plan');
    const conversationsElement = document.getElementById('status-conversations');
    
    if (userElement) userElement.textContent = 'Not logged in';
    if (planElement) planElement.textContent = 'Free';
    if (conversationsElement) conversationsElement.textContent = '-- remaining';
  }
}

// Initialize settings modal with callback to update status bar
settingsModal = new SettingsModal(() => {
  // Callback when settings are saved
  updateStatusBar();
});
settingsModal.initializeModal(document.body);

// Update status bar on startup
setTimeout(() => {
  updateStatusBar();
}, 500);