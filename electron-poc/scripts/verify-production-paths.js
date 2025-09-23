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
function firstExisting(...candidates) {
  return candidates.find((p) => p && fs.existsSync(p));
}

const packagedAppRoot = firstExisting(
  path.join(__dirname, '../out/Hive Consensus-darwin-arm64/Hive Consensus.app/Contents/Resources/app.asar.unpacked'),
  path.join(__dirname, '../out/Hive Consensus-darwin-arm64/Hive Consensus.app/Contents/Resources/default_app.asar.unpacked')
);

const webpackMainPath = firstExisting(
  path.join(__dirname, '../.webpack/main'),
  packagedAppRoot && path.join(packagedAppRoot, '.webpack/main')
);
if (!webpackMainPath) {
  const expected = path.join(__dirname, '../.webpack/main');
  console.error('❌ Webpack main output not found at:', expected);
  console.log('   Run "npm run build" first');
  hasErrors = true;
} else {
  console.log('✅ Webpack main output found at:', webpackMainPath);
}

// Check if memory service is built
const memoryServicePath = firstExisting(
  path.join(__dirname, '../.webpack/main/memory-service/index.js'),
  packagedAppRoot && path.join(packagedAppRoot, '.webpack/main/memory-service/index.js')
);
if (!memoryServicePath) {
  const expected = path.join(__dirname, '../.webpack/main/memory-service/index.js');
  console.error('❌ Memory service not built at:', expected);
  console.log('   The BuildMemoryServicePlugin should build this automatically');
  hasErrors = true;
} else {
  console.log('✅ Memory service built at:', memoryServicePath);
}

// Check if binaries exist
const backendBinaryPath = firstExisting(
  path.join(__dirname, '../binaries/hive-backend-server-enhanced'),
  packagedAppRoot && path.join(packagedAppRoot, '.webpack/main/binaries/hive-backend-server-enhanced')
);
if (!backendBinaryPath) {
  const expected = path.join(__dirname, '../binaries/hive-backend-server-enhanced');
  console.error('❌ Backend binary not found at:', expected);
  console.log('   Build it with: cd ../../hive && cargo build --release --bin hive-backend-server-enhanced');
  console.log('   Then copy it: cp target/release/hive-backend-server-enhanced ../electron-poc/binaries/');
  hasErrors = true;
} else {
  console.log('✅ Backend binary found at:', backendBinaryPath);
}

// Check if Python runtime exists
const pythonRuntimePath = firstExisting(
  path.join(__dirname, '../resources/python-runtime/python'),
  packagedAppRoot && path.join(packagedAppRoot, 'resources/python-runtime/python')
);
if (!pythonRuntimePath) {
  const expected = path.join(__dirname, '../resources/python-runtime/python');
  console.error('❌ Python runtime not found at:', expected);
  console.log('   Bundle it with: npm run bundle-python');
  hasErrors = true;
} else {
  console.log('✅ Python runtime found at:', pythonRuntimePath);
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
