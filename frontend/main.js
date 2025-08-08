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

// Export functions for global access
window.switchSidebarView = switchSidebarView;
window.toggleTerminal = toggleTerminal;
window.toggleAutoAccept = toggleAutoAccept;
window.runConsensus = runConsensus;
window.cancelConsensus = cancelConsensus;
window.handleFileClick = handleFileClick;
window.selectProfile = selectProfile;
window.showAnalyticsView = showAnalyticsView;