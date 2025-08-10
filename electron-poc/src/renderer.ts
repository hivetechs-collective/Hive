/**
 * Hive Consensus - Exact Dioxus GUI Recreation
 * Layout: Left Sidebar | LazyGit Panel | Center (with bottom Terminal) | Right Consensus Chat
 */

import './index.css';
import hiveLogo from './Hive-Logo-small.jpg';
import { SettingsModal } from './settings-modal';
import { ConsensusWebSocket, formatTokens, formatCost, STAGE_DISPLAY_NAMES } from './consensus-websocket';

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
        <div class="profile-display" id="active-profile-display">
          <span class="profile-label">Profile:</span>
          <span class="profile-name" id="active-profile-name">Loading...</span>
        </div>
        <div class="pipeline-stages">
          <div class="stage" data-stage="generator">
            <div class="stage-progress">
              <div class="stage-label">Generator</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
              <div class="stage-model" id="generator-model">--</div>
            </div>
          </div>
          <div class="stage" data-stage="refiner">
            <div class="stage-progress">
              <div class="stage-label">Refiner</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
              <div class="stage-model" id="refiner-model">--</div>
            </div>
          </div>
          <div class="stage" data-stage="validator">
            <div class="stage-progress">
              <div class="stage-label">Validator</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
              <div class="stage-model" id="validator-model">--</div>
            </div>
          </div>
          <div class="stage" data-stage="curator">
            <div class="stage-progress">
              <div class="stage-label">Curator</div>
              <div class="progress-bar"><div class="progress-fill"></div></div>
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
let settingsModal: SettingsModal | null = null;
let dailyUsageCount = 0;
let dailyLimit = 5;
let totalTokens = 0;
let totalCost = 0;
let activeProfile: any = null;
let consensusWebSocket: ConsensusWebSocket | null = null;
let currentStreamContent: Map<string, string> = new Map();

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

// Declare runConsensusViaREST as a variable that will hold the function
let runConsensusViaREST: (query: string) => Promise<void>;

function addChatMessage(message: string, isSystem: boolean = false) {
  const messageDiv = document.createElement('div');
  messageDiv.className = `chat-message ${isSystem ? 'system' : 'user'}`;
  messageDiv.innerHTML = `
    <div class="message-time">[${new Date().toLocaleTimeString()}]</div>
    <div class="message-content">${message}</div>
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
  totalTokens = 0;
  totalCost = 0;
  updateConsensusStats();
  resetStageStatus();
  updateStatus('Running Consensus...', 'processing');
  addLogEntry('🚀 Starting streaming consensus pipeline...', 'info');
  
  // Test query
  const testQuery = "What is the capital of France?";
  addChatMessage(testQuery, false);
  
  // Get current profile from settings or use default
  const currentProfileName = activeProfile?.name || 'balanced-performer';
  
  // Start consensus via WebSocket
  consensusWebSocket.startConsensus(testQuery, currentProfileName);
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
  totalTokens = 0;
  totalCost = 0;
  updateConsensusStats();
  resetStageStatus();
  
  addLogEntry(`🚀 Starting streaming consensus for: "${query}"`, 'info');
  
  // Get current profile from settings or use default
  const currentProfileName = activeProfile?.name || 'Free Also';
  
  // Start consensus via WebSocket
  consensusWebSocket.startConsensus(query, currentProfileName);
});

// Fallback REST API function
runConsensusViaREST = async (query: string) => {
  isProcessing = true;
  totalTokens = 0;
  totalCost = 0;
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
function initializeWebSocket() {
  const wsUrl = 'ws://127.0.0.1:8765/ws';
  
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
      updateModelDisplay(stageName, model);
      addLogEntry(`▶️ ${stage} started with ${model}`, 'info');
      currentStreamContent.set(stageName, '');
    },
    
    onStreamChunk: (stage, chunk) => {
      // Accumulate content for each stage
      const stageName = stage.toLowerCase();
      const currentContent = currentStreamContent.get(stageName) || '';
      currentStreamContent.set(stageName, currentContent + chunk);
      
      // Show streaming output from all stages, not just Curator
      const chatContent = document.getElementById('chat-content');
      
      // Find or create message for this stage
      let stageMessage = chatContent?.querySelector(`.streaming-${stageName}`);
      
      if (!stageMessage) {
        // Create new message container for this stage
        const message = document.createElement('div');
        message.className = `chat-message system streaming streaming-${stageName}`;
        message.innerHTML = `
          <div class="message-time">[${new Date().toLocaleTimeString()}] ${stage}</div>
          <div class="message-content"></div>
        `;
        chatContent?.appendChild(message);
        stageMessage = message;
      }
      
      // Update content - convert markdown-like formatting to HTML
      const contentEl = stageMessage.querySelector('.message-content');
      if (contentEl) {
        // Simple markdown to HTML conversion
        let htmlContent = currentStreamContent.get(stageName) || '';
        
        // Convert markdown formatting
        htmlContent = htmlContent
          .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>') // Bold
          .replace(/\*(.*?)\*/g, '<em>$1</em>') // Italic
          .replace(/\n\n/g, '</p><p>') // Paragraphs
          .replace(/\n/g, '<br>'); // Line breaks
        
        // Wrap in paragraph if not already
        if (!htmlContent.startsWith('<p>')) {
          htmlContent = '<p>' + htmlContent + '</p>';
        }
        
        contentEl.innerHTML = htmlContent;
      }
      
      // Always auto-scroll to show the newest content
      autoScrollChat();
    },
    
    onStageProgress: (stage, percentage, tokens) => {
      const stageName = stage.toLowerCase();
      updateStageProgress(stageName, percentage);
      
      // Update token count
      if (stage === 'Generator') {
        totalTokens = tokens;
      } else {
        totalTokens += tokens;
      }
      updateConsensusStats();
    },
    
    onStageCompleted: (stage, tokens, cost) => {
      const stageName = stage.toLowerCase();
      updateStageStatus(stageName, 'completed');
      totalTokens += tokens;
      totalCost += cost;
      updateConsensusStats();
      addLogEntry(`✅ ${stage} completed (${tokens} tokens, ${formatCost(cost)})`, 'success');
    },
    
    onConsensusComplete: (result, finalTokens, finalCost) => {
      totalTokens = finalTokens;
      totalCost = finalCost;
      updateConsensusStats();
      
      // Remove all streaming indicators
      const chatContent = document.getElementById('chat-content');
      const streamingMessages = chatContent?.querySelectorAll('.streaming');
      streamingMessages?.forEach(msg => {
        msg.classList.remove('streaming');
        // Remove stage-specific streaming classes
        msg.className = msg.className.replace(/streaming-\w+/g, '').trim();
      });
      
      // Add final consensus result if provided
      if (result && result.trim()) {
        const finalMessage = document.createElement('div');
        finalMessage.className = 'chat-message system';
        finalMessage.innerHTML = `
          <div class="message-time">[${new Date().toLocaleTimeString()}] Final Consensus</div>
          <div class="message-content"><strong>${result}</strong></div>
        `;
        chatContent?.appendChild(finalMessage);
      }
      
      // Auto-scroll to ensure the complete result is visible
      autoScrollChat();
      
      addLogEntry(`🎯 Consensus complete! Total: ${formatTokens(finalTokens)} tokens, ${formatCost(finalCost)}`, 'success');
      
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
    },
    
    onAIHelperDecision: (directMode, reason) => {
      addLogEntry(`🤖 AI Helper: ${reason}`, 'info');
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
  const stageElement = document.querySelector(`[data-stage="${stage}"] .progress-fill`) as HTMLElement;
  if (stageElement) {
    stageElement.style.width = `${percentage}%`;
  }
}

function resetStageStatus() {
  ['generator', 'refiner', 'validator', 'curator'].forEach(stage => {
    updateStageStatus(stage, 'ready');
    updateStageProgress(stage, 0);
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
          planElement.textContent = tier;
          console.log('Set plan display to:', tier);
        }
        
        // Update conversations display based on what D1 actually returns
        if (conversationsElement) {
          console.log('Updating conversations with:', {
            remaining: result.licenseInfo.remaining,
            dailyUsed: result.licenseInfo.dailyUsed,
            dailyLimit: result.licenseInfo.dailyLimit
          });
          
          if (result.licenseInfo.remaining !== undefined) {
            // D1 provided remaining count
            if (result.licenseInfo.remaining === 'unlimited' || result.licenseInfo.remaining === 2147483647 || result.licenseInfo.remaining === 4294967295) {
              conversationsElement.textContent = 'Unlimited';
            } else {
              // Calculate used from remaining
              const limit = result.licenseInfo.dailyLimit || 10;
              const used = limit - result.licenseInfo.remaining;
              dailyUsageCount = used;
              dailyLimit = limit;
              conversationsElement.textContent = `${used} used / ${result.licenseInfo.remaining} remaining`;
            }
          } else if (result.licenseInfo.dailyUsed !== undefined && result.licenseInfo.dailyLimit !== undefined) {
            // D1 provided used count and limit
            dailyUsageCount = result.licenseInfo.dailyUsed;
            dailyLimit = result.licenseInfo.dailyLimit;
            const remaining = result.licenseInfo.dailyLimit - result.licenseInfo.dailyUsed;
            conversationsElement.textContent = `${result.licenseInfo.dailyUsed} used / ${remaining} remaining`;
          } else if (result.licenseInfo.dailyLimit !== undefined) {
            // D1 only provided limit (no usage data from validation endpoint)
            // Track usage locally since D1 tracks via pre/post conversation
            dailyLimit = result.licenseInfo.dailyLimit;
            conversationsElement.textContent = `${dailyUsageCount} used / ${dailyLimit} remaining`;
          } else {
            // No usage info available
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
settingsModal = new SettingsModal(() => {
  // Callback when settings are saved
  updateStatusBar();
  loadActiveProfile(); // Reload profile when settings change
});
settingsModal.initializeModal(document.body);

// Function to update just the conversation count
function updateConversationCount() {
  const conversationsElement = document.getElementById('status-conversations');
  if (conversationsElement) {
    const remaining = Math.max(0, dailyLimit - dailyUsageCount);
    conversationsElement.textContent = `${dailyUsageCount} used / ${remaining} remaining`;
  }
}

// Function to update consensus stats (tokens and cost)
function updateConsensusStats() {
  const tokenElement = document.getElementById('token-count');
  const costElement = document.getElementById('cost-count');
  
  if (tokenElement) {
    tokenElement.textContent = totalTokens.toLocaleString();
  }
  
  if (costElement) {
    costElement.textContent = `$${totalCost.toFixed(4)}`;
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
setTimeout(() => {
  updateStatusBar();
  loadActiveProfile();
}, 500);