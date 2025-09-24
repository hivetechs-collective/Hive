#!/usr/bin/env node
// Lightweight DB tests for Welcome page persistence using a local sqlite file
const path = require('path');
const fs = require('fs');
const sqlite3 = require('sqlite3').verbose();

const dbPath = path.join(__dirname, '..', 'hive-ai.db');
console.log('Using test database at:', dbPath);

const db = new sqlite3.Database(dbPath);

function run(sql, params = []) {
  return new Promise((resolve, reject) => {
    db.run(sql, params, function (err) {
      if (err) reject(err); else resolve(this);
    });
  });
}

function get(sql, params = []) {
  return new Promise((resolve, reject) => {
    db.get(sql, params, (err, row) => {
      if (err) reject(err); else resolve(row);
    });
  });
}

function all(sql, params = []) {
  return new Promise((resolve, reject) => {
    db.all(sql, params, (err, rows) => {
      if (err) reject(err); else resolve(rows);
    });
  });
}

(async () => {
  try {
    // Ensure schema
    await run(`CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL,
      updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )`);
    await run(`CREATE TABLE IF NOT EXISTS recent_folders (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      folder_path TEXT NOT NULL UNIQUE,
      last_opened TEXT DEFAULT CURRENT_TIMESTAMP,
      tab_count INTEGER DEFAULT 0
    )`);

    // Settings round-trip for welcome.showOnStartup
    const key = 'welcome.showOnStartup';
    const prev = await get('SELECT value FROM settings WHERE key=?', [key]);
    await run('INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)', [key, '1']);
    const got1 = await get('SELECT value FROM settings WHERE key=?', [key]);
    if (!got1 || got1.value !== '1') throw new Error('Failed to set welcome.showOnStartup=1');
    await run('INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)', [key, '0']);
    const got0 = await get('SELECT value FROM settings WHERE key=?', [key]);
    if (!got0 || got0.value !== '0') throw new Error('Failed to set welcome.showOnStartup=0');
    // Restore previous value
    if (prev && typeof prev.value !== 'undefined') {
      await run('INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)', [key, prev.value]);
    }

    // Recents add/remove
    const testPath = path.join(process.cwd(), 'tmp-recent-' + Date.now());
    await run(`INSERT OR REPLACE INTO recent_folders (folder_path, last_opened, tab_count) VALUES (?, CURRENT_TIMESTAMP, 3)`, [testPath]);
    const rows = await all('SELECT folder_path FROM recent_folders WHERE folder_path=?', [testPath]);
    if (!rows || rows.length !== 1) throw new Error('Failed to add recent folder');
    await run('DELETE FROM recent_folders WHERE folder_path=?', [testPath]);
    const rowsAfter = await all('SELECT folder_path FROM recent_folders WHERE folder_path=?', [testPath]);
    if (rowsAfter.length !== 0) throw new Error('Failed to remove recent folder');

    // Session insert/load sanity
    await run(`CREATE TABLE IF NOT EXISTS sessions (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      folder_path TEXT NOT NULL UNIQUE,
      tabs TEXT NOT NULL,
      active_tab TEXT,
      created_at TEXT DEFAULT CURRENT_TIMESTAMP,
      updated_at TEXT DEFAULT CURRENT_TIMESTAMP
    )`);
    const tabsJson = JSON.stringify([{ path: 'README.md', unsaved: false }]);
    await run(`INSERT OR REPLACE INTO sessions (folder_path, tabs, active_tab, updated_at) VALUES (?, ?, ?, CURRENT_TIMESTAMP)`, [testPath, tabsJson, 'README.md']);
    const sess = await get('SELECT tabs, active_tab FROM sessions WHERE folder_path=?', [testPath]);
    if (!sess) throw new Error('Failed to insert session');
    await run('DELETE FROM sessions WHERE folder_path=?', [testPath]);

    console.log('✓ DB settings and recents round-trip passed');
    db.close();
  } catch (err) {
    console.error('DB test failed:', err);
    db.close();
    process.exit(1);
  }
})();
