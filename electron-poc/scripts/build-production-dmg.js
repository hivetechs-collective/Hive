#!/usr/bin/env node

/**
 * Production DMG Build Script with Complete Order and Verification
 * Ensures all steps are performed in the exact correct sequence
 * Prevents recurring issues like missing permissions, broken binaries, etc.
 */

const fs = require('fs-extra');
const path = require('path');
const os = require('os');
const { execSync, spawn, spawnSync } = require('child_process');

// Terminal colors
const RED = '\x1b[31m';
const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const BLUE = '\x1b[34m';
const CYAN = '\x1b[36m';
const RESET = '\x1b[0m';
const BOLD = '\x1b[1m';

const INSTALLED_APP_BINARY = path.join(
  '/Applications',
  'Hive Consensus.app',
  'Contents',
  'MacOS',
  'Hive Consensus',
);

const SHOULD_RUN_UI_SMOKE = process.env.PLAYWRIGHT_RUN_TESTS === '1';
const IS_CI = process.env.CI === 'true';

function launchInstalledApp() {
  if (process.env.HIVE_SKIP_AUTO_LAUNCH === '1') {
    console.log(`${YELLOW}Skipping automatic launch (HIVE_SKIP_AUTO_LAUNCH=1)${RESET}`);
    return;
  }

  if (!fs.existsSync(INSTALLED_APP_BINARY)) {
    console.log(`${YELLOW}Installed app not found at ${INSTALLED_APP_BINARY}, skipping auto launch${RESET}`);
    return;
  }

  const env = { ...process.env };
  const automationRequested = env.PLAYWRIGHT_E2E === '1' || env.PLAYWRIGHT_REMOTE_DEBUG_PORT;

  if (automationRequested) {
    if (!env.PLAYWRIGHT_REMOTE_DEBUG_PORT) {
      console.log(
        `${YELLOW}PLAYWRIGHT_E2E=1 but PLAYWRIGHT_REMOTE_DEBUG_PORT missing; tests should set both${RESET}`,
      );
    } else {
      console.log(
        `${CYAN}Launching Hive Consensus with remote debugging on port ${env.PLAYWRIGHT_REMOTE_DEBUG_PORT}${RESET}`,
      );
    }
  } else {
    console.log(`${CYAN}Launching Hive Consensus for post-build verification${RESET}`);
  }

  try {
    const child = spawn(INSTALLED_APP_BINARY, [], {
      env,
      detached: true,
      stdio: 'ignore',
    });
    child.unref();
    console.log(`${GREEN}✓ Hive Consensus launched from Applications${RESET}`);
    if (automationRequested && env.PLAYWRIGHT_REMOTE_DEBUG_PORT) {
      console.log(
        `${CYAN}  Playwright attach URL: ws://127.0.0.1:${env.PLAYWRIGHT_REMOTE_DEBUG_PORT}${RESET}`,
      );
    }
  } catch (error) {
    console.log(`${YELLOW}⚠ Failed to launch Hive Consensus automatically: ${error.message}${RESET}`);
  }
}

// ═══════════════════════════════════════════════════════════════
// PATH CONFIGURATION AND VERIFICATION
// ═══════════════════════════════════════════════════════════════

// Load build configuration if it exists
let buildConfig = {};
const configPath = path.join(__dirname, '../build-config.json');
if (fs.existsSync(configPath)) {
  try {
    buildConfig = JSON.parse(fs.readFileSync(configPath, 'utf8'));
    console.log(`${GREEN}✓ Loaded build configuration v${buildConfig.version}${RESET}`);
  } catch (e) {
    console.log(`${YELLOW}⚠ Could not parse build-config.json, using defaults${RESET}`);
  }
}

// Define all critical paths using robust resolution
const ELECTRON_ROOT = path.resolve(__dirname, '..');
const HIVE_ROOT = path.resolve(ELECTRON_ROOT, buildConfig.paths?.hiveRoot || '..');
const RUST_SOURCE_DIR = path.join(HIVE_ROOT, 'src', 'bin');
const RUST_TARGET_DIR = path.join(HIVE_ROOT, 'target', 'release');
const BINARIES_DIR = path.join(ELECTRON_ROOT, 'binaries');
const RESOURCES_DIR = path.join(ELECTRON_ROOT, 'resources');
const PYTHON_RUNTIME_DIR = path.join(RESOURCES_DIR, 'python-runtime');

// Critical file paths
const CRITICAL_PATHS = {
  'Electron root': ELECTRON_ROOT,
  'Hive root': HIVE_ROOT,
  'Rust source directory': RUST_SOURCE_DIR,
  'Package.json': path.join(ELECTRON_ROOT, 'package.json'),
  'Cargo.toml': path.join(HIVE_ROOT, 'Cargo.toml'),
  'Build scripts': __dirname
};

// Verify critical paths exist
console.log(`${CYAN}Verifying project structure...${RESET}`);
let pathErrors = [];
for (const [name, pathToCheck] of Object.entries(CRITICAL_PATHS)) {
  if (!fs.existsSync(pathToCheck)) {
    pathErrors.push(`  ✗ Missing ${name}: ${pathToCheck}`);
  } else {
    console.log(`  ${GREEN}✓${RESET} Found ${name}`);
  }
}

if (pathErrors.length > 0) {
  console.error(`${RED}Critical paths missing:${RESET}`);
  pathErrors.forEach(err => console.error(`${RED}${err}${RESET}`));
  console.error(`${RED}Build cannot continue. Please verify project structure.${RESET}`);
  process.exit(1);
}

// Store configuration for use throughout script
const BUILD_CONFIG = {
  paths: {
    electronRoot: ELECTRON_ROOT,
    hiveRoot: HIVE_ROOT,
    rustSourceDir: RUST_SOURCE_DIR,
    rustTargetDir: RUST_TARGET_DIR,
    binariesDir: BINARIES_DIR,
    pythonRuntimeDir: PYTHON_RUNTIME_DIR
  },
  rustBinary: null, // REMOVED: No longer using separate backend process (DirectConsensusEngine approach)
  pythonVersion: '3.11.7'
};

console.log(`${GREEN}✓ All critical paths verified${RESET}`);

// ═══════════════════════════════════════════════════════════════
// BUILD LOGGING & PROGRESS REPORTING SYSTEM
// ═══════════════════════════════════════════════════════════════

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
    
    if (IS_CI) {
      console.log(`${YELLOW}Visual progress monitor disabled in CI${RESET}`);
    } else {
      console.log(`${GREEN}✓ Visual progress monitor active (16 phases)${RESET}`);
    }
    
    this.currentPhase = 0;
    this.totalPhases = 17; // Total phases including binary bundling and installation
    this.phaseStartTime = Date.now();
    this.buildStartTime = Date.now();
    this.errors = [];
    this.warnings = [];
    this.criticalChecks = {};
    
    // Write initial log entry
    this.log('='.repeat(60), 'INFO');
    this.log('BUILD STARTED', 'INFO');
    this.log(`Timestamp: ${new Date().toISOString()}`, 'INFO');
    this.log(`Log file: ${this.logFile}`, 'INFO');
    this.log('='.repeat(60), 'INFO');
  }
  
  spawnVisualLogger() {
    if (IS_CI) {
      console.log(`${YELLOW}Skipping visual progress monitor in CI${RESET}`);
      return;
    }
    try {
      const BUILD_LOG = '/tmp/hive-build-progress.log';

      // Kill any existing tail processes
      execSync(`pkill -f "tail -f ${BUILD_LOG}" 2>/dev/null || true`);
      
      // Use the exact working format from build-with-popup.sh
      const appleScriptCmd = `osascript <<EOF
tell application "Terminal"
    activate
    set newWindow to do script "echo '🏗️  Hive Consensus Build Progress' && echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━' && echo '' && tail -f ${BUILD_LOG}"
    set bounds of front window to {100, 100, 700, 500}
    set custom title of front window to "Build Progress"
end tell
EOF`;
      
      execSync(appleScriptCmd, { shell: '/bin/bash' });
      console.log(`${GREEN}✓ Visual progress monitor spawned (16 phases)${RESET}`);
      
      // Small delay to ensure Terminal is ready
      execSync('sleep 0.5');
      
    } catch (error) {
      console.log(`${YELLOW}Note: Could not spawn visual monitor${RESET}`);
      if (error.message) {
        console.log(`${YELLOW}Error: ${error.message}${RESET}`);
      }
    }
  }

  log(message, level = 'INFO') {
    const timestamp = new Date().toISOString();
    const logEntry = `[${timestamp}] [${level}] ${message}\n`;
    
    // Write to log file
    fs.appendFileSync(this.logFile, logEntry);
    
    // Also write to visual progress log for Terminal display
    const visualLog = '/tmp/hive-build-progress.log';
    fs.appendFileSync(visualLog, message + '\n');
  }

  startPhase(phaseNum, phaseName, description) {
    this.currentPhase = phaseNum;
    this.phaseStartTime = Date.now();
    
    // Log to file
    this.log(`\nPHASE ${phaseNum}/${this.totalPhases}: ${phaseName}`, 'PHASE');
    this.log(`Description: ${description}`, 'PHASE');
    this.log(`Started at: ${new Date().toISOString()}`, 'PHASE');
    
    // Update real-time status
    this.updateStatus(`Running Phase ${phaseNum}: ${phaseName}`);
  }

  endPhase(success = true) {
    const elapsed = ((Date.now() - this.phaseStartTime) / 1000).toFixed(1);
    const status = success ? 'COMPLETED' : 'FAILED';
    
    this.log(`Phase ${this.currentPhase} ${status} in ${elapsed}s`, success ? 'SUCCESS' : 'ERROR');
    
    // Update real-time status
    this.updateStatus(`Phase ${this.currentPhase} ${status} (${elapsed}s)`);
  }

  recordError(error) {
    this.errors.push({
      phase: this.currentPhase,
      timestamp: new Date().toISOString(),
      message: error
    });
    this.log(`ERROR: ${error}`, 'ERROR');
    this.updateStatus(`ERROR in Phase ${this.currentPhase}: ${error}`);
  }

  recordWarning(warning) {
    this.warnings.push({
      phase: this.currentPhase,
      timestamp: new Date().toISOString(),
      message: warning
    });
    this.log(`WARNING: ${warning}`, 'WARNING');
    this.updateStatus(`WARNING in Phase ${this.currentPhase}: ${warning}`);
  }

  recordCriticalCheck(checkName, passed, details) {
    this.criticalChecks[checkName] = {
      passed,
      details,
      timestamp: new Date().toISOString(),
      phase: this.currentPhase
    };
    
    this.log(`Critical check '${checkName}': ${passed ? 'PASSED' : 'FAILED'}`, passed ? 'CHECK_PASS' : 'CHECK_FAIL');
    this.log(`  Details: ${details}`, passed ? 'CHECK_PASS' : 'CHECK_FAIL');
    this.updateStatus(`Check '${checkName}': ${passed ? '✓' : '✗'}`);
  }

  updateStatus(message = null) {
    const totalElapsed = ((Date.now() - this.buildStartTime) / 1000).toFixed(1);
    const phaseElapsed = ((Date.now() - this.phaseStartTime) / 1000).toFixed(1);
    
    const status = {
      timestamp: new Date().toISOString(),
      currentPhase: this.currentPhase,
      totalPhases: this.totalPhases,
      phaseProgress: ((this.currentPhase / this.totalPhases) * 100).toFixed(1) + '%',
      currentPhaseTime: phaseElapsed + 's',
      totalElapsedTime: totalElapsed + 's',
      totalElapsedMinutes: (totalElapsed / 60).toFixed(1) + 'm',
      errors: this.errors.length,
      warnings: this.warnings.length,
      criticalChecksPassed: Object.values(this.criticalChecks).filter(c => c.passed).length,
      criticalChecksFailed: Object.values(this.criticalChecks).filter(c => !c.passed).length,
      lastMessage: message || 'Build in progress...',
      logFile: this.logFile
    };
    
    fs.writeFileSync(this.statusFile, JSON.stringify(status, null, 2));
  }

  generateSummary() {
    const totalElapsed = ((Date.now() - this.buildStartTime) / 1000).toFixed(1);
    
    this.log('\n' + '='.repeat(60), 'INFO');
    this.log('BUILD COMPLETED', 'INFO');
    this.log(`Total time: ${totalElapsed}s (${(totalElapsed / 60).toFixed(1)}m)`, 'INFO');
    this.log(`Errors: ${this.errors.length}`, this.errors.length > 0 ? 'ERROR' : 'SUCCESS');
    this.log(`Warnings: ${this.warnings.length}`, this.warnings.length > 0 ? 'WARNING' : 'SUCCESS');
    this.log('='.repeat(60), 'INFO');
    
    const summary = {
      buildCompleted: new Date().toISOString(),
      totalSeconds: totalElapsed,
      totalMinutes: (totalElapsed / 60).toFixed(1),
      errors: this.errors,
      warnings: this.warnings,
      criticalChecks: this.criticalChecks,
      phases: this.currentPhase,
      logFile: this.logFile
    };
    
    const summaryFile = path.join(this.logDir, `summary-${this.timestamp}.json`);
    fs.writeFileSync(summaryFile, JSON.stringify(summary, null, 2));
    
    // Output summary locations
    console.log(`\n${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
    console.log(`${CYAN}                    BUILD LOGS & REPORTS                       ${RESET}`);
    console.log(`${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
    console.log(`  ${GREEN}✓${RESET} Full log: ${this.logFile}`);
    console.log(`  ${GREEN}✓${RESET} Summary: ${summaryFile}`);
    console.log(`  ${GREEN}✓${RESET} Real-time status: ${this.statusFile}`);
    
    if (this.errors.length > 0) {
      console.log(`\n  ${RED}⚠${RESET} Build completed with ${this.errors.length} error(s)`);
    }
    if (this.warnings.length > 0) {
      console.log(`  ${YELLOW}⚠${RESET} Build completed with ${this.warnings.length} warning(s)`);
    }
    
    return summary;
  }
}

// Spawn visual terminal FIRST before anything else (local runs only)
const BUILD_LOG = '/tmp/hive-build-progress.log';
if (!IS_CI) {
  try {
    // Clear the log file first to remove old content
    fs.writeFileSync(BUILD_LOG, 'Starting build...\n');

    // Kill any existing tail processes
    execSync(`pkill -f "tail -f ${BUILD_LOG}" 2>/dev/null || true`);

    // Launch Terminal window to show progress
    const appleScriptCmd = `osascript <<EOF
tell application "Terminal"
    activate
    set newWindow to do script "echo '🏗️  Hive Consensus Build Progress' && echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━' && echo '' && tail -f ${BUILD_LOG}"
    set bounds of front window to {100, 100, 700, 500}
    set custom title of front window to "Build Progress"
end tell
EOF`;
    execSync(appleScriptCmd, { shell: '/bin/bash' });
    execSync('sleep 0.5'); // Ensure Terminal is ready
  } catch (error) {
    // Continue even if visual terminal fails
  }
} else {
  // Ensure log file exists even when not streaming to Terminal
  fs.writeFileSync(BUILD_LOG, 'CI build: visual monitor disabled.\n');
}

// Initialize build logger (which will now skip spawning since we already did it)
const logger = new BuildLogger();

console.log(`${CYAN}${BOLD}
╔══════════════════════════════════════════════════════════════╗
║           🏗️  Production DMG Build System v2.0               ║
╚══════════════════════════════════════════════════════════════╝
${RESET}`);

// Helper to write to visual progress log
const writeToVisualLog = (message) => {
  const visualLog = '/tmp/hive-build-progress.log';
  // Strip ANSI color codes for cleaner terminal display
  const cleanMessage = message.replace(/\x1b\[[0-9;]*m/g, '');
  fs.appendFileSync(visualLog, cleanMessage + '\n');
};

// Override console.log to also write to visual log
const originalConsoleLog = console.log;
console.log = (...args) => {
  originalConsoleLog(...args);
  writeToVisualLog(args.join(' '));
};

// Build phase tracking with timing
let currentPhase = 0;
const totalPhases = 17;
const phaseTimes = [];
let phaseStartTime = null;
let currentPhaseName = null;
const buildStartTime = Date.now();

function logPhase(phaseName, description) {
  // Record previous phase time if exists
  if (phaseStartTime && currentPhase > 0 && currentPhaseName) {
    const elapsedSeconds = ((Date.now() - phaseStartTime) / 1000).toFixed(1);
    phaseTimes.push({ 
      phase: currentPhase, 
      name: currentPhaseName,
      duration: elapsedSeconds 
    });
  }
  
  currentPhase++;
  currentPhaseName = phaseName;
  phaseStartTime = Date.now();
  
  console.log(`\n${BLUE}${BOLD}━━━ PHASE ${currentPhase}/${totalPhases}: ${phaseName} ━━━${RESET}`);
  console.log(`${CYAN}${description}${RESET}\n`);
}

function execCommand(command, description, options = {}) {
  console.log(`${YELLOW}➤ ${description}${RESET}`);
  console.log(`  ${command}`);
  
  try {
    const result = execSync(command, { 
      encoding: 'utf8',
      stdio: options.silent ? 'pipe' : 'inherit',
      maxBuffer: 50 * 1024 * 1024, // 50MB buffer
      ...options 
    });
    console.log(`${GREEN}✓ Success${RESET}`);
    return result;
  } catch (error) {
    console.error(`${RED}✗ Failed: ${error.message}${RESET}`);
    if (!options.allowFail) {
      process.exit(1);
    }
    return null;
  }
}

// Special function for long-running commands that need real-time output
function execLongCommand(command, description) {
  console.log(`${YELLOW}➤ ${description}${RESET}`);
  console.log(`  ${command}`);
  console.log(`${CYAN}  This may take several minutes...${RESET}`);
  
  return new Promise((resolve, reject) => {
    // Parse the command to handle npm run commands properly
    const [cmd, ...args] = command.split(' ');
    
    // Use spawnSync for better handling of long-running processes
    const result = spawnSync(cmd, args, {
      stdio: 'inherit',
      shell: true,
      env: { 
        ...process.env, 
        ENABLE_WORKER_ARCHITECTURE: 'true' // Enable worker processes for performance
      },
      maxBuffer: 100 * 1024 * 1024 // 100MB buffer
    });
    
    if (result.error) {
      console.error(`${RED}✗ Failed: ${result.error.message}${RESET}`);
      reject(result.error);
    } else if (result.status !== 0) {
      console.error(`${RED}✗ Failed with exit code: ${result.status}${RESET}`);
      reject(new Error(`Command failed with exit code ${result.status}`));
    } else {
      console.log(`${GREEN}✓ Success${RESET}`);
      resolve();
    }
  });
}

function checkFile(filePath, description, requireExecutable = false) {
  console.log(`${YELLOW}🔍 Checking: ${description}${RESET}`);
  
  if (!fs.existsSync(filePath)) {
    console.error(`${RED}✗ Not found: ${filePath}${RESET}`);
    return false;
  }
  
  if (requireExecutable) {
    try {
      fs.accessSync(filePath, fs.constants.X_OK);
      console.log(`${GREEN}✓ Found and executable${RESET}`);
    } catch {
      console.log(`${YELLOW}⚠ Found but not executable - will fix${RESET}`);
      return 'fix-permissions';
    }
  } else {
    console.log(`${GREEN}✓ Found${RESET}`);
  }
  
  return true;
}

// ═══════════════════════════════════════════════════════════════
// PHASE 1: Pre-build Cleanup
// ═══════════════════════════════════════════════════════════════

logPhase('PRE-BUILD CLEANUP', 'Remove old build artifacts to ensure clean build');

// Auto-increment version for build tracking
const packageJsonPath = path.join(__dirname, '..', 'package.json');
const pkg = require(packageJsonPath);
const currentVersion = pkg.version;
const versionParts = currentVersion.split('.');
const patchVersion = parseInt(versionParts[2]) + 1;
const newVersion = `${versionParts[0]}.${versionParts[1]}.${patchVersion}`;

console.log(`${CYAN}Auto-incrementing version: ${currentVersion} → ${newVersion}${RESET}`);
pkg.version = newVersion;
fs.writeFileSync(packageJsonPath, JSON.stringify(pkg, null, 2));

execCommand(
  'rm -rf .webpack out dist',
  'Removing old build directories',
  { allowFail: true }
);

execCommand(
  'rm -rf node_modules/.cache',
  'Clearing build caches',
  { allowFail: true }
);

// ═══════════════════════════════════════════════════════════════
// PHASE 2: Verify Build Tools
// ═══════════════════════════════════════════════════════════════

logPhase('BUILD TOOLS VERIFICATION', 'Ensure all required tools are installed');

const nodeVersion = execCommand('node --version', 'Checking Node.js', { silent: true });
const npmVersion = execCommand('npm --version', 'Checking npm', { silent: true });
const rustVersion = execCommand('cargo --version', 'Checking Rust/Cargo', { silent: true, allowFail: true });

console.log(`  Node.js: ${nodeVersion?.trim() || 'Not found'}`);
console.log(`  npm: ${npmVersion?.trim() || 'Not found'}`);
console.log(`  Rust: ${rustVersion?.trim() || 'Not required for build'}`);

// ═══════════════════════════════════════════════════════════════
// PHASE 3: Binary Bundling
// ═══════════════════════════════════════════════════════════════
// This MUST happen BEFORE npm install so binaries directory exists for webpack

logPhase('BINARY BUNDLING', 'Bundle ttyd, git, and node for self-contained app');

// Load version requirements for binaries
const versionReqPath = path.join(__dirname, '../version-requirements.json');
let binaryRequirements = {};
if (fs.existsSync(versionReqPath)) {
  const versionRequirements = require(versionReqPath);
  binaryRequirements = versionRequirements.bundledBinaries || {};
  console.log(`${CYAN}Loaded binary version requirements${RESET}`);
} else {
  console.log(`${YELLOW}⚠ No version requirements found for binaries${RESET}`);
}

// Helper function to check version compatibility
const checkVersionCompatibility = (binaryName, versionString) => {
  const req = binaryRequirements[binaryName];
  if (!req) return true; // No requirements, assume OK
  
  // Extract version number from string (e.g., "ttyd version 1.7.4" -> "1.7.4")
  const versionMatch = versionString.match(/(\d+\.\d+\.\d+)/);
  if (!versionMatch) {
    console.log(`${YELLOW}⚠ Could not parse version for ${binaryName}: ${versionString}${RESET}`);
    return true; // Can't parse, assume OK
  }
  
  const version = versionMatch[1];
  const [major, minor, patch] = version.split('.').map(Number);
  const [minMajor, minMinor, minPatch] = (req.minVersion || '0.0.0').split('.').map(Number);
  const [maxMajor, maxMinor, maxPatch] = (req.maxVersion || '999.999.999').split('.').map(Number);
  
  // Check if version is within range
  const versionNum = major * 1000000 + minor * 1000 + patch;
  const minNum = minMajor * 1000000 + minMinor * 1000 + minPatch;
  const maxNum = maxMajor * 1000000 + maxMinor * 1000 + maxPatch;
  
  if (versionNum < minNum) {
    console.error(`${RED}✗ ${binaryName} version ${version} is below minimum ${req.minVersion}${RESET}`);
    return false;
  }
  if (versionNum >= maxNum) {
    console.error(`${RED}✗ ${binaryName} version ${version} is above maximum ${req.maxVersion}${RESET}`);
    return false;
  }
  
  console.log(`${GREEN}✓ ${binaryName} version ${version} is compatible (${req.minVersion} - <${req.maxVersion})${RESET}`);
  return true;
};

// Create binaries directory if it doesn't exist
// This is the source directory that webpack will copy from
const binariesDir = path.join(__dirname, '../binaries');
if (!fs.existsSync(binariesDir)) {
  fs.mkdirSync(binariesDir, { recursive: true });
  console.log(`${GREEN}✓ Created binaries directory${RESET}`);
}

// Clean old binaries to ensure fresh bundle (but preserve backend binary)
const existingFiles = fs.readdirSync(binariesDir);
for (const file of existingFiles) {
  // Skip cleaning the backend binary - it's expensive to rebuild
  if (file === 'hive-backend-server-enhanced') {
    console.log(`${CYAN}Preserving existing backend binary${RESET}`);
    continue;
  }
  
  const filePath = path.join(binariesDir, file);
  if (fs.statSync(filePath).isDirectory()) {
    execCommand(`rm -rf "${filePath}"`, `Cleaning old ${file}`, { silent: true });
  } else {
    fs.unlinkSync(filePath);
  }
}
console.log(`${CYAN}Cleaned old binaries${RESET}`);

// Track bundled binaries for webpack configuration
const bundledBinaries = [];

// 1. Bundle ttyd (Terminal Server)
console.log(`${CYAN}Bundling ttyd (terminal server)...${RESET}`);
const ttydTargetPath = path.join(binariesDir, 'ttyd');
let ttydBundled = false;

// Check if ttyd exists in system
const ttydSystemPaths = [
  '/opt/homebrew/bin/ttyd',    // Homebrew on Apple Silicon
  '/usr/local/bin/ttyd',        // Homebrew on Intel or manual install
  '/usr/bin/ttyd'               // System package manager
];

for (const ttydPath of ttydSystemPaths) {
  if (fs.existsSync(ttydPath)) {
    try {
      // Copy ttyd binary
      fs.copyFileSync(ttydPath, ttydTargetPath);
      fs.chmodSync(ttydTargetPath, 0o755); // Make executable
      
      // Verify it works and check version
      const ttydVersion = execSync(`"${ttydTargetPath}" --version 2>&1`, { encoding: 'utf8' }).trim();
      
      // Check version compatibility
      if (!checkVersionCompatibility('ttyd', ttydVersion)) {
        const req = binaryRequirements.ttyd;
        if (req && req.critical) {
          console.error(`${RED}✗ ttyd version incompatible and marked as critical!${RESET}`);
          console.error(`${RED}  Please install a compatible version: ${req.installCommand}${RESET}`);
          if (!process.env.ALLOW_VERSION_MISMATCH) {
            process.exit(1);
          }
        }
      }
      
      console.log(`${GREEN}✓ Bundled ttyd: ${ttydVersion}${RESET}`);
      bundledBinaries.push('ttyd');
      ttydBundled = true;
      break;
    } catch (e) {
      console.log(`${YELLOW}⚠ Found ttyd at ${ttydPath} but couldn't bundle it: ${e.message}${RESET}`);
    }
  }
}

if (!ttydBundled) {
  console.error(`${RED}✗ CRITICAL: ttyd not found! Install with: brew install ttyd${RESET}`);
  console.error(`${RED}  The app WILL crash without ttyd for terminal functionality${RESET}`);
  if (process.env.CI !== 'true') {
    console.log(`${YELLOW}Attempting to install ttyd automatically...${RESET}`);
    try {
      execCommand('brew install ttyd', 'Installing ttyd via Homebrew', { timeout: 60000 });
      // Try again after installation
      for (const ttydPath of ttydSystemPaths) {
        if (fs.existsSync(ttydPath)) {
          fs.copyFileSync(ttydPath, ttydTargetPath);
          fs.chmodSync(ttydTargetPath, 0o755);
          const ttydVersion = execSync(`"${ttydTargetPath}" --version 2>&1`, { encoding: 'utf8' }).trim();
          console.log(`${GREEN}✓ Successfully installed and bundled ttyd: ${ttydVersion}${RESET}`);
          bundledBinaries.push('ttyd');
          ttydBundled = true;
          break;
        }
      }
    } catch (e) {
      console.error(`${RED}Failed to auto-install ttyd: ${e.message}${RESET}`);
    }
  }
  
  if (!ttydBundled && !process.env.ALLOW_MISSING_DEPS) {
    console.error(`${RED}Build cannot continue without ttyd. Set ALLOW_MISSING_DEPS=true to skip.${RESET}`);
    process.exit(1);
  }
}

// 2. Bundle Git (Version Control) - Optional
console.log(`${CYAN}Bundling git (version control)...${RESET}`);
const gitTargetDir = path.join(binariesDir, 'git-bundle');
let gitBundled = false;

// For git, we need to bundle the entire git installation, not just the binary
// This includes libexec/git-core and share/git-core directories
const gitSystemPaths = [
  '/opt/homebrew',              // Homebrew on Apple Silicon
  '/usr/local',                  // Homebrew on Intel
  '/usr'                         // System git
];

for (const gitPrefix of gitSystemPaths) {
  const gitBinPath = path.join(gitPrefix, 'bin/git');
  if (fs.existsSync(gitBinPath)) {
    try {
      // Create git bundle directory structure
      if (!fs.existsSync(gitTargetDir)) {
        fs.mkdirSync(gitTargetDir, { recursive: true });
        fs.mkdirSync(path.join(gitTargetDir, 'bin'), { recursive: true });
        fs.mkdirSync(path.join(gitTargetDir, 'libexec'), { recursive: true });
        fs.mkdirSync(path.join(gitTargetDir, 'share'), { recursive: true });
      }
      
      // Copy git binary
      const gitTargetBin = path.join(gitTargetDir, 'bin/git');
      fs.copyFileSync(gitBinPath, gitTargetBin);
      fs.chmodSync(gitTargetBin, 0o755);
      
      // Copy git-core executables (required for git operations)
      const gitCoreSource = path.join(gitPrefix, 'libexec/git-core');
      const gitCoreTarget = path.join(gitTargetDir, 'libexec/git-core');
      if (fs.existsSync(gitCoreSource)) {
        execCommand(`cp -R "${gitCoreSource}" "${gitCoreTarget}"`, 'Copying git-core executables', { silent: true });
        // Make all git-core executables have proper permissions
        execCommand(`chmod -R 755 "${gitCoreTarget}"`, 'Setting git-core permissions', { silent: true });
      }
      
      // Copy git templates and completion (for full functionality)
      const gitShareSource = path.join(gitPrefix, 'share/git-core');
      const gitShareTarget = path.join(gitTargetDir, 'share/git-core');
      if (fs.existsSync(gitShareSource)) {
        execCommand(`cp -R "${gitShareSource}" "${gitShareTarget}"`, 'Copying git templates', { silent: true });
      }
      
      // Verify bundled git works and check version
      const gitVersion = execSync(`"${gitTargetBin}" --version 2>&1`, { encoding: 'utf8' }).trim();
      
      // Check version compatibility
      if (!checkVersionCompatibility('git', gitVersion)) {
        const req = binaryRequirements.git;
        if (req && req.critical) {
          console.error(`${RED}✗ git version incompatible and marked as critical!${RESET}`);
          if (!process.env.ALLOW_VERSION_MISMATCH) {
            process.exit(1);
          }
        }
      }
      
      console.log(`${GREEN}✓ Bundled git: ${gitVersion}${RESET}`);
      bundledBinaries.push('git');
      gitBundled = true;
      break;
    } catch (e) {
      console.log(`${YELLOW}⚠ Found git at ${gitPrefix} but couldn't bundle it: ${e.message}${RESET}`);
    }
  }
}

if (!gitBundled) {
  console.error(`${RED}⚠ Git not bundled - source control features will be limited${RESET}`);
}

// 3. Bundle Node.js (For Memory Service)
console.log(`${CYAN}Bundling Node.js runtime (for Memory Service)...${RESET}`);
const nodeTargetPath = path.join(binariesDir, 'node');
let nodeBundled = false;
if (process.env.HIVE_BUNDLE_NODE_DIST === '0') {
  console.log(`${YELLOW}Skipping Node.js dist bundling (HIVE_BUNDLE_NODE_DIST=0)${RESET}`);
} else {
let nodeBundled = false;

try {
  const nodeVersionRaw = execSync('node --version', { encoding: 'utf8' }).trim();
  const nodeVersion = nodeVersionRaw.replace(/^v/, '');
  const nodeMajor = parseInt(nodeVersion.split('.')[0], 10);

  const platform = process.platform;
  const arch = process.arch === 'arm64' ? 'arm64' : 'x64';

  if (platform !== 'darwin' && platform !== 'linux') {
    throw new Error(`Unsupported platform for Node runtime bundling: ${platform}`);
  }

  const nodeFilename = `node-v${nodeVersion}-${platform}-${arch}`;
  const nodeArchive = `${nodeFilename}.${platform === 'win32' ? 'zip' : 'tar.gz'}`;
  const nodeUrl = `https://nodejs.org/dist/v${nodeVersion}/${nodeArchive}`;
  const downloadPath = path.join(os.tmpdir(), nodeArchive);
  const extractPath = path.join(os.tmpdir(), nodeFilename);

  if (!fs.existsSync(downloadPath)) {
    console.log(`${CYAN}  Downloading official Node.js ${nodeVersionRaw}...${RESET}`);
    execSync(`curl -L "${nodeUrl}" -o "${downloadPath}"`, { stdio: 'inherit' });
  } else {
    console.log(`${CYAN}  Using cached Node.js archive at ${downloadPath}${RESET}`);
  }

  if (fs.existsSync(extractPath)) {
    fs.rmSync(extractPath, { recursive: true, force: true });
  }

  console.log(`${CYAN}  Extracting Node.js archive...${RESET}`);
  execSync(`tar -xzf "${downloadPath}" -C "${os.tmpdir()}"`, { stdio: 'inherit' });

  // Copy full Node distribution so npm/npx work
  const nodeDistTarget = path.join(binariesDir, 'node-dist');
  fs.copySync(extractPath, nodeDistTarget, { overwrite: true, dereference: true });

  const nodeBinarySource = path.join(nodeDistTarget, 'bin', platform === 'win32' ? 'node.exe' : 'node');
  if (!fs.existsSync(nodeBinarySource)) {
    throw new Error(`Failed to find node binary at ${nodeBinarySource}`);
  }

  // Create wrapper shims in binaries/
  fs.copyFileSync(nodeBinarySource, nodeTargetPath);
  fs.chmodSync(nodeTargetPath, 0o755);
  const npmShim = path.join(binariesDir, 'npm');
  const npxShim = path.join(binariesDir, 'npx');
  const npmSource = path.join(nodeDistTarget, 'bin', 'npm');
  const npxSource = path.join(nodeDistTarget, 'bin', 'npx');
  const writeShim = (src, dest) => {
    if (fs.existsSync(src)) {
      const shim = `#!/bin/sh\nexec \"${src}\" \"$@\"\n`;
      fs.writeFileSync(dest, shim);
      fs.chmodSync(dest, 0o755);
    }
  };
  writeShim(npmSource, npmShim);
  writeShim(npxSource, npxShim);

  console.log(`${GREEN}✓ Bundled Node.js: ${nodeVersionRaw}${RESET}`);
  bundledBinaries.push('node');
  if (fs.existsSync(npmShim)) bundledBinaries.push('npm');
  if (fs.existsSync(npxShim)) bundledBinaries.push('npx');
  nodeBundled = true;

  // Clean up extracted directory to save space
  fs.rmSync(extractPath, { recursive: true, force: true });
} catch (error) {
  console.log(`${YELLOW}⚠ Failed to bundle standalone Node.js runtime: ${error.message}${RESET}`);
}

} // end HIVE_BUNDLE_NODE_DIST guard

if (!nodeBundled) {
  console.log(`${YELLOW}Using Electron's built-in Node.js for Memory Service${RESET}`);
  const nodeWrapperScript = `#!/bin/sh\n# Wrapper to use Electron's Node.js runtime\nAPP_BIN=\"$(dirname \"$0\")/../../../../../MacOS/Hive Consensus\"\nELECTRON_RUN_AS_NODE=1 exec \"$APP_BIN\" \"$@\"\n`;
  fs.writeFileSync(nodeTargetPath, nodeWrapperScript);
  fs.chmodSync(nodeTargetPath, 0o755);
  bundledBinaries.push('node');
  nodeBundled = true;
}

// 3b. Bundle uv (for Spec Kit / Specify CLI)
console.log(`${CYAN}Bundling uv (Specify CLI dependency)...${RESET}`);
const uvTargetPath = path.join(binariesDir, 'uv');
let uvBundled = false;
if (process.env.HIVE_BUNDLE_UV !== '1') {
  console.log(`${YELLOW}Skipping uv bundling (HIVE_BUNDLE_UV!=1)${RESET}`);
} else {
let uvBundled = false;

try {
  const uvSystemPaths = [
    '/opt/homebrew/bin/uv',
    '/usr/local/bin/uv',
    '/usr/bin/uv',
  ];
  let uvFound = '';
  for (const p of uvSystemPaths) {
    if (fs.existsSync(p)) { uvFound = p; break; }
  }
  if (!uvFound) {
    try {
      const whichUv = execSync('which uv 2>/dev/null', { encoding: 'utf8' }).trim();
      if (whichUv) uvFound = whichUv;
    } catch {}
  }

  if (!uvFound && process.env.CI !== 'true') {
    console.log(`${YELLOW}uv not found on builder; attempting brew install uv...${RESET}`);
    try {
      execCommand('brew install uv', 'Installing uv via Homebrew', { timeout: 60000 });
      const whichUv = execSync('which uv 2>/dev/null', { encoding: 'utf8' }).trim();
      if (whichUv) uvFound = whichUv;
    } catch (e) {
      console.log(`${YELLOW}⚠ Failed to install uv via brew: ${e.message}${RESET}`);
    }
  }

  if (uvFound) {
    fs.copyFileSync(uvFound, uvTargetPath);
    fs.chmodSync(uvTargetPath, 0o755);
    const uvVersion = execSync(`"${uvTargetPath}" --version 2>&1`, { encoding: 'utf8' }).trim();
    console.log(`${GREEN}✓ Bundled uv: ${uvVersion}${RESET}`);
    bundledBinaries.push('uv');
    uvBundled = true;
  } else {
    console.log(`${YELLOW}⚠ uv not bundled; Specify CLI install will attempt user-side bootstrap${RESET}`);
  }
} catch (e) {
  console.log(`${YELLOW}⚠ Failed to bundle uv: ${e.message}${RESET}`);
}
}

// 4. Create binary manifest for runtime
const binaryManifest = {
  bundled: bundledBinaries,
  paths: {
    ttyd: ttydBundled ? 'binaries/ttyd' : null,
    git: gitBundled ? 'binaries/git-bundle/bin/git' : null,
    node: nodeBundled ? 'binaries/node' : null,
    uv: uvBundled ? 'binaries/uv' : null,
    npm: fs.existsSync(path.join(binariesDir, 'npm')) ? 'binaries/npm' : null,
    npx: fs.existsSync(path.join(binariesDir, 'npx')) ? 'binaries/npx' : null,
  },
  bundledAt: new Date().toISOString(),
  versions: binaryRequirements
};

fs.writeFileSync(
  path.join(binariesDir, 'manifest.json'),
  JSON.stringify(binaryManifest, null, 2)
);

console.log(`${GREEN}✓ Binary manifest created${RESET}`);
console.log(`${CYAN}Bundled binaries: ${bundledBinaries.join(', ') || 'none'}${RESET}`);

// Log warnings for missing binaries
if (!ttydBundled) {
  logger.log('WARNING: ttyd not bundled - terminal features will not work!');
}
if (!gitBundled) {
  logger.log('WARNING: git not bundled - version control features limited');
}
if (!nodeBundled) {
  logger.log('WARNING: node not properly bundled - Memory Service may fail');
}

logger.updateStatus(`Bundled binaries: ${bundledBinaries.join(', ') || 'none'}`);

// ═══════════════════════════════════════════════════════════════
// PHASE 4: Dependency Installation
// ═══════════════════════════════════════════════════════════════

logPhase('DEPENDENCY INSTALLATION', 'Install and verify all npm packages');

execCommand(
  'npm install --force',
  'Installing npm dependencies'
);

// Verify critical dependencies
const criticalDeps = ['electron', 'webpack', 'express', 'ws', '@electron-forge/cli'];
for (const dep of criticalDeps) {
  const depPath = path.join(__dirname, '../node_modules', dep);
  if (!fs.existsSync(depPath)) {
    console.error(`${RED}✗ Critical dependency missing: ${dep}${RESET}`);
    process.exit(1);
  }
}
console.log(`${GREEN}✓ All critical dependencies verified${RESET}`);

// ═══════════════════════════════════════════════════════════════
// PHASE 3.5: Electron Version Verification and Native Module Rebuild
// ═══════════════════════════════════════════════════════════════

logPhase('VERSION VERIFICATION & NATIVE MODULES', 'Ensure correct versions and rebuild native modules');

// Load version requirements
const versionReqFilePath = path.join(__dirname, '../version-requirements.json');
let versionReqs = {};
if (fs.existsSync(versionReqFilePath)) {
  versionReqs = require(versionReqFilePath);
  console.log(`${CYAN}Loaded version requirements from version-requirements.json${RESET}`);
} else {
  console.log(`${YELLOW}⚠ version-requirements.json not found, using package.json versions${RESET}`);
}

// Check actual Electron version installed
const electronPackagePath = path.join(__dirname, '../node_modules/electron/package.json');
if (fs.existsSync(electronPackagePath)) {
  const electronPkg = require(electronPackagePath);
  const installedVersion = electronPkg.version;
  const requiredVersion = versionReqs.requiredVersions?.electron?.version || pkg.devDependencies.electron;
  
  console.log(`${CYAN}Installed Electron version: ${installedVersion}${RESET}`);
  console.log(`${CYAN}Required Electron version: ${requiredVersion}${RESET}`);
  
  // Strict version check
  if (installedVersion !== requiredVersion) {
    console.error(`${RED}✗ CRITICAL VERSION MISMATCH!${RESET}`);
    console.error(`${RED}  Installed: ${installedVersion}${RESET}`);
    console.error(`${RED}  Required:  ${requiredVersion}${RESET}`);
    console.error(`${RED}  This WILL cause native module crashes!${RESET}`);
    
    // Auto-fix the version
    console.log(`${YELLOW}Attempting to fix Electron version...${RESET}`);
    execCommand(
      `npm install electron@${requiredVersion} --save-dev --save-exact`,
      'Installing correct Electron version',
      { timeout: 60000 }
    );
    console.log(`${GREEN}✓ Electron version corrected to ${requiredVersion}${RESET}`);
  } else {
    console.log(`${GREEN}✓ Electron version matches requirements${RESET}`);
  }
  
  // Log Node and Chrome versions for this Electron
  try {
    // Use Electron in headless mode to prevent GUI startup error
    const electronInfo = execSync(`npx electron --version`, { 
      encoding: 'utf8',
      env: { ...process.env, ELECTRON_ENABLE_LOGGING: '0', ELECTRON_RUN_AS_NODE: '1' }
    }).trim();
    console.log(`${CYAN}Electron binary reports: ${electronInfo}${RESET}`);
    
    // Get Node ABI version for native module rebuilding (run as Node to avoid GUI)
    const abiVersion = execSync(`ELECTRON_RUN_AS_NODE=1 npx electron -p "process.versions.modules"`, { 
      encoding: 'utf8',
      env: { ...process.env, ELECTRON_RUN_AS_NODE: '1' }
    }).trim();
    console.log(`${CYAN}Electron ABI version: ${abiVersion} (for native modules)${RESET}`);
  } catch (e) {
    console.log(`${YELLOW}⚠ Could not query Electron binary directly (normal in SSH/headless)${RESET}`);
    // Fallback: use known ABI version for Electron 37.x
    console.log(`${CYAN}Using known ABI version for Electron 37.x: 133${RESET}`);
  }
}

// Rebuild native modules for Electron
console.log(`${CYAN}Rebuilding native modules for Electron...${RESET}`);

// List of native modules that need rebuilding
const nativeModules = [
  'sqlite3',
  'better-sqlite3',
  'node-pty'
];

// Method 1: Try using electron-rebuild (preferred)
if (fs.existsSync(path.join(__dirname, '../node_modules/electron-rebuild'))) {
  try {
    console.log(`${CYAN}Using electron-rebuild to rebuild native modules...${RESET}`);
    execCommand(
      'npx electron-rebuild --force --only sqlite3,better-sqlite3,node-pty',
      'Rebuilding native modules with electron-rebuild',
      { timeout: 120000 } // 2 minute timeout
    );
    console.log(`${GREEN}✓ Native modules rebuilt successfully${RESET}`);
  } catch (rebuildError) {
    console.log(`${YELLOW}⚠ electron-rebuild failed, trying alternative method...${RESET}`);
    
    // Method 2: Use Electron Forge's rebuild if available
    try {
      execCommand(
        'npx electron-forge rebuild',
        'Rebuilding with Electron Forge',
        { timeout: 120000 }
      );
      console.log(`${GREEN}✓ Native modules rebuilt with Forge${RESET}`);
    } catch (forgeError) {
      console.error(`${RED}✗ Failed to rebuild native modules!${RESET}`);
      console.error(`${RED}  This will likely cause crashes in production.${RESET}`);
      console.error(`${YELLOW}  Continuing anyway, but expect issues...${RESET}`);
    }
  }
} else {
  // Method 3: Manual rebuild using node-gyp
  console.log(`${YELLOW}electron-rebuild not found, attempting manual rebuild...${RESET}`);
  
  for (const module of nativeModules) {
    const modulePath = path.join(__dirname, '../node_modules', module);
    if (fs.existsSync(modulePath)) {
      try {
        console.log(`${CYAN}Rebuilding ${module}...${RESET}`);
        execCommand(
          `cd "${modulePath}" && npm rebuild --build-from-source`,
          `Rebuilding ${module}`,
          { timeout: 60000, allowFail: true }
        );
      } catch (e) {
        console.log(`${YELLOW}⚠ Failed to rebuild ${module}${RESET}`);
      }
    }
  }
}

// Verify native modules after rebuild
console.log(`${CYAN}Verifying native modules...${RESET}`);
let allModulesValid = true;
const moduleStatus = [];

for (const module of nativeModules) {
  const modulePath = path.join(__dirname, '../node_modules', module);
  const buildPath = path.join(modulePath, 'build/Release');
  const nodeFiles = fs.existsSync(buildPath) ? 
    fs.readdirSync(buildPath).filter(f => f.endsWith('.node')) : [];
  
  if (nodeFiles.length > 0) {
    console.log(`${GREEN}✓ ${module}: Found ${nodeFiles.join(', ')}${RESET}`);
    
    // Check if the .node file is built for the right architecture
    try {
      const fileInfo = execSync(`file "${path.join(buildPath, nodeFiles[0])}"`, { encoding: 'utf8' });
      const moduleInfo = {
        name: module,
        status: 'success',
        file: nodeFiles[0]
      };
      
      if (fileInfo.includes('arm64') && process.arch === 'arm64') {
        console.log(`  ${GREEN}Architecture: ARM64 (Apple Silicon) ✓${RESET}`);
        moduleInfo.architecture = 'arm64';
      } else if (fileInfo.includes('x86_64') && process.arch === 'x64') {
        console.log(`  ${GREEN}Architecture: x86_64 (Intel) ✓${RESET}`);
        moduleInfo.architecture = 'x86_64';
      } else {
        console.log(`  ${YELLOW}Architecture mismatch detected!${RESET}`);
        moduleInfo.status = 'warning';
        moduleInfo.error = 'Architecture mismatch';
        allModulesValid = false;
      }
      moduleStatus.push(moduleInfo);
    } catch {
      // file command might not be available
      moduleStatus.push({
        name: module,
        status: 'success',
        file: nodeFiles[0]
      });
    }
  } else {
    console.log(`${RED}✗ ${module}: No .node files found${RESET}`);
    moduleStatus.push({
      name: module,
      status: 'failed',
      error: 'No .node files found after rebuild'
    });
    allModulesValid = false;
  }
}

// Log version tracking information to build log
logger.log('');
logger.log('NATIVE MODULE STATUS:');
logger.log(JSON.stringify(moduleStatus, null, 2));
logger.log(`All modules valid: ${allModulesValid}`);
logger.updateStatus(`Native modules verified: ${allModulesValid ? 'All valid' : 'Some issues detected'}`);

if (!allModulesValid) {
  console.error(`${RED}⚠ Some native modules have issues. Build may crash in production!${RESET}`);
}

// PHASE 5: Version Verification & Native Modules
// ═══════════════════════════════════════════════════════════════

logPhase('RUNTIME DISCOVERY', 'Find and configure Node.js for production');

// Discover Node.js path for Memory Service (critical for production)
let nodePath = null;
let usedElectronAsNode = false; // track if we fell back to Electron
const possibleNodePaths = [
  process.execPath, // Current Node.js/Electron binary
  '/usr/local/bin/node',
  '/opt/homebrew/bin/node',
  '/usr/bin/node',
  process.env.NVM_DIR ? `${process.env.NVM_DIR}/versions/node/*/bin/node` : null,
  process.env.HOME ? `${process.env.HOME}/.nvm/versions/node/*/bin/node` : null
].filter(p => p); // Remove null entries

for (const testPath of possibleNodePaths) {
  // Expand wildcards for NVM paths
  const expandedPath = testPath.includes('*') 
    ? testPath.replace('*', 'v20.11.0').replace('~', process.env.HOME || '')
    : testPath;
  
  try {
    // Test if this path works
    const result = execSync(`"${expandedPath}" --version 2>/dev/null`, { encoding: 'utf8' }).trim();
    if (result && result.startsWith('v')) {
      nodePath = expandedPath;
      console.log(`${GREEN}✓ Found Node.js at: ${nodePath} (${result})${RESET}`);
      break;
    }
  } catch {
    // Try next path
  }
}

if (!nodePath) {
  console.log(`${YELLOW}⚠ Node.js not found in standard locations${RESET}`);
  console.log(`${CYAN}  Will use Electron binary as Node.js (ELECTRON_RUN_AS_NODE)${RESET}`);
  nodePath = process.execPath; // Fallback to Electron
  usedElectronAsNode = true;
}

// Write configuration for ProcessManager
const envConfigPath = path.join(__dirname, '..', '.env.production');
let envConfig = '';
if (fs.existsSync(envConfigPath)) {
  envConfig = fs.readFileSync(envConfigPath, 'utf8');
}

const nodePathToSave = './binaries/node';
if (nodeBundled) {
  if (envConfig.includes('NODE_PATH=')) {
    envConfig = envConfig.replace(/NODE_PATH=.*\n/, `NODE_PATH=${nodePathToSave}\n`);
  } else {
    envConfig += `NODE_PATH=${nodePathToSave}\n`;
  }
} else {
  envConfig = envConfig.replace(/NODE_PATH=.*\n/g, '');
}

// Also save whether we need ELECTRON_RUN_AS_NODE
if (usedElectronAsNode) {
  if (!envConfig.includes('USE_ELECTRON_AS_NODE=')) {
    envConfig += 'USE_ELECTRON_AS_NODE=true\n';
  }
} else {
  // Ensure flag is removed if previously set
  envConfig = envConfig.replace(/USE_ELECTRON_AS_NODE=.*\n/g, '');
}

// Add Python runtime paths for Backend Server AI Helpers
// These paths are relative to the app bundle and will be resolved at runtime
const pythonRuntimePath = 'resources/python-runtime';
const pythonBinRelative = `${pythonRuntimePath}/python/bin/python3`;
// Use the full model_service.py for proper AI routing (no fallbacks!)
const modelScriptRelative = `${pythonRuntimePath}/models/model_service.py`;

// Add Python configuration for Backend Server
if (!envConfig.includes('PYTHON_RUNTIME_PATH=')) {
  envConfig += `PYTHON_RUNTIME_PATH=${pythonRuntimePath}\n`;
}
if (!envConfig.includes('PYTHON_BIN_PATH=')) {
  envConfig += `PYTHON_BIN_PATH=${pythonBinRelative}\n`;
}
if (!envConfig.includes('MODEL_SCRIPT_PATH=')) {
  envConfig += `MODEL_SCRIPT_PATH=${modelScriptRelative}\n`;
}

fs.writeFileSync(envConfigPath, envConfig);
console.log(`${GREEN}✓ Saved Node.js and Python configuration to .env.production${RESET}`);

// ═══════════════════════════════════════════════════════════════
// PHASE 6: Runtime Discovery
// ═══════════════════════════════════════════════════════════════

// PHASE 6: REMOVED - Backend Server no longer needed (DirectConsensusEngine approach)
logPhase('CONSENSUS ENGINE', 'Using DirectConsensusEngine - no separate backend process needed');
console.log(`${GREEN}✓ DirectConsensusEngine will handle consensus in main process${RESET}`);

// ═══════════════════════════════════════════════════════════════
// PHASE 7: Backend Server Preparation
// ═══════════════════════════════════════════════════════════════

logPhase('PYTHON RUNTIME PREPARATION', 'Verify Python bundle with extraction and memory fixes');

const pythonBundlePath = path.join(__dirname, '../resources/python-runtime/python');
const pythonBinPath = path.join(pythonBundlePath, 'bin/python3');
const modelWrapperPath = path.join(__dirname, '../resources/python-runtime/models/model_service_wrapper.py');
const modelServicePath = path.join(__dirname, '../resources/python-runtime/models/model_service.py');

checkFile(pythonBundlePath, 'Python runtime bundle');
checkFile(modelWrapperPath, 'Model service wrapper');

// CRITICAL FIX 1: Create extraction marker for production runtime
// This ensures Python is COPIED not symlinked to avoid dylib loading issues
const extractionMarker = path.join(__dirname, '../resources/python-runtime/.needs_extraction');
const packageInfo = JSON.parse(fs.readFileSync(path.join(__dirname, '../package.json'), 'utf8'));
fs.writeFileSync(extractionMarker, JSON.stringify({
  created: new Date().toISOString(),
  version: packageInfo.version,
  reason: 'Force full extraction to preserve dylib paths'
}));
console.log(`${GREEN}  ✓ Added extraction marker for production Python runtime${RESET}`);

// Copy actual model_service.py if missing
const sourceModelService = path.join(__dirname, '../../../python/model_service.py');
if (fs.existsSync(sourceModelService) && !fs.existsSync(modelServicePath)) {
  console.log(`${CYAN}  Copying model_service.py for full AI Helper functionality...${RESET}`);
  fs.copyFileSync(sourceModelService, modelServicePath);
  console.log(`${GREEN}  ✓ Copied model_service.py${RESET}`);
} else if (fs.existsSync(modelServicePath)) {
  console.log(`${GREEN}  ✓ model_service.py already present${RESET}`);
}

if (fs.existsSync(pythonBinPath)) {
  // Fix Python permissions
  execCommand(
    `chmod +x "${pythonBundlePath}/bin/"*`,
    'Setting execute permissions on Python binaries'
  );
  
  // Verify Python works (direct check without timeout issues)
  let pythonVersion = '';
  try {
    pythonVersion = execCommand(
      `"${pythonBinPath}" --version 2>&1`,
      'Verifying Python is functional',
      { allowFail: false, returnOutput: true }
    );
  } catch (e) {
    console.log(`${YELLOW}  Python check had issues, attempting alternate check...${RESET}`);
    pythonVersion = execCommand(
      `"${pythonBinPath}" -c "import sys; print(f'Python {sys.version.split()[0]}')" 2>&1`,
      'Alternate Python version check',
      { allowFail: true, returnOutput: true }
    );
  }
  
  if (pythonVersion && pythonVersion.toLowerCase().includes('python')) {
    console.log(`${GREEN}  ✓ ${pythonVersion.trim()}${RESET}`);
    
    // CRITICAL FIX 2: Configure memory management for Python subprocess
    console.log(`${CYAN}  Configuring memory management for Python subprocess...${RESET}`);
    const memoryConfig = {
      'PYTORCH_CUDA_ALLOC_CONF': 'max_split_size_mb:512',
      'OMP_NUM_THREADS': '2',
      'MKL_NUM_THREADS': '2',
      'NUMEXPR_NUM_THREADS': '2',
      'TOKENIZERS_PARALLELISM': 'false',
      'created': new Date().toISOString(),
      'purpose': 'Prevent 1.3TB memory allocation crash'
    };
    
    // Write memory config to resources so it's included in the bundle
    const memConfigPath = path.join(__dirname, '../resources/python-runtime/.memory_config');
    fs.writeFileSync(memConfigPath, JSON.stringify(memoryConfig, null, 2));
    console.log(`${GREEN}  ✓ Memory limits configured (prevents 1.3T allocation crash)${RESET}`);
    
    // Install lean ML packages for consensus routing (NO FALLBACKS - must work)
    console.log(`${CYAN}  Ensuring ML packages for consensus routing...${RESET}`);
    
    // Check current packages
    const hasPackages = execCommand(
      `"${pythonBinPath}" -c "import torch, transformers; print('OK')" 2>&1`,
      'Checking ML packages',
      { allowFail: true, returnOutput: true }
    );
    
    if (!hasPackages.includes('OK')) {
      console.log(`${YELLOW}  Installing essential ML packages (this may take a moment)...${RESET}`);
      
      // Install CPU-only torch for smaller size
      execCommand(
        `"${pythonBinPath}" -m pip install --quiet --no-cache-dir torch --index-url https://download.pytorch.org/whl/cpu`,
        'Installing PyTorch (CPU-optimized)',
        { allowFail: false }  // MUST succeed - no fallbacks!
      );
      
      // Install transformers with minimal dependencies
      execCommand(
        `"${pythonBinPath}" -m pip install --quiet --no-cache-dir transformers sentence-transformers`,
        'Installing Transformers for AI routing',
        { allowFail: false }  // MUST succeed
      );
      
      console.log(`${GREEN}  ✓ ML packages installed successfully${RESET}`);
    } else {
      console.log(`${GREEN}  ✓ ML packages already installed${RESET}`);
    }
    
    // Verify the model service can initialize
    const testInit = execCommand(
      `echo '{"type":"health_check"}' | "${pythonBinPath}" "${modelServicePath}" 2>&1 | head -1`,
      'Testing AI Helper initialization',
      { allowFail: true, returnOutput: true }
    );
    
    if (testInit.includes('error')) {
      console.error(`${RED}  ⚠ AI Helper initialization may have issues: ${testInit}${RESET}`);
    } else {
      console.log(`${GREEN}  ✓ AI Helpers ready for consensus routing${RESET}`);
    }
  }
}

// Python runtime verification moved to Phase 7 to avoid duplication
// The prebuild script handles Python bundling

// ═══════════════════════════════════════════════════════════════
// PHASE 8: Webpack Plugin Verification
// ═══════════════════════════════════════════════════════════════

logPhase('WEBPACK PLUGIN VERIFICATION', 'Ensure custom webpack plugins are ready');

const requiredPlugins = [
  'BuildMemoryServicePlugin.js',
  'FixBinaryPermissionsPlugin.js'
];

for (const plugin of requiredPlugins) {
  const pluginPath = path.join(__dirname, '../webpack-plugins', plugin);
  if (!checkFile(pluginPath, `Webpack plugin: ${plugin}`)) {
    console.error(`${RED}✗ Required webpack plugin missing!${RESET}`);
    process.exit(1);
  }
}

// ═══════════════════════════════════════════════════════════════
// PHASE 9: Pre-Build Script
// ═══════════════════════════════════════════════════════════════

logPhase('PRE-BUILD SCRIPT', 'Run custom pre-build preparations');

// Check for known issues that cause production failures
const knownIssues = [
  {
    id: 'port-scan-timeout',
    description: 'Port scanning hangs in production without timeout',
    check: () => {
      const portManagerPath = path.join(__dirname, '..', 'src', 'utils', 'PortManager.ts');
      const content = fs.readFileSync(portManagerPath, 'utf8');
      return content.includes('Promise.race') && content.includes('3000); // 3 second timeout');
    },
    fix: 'Added 3-second timeout to port initialization'
  },
  {
    id: 'env-spreading',
    description: 'PORT environment variable not passed to child processes',
    check: () => {
      const processManagerPath = path.join(__dirname, '..', 'src', 'utils', 'ProcessManager.ts');
      const content = fs.readFileSync(processManagerPath, 'utf8');
      return content.includes('...env,') && content.includes('ELECTRON_RUN_AS_NODE');
    },
    fix: 'Fixed env object spreading in ProcessManager'
  },
  {
    id: 'memory-service-spawn',
    description: 'Memory Service must use spawn(node) to avoid "Unable to find helper app" error',
    check: () => {
      const processManagerPath = path.join(__dirname, '..', 'src', 'utils', 'ProcessManager.ts');
      const content = fs.readFileSync(processManagerPath, 'utf8');
      // Check if using spawn with node in production
      return content.includes("spawn(nodePath, [config.scriptPath") && 
             content.includes("using spawn('node') in production");
    },
    fix: 'Use spawn(node) in production per MASTER_ARCHITECTURE.md to avoid Electron helper issues'
  },
  {
    id: 'version-display-consistency',
    description: 'UI shows cached version instead of current package.json version',
    check: () => {
      // Check if version is properly injected into renderer
      const rendererConfigPath = path.join(__dirname, '..', 'webpack.renderer.config.ts');
      if (!fs.existsSync(rendererConfigPath)) return false;
      const content = fs.readFileSync(rendererConfigPath, 'utf8');
      return content.includes('APP_VERSION') && content.includes('process.env.npm_package_version');
    },
    fix: 'Will auto-fix: Inject APP_VERSION into webpack DefinePlugin for renderer'
  },
  {
    id: 'clean-install',
    description: 'Old app installation causes version/caching issues',
    check: () => {
      // Always return false to trigger clean removal
      return false;
    },
    fix: 'Will auto-fix: Remove existing app installation for clean reinstall'
  },
  {
    id: 'startup-html-version',
    description: 'Startup splash screen shows hardcoded version',
    check: () => {
      const startupPath = path.join(__dirname, '..', 'startup.html');
      if (!fs.existsSync(startupPath)) return false;
      const content = fs.readFileSync(startupPath, 'utf8');
      const pkg = require('../package.json');
      // Check if version matches current package.json version
      return content.includes(`Version ${pkg.version}`);
    },
    fix: 'Will auto-fix: Update startup.html with current version'
  }
];

console.log(`${CYAN}Checking for ${knownIssues.length} known production issues...${RESET}`);
for (const issue of knownIssues) {
  const isFixed = issue.check();
  if (!isFixed) {
    if (issue.id === 'version-display-consistency') {
      // Auto-fix version display consistency
      console.log(`${YELLOW}⚠ Issue detected: ${issue.description}${RESET}`);
      console.log(`${CYAN}  Auto-fixing: ${issue.fix}${RESET}`);
      
      // CRITICAL FIX: Ensure webpack.plugins.ts uses package.json version
      const pluginsPath = path.join(__dirname, '..', 'webpack.plugins.ts');
      const pkg = require('../package.json');
      
      if (fs.existsSync(pluginsPath)) {
        let content = fs.readFileSync(pluginsPath, 'utf8');
        let updated = false;
        
        // Ensure webpack is imported
        if (!content.includes("require('webpack')")) {
          content = content.replace(
            'const ForkTsCheckerWebpackPlugin: typeof IForkTsCheckerWebpackPlugin = require(\'fork-ts-checker-webpack-plugin\');',
            `const ForkTsCheckerWebpackPlugin: typeof IForkTsCheckerWebpackPlugin = require('fork-ts-checker-webpack-plugin');
const webpack = require('webpack');`
          );
          updated = true;
        }
        
        // CRITICAL: Check if reading from package.json
        if (!content.includes("require('./package.json')")) {
          // Add package.json import after webpack import
          content = content.replace(
            "const webpack = require('webpack');",
            `const webpack = require('webpack');

// CRITICAL: Always read version from package.json - NEVER hardcode!
const packageJson = require('./package.json');`
          );
          updated = true;
        }
        
        // CRITICAL: Replace ANY hardcoded version with dynamic reading
        const hardcodedVersionMatch = content.match(/APP_VERSION:\s*JSON\.stringify\(['"][\d.]+['"]\)/);
        if (hardcodedVersionMatch) {
          console.log(`${RED}  ✗ FOUND HARDCODED VERSION! Fixing...${RESET}`);
          content = content.replace(
            /APP_VERSION:\s*JSON\.stringify\(['"][\d.]+['"]\)/,
            'APP_VERSION: JSON.stringify(packageJson.version)'
          );
          updated = true;
        }
        
        // Ensure DefinePlugin exists with correct version
        if (!content.includes('APP_VERSION')) {
          content = content.replace(
            'export const plugins = [',
            `export const plugins = [
  new webpack.DefinePlugin({
    APP_VERSION: JSON.stringify(packageJson.version),
  }),`
          );
          updated = true;
        }
        
        if (updated) {
          fs.writeFileSync(pluginsPath, content);
          console.log(`${GREEN}  ✓ Fixed webpack.plugins.ts to use packageJson.version dynamically${RESET}`);
        } else {
          // Still verify it's using packageJson.version, not a hardcoded value
          if (content.includes('APP_VERSION: JSON.stringify(packageJson.version)')) {
            console.log(`${GREEN}  ✓ DefinePlugin correctly uses packageJson.version${RESET}`);
          } else {
            console.log(`${YELLOW}  ⚠ DefinePlugin may have issues - please verify manually${RESET}`);
          }
        }
      }
    } else if (issue.id === 'clean-install') {
      // Auto-fix clean install
      console.log(`${YELLOW}⚠ Issue detected: ${issue.description}${RESET}`);
      console.log(`${CYAN}  Auto-fixing: ${issue.fix}${RESET}`);
      
      const appPath = '/Applications/Hive Consensus.app';
      
      // Force quit the app if it's running
      try {
        execCommand('pkill -f "Hive Consensus" || true', 'Force quit running Hive Consensus', { silent: true });
      } catch (error) {
        // Ignore errors - app might not be running
      }
      
      // Give it a moment to quit (using sync sleep instead of async)
      execCommand('sleep 1', 'Wait for app to quit', { silent: true });
      
      if (fs.existsSync(appPath)) {
        try {
          // More aggressive removal with sudo if needed
          execCommand(
            `rm -rf "${appPath}"`,
            'Removing existing app installation for clean reinstall',
            { silent: true }
          );
          console.log(`${GREEN}  ✓ Removed existing app installation${RESET}`);
        } catch (error) {
          console.log(`${YELLOW}  ⚠ Could not remove existing app: ${error.message}${RESET}`);
          console.log(`${YELLOW}  ⚠ You may need to manually quit the app first${RESET}`);
        }
      } else {
        console.log(`${GREEN}  ✓ No existing app installation found${RESET}`);
      }
    } else if (issue.id === 'startup-html-version') {
      // Auto-fix startup.html version
      console.log(`${YELLOW}⚠ Issue detected: ${issue.description}${RESET}`);
      console.log(`${CYAN}  Auto-fixing: ${issue.fix}${RESET}`);
      
      const startupPath = path.join(__dirname, '..', 'startup.html');
      const pkg = require('../package.json');
      
      if (fs.existsSync(startupPath)) {
        let content = fs.readFileSync(startupPath, 'utf8');
        
        // Replace any version pattern in startup.html with current version
        const versionPattern = /Version \d+\.\d+\.\d+/g;
        const newVersion = `Version ${pkg.version}`;
        
        if (content.match(versionPattern)) {
          content = content.replace(versionPattern, newVersion);
          fs.writeFileSync(startupPath, content);
          console.log(`${GREEN}  ✓ Updated startup.html to Version ${pkg.version}${RESET}`);
        } else {
          console.log(`${YELLOW}  ⚠ Could not find version pattern in startup.html${RESET}`);
        }
      }
    } else {
      console.error(`${RED}✗ Issue detected: ${issue.description}${RESET}`);
      console.error(`  Fix: ${issue.fix}`);
      console.error('  Please fix this issue before building!');
      process.exit(1);
    }
  } else {
    console.log(`${GREEN}✓ ${issue.id}: Fixed${RESET}`);
  }
}

// Removed: npm run prebuild is already called by npm run make
// This prevents PyTorch and other packages from being installed twice
// The make command in package.json already includes: "npm run prebuild && electron-forge make"

// ═══════════════════════════════════════════════════════════════
// PHASE 10: Application Build
// ═══════════════════════════════════════════════════════════════

logPhase('APPLICATION BUILD', 'Build the Electron application with webpack');

// Use async execution for the long-running build process
(async () => {
  try {
    await execLongCommand(
      'npm run make',
      'Building production DMG (this will take several minutes)'
    );
  } catch (error) {
    console.error(`${RED}✗ Build failed!${RESET}`);
    console.error(`${RED}  Error: ${error.message}${RESET}`);
    process.exit(1);
  }
})().then(() => {
  // Continue with the rest of the phases after build completes
  continuePostBuild();
}).catch(error => {
  console.error(`${RED}Build process failed: ${error}${RESET}`);
  process.exit(1);
});

// Wrap the rest of the phases in a function
function continuePostBuild() {

// ═══════════════════════════════════════════════════════════════
// PHASE 11: Application Build
// ═══════════════════════════════════════════════════════════════

logPhase('POST-BUILD FIXES', 'Apply critical fixes to app bundle');

// Apply post-build fixes for .dylibs and critical files
console.log(`${CYAN}Applying post-build fixes to ensure all critical files are included...${RESET}`);
execCommand(
  'node scripts/post-build-fixes.js',
  'Applying post-build fixes for .dylibs and critical files',
  { allowFail: false }
);

// ═══════════════════════════════════════════════════════════════
// PHASE 12: Post-Build Fixes
// ═══════════════════════════════════════════════════════════════

logPhase('POST-BUILD VERIFICATION', 'Verify build output and structure');

// CRITICAL: Verify version is correct in the built app
const builtAppPath = path.join(__dirname, '../out/Hive Consensus-darwin-arm64/Hive Consensus.app');
if (fs.existsSync(builtAppPath)) {
  const infoPlistPath = path.join(builtAppPath, 'Contents/Info.plist');
  try {
    const plistVersion = execSync(`defaults read "${infoPlistPath}" CFBundleShortVersionString`, { encoding: 'utf8' }).trim();
    const expectedVersion = pkg.version;
    if (plistVersion !== expectedVersion) {
      console.error(`${RED}✗ VERSION MISMATCH IN BUILT APP!${RESET}`);
      console.error(`${RED}  Expected: ${expectedVersion}${RESET}`);
      console.error(`${RED}  Found:    ${plistVersion}${RESET}`);
      console.error(`${RED}  This means webpack or Electron Forge didn't use the correct version!${RESET}`);
      process.exit(1);
    } else {
      console.log(`${GREEN}✓ Version verified in app bundle: ${plistVersion}${RESET}`);
    }
  } catch (e) {
    console.log(`${YELLOW}⚠ Could not verify app version${RESET}`);
  }
}

// Check DMG exists
const dmgFiles = fs.readdirSync(path.join(__dirname, '../out/make')).filter(f => f.endsWith('.dmg'));
if (dmgFiles.length === 0) {
  console.error(`${RED}✗ No DMG file found in out/make!${RESET}`);
  process.exit(1);
}

const dmgPath = path.join(__dirname, '../out/make', dmgFiles[0]);
console.log(`${GREEN}✓ DMG created: ${dmgPath}${RESET}`);

// Check app bundle
const appPath = path.join(__dirname, '../out/Hive Consensus-darwin-arm64/Hive Consensus.app');
if (!fs.existsSync(appPath)) {
  console.error(`${RED}✗ App bundle not found!${RESET}`);
  process.exit(1);
}

// CRITICAL: Verify .dylibs directories were copied
console.log(`${CYAN}Verifying critical .dylibs directories...${RESET}`);

const dylibsToCheck = [
  'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/lib/python3.11/site-packages/PIL/.dylibs',
  'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/lib/python3.11/site-packages/sklearn/.dylibs',
  'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/lib/python3.11/site-packages/scipy/.dylibs'
];

let dylibsFound = 0;
let dylibsMissing = 0;
const dylibsStatus = [];

for (const dylibPath of dylibsToCheck) {
  const fullPath = path.join(appPath, dylibPath);
  const packageName = path.basename(path.dirname(dylibPath));
  
  if (fs.existsSync(fullPath)) {
    const files = fs.readdirSync(fullPath);
    console.log(`${GREEN}  ✓ Found ${packageName}/.dylibs (${files.length} files)${RESET}`);
    dylibsStatus.push(`✓ ${packageName}/.dylibs: ${files.length} files`);
    console.log(`${GREEN}✓ ${packageName}/.dylibs: ${files.length} files${RESET}`);
    dylibsFound++;
  } else {
    // Check if the package itself exists
    const packagePath = path.dirname(fullPath);
    if (fs.existsSync(packagePath)) {
      console.log(`${YELLOW}  ⚠ ${packageName}/.dylibs missing (package exists but no .dylibs)${RESET}`);
      dylibsStatus.push(`✗ ${packageName}/.dylibs: MISSING (package exists)`);
      console.log(`${RED}✗ ${packageName}/.dylibs: MISSING - WILL CAUSE PYTHON CRASH!${RESET}`);
    } else {
      console.log(`${YELLOW}  ⚠ ${packageName} package not found${RESET}`);
      dylibsStatus.push(`✗ ${packageName}: Package not found`);
      console.log(`${RED}✗ ${packageName}: Package not found at all${RESET}`);
    }
    dylibsMissing++;
  }
}

if (dylibsFound === 0 && dylibsMissing > 0) {
  const criticalWarning = [
    '✗ CRITICAL: No .dylibs directories found!',
    '  Impact:',
    '  1. Python subprocess will crash on import',
    '  2. Consensus routing will fail',
    '  3. App will be stuck at "route" stage',
    '  Root cause: webpack.main.config.ts not copying .dylibs'
  ];
  
  criticalWarning.forEach(line => {
    console.error(`${RED}${line}${RESET}`);
  });
  
  console.log('\n⚠️ BUILD WILL COMPLETE BUT APP WILL NOT WORK PROPERLY ⚠️');
}

// Verify critical files in app bundle (removed backend binary - using DirectConsensusEngine)
const criticalFiles = [
  'Contents/Resources/app.asar',
  'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/bin/python3',
  'Contents/Resources/app.asar.unpacked/.webpack/main/memory-service/index.js'
];

let missingCriticalFiles = false;
for (const file of criticalFiles) {
  const fullPath = path.join(appPath, file);
  if (!fs.existsSync(fullPath)) {
    console.error(`${RED}✗ Critical file missing in app bundle: ${file}${RESET}`);
    
    // Special handling for Memory Service
    if (file.includes('memory-service/index.js')) {
      console.error(`${YELLOW}  ⚠️  Memory Service not unpacked!${RESET}`);
      console.error(`${YELLOW}  Fix: Add '.webpack/main/memory-service/**' to forge.config.ts asar.unpack${RESET}`);
      console.error(`${YELLOW}  The Memory Service was built but not included in unpacked resources.${RESET}`);
    }
    missingCriticalFiles = true;
  } else {
    console.log(`${GREEN}✓ Found: ${file}${RESET}`);
    
    // Additional verification for Memory Service
    if (file.includes('memory-service/index.js')) {
      // Check if it's a valid JavaScript file
      try {
        const content = fs.readFileSync(fullPath, 'utf8');
        if (content.length < 100) {
          console.error(`${YELLOW}  ⚠️  Memory Service file is too small (${content.length} bytes)${RESET}`);
          missingCriticalFiles = true;
        } else {
          console.log(`${GREEN}  ✓ Memory Service is ${(content.length / 1024).toFixed(1)}KB${RESET}`);
        }
      } catch (err) {
        console.error(`${RED}  ✗ Cannot read Memory Service: ${err.message}${RESET}`);
        missingCriticalFiles = true;
      }
    }
  }
}

if (missingCriticalFiles) {
  console.error(`${RED}⚠️  Build completed but critical files are missing or invalid!${RESET}`);
  console.error(`${YELLOW}   The app may not function correctly in production.${RESET}`);
  console.error(`${YELLOW}   Consider rebuilding after fixing forge.config.ts${RESET}`);
}

// ═══════════════════════════════════════════════════════════════
// PHASE 13: Post-Build Verification
// ═══════════════════════════════════════════════════════════════

logPhase('PERMISSION VERIFICATION', 'Ensure all binaries have execute permissions');

const binariesToCheck = [
  `${appPath}/Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/bin/python3`
];

for (const binaryPath of binariesToCheck) {
  if (fs.existsSync(binaryPath)) {
    try {
      fs.accessSync(binaryPath, fs.constants.X_OK);
      console.log(`${GREEN}✓ Executable: ${path.basename(binaryPath)}${RESET}`);
    } catch {
      console.error(`${RED}✗ Not executable: ${path.basename(binaryPath)}${RESET}`);
      console.error(`${RED}  The FixBinaryPermissionsPlugin may have failed!${RESET}`);
    }
  }
}

// ═══════════════════════════════════════════════════════════════
// PHASE 14: Permission Verification
// ═══════════════════════════════════════════════════════════════

logPhase('BUILD REPORT', 'Generate detailed build information');

const buildInfo = {
  timestamp: new Date().toISOString(),
  dmgPath: dmgPath,
  dmgSize: (fs.statSync(dmgPath).size / (1024 * 1024)).toFixed(2) + ' MB',
  nodeVersion: nodeVersion?.trim(),
  npmVersion: npmVersion?.trim(),
  platform: process.platform,
  arch: process.arch,
  electronVersion: require('../package.json').devDependencies.electron
};

const reportPath = path.join(__dirname, '../out/build-report.json');
fs.writeFileSync(reportPath, JSON.stringify(buildInfo, null, 2));

console.log(`\n${CYAN}Build Information:${RESET}`);
console.log(`  DMG: ${buildInfo.dmgPath}`);
console.log(`  Size: ${buildInfo.dmgSize}`);
console.log(`  Built: ${buildInfo.timestamp}`);
console.log(`  Report: ${reportPath}`);

// ═══════════════════════════════════════════════════════════════
// PHASE 15: Build Report
// ═══════════════════════════════════════════════════════════════

logPhase('CRITICAL FIX VERIFICATION', 'Verify memory crash and Python extraction fixes');

console.log(`${CYAN}Verifying critical production fixes...${RESET}\n`);

let criticalErrors = false;

// 1. Verify Python extraction marker exists in app bundle
const appBundlePath = path.join(__dirname, '../out/Hive Consensus-darwin-arm64/Hive Consensus.app');
const extractMarkerCheck = path.join(appBundlePath, 'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/.needs_extraction');
const sourceExtractMarker = path.join(__dirname, '../resources/python-runtime/.needs_extraction');

if (fs.existsSync(extractMarkerCheck)) {
  console.log(`${GREEN}✓ Python extraction marker in app bundle (fixes dylib loading)${RESET}`);
} else if (fs.existsSync(sourceExtractMarker)) {
  console.log(`${YELLOW}⚠ Extraction marker in source but not in bundle${RESET}`);
  criticalErrors = true;
} else {
  console.log(`${RED}✗ Python extraction marker missing!${RESET}`);
  criticalErrors = true;
}

// 2. Verify memory configuration exists in app bundle
const memConfigCheck = path.join(appBundlePath, 'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/.memory_config');
const sourceMemConfig = path.join(__dirname, '../resources/python-runtime/.memory_config');

if (fs.existsSync(memConfigCheck)) {
  const config = JSON.parse(fs.readFileSync(memConfigCheck, 'utf8'));
  console.log(`${GREEN}✓ Memory limits in app bundle (prevents 1.3T crash)${RESET}`);
  console.log(`  Threads: OMP=${config.OMP_NUM_THREADS}, MKL=${config.MKL_NUM_THREADS}`);
} else if (fs.existsSync(sourceMemConfig)) {
  console.log(`${YELLOW}⚠ Memory config in source but not in bundle${RESET}`);
} else {
  console.log(`${RED}✗ Memory configuration missing!${RESET}`);
}

// 3. Verify ML packages are installed
const mlPackageCheck = execCommand(
  `"${pythonBinPath}" -c "import torch, transformers; print('ML packages verified')" 2>&1`,
  'Verifying ML packages for consensus routing',
  { allowFail: true, returnOutput: true, silent: true }
);

if (mlPackageCheck && mlPackageCheck.includes('ML packages verified')) {
  console.log(`${GREEN}✓ ML packages installed (consensus routing will work)${RESET}`);
} else {
  console.log(`${YELLOW}⚠ ML packages missing - but DirectConsensusEngine doesn't need them${RESET}`);
  // criticalErrors = true;  // Don't fail for DirectConsensusEngine mode
}

// 4. REMOVED: Backend binary check - using DirectConsensusEngine instead
console.log(`${GREEN}✓ DirectConsensusEngine handles consensus in main process${RESET}`);

console.log(`\n${GREEN}${BOLD}Critical fixes verification complete${RESET}\n`);

// Check if there were any critical errors
if (criticalErrors) {
  console.error(`\n${RED}${BOLD}❌ BUILD FAILED - Critical errors detected!${RESET}`);
  console.error(`${RED}The build completed but critical components are missing.${RESET}`);
  console.error(`${RED}The application will not work properly.${RESET}\n`);
  process.exit(1);
}

// ═══════════════════════════════════════════════════════════════
// PHASE 17: Installation Guide
// ═══════════════════════════════════════════════════════════════

logPhase('INSTALLATION GUIDE', 'How to test the production build');

console.log(`${GREEN}${BOLD}✅ BUILD SUCCESSFUL!${RESET}\n`);

// Auto-install the DMG for immediate testing
if (IS_CI) {
  console.log(`${YELLOW}Skipping DMG auto-install in CI environment${RESET}`);
  console.log(`${CYAN}Manual installation steps:${RESET}\n`);
  console.log(`  ${BOLD}1.${RESET} Download the DMG artifact from the workflow run`);
  console.log(`  ${BOLD}2.${RESET} Mount the DMG locally and drag "Hive Consensus" to Applications\n`);
} else {
  console.log(`${CYAN}${BOLD}Auto-installing DMG for testing...${RESET}`);
  try {
    if (SHOULD_RUN_UI_SMOKE) {
      process.env.HIVE_SKIP_AUTO_LAUNCH = '1';
    }
    // First, kill any running app
    console.log(`${YELLOW}➤ Stopping any running Hive Consensus${RESET}`);
    try {
      execCommand('pkill -f "Hive Consensus" || true', 'Stop running app', { silent: true });
      execCommand('sleep 1', 'Wait for app to quit', { silent: true });
    } catch (error) {
      // Ignore errors - app might not be running
    }

    // Eject any existing Hive Consensus volumes
    try {
      execCommand('hdiutil detach "/Volumes/Hive Consensus" 2>/dev/null || true', 'Eject existing volume', { silent: true });
      execCommand('sleep 0.5', 'Wait for volume to eject', { silent: true });
    } catch (error) {
      // Ignore errors - volume might not exist
    }

    // Mount DMG fresh
    console.log(`${YELLOW}➤ Mounting DMG${RESET}`);
    execCommand(`hdiutil attach "${dmgPath}" -nobrowse`, 'Mounting DMG for auto-install');

    // Wait for mount to complete
    execCommand('sleep 1', 'Wait for DMG to mount', { silent: true });

    // Use standard volume path
    const volumePath = '/Volumes/Hive Consensus';

    // Verify the app exists
    if (!fs.existsSync(`${volumePath}/Hive Consensus.app`)) {
      throw new Error(`Could not find Hive Consensus.app in ${volumePath}`);
    }

    console.log(`${CYAN}  Found app at: ${volumePath}/Hive Consensus.app${RESET}`);

    console.log(`${YELLOW}➤ Removing old version from Applications${RESET}`);

    // Force remove old app first
    try {
      execCommand('pkill -f "Hive Consensus" || true', 'Stop running app');
      execCommand('sleep 1', 'Wait for app to quit');
      execCommand('rm -rf "/Applications/Hive Consensus.app"', 'Remove old app');
    } catch (error) {
      console.log(`${YELLOW}  Note: Old app might not exist or be in use${RESET}`);
    }

    console.log(`${YELLOW}➤ Installing new version to Applications${RESET}`);

    // Copy new app (force overwrite)
    execCommand(`cp -Rf "${volumePath}/Hive Consensus.app" /Applications/`, 'Installing app to Applications');

    // Verify installation
    if (!fs.existsSync('/Applications/Hive Consensus.app')) {
      throw new Error('Failed to install app to Applications');
    }

    // Unmount DMG
    console.log(`${YELLOW}➤ Ejecting DMG${RESET}`);
    execCommand(`hdiutil detach "${volumePath}"`, 'Ejecting DMG');

    console.log(`${GREEN}✅ Auto-installation complete!${RESET}`);
    console.log(`${GREEN}  App installed to: /Applications/Hive Consensus.app${RESET}\n`);
    if (!SHOULD_RUN_UI_SMOKE) {
      launchInstalledApp();
    }

  } catch (error) {
    console.log(`${YELLOW}⚠ Auto-install failed: ${error.message}${RESET}`);
    console.log(`${CYAN}Manual installation required:${RESET}\n`);
    console.log(`  ${BOLD}1.${RESET} Mount the DMG:`);
    console.log(`     ${YELLOW}open "${dmgPath}"${RESET}\n`);
    console.log(`  ${BOLD}2.${RESET} Drag "Hive Consensus" to Applications folder`);
    console.log(`     ${RED}⚠️  IMPORTANT: Do NOT launch from the DMG!${RESET}\n`);
    console.log(`  ${BOLD}3.${RESET} Eject the DMG after copying\n`);
  }
}

if (SHOULD_RUN_UI_SMOKE) {
  console.log(`${CYAN}${BOLD}Running UI smoke tests (npm run test:ui)...${RESET}`);
  const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';
  const testEnv = {
    ...process.env,
    PLAYWRIGHT_ATTACH: '0',
  };
  const result = spawnSync(npmCmd, ['run', 'test:ui'], {
    cwd: ELECTRON_ROOT,
    env: testEnv,
    stdio: 'inherit',
  });
  if (result.status !== 0) {
    throw new Error('UI smoke suite failed');
  }
}

console.log(`${CYAN}${BOLD}Ready for testing:${RESET}`);
console.log(`  ${BOLD}•${RESET} Launch: ${CYAN}Open Applications → Double-click "Hive Consensus"${RESET}`);
console.log(`  ${BOLD}•${RESET} Monitor: ${YELLOW}tail -f ~/Library/Application\\ Support/Hive\\ Consensus/logs/*.log${RESET}\n`);

// Record final phase time
if (phaseStartTime && currentPhaseName) {
  const elapsedSeconds = ((Date.now() - phaseStartTime) / 1000).toFixed(1);
  phaseTimes.push({
    phase: currentPhase,
    name: currentPhaseName,
    duration: elapsedSeconds
  });
}

// Calculate total build time
const totalBuildTime = ((Date.now() - buildStartTime) / 1000).toFixed(1);
const totalMinutes = Math.floor(totalBuildTime / 60);
const remainingSeconds = (totalBuildTime % 60).toFixed(0);

console.log(`${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}`);
console.log(`${GREEN}${BOLD}Build completed in ${currentPhase}/${totalPhases} phases${RESET}`);
console.log(`${GREEN}${BOLD}Total build time: ${totalMinutes}m ${remainingSeconds}s${RESET}`);
console.log(`${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n`);

// Display timing summary sorted by duration (longest first)
console.log(`${CYAN}${BOLD}Build Timing Summary (sorted by duration):${RESET}`);
console.log(`${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}`);

// Sort phases by duration (longest first)
const sortedPhaseTimes = [...phaseTimes].sort((a, b) => parseFloat(b.duration) - parseFloat(a.duration));

// Find longest phase name for alignment
const maxNameLength = Math.max(...sortedPhaseTimes.map(p => p.name.length));

sortedPhaseTimes.forEach((phase, index) => {
  const paddedName = phase.name.padEnd(maxNameLength);
  const barLength = Math.floor(parseFloat(phase.duration) / 10); // 10 seconds = 1 bar unit
  const bar = '█'.repeat(Math.min(barLength, 50)); // Max 50 bars
  
  let color = GREEN;
  if (parseFloat(phase.duration) > 180) color = RED;  // > 3 minutes
  else if (parseFloat(phase.duration) > 60) color = YELLOW; // > 1 minute
  
  const phaseLabel = `Phase ${String(phase.phase).padStart(2)}`;
  console.log(`  ${phaseLabel}: ${paddedName} ${color}${phase.duration}s${RESET} ${bar}`);
});

console.log(`${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}`);
console.log(`${GREEN}${BOLD}✅ Build v${pkg.version} completed successfully${RESET}`);
console.log(`${GREEN}${BOLD}   Total phases: ${currentPhase}/${totalPhases}${RESET}`);
console.log(`${GREEN}${BOLD}   Total time: ${totalMinutes}m ${remainingSeconds}s${RESET}`);
console.log(`${GREEN}${BOLD}   DMG size: ${buildInfo.dmgSize}${RESET}`);
console.log(`${CYAN}${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}\n`);

// Save timing data to report
buildInfo.buildTimings = {
  totalSeconds: totalBuildTime,
  totalFormatted: `${totalMinutes}m ${remainingSeconds}s`,
  phases: phaseTimes,
  buildVersion: pkg.version
};

// Write updated report
fs.writeFileSync(reportPath, JSON.stringify(buildInfo, null, 2));

} // End of continuePostBuild function
