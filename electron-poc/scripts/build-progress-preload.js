/**
 * Preload script for build progress window
 * Provides secure bridge between renderer and main process
 */

const { contextBridge, ipcRenderer } = require('electron');

// Expose protected API to renderer
contextBridge.exposeInMainWorld('buildAPI', {
  getBuildStatus: () => ipcRenderer.invoke('get-build-status'),
  closeWindow: () => ipcRenderer.send('close-window'),
  onBuildUpdate: (callback) => {
    ipcRenderer.on('build-update', (event, data) => callback(data));
  }
});

// Initialize renderer when DOM is ready
window.addEventListener('DOMContentLoaded', () => {
  const elements = {
    phaseName: document.getElementById('phaseName'),
    phaseCounter: document.getElementById('phaseCounter'),
    progressBar: document.getElementById('progressBar'),
    progressText: document.getElementById('progressText'),
    errorCount: document.getElementById('errorCount'),
    warningCount: document.getElementById('warningCount'),
    elapsedTime: document.getElementById('elapsedTime'),
    logContainer: document.getElementById('logContainer'),
    statusIndicator: document.getElementById('statusIndicator'),
    statusText: document.getElementById('statusText'),
    closeButton: document.getElementById('closeButton')
  };

  let startTime = Date.now();

  // Update elapsed time every second
  setInterval(() => {
    const elapsed = Date.now() - startTime;
    const minutes = Math.floor(elapsed / 60000);
    const seconds = Math.floor((elapsed % 60000) / 1000);
    elements.elapsedTime.textContent = `${minutes}:${seconds.toString().padStart(2, '0')}`;
  }, 1000);

  // Handle build updates
  window.buildAPI.onBuildUpdate((data) => {
    // Update phase info
    elements.phaseName.textContent = data.phaseName || 'Processing...';
    elements.phaseCounter.textContent = `Phase ${data.currentPhase} of ${data.totalPhases}`;
    
    // Update progress bar
    const progress = Math.round(data.progress || 0);
    elements.progressBar.style.width = `${progress}%`;
    elements.progressText.textContent = `${progress}%`;
    
    // Update stats
    elements.errorCount.textContent = data.errors?.length || 0;
    elements.warningCount.textContent = data.warnings?.length || 0;
    
    // Update logs
    if (data.logs && data.logs.length > 0) {
      elements.logContainer.innerHTML = '';
      const latestLogs = data.logs.slice(-50); // Show last 50 logs
      
      latestLogs.forEach(log => {
        const entry = document.createElement('div');
        entry.className = 'log-entry';
        
        if (log.includes('✅')) {
          entry.classList.add('success');
        } else if (log.includes('Error') || log.includes('Failed')) {
          entry.classList.add('error');
        } else if (log.includes('Warning')) {
          entry.classList.add('warning');
        }
        
        entry.textContent = log;
        elements.logContainer.appendChild(entry);
      });
      
      // Auto-scroll to bottom
      elements.logContainer.scrollTop = elements.logContainer.scrollHeight;
    }
    
    // Update status
    if (data.status === 'complete') {
      elements.statusIndicator.className = 'status-indicator complete';
      elements.statusText.textContent = 'Build Complete!';
      elements.closeButton.disabled = false;
      
      // Add completion animation
      elements.progressBar.style.background = 'linear-gradient(90deg, #44FF44, #00FF00)';
      
      // Play completion sound if available
      const audio = new Audio('data:audio/wav;base64,UklGRg=='); // Add actual sound data
      audio.play().catch(() => {});
      
    } else if (data.errors && data.errors.length > 0) {
      elements.statusIndicator.className = 'status-indicator error';
      elements.statusText.textContent = 'Build Failed';
      elements.closeButton.disabled = false;
      elements.progressBar.style.background = 'linear-gradient(90deg, #FF4444, #CC0000)';
    } else {
      elements.statusIndicator.className = 'status-indicator';
      elements.statusText.textContent = 'Building...';
    }
  });

  // Handle close button
  elements.closeButton.addEventListener('click', () => {
    window.buildAPI.closeWindow();
  });

  // Get initial status
  window.buildAPI.getBuildStatus().then(data => {
    if (data) {
      window.buildAPI.onBuildUpdate(data);
    }
  });
});