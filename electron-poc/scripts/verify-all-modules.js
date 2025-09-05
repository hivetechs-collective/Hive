#!/usr/bin/env node

/**
 * Verify all module imports are valid BEFORE attempting to build
 * This prevents webpack from failing late in the build process
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🔍 Verifying all module imports...\n');

let hasErrors = false;
const errors = [];

// Step 1: Run TypeScript compiler in check mode
console.log('📝 Running TypeScript type checking...');
try {
  execSync('npx tsc --noEmit', { 
    stdio: 'pipe',
    encoding: 'utf8' 
  });
  console.log('✅ TypeScript compilation successful');
} catch (error) {
  console.error('❌ TypeScript compilation failed');
  
  // Parse the error to find missing modules
  const output = error.stdout + error.stderr;
  const lines = output.split('\n');
  
  lines.forEach(line => {
    if (line.includes("Cannot find module") || line.includes("Could not find a declaration file")) {
      errors.push(line.trim());
      hasErrors = true;
    }
  });
  
  if (errors.length > 0) {
    console.error('\nMissing modules detected:');
    errors.forEach(err => console.error(`  - ${err}`));
  }
}

// Step 2: Check for common problematic imports
console.log('\n🔎 Checking for problematic imports...');
const srcDir = path.join(__dirname, '../src');
const problematicPatterns = [
  /import.*from\s+['"]\.\/utils\/RuntimeVerification['"]/g,
  /import.*from\s+['"]\.\/utils\/WorkerManager['"]/g,
  /import.*from\s+['"]\.\/memory-service\/worker['"]/g,
  /require\(['"]\.\/utils\/RuntimeVerification['"]\)/g,
];

function checkFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  const fileName = path.relative(process.cwd(), filePath);
  
  problematicPatterns.forEach(pattern => {
    const matches = content.match(pattern);
    if (matches) {
      matches.forEach(match => {
        errors.push(`${fileName}: Found problematic import: ${match}`);
        hasErrors = true;
      });
    }
  });
}

function walkDirectory(dir) {
  const files = fs.readdirSync(dir);
  
  files.forEach(file => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    
    if (stat.isDirectory() && !file.startsWith('.') && file !== 'node_modules') {
      walkDirectory(filePath);
    } else if (file.endsWith('.ts') || file.endsWith('.tsx')) {
      checkFile(filePath);
    }
  });
}

walkDirectory(srcDir);

// Step 3: Verify critical files exist
console.log('\n📁 Verifying critical files exist...');
const criticalFiles = [
  'src/index.ts',
  'src/preload.ts',
  'src/renderer.ts',
  'src/utils/ProcessManager.ts',
  'src/utils/PortManager.ts',
  'src/utils/SafeLogger.ts',
  'src/startup/StartupOrchestrator.ts',
  'webpack.main.config.ts',
  'webpack.renderer.config.ts',
  'forge.config.ts',
];

criticalFiles.forEach(file => {
  const filePath = path.join(__dirname, '..', file);
  if (!fs.existsSync(filePath)) {
    errors.push(`Missing critical file: ${file}`);
    hasErrors = true;
  }
});

// Step 4: Check for circular dependencies
console.log('\n🔄 Checking for circular dependencies...');
try {
  // Note: This would require dependency-cruiser to be installed
  // For now, we'll skip this check if not available
  execSync('which depcruise', { stdio: 'ignore' });
  const result = execSync('npx depcruise src --validate', { 
    encoding: 'utf8',
    stdio: 'pipe'
  });
  console.log('✅ No circular dependencies found');
} catch (error) {
  // Dependency cruiser not installed or found issues
  if (error.stdout && error.stdout.includes('circular')) {
    console.warn('⚠️ Circular dependencies detected (non-critical)');
  }
}

// Final report
console.log('\n' + '='.repeat(60));
if (hasErrors) {
  console.error('❌ Module verification FAILED!\n');
  console.error('Found the following issues:');
  errors.forEach(err => console.error(`  • ${err}`));
  console.error('\nFix these issues before building:');
  console.error('1. Remove or create missing modules');
  console.error('2. Update imports to point to existing modules');
  console.error('3. Run "npm run verify:modules" again');
  process.exit(1);
} else {
  console.log('✅ All modules verified successfully!');
  console.log('Safe to proceed with build.');
}