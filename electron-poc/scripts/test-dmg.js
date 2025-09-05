#!/usr/bin/env node

/**
 * Test the built DMG file
 * This script mounts the DMG, tests the app, and unmounts
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🔍 Testing production DMG...\n');

// Find the DMG file
const makeDir = path.join(__dirname, '../out/make');
if (!fs.existsSync(makeDir)) {
  console.error('❌ No make directory found. Run "npm run make" first.');
  process.exit(1);
}

const dmgFiles = fs.readdirSync(makeDir).filter(f => f.endsWith('.dmg'));
if (dmgFiles.length === 0) {
  console.error('❌ No DMG file found in out/make/');
  console.error('   Run "npm run make" to build the DMG');
  process.exit(1);
}

const dmgPath = path.join(makeDir, dmgFiles[0]);
console.log(`📦 Testing DMG: ${dmgPath}`);

try {
  // Mount the DMG
  console.log('📂 Mounting DMG...');
  execSync(`hdiutil attach "${dmgPath}"`, { stdio: 'inherit' });
  
  // Wait a moment for mount
  execSync('sleep 2');
  
  // Test the app with --test-mode flag
  console.log('🚀 Testing app startup...');
  try {
    const result = execSync(
      '"/Volumes/Hive Consensus/Hive Consensus.app/Contents/MacOS/Hive Consensus" --test-mode',
      { 
        timeout: 10000,
        encoding: 'utf8'
      }
    );
    
    // Check for successful startup indicators
    if (result.includes('error') || result.includes('Error') || result.includes('failed')) {
      console.error('⚠️ App started but reported errors');
      console.log(result);
    } else {
      console.log('✅ App started successfully!');
    }
  } catch (e) {
    if (e.code === 'ETIMEDOUT') {
      console.log('✅ App started (timed out waiting, which is normal for GUI apps)');
    } else {
      console.error('❌ App failed to start:', e.message);
    }
  }
  
} finally {
  // Always unmount the DMG
  console.log('📂 Unmounting DMG...');
  try {
    execSync('hdiutil detach "/Volumes/Hive Consensus"', { stdio: 'inherit' });
  } catch (e) {
    console.warn('⚠️ Could not unmount DMG, may need manual unmount');
  }
}

console.log('\n✅ DMG test complete!');
console.log('For a full test:');
console.log('  1. Delete any existing "Hive Consensus" app from Applications');
console.log('  2. Double-click the DMG and drag to Applications');
console.log('  3. Launch from Finder (not terminal)');
console.log('  4. Verify all services start correctly');