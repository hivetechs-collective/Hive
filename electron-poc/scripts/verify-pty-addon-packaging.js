#!/usr/bin/env node
/*
 Verifies hive_pty.node exists in packaged app under app.asar.unpacked/native/hive_pty.
 Usage: node scripts/verify-pty-addon-packaging.js
*/
const fs = require('fs');
const path = require('path');

const OUT = path.resolve(__dirname, '..', 'out');
let found = [];
function walk(dir) {
  let entries; try { entries = fs.readdirSync(dir, { withFileTypes: true }); } catch { return; }
  for (const e of entries) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p);
    else if (e.isFile() && e.name === 'hive_pty.node' && p.includes(path.join('app.asar.unpacked','native','hive_pty'))) {
      found.push(p);
    }
  }
}

walk(OUT);

if (found.length) {
  console.log('[verify-pty-addon-packaging] OK');
  for (const p of found) console.log(' -', p);
  process.exit(0);
} else {
  console.error('[verify-pty-addon-packaging] FAILED: hive_pty.node not found under app.asar.unpacked/native/hive_pty');
  process.exit(1);
}

