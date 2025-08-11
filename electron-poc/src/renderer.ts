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
let currentStageTokens = 0; // Track tokens for the current stage only
let activeProfile: any = null;
let consensusWebSocket: ConsensusWebSocket | null = null;
let currentStreamContent: Map<string, string> = new Map();

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
  let html = markdown;
  
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
  
  // Get current profile from settings or use default
  const currentProfileName = activeProfile?.name || 'balanced-performer';
  
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
  totalTokens = 0;
  totalCost = 0;
  currentStageTokens = 0;
  updateConsensusStats();
  resetStageStatus();
  
  addLogEntry(`🚀 Starting streaming consensus for: "${query}"`, 'info');
  
  // Create new conversation if needed
  if (!currentConversationId) {
    currentConversationId = generateConversationId();
    addLogEntry(`📝 New conversation started: ${currentConversationId}`, 'info');
  }
  
  // Add to conversation history
  conversationHistory.push({ role: 'user', content: query });
  
  // Get current profile from settings or use default
  const currentProfileName = activeProfile?.name || 'Free Also';
  
  // Start consensus via WebSocket with conversation context
  consensusWebSocket.startConsensus(query, currentProfileName, currentConversationId, conversationHistory);
});

// Fallback REST API function
runConsensusViaREST = async (query: string) => {
  isProcessing = true;
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
function initializeWebSocket() {
  // Prevent multiple initializations
  if (consensusWebSocket) {
    console.log('WebSocket already initialized');
    return;
  }
  
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
      updateStageProgress(stageName, 25); // Start at 25% when stage begins
      updateModelDisplay(stageName, model);
      addLogEntry(`▶️ ${stage} started with ${model}`, 'info');
      currentStreamContent.set(stageName, '');
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
      
      // Mark as no longer processing
      isProcessing = false;
      
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
setTimeout(() => {
  updateStatusBar();
  loadActiveProfile();
}, 500);