import assert from 'assert';
import { Database } from 'sqlite3';
import { ensureOpenRouterSchema, syncOpenRouter } from '../src/maintenance/OpenRouterSync';

function test(name: string, fn: () => Promise<void> | void) {
  (async () => {
    try { await fn(); console.log(`✓ ${name}`); } catch (e) { console.error(`✗ ${name}`); throw e; }
  })();
}

function createTestDb(file = ':memory:'): Database {
  const db = new Database(file);
  db.exec('PRAGMA foreign_keys=ON;');
  db.exec('PRAGMA journal_mode=WAL;');
  db.exec('PRAGMA synchronous=NORMAL;');
  db.exec('PRAGMA busy_timeout=5000;');
  return db;
}

const sampleResponse = {
  data: [
    {
      id: 'openai/gpt-4-turbo',
      name: 'GPT-4 Turbo',
      description: 'OpenAI GPT-4 Turbo',
      context_length: 128000,
      pricing: { prompt: '0.01', completion: '0.02' },
      provider: { id: 'openai', name: 'OpenAI', website_url: 'https://openai.com' }
    },
    {
      id: 'anthropic/claude-3-opus',
      name: 'Claude 3 Opus',
      description: 'Anthropic Claude 3 Opus',
      context_length: 200000,
      pricing: { prompt: '0.015', completion: '0.03' },
      provider: { id: 'anthropic', name: 'Anthropic', website_url: 'https://anthropic.com' }
    }
  ]
};

const sampleResponseAfterRemoval = {
  data: [
    {
      id: 'openai/gpt-4-turbo',
      name: 'GPT-4 Turbo',
      description: 'OpenAI GPT-4 Turbo',
      context_length: 128000,
      pricing: { prompt: '0.01', completion: '0.02' },
      provider: { id: 'openai', name: 'OpenAI', website_url: 'https://openai.com' }
    }
  ]
};

const okFetch = (payload: any) => async () => ({ ok: true, json: async () => payload });

test('openrouter sync upserts providers and models', async () => {
  const db = createTestDb();
  await ensureOpenRouterSchema(db);
  const res = await syncOpenRouter(db, 'sk-or-test', okFetch(sampleResponse) as any);
  assert.ok(res.providersUpserted >= 2);
  assert.ok(res.modelsUpserted >= 2);

  await new Promise<void>((resolve, reject) => db.get(
    `SELECT COUNT(*) as c FROM openrouter_models WHERE is_active = 1`, [], (err, row: any) => {
      if (err) reject(err); else { try { assert.strictEqual(row.c, 2); resolve(); } catch (e) { reject(e); } }
    }
  ));
});

test('openrouter sync deactivates missing models', async () => {
  const db = createTestDb();
  await ensureOpenRouterSchema(db);
  await syncOpenRouter(db, 'sk-or-test', okFetch(sampleResponse) as any);
  const res2 = await syncOpenRouter(db, 'sk-or-test', okFetch(sampleResponseAfterRemoval) as any);
  assert.ok(res2.modelsDeactivated >= 1);

  // Verify deactivation
  await new Promise<void>((resolve, reject) => db.get(
    `SELECT COUNT(*) as c FROM openrouter_models WHERE is_active = 0`, [], (err, row: any) => {
      if (err) reject(err); else { try { assert.ok(row.c >= 1); resolve(); } catch (e) { reject(e); } }
    }
  ));
});

console.log('All openrouter-sync tests scheduled');

