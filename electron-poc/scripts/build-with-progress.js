#!/usr/bin/env node

/**
 * Build with Progress Window
 * Launches a visual progress window and runs the build
 */

const { spawn } = require('child_process');
const WebSocket = require('ws');
const path = require('path');
const fs = require('fs');

class BuildWithProgress {
  constructor() {
    this.progressWindow = null;
    this.buildProcess = null;
    this.ws = null;
    this.currentPhase = 0;
    this.totalPhases = 15;
    this.phaseStartTime = Date.now();
    this.phases = [
      'Environment Setup',
      'Version Verification',
      'Dependency Check',
      'Native Module Verification',
      'Runtime Dependencies Discovery',
      'Webpack Configuration',
      'Pre-build Scripts',
      'Webpack Compilation',
      'Memory Service Build',
      'Backend Server Build',
      'Resource Copying',
      'Binary Permissions',
      'Package Creation',
      'DMG Building',
      'Installation'
    ];
  }

  async start() {
    console.log('🚀 Starting build with progress window...');
    
    // Launch progress window first
    await this.launchProgressWindow();
    
    // Wait a moment for window to initialize
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Connect to progress window
    await this.connectToProgressWindow();
    
    // Start the build
    await this.startBuild();
  }

  async launchProgressWindow() {
    console.log('📊 Launching progress window...');
    
    const electronPath = require('electron');
    const windowScript = path.join(__dirname, 'build-progress-window.js');
    
    this.progressWindow = spawn(electronPath, [windowScript], {
      stdio: 'inherit',
      // Launch Electron as a GUI app (do NOT set ELECTRON_RUN_AS_NODE)
      env: { ...process.env }
    });
    
    this.progressWindow.on('error', (error) => {
      console.error('Failed to launch progress window:', error);
    });
    
    this.progressWindow.on('exit', (code) => {
      console.log('Progress window exited with code:', code);
    });
  }

  async connectToProgressWindow() {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket('ws://localhost:9999');
      
      this.ws.on('open', () => {
        console.log('✅ Connected to progress window');
        resolve();
      });
      
      this.ws.on('error', (error) => {
        console.error('Failed to connect to progress window:', error);
        reject(error);
      });
      
      this.ws.on('close', () => {
        console.log('Disconnected from progress window');
      });
    });
  }

  sendUpdate(type, data) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({ type, ...data }));
    }
  }

  async startBuild() {
    console.log('🔨 Starting production build...');
    
    this.sendUpdate('phase-start', {
      phase: 0,
      name: 'Environment Setup'
    });
    
    // Run the actual build script
    const buildScript = path.join(__dirname, 'build-production-dmg.js');
    
    this.buildProcess = spawn('node', [buildScript], {
      stdio: 'pipe',
      env: { ...process.env, SHOW_PROGRESS: 'false' } // Disable console progress
    });
    
    // Parse build output and send updates
    this.buildProcess.stdout.on('data', (data) => {
      const output = data.toString();
      console.log(output);
      
      // Parse phase changes
      if (output.includes('Phase')) {
        const phaseMatch = output.match(/Phase (\d+)(?:\.(\d+))?:\s*(.+)/);
        if (phaseMatch) {
          const phaseNum = parseInt(phaseMatch[1]);
          const phaseName = phaseMatch[3];
          
          // Complete previous phase
          if (this.currentPhase > 0) {
            this.sendUpdate('phase-complete', {
              phase: this.currentPhase,
              name: this.phases[this.currentPhase - 1],
              duration: Date.now() - this.phaseStartTime
            });
          }
          
          // Start new phase
          this.currentPhase = phaseNum;
          this.phaseStartTime = Date.now();
          
          this.sendUpdate('phase-start', {
            phase: phaseNum,
            name: phaseName
          });
        }
      }
      
      // Send log updates
      this.sendUpdate('log', {
        message: output.trim()
      });
      
      // Parse errors
      if (output.includes('Error') || output.includes('Failed')) {
        this.sendUpdate('error', {
          message: output.trim()
        });
      }
      
      // Parse warnings
      if (output.includes('Warning')) {
        this.sendUpdate('warning', {
          message: output.trim()
        });
      }
      
      // Check for specific progress indicators
      if (output.includes('✓') || output.includes('Success')) {
        this.sendUpdate('log', {
          message: '✅ ' + output.trim()
        });
      }
    });
    
    this.buildProcess.stderr.on('data', (data) => {
      const error = data.toString();
      console.error(error);
      
      this.sendUpdate('error', {
        message: error.trim()
      });
    });
    
    this.buildProcess.on('exit', (code) => {
      console.log(`Build process exited with code: ${code}`);
      
      // Send completion
      this.sendUpdate('build-complete', {
        success: code === 0,
        exitCode: code
      });
      
      // Close WebSocket after a delay
      setTimeout(() => {
        if (this.ws) {
          this.ws.close();
        }
      }, 10000);
    });
  }
}

// Run if executed directly
if (require.main === module) {
  const builder = new BuildWithProgress();
  builder.start().catch(error => {
    console.error('Build failed:', error);
    process.exit(1);
  });
}

module.exports = BuildWithProgress;
