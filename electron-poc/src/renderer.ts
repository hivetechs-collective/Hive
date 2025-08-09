/**
 * Hive Consensus - VS Code-like Electron Interface
 * Enhanced UI matching the Dioxus GUI design
 */

import './index.css';

// Activity Bar Items (matching Dioxus structure)
interface ActivityBarItem {
  id: string;
  title: string;
  icon: string;
  badge?: { value: string; isNumber: boolean };
  enabled: boolean;
}

const ACTIVITY_BAR_ITEMS: ActivityBarItem[] = [
  {
    id: 'explorer',
    title: 'Explorer (Ctrl+Shift+E)', 
    icon: '📁',
    enabled: true
  },
  {
    id: 'consensus',
    title: 'Consensus Engine (Ctrl+Shift+C)',
    icon: '🧠', 
    enabled: true
  },
  {
    id: 'git',
    title: 'Source Control (Ctrl+Shift+G)',
    icon: '🌿',
    enabled: true
  },
  {
    id: 'terminal',
    title: 'Terminal (Ctrl+`)',
    icon: '💻',
    enabled: true
  }
];

// Create VS Code-like interface
document.body.innerHTML = `
<div class="vscode-workbench">
  <!-- Title Bar -->
  <div class="title-bar">
    <div class="title-bar-left">
      <div class="window-controls">
        <div class="window-control close"></div>
        <div class="window-control minimize"></div>
        <div class="window-control maximize"></div>
      </div>
    </div>
    <div class="title-bar-center">
      🐝 Hive Consensus - Day 0 Validation
    </div>
    <div class="title-bar-right"></div>
  </div>

  <!-- Main Content Area -->
  <div class="main-container">
    <!-- Activity Bar -->
    <div class="activity-bar">
      <div class="activity-bar-top">
        ${ACTIVITY_BAR_ITEMS.map(item => `
          <div class="activity-bar-item ${item.id === 'consensus' ? 'active' : ''}" 
               data-id="${item.id}" 
               title="${item.title}">
            <div class="activity-bar-icon">${item.icon}</div>
            ${item.badge ? `<div class="activity-badge">${item.badge.value}</div>` : ''}
          </div>
        `).join('')}
      </div>
      <div class="activity-bar-bottom">
        <div class="activity-bar-item" data-id="settings" title="Settings">
          <div class="activity-bar-icon">⚙️</div>
        </div>
      </div>
    </div>

    <!-- Sidebar -->
    <div class="sidebar">
      <div class="sidebar-header">
        <div class="sidebar-title" id="sidebar-title">CONSENSUS ENGINE</div>
      </div>
      <div class="sidebar-content" id="sidebar-content">
        <!-- Consensus Panel -->
        <div id="consensus-panel" class="panel">
          <div class="section">
            <div class="section-header">4-Stage Pipeline</div>
            <div class="stage-pipeline">
              <div class="stage" data-stage="generator">
                <div class="stage-icon">🎯</div>
                <div class="stage-name">Generator</div>
                <div class="stage-status">Ready</div>
              </div>
              <div class="stage" data-stage="refiner">
                <div class="stage-icon">✨</div>
                <div class="stage-name">Refiner</div>
                <div class="stage-status">Waiting</div>
              </div>
              <div class="stage" data-stage="validator">
                <div class="stage-icon">✅</div>
                <div class="stage-name">Validator</div>
                <div class="stage-status">Waiting</div>
              </div>
              <div class="stage" data-stage="curator">
                <div class="stage-icon">🎨</div>
                <div class="stage-name">Curator</div>
                <div class="stage-status">Waiting</div>
              </div>
            </div>
          </div>
          
          <div class="section">
            <div class="section-header">Actions</div>
            <div class="action-buttons">
              <button id="test-connection-btn" class="vscode-button primary">
                <span class="button-icon">🔗</span>
                Test Connection
              </button>
              <button id="run-consensus-btn" class="vscode-button primary">
                <span class="button-icon">🚀</span>
                Run Consensus
              </button>
            </div>
          </div>

          <div class="section">
            <div class="section-header">Status</div>
            <div id="status-indicator" class="status-indicator ready">
              <div class="status-dot"></div>
              <div class="status-text">Ready</div>
            </div>
          </div>
        </div>

        <!-- Explorer Panel (hidden by default) -->
        <div id="explorer-panel" class="panel" style="display: none;">
          <div class="section">
            <div class="section-header">
              EXPLORER
              <div class="section-actions">
                <button class="icon-button" title="New File">📄</button>
                <button class="icon-button" title="New Folder">📁</button>
                <button class="icon-button" title="Refresh">🔄</button>
              </div>
            </div>
            <div class="file-tree">
              <div class="file-item folder">
                <span class="file-icon">📁</span>
                <span class="file-name">hive-consensus</span>
              </div>
              <div class="file-item file" style="margin-left: 16px;">
                <span class="file-icon">📄</span>
                <span class="file-name">package.json</span>
              </div>
              <div class="file-item file" style="margin-left: 16px;">
                <span class="file-icon">⚙️</span>
                <span class="file-name">electron-poc</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Main Editor Area -->
    <div class="editor-container">
      <!-- Tabs -->
      <div class="editor-tabs">
        <div class="tab active">
          <span class="tab-icon">🧠</span>
          <span class="tab-name">Consensus Test</span>
          <span class="tab-close">×</span>
        </div>
      </div>
      
      <!-- Editor Content -->
      <div class="editor-content">
        <div class="results-container">
          <div class="results-header">
            <h3>Day 0 Validation Results</h3>
            <div class="connection-status" id="connection-status">
              <div class="connection-dot"></div>
              <span>Backend: Connecting...</span>
            </div>
          </div>
          <div class="results-output" id="results-output">
            <div class="log-entry">
              <span class="timestamp">[${new Date().toLocaleTimeString()}]</span>
              <span class="message">Initializing Hive Consensus validation...</span>
            </div>
          </div>
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

// DOM elements
const statusIndicator = document.getElementById('status-indicator')!;
const statusText = statusIndicator.querySelector('.status-text')!;
const resultsOutput = document.getElementById('results-output')!;
const connectionStatus = document.getElementById('connection-status')!;
const backendStatus = document.getElementById('backend-status')!;

// Utility functions
function addLogEntry(message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') {
  const entry = document.createElement('div');
  entry.className = `log-entry ${type}`;
  entry.innerHTML = `
    <span class="timestamp">[${new Date().toLocaleTimeString()}]</span>
    <span class="message">${message}</span>
  `;
  resultsOutput.appendChild(entry);
  resultsOutput.scrollTop = resultsOutput.scrollHeight;
}

function updateStatus(text: string, className: string) {
  statusIndicator.className = `status-indicator ${className}`;
  statusText.textContent = text;
}

function updateConnectionStatus(connected: boolean) {
  isConnected = connected;
  const dot = connectionStatus.querySelector('.connection-dot')!;
  const span = connectionStatus.querySelector('span')!;
  
  if (connected) {
    dot.className = 'connection-dot connected';
    span.textContent = 'Backend: Connected';
    backendStatus.textContent = 'Connected';
  } else {
    dot.className = 'connection-dot connecting';
    span.textContent = 'Backend: Connecting...';
    backendStatus.textContent = 'Connecting...';
  }
}

function updateStageStatus(stage: string, status: 'ready' | 'running' | 'completed' | 'error') {
  const stageElement = document.querySelector(`[data-stage="${stage}"]`);
  if (stageElement) {
    const statusElement = stageElement.querySelector('.stage-status')!;
    stageElement.className = `stage ${status}`;
    
    switch (status) {
      case 'ready':
        statusElement.textContent = 'Ready';
        break;
      case 'running':
        statusElement.textContent = 'Running...';
        break;
      case 'completed':
        statusElement.textContent = 'Completed';
        break;
      case 'error':
        statusElement.textContent = 'Error';
        break;
    }
  }
}

// Activity bar click handlers
document.querySelectorAll('.activity-bar-item').forEach(item => {
  item.addEventListener('click', () => {
    const id = item.getAttribute('data-id');
    if (!id) return;

    // Update active state
    document.querySelectorAll('.activity-bar-item').forEach(i => i.classList.remove('active'));
    item.classList.add('active');

    // Switch panels
    switchToView(id);
  });
});

function switchToView(viewId: string) {
  currentView = viewId;
  const sidebarTitle = document.getElementById('sidebar-title')!;
  const consensusPanel = document.getElementById('consensus-panel')!;
  const explorerPanel = document.getElementById('explorer-panel')!;

  // Hide all panels
  consensusPanel.style.display = 'none';
  explorerPanel.style.display = 'none';

  switch (viewId) {
    case 'consensus':
      sidebarTitle.textContent = 'CONSENSUS ENGINE';
      consensusPanel.style.display = 'block';
      break;
    case 'explorer':
      sidebarTitle.textContent = 'EXPLORER';
      explorerPanel.style.display = 'block';
      break;
    case 'git':
      sidebarTitle.textContent = 'SOURCE CONTROL';
      addLogEntry('Source control panel coming soon...', 'info');
      break;
    case 'terminal':
      sidebarTitle.textContent = 'TERMINAL';
      addLogEntry('Terminal panel coming soon...', 'info');
      break;
    case 'settings':
      sidebarTitle.textContent = 'SETTINGS';
      addLogEntry('Settings panel coming soon...', 'info');
      break;
  }
}

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
  
  try {
    const result = await (window as any).backendAPI.runConsensus("What is the capital of France?");
    
    setTimeout(() => {
      updateStageStatus('curator', 'completed');
      updateStatus('Consensus Complete!', 'success');
      addLogEntry(`🎯 Consensus completed in ${result.duration_ms}ms`, 'success');
      addLogEntry(`📝 Model: ${result.model_used}`, 'info');
      addLogEntry(`💬 Result: ${result.result.substring(0, 200)}${result.result.length > 200 ? '...' : ''}`, 'success');
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

// Initialize default view
switchToView('consensus');
addLogEntry('🐝 Hive Consensus Day 0 Validation started', 'info');
addLogEntry('📋 Click buttons above to test the Electron + Rust architecture', 'info');