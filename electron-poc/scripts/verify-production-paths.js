#!/usr/bin/env node

/**
 * Verify production paths are correctly set up
 * This script checks that all necessary files and paths exist
 */

const fs = require('fs');
const path = require('path');

console.log('🔍 Verifying production paths...\n');

let hasErrors = false;

// Check if webpack output exists
const webpackMainPath = path.join(__dirname, '../.webpack/main');
if (!fs.existsSync(webpackMainPath)) {
  console.error('❌ Webpack main output not found at:', webpackMainPath);
  console.log('   Run "npm run build" first');
  hasErrors = true;
} else {
  console.log('✅ Webpack main output found');
}

// Check if memory service is built
const memoryServicePath = path.join(__dirname, '../.webpack/main/memory-service/index.js');
if (!fs.existsSync(memoryServicePath)) {
  console.error('❌ Memory service not built at:', memoryServicePath);
  console.log('   The BuildMemoryServicePlugin should build this automatically');
  hasErrors = true;
} else {
  console.log('✅ Memory service built');
}

// Check if binaries exist
const backendBinaryPath = path.join(__dirname, '../binaries/hive-backend-server-enhanced');
if (!fs.existsSync(backendBinaryPath)) {
  console.error('❌ Backend binary not found at:', backendBinaryPath);
  console.log('   Build it with: cd ../../hive && cargo build --release --bin hive-backend-server-enhanced');
  console.log('   Then copy it: cp target/release/hive-backend-server-enhanced ../electron-poc/binaries/');
  hasErrors = true;
} else {
  console.log('✅ Backend binary found');
}

// Check if Python runtime exists
const pythonRuntimePath = path.join(__dirname, '../resources/python-runtime/python');
if (!fs.existsSync(pythonRuntimePath)) {
  console.error('❌ Python runtime not found at:', pythonRuntimePath);
  console.log('   Bundle it with: npm run bundle-python');
  hasErrors = true;
} else {
  console.log('✅ Python runtime found');
}

// Check package.json has correct productName
const packageJson = require('../package.json');
if (packageJson.productName !== 'Hive Consensus') {
  console.error('❌ Package.json productName should be "Hive Consensus"');
  hasErrors = true;
} else {
  console.log('✅ Package.json configured correctly');
}

// Check forge.config exists
const forgeConfigPath = path.join(__dirname, '../forge.config.ts');
if (!fs.existsSync(forgeConfigPath)) {
  console.error('❌ forge.config.ts not found');
  hasErrors = true;
} else {
  console.log('✅ Forge config found');
}

console.log('');
if (hasErrors) {
  console.error('❌ Production path verification failed!');
  console.log('Fix the issues above before building the DMG.');
  process.exit(1);
} else {
  console.log('✅ All production paths verified successfully!');
  console.log('Ready to build DMG with: npm run make');
}