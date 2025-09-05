#!/usr/bin/env node

// Test script to verify Qwen Code integration is complete and working

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🧪 Testing Qwen Code Integration\n');

// Color codes for output
const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const YELLOW = '\x1b[33m';
const RESET = '\x1b[0m';

function success(msg) {
  console.log(`${GREEN}✅ ${msg}${RESET}`);
}

function error(msg) {
  console.log(`${RED}❌ ${msg}${RESET}`);
}

function info(msg) {
  console.log(`${YELLOW}ℹ️  ${msg}${RESET}`);
}

let allPassed = true;

// Test 1: Check CLI_TOOLS_REGISTRY configuration
info('Testing Qwen Code configuration in CLI_TOOLS_REGISTRY...');
try {
  const typesContent = fs.readFileSync(path.join(__dirname, 'src/shared/types/cli-tools.ts'), 'utf8');
  const checks = [
    { pattern: "'qwen-code':", name: 'Tool ID' },
    { pattern: "name: 'Qwen Code'", name: 'Tool name' },
    { pattern: "command: 'qwen-code'", name: 'Command' },
    { pattern: "installCommand: 'npm install -g @qwen-code/qwen-code", name: 'Install command' },
    { pattern: "updateCommand: 'npm update -g @qwen-code/qwen-code'", name: 'Update command' },
    { pattern: "versionCommand: 'qwen-code --version'", name: 'Version command' },
    { pattern: /versionRegex:.*qwen-code/, name: 'Version regex' },
    { pattern: "requiresNode: true", name: 'Node.js requirement' }
  ];
  
  checks.forEach(check => {
    if (typeof check.pattern === 'string' ? 
        typesContent.includes(check.pattern) : 
        check.pattern.test(typesContent)) {
      success(`${check.name} configured`);
    } else {
      error(`${check.name} missing or incorrect`);
      allPassed = false;
    }
  });
} catch (err) {
  error(`Failed to check configuration: ${err.message}`);
  allPassed = false;
}

// Test 2: Check module-level import
info('\nTesting CLI_TOOLS_REGISTRY import...');
try {
  const indexContent = fs.readFileSync(path.join(__dirname, 'src/index.ts'), 'utf8');
  if (indexContent.includes("import { CLI_TOOLS_REGISTRY }")) {
    success('CLI_TOOLS_REGISTRY is imported at module level');
  } else {
    error('CLI_TOOLS_REGISTRY not imported properly');
    allPassed = false;
  }
} catch (err) {
  error(`Failed to read index.ts: ${err.message}`);
  allPassed = false;
}

// Test 3: Check version detection in handlers
info('\nTesting Qwen-specific version detection...');
try {
  const indexContent = fs.readFileSync(path.join(__dirname, 'src/index.ts'), 'utf8');
  
  // Check install handler
  if (indexContent.includes("toolId === 'qwen-code'") && 
      indexContent.includes("qwen-code\\/|v?)(\\d+\\.\\d+\\.\\d+)")) {
    success('Install handler has Qwen version detection');
  } else {
    error('Install handler missing Qwen version detection');
    allPassed = false;
  }
  
  // Check update handler
  if (indexContent.includes("'qwen-code': '@qwen-code/qwen-code'")) {
    success('Update handler has Qwen package mapping');
  } else {
    error('Update handler missing Qwen package mapping');
    allPassed = false;
  }
} catch (err) {
  error(`Failed to check handlers: ${err.message}`);
  allPassed = false;
}

// Test 4: Check terminal display name
info('\nTesting terminal display name...');
try {
  const terminalContent = fs.readFileSync(path.join(__dirname, 'src/terminal-ipc-handlers.ts'), 'utf8');
  
  if (terminalContent.includes("'qwen-code': 'Qwen'")) {
    success('Terminal display name configured');
  } else {
    error('Terminal display name missing');
    allPassed = false;
  }
} catch (err) {
  error(`Failed to check terminal handlers: ${err.message}`);
  allPassed = false;
}

// Test 5: Check renderer implementation
info('\nTesting renderer implementation...');
try {
  const rendererContent = fs.readFileSync(path.join(__dirname, 'src/renderer.ts'), 'utf8');
  
  // Check for dynamic card creation
  if (rendererContent.includes("electronAPI.detectCliTool('qwen-code')") &&
      rendererContent.includes("createCliToolCard")) {
    success('Qwen uses dynamic card with detection');
  } else {
    error('Qwen not using dynamic card properly');
    allPassed = false;
  }
  
  // Check for badge
  if (rendererContent.includes("badgeText: 'FREE 2K/DAY'")) {
    success('FREE 2K/DAY badge configured');
  } else {
    error('Badge not configured');
    allPassed = false;
  }
  
  // Check description
  if (rendererContent.includes("2000 req/day free")) {
    success('Description mentions free tier');
  } else {
    error('Description missing free tier info');
    allPassed = false;
  }
} catch (err) {
  error(`Failed to check renderer: ${err.message}`);
  allPassed = false;
}

// Test 6: Check UI refresh mechanism
info('\nTesting UI refresh mechanism...');
try {
  const rendererContent = fs.readFileSync(path.join(__dirname, 'src/renderer.ts'), 'utf8');
  
  if (rendererContent.includes("renderCliToolsPanel(forceRefresh: boolean = false)")) {
    success('ForceRefresh parameter exists');
  } else {
    error('ForceRefresh parameter missing');
    allPassed = false;
  }
  
  if (rendererContent.includes("await renderCliToolsPanel(true)")) {
    success('Install/Update handlers use forceRefresh');
  } else {
    error('Handlers not using forceRefresh');
    allPassed = false;
  }
} catch (err) {
  error(`Failed to check UI refresh: ${err.message}`);
  allPassed = false;
}

// Test 7: Check if Qwen Code is installed
info('\nChecking Qwen Code installation status...');
try {
  execSync('qwen-code --version', { stdio: 'ignore' });
  success('Qwen Code is installed');
  info('Run "npm uninstall -g @qwen-code/qwen-code" to test fresh install');
} catch (err) {
  info('Qwen Code not installed (ready for fresh install test)');
}

// Summary
console.log('\n📊 Test Summary:');
console.log('================');
if (allPassed) {
  console.log(`${GREEN}All tests passed! Qwen Code integration is complete.${RESET}`);
} else {
  console.log(`${RED}Some tests failed. Please fix the issues above.${RESET}`);
}

console.log('\n🎯 Next Steps:');
console.log('1. Run the app: npm start');
console.log('2. Click Install button for Qwen Code');
console.log('3. Verify panel refreshes after install');
console.log('4. Click Configure to set up Memory Service');
console.log('5. Click Launch to open terminal');
console.log('6. Verify "Qwen" appears in terminal tab');
console.log('7. Test Update button functionality');
console.log('\n📝 Documentation:');
console.log('- Qwen Code offers 2000 requests/day FREE');
console.log('- OAuth authentication available (no API key needed)');
console.log('- Supports Memory Service integration');
console.log('- Requires Node.js 20+');