#!/usr/bin/env node

/**
 * Post-build fixes for production DMG
 * Ensures all critical files are properly included in the app bundle
 * This handles what Electron Forge's asar.unpack might miss
 */

const fs = require('fs-extra');
const path = require('path');

// Colors for console output
const GREEN = '\x1b[32m';
const RED = '\x1b[31m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const RESET = '\x1b[0m';

function logStep(message) {
  console.log(`${CYAN}  → ${message}${RESET}`);
}

function logSuccess(message) {
  console.log(`${GREEN}  ✓ ${message}${RESET}`);
}

function logError(message) {
  console.error(`${RED}  ✗ ${message}${RESET}`);
}

function logWarning(message) {
  console.log(`${YELLOW}  ⚠ ${message}${RESET}`);
}

async function fixDylibsInAppBundle(appPath) {
  console.log(`\n${CYAN}🔧 Fixing .dylibs in app bundle...${RESET}`);
  
  const pythonBasePath = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime');
  const sourcePythonPath = path.join(__dirname, '../resources/python-runtime');
  
  // Critical packages that need .dylibs
  const packagesNeedingDylibs = [
    'PIL',
    'sklearn', 
    'scipy',
    'numpy',
    'torch',
    'transformers'
  ];
  
  let fixedCount = 0;
  let missingCount = 0;
  
  for (const pkg of packagesNeedingDylibs) {
    // Try multiple possible locations
    const possiblePaths = [
      `python/lib/python3.11/site-packages/${pkg}/.dylibs`,
      `python/lib/python3.11/site-packages/${pkg.toLowerCase()}/.dylibs`,
      `python/lib/python3.11/site-packages/${pkg}/.dylibs`
    ];
    
    for (const dylibPath of possiblePaths) {
      const sourceDylibs = path.join(sourcePythonPath, dylibPath);
      const targetDylibs = path.join(pythonBasePath, dylibPath);
      
      if (fs.existsSync(sourceDylibs)) {
        if (!fs.existsSync(targetDylibs)) {
          logStep(`Copying ${pkg}/.dylibs to app bundle...`);
          try {
            await fs.copy(sourceDylibs, targetDylibs, {
              overwrite: true,
              preserveTimestamps: true
            });
            const files = fs.readdirSync(targetDylibs);
            logSuccess(`Copied ${pkg}/.dylibs (${files.length} files)`);
            fixedCount++;
          } catch (err) {
            logError(`Failed to copy ${pkg}/.dylibs: ${err.message}`);
            missingCount++;
          }
        } else {
          const files = fs.readdirSync(targetDylibs);
          logSuccess(`${pkg}/.dylibs already present (${files.length} files)`);
        }
        break; // Found it, no need to check other paths
      }
    }
  }
  
  return { fixedCount, missingCount };
}

async function fixCriticalFiles(appPath) {
  console.log(`\n${CYAN}📄 Fixing critical configuration files...${RESET}`);
  
  const pythonBasePath = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime');
  const sourcePythonPath = path.join(__dirname, '../resources/python-runtime');
  
  const criticalFiles = [
    {
      name: '.needs_extraction',
      content: JSON.stringify({
        created: new Date().toISOString(),
        version: require('../package.json').version,
        reason: 'Force full extraction to preserve dylib paths'
      }, null, 2)
    },
    {
      name: '.memory_config',
      content: JSON.stringify({
        'PYTORCH_CUDA_ALLOC_CONF': 'max_split_size_mb:512',
        'OMP_NUM_THREADS': '2',
        'MKL_NUM_THREADS': '2',
        'NUMEXPR_NUM_THREADS': '2',
        'TOKENIZERS_PARALLELISM': 'false',
        'created': new Date().toISOString(),
        'purpose': 'Prevent 1.3TB memory allocation crash'
      }, null, 2)
    }
  ];
  
  let fixedCount = 0;
  
  for (const file of criticalFiles) {
    const targetPath = path.join(pythonBasePath, file.name);
    
    if (!fs.existsSync(targetPath)) {
      logStep(`Creating ${file.name}...`);
      try {
        fs.writeFileSync(targetPath, file.content);
        logSuccess(`Created ${file.name}`);
        fixedCount++;
      } catch (err) {
        logError(`Failed to create ${file.name}: ${err.message}`);
      }
    } else {
      logSuccess(`${file.name} already present`);
    }
  }
  
  return fixedCount;
}

async function fixBinaryPermissions(appPath) {
  console.log(`\n${CYAN}🔐 Fixing binary permissions...${RESET}`);
  
  const binaries = [
    'Contents/Resources/app.asar.unpacked/.webpack/main/binaries/hive-backend-server-enhanced',
    'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime/python/bin/python3'
  ];
  
  let fixedCount = 0;
  
  for (const binaryPath of binaries) {
    const fullPath = path.join(appPath, binaryPath);
    const binaryName = path.basename(fullPath);
    
    if (fs.existsSync(fullPath)) {
      try {
        fs.chmodSync(fullPath, 0o755);
        logSuccess(`Set execute permission on ${binaryName}`);
        fixedCount++;
      } catch (err) {
        logError(`Failed to set permission on ${binaryName}: ${err.message}`);
      }
    } else {
      logWarning(`Binary not found: ${binaryName}`);
    }
  }
  
  return fixedCount;
}

async function verifyFixes(appPath) {
  console.log(`\n${CYAN}🔍 Verifying all fixes...${RESET}`);
  
  const pythonBasePath = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/resources/python-runtime');
  
  // Check .dylibs
  const dylibsToCheck = ['PIL', 'sklearn', 'scipy'];
  let dylibsOk = 0;
  
  for (const pkg of dylibsToCheck) {
    const dylibPath = path.join(pythonBasePath, `python/lib/python3.11/site-packages/${pkg}/.dylibs`);
    if (fs.existsSync(dylibPath)) {
      const files = fs.readdirSync(dylibPath);
      if (files.length > 0) {
        logSuccess(`${pkg}/.dylibs verified (${files.length} files)`);
        dylibsOk++;
      } else {
        logWarning(`${pkg}/.dylibs directory empty`);
      }
    } else {
      logError(`${pkg}/.dylibs missing`);
    }
  }
  
  // Check critical files
  const extractionMarker = path.join(pythonBasePath, '.needs_extraction');
  const memoryConfig = path.join(pythonBasePath, '.memory_config');
  
  if (fs.existsSync(extractionMarker)) {
    logSuccess('Extraction marker verified');
  } else {
    logError('Extraction marker missing');
  }
  
  if (fs.existsSync(memoryConfig)) {
    logSuccess('Memory config verified');
  } else {
    logError('Memory config missing');
  }
  
  return dylibsOk >= dylibsToCheck.length;
}

async function main() {
  console.log(`\n${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
  console.log(`${CYAN}                 POST-BUILD FIXES FOR PRODUCTION                ${RESET}`);
  console.log(`${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
  
  // Find the app bundle
  const appPath = path.join(__dirname, '../out/Hive Consensus-darwin-arm64/Hive Consensus.app');
  
  if (!fs.existsSync(appPath)) {
    logError('App bundle not found! Build must complete first.');
    process.exit(1);
  }
  
  console.log(`${GREEN}✓ Found app bundle${RESET}`);
  
  // Apply fixes
  const { fixedCount: dylibsFixed, missingCount: dylibsMissing } = await fixDylibsInAppBundle(appPath);
  const criticalFixed = await fixCriticalFiles(appPath);
  const permissionsFixed = await fixBinaryPermissions(appPath);
  
  // Verify everything
  const allGood = await verifyFixes(appPath);
  
  // Summary
  console.log(`\n${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
  console.log(`${CYAN}                           SUMMARY                              ${RESET}`);
  console.log(`${CYAN}═══════════════════════════════════════════════════════════════${RESET}`);
  console.log(`  .dylibs fixed: ${dylibsFixed}`);
  console.log(`  Critical files fixed: ${criticalFixed}`);
  console.log(`  Permissions fixed: ${permissionsFixed}`);
  
  if (allGood) {
    console.log(`\n${GREEN}✅ All critical fixes applied successfully!${RESET}`);
    console.log(`${GREEN}   The app should now run without consensus routing issues.${RESET}`);
  } else {
    console.log(`\n${YELLOW}⚠️  Some issues remain. Check the output above.${RESET}`);
  }
}

// Run if executed directly
if (require.main === module) {
  main().catch(err => {
    console.error(`${RED}Fatal error: ${err.message}${RESET}`);
    process.exit(1);
  });
}

module.exports = { fixDylibsInAppBundle, fixCriticalFiles, fixBinaryPermissions, verifyFixes };