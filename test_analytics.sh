#!/bin/bash

echo "🧪 Testing Hive Analytics Data Flow"
echo "==================================="

# Get current timestamp
NOW=$(date -u +"%Y-%m-%dT%H:%M:%S+00:00")
echo "Current UTC time: $NOW"

# Check today's conversations
echo -e "\n📊 Today's Conversations:"
sqlite3 ~/.hive/hive-ai.db "
SELECT COUNT(*) as count, 
       COALESCE(SUM(total_cost), 0) as total_cost,
       COUNT(CASE WHEN total_cost > 0 THEN 1 END) as paid_queries,
       COUNT(CASE WHEN total_cost = 0 THEN 1 END) as direct_queries
FROM conversations 
WHERE date(created_at) = date('now');"

echo -e "\n📋 Recent Conversations (last 5):"
sqlite3 ~/.hive/hive-ai.db "
SELECT substr(id, 1, 8) as id_start, 
       created_at,
       printf('%.6f', total_cost) as cost,
       substr(title, 1, 50) as title
FROM conversations 
ORDER BY created_at DESC 
LIMIT 5;"

echo -e "\n💰 Cost Tracking by Stage (recent pipeline run):"
sqlite3 ~/.hive/hive-ai.db "
SELECT ct.conversation_id,
       om.name as model,
       printf('%.6f', ct.total_cost) as cost,
       ct.created_at
FROM cost_tracking ct
JOIN openrouter_models om ON ct.model_id = om.internal_id
WHERE ct.conversation_id = (
    SELECT id FROM conversations 
    WHERE total_cost > 0 
    ORDER BY created_at DESC 
    LIMIT 1
)
ORDER BY ct.created_at;"

echo -e "\n🔍 Analytics Query Test (matching Rust code):"
sqlite3 ~/.hive/hive-ai.db "
WITH metrics AS (
    SELECT 
        COUNT(*) as total_queries,
        printf('%.6f', COALESCE(SUM(total_cost), 0.0)) as total_cost,
        COUNT(CASE WHEN created_at >= date('now', 'start of day') || 'T00:00:00+00:00' THEN 1 END) as today_queries,
        printf('%.6f', COALESCE(SUM(CASE WHEN created_at >= date('now', 'start of day') || 'T00:00:00+00:00' THEN total_cost END), 0.0)) as today_cost
    FROM conversations
)
SELECT * FROM metrics;"

echo -e "\n✅ Analytics test complete!"