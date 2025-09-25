#!/usr/bin/env node
/*
 * DMG-mounted Memory Service harness
 * Mounts a DMG, launches the packaged memory-service via the bundled Node
 * runtime on a dynamic port, verifies /health, then detaches the DMG.
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

const DMG_DEFAULT = path.join(__dirname, '..', 'out', 'make', 'Hive Consensus.dmg');

function sh(cmd, opts = {}) {
  return execSync(cmd, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'], ...opts });
}

function findMountPointFromAttachOutput(out) {
  // Prefer a mount that actually contains our app
  try {
    const vols = fs.readdirSync('/Volumes');
    for (const v of vols) {
      const mp = path.join('/Volumes', v);
      if (fs.existsSync(path.join(mp, 'Hive Consensus.app'))) return mp;
    }
  } catch {}
  // Fallback: last /Volumes token from attach output
  const tokens = out.split(/\s+/);
  const vols2 = tokens.filter(t => t.startsWith('/Volumes/'));
  return vols2.length ? vols2[vols2.length - 1] : null;
}

async function findFreePort(start = 3900, end = 3999) {
  const net = require('net');
  function tryPort(p) {
    return new Promise((resolve) => {
      const server = net.createServer();
      server.unref();
      server.on('error', () => resolve(null));
      server.listen(p, '127.0.0.1', () => {
        const { port } = server.address();
        server.close(() => resolve(port));
      });
    });
  }
  for (let p = start; p <= end; p++) {
    const free = await tryPort(p);
    if (free) return free;
  }
  throw new Error('No free port found');
}

async function waitForHealth(port, timeoutMs = 15000) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/health`, { signal: AbortSignal.timeout(1500) });
      if (res.ok) {
        const body = await res.json();
        if (body && body.status === 'healthy') return body;
      }
    } catch {}
    await new Promise(r => setTimeout(r, 300));
  }
  throw new Error('Timed out waiting for /health');
}

async function main() {
  const dmgPath = process.argv[2] || DMG_DEFAULT;
  if (!fs.existsSync(dmgPath)) {
    console.error(`DMG not found: ${dmgPath}`);
    process.exit(2);
  }
  let mountPoint = null;
  try {
    console.log(`[dmg-harness] Attaching DMG: ${dmgPath}`);
    const out = sh(`hdiutil attach "${dmgPath}" -nobrowse`);
    mountPoint = findMountPointFromAttachOutput(out) || '/Volumes/Hive Consensus';
    console.log(`[dmg-harness] Mounted at: ${mountPoint}`);

    const appPath = path.join(mountPoint, 'Hive Consensus.app');
    const nodePath = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/binaries/node');
    const entry = path.join(appPath, 'Contents/Resources/app.asar.unpacked/.webpack/main/memory-service/index.js');

    for (const p of [appPath, nodePath, entry]) {
      if (!fs.existsSync(p)) {
        console.error(`[dmg-harness] Missing path: ${p}`);
        process.exit(3);
      }
    }

    try {
      const ls = sh(`ls -l "${nodePath}"`);
      console.log(`[dmg-harness] Node perms: ${ls.trim()}`);
    } catch {}
    try {
      const ent = sh(`codesign -d --entitlements :- "${nodePath}"`);
      console.log(`[dmg-harness] Node entitlements:\n${ent}`);
    } catch (e) {
      console.log(`[dmg-harness] entitlements read failed: ${e.message}`);
    }

    const port = await findFreePort();
    console.log(`[dmg-harness] Launching memory-service on port ${port}`);

    const env = { ...process.env, NODE_ENV: 'production', PORT: String(port), MEMORY_SERVICE_PORT: String(port), ELECTRON_RUN_AS_NODE: '1' };
    const child = spawn(nodePath, [entry], { env, stdio: ['ignore', 'pipe', 'pipe'] });

    let stdout = '';
    let stderr = '';
    child.stdout.on('data', d => { stdout += d.toString(); });
    child.stderr.on('data', d => { stderr += d.toString(); });

    try {
      const health = await waitForHealth(port, 20000);
      console.log(`[dmg-harness] Health OK: ${JSON.stringify(health)}`);
    } catch (e) {
      console.error('[dmg-harness] Health check failed');
      if (stdout) console.error('stdout:\n' + stdout);
      if (stderr) console.error('stderr:\n' + stderr);
      try { console.error(sh(`codesign -dv --verbose=4 "${nodePath}"`)); } catch {}
      process.exitCode = 1;
    } finally {
      try { child.kill('SIGTERM'); } catch {}
      await new Promise(r => setTimeout(r, 400));
    }
  } finally {
    if (mountPoint) {
      try {
        console.log('[dmg-harness] Detaching DMG...');
        sh(`hdiutil detach "${mountPoint}"`);
      } catch (e) {
        console.log(`[dmg-harness] detach failed: ${e.message}`);
      }
    }
  }
}

main().catch(err => { console.error(err?.stack || String(err)); process.exit(1); });
