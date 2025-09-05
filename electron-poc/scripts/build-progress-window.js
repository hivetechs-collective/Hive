#!/usr/bin/env node

/**
 * Build Progress Popup Window
 * Shows real-time build progress in a visual popup window
 */

const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const fs = require('fs');
const WebSocket = require('ws');

class BuildProgressWindow {
  constructor() {
    this.window = null;
    this.wsServer = null;
    this.buildData = {
      currentPhase: 0,
      totalPhases: 15,
      phaseName: 'Initializing...',
      progress: 0,
      logs: [],
      errors: [],
      warnings: [],
      startTime: Date.now(),
      status: 'running'
    };
  }

  async start() {
    // Initialize Electron app
    await app.whenReady();
    
    // Create WebSocket server for build script communication
    this.createWebSocketServer();
    
    // Create the window
    this.createWindow();
    
    // Setup IPC handlers
    this.setupIPC();
  }

  createWindow() {
    this.window = new BrowserWindow({
      width: 600,
      height: 800,
      minWidth: 500,
      minHeight: 600,
      webPreferences: {
        nodeIntegration: false,
        contextIsolation: true,
        preload: path.join(__dirname, 'build-progress-preload.js')
      },
      title: 'Hive Consensus Build Progress',
      center: true,
      resizable: true,
      minimizable: true,
      maximizable: false,
      alwaysOnTop: true,
      frame: true,
      transparent: false,
      backgroundColor: '#1e1e1e',
      icon: path.join(__dirname, '..', 'resources', 'icon.png'),
      show: false
    });

    // Load the HTML
    this.window.loadFile(path.join(__dirname, 'build-progress.html'));

    // Show when ready
    this.window.once('ready-to-show', () => {
      this.window.show();
    });

    // Prevent closing during build
    this.window.on('close', (e) => {
      if (this.buildData.status === 'running') {
        e.preventDefault();
        // Minimize instead
        this.window.minimize();
      }
    });
  }

  createWebSocketServer() {
    this.wsServer = new WebSocket.Server({ port: 9999 });
    
    this.wsServer.on('connection', (ws) => {
      console.log('[BuildProgress] Build script connected');
      
      ws.on('message', (message) => {
        try {
          const data = JSON.parse(message.toString());
          this.handleBuildUpdate(data);
        } catch (error) {
          console.error('[BuildProgress] Error parsing message:', error);
        }
      });
      
      // Send initial state
      ws.send(JSON.stringify({ type: 'connected', data: this.buildData }));
    });
  }

  handleBuildUpdate(update) {
    switch (update.type) {
      case 'phase-start':
        this.buildData.currentPhase = update.phase;
        this.buildData.phaseName = update.name;
        this.buildData.progress = (update.phase / this.buildData.totalPhases) * 100;
        break;
        
      case 'phase-complete':
        this.buildData.logs.push(`✅ ${update.name} completed in ${update.duration}ms`);
        break;
        
      case 'log':
        this.buildData.logs.push(update.message);
        if (this.buildData.logs.length > 100) {
          this.buildData.logs.shift(); // Keep only last 100 logs
        }
        break;
        
      case 'error':
        this.buildData.errors.push(update.message);
        break;
        
      case 'warning':
        this.buildData.warnings.push(update.message);
        break;
        
      case 'build-complete':
        this.buildData.status = 'complete';
        this.buildData.progress = 100;
        this.showCompletionNotification(update.success);
        break;
    }
    
    // Send update to renderer
    if (this.window && !this.window.isDestroyed()) {
      this.window.webContents.send('build-update', this.buildData);
    }
  }

  setupIPC() {
    ipcMain.handle('get-build-status', () => {
      return this.buildData;
    });
    
    ipcMain.on('close-window', () => {
      if (this.buildData.status !== 'running') {
        this.cleanup();
      }
    });
  }

  showCompletionNotification(success) {
    const { Notification } = require('electron');
    
    new Notification({
      title: success ? '✅ Build Complete!' : '❌ Build Failed',
      body: success 
        ? `Build completed successfully in ${this.getElapsedTime()}`
        : `Build failed with ${this.buildData.errors.length} errors`,
      icon: path.join(__dirname, '..', 'resources', 'icon.png')
    }).show();
    
    // Allow window to close after 5 seconds
    setTimeout(() => {
      this.buildData.status = 'done';
    }, 5000);
  }

  getElapsedTime() {
    const elapsed = Date.now() - this.buildData.startTime;
    const minutes = Math.floor(elapsed / 60000);
    const seconds = Math.floor((elapsed % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  }

  cleanup() {
    if (this.wsServer) {
      this.wsServer.close();
    }
    if (this.window && !this.window.isDestroyed()) {
      this.window.destroy();
    }
    app.quit();
  }
}

// Start the progress window
const progressWindow = new BuildProgressWindow();
progressWindow.start().catch(console.error);

// Handle app events
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

module.exports = BuildProgressWindow;