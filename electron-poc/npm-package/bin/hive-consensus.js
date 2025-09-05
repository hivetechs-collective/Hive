#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const os = require('os');

// Determine platform-specific paths
const platform = os.platform();
const arch = os.arch();
const appName = 'Hive Consensus';

function getAppPath() {
  const installDir = path.join(__dirname, '..', 'app');
  
  switch (platform) {
    case 'darwin':
      return path.join(installDir, `${appName}.app`, 'Contents', 'MacOS', appName);
    case 'win32':
      return path.join(installDir, `${appName}.exe`);
    case 'linux':
      return path.join(installDir, `${appName}.AppImage`);
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }
}

function launchApp() {
  const appPath = getAppPath();
  
  if (!fs.existsSync(appPath)) {
    console.error(`Hive Consensus is not installed properly.`);
    console.error(`Expected app at: ${appPath}`);
    console.error(`Please run: npm install -g @hivetechs/consensus`);
    process.exit(1);
  }
  
  // Launch the Electron app
  const child = spawn(appPath, process.argv.slice(2), {
    detached: true,
    stdio: 'ignore'
  });
  
  child.unref();
  
  console.log('Launching Hive Consensus...');
}

// Check if this is the first run after install
if (!fs.existsSync(getAppPath())) {
  console.log('First run detected. Please wait for installation to complete...');
  setTimeout(() => {
    if (fs.existsSync(getAppPath())) {
      launchApp();
    } else {
      console.error('Installation failed. Please reinstall.');
      process.exit(1);
    }
  }, 5000);
} else {
  launchApp();
}