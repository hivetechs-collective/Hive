-- ============================================
-- Memory-Context System Verification Queries
-- CORRECT DATABASE: ~/.hive/hive-ai.db
-- ============================================

-- 1. CHECK IF USER MESSAGES ARE BEING STORED
-- Should show your PowerShell questions
SELECT 
    id,
    conversation_id,
    role,
    substr(content, 1, 100) as content_preview,
    timestamp,
    datetime(timestamp, 'localtime') as local_time
FROM messages 
WHERE role = 'user'
  AND date(timestamp) = date('now')
ORDER BY timestamp DESC
LIMIT 5;

-- 2. CHECK IF ASSISTANT RESPONSES ARE BEING STORED  
SELECT 
    id,
    conversation_id,
    role,
    substr(content, 1, 100) as content_preview,
    consensus_path,
    consensus_rounds,
    model_used,
    tokens_used,
    cost,
    timestamp
FROM messages 
WHERE role = 'assistant'
  AND date(timestamp) = date('now')
ORDER BY timestamp DESC
LIMIT 5;

-- 3. CHECK MEMORY CONTEXT LOGS
SELECT 
    log_id,
    conversation_id,
    memories_retrieved_recent,
    memories_retrieved_today,
    memories_retrieved_week,
    memories_retrieved_semantic,
    (memories_retrieved_recent + memories_retrieved_today + 
     memories_retrieved_week + memories_retrieved_semantic) as total_memories,
    routing_decision,
    memory_stage_duration_ms,
    context_stage_duration_ms,
    datetime(timestamp, 'localtime') as local_time
FROM memory_context_logs
ORDER BY timestamp DESC
LIMIT 5;

-- 4. CHECK TODAY'S MESSAGES
SELECT 
    'Total Messages Today' as metric,
    COUNT(*) as count
FROM messages 
WHERE date(timestamp) = date('now')
UNION ALL
SELECT 
    'User Messages Today' as metric,
    COUNT(*) as count
FROM messages 
WHERE date(timestamp) = date('now')
  AND role = 'user'
UNION ALL
SELECT 
    'Assistant Messages Today' as metric,
    COUNT(*) as count
FROM messages 
WHERE date(timestamp) = date('now')
  AND role = 'assistant';

-- 5. CHECK CONVERSATION CONTINUITY
SELECT 
    conversation_id,
    COUNT(*) as message_count,
    MIN(timestamp) as conversation_start,
    MAX(timestamp) as last_message
FROM messages
WHERE date(timestamp) = date('now')
GROUP BY conversation_id
ORDER BY MAX(timestamp) DESC;

-- 6. VERIFY MEMORY SERVICE IS STORING DATA
SELECT 
    id,
    substr(content, 1, 80) as content_preview,
    timestamp,
    CASE 
        WHEN datetime(timestamp) > datetime('now', '-2 hours') THEN 'RECENT (should be retrieved)'
        WHEN datetime(timestamp) > datetime('now', '-24 hours') THEN 'TODAY (should be retrieved)'
        WHEN datetime(timestamp) > datetime('now', '-7 days') THEN 'WEEK (should be retrieved)'
        ELSE 'SEMANTIC (keyword match only)'
    END as memory_layer
FROM messages
WHERE role IN ('user', 'assistant')
ORDER BY timestamp DESC
LIMIT 10;