#!/usr/bin/env node

/**
 * Real-Time Build Monitor for Hive Consensus
 * Provides live streaming updates during build process
 */

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const WebSocket = require('ws');
const http = require('http');

// Colors for terminal output
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const MAGENTA = '\x1b[35m';
const CYAN = '\x1b[36m';
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';
const CLEAR = '\x1b[2J\x1b[H';

class RealtimeBuildMonitor {
  constructor(options = {}) {
    this.port = options.port || 8888;
    this.buildLogDir = path.join(__dirname, '../build-logs');
    this.logFile = null;
    this.wsServer = null;
    this.clients = new Set();
    this.buildData = {
      buildId: null,
      startTime: null,
      currentPhase: null,
      phases: [],
      phaseDetails: {},
      errors: [],
      warnings: [],
      metrics: {},
      liveOutput: [],
      status: 'IDLE'
    };
    
    // Ensure log directory exists
    if (!fs.existsSync(this.buildLogDir)) {
      fs.mkdirSync(this.buildLogDir, { recursive: true });
    }
  }

  // Start WebSocket server for real-time updates
  startServer() {
    const server = http.createServer();
    this.wsServer = new WebSocket.Server({ server });
    
    this.wsServer.on('connection', (ws) => {
      console.log(`${GREEN}[MONITOR] New client connected${RESET}`);
      this.clients.add(ws);
      
      // Send current state to new client
      ws.send(JSON.stringify({
        type: 'STATE_UPDATE',
        data: this.buildData
      }));
      
      ws.on('close', () => {
        this.clients.delete(ws);
        console.log(`${YELLOW}[MONITOR] Client disconnected${RESET}`);
      });
    });
    
    server.listen(this.port, () => {
      console.log(`${CYAN}${BOLD}Real-Time Build Monitor started on port ${this.port}${RESET}`);
      console.log(`${CYAN}Connect with: ws://localhost:${this.port}${RESET}\n`);
    });
  }
  
  // Broadcast update to all connected clients
  broadcast(message) {
    const data = JSON.stringify(message);
    this.clients.forEach(client => {
      if (client.readyState === WebSocket.OPEN) {
        client.send(data);
      }
    });
  }
  
  // Start monitoring a new build
  startBuild(buildId) {
    this.buildData = {
      buildId: buildId || new Date().toISOString(),
      startTime: Date.now(),
      currentPhase: null,
      phases: [],
      phaseDetails: {},
      errors: [],
      warnings: [],
      metrics: {},
      liveOutput: [],
      status: 'IN_PROGRESS'
    };
    
    // Create log file
    this.logFile = path.join(this.buildLogDir, `realtime-${this.buildData.buildId}.log`);
    
    this.broadcast({
      type: 'BUILD_START',
      data: this.buildData
    });
    
    this.log(`Build started: ${buildId}`, 'INFO');
  }
  
  // Start a new phase
  startPhase(name, description) {
    const phase = {
      name,
      description,
      startTime: Date.now(),
      status: 'IN_PROGRESS',
      logs: [],
      progress: 0
    };
    
    this.buildData.currentPhase = name;
    this.buildData.phases.push(name);
    this.buildData.phaseDetails[name] = phase;
    
    this.broadcast({
      type: 'PHASE_START',
      data: { phase: name, description }
    });
    
    this.displayDashboard();
  }
  
  // Update phase progress
  updatePhaseProgress(phaseName, progress, message) {
    if (!this.buildData.phaseDetails[phaseName]) return;
    
    this.buildData.phaseDetails[phaseName].progress = progress;
    
    if (message) {
      this.buildData.phaseDetails[phaseName].logs.push({
        timestamp: Date.now(),
        message
      });
    }
    
    this.broadcast({
      type: 'PHASE_PROGRESS',
      data: { phase: phaseName, progress, message }
    });
    
    this.displayDashboard();
  }
  
  // Complete a phase
  completePhase(phaseName, status = 'SUCCESS') {
    if (!this.buildData.phaseDetails[phaseName]) return;
    
    const phase = this.buildData.phaseDetails[phaseName];
    phase.status = status;
    phase.endTime = Date.now();
    phase.duration = phase.endTime - phase.startTime;
    
    this.broadcast({
      type: 'PHASE_COMPLETE',
      data: { phase: phaseName, status, duration: phase.duration }
    });
    
    this.displayDashboard();
  }
  
  // Log a message
  log(message, level = 'INFO', phase = null) {
    const entry = {
      timestamp: Date.now(),
      level,
      message,
      phase: phase || this.buildData.currentPhase
    };
    
    // Add to appropriate collection
    if (level === 'ERROR') {
      this.buildData.errors.push(entry);
    } else if (level === 'WARNING') {
      this.buildData.warnings.push(entry);
    }
    
    // Add to live output (keep last 100 entries)
    this.buildData.liveOutput.push(entry);
    if (this.buildData.liveOutput.length > 100) {
      this.buildData.liveOutput.shift();
    }
    
    // Write to file
    if (this.logFile) {
      fs.appendFileSync(this.logFile, `[${new Date().toISOString()}] [${level}] ${message}\n`);
    }
    
    this.broadcast({
      type: 'LOG',
      data: entry
    });
    
    // Display in console with color
    const color = level === 'ERROR' ? RED : 
                 level === 'WARNING' ? YELLOW : 
                 level === 'SUCCESS' ? GREEN : RESET;
    console.log(`${color}[${level}] ${message}${RESET}`);
  }
  
  // Track a metric
  trackMetric(name, value, unit = '') {
    this.buildData.metrics[name] = {
      value,
      unit,
      timestamp: Date.now()
    };
    
    this.broadcast({
      type: 'METRIC',
      data: { name, value, unit }
    });
  }
  
  // Display real-time dashboard in terminal
  displayDashboard() {
    if (!this.buildData.buildId) return;
    
    const elapsed = Date.now() - this.buildData.startTime;
    const elapsedMin = Math.floor(elapsed / 60000);
    const elapsedSec = Math.floor((elapsed % 60000) / 1000);
    
    console.log(CLEAR);
    console.log(`${CYAN}${BOLD}═══════════════════════════════════════════════════════════════${RESET}`);
    console.log(`${BLUE}${BOLD}               HIVE CONSENSUS BUILD MONITOR${RESET}`);
    console.log(`${CYAN}═══════════════════════════════════════════════════════════════${RESET}\n`);
    
    console.log(`${BOLD}Build ID:${RESET}      ${this.buildData.buildId}`);
    console.log(`${BOLD}Elapsed:${RESET}       ${elapsedMin}m ${elapsedSec}s`);
    console.log(`${BOLD}Current Phase:${RESET} ${this.buildData.currentPhase || 'None'}`);
    console.log(`${BOLD}Status:${RESET}        ${this.getStatusIcon(this.buildData.status)} ${this.buildData.status}`);
    console.log();
    
    // Phase progress
    console.log(`${CYAN}${BOLD}PHASES:${RESET}`);
    this.buildData.phases.forEach(phaseName => {
      const phase = this.buildData.phaseDetails[phaseName];
      const icon = this.getStatusIcon(phase.status);
      const progress = phase.progress || 0;
      const progressBar = this.createProgressBar(progress);
      
      console.log(`  ${icon} ${phaseName}`);
      if (phase.status === 'IN_PROGRESS') {
        console.log(`     ${progressBar} ${progress}%`);
      } else if (phase.duration) {
        console.log(`     Duration: ${(phase.duration / 1000).toFixed(2)}s`);
      }
    });
    
    console.log();
    
    // Metrics
    if (Object.keys(this.buildData.metrics).length > 0) {
      console.log(`${CYAN}${BOLD}METRICS:${RESET}`);
      Object.entries(this.buildData.metrics).forEach(([name, data]) => {
        console.log(`  ${name}: ${data.value}${data.unit}`);
      });
      console.log();
    }
    
    // Errors and Warnings
    console.log(`${BOLD}Errors:${RESET}   ${this.buildData.errors.length > 0 ? RED : GREEN}${this.buildData.errors.length}${RESET}`);
    console.log(`${BOLD}Warnings:${RESET} ${this.buildData.warnings.length > 0 ? YELLOW : GREEN}${this.buildData.warnings.length}${RESET}`);
    
    // Recent logs
    console.log(`\n${CYAN}${BOLD}RECENT ACTIVITY:${RESET}`);
    const recentLogs = this.buildData.liveOutput.slice(-5);
    recentLogs.forEach(log => {
      const color = log.level === 'ERROR' ? RED : 
                   log.level === 'WARNING' ? YELLOW : 
                   log.level === 'SUCCESS' ? GREEN : RESET;
      const time = new Date(log.timestamp).toLocaleTimeString();
      console.log(`  ${time} ${color}[${log.level}]${RESET} ${log.message}`);
    });
    
    console.log(`\n${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
  }
  
  // Create progress bar
  createProgressBar(percentage) {
    const width = 20;
    const filled = Math.floor(width * percentage / 100);
    const empty = width - filled;
    return `[${GREEN}${'█'.repeat(filled)}${RESET}${'░'.repeat(empty)}]`;
  }
  
  // Get status icon
  getStatusIcon(status) {
    switch (status) {
      case 'SUCCESS': return `${GREEN}✓${RESET}`;
      case 'FAILED': return `${RED}✗${RESET}`;
      case 'WARNING': return `${YELLOW}⚠${RESET}`;
      case 'IN_PROGRESS': return `${BLUE}●${RESET}`;
      default: return '○';
    }
  }
  
  // Complete the build
  completeBuild(status = 'SUCCESS') {
    this.buildData.status = status;
    this.buildData.endTime = Date.now();
    this.buildData.duration = this.buildData.endTime - this.buildData.startTime;
    
    this.broadcast({
      type: 'BUILD_COMPLETE',
      data: {
        status,
        duration: this.buildData.duration,
        errors: this.buildData.errors.length,
        warnings: this.buildData.warnings.length
      }
    });
    
    // Save final report
    const reportFile = path.join(this.buildLogDir, `report-${this.buildData.buildId}.json`);
    fs.writeFileSync(reportFile, JSON.stringify(this.buildData, null, 2));
    
    this.displayDashboard();
    
    console.log(`\n${GREEN}${BOLD}Build complete! Report saved to: ${reportFile}${RESET}\n`);
  }
  
  // Monitor a build script execution
  async monitorBuildScript(scriptPath, args = []) {
    const buildId = new Date().toISOString();
    this.startBuild(buildId);
    
    return new Promise((resolve, reject) => {
      const child = spawn('node', [scriptPath, ...args], {
        env: { ...process.env, REALTIME_MONITOR_PORT: this.port }
      });
      
      child.stdout.on('data', (data) => {
        const lines = data.toString().split('\n');
        lines.forEach(line => {
          if (line.trim()) {
            this.processScriptOutput(line);
          }
        });
      });
      
      child.stderr.on('data', (data) => {
        this.log(data.toString(), 'ERROR');
      });
      
      child.on('close', (code) => {
        const status = code === 0 ? 'SUCCESS' : 'FAILED';
        this.completeBuild(status);
        
        if (code === 0) {
          resolve(this.buildData);
        } else {
          reject(new Error(`Build failed with code ${code}`));
        }
      });
    });
  }
  
  // Process output from build script
  processScriptOutput(line) {
    // Parse special markers
    if (line.includes('[PHASE_START]')) {
      const match = line.match(/\[PHASE_START\] (.+?) - (.+)/);
      if (match) {
        this.startPhase(match[1], match[2]);
        return;
      }
    }
    
    if (line.includes('[PHASE_PROGRESS]')) {
      const match = line.match(/\[PHASE_PROGRESS\] (.+?) - (\d+)% - (.+)/);
      if (match) {
        this.updatePhaseProgress(match[1], parseInt(match[2]), match[3]);
        return;
      }
    }
    
    if (line.includes('[PHASE_COMPLETE]')) {
      const match = line.match(/\[PHASE_COMPLETE\] (.+?) - (.+)/);
      if (match) {
        this.completePhase(match[1], match[2]);
        return;
      }
    }
    
    if (line.includes('[METRIC]')) {
      const match = line.match(/\[METRIC\] (.+?): (.+)/);
      if (match) {
        this.trackMetric(match[1], match[2]);
        return;
      }
    }
    
    // Default log
    const level = line.includes('ERROR') ? 'ERROR' :
                  line.includes('WARNING') ? 'WARNING' :
                  line.includes('SUCCESS') || line.includes('✓') ? 'SUCCESS' : 'INFO';
    
    this.log(line, level);
  }
}

// Export for use in build scripts
module.exports = RealtimeBuildMonitor;

// CLI interface
if (require.main === module) {
  const command = process.argv[2];
  const monitor = new RealtimeBuildMonitor();
  
  if (command === 'serve') {
    monitor.startServer();
    console.log(`${CYAN}Waiting for build connections...${RESET}`);
  } else if (command === 'watch') {
    const scriptPath = process.argv[3];
    if (!scriptPath) {
      console.error(`${RED}Usage: realtime-build-monitor watch <script-path>${RESET}`);
      process.exit(1);
    }
    
    monitor.startServer();
    setTimeout(() => {
      monitor.monitorBuildScript(scriptPath, process.argv.slice(4))
        .then(() => {
          console.log(`${GREEN}Build monitoring complete${RESET}`);
          process.exit(0);
        })
        .catch(err => {
          console.error(`${RED}Build monitoring failed: ${err.message}${RESET}`);
          process.exit(1);
        });
    }, 1000);
  } else {
    console.log(`${CYAN}${BOLD}Real-Time Build Monitor${RESET}`);
    console.log(`\nUsage:`);
    console.log(`  ${GREEN}serve${RESET}  - Start monitoring server`);
    console.log(`  ${GREEN}watch${RESET}  - Monitor a build script execution`);
    console.log(`\nExample:`);
    console.log(`  realtime-build-monitor watch ./scripts/build-production-dmg.js`);
  }
}