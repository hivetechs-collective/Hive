import type { Database } from 'sqlite3';

type FetchLike = typeof fetch;

function all<T = any>(db: Database, sql: string, params: any[] = []): Promise<T[]> {
  return new Promise((resolve, reject) => db.all(sql, params, (e, rows) => e ? reject(e) : resolve(rows as T[])));
}

function run(db: Database, sql: string, params: any[] = []): Promise<void> {
  return new Promise((resolve, reject) => db.run(sql, params, (e) => e ? reject(e) : resolve()));
}

export interface D1QueueRow {
  id: number;
  user_id: string;
  conversation_id: string;
  timestamp: string;
  status: string;
  attempts: number;
}

export async function processD1UsageQueueOnce(
  db: Database,
  getAuthToken: () => Promise<string | null>,
  fetchImpl: FetchLike = fetch as any,
  maxBatch = 10,
  maxAttempts = 5
): Promise<{ sent: number; failed: number } > {
  const rows = await all<D1QueueRow>(db,
    `SELECT id, user_id, conversation_id, timestamp, status, attempts
     FROM d1_usage_queue
     WHERE status IN ('pending','error') AND attempts < ?
     ORDER BY id ASC
     LIMIT ?`,
    [maxAttempts, maxBatch]
  );
  if (!rows.length) return { sent: 0, failed: 0 };

  const token = await getAuthToken();
  if (!token) {
    // No token -> mark as error but don't increment attempts to avoid burning retries
    for (const r of rows) {
      await run(db, `UPDATE d1_usage_queue SET status='error', last_error='no auth token' WHERE id=?`, [r.id]);
    }
    return { sent: 0, failed: rows.length };
  }

  let sent = 0; let failed = 0;
  for (const r of rows) {
    try {
      const resp = await fetchImpl('https://gateway.hivetechs.io/v1/usage/record', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
          'User-Agent': 'hive-electron/2.0.0'
        },
        body: JSON.stringify({ user_id: r.user_id, conversation_id: r.conversation_id, timestamp: r.timestamp })
      });
      if (resp.ok) {
        await run(db, `UPDATE d1_usage_queue SET status='sent', attempts=attempts+1, last_error=NULL WHERE id=?`, [r.id]);
        sent++;
      } else {
        const txt = await resp.text().catch(() => '');
        await run(db, `UPDATE d1_usage_queue SET status='error', attempts=attempts+1, last_error=? WHERE id=?`, [txt || String(resp.status), r.id]);
        failed++;
      }
    } catch (e: any) {
      await run(db, `UPDATE d1_usage_queue SET status='error', attempts=attempts+1, last_error=? WHERE id=?`, [e?.message || 'network', r.id]);
      failed++;
    }
  }
  return { sent, failed };
}

