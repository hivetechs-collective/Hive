#!/usr/bin/env node

// Script to download LazyGit binary for current platform
const https = require('https');
const fs = require('fs');
const path = require('path');
const { exec } = require('child_process');
const { promisify } = require('util');
const execAsync = promisify(exec);

const LAZYGIT_VERSION = 'v0.40.2';
const platform = process.platform;
const arch = process.arch;

async function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    console.log(`Downloading from ${url}...`);
    
    const file = fs.createWriteStream(dest);
    
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        downloadFile(response.headers.location, dest).then(resolve).catch(reject);
        return;
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }
      
      const total = parseInt(response.headers['content-length'], 10);
      let downloaded = 0;
      
      response.on('data', (chunk) => {
        downloaded += chunk.length;
        const percent = ((downloaded / total) * 100).toFixed(1);
        process.stdout.write(`\rDownloading... ${percent}%`);
      });
      
      response.pipe(file);
      
      file.on('finish', () => {
        file.close();
        console.log('\nDownload complete!');
        resolve();
      });
      
      file.on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    }).on('error', reject);
  });
}

async function extractTarGz(file, dest) {
  console.log(`Extracting ${file}...`);
  await execAsync(`tar -xzf ${file} -C ${dest}`);
  console.log('Extraction complete!');
}

async function main() {
  // Determine download URL based on platform
  let downloadUrl;
  let needsExtraction = false;
  
  if (platform === 'darwin') {
    if (arch === 'arm64') {
      downloadUrl = `https://github.com/jesseduffield/lazygit/releases/download/${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION.substring(1)}_Darwin_arm64.tar.gz`;
    } else {
      downloadUrl = `https://github.com/jesseduffield/lazygit/releases/download/${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION.substring(1)}_Darwin_x86_64.tar.gz`;
    }
    needsExtraction = true;
  } else if (platform === 'linux') {
    downloadUrl = `https://github.com/jesseduffield/lazygit/releases/download/${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION.substring(1)}_Linux_x86_64.tar.gz`;
    needsExtraction = true;
  } else if (platform === 'win32') {
    downloadUrl = `https://github.com/jesseduffield/lazygit/releases/download/${LAZYGIT_VERSION}/lazygit_${LAZYGIT_VERSION.substring(1)}_Windows_x86_64.zip`;
    needsExtraction = true;
  } else {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
  }
  
  // Create resources directory
  const resourcesDir = path.join(__dirname, '..', 'resources', 'lazygit', `${platform}-${arch === 'arm64' ? 'arm64' : 'x64'}`);
  if (!fs.existsSync(resourcesDir)) {
    fs.mkdirSync(resourcesDir, { recursive: true });
  }
  
  // Download file
  const tempFile = path.join(resourcesDir, needsExtraction ? 'lazygit-temp.tar.gz' : 'lazygit');
  
  try {
    await downloadFile(downloadUrl, tempFile);
    
    if (needsExtraction) {
      // Extract the archive
      if (platform === 'win32') {
        // For Windows, we'd need to use a zip extraction tool
        console.log('Please extract the zip file manually for Windows');
      } else {
        await extractTarGz(tempFile, resourcesDir);
        
        // Delete temp file
        fs.unlinkSync(tempFile);
        
        // Make binary executable
        const binaryPath = path.join(resourcesDir, 'lazygit');
        fs.chmodSync(binaryPath, 0o755);
      }
    } else {
      // Make binary executable
      fs.chmodSync(tempFile, 0o755);
    }
    
    console.log(`✅ LazyGit ${LAZYGIT_VERSION} downloaded successfully for ${platform}-${arch}`);
    console.log(`📁 Location: ${resourcesDir}`);
    
    // Test the binary
    const binaryPath = path.join(resourcesDir, platform === 'win32' ? 'lazygit.exe' : 'lazygit');
    try {
      const { stdout } = await execAsync(`${binaryPath} --version`);
      console.log(`✅ Binary test successful: ${stdout.trim()}`);
    } catch (error) {
      console.error('⚠️ Binary test failed:', error.message);
    }
  } catch (error) {
    console.error('Failed to download LazyGit:', error);
    process.exit(1);
  }
}

main();