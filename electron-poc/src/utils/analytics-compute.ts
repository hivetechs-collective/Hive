import { Database } from 'sqlite3';

export interface AnalyticsResult {
  todayQueries: number;
  totalQueries: number;
  todayCost: number;
  todayAvgResponseTime: number;
  todayTokenUsage: { total: number; input: number; output: number };
  totalCost: number;
  avgResponseTime: number;
  tokenUsage: { total: number; input: number; output: number };
  recentActivity: Array<{ timestamp: string; question?: string; cost: number; total_tokens_input?: number; total_tokens_output?: number; duration?: number; conversation_id: string }>;
  modelUsage: { [model: string]: number };
  costByModel: { [model: string]: number };
  hourlyStats: Array<{ hour: string; queries: number; cost: number; avgTime: number }>;
  alerts?: Array<{ type: 'warning' | 'error' | 'info'; message: string; timestamp: string }>;
}

export type AnalyticsPeriod = '24h' | '7d' | '30d';

export async function computeAnalytics(db: Database, userId: string | '*' = '*', period: AnalyticsPeriod = '24h'): Promise<AnalyticsResult> {
  return new Promise((resolve) => {
    const result: any = {};

    // Determine time window for the selected period
    const now = new Date();
    const windowClause = period === '24h'
      ? `timestamp >= datetime('now','-24 hours')`
      : period === '7d'
      ? `timestamp >= datetime('now','-7 days')`
      : `timestamp >= datetime('now','-30 days')`;

    // Queries in selected period
    const userFilter = userId === '*' ? '' : ' AND user_id = ? ';
    const userParams: any[] = userId === '*' ? [] : [userId];

    db.get(
      `SELECT COUNT(*) as count FROM conversation_usage WHERE ${windowClause} ${userFilter}`,
      userParams,
      (err1, row1: any) => {
        result.todayQueries = row1?.count || 0;

        // Total queries
        db.get(`SELECT COUNT(*) as count FROM conversation_usage ${userId === '*' ? '' : 'WHERE user_id = ?'}`, userParams, (errT, rowT: any) => {
          result.totalQueries = rowT?.count || 0;

          // Period cost/tokens + avg time
          db.get(
            `SELECT SUM(c.total_cost) as total_cost, SUM(c.total_tokens_input) as total_input, SUM(c.total_tokens_output) as total_output, AVG(pm.total_duration / 1000.0) as avg_time
             FROM conversations c
             INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
             LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
             WHERE ${windowClause} ${userFilter}`,
            userParams,
            (err2, row2: any) => {
              result.todayCost = row2?.total_cost || 0;
              result.todayAvgResponseTime = row2?.avg_time || 0;
              result.todayTokenUsage = {
                total: (row2?.total_input || 0) + (row2?.total_output || 0),
                input: row2?.total_input || 0,
                output: row2?.total_output || 0
              };

              // All-time totals
              db.get(
                `SELECT SUM(c.total_cost) as total_cost, SUM(c.total_tokens_input) as total_input, SUM(c.total_tokens_output) as total_output, AVG(pm.total_duration / 1000.0) as avg_time
                 FROM conversations c
                 INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
                 LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
                 ${userId === '*' ? '' : 'WHERE cu.user_id = ?'}`,
                userParams,
                (err3, row3: any) => {
                  result.totalCost = row3?.total_cost || 0;
                  result.avgResponseTime = row3?.avg_time || 0;
                  result.tokenUsage = {
                    total: (row3?.total_input || 0) + (row3?.total_output || 0),
                    input: row3?.total_input || 0,
                    output: row3?.total_output || 0
                  };

                  // Recent activity
                  db.all(
                    `SELECT c.id as conversation_id, kc.question, c.total_cost as cost, c.total_tokens_input, c.total_tokens_output, pm.total_duration as duration, cu.timestamp
                     FROM conversation_usage cu
                     INNER JOIN conversations c ON c.id = cu.conversation_id
                     LEFT JOIN knowledge_conversations kc ON c.id = kc.conversation_id
                     LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
                     ${userId === '*' ? '' : 'WHERE cu.user_id = ?'} ORDER BY cu.timestamp DESC LIMIT 10`,
                    userParams,
                    (err4, rows4: any[]) => {
                      result.recentActivity = rows4 || [];

                      // Model usage from stage_outputs fallback to profiles
                      db.all(
                        `SELECT so.model, COUNT(*) as count, SUM(so.cost) as totalCost
                         FROM stage_outputs so
                         INNER JOIN conversation_usage cu ON so.conversation_id = cu.conversation_id
                         ${userId === '*' ? '' : 'WHERE cu.user_id = ?'} GROUP BY so.model ORDER BY totalCost DESC`,
                        userParams,
                        (err5, rows5: any[]) => {
                          const modelUsage: any = {};
                          const modelCosts: any = {};
                          (rows5 || []).forEach((r: any) => {
                            const name = r.model?.split('/').pop() || r.model;
                            if (r.count > 0) {
                              modelUsage[name] = r.count;
                              modelCosts[name] = r.totalCost || 0;
                            }
                          });
                          result.modelUsage = modelUsage;
                          result.costByModel = modelCosts;

                          // Hourly stats last 24h
                          const hourly: any[] = [];
                  const now = new Date();
                  const next = (i: number) => {
                            if (i < 0) {
                              result.hourlyStats = hourly;
                              resolve(result as AnalyticsResult);
                              return;
                            }
                            const startISO = new Date(now.getTime() - (i + 1) * 3600_000).toISOString();
                            const endISO = new Date(now.getTime() - i * 3600_000).toISOString();
                            db.get(
                              `SELECT COUNT(DISTINCT cu.conversation_id) as queries, SUM(c.total_cost) as cost, AVG(pm.total_duration / 1000.0) as avg_time
                               FROM conversation_usage cu
                               LEFT JOIN conversations c ON c.id = cu.conversation_id
                               LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
                               WHERE datetime(cu.timestamp) >= datetime(?) AND datetime(cu.timestamp) < datetime(?) ${userFilter}`,
                              [startISO, endISO, ...userParams],
                              (err6, row6: any) => {
                                const hourLabel = new Date(startISO).getHours().toString().padStart(2, '0') + ':00';
                                hourly.push({ hour: hourLabel, queries: row6?.queries || 0, cost: row6?.cost || 0, avgTime: row6?.avg_time || 0 });
                                next(i - 1);
                              }
                            );
                          };
                          next(23);
                        }
                      );
                    }
                  );
                }
              );
            }
          );
        });
      }
    );
  });
}
