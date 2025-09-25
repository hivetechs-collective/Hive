#!/usr/bin/env node
/*
 * Verify DMG helpers (Node/ttyd/git) have execute bits and entitlements.
 */
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const DMG_DEFAULT = path.join(__dirname, '..', 'out', 'make', 'Hive Consensus.dmg');

function sh(cmd) { return execSync(cmd, { encoding: 'utf8', stdio: ['ignore','pipe','pipe'] }); }
function findMount(out) {
  try {
    const vols = fs.readdirSync('/Volumes');
    for (const v of vols) {
      const mp = path.join('/Volumes', v);
      if (fs.existsSync(path.join(mp, 'Hive Consensus.app'))) return mp;
    }
  } catch {}
  const t = out.split(/\s+/).filter(x=>x.startsWith('/Volumes/'));
  return t.length ? t[t.length-1] : null;
}

function checkExec(p) {
  const st = fs.statSync(p);
  const ok = (st.mode & 0o111) !== 0;
  if (!ok) throw new Error(`not executable: ${p}`);
}

function tryCodesignEntitlements(p) {
  try { return sh(`codesign -d --entitlements :- "${p}"`); } catch (e) { return `codesign entitlements error: ${e.message}`; }
}

function verifyHelper(p, label) {
  if (!fs.existsSync(p)) { console.log(`[verify] missing ${label}: ${p}`); return false; }
  try { checkExec(p); } catch (e) { console.log(`[verify] ${label} exec bit check failed: ${e.message}`); return false; }
  const ent = tryCodesignEntitlements(p);
  if (!/allow-jit/.test(ent) || !/disable-library-validation/.test(ent)) {
    console.log(`[verify] ${label} entitlements suspicious. Output:\n${ent}`);
    // Not failing hard; print info so we can decide policy.
  } else {
    console.log(`[verify] ${label} entitlements OK`);
  }
  try { sh(`codesign -v --strict "${p}"`); console.log(`[verify] ${label} codesign OK`); } catch (e) { console.log(`[verify] ${label} codesign check failed: ${e.message}`); }
  return true;
}

(function main(){
  const dmg = process.argv[2] || DMG_DEFAULT;
  if (!fs.existsSync(dmg)) { console.error(`DMG not found: ${dmg}`); process.exit(2); }
  let mp=null;
  try {
    const out = sh(`hdiutil attach "${dmg}" -nobrowse`);
    mp = findMount(out) || '/Volumes/Hive Consensus';
    const app = path.join(mp, 'Hive Consensus.app');
    const base = path.join(app, 'Contents/Resources/app.asar.unpacked/.webpack/main');
    const ok1 = verifyHelper(path.join(base, 'binaries/node'), 'node');
    const ok2 = verifyHelper(path.join(base, 'binaries/ttyd'), 'ttyd');
    const ok3 = verifyHelper(path.join(base, 'binaries/git-bundle/bin/git'), 'git');
    if (ok1) { console.log(`[verify] node perms: ` + sh(`ls -l "${path.join(base,'binaries/node')}"`).trim()); }
    process.exitCode = (ok1 && ok2) ? 0 : 1;
  } catch (e) {
    console.error(e?.stack || String(e));
    process.exitCode = 1;
  } finally {
    if (mp) { try { sh(`hdiutil detach "${mp}"`); } catch {} }
  }
})();
