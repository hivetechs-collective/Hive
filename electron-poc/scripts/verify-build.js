#!/usr/bin/env node

/**
 * Build Verification Script
 * Ensures critical configurations survive webpack bundling
 */

const fs = require('fs');
const path = require('path');

console.log('🔍 Verifying build integrity...\n');

let hasErrors = false;

// Check if webpack output exists
const mainBundlePath = path.join(__dirname, '..', '.webpack', 'arm64', 'main', 'index.js');
if (!fs.existsSync(mainBundlePath)) {
  console.error('❌ Main bundle not found at:', mainBundlePath);
  process.exit(1);
}

// Read the bundled output
const mainBundle = fs.readFileSync(mainBundlePath, 'utf8');

// Critical checks
const checks = [
  {
    name: "stdio: 'inherit' configuration",
    pattern: /stdio.*["']inherit["']/,
    required: true,
    errorMsg: "stdio: 'inherit' not found in bundled output! Consensus routing will fail."
  },
  {
    name: "Broken stdio configuration",
    pattern: /\["ignore".*"pipe".*"pipe"\]/,
    required: false, // Should NOT be present
    errorMsg: "Broken stdio config ['ignore', 'pipe', 'pipe'] still present!"
  },
  {
    name: "ProcessManager class",
    pattern: /ProcessManager/,
    required: true,
    errorMsg: "ProcessManager not found in bundle!"
  },
  {
    name: "Binary process spawning",
    pattern: /spawn.*Binary/i,
    required: true,
    errorMsg: "Binary process spawning code not found!"
  }
];

console.log('Running critical checks:\n');

checks.forEach(check => {
  const found = check.pattern.test(mainBundle);
  
  if (check.required && !found) {
    console.error(`❌ ${check.name}: FAILED`);
    console.error(`   ${check.errorMsg}`);
    hasErrors = true;
  } else if (!check.required && found) {
    console.error(`❌ ${check.name}: FAILED`);
    console.error(`   ${check.errorMsg}`);
    hasErrors = true;
  } else {
    console.log(`✅ ${check.name}: PASSED`);
  }
});

// Check for minification issues
console.log('\n🔍 Checking for minification issues...\n');

// Look for the exact string patterns we need
const criticalStrings = ['inherit', 'stdio', 'ProcessManager', 'spawn'];
criticalStrings.forEach(str => {
  if (!mainBundle.includes(str)) {
    console.warn(`⚠️  Warning: String '${str}' not found as literal (may be minified)`);
  } else {
    console.log(`✅ String '${str}' preserved`);
  }
});

// Summary
console.log('\n' + '='.repeat(50));
if (hasErrors) {
  console.error('\n❌ BUILD VERIFICATION FAILED!');
  console.error('The webpack build has critical issues that will break consensus routing.');
  console.error('\nTo fix:');
  console.error('1. Clear all caches: rm -rf .webpack/ out/ node_modules/.cache/ dist/');
  console.error('2. Rebuild: npm run package');
  console.error('3. If issue persists, check webpack configuration');
  process.exit(1);
} else {
  console.log('\n✅ BUILD VERIFICATION PASSED!');
  console.log('All critical configurations are preserved in the bundled output.');
}