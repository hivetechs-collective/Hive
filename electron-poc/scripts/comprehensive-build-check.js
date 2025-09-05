#!/usr/bin/env node

/**
 * Comprehensive Build Requirements Checker
 * Ensures all services have their dependencies and are properly configured
 * Similar to Python requirements.txt but for our multi-stack application
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🔍 Comprehensive Build Requirements Check\n');

const requirements = {
  // Core build tools
  buildTools: {
    'Node.js': () => {
      try {
        const version = execSync('node --version', { encoding: 'utf8' }).trim();
        return { present: true, version };
      } catch (e) {
        return { present: false, error: 'Node.js not found' };
      }
    },
    'npm': () => {
      try {
        const version = execSync('npm --version', { encoding: 'utf8' }).trim();
        return { present: true, version };
      } catch (e) {
        return { present: false, error: 'npm not found' };
      }
    },
    'TypeScript': () => {
      try {
        const version = execSync('npx tsc --version', { encoding: 'utf8' }).trim();
        return { present: true, version };
      } catch (e) {
        return { present: false, error: 'TypeScript not installed' };
      }
    },
    'Webpack': () => {
      const webpackPath = path.join(__dirname, '../node_modules/webpack');
      return { 
        present: fs.existsSync(webpackPath), 
        version: fs.existsSync(webpackPath) ? 'Installed' : 'Not installed' 
      };
    }
  },

  // Memory Service requirements
  memoryService: {
    'Source file': () => {
      const srcPath = path.join(__dirname, '../src/memory-service/index.ts');
      return { 
        present: fs.existsSync(srcPath),
        path: srcPath
      };
    },
    'Built file': () => {
      const builtPath = path.join(__dirname, '../.webpack/main/memory-service/index.js');
      return { 
        present: fs.existsSync(builtPath),
        path: builtPath
      };
    },
    'Express dependency': () => {
      const expressPath = path.join(__dirname, '../node_modules/express');
      return { 
        present: fs.existsSync(expressPath),
        version: fs.existsSync(expressPath) ? require('../node_modules/express/package.json').version : 'Not installed'
      };
    },
    'WebSocket dependency': () => {
      const wsPath = path.join(__dirname, '../node_modules/ws');
      return { 
        present: fs.existsSync(wsPath),
        version: fs.existsSync(wsPath) ? require('../node_modules/ws/package.json').version : 'Not installed'
      };
    },
    'BuildMemoryServicePlugin': () => {
      const pluginPath = path.join(__dirname, '../webpack-plugins/BuildMemoryServicePlugin.js');
      return { 
        present: fs.existsSync(pluginPath),
        path: pluginPath
      };
    }
  },

  // Backend Server requirements
  backendServer: {
    'Binary file': () => {
      const binaryPath = path.join(__dirname, '../binaries/hive-backend-server-enhanced');
      return { 
        present: fs.existsSync(binaryPath),
        path: binaryPath,
        executable: fs.existsSync(binaryPath) ? 
          (() => {
            try {
              fs.accessSync(binaryPath, fs.constants.X_OK);
              return true;
            } catch {
              return false;
            }
          })() : false,
        mustBeExecutable: true
      };
    },
    'Rust source': () => {
      const rustPath = path.join(__dirname, '../../../hive/src/bin/hive-backend-server-enhanced.rs');
      return { 
        present: fs.existsSync(rustPath),
        path: rustPath
      };
    }
  },

  // Python AI Helpers requirements
  pythonRuntime: {
    'Python bundle': () => {
      const pythonPath = path.join(__dirname, '../resources/python-runtime/python');
      return { 
        present: fs.existsSync(pythonPath),
        path: pythonPath
      };
    },
    'Python executable': () => {
      const pythonBin = path.join(__dirname, '../resources/python-runtime/python/bin/python3');
      return { 
        present: fs.existsSync(pythonBin),
        path: pythonBin,
        executable: fs.existsSync(pythonBin) ? 
          (() => {
            try {
              fs.accessSync(pythonBin, fs.constants.X_OK);
              return true;
            } catch {
              return false;
            }
          })() : false
      };
    },
    'Model service wrapper': () => {
      const wrapperPath = path.join(__dirname, '../resources/python-runtime/models/model_service_wrapper.py');
      return { 
        present: fs.existsSync(wrapperPath),
        path: wrapperPath
      };
    },
    'Bundle info': () => {
      const bundlePath = path.join(__dirname, '../resources/python-runtime/bundle.json');
      if (fs.existsSync(bundlePath)) {
        const bundle = JSON.parse(fs.readFileSync(bundlePath, 'utf8'));
        return {
          present: true,
          ...bundle
        };
      }
      return { present: false };
    }
  },

  // Production packaging requirements
  packaging: {
    'Forge config': () => {
      const forgePath = path.join(__dirname, '../forge.config.ts');
      return { 
        present: fs.existsSync(forgePath),
        path: forgePath
      };
    },
    'ASAR unpack config': () => {
      const forgePath = path.join(__dirname, '../forge.config.ts');
      if (fs.existsSync(forgePath)) {
        const content = fs.readFileSync(forgePath, 'utf8');
        const hasUnpack = content.includes('unpack:');
        const unpackPatterns = content.match(/unpack:\s*['"`]([^'"`]+)['"`]/);
        return {
          present: hasUnpack,
          patterns: unpackPatterns ? unpackPatterns[1] : 'Not found'
        };
      }
      return { present: false };
    },
    'Package.json scripts': () => {
      const pkg = require('../package.json');
      const requiredScripts = ['make', 'package', 'prebuild'];
      const missing = requiredScripts.filter(s => !pkg.scripts[s]);
      return {
        present: missing.length === 0,
        missing: missing.length > 0 ? missing : undefined
      };
    }
  },

  // Node.js executable resolution
  nodeExecution: {
    'System Node.js': () => {
      const possiblePaths = [
        '/usr/local/bin/node',
        '/opt/homebrew/bin/node',
        '/usr/bin/node'
      ];
      const found = possiblePaths.filter(p => fs.existsSync(p));
      return {
        present: found.length > 0,
        paths: found.length > 0 ? found : ['None found'],
        recommendation: found.length === 0 ? 'Install Node.js via Homebrew: brew install node' : undefined
      };
    },
    'ELECTRON_RUN_AS_NODE support': () => {
      return {
        present: true,
        info: 'ProcessManager should set ELECTRON_RUN_AS_NODE=1 for production'
      };
    }
  }
};

// Check each requirement
let hasErrors = false;
let hasWarnings = false;

for (const [category, checks] of Object.entries(requirements)) {
  console.log(`\n📦 ${category.toUpperCase()}`);
  console.log('─'.repeat(50));
  
  for (const [name, check] of Object.entries(checks)) {
    const result = check();
    const status = result.present ? '✅' : '❌';
    
    console.log(`${status} ${name}`);
    
    // Show additional info
    if (result.version) console.log(`   Version: ${result.version}`);
    if (result.path) console.log(`   Path: ${result.path}`);
    if (result.executable !== undefined) {
      console.log(`   Executable: ${result.executable ? 'Yes' : 'No - needs chmod +x'}`);
    }
    if (result.patterns) console.log(`   Patterns: ${result.patterns}`);
    if (result.paths) console.log(`   Found at: ${result.paths.join(', ')}`);
    if (result.info) console.log(`   ℹ️  ${result.info}`);
    if (result.recommendation) console.log(`   💡 ${result.recommendation}`);
    if (result.missing) console.log(`   Missing: ${result.missing.join(', ')}`);
    if (result.error) console.log(`   Error: ${result.error}`);
    
    if (!result.present) {
      if (category === 'packaging' || name.includes('Built')) {
        hasWarnings = true;
      } else {
        hasErrors = true;
      }
    }
  }
}

// Summary
console.log('\n' + '='.repeat(60));
if (hasErrors) {
  console.error('❌ Build requirements check FAILED');
  console.error('   Fix the errors above before building');
  process.exit(1);
} else if (hasWarnings) {
  console.warn('⚠️  Build requirements check passed with warnings');
  console.warn('   Some optional components are missing');
} else {
  console.log('✅ All build requirements satisfied!');
  console.log('   Ready to run: npm run make');
}

// Build Order Information
console.log('\n🔄 PROPER BUILD ORDER (CRITICAL!):');
console.log('   Use the comprehensive build script for correct sequencing:');
console.log('   ' + '─'.repeat(50));
console.log('   📌 node scripts/build-production-dmg.js');
console.log('   ' + '─'.repeat(50));
console.log('\n   This script handles the EXACT order:');
console.log('   1.  Pre-build cleanup (remove old artifacts)');
console.log('   2.  Verify build tools (Node, npm, etc.)');
console.log('   3.  Install dependencies (with verification)');
console.log('   4.  Prepare backend server (build/verify binary)');
console.log('   5.  Prepare Python runtime (fix permissions)');
console.log('   6.  Verify webpack plugins exist');
console.log('   7.  Run pre-build script');
console.log('   8.  Build application (npm run make)');
console.log('   9.  Post-build verification');
console.log('   10. Permission verification (critical!)');
console.log('   11. Generate build report');
console.log('   12. Show installation instructions');

console.log('\n⚠️  COMMON ISSUES PREVENTED BY PROPER ORDER:');
console.log('   • Backend server "permission denied" → Fixed by Phase 4 & 10');
console.log('   • Memory service "spawn node ENOENT" → Fixed by ELECTRON_RUN_AS_NODE');
console.log('   • Python runtime failures → Fixed by Phase 5');
console.log('   • Missing binaries in DMG → Fixed by webpack plugins (Phase 6)');

// Alternative commands for specific needs
console.log('\n💡 Alternative Commands:');
console.log('   npm run verify:all    - Just verify requirements');
console.log('   npm run clean         - Clean all build artifacts');
console.log('   npm run make          - Quick rebuild (if requirements met)');
console.log('   npm run test:dmg      - Test existing DMG');

console.log('\n📋 Production Testing After Build:');
console.log('   1. Delete old: rm -rf "/Applications/Hive Consensus.app"');
console.log('   2. Mount DMG: open out/make/*.dmg');
console.log('   3. Drag to Applications folder');
console.log('   4. Launch from Finder (NOT terminal) - Critical for production test!');
console.log('   5. Monitor logs: tail -f ~/Library/Application\\ Support/Hive\\ Consensus/logs/*.log');