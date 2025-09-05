#!/usr/bin/env node
/**
 * Bundle Python Runtime for Production
 * Creates a standalone Python environment with all required dependencies for AI Helpers
 * 
 * This script:
 * 1. Downloads a standalone Python distribution
 * 2. Installs required packages (torch, transformers, etc.)
 * 3. Packages everything for inclusion in the Electron app
 */

const fs = require('fs-extra');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');
const tar = require('tar');
const os = require('os');

// Configuration
const PYTHON_VERSION = '3.11.7';
const PLATFORM = process.platform; // 'darwin', 'linux', 'win32'
const ARCH = process.arch; // 'x64', 'arm64'

// Python standalone builds from indygreg/python-build-standalone
const PYTHON_URLS = {
  'darwin-x64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-x86_64-apple-darwin-install_only.tar.gz`,
  'darwin-arm64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-aarch64-apple-darwin-install_only.tar.gz`,
  'linux-x64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-x86_64-unknown-linux-gnu-install_only.tar.gz`,
  'win32-x64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-x86_64-pc-windows-msvc-shared-install_only.tar.gz`
};

const RESOURCES_DIR = path.join(__dirname, '..', 'resources');
const PYTHON_RUNTIME_DIR = path.join(RESOURCES_DIR, 'python-runtime');
const PYTHON_DIR = path.join(PYTHON_RUNTIME_DIR, 'python');
const DOWNLOAD_DIR = path.join(__dirname, '..', '.python-download');

// Required Python packages for AI Helpers (minimal versions for size)
const REQUIRED_PACKAGES = [
  'numpy==1.24.3',
  'torch==2.0.1',  // CPU-only version to reduce size
  'transformers==4.36.0',
  'sentence-transformers==2.2.2',
  'chromadb==0.4.22',
  'requests',  // For downloading models
  'tqdm'       // Progress bars
];

/**
 * Download file from URL
 */
function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    console.log(`📥 Downloading from ${url}...`);
    const file = fs.createWriteStream(destPath);
    
    https.get(url, { 
      headers: { 'User-Agent': 'Hive-Consensus' },
      timeout: 60000 
    }, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Handle redirect
        file.close();
        downloadFile(response.headers.location, destPath).then(resolve).catch(reject);
        return;
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }
      
      const totalSize = parseInt(response.headers['content-length'], 10);
      let downloaded = 0;
      
      response.on('data', (chunk) => {
        downloaded += chunk.length;
        const percent = ((downloaded / totalSize) * 100).toFixed(1);
        process.stdout.write(`\r📊 Progress: ${percent}% (${(downloaded / 1024 / 1024).toFixed(1)}MB / ${(totalSize / 1024 / 1024).toFixed(1)}MB)`);
      });
      
      response.pipe(file);
      
      file.on('finish', () => {
        file.close();
        console.log('\n✅ Download complete');
        resolve();
      });
    }).on('error', (err) => {
      fs.unlinkSync(destPath);
      reject(err);
    });
  });
}

/**
 * Extract tar.gz archive
 */
async function extractArchive(archivePath, destDir) {
  console.log(`📦 Extracting Python runtime...`);
  await fs.ensureDir(destDir);
  
  await tar.x({
    file: archivePath,
    cwd: destDir,
    strip: 1  // Remove top-level directory from archive
  });
  
  console.log('✅ Extraction complete');
}

/**
 * Install Python packages
 */
function installPackages(pythonPath) {
  console.log('📚 Installing required Python packages...');
  
  const pipPath = PLATFORM === 'win32' 
    ? path.join(path.dirname(pythonPath), 'Scripts', 'pip.exe')
    : pythonPath;
  
  const pipCommand = PLATFORM === 'win32' 
    ? pipPath 
    : `${pythonPath} -m pip`;
  
  // Upgrade pip first
  console.log('⬆️  Upgrading pip...');
  try {
    execSync(`${pipCommand} install --upgrade pip`, {
      stdio: 'inherit',
      env: { ...process.env, PYTHONPATH: '' }
    });
  } catch (e) {
    console.warn('⚠️  pip upgrade failed, continuing...');
  }
  
  // Install packages with CPU-only torch to reduce size
  const torchIndex = PLATFORM === 'darwin' && ARCH === 'arm64'
    ? '' // Default index for M1/M2 Macs
    : '--index-url https://download.pytorch.org/whl/cpu'; // CPU-only for others
  
  for (const pkg of REQUIRED_PACKAGES) {
    console.log(`📦 Installing ${pkg}...`);
    try {
      const installCmd = pkg.startsWith('torch') && torchIndex
        ? `${pipCommand} install ${pkg} ${torchIndex}`
        : `${pipCommand} install ${pkg}`;
      
      execSync(installCmd, {
        stdio: 'inherit',
        env: { ...process.env, PYTHONPATH: '' }
      });
    } catch (error) {
      console.error(`❌ Failed to install ${pkg}: ${error.message}`);
      // Continue with other packages
    }
  }
  
  console.log('✅ Package installation complete');
}

/**
 * Copy model service script
 */
async function copyModelService() {
  console.log('📋 Copying model service script...');
  
  const modelsDir = path.join(PYTHON_RUNTIME_DIR, 'models');
  await fs.ensureDir(modelsDir);
  
  const sourceScript = path.join(RESOURCES_DIR, 'python-runtime', 'models', 'model_service.py');
  const destScript = path.join(modelsDir, 'model_service.py');
  
  if (await fs.pathExists(sourceScript)) {
    await fs.copy(sourceScript, destScript);
    console.log('✅ Model service script copied');
  } else {
    console.warn('⚠️  Model service script not found at', sourceScript);
  }
}

/**
 * Optimize the bundle size
 */
async function optimizeBundle() {
  console.log('🔧 Optimizing bundle size...');
  
  // Remove unnecessary files to reduce size
  const itemsToRemove = [
    'test',
    'tests',
    '__pycache__',
    '*.pyc',
    '*.pyo',
    'idle_test',
    'tkinter',  // GUI library not needed
    'ensurepip',  // pip installer not needed in production
    'distutils',
    'setuptools/_distutils',
    'pip/_internal/commands',  // Keep pip minimal
  ];
  
  const libDir = path.join(PYTHON_DIR, 'lib');
  const sitePackagesDir = path.join(libDir, `python${PYTHON_VERSION.slice(0, 4)}`, 'site-packages');
  
  for (const item of itemsToRemove) {
    const patterns = [
      path.join(libDir, '**', item),
      path.join(sitePackagesDir, '**', item)
    ];
    
    for (const pattern of patterns) {
      try {
        const matches = await fs.glob(pattern);
        for (const match of matches) {
          await fs.remove(match);
        }
      } catch (e) {
        // Ignore errors
      }
    }
  }
  
  console.log('✅ Bundle optimization complete');
}

/**
 * Main bundling function
 */
async function main() {
  console.log('🐍 Python Runtime Bundler for Hive Consensus');
  console.log(`📍 Platform: ${PLATFORM}-${ARCH}`);
  console.log(`🎯 Python Version: ${PYTHON_VERSION}`);
  console.log('');
  
  const platformKey = `${PLATFORM}-${ARCH}`;
  const pythonUrl = PYTHON_URLS[platformKey];
  
  if (!pythonUrl) {
    console.error(`❌ Unsupported platform: ${platformKey}`);
    process.exit(1);
  }
  
  try {
    // Clean up previous builds
    console.log('🧹 Cleaning previous builds...');
    await fs.remove(PYTHON_RUNTIME_DIR);
    await fs.remove(DOWNLOAD_DIR);
    await fs.ensureDir(DOWNLOAD_DIR);
    await fs.ensureDir(PYTHON_RUNTIME_DIR);
    
    // Download Python
    const archivePath = path.join(DOWNLOAD_DIR, 'python.tar.gz');
    if (!await fs.pathExists(archivePath)) {
      await downloadFile(pythonUrl, archivePath);
    } else {
      console.log('📦 Using cached Python download');
    }
    
    // Extract Python
    await extractArchive(archivePath, PYTHON_DIR);
    
    // Find Python executable
    let pythonPath;
    if (PLATFORM === 'win32') {
      pythonPath = path.join(PYTHON_DIR, 'python.exe');
    } else {
      pythonPath = path.join(PYTHON_DIR, 'bin', 'python3');
    }
    
    if (!await fs.pathExists(pythonPath)) {
      throw new Error(`Python executable not found at ${pythonPath}`);
    }
    
    console.log(`✅ Python found at: ${pythonPath}`);
    
    // Install packages
    installPackages(pythonPath);
    
    // Copy model service
    await copyModelService();
    
    // Optimize bundle
    await optimizeBundle();
    
    // Create a marker file with version info
    const markerPath = path.join(PYTHON_RUNTIME_DIR, 'bundle.json');
    await fs.writeJson(markerPath, {
      version: PYTHON_VERSION,
      platform: platformKey,
      created: new Date().toISOString(),
      packages: REQUIRED_PACKAGES
    }, { spaces: 2 });
    
    // Clean up download directory
    await fs.remove(DOWNLOAD_DIR);
    
    // Report bundle size
    const getDirSize = (dir) => {
      const { size } = execSync(`du -sh "${dir}"`, { encoding: 'utf8' });
      return size.split('\t')[0];
    };
    
    const bundleSize = getDirSize(PYTHON_RUNTIME_DIR);
    
    console.log('');
    console.log('🎉 Python runtime bundled successfully!');
    console.log(`📦 Bundle size: ${bundleSize}`);
    console.log(`📍 Location: ${PYTHON_RUNTIME_DIR}`);
    console.log('');
    console.log('Next steps:');
    console.log('1. Run "npm run make" to build the production app');
    console.log('2. The Python runtime will be included automatically');
    
  } catch (error) {
    console.error('❌ Error bundling Python:', error);
    process.exit(1);
  }
}

// Run the bundler
main().catch(console.error);