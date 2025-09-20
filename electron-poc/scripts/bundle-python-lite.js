#!/usr/bin/env node
/**
 * Lightweight Python Runtime Bundler for Production
 * Creates a minimal Python environment that can install packages on-demand
 * 
 * This script creates a small Python bundle that:
 * 1. Includes Python interpreter
 * 2. Includes pip for package management
 * 3. Includes essential packages only
 * 4. Downloads ML models on first use
 */

const fs = require('fs-extra');
const path = require('path');
const { execSync, spawn } = require('child_process');
const https = require('https');
const tar = require('tar');

// Configuration
const PYTHON_VERSION = '3.11.7';
const PLATFORM = process.platform; // 'darwin', 'linux', 'win32'
const ARCH = process.arch; // 'x64', 'arm64'

// Python standalone builds (smaller versions without full stdlib)
const PYTHON_URLS = {
  'darwin-x64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-x86_64-apple-darwin-install_only.tar.gz`,
  'darwin-arm64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-aarch64-apple-darwin-install_only.tar.gz`,
  'linux-x64': `https://github.com/indygreg/python-build-standalone/releases/download/20240107/cpython-${PYTHON_VERSION}+20240107-x86_64-unknown-linux-gnu-install_only.tar.gz`
};

const RESOURCES_DIR = path.join(__dirname, '..', 'resources');
const PYTHON_RUNTIME_DIR = path.join(RESOURCES_DIR, 'python-runtime');
const PYTHON_DIR = path.join(PYTHON_RUNTIME_DIR, 'python');
const DOWNLOAD_DIR = path.join(__dirname, '..', '.python-download');

// Essential packages only (small size)
const ESSENTIAL_PACKAGES = [
  'requests',  // For API calls
  'numpy',     // Basic array operations
];

/**
 * Create a Python wrapper that handles missing packages gracefully
 */
async function createPythonWrapper() {
  const wrapperContent = `#!/usr/bin/env python3
"""
AI Helper Service Wrapper
Handles package installation on-demand for production
"""
import sys
import os
import subprocess
import json

def ensure_package(package_name, import_name=None):
    """Ensure a package is installed, installing it if necessary"""
    if import_name is None:
        import_name = package_name
    
    try:
        __import__(import_name)
        return True
    except ImportError:
        print(f"Installing {package_name}...", file=sys.stderr)
        try:
            subprocess.check_call([
                sys.executable, '-m', 'pip', 'install', 
                '--quiet', '--disable-pip-version-check',
                package_name
            ])
            return True
        except subprocess.CalledProcessError:
            return False

# For AI Helpers, we'll skip heavy ML packages and use simpler alternatives
# The consensus routing can work without them in production
def init_minimal_mode():
    """Initialize minimal mode without ML packages"""
    return {
        "type": "initialized",
        "mode": "minimal",
        "message": "AI Helpers running in minimal mode (no ML models)"
    }

# Check if we're being called as model_service
if __name__ == "__main__":
    # Ensure basic packages
    ensure_package('requests')
    
    # Check if heavy packages are available
    try:
        import torch
        import transformers
        mode = "full"
    except ImportError:
        mode = "minimal"
    
    if mode == "minimal":
        # Run in minimal mode - just echo back decisions
        print(json.dumps(init_minimal_mode()))
        
        # Simple routing decision based on query length and keywords
        while True:
            try:
                line = input()
                request = json.loads(line)
                
                # Simple heuristic for routing decision
                if request.get("type") == "route_decision":
                    query = request.get("query", "")
                    # Simple queries are short and don't have complex keywords
                    is_simple = (
                        len(query) < 100 and 
                        not any(word in query.lower() for word in [
                            'analyze', 'debug', 'implement', 'architecture', 
                            'design', 'optimize', 'refactor'
                        ])
                    )
                    
                    response = {
                        "type": "route_decision",
                        "request_id": request.get("request_id"),
                        "mode": "simple" if is_simple else "complex",
                        "confidence": 0.8
                    }
                    print(json.dumps(response))
                    sys.stdout.flush()
            except EOFError:
                break
            except Exception as e:
                error_response = {
                    "type": "error",
                    "error": str(e)
                }
                print(json.dumps(error_response))
                sys.stdout.flush()
    else:
        # Full mode with ML packages - import the real service
        from model_service import main
        main()
`;

  const wrapperPath = path.join(PYTHON_RUNTIME_DIR, 'models', 'model_service_wrapper.py');
  await fs.ensureDir(path.dirname(wrapperPath));
  await fs.writeFile(wrapperPath, wrapperContent);
  await fs.chmod(wrapperPath, 0o755);
  
  console.log('✅ Created Python wrapper for graceful degradation');
}

/**
 * Download file from URL with progress
 */
function downloadFile(url, destPath) {
  return new Promise((resolve, reject) => {
    console.log(`📥 Downloading Python runtime...`);
    const file = fs.createWriteStream(destPath);
    
    https.get(url, { 
      headers: { 'User-Agent': 'Hive-Consensus' },
      timeout: 60000 
    }, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        file.close();
        downloadFile(response.headers.location, destPath).then(resolve).catch(reject);
        return;
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Download failed: ${response.statusCode}`));
        return;
      }
      
      const totalSize = parseInt(response.headers['content-length'], 10);
      let downloaded = 0;
      
      response.on('data', (chunk) => {
        downloaded += chunk.length;
        if (totalSize) {
          const percent = ((downloaded / totalSize) * 100).toFixed(1);
          process.stdout.write(`\r  Progress: ${percent}%`);
        }
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
 * Extract Python archive
 */
async function extractPython(archivePath, destDir) {
  console.log('📦 Extracting Python...');
  await fs.ensureDir(destDir);
  
  await tar.x({
    file: archivePath,
    cwd: destDir,
    strip: 1
  });
  
  console.log('✅ Extraction complete');
}

/**
 * Install minimal packages and ML packages for AI Helpers
 */
function installMinimalPackages(pythonPath) {
  console.log('📚 Installing essential packages...');
  
  const pipCommand = PLATFORM === 'win32' 
    ? `"${pythonPath}" -m pip` 
    : `${pythonPath} -m pip`;
  
  // Ensure pip is up to date
  try {
    execSync(`${pipCommand} install --upgrade pip --quiet`, { stdio: 'inherit' });
  } catch (e) {
    console.warn('⚠️  pip upgrade skipped');
  }
  
  // Install only essential packages
  for (const pkg of ESSENTIAL_PACKAGES) {
    console.log(`  Installing ${pkg}...`);
    try {
      execSync(`${pipCommand} install ${pkg} --quiet`, { stdio: 'inherit' });
    } catch (e) {
      console.warn(`  ⚠️ ${pkg} installation failed, continuing...`);
    }
  }
  
  console.log('✅ Essential packages installed');
  
  // Install ML packages for consensus routing (required, no fallbacks)
  console.log('🤖 Installing ML packages for AI Helpers...');
  
  // Check if ML packages are already installed
  try {
    execSync(`${pythonPath} -c "import torch, transformers; print('ML packages found')"`, { stdio: 'pipe' });
    console.log('✅ ML packages already installed');
  } catch (e) {
    console.log('  Installing PyTorch (CPU-optimized)...');
    try {
      // Install CPU-only torch for smaller size
      execSync(`${pipCommand} install torch --index-url https://download.pytorch.org/whl/cpu --quiet`, { stdio: 'inherit' });
      console.log('  ✅ PyTorch installed');
    } catch (e) {
      console.error('  ❌ PyTorch installation failed - AI Helpers will not work properly!');
      process.exit(1); // FAIL the build - no fallbacks!
    }
    
    console.log('  Installing Transformers and Sentence-Transformers...');
    try {
      execSync(`${pipCommand} install transformers sentence-transformers --quiet`, { stdio: 'inherit' });
      console.log('  ✅ Transformers installed');
    } catch (e) {
      console.error('  ❌ Transformers installation failed - AI Helpers will not work properly!');
      process.exit(1); // FAIL the build - no fallbacks!
    }
    
    console.log('✅ ML packages installed successfully');
  }
}

/**
 * Clean up unnecessary files
 */
async function cleanupPython() {
  console.log('🧹 Optimizing Python bundle...');
  
  const pythonLibDir = PLATFORM === 'win32'
    ? path.join(PYTHON_DIR, 'Lib')
    : path.join(PYTHON_DIR, 'lib', `python${PYTHON_VERSION.slice(0,3)}`);
  
  // Remove test directories and other unnecessary files
  const toRemove = [
    'test', 'tests', 'idle_test', '__pycache__',
    'distutils/tests', 'unittest/test', 'lib2to3/tests'
  ];
  
  for (const item of toRemove) {
    const targetPath = path.join(pythonLibDir, item);
    if (await fs.pathExists(targetPath)) {
      await fs.remove(targetPath);
    }
  }
  
  console.log('✅ Optimization complete');
}

/**
 * Main bundling function
 */
async function main() {
  console.log('🐍 Lightweight Python Bundler for Hive Consensus');
  console.log(`📍 Platform: ${PLATFORM}-${ARCH}`);
  console.log('');
  
  const platformKey = `${PLATFORM}-${ARCH}`;
  const pythonUrl = PYTHON_URLS[platformKey];
  
  if (!pythonUrl) {
    console.error(`❌ Unsupported platform: ${platformKey}`);
    console.log('⚠️  Skipping Python bundling - AI Helpers will not be available');
    return;
  }
  
  try {
    // Clean previous builds
    console.log('🧹 Cleaning previous builds...');
    await fs.remove(PYTHON_RUNTIME_DIR);
    await fs.ensureDir(PYTHON_RUNTIME_DIR);
    await fs.ensureDir(DOWNLOAD_DIR);
    
    // Download Python if needed
    const archivePath = path.join(DOWNLOAD_DIR, 'python.tar.gz');
    if (!await fs.pathExists(archivePath)) {
      await downloadFile(pythonUrl, archivePath);
    } else {
      console.log('📦 Using cached Python download');
    }
    
    // Extract Python
    await extractPython(archivePath, PYTHON_DIR);
    
    // Find Python executable
    const pythonPath = PLATFORM === 'win32'
      ? path.join(PYTHON_DIR, 'python.exe')
      : path.join(PYTHON_DIR, 'bin', 'python3');
    
    if (!await fs.pathExists(pythonPath)) {
      throw new Error(`Python not found at ${pythonPath}`);
    }
    
    console.log(`✅ Python ready at: ${pythonPath}`);
    
    // PROPER ORDER: Copy files FIRST, then install packages
    
    // STEP 1: Ensure model_service.py is available
    console.log('📄 Setting up AI Helper scripts...');
    const destModelService = path.join(PYTHON_RUNTIME_DIR, 'models', 'model_service.py');
    await fs.ensureDir(path.dirname(destModelService));
    
    // Check if already in resources (preferred location)
    const resourceModelService = path.join(RESOURCES_DIR, 'python-runtime', 'models', 'model_service.py');
    let modelServiceCopied = false;
    
    if (await fs.pathExists(resourceModelService)) {
      // Already exists in resources, just ensure it's in runtime dir
      await fs.copy(resourceModelService, destModelService);
      console.log('  ✅ Using model_service.py from resources');
      modelServiceCopied = true;
    } else {
      // Try to find it in main hive repo (absolute path for reliability)
      const mainModelService = '/Users/veronelazio/Developer/Private/hive/python/model_service.py';
      if (await fs.pathExists(mainModelService)) {
        await fs.copy(mainModelService, destModelService);
        // Also copy to resources for next time
        await fs.copy(mainModelService, resourceModelService);
        console.log('  ✅ Copied model_service.py from hive repo');
        modelServiceCopied = true;
      }
    }
    
    if (!modelServiceCopied) {
      console.error('  ❌ model_service.py not found - AI Helpers will NOT work!');
      console.error('     Please ensure model_service.py is in resources/python-runtime/models/');
      if (process.env.ALLOW_MISSING_DEPS === '1' || process.env.ALLOW_MISSING_DEPS === 'true') {
        console.log('  ⚠️  ALLOW_MISSING_DEPS set; continuing without model_service.py');
      } else {
        process.exit(1); // FAIL BUILD - no fallbacks unless explicitly allowed
      }
    }
    
    // STEP 2: Create wrapper for compatibility (but use model_service.py in production)
    await createPythonWrapper();
    
    // STEP 3: Install packages (including ML packages)
    installMinimalPackages(pythonPath);
    
    // Clean up
    await cleanupPython();
    await fs.remove(DOWNLOAD_DIR);
    
    // Create info file
    await fs.writeJson(path.join(PYTHON_RUNTIME_DIR, 'bundle.json'), {
      version: PYTHON_VERSION,
      platform: platformKey,
      mode: 'lightweight',
      created: new Date().toISOString()
    }, { spaces: 2 });
    
    // Report size
    const sizeOutput = execSync(`du -sh "${PYTHON_RUNTIME_DIR}"`, { encoding: 'utf8' });
    const size = sizeOutput.split('\t')[0];
    
    console.log('');
    console.log('✨ Lightweight Python runtime ready!');
    console.log(`📦 Bundle size: ${size}`);
    console.log(`📍 Location: ${PYTHON_RUNTIME_DIR}`);
    console.log('');
    console.log('Features:');
    console.log('  ✅ Python interpreter included');
    console.log('  ✅ Essential packages only (small size)');
    console.log('  ✅ Graceful degradation for AI Helpers');
    console.log('  ✅ On-demand package installation');
    
  } catch (error) {
    console.error('❌ Error:', error.message);
    console.log('');
    console.log('⚠️  Python bundling failed - AI Helpers will use fallback mode');
    
    // Create a minimal fallback structure
    await fs.ensureDir(path.join(PYTHON_RUNTIME_DIR, 'models'));
    await createPythonWrapper();
  }
}

// Run the bundler
main().catch(console.error);
