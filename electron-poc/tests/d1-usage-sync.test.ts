import assert from 'assert';
import { Database } from 'sqlite3';
import { processD1UsageQueueOnce } from '../src/maintenance/D1UsageSync';

function createDb(): Database { const db = new Database(':memory:'); db.exec('PRAGMA foreign_keys=ON;'); return db; }

async function exec(db: Database, sql: string, params: any[] = []): Promise<void> { return new Promise((res, rej) => db.run(sql, params, (e) => e ? rej(e) : res())); }
async function get<T = any>(db: Database, sql: string, params: any[] = []): Promise<T | undefined> { return new Promise((res, rej) => db.get(sql, params, (e, r) => e ? rej(e) : res(r as T))); }

const okFetch = async () => ({ ok: true, text: async () => '' });
const badFetch = async () => ({ ok: false, text: async () => 'denied' });

function tokenProvider(token: string | null) { return async () => token; }

async function setup(db: Database) {
  await exec(db, `CREATE TABLE d1_usage_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    conversation_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    status TEXT DEFAULT 'pending',
    attempts INTEGER DEFAULT 0,
    last_error TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
  )`);
}

(async () => {
  const db = createDb(); await setup(db);
  await exec(db, `INSERT INTO d1_usage_queue (user_id, conversation_id, timestamp) VALUES ('u1','c1','2025-09-13T00:00:00Z')`);
  const res = await processD1UsageQueueOnce(db, tokenProvider('HIVE-XXXX-XXXX-XXXX'), okFetch as any);
  assert.strictEqual(res.sent, 1);
  const row: any = await get(db, `SELECT status, attempts FROM d1_usage_queue WHERE conversation_id='c1'`);
  assert.strictEqual(row.status, 'sent');
  assert.strictEqual(row.attempts, 1);
  console.log('✓ d1 queue success path');
})();

(async () => {
  const db = createDb(); await setup(db);
  await exec(db, `INSERT INTO d1_usage_queue (user_id, conversation_id, timestamp) VALUES ('u2','c2','2025-09-13T00:00:00Z')`);
  const res = await processD1UsageQueueOnce(db, tokenProvider('HIVE-XXXX-XXXX-XXXX'), badFetch as any);
  assert.strictEqual(res.failed, 1);
  const row: any = await get(db, `SELECT status, attempts, last_error FROM d1_usage_queue WHERE conversation_id='c2'`);
  assert.strictEqual(row.status, 'error');
  assert.strictEqual(row.attempts, 1);
  console.log('✓ d1 queue failure path');
})();

