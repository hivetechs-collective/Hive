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

      // 3. Ensure helper binaries (node/ttyd/git) are executable BEFORE packaging
      const helpers = [
        { label: 'node', p: path.join(outputPath, 'binaries', 'node') },
        { label: 'ttyd', p: path.join(outputPath, 'binaries', 'ttyd') },
        { label: 'git',  p: path.join(outputPath, 'binaries', 'git-bundle', 'bin', 'git') },
        { label: 'uv',   p: path.join(outputPath, 'binaries', 'uv') },
        { label: 'npm',  p: path.join(outputPath, 'binaries', 'npm') },
        { label: 'npx',  p: path.join(outputPath, 'binaries', 'npx') },
      ];
      for (const h of helpers) {
        if (fs.existsSync(h.p)) {
          try {
            execSync(`chmod 755 "${h.p}"`, { stdio: 'inherit' });
            console.log(`[FixBinaryPermissionsPlugin] Ensured exec bit for ${h.label}: ${h.p}`);
          } catch (error) {
            console.error(`[FixBinaryPermissionsPlugin] Failed chmod for ${h.label}:`, error.message);
          }
        }
      }

      // 3a. Best-effort: if git-core exists, mark its binaries executable as well
      const gitCoreDir = path.join(outputPath, 'binaries', 'git-bundle', 'libexec', 'git-core');
      if (fs.existsSync(gitCoreDir)) {
        try {
          execSync(`chmod -R 755 "${gitCoreDir}"`, { stdio: 'inherit' });
          console.log('[FixBinaryPermissionsPlugin] Ensured exec bits for git-core helpers');
        } catch (error) {
          console.error('[FixBinaryPermissionsPlugin] Failed to chmod git-core helpers:', error.message);
        }
      }
      
      callback();
    });
  }
}

module.exports = FixBinaryPermissionsPlugin;
