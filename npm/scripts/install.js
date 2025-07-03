#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');
const tar = require('tar');

const RELEASE_URL = 'https://api.github.com/repos/hivetechs-collective/hive/releases/latest';
const BIN_DIR = path.join(__dirname, '..', 'bin');

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;
  
  const platformMap = {
    'darwin': 'macos',
    'linux': 'linux', 
    'win32': 'windows'
  };
  
  const archMap = {
    'x64': 'x86_64',
    'arm64': 'aarch64'
  };
  
  return {
    platform: platformMap[platform],
    arch: archMap[arch],
    ext: platform === 'win32' ? '.exe' : ''
  };
}

async function downloadBinary() {
  console.log('🐝 Installing HiveTechs Consensus...');
  
  const { platform, arch, ext } = getPlatformInfo();
  
  if (!platform || !arch) {
    throw new Error(`Unsupported platform: ${process.platform}-${process.arch}`);
  }
  
  try {
    // Get latest release info
    const releaseData = await fetchJson(RELEASE_URL);
    const version = releaseData.tag_name;
    
    // Find the correct asset
    const assetName = `hive-${version}-${platform}-${arch}.tar.gz`;
    const asset = releaseData.assets.find(a => a.name === assetName);
    
    if (!asset) {
      throw new Error(`Binary not found for ${platform}-${arch}`);
    }
    
    console.log(`📦 Downloading ${assetName}...`);
    
    // Create bin directory
    if (!fs.existsSync(BIN_DIR)) {
      fs.mkdirSync(BIN_DIR, { recursive: true });
    }
    
    // Download and extract
    const tarPath = path.join(BIN_DIR, assetName);
    await downloadFile(asset.browser_download_url, tarPath);
    
    // Extract binary
    await tar.extract({
      file: tarPath,
      cwd: BIN_DIR
    });
    
    // Make executable
    const binaryPath = path.join(BIN_DIR, `hive${ext}`);
    if (fs.existsSync(binaryPath)) {
      fs.chmodSync(binaryPath, '755');
      console.log('✅ HiveTechs Consensus installed successfully!');
      console.log(`🚀 Run 'hive --help' to get started`);
    } else {
      throw new Error('Binary extraction failed');
    }
    
    // Cleanup
    fs.unlinkSync(tarPath);
    
  } catch (error) {
    console.error('❌ Installation failed:', error.message);
    process.exit(1);
  }
}

function fetchJson(url) {
  return new Promise((resolve, reject) => {
    https.get(url, {
      headers: { 'User-Agent': 'hivetechs-hive-installer' }
    }, (res) => {
      let data = '';
      res.on('data', chunk => data += chunk);
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (res) => {
      res.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

// Run installation
downloadBinary();