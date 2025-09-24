import type { Database } from 'sqlite3';
import crypto from 'crypto';

export interface OpenRouterProvider {
  id: string;
  name?: string;
  website_url?: string;
}

export interface OpenRouterModel {
  id: string; // provider/model-id like "openai/gpt-4"
  name?: string;
  description?: string;
  context_length?: number;
  pricing?: { prompt?: string | number; completion?: string | number };
  provider?: OpenRouterProvider;
}

export interface OpenRouterModelsResponse {
  data?: OpenRouterModel[];
}

function sha1(text: string): string {
  return crypto.createHash('sha1').update(text).digest('hex');
}

function nowIso(): string {
  return new Date().toISOString();
}

function run(db: Database, sql: string, params: any[] = []): Promise<void> {
  return new Promise((resolve, reject) => {
    db.run(sql, params, (err) => (err ? reject(err) : resolve()));
  });
}

function all<T = any>(db: Database, sql: string, params: any[] = []): Promise<T[]> {
  return new Promise((resolve, reject) => {
    db.all(sql, params, (err, rows) => (err ? reject(err) : resolve(rows as T[])));
  });
}

function get<T = any>(db: Database, sql: string, params: any[] = []): Promise<T | undefined> {
  return new Promise((resolve, reject) => {
    db.get(sql, params, (err, row) => (err ? reject(err) : resolve(row as T)));
  });
}

export async function ensureOpenRouterSchema(db: Database): Promise<void> {
  await run(db, `CREATE TABLE IF NOT EXISTS openrouter_providers (
    provider_id TEXT PRIMARY KEY,
    name TEXT,
    website TEXT,
    last_seen_at TEXT,
    is_active INTEGER DEFAULT 1
  )`);

  await run(db, `CREATE TABLE IF NOT EXISTS openrouter_models (
    internal_id TEXT PRIMARY KEY,
    openrouter_id TEXT UNIQUE,
    provider_id TEXT,
    name TEXT,
    description TEXT,
    context_window INTEGER,
    pricing_input REAL,
    pricing_output REAL,
    is_active INTEGER DEFAULT 1,
    last_seen_at TEXT,
    replaced_by TEXT,
    FOREIGN KEY (provider_id) REFERENCES openrouter_providers(provider_id)
  )`);

  await run(db, `CREATE TABLE IF NOT EXISTS model_aliases (
    internal_id TEXT,
    openrouter_id TEXT,
    active INTEGER DEFAULT 1,
    PRIMARY KEY (internal_id, openrouter_id),
    FOREIGN KEY (internal_id) REFERENCES openrouter_models(internal_id)
  )`);
}

function parseNumber(x: any): number | null {
  if (x === null || x === undefined) return null;
  const n = typeof x === 'string' ? parseFloat(x) : Number(x);
  return Number.isFinite(n) ? n : null;
}

export async function syncOpenRouter(
  db: Database,
  apiKey: string,
  fetchImpl: typeof fetch = fetch as any
): Promise<{ providersUpserted: number; modelsUpserted: number; modelsDeactivated: number }> {
  await ensureOpenRouterSchema(db);

  const resp = await fetchImpl('https://openrouter.ai/api/v1/models', {
    headers: {
      Authorization: `Bearer ${apiKey}`,
      'HTTP-Referer': 'https://hivetechs.io',
      'X-Title': 'hive-ai'
    }
  });
  if (!resp.ok) throw new Error(`OpenRouter models fetch failed: ${resp.status}`);
  const json = (await resp.json()) as OpenRouterModelsResponse;
  const models = (json?.data || []).filter(m => m && m.id);
  const seenIds = new Set<string>();
  const ts = nowIso();

  // Upsert providers and models
  let providersUpserted = 0;
  let modelsUpserted = 0;

  for (const m of models) {
    const providerId = m.provider?.id || (m.id.includes('/') ? m.id.split('/')[0] : 'unknown');
    const providerName = m.provider?.name || providerId;
    const providerSite = m.provider?.website_url || '';

    await run(db, `INSERT INTO openrouter_providers (provider_id, name, website, last_seen_at, is_active)
                   VALUES (?, ?, ?, ?, 1)
                   ON CONFLICT(provider_id) DO UPDATE SET
                     name=excluded.name,
                     website=excluded.website,
                     last_seen_at=excluded.last_seen_at,
                     is_active=1`, [providerId, providerName, providerSite, ts]);
    providersUpserted++;

    // Determine internal_id: stable hash of openrouter_id
    const openrouterId = m.id;
    const internalId = sha1(openrouterId);
    const name = m.name || openrouterId.split('/').pop() || openrouterId;
    const desc = m.description || '';
    const ctx = m.context_length || null;
    const priceIn = parseNumber(m.pricing?.prompt) ?? null;
    const priceOut = parseNumber(m.pricing?.completion) ?? null;

    await run(db, `INSERT INTO openrouter_models (
        internal_id, openrouter_id, provider_id, name, description,
        context_window, pricing_input, pricing_output, is_active, last_seen_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, ?)
      ON CONFLICT(openrouter_id) DO UPDATE SET
        provider_id=excluded.provider_id,
        name=excluded.name,
        description=excluded.description,
        context_window=excluded.context_window,
        pricing_input=excluded.pricing_input,
        pricing_output=excluded.pricing_output,
        is_active=1,
        last_seen_at=excluded.last_seen_at`,
      [internalId, openrouterId, providerId, name, desc, ctx, priceIn, priceOut, ts]
    );
    modelsUpserted++;

    // Ensure alias exists and active
    await run(db, `INSERT INTO model_aliases (internal_id, openrouter_id, active)
                   VALUES (?, ?, 1)
                   ON CONFLICT(internal_id, openrouter_id) DO UPDATE SET active=1`,
      [internalId, openrouterId]
    );

    seenIds.add(openrouterId);
  }

  // Deactivate models not seen in this fetch
  const allActive = await all<{ openrouter_id: string }>(db,
    `SELECT openrouter_id FROM openrouter_models WHERE is_active = 1`
  );
  let modelsDeactivated = 0;
  for (const row of allActive) {
    if (!seenIds.has(row.openrouter_id)) {
      await run(db, `UPDATE openrouter_models SET is_active = 0 WHERE openrouter_id = ?`, [row.openrouter_id]);
      modelsDeactivated++;
    }
  }

  return { providersUpserted, modelsUpserted, modelsDeactivated };
}

