const fs = require('fs');
const path = require('path');
const https = require('https');
const { pipeline } = require('stream/promises');
const { Extract } = require('unzipper');
const tar = require('tar');
const os = require('os');

const DOWNLOAD_BASE = 'https://downloads.hivetechs.io/releases/latest';

async function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        return downloadFile(response.headers.location, dest).then(resolve).catch(reject);
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }
      
      response.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', reject);
  });
}

async function install() {
  const platform = os.platform();
  const arch = os.arch() === 'x64' ? 'x64' : 'arm64';
  const installDir = path.join(__dirname, '..', 'app');
  
  // Create app directory
  if (!fs.existsSync(installDir)) {
    fs.mkdirSync(installDir, { recursive: true });
  }
  
  let downloadUrl;
  let filename;
  
  switch (platform) {
    case 'darwin':
      filename = `Hive-Consensus-mac-${arch}.dmg`;
      downloadUrl = `${DOWNLOAD_BASE}/mac/${filename}`;
      break;
    case 'win32':
      filename = `Hive-Consensus-Setup.exe`;
      downloadUrl = `${DOWNLOAD_BASE}/win/${filename}`;
      break;
    case 'linux':
      filename = `Hive-Consensus-linux-${arch}.AppImage`;
      downloadUrl = `${DOWNLOAD_BASE}/linux/${filename}`;
      break;
    default:
      throw new Error(`Unsupported platform: ${platform}`);
  }
  
  const tempFile = path.join(os.tmpdir(), filename);
  
  console.log(`Downloading Hive Consensus for ${platform} ${arch}...`);
  console.log(`From: ${downloadUrl}`);
  
  try {
    await downloadFile(downloadUrl, tempFile);
    console.log('Download complete. Installing...');
    
    // Platform-specific installation
    if (platform === 'darwin') {
      // For macOS, we need to mount the DMG and copy the app
      const { execSync } = require('child_process');
      
      // Mount DMG
      console.log('Mounting DMG...');
      execSync(`hdiutil attach "${tempFile}" -nobrowse -quiet`);
      
      // Copy app
      const mountPoint = '/Volumes/Hive Consensus';
      const appPath = path.join(mountPoint, 'Hive Consensus.app');
      const destPath = path.join(installDir, 'Hive Consensus.app');
      
      console.log('Copying application...');
      execSync(`cp -R "${appPath}" "${destPath}"`);
      
      // Unmount DMG
      execSync(`hdiutil detach "${mountPoint}" -quiet`);
      
      // Remove quarantine attribute
      execSync(`xattr -d com.apple.quarantine "${destPath}" 2>/dev/null || true`);
      
    } else if (platform === 'linux') {
      // For Linux, copy AppImage and make executable
      const destPath = path.join(installDir, 'Hive Consensus.AppImage');
      fs.copyFileSync(tempFile, destPath);
      fs.chmodSync(destPath, 0o755);
      
    } else if (platform === 'win32') {
      // For Windows, run the installer silently
      const { execSync } = require('child_process');
      execSync(`"${tempFile}" /S /D="${installDir}"`);
    }
    
    // Clean up
    fs.unlinkSync(tempFile);
    
    console.log('✅ Hive Consensus installed successfully!');
    console.log('Run "hive-consensus" to launch the application.');
    
  } catch (error) {
    console.error('Installation failed:', error);
    process.exit(1);
  }
}

// Run installation
install().catch(console.error);