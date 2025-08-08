const { invoke } = window.__TAURI__.core;

// State management
let isProcessing = false;
let currentView = 'explorer';
let autoAccept = false;
let terminalVisible = true;

// Initialize on page load
window.addEventListener("DOMContentLoaded", () => {
  setupEventListeners();
  loadFileExplorer();
  logToTerminal("🐝 HiveTechs Consensus initialized");
  
  // Setup keyboard shortcuts
  document.addEventListener('keydown', handleGlobalKeyDown);
});

function setupEventListeners() {
  // Query input - Enter to execute
  const queryInput = document.getElementById('queryInput');
  queryInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      runConsensus();
    }
  });
}

// Handle global keyboard shortcuts
function handleGlobalKeyDown(e) {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const ctrlOrCmd = isMac ? e.metaKey : e.ctrlKey;
  
  if (ctrlOrCmd) {
    switch(e.key) {
      case 't':
        e.preventDefault();
        toggleTerminal();
        break;
      case 'c':
        if (isProcessing) {
          e.preventDefault();
          cancelConsensus();
        }
        break;
    }
  }
  
  // Shift+Tab toggles auto-accept
  if (e.key === 'Tab' && e.shiftKey) {
    e.preventDefault();
    toggleAutoAccept();
  }
}

// Switch sidebar view
function switchSidebarView(view) {
  currentView = view;
  
  // Update button states
  document.querySelectorAll('.view-toggle-btn').forEach(btn => {
    btn.classList.remove('active');
  });
  event.target.classList.add('active');
  
  // Update explorer header
  const sectionTitle = document.querySelector('.sidebar-section-title');
  
  switch(view) {
    case 'explorer':
      sectionTitle.textContent = 'EXPLORER';
      loadFileExplorer();
      break;
    case 'git':
      sectionTitle.textContent = 'SOURCE CONTROL';
      loadGitStatus();
      break;
  }
}

// Toggle terminal visibility
function toggleTerminal() {
  const terminal = document.getElementById('terminalContainer');
  terminalVisible = !terminalVisible;
  terminal.style.display = terminalVisible ? 'flex' : 'none';
  logToTerminal(`🖥️ Terminal ${terminalVisible ? 'shown' : 'hidden'}`);
}

// Toggle auto-accept
function toggleAutoAccept() {
  autoAccept = !autoAccept;
  const toggle = document.getElementById('autoAcceptToggle');
  toggle.classList.toggle('active', autoAccept);
  logToTerminal(`🔄 Auto-accept ${autoAccept ? 'enabled' : 'disabled'}`);
}

// Run consensus
async function runConsensus() {
  if (isProcessing) return;
  
  const queryInput = document.getElementById('queryInput');
  const query = queryInput.value.trim();
  
  if (!query) {
    logToTerminal("⚠️ Error: Query cannot be empty");
    return;
  }
  
  isProcessing = true;
  updateButtonStates();
  resetStages();
  
  logToTerminal(`🐝 Running consensus for: "${query}"`);
  updateResponseContent("Processing your query...\n");
  
  try {
    // Use streaming consensus for real-time updates
    await runConsensusWithStreaming(query);
    
    // Clear input after successful run
    queryInput.value = '';
    
  } catch (error) {
    updateResponseContent(`❌ Error: ${error}`);
    logToTerminal(`❌ Error: ${error}`);
  } finally {
    isProcessing = false;
    updateButtonStates();
  }
}

// Run consensus with streaming updates
async function runConsensusWithStreaming(query) {
  // Set up event listeners for progress updates
  const unlisten1 = await window.__TAURI__.event.listen('consensus-progress', (event) => {
    const progress = event.payload;
    const stageName = progress.stage.toLowerCase();
    
    updateStage(stageName, 'active', progress.message);
    
    if (progress.progress === 100) {
      updateStage(stageName, 'complete', progress.message);
    }
  });
  
  const unlisten2 = await window.__TAURI__.event.listen('consensus-complete', (event) => {
    const result = event.payload;
    updateResponseContent(result.result);
    logToTerminal(`✅ Consensus completed in ${result.duration_ms}ms | Cost: $${result.total_cost.toFixed(4)} | Tokens: ${result.total_tokens}`);
    
    // Update analytics
    updateAnalytics({
      total_queries: 1,
      total_cost: result.total_cost,
      total_tokens: result.total_tokens
    });
    
    // Clean up listeners
    unlisten1();
    unlisten2();
  });
  
  // Start streaming consensus
  try {
    await invoke("run_consensus_streaming", { query });
  } catch (error) {
    // Clean up listeners on error
    unlisten1();
    unlisten2();
    throw error;
  }
}

// Cancel consensus
async function cancelConsensus() {
  if (!isProcessing) return;
  
  logToTerminal("🛑 Cancelling consensus...");
  
  try {
    await invoke("cancel_consensus");
    isProcessing = false;
    updateButtonStates();
    resetStages();
    updateResponseContent("❌ Consensus cancelled by user");
    logToTerminal("✅ Consensus cancelled");
  } catch (error) {
    logToTerminal(`❌ Failed to cancel: ${error}`);
  }
}

// Update stage display
function updateStage(stageName, status, message) {
  const stageElement = document.getElementById(`stage-${stageName}`);
  if (!stageElement) return;
  
  // Remove all status classes
  stageElement.classList.remove('active', 'complete');
  
  // Add new status class
  if (status) {
    stageElement.classList.add(status);
  }
  
  // Update content if message provided
  if (message) {
    const contentElement = stageElement.querySelector('.stage-content');
    if (contentElement) {
      contentElement.textContent = message;
    }
  }
}

// Reset all stages
function resetStages() {
  const stages = ['generator', 'refiner', 'validator', 'curator'];
  stages.forEach(stage => {
    updateStage(stage, null, stage.charAt(0).toUpperCase() + stage.slice(1));
  });
}

// Update button states
function updateButtonStates() {
  const runBtn = document.getElementById('runBtn');
  const cancelBtn = document.getElementById('cancelBtn');
  
  runBtn.disabled = isProcessing;
  cancelBtn.disabled = !isProcessing;
}

// Update response content
function updateResponseContent(content) {
  const responseContent = document.getElementById('responseContent');
  responseContent.textContent = content;
  
  // Auto-scroll to bottom
  const responseArea = responseContent.parentElement;
  responseArea.scrollTop = responseArea.scrollHeight;
}

// Log to terminal
function logToTerminal(message) {
  const terminal = document.getElementById('terminalContent');
  const timestamp = new Date().toLocaleTimeString();
  terminal.innerHTML += `<br>$ [${timestamp}] ${message}`;
  terminal.scrollTop = terminal.scrollHeight;
}

// Load file explorer
async function loadFileExplorer() {
  if (currentView !== 'explorer') return;
  
  try {
    const files = await invoke("read_directory", { path: "." });
    displayFileTree(files);
  } catch (error) {
    console.error("Failed to load file explorer:", error);
    logToTerminal(`❌ Failed to load files: ${error}`);
  }
}

// Display file tree
function displayFileTree(files) {
  const fileTree = document.getElementById('fileTree');
  
  let html = '';
  for (const file of files) {
    const icon = file.is_dir ? '📁' : getFileIcon(file.name);
    html += `
      <div class="file-item" onclick="handleFileClick('${file.path}', ${file.is_dir})">
        ${icon} ${file.name}
      </div>
    `;
  }
  fileTree.innerHTML = html;
}

// Get appropriate file icon based on extension
function getFileIcon(filename) {
  const ext = filename.split('.').pop().toLowerCase();
  const iconMap = {
    'rs': '🦀',
    'js': '📜',
    'ts': '📘',
    'json': '📋',
    'html': '🌐',
    'css': '🎨',
    'md': '📝',
    'toml': '⚙️',
    'lock': '🔒'
  };
  return iconMap[ext] || '📄';
}

// Handle file click
async function handleFileClick(path, isDir) {
  if (isDir) {
    try {
      const files = await invoke("read_directory", { path });
      displayFileTree(files);
      
      // Update current path display
      document.getElementById('currentPath').textContent = path;
      logToTerminal(`📁 Opened directory: ${path}`);
    } catch (error) {
      logToTerminal(`❌ Error opening directory: ${error}`);
    }
  } else {
    try {
      const content = await invoke("read_file", { path });
      // TODO: Display in editor with syntax highlighting
      logToTerminal(`📄 Opened file: ${path}`);
      
      // For now, show file content in the editor area
      const welcomeView = document.getElementById('welcomeView');
      welcomeView.innerHTML = `
        <div style="font-family: 'Consolas', 'Monaco', monospace;">
          <h3 style="color: var(--hive-yellow); margin-bottom: 10px;">${path}</h3>
          <pre style="color: var(--text-secondary); overflow: auto; background: var(--hive-dark-bg); padding: 15px; border-radius: 6px;">${escapeHtml(content)}</pre>
        </div>
      `;
    } catch (error) {
      logToTerminal(`❌ Error opening file: ${error}`);
    }
  }
}

// Load git status
async function loadGitStatus() {
  if (currentView !== 'git') return;
  
  const fileTree = document.getElementById('fileTree');
  
  // For now, show placeholder git info
  fileTree.innerHTML = `
    <div class="file-item">📝 Modified (3)</div>
    <div class="file-item" style="padding-left: 20px;">M src/main.rs</div>
    <div class="file-item" style="padding-left: 20px;">M frontend/index.html</div>
    <div class="file-item" style="padding-left: 20px;">M frontend/main.js</div>
    <div class="file-item">📦 Staged (0)</div>
    <div class="file-item">🔀 Branches</div>
    <div class="file-item" style="padding-left: 20px;">• main</div>
    <div class="file-item" style="padding-left: 20px;">  develop</div>
  `;
  
  logToTerminal("🔀 Git status loaded");
}

// Update analytics display
let cumulativeAnalytics = {
  totalQueries: 0,
  totalCost: 0,
  successCount: 0,
  totalTime: 0
};

function updateAnalytics(newData) {
  cumulativeAnalytics.totalQueries += newData.total_queries || 0;
  cumulativeAnalytics.totalCost += newData.total_cost || 0;
  
  // Update UI if analytics view is visible
  const analyticsView = document.getElementById('analyticsView');
  if (analyticsView.style.display !== 'none') {
    document.getElementById('totalQueries').textContent = cumulativeAnalytics.totalQueries;
    document.getElementById('totalCost').textContent = `$${cumulativeAnalytics.totalCost.toFixed(4)}`;
    document.getElementById('successRate').textContent = '98.5%';
    document.getElementById('avgResponse').textContent = '2.3s';
  }
}

// Show analytics view
function showAnalyticsView() {
  document.getElementById('welcomeView').style.display = 'none';
  document.getElementById('analyticsView').style.display = 'block';
  
  // Load analytics data
  loadAnalyticsData();
}

// Load analytics data from backend
async function loadAnalyticsData() {
  try {
    const analytics = await invoke("get_analytics_data");
    
    document.getElementById('totalQueries').textContent = analytics.total_queries;
    document.getElementById('totalCost').textContent = `$${analytics.total_cost.toFixed(4)}`;
    document.getElementById('successRate').textContent = `${analytics.success_rate.toFixed(1)}%`;
    document.getElementById('avgResponse').textContent = `${analytics.avg_response_time.toFixed(1)}s`;
    
    logToTerminal("📊 Analytics data loaded");
  } catch (error) {
    console.error("Failed to load analytics:", error);
  }
}

// Load consensus profiles
async function loadConsensusProfiles() {
  try {
    const profiles = await invoke("get_profiles");
    
    // Could display profiles in a dropdown or list
    logToTerminal(`📋 Loaded ${profiles.length} consensus profiles`);
  } catch (error) {
    console.error("Failed to load profiles:", error);
  }
}

// Select consensus profile
async function selectProfile(profileId) {
  try {
    await invoke("set_active_profile", { profileName: profileId });
    logToTerminal(`✅ Selected profile: ${profileId}`);
  } catch (error) {
    logToTerminal(`❌ Error selecting profile: ${error}`);
  }
}

// Utility function to escape HTML
function escapeHtml(text) {
  const map = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  };
  return text.replace(/[&<>"']/g, m => map[m]);
}

// Phase 1: Profile Management Functions
async function changeProfile(profileId) {
  try {
    await invoke("set_active_profile", { profileName: profileId });
    logToTerminal(`✅ Profile changed to: ${profileId}`);
    
    // Update UI to reflect profile change
    const profileName = {
      'speed': '⚡ Speed',
      'balanced': '⚖️ Balanced', 
      'quality': '🎨 Quality',
      'consensus': '🧬 Consensus'
    }[profileId] || profileId;
    
    // Could update consensus behavior here
  } catch (error) {
    logToTerminal(`❌ Error changing profile: ${error}`);
  }
}

// Phase 1: Settings Dialog Functions
function openSettings() {
  const dialog = document.getElementById('settingsDialog');
  dialog.style.display = 'flex';
  
  // Load current API key status
  loadApiKeyStatus();
  
  logToTerminal('⚙️ Settings dialog opened');
}

function closeSettings() {
  const dialog = document.getElementById('settingsDialog');
  dialog.style.display = 'none';
  logToTerminal('⚙️ Settings dialog closed');
}

function switchSettingsTab(tabName) {
  // Update tab buttons
  document.querySelectorAll('.settings-tab').forEach(tab => {
    tab.classList.remove('active');
  });
  event.target.classList.add('active');
  
  // Update tab content
  document.querySelectorAll('.settings-tab-content').forEach(content => {
    content.style.display = 'none';
  });
  
  const tabId = tabName.replace('-', '') + 'Tab';
  const tabContent = document.getElementById(tabId);
  if (tabContent) {
    tabContent.style.display = 'block';
  }
}

async function loadApiKeyStatus() {
  try {
    const status = await invoke('get_api_keys_status');
    
    // Update OpenRouter status
    const openrouterStatus = document.getElementById('openrouterStatus');
    if (status.openrouter.configured) {
      openrouterStatus.textContent = '✅ Configured';
      openrouterStatus.style.color = 'var(--success)';
    } else {
      openrouterStatus.textContent = '❌ Not configured';
      openrouterStatus.style.color = 'var(--error)';
    }
    
    // Update Anthropic status
    const anthropicStatus = document.getElementById('anthropicStatus');
    if (status.anthropic.configured) {
      anthropicStatus.textContent = '✅ Configured';
      anthropicStatus.style.color = 'var(--success)';
    } else {
      anthropicStatus.textContent = '❌ Not configured';
      anthropicStatus.style.color = 'var(--error)';
    }
  } catch (error) {
    console.error('Failed to load API key status:', error);
  }
}

async function validateApiKey(provider) {
  const inputId = provider + 'Key';
  const statusId = provider + 'Status';
  
  const input = document.getElementById(inputId);
  const status = document.getElementById(statusId);
  
  const apiKey = input.value.trim();
  
  if (!apiKey) {
    status.textContent = '⚠️ Please enter an API key';
    status.style.color = 'var(--warning)';
    return;
  }
  
  status.textContent = '🔄 Validating...';
  status.style.color = 'var(--text-muted)';
  
  try {
    // First validate the format and test the key
    const isValid = await invoke('validate_api_key', { 
      provider: provider,
      apiKey: apiKey
    });
    
    if (isValid) {
      // Save the validated key
      await invoke('save_api_key_secure', {
        provider: provider,
        apiKey: apiKey
      });
      
      status.textContent = '✅ Valid and saved';
      status.style.color = 'var(--success)';
      logToTerminal(`✅ ${provider} API key validated and saved`);
      
      // Clear the input for security
      input.value = '';
    } else {
      status.textContent = '❌ Invalid API key';
      status.style.color = 'var(--error)';
    }
  } catch (error) {
    status.textContent = '❌ Validation failed';
    status.style.color = 'var(--error)';
    logToTerminal(`❌ Error validating ${provider} key: ${error}`);
  }
}

// Phase 1: Git/LazyGit Integration
let lazyGitTerminalId = null;

async function openGit() {
  // Switch to git view
  currentView = 'git';
  
  // Update button states
  document.querySelectorAll('.view-toggle-btn').forEach(btn => {
    btn.classList.remove('active');
  });
  document.querySelector('[onclick*="openGit"]').classList.add('active');
  
  // Update explorer header
  const sectionTitle = document.querySelector('.sidebar-section-title');
  sectionTitle.textContent = 'SOURCE CONTROL';
  
  // Load git status
  await loadGitView();
}

async function loadGitView() {
  const fileTree = document.getElementById('fileTree');
  
  try {
    // Get git status from backend
    const status = await invoke('get_git_status', { path: '.' });
    
    let html = '<div style="padding: 10px;">';
    
    // Current branch
    html += `<div class="git-branch">🔀 ${status.branch}</div>`;
    
    // Show ahead/behind if applicable
    if (status.ahead > 0 || status.behind > 0) {
      html += `<div class="git-sync">↑${status.ahead} ↓${status.behind}</div>`;
    }
    
    // Staged changes
    if (status.staged.length > 0) {
      html += '<div class="git-section">📦 Staged Changes</div>';
      for (const file of status.staged) {
        html += `<div class="git-file staged">${getGitStatusIcon(file.status)} ${file.path}</div>`;
      }
    }
    
    // Unstaged changes
    if (status.unstaged.length > 0) {
      html += '<div class="git-section">📝 Changes</div>';
      for (const file of status.unstaged) {
        html += `<div class="git-file modified">${getGitStatusIcon(file.status)} ${file.path}</div>`;
      }
    }
    
    // Untracked files
    if (status.untracked.length > 0) {
      html += '<div class="git-section">❓ Untracked</div>';
      for (const file of status.untracked) {
        html += `<div class="git-file untracked">U ${file.path}</div>`;
      }
    }
    
    // LazyGit button
    html += `
      <div style="margin-top: 20px;">
        <button class="btn" onclick="openLazyGit()" style="width: 100%;">
          🚀 Open LazyGit
        </button>
      </div>
    `;
    
    html += '</div>';
    fileTree.innerHTML = html;
    
    logToTerminal(`🔀 Git status loaded - Branch: ${status.branch}`);
  } catch (error) {
    fileTree.innerHTML = `
      <div style="padding: 10px; color: var(--text-muted);">
        <p>Not a git repository</p>
        <button class="btn" onclick="initGitRepo()" style="margin-top: 10px;">
          Initialize Git Repository
        </button>
      </div>
    `;
    logToTerminal('ℹ️ Not in a git repository');
  }
}

function getGitStatusIcon(status) {
  const icons = {
    'added': 'A',
    'modified': 'M',
    'deleted': 'D',
    'renamed': 'R'
  };
  return icons[status] || '?';
}

async function openLazyGit() {
  try {
    logToTerminal('🚀 Starting LazyGit...');
    
    // Create LazyGit terminal
    const terminalInfo = await invoke('create_lazygit_terminal', { 
      path: '.',
      window: window.__TAURI__.window.getCurrent()
    });
    
    lazyGitTerminalId = terminalInfo.id;
    
    // Switch to git panel if not already there
    const gitPanel = document.getElementById('gitPanel');
    if (gitPanel) {
      gitPanel.innerHTML = `
        <div class="lazygit-container" style="height: 100%; background: #000;">
          <div class="lazygit-header" style="padding: 10px; background: var(--hive-dark-bg-secondary); border-bottom: 1px solid var(--border-color);">
            <span style="color: var(--hive-yellow);">🚀 LazyGit Terminal</span>
            <button onclick="closeLazyGit()" style="float: right; background: transparent; border: none; color: var(--text-muted); cursor: pointer;">✕</button>
          </div>
          <div id="lazygitTerminal" style="height: calc(100% - 40px); padding: 10px; font-family: 'Consolas', monospace; overflow-y: auto;">
            <div style="color: #0f0;">Connecting to LazyGit...</div>
          </div>
        </div>
      `;
    }
    
    // Listen for terminal output
    const unlisten = await window.__TAURI__.event.listen('terminal-output', (event) => {
      if (event.payload.id === lazyGitTerminalId && event.payload.is_lazygit) {
        const terminal = document.getElementById('lazygitTerminal');
        if (terminal) {
          // Append output (could be enhanced with proper terminal emulation)
          terminal.innerHTML += `<div>${escapeHtml(event.payload.data)}</div>`;
          terminal.scrollTop = terminal.scrollHeight;
        }
      }
    });
    
    // Store unlisten function for cleanup
    window.lazyGitUnlisten = unlisten;
    
    logToTerminal('✅ LazyGit terminal created');
  } catch (error) {
    logToTerminal(`❌ Failed to start LazyGit: ${error}`);
    alert(`Failed to start LazyGit: ${error}`);
  }
}

async function closeLazyGit() {
  if (lazyGitTerminalId) {
    try {
      await invoke('close_terminal', { id: lazyGitTerminalId });
      lazyGitTerminalId = null;
      
      // Clean up event listener
      if (window.lazyGitUnlisten) {
        window.lazyGitUnlisten();
        window.lazyGitUnlisten = null;
      }
      
      // Restore git view
      await loadGitView();
      
      logToTerminal('✅ LazyGit terminal closed');
    } catch (error) {
      console.error('Failed to close LazyGit:', error);
    }
  }
}

async function initGitRepo() {
  try {
    // Use shell command to initialize git repo
    await invoke('shell_execute', { command: 'git init' });
    logToTerminal('✅ Git repository initialized');
    
    // Reload git view
    await loadGitView();
  } catch (error) {
    logToTerminal(`❌ Failed to initialize git: ${error}`);
  }
}

// Add CSS for git-specific styling
const style = document.createElement('style');
style.textContent = `
  .git-branch {
    color: var(--hive-yellow);
    font-weight: 600;
    margin-bottom: 10px;
  }
  
  .git-sync {
    color: var(--text-muted);
    font-size: 12px;
    margin-bottom: 15px;
  }
  
  .git-section {
    color: var(--text-secondary);
    font-weight: 600;
    margin: 15px 0 5px 0;
    font-size: 12px;
    text-transform: uppercase;
  }
  
  .git-file {
    padding: 3px 0;
    padding-left: 15px;
    font-size: 13px;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .git-file:hover {
    background-color: rgba(255, 193, 7, 0.1);
  }
  
  .git-file.staged {
    color: #4CAF50;
  }
  
  .git-file.modified {
    color: #FFC107;
  }
  
  .git-file.untracked {
    color: #9E9E9E;
  }
  
  .lazygit-container {
    display: flex;
    flex-direction: column;
  }
`;
document.head.appendChild(style);

// Export functions for global access
window.switchSidebarView = switchSidebarView;
window.toggleTerminal = toggleTerminal;
window.toggleAutoAccept = toggleAutoAccept;
window.runConsensus = runConsensus;
window.cancelConsensus = cancelConsensus;
window.handleFileClick = handleFileClick;
window.selectProfile = selectProfile;
window.showAnalyticsView = showAnalyticsView;

// Phase 1 exports
window.changeProfile = changeProfile;
window.openSettings = openSettings;
window.closeSettings = closeSettings;
window.switchSettingsTab = switchSettingsTab;
window.validateApiKey = validateApiKey;
window.openGit = openGit;
window.openLazyGit = openLazyGit;
window.closeLazyGit = closeLazyGit;
window.initGitRepo = initGitRepo;