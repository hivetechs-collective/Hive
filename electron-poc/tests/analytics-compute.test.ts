import assert from 'assert';
import { Database } from 'sqlite3';
import { computeAnalytics } from '../src/utils/analytics-compute';

function test(name: string, fn: () => Promise<void> | void) {
  Promise.resolve(fn()).then(() => console.log(`✓ ${name}`)).catch((e) => { console.error(`✗ ${name}`); throw e; });
}

const USER_ID = '3034c561-e193-4968-a575-f1b165d31a5b';

test('analytics updates after inserting a conversation (24h)', async () => {
  const dbPath = __dirname + '/../hive-ai.db';
  const db = new Database(dbPath);

  // Ensure minimal schema
  await run(db, `CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    title TEXT,
    profile_id TEXT,
    total_cost REAL DEFAULT 0,
    total_tokens_input INTEGER DEFAULT 0,
    total_tokens_output INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
  )`, []);
  await run(db, `CREATE TABLE IF NOT EXISTS conversation_usage (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP
  )`, []);
  await run(db, `CREATE TABLE IF NOT EXISTS performance_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id TEXT NOT NULL,
    timestamp TEXT DEFAULT CURRENT_TIMESTAMP,
    total_duration INTEGER,
    total_cost REAL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
  )`, []);

  const convId = `conv_test_${Date.now()}`;
  const now = new Date().toISOString();

  // Ensure parent rows
  await run(db, `INSERT OR IGNORE INTO conversations (id, user_id, title, total_cost, total_tokens_input, total_tokens_output, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)`, [
    convId, USER_ID, 'Analytics Test', 1.23, 100, 50, now, now
  ]);
  await run(db, `INSERT INTO conversation_usage (user_id, conversation_id, timestamp) VALUES (?, ?, ?)`, [USER_ID, convId, now]);
  await run(db, `INSERT OR REPLACE INTO performance_metrics (conversation_id, timestamp, total_duration, total_cost, created_at) VALUES (?, ?, ?, ?, ?)`, [convId, now, 2500, 1.23, now]);

  const data = await computeAnalytics(db, USER_ID, '24h');
  assert.ok(data.todayQueries >= 1, 'todayQueries should be >= 1');
  assert.ok(data.todayCost >= 1.23, 'todayCost should include inserted cost');
  assert.ok(data.todayTokenUsage.total >= 150, 'todayTokenUsage total should include tokens');

  db.close();
});

test('analytics period selection 7d/30d compute without error', async () => {
  const dbPath = __dirname + '/../hive-ai.db';
  const db = new Database(dbPath);
  const d7 = await computeAnalytics(db, USER_ID, '7d');
  const d30 = await computeAnalytics(db, USER_ID, '30d');
  assert.ok(d7.todayQueries >= 0);
  assert.ok(d30.todayQueries >= d7.todayQueries);
  db.close();
});

function run(db: Database, sql: string, params: any[]): Promise<void> {
  return new Promise((resolve, reject) => {
    db.run(sql, params, (err) => (err ? reject(err) : resolve()));
  });
}

console.log('All analytics-compute tests scheduled');
