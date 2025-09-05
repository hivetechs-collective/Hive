// Build monitoring enhancements for build-production-dmg.js
// Add these functions to the build script for better real-time monitoring

const fs = require('fs');
const path = require('path');

// Enhanced logging system with real-time log file
class BuildLogger {
  constructor() {
    this.logDir = path.join(__dirname, '../build-logs');
    this.timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    this.logFile = path.join(this.logDir, `build-${this.timestamp}.log`);
    this.statusFile = path.join(this.logDir, 'current-status.json');
    
    // Create log directory if it doesn't exist
    if (!fs.existsSync(this.logDir)) {
      fs.mkdirSync(this.logDir, { recursive: true });
    }
    
    this.currentPhase = 0;
    this.totalPhases = 14;
    this.phaseStartTime = Date.now();
    this.buildStartTime = Date.now();
    this.errors = [];
    this.warnings = [];
    this.criticalChecks = {};
  }

  log(message, level = 'INFO') {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] [${level}] ${message}\n`;
    
    // Write to log file
    fs.appendFileSync(this.logFile, logEntry);
    
    // Also output to console with color
    const colors = {
      'ERROR': '\x1b[31m',
      'WARNING': '\x1b[33m',
      'SUCCESS': '\x1b[32m',
      'INFO': '\x1b[36m',
      'DEBUG': '\x1b[90m'
    };
    const color = colors[level] || '';
    const reset = '\x1b[0m';
    console.log(`${color}${message}${reset}`);
  }

  startPhase(phaseNum, phaseName, description) {
    this.currentPhase = phaseNum;
    this.phaseStartTime = Date.now();
    
    this.log(`\n${'='.repeat(60)}`, 'INFO');
    this.log(`PHASE ${phaseNum}/${this.totalPhases}: ${phaseName}`, 'INFO');
    this.log(description, 'INFO');
    this.log(`${'='.repeat(60)}`, 'INFO');
    
    this.updateStatus();
  }

  endPhase(success = true) {
    const elapsed = ((Date.now() - this.phaseStartTime) / 1000).toFixed(1);
    if (success) {
      this.log(`✓ Phase ${this.currentPhase} completed in ${elapsed}s`, 'SUCCESS');
    } else {
      this.log(`✗ Phase ${this.currentPhase} failed after ${elapsed}s`, 'ERROR');
    }
    this.updateStatus();
  }

  recordError(error) {
    this.errors.push({
      phase: this.currentPhase,
      timestamp: new Date().toISOString(),
      message: error
    });
    this.log(`ERROR: ${error}`, 'ERROR');
    this.updateStatus();
  }

  recordWarning(warning) {
    this.warnings.push({
      phase: this.currentPhase,
      timestamp: new Date().toISOString(),
      message: warning
    });
    this.log(`WARNING: ${warning}`, 'WARNING');
    this.updateStatus();
  }

  recordCriticalCheck(checkName, passed, details) {
    this.criticalChecks[checkName] = {
      passed,
      details,
      timestamp: new Date().toISOString()
    };
    
    if (passed) {
      this.log(`✓ Critical check passed: ${checkName}`, 'SUCCESS');
    } else {
      this.log(`✗ Critical check failed: ${checkName} - ${details}`, 'ERROR');
    }
    this.updateStatus();
  }

  updateStatus() {
    const totalElapsed = ((Date.now() - this.buildStartTime) / 1000).toFixed(1);
    const status = {
      timestamp: new Date().toISOString(),
      currentPhase: this.currentPhase,
      totalPhases: this.totalPhases,
      progress: ((this.currentPhase / this.totalPhases) * 100).toFixed(1) + '%',
      elapsedSeconds: totalElapsed,
      errors: this.errors.length,
      warnings: this.warnings.length,
      criticalChecks: this.criticalChecks,
      logFile: this.logFile,
      lastUpdate: new Date().toISOString()
    };
    
    fs.writeFileSync(this.statusFile, JSON.stringify(status, null, 2));
  }

  generateSummary() {
    const totalElapsed = ((Date.now() - this.buildStartTime) / 1000).toFixed(1);
    
    this.log('\n' + '='.repeat(60), 'INFO');
    this.log('BUILD SUMMARY', 'INFO');
    this.log('='.repeat(60), 'INFO');
    this.log(`Total time: ${totalElapsed} seconds`, 'INFO');
    this.log(`Errors: ${this.errors.length}`, this.errors.length > 0 ? 'ERROR' : 'SUCCESS');
    this.log(`Warnings: ${this.warnings.length}`, this.warnings.length > 0 ? 'WARNING' : 'SUCCESS');
    
    // Critical checks summary
    this.log('\nCritical Checks:', 'INFO');
    for (const [check, result] of Object.entries(this.criticalChecks)) {
      const status = result.passed ? '✓' : '✗';
      const color = result.passed ? 'SUCCESS' : 'ERROR';
      this.log(`  ${status} ${check}: ${result.details}`, color);
    }

    // Write final summary to file
    const summary = {
      buildCompleted: new Date().toISOString(),
      totalSeconds: totalElapsed,
      errors: this.errors,
      warnings: this.warnings,
      criticalChecks: this.criticalChecks
    };
    
    const summaryFile = path.join(this.logDir, `summary-${this.timestamp}.json`);
    fs.writeFileSync(summaryFile, JSON.stringify(summary, null, 2));
    
    this.log(`\nFull logs available at: ${this.logFile}`, 'INFO');
    this.log(`Summary available at: ${summaryFile}`, 'INFO');
  }
}

// Progress tracker for long-running operations
class ProgressTracker {
  constructor(logger) {
    this.logger = logger;
    this.interval = null;
  }

  startProgress(operationName, estimatedSeconds = 60) {
    let elapsed = 0;
    this.logger.log(`Starting: ${operationName} (estimated ${estimatedSeconds}s)`, 'INFO');
    
    this.interval = setInterval(() => {
      elapsed += 5;
      const progress = Math.min(100, (elapsed / estimatedSeconds) * 100);
      process.stdout.write(`\r  Progress: ${progress.toFixed(0)}% [${elapsed}s / ${estimatedSeconds}s]`);
      
      if (elapsed >= estimatedSeconds) {
        process.stdout.write(' (still running...)\r');
      }
    }, 5000);
  }

  stopProgress(success = true) {
    if (this.interval) {
      clearInterval(this.interval);
      this.interval = null;
      process.stdout.write('\r' + ' '.repeat(80) + '\r'); // Clear the line
      this.logger.log(success ? '  ✓ Completed' : '  ✗ Failed', success ? 'SUCCESS' : 'ERROR');
    }
  }
}

// Critical fixes verifier with detailed reporting
class CriticalFixesVerifier {
  constructor(logger) {
    this.logger = logger;
  }

  async verifyAll() {
    this.logger.log('\n🔍 Verifying Critical Production Fixes', 'INFO');
    
    // 1. Python extraction marker
    const extractionMarker = path.join(__dirname, '../resources/python-runtime/.needs_extraction');
    if (fs.existsSync(extractionMarker)) {
      const content = JSON.parse(fs.readFileSync(extractionMarker, 'utf8'));
      this.logger.recordCriticalCheck('Python Extraction Marker', true, 
        `Created at ${content.created} for v${content.version}`);
    } else {
      this.logger.recordCriticalCheck('Python Extraction Marker', false, 
        'File not found - Python will use symlinks (causes dylib failures)');
    }

    // 2. Memory configuration
    const memConfig = path.join(__dirname, '../resources/python-runtime/.memory_config');
    if (fs.existsSync(memConfig)) {
      const config = JSON.parse(fs.readFileSync(memConfig, 'utf8'));
      this.logger.recordCriticalCheck('Memory Configuration', true, 
        `Thread limits: OMP=${config.OMP_NUM_THREADS}, MKL=${config.MKL_NUM_THREADS}`);
    } else {
      this.logger.recordCriticalCheck('Memory Configuration', false, 
        'File not found - Risk of 1.3TB memory allocation');
    }

    // 3. Webpack .dylibs configuration
    const webpackConfig = path.join(__dirname, '../webpack.main.config.ts');
    const webpackContent = fs.readFileSync(webpackConfig, 'utf8');
    const hasDylibsConfig = webpackContent.includes('.dylibs');
    this.logger.recordCriticalCheck('Webpack .dylibs Configuration', hasDylibsConfig, 
      hasDylibsConfig ? 'CopyWebpackPlugin configured for .dylibs' : 'Missing .dylibs configuration');

    // 4. Binary permissions
    const backendBinary = path.join(__dirname, '../binaries/hive-backend-server-enhanced');
    if (fs.existsSync(backendBinary)) {
      const stats = fs.statSync(backendBinary);
      const isExecutable = (stats.mode & 0o111) !== 0;
      this.logger.recordCriticalCheck('Backend Binary Permissions', isExecutable, 
        isExecutable ? 'Binary is executable' : 'Binary is not executable');
    } else {
      this.logger.recordCriticalCheck('Backend Binary Permissions', false, 'Binary not found');
    }

    // 5. Python runtime
    const pythonBinary = path.join(__dirname, '../resources/python-runtime/python/bin/python3');
    if (fs.existsSync(pythonBinary)) {
      this.logger.recordCriticalCheck('Python Runtime', true, 'Python binary present');
    } else {
      this.logger.recordCriticalCheck('Python Runtime', false, 'Python binary missing');
    }
  }

  async verifyBuildOutput(appPath) {
    this.logger.log('\n🔍 Verifying Build Output', 'INFO');
    
    // Check for .dylibs in built app
    const dylibsToCheck = [
      'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/lib/python3.11/site-packages/PIL/.dylibs',
      'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/lib/python3.11/site-packages/sklearn/.dylibs',
      'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/lib/python3.11/site-packages/scipy/.dylibs'
    ];

    let dylibsFound = 0;
    for (const dylibPath of dylibsToCheck) {
      const fullPath = path.join(appPath, dylibPath);
      const packageName = path.basename(path.dirname(dylibPath));
      
      if (fs.existsSync(fullPath)) {
        const files = fs.readdirSync(fullPath);
        this.logger.recordCriticalCheck(`${packageName}/.dylibs`, true, 
          `Found ${files.length} dylib files`);
        dylibsFound++;
      } else {
        this.logger.recordCriticalCheck(`${packageName}/.dylibs`, false, 
          'Directory missing - consensus will fail');
      }
    }

    return dylibsFound > 0;
  }
}

// Export for use in build script
module.exports = {
  BuildLogger,
  ProgressTracker,
  CriticalFixesVerifier
};

// Example integration in build script:
/*
const { BuildLogger, ProgressTracker, CriticalFixesVerifier } = require('./build-monitor-enhancements');

const logger = new BuildLogger();
const progress = new ProgressTracker(logger);
const verifier = new CriticalFixesVerifier(logger);

// Start of build
logger.startPhase(1, 'PRE-BUILD CLEANUP', 'Remove old build artifacts');

// For long operations
progress.startProgress('Webpack compilation', 120);
// ... webpack runs ...
progress.stopProgress(true);

// At critical points
await verifier.verifyAll();

// At end
logger.generateSummary();
*/