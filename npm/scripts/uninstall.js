#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const BIN_DIR = path.join(__dirname, '..', 'bin');

function cleanup() {
  console.log('🧹 Cleaning up HiveTechs Consensus...');
  
  try {
    // Remove binary files
    const binaries = ['hive', 'hive.exe'];
    
    for (const binary of binaries) {
      const binaryPath = path.join(BIN_DIR, binary);
      if (fs.existsSync(binaryPath)) {
        fs.unlinkSync(binaryPath);
        console.log(`✅ Removed ${binary}`);
      }
    }
    
    // Clean up any temporary files
    const tempFiles = fs.readdirSync(BIN_DIR).filter(f => 
      f.endsWith('.tar.gz') || f.endsWith('.tmp')
    );
    
    for (const file of tempFiles) {
      const filePath = path.join(BIN_DIR, file);
      fs.unlinkSync(filePath);
      console.log(`✅ Removed temporary file: ${file}`);
    }
    
    console.log('👋 HiveTechs Consensus uninstalled successfully');
    
  } catch (error) {
    // Errors during uninstall are not critical
    console.log('⚠️  Some cleanup tasks failed:', error.message);
  }
}

// Run cleanup
cleanup();