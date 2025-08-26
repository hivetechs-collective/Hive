const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class FixPythonPermissionsPlugin {
  apply(compiler) {
    compiler.hooks.afterEmit.tapAsync('FixPythonPermissionsPlugin', (compilation, callback) => {
      const outputPath = compilation.outputOptions.path;
      const pythonBinPath = path.join(outputPath, 'resources', 'python-runtime', 'python', 'bin');
      
      // Check if the Python bin directory exists
      if (fs.existsSync(pythonBinPath)) {
        console.log('[FixPythonPermissionsPlugin] Fixing Python binary permissions...');
        
        try {
          // Make all files in bin directory executable
          execSync(`chmod +x "${pythonBinPath}"/*`, { stdio: 'inherit' });
          console.log('[FixPythonPermissionsPlugin] Python binary permissions fixed!');
        } catch (error) {
          console.error('[FixPythonPermissionsPlugin] Failed to fix permissions:', error.message);
        }
      } else {
        console.log('[FixPythonPermissionsPlugin] Python bin directory not found, skipping permissions fix');
      }
      
      callback();
    });
  }
}

module.exports = FixPythonPermissionsPlugin;