#!/usr/bin/env node
/*
 * Smoke-test ttyd inside the packaged DMG by mounting it, spawning ttyd
 * with the bundled libs/plugins, probing HTTP, and then cleaning up.
 */
const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const net = require('net');

function sh(cmd) {
  return execSync(cmd, { encoding: 'utf8', stdio: ['ignore','pipe','pipe'] });
}

async function portIsUp(port, host='127.0.0.1') {
  return new Promise((resolve) => {
    const sock = net.createConnection({ port, host });
    const t = setTimeout(() => { try { sock.destroy(); } catch {} ; resolve(false); }, 150);
    sock.on('connect', () => { clearTimeout(t); try { sock.destroy(); } catch {}; resolve(true); });
    sock.on('error', () => { clearTimeout(t); resolve(false); });
  });
}

async function waitHttp(url, timeoutMs=6000, intervalMs=200) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url, { cache: 'no-store' });
      if (res.ok) return true;
    } catch {}
    await new Promise(r => setTimeout(r, intervalMs));
  }
  return false;
}

function findMount() {
  try {
    const vols = fs.readdirSync('/Volumes');
    for (const v of vols) {
      const mp = path.join('/Volumes', v);
      if (fs.existsSync(path.join(mp, 'Hive Consensus.app'))) return mp;
    }
  } catch {}
  return null;
}

(async function main() {
  const dmg = process.argv[2] || path.join(__dirname, '..', 'out', 'make', 'Hive Consensus.dmg');
  if (!fs.existsSync(dmg)) {
    console.error(`[smoke-ttyd] DMG not found: ${dmg}`);
    process.exit(2);
  }

  let mp = null;
  let ttyd = null;
  let libsDir = null;
  let child = null;
  try {
    const out = sh(`hdiutil attach "${dmg}" -nobrowse`);
    mp = findMount() || '/Volumes/Hive Consensus';
    const base = path.join(mp, 'Hive Consensus.app', 'Contents', 'Resources', 'app.asar.unpacked', '.webpack', 'main', 'binaries');
    ttyd = path.join(base, 'ttyd');
    libsDir = path.join(base, 'ttyd-libs');
    if (!fs.existsSync(ttyd)) throw new Error(`ttyd not found in DMG: ${ttyd}`);

    // Pick a port
    const port = 7785 + Math.floor(Math.random() * 50);
    const args = ['--port', String(port), '--interface', '127.0.0.1', '--once', '--writable', 'bash'];

    // Env: prefer bundled libs; if plugin present, set LWS_PLUGINS; else disable plugins
    const env = { ...process.env };
    const hasLibs = fs.existsSync(libsDir);
    const hasEvlib = hasLibs && fs.readdirSync(libsDir).some(n => n.startsWith('libwebsockets-evlib') && n.endsWith('.dylib'));
    if (hasLibs) {
      env.DYLD_LIBRARY_PATH = env.DYLD_LIBRARY_PATH ? `${libsDir}:${env.DYLD_LIBRARY_PATH}` : libsDir;
      env.DYLD_FALLBACK_LIBRARY_PATH = env.DYLD_FALLBACK_LIBRARY_PATH ? `${libsDir}:${env.DYLD_FALLBACK_LIBRARY_PATH}` : libsDir;
    }
    // Always prefer pluginless operation in smoke test
    env.LWS_NO_PLUGINS = '1';

    child = spawn(ttyd, args, { stdio: ['ignore','pipe','pipe'], env });
    let stderrBuf = '';
    child.stderr.on('data', d => { stderrBuf += d.toString(); });

    // Wait for port + HTTP health
    let ok = false;
    for (let i=0;i<30;i++) {
      if (await portIsUp(port) && await waitHttp(`http://127.0.0.1:${port}/`)) { ok = true; break; }
      await new Promise(r => setTimeout(r, 200));
    }

    if (!ok) {
      console.error(`[smoke-ttyd] ttyd did not become ready on ${port}`);
      if (stderrBuf) console.error('[smoke-ttyd] stderr:\n' + stderrBuf.trim());
      process.exitCode = 1;
    } else {
      console.log(`[smoke-ttyd] PASS: ttyd responded on http://127.0.0.1:${port}/`);
      process.exitCode = 0;
    }
  } catch (e) {
    console.error('[smoke-ttyd] error:', e?.message || String(e));
    process.exitCode = 1;
  } finally {
    try { if (child && !child.killed) { child.kill('SIGTERM'); } } catch {}
    if (mp) { try { sh(`hdiutil detach "${mp}"`); } catch {} }
  }
})();
