#!/usr/bin/env node

/**
 * Build Phase Reporter for Hive Consensus
 * Provides detailed phase-by-phase build status and validation
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Colors for terminal output
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const MAGENTA = '\x1b[35m';
const CYAN = '\x1b[36m';
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';

class BuildPhaseReporter {
  constructor(buildId) {
    this.buildId = buildId || new Date().toISOString().replace(/[:.]/g, '-');
    this.buildLogDir = path.join(__dirname, '../build-logs');
    this.reportFile = path.join(this.buildLogDir, `build-report-${this.buildId}.json`);
    this.statusFile = path.join(this.buildLogDir, 'current-build-status.json');
    this.phases = [];
    this.currentPhase = null;
    this.startTime = Date.now();
    
    // Ensure log directory exists
    if (!fs.existsSync(this.buildLogDir)) {
      fs.mkdirSync(this.buildLogDir, { recursive: true });
    }
    
    // Initialize report
    this.report = {
      buildId: this.buildId,
      startTime: new Date().toISOString(),
      version: null,
      phases: {},
      errors: [],
      warnings: [],
      validations: {},
      performance: {},
      finalStatus: 'IN_PROGRESS'
    };
  }

  // Start a new build phase
  startPhase(phaseName, description) {
    if (this.currentPhase) {
      this.endPhase();
    }
    
    console.log(`\n${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}`);
    console.log(`${BLUE}${BOLD}[PHASE] ${phaseName}${RESET}`);
    console.log(`${CYAN}Description: ${description}${RESET}`);
    console.log(`${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n`);
    
    this.currentPhase = {
      name: phaseName,
      description: description,
      startTime: Date.now(),
      logs: [],
      status: 'IN_PROGRESS',
      validations: []
    };
    
    this.phases.push(phaseName);
    this.report.phases[phaseName] = this.currentPhase;
    this.saveStatus();
  }
  
  // End current phase
  endPhase(status = 'SUCCESS') {
    if (!this.currentPhase) return;
    
    const duration = Date.now() - this.currentPhase.startTime;
    this.currentPhase.endTime = Date.now();
    this.currentPhase.duration = duration;
    this.currentPhase.status = status;
    
    const statusIcon = status === 'SUCCESS' ? '✅' : status === 'WARNING' ? '⚠️' : '❌';
    const statusColor = status === 'SUCCESS' ? GREEN : status === 'WARNING' ? YELLOW : RED;
    
    console.log(`\n${statusColor}${BOLD}[PHASE COMPLETE] ${this.currentPhase.name} ${statusIcon}${RESET}`);
    console.log(`${CYAN}Duration: ${(duration / 1000).toFixed(2)}s${RESET}\n`);
    
    this.currentPhase = null;
    this.saveReport();
  }
  
  // Log a message for current phase
  log(message, level = 'INFO') {
    const timestamp = new Date().toISOString();
    const logEntry = { timestamp, level, message };
    
    if (this.currentPhase) {
      this.currentPhase.logs.push(logEntry);
    }
    
    const color = level === 'ERROR' ? RED : level === 'WARNING' ? YELLOW : level === 'SUCCESS' ? GREEN : RESET;
    console.log(`${color}[${level}] ${message}${RESET}`);
    
    if (level === 'ERROR') {
      this.report.errors.push(logEntry);
    } else if (level === 'WARNING') {
      this.report.warnings.push(logEntry);
    }
    
    this.saveStatus();
  }
  
  // Add validation check
  validate(checkName, condition, message) {
    const validation = {
      name: checkName,
      passed: condition,
      message: message,
      timestamp: new Date().toISOString()
    };
    
    if (this.currentPhase) {
      this.currentPhase.validations.push(validation);
    }
    
    this.report.validations[checkName] = validation;
    
    const icon = condition ? '✓' : '✗';
    const color = condition ? GREEN : RED;
    console.log(`  ${color}[${icon}] ${checkName}: ${message}${RESET}`);
    
    return condition;
  }
  
  // Track performance metric
  trackMetric(metricName, value, unit = '') {
    this.report.performance[metricName] = {
      value: value,
      unit: unit,
      timestamp: new Date().toISOString()
    };
    
    console.log(`${MAGENTA}[METRIC] ${metricName}: ${value}${unit}${RESET}`);
  }
  
  // Save current status for monitoring
  saveStatus() {
    const status = {
      buildId: this.buildId,
      currentPhase: this.currentPhase ? this.currentPhase.name : 'NONE',
      completedPhases: this.phases.filter(p => 
        this.report.phases[p] && this.report.phases[p].status !== 'IN_PROGRESS'
      ),
      totalPhases: this.phases.length,
      errors: this.report.errors.length,
      warnings: this.report.warnings.length,
      elapsed: Date.now() - this.startTime,
      lastUpdate: new Date().toISOString()
    };
    
    fs.writeFileSync(this.statusFile, JSON.stringify(status, null, 2));
  }
  
  // Save full report
  saveReport() {
    fs.writeFileSync(this.reportFile, JSON.stringify(this.report, null, 2));
  }
  
  // Finalize build
  finalize(status = 'SUCCESS') {
    if (this.currentPhase) {
      this.endPhase();
    }
    
    this.report.endTime = new Date().toISOString();
    this.report.totalDuration = Date.now() - this.startTime;
    this.report.finalStatus = status;
    
    // Generate summary
    const summary = this.generateSummary();
    this.report.summary = summary;
    
    this.saveReport();
    this.printSummary(summary);
    
    return summary;
  }
  
  // Generate build summary
  generateSummary() {
    const totalPhases = Object.keys(this.report.phases).length;
    const successfulPhases = Object.values(this.report.phases)
      .filter(p => p.status === 'SUCCESS').length;
    const failedPhases = Object.values(this.report.phases)
      .filter(p => p.status === 'FAILED').length;
    const validationsPassed = Object.values(this.report.validations)
      .filter(v => v.passed).length;
    const totalValidations = Object.keys(this.report.validations).length;
    
    return {
      buildId: this.buildId,
      version: this.report.version,
      duration: `${(this.report.totalDuration / 1000).toFixed(2)}s`,
      phases: {
        total: totalPhases,
        successful: successfulPhases,
        failed: failedPhases
      },
      validations: {
        passed: validationsPassed,
        total: totalValidations
      },
      errors: this.report.errors.length,
      warnings: this.report.warnings.length,
      status: this.report.finalStatus
    };
  }
  
  // Print formatted summary
  printSummary(summary) {
    console.log(`\n${CYAN}${BOLD}════════════════════════════════════════════════════════════${RESET}`);
    console.log(`${BLUE}${BOLD}                    BUILD SUMMARY${RESET}`);
    console.log(`${CYAN}════════════════════════════════════════════════════════════${RESET}\n`);
    
    const statusColor = summary.status === 'SUCCESS' ? GREEN : 
                       summary.status === 'WARNING' ? YELLOW : RED;
    const statusIcon = summary.status === 'SUCCESS' ? '✅' : 
                      summary.status === 'WARNING' ? '⚠️' : '❌';
    
    console.log(`${BOLD}Build ID:${RESET}     ${summary.buildId}`);
    console.log(`${BOLD}Version:${RESET}      ${summary.version || 'N/A'}`);
    console.log(`${BOLD}Duration:${RESET}     ${summary.duration}`);
    console.log(`${BOLD}Status:${RESET}       ${statusColor}${summary.status} ${statusIcon}${RESET}`);
    console.log();
    console.log(`${BOLD}Phases:${RESET}       ${summary.phases.successful}/${summary.phases.total} successful`);
    console.log(`${BOLD}Validations:${RESET}  ${summary.validations.passed}/${summary.validations.total} passed`);
    console.log(`${BOLD}Errors:${RESET}       ${summary.errors > 0 ? RED : GREEN}${summary.errors}${RESET}`);
    console.log(`${BOLD}Warnings:${RESET}     ${summary.warnings > 0 ? YELLOW : GREEN}${summary.warnings}${RESET}`);
    
    console.log(`\n${CYAN}════════════════════════════════════════════════════════════${RESET}\n`);
    
    // Save summary to file
    const summaryFile = path.join(this.buildLogDir, 'latest-build-summary.json');
    fs.writeFileSync(summaryFile, JSON.stringify(summary, null, 2));
    console.log(`${GREEN}Summary saved to: ${summaryFile}${RESET}`);
  }
  
  // Check last build status (static method)
  static checkLastBuild() {
    const buildLogDir = path.join(__dirname, '../build-logs');
    const summaryFile = path.join(buildLogDir, 'latest-build-summary.json');
    
    if (!fs.existsSync(summaryFile)) {
      console.log(`${YELLOW}No previous build found${RESET}`);
      return null;
    }
    
    const summary = JSON.parse(fs.readFileSync(summaryFile, 'utf8'));
    const statusColor = summary.status === 'SUCCESS' ? GREEN : 
                       summary.status === 'WARNING' ? YELLOW : RED;
    
    console.log(`\n${CYAN}${BOLD}═══════════════ LAST BUILD STATUS ═══════════════${RESET}`);
    console.log(`${BOLD}Build ID:${RESET}     ${summary.buildId}`);
    console.log(`${BOLD}Version:${RESET}      ${summary.version || 'N/A'}`);
    console.log(`${BOLD}Status:${RESET}       ${statusColor}${summary.status}${RESET}`);
    console.log(`${BOLD}Duration:${RESET}     ${summary.duration}`);
    console.log(`${BOLD}Phases:${RESET}       ${summary.phases.successful}/${summary.phases.total}`);
    console.log(`${BOLD}Validations:${RESET}  ${summary.validations.passed}/${summary.validations.total}`);
    console.log(`${CYAN}═════════════════════════════════════════════════${RESET}\n`);
    
    return summary;
  }
  
  // Monitor build progress (static method)
  static async monitor() {
    const buildLogDir = path.join(__dirname, '../build-logs');
    const statusFile = path.join(buildLogDir, 'current-build-status.json');
    
    if (!fs.existsSync(statusFile)) {
      console.log(`${YELLOW}No active build found${RESET}`);
      return;
    }
    
    console.log(`${CYAN}${BOLD}Monitoring build progress...${RESET}`);
    console.log(`Press Ctrl+C to stop monitoring\n`);
    
    let lastUpdate = '';
    
    const interval = setInterval(() => {
      if (!fs.existsSync(statusFile)) {
        console.log(`${GREEN}Build completed${RESET}`);
        clearInterval(interval);
        return;
      }
      
      const status = JSON.parse(fs.readFileSync(statusFile, 'utf8'));
      
      if (status.lastUpdate !== lastUpdate) {
        lastUpdate = status.lastUpdate;
        
        console.clear();
        console.log(`${CYAN}${BOLD}═══════════════ BUILD PROGRESS ═══════════════${RESET}`);
        console.log(`${BOLD}Build ID:${RESET}       ${status.buildId}`);
        console.log(`${BOLD}Current Phase:${RESET}  ${status.currentPhase}`);
        console.log(`${BOLD}Progress:${RESET}       ${status.completedPhases.length}/${status.totalPhases} phases`);
        console.log(`${BOLD}Elapsed:${RESET}        ${(status.elapsed / 1000).toFixed(1)}s`);
        console.log(`${BOLD}Errors:${RESET}         ${status.errors > 0 ? RED : GREEN}${status.errors}${RESET}`);
        console.log(`${BOLD}Warnings:${RESET}       ${status.warnings > 0 ? YELLOW : GREEN}${status.warnings}${RESET}`);
        console.log(`\n${CYAN}Completed Phases:${RESET}`);
        status.completedPhases.forEach(phase => {
          console.log(`  ${GREEN}✓${RESET} ${phase}`);
        });
        console.log(`${CYAN}═════════════════════════════════════════════${RESET}`);
      }
    }, 1000);
  }
}

// Export for use in build scripts
module.exports = BuildPhaseReporter;

// CLI interface
if (require.main === module) {
  const command = process.argv[2];
  
  if (command === 'check') {
    BuildPhaseReporter.checkLastBuild();
  } else if (command === 'monitor') {
    BuildPhaseReporter.monitor();
  } else {
    console.log(`${CYAN}${BOLD}Build Phase Reporter${RESET}`);
    console.log(`\nUsage:`);
    console.log(`  ${GREEN}check${RESET}    - Check last build status`);
    console.log(`  ${GREEN}monitor${RESET}  - Monitor current build progress`);
  }
}