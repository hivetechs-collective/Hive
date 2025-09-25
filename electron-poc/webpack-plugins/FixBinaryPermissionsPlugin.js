const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

class FixBinaryPermissionsPlugin {
  apply(compiler) {
    compiler.hooks.afterEmit.tapAsync('FixBinaryPermissionsPlugin', (compilation, callback) => {
      const outputPath = compilation.outputOptions.path;

      // 1. Fix Python binary permissions
      const pythonBinPath = path.join(outputPath, 'resources', 'python-runtime', 'python', 'bin');
      if (fs.existsSync(pythonBinPath)) {
        console.log('[FixBinaryPermissionsPlugin] Fixing Python binary permissions...');
        try {
          execSync(`chmod +x "${pythonBinPath}"/*`, { stdio: 'inherit' });
          console.log('[FixBinaryPermissionsPlugin] Python binary permissions fixed!');
        } catch (error) {
          console.error('[FixBinaryPermissionsPlugin] Failed to fix Python permissions:', error.message);
        }
      }
      
      // 2. Fix Backend Server binary permissions
      const backendServerPath = path.join(outputPath, 'binaries', 'hive-backend-server-enhanced');
      if (fs.existsSync(backendServerPath)) {
        console.log('[FixBinaryPermissionsPlugin] Fixing Backend Server permissions...');
        try {
          execSync(`chmod +x "${backendServerPath}"`, { stdio: 'inherit' });
          console.log('[FixBinaryPermissionsPlugin] Backend Server permissions fixed!');
        } catch (error) {
          console.error('[FixBinaryPermissionsPlugin] Failed to fix Backend Server permissions:', error.message);
        }
      } else {
        console.log('[FixBinaryPermissionsPlugin] Backend Server binary not found at:', backendServerPath);
      }

      // 3. Fix bundled CLI/runtime binaries (Node, ttyd, git) before packaging
      const cliBinaries = [
        {
          label: 'Node runtime',
          target: path.join(outputPath, 'binaries', 'node')
        },
        {
          label: 'ttyd binary',
          target: path.join(outputPath, 'binaries', 'ttyd')
        },
        {
          label: 'git binary',
          target: path.join(outputPath, 'binaries', 'git-bundle', 'bin', 'git')
        }
      ];

      for (const entry of cliBinaries) {
        if (!entry.target) continue;
        if (fs.existsSync(entry.target)) {
          try {
            fs.chmodSync(entry.target, 0o755);
            console.log(`[FixBinaryPermissionsPlugin] ${entry.label} permissions set to 755`);
          } catch (error) {
            console.error(`[FixBinaryPermissionsPlugin] Failed to set ${entry.label} permissions:`, error.message);
          }
        } else {
          console.log(`[FixBinaryPermissionsPlugin] ${entry.label} not found at: ${entry.target}`);
        }
      }

      callback();
    });
  }
}

module.exports = FixBinaryPermissionsPlugin;
