import type { Database } from 'sqlite3';
import crypto from 'crypto';

function sha1(text: string): string {
  return crypto.createHash('sha1').update(text).digest('hex');
}

function get<T = any>(db: Database, sql: string, params: any[] = []): Promise<T | undefined> {
  return new Promise((resolve, reject) => {
    db.get(sql, params, (err, row) => (err ? reject(err) : resolve(row as T)));
  });
}

function all<T = any>(db: Database, sql: string, params: any[] = []): Promise<T[]> {
  return new Promise((resolve, reject) => {
    db.all(sql, params, (err, rows) => (err ? reject(err) : resolve(rows as T[])));
  });
}

export async function resolveModelId(db: Database, modelRef: string): Promise<string> {
  const isHex40 = /^[a-f0-9]{40}$/i.test(modelRef);
  try {
    if (isHex40) {
      // Treat as internal_id: find an active alias first
      const alias = await get<{ openrouter_id: string }>(db,
        `SELECT openrouter_id FROM model_aliases WHERE internal_id = ? AND active = 1 LIMIT 1`,
        [modelRef]
      );
      if (alias?.openrouter_id) return alias.openrouter_id;
      // Fallback: active model with same internal_id
      const row = await get<{ openrouter_id: string }>(db,
        `SELECT openrouter_id FROM openrouter_models WHERE internal_id = ? AND is_active = 1 LIMIT 1`,
        [modelRef]
      );
      if (row?.openrouter_id) return row.openrouter_id;
      // Last resort: any alias
      const anyAlias = await get<{ openrouter_id: string }>(db,
        `SELECT openrouter_id FROM model_aliases WHERE internal_id = ? LIMIT 1`,
        [modelRef]
      );
      if (anyAlias?.openrouter_id) return anyAlias.openrouter_id;
      return modelRef;
    }

    // Treat as openrouter_id string
    const row = await get<{ is_active: number; internal_id: string }>(db,
      `SELECT is_active, internal_id FROM openrouter_models WHERE openrouter_id = ? LIMIT 1`,
      [modelRef]
    );
    if (row?.is_active === 1) return modelRef;
    // Inactive or unknown: try resolve via internal_id
    const internalId = row?.internal_id || sha1(modelRef);
    const alias = await get<{ openrouter_id: string }>(db,
      `SELECT openrouter_id FROM model_aliases WHERE internal_id = ? AND active = 1 LIMIT 1`,
      [internalId]
    );
    if (alias?.openrouter_id) return alias.openrouter_id;
    // Fallback: any active model with same internal id
    const active = await get<{ openrouter_id: string }>(db,
      `SELECT openrouter_id FROM openrouter_models WHERE internal_id = ? AND is_active = 1 LIMIT 1`,
      [internalId]
    );
    if (active?.openrouter_id) return active.openrouter_id;
    return modelRef;
  } catch {
    return modelRef;
  }
}

export async function resolveProfileStageModels(db: Database, profile: { generator_model: string; refiner_model: string; validator_model: string; curator_model: string }) {
  return {
    generator_model: await resolveModelId(db, profile.generator_model),
    refiner_model: await resolveModelId(db, profile.refiner_model),
    validator_model: await resolveModelId(db, profile.validator_model),
    curator_model: await resolveModelId(db, profile.curator_model)
  };
}

