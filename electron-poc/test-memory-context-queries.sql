-- ============================================
-- Memory-Context System Verification Queries
-- ============================================
-- Run these queries after testing to verify the complete loop is working

-- 1. CHECK IF USER MESSAGES ARE BEING STORED
-- Should show user questions with timestamps
SELECT 
    id,
    conversation_id,
    role,
    substr(content, 1, 100) as content_preview,
    timestamp,
    datetime(timestamp, 'localtime') as local_time
FROM messages 
WHERE role = 'user'
ORDER BY timestamp DESC
LIMIT 5;

-- 2. CHECK IF ASSISTANT RESPONSES ARE BEING STORED  
-- Should show AI responses with consensus path (SIMPLE/COMPLEX)
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
ORDER BY timestamp DESC
LIMIT 5;

-- 3. CHECK PARENT-CHILD MESSAGE RELATIONSHIPS
-- Should show user questions linked to assistant responses
SELECT 
    u.id as user_msg_id,
    substr(u.content, 1, 50) as user_question,
    a.id as assistant_msg_id,
    substr(a.content, 1, 50) as assistant_response,
    a.parent_message_id,
    a.consensus_path
FROM messages u
LEFT JOIN messages a ON a.parent_message_id = u.id
WHERE u.role = 'user'
ORDER BY u.timestamp DESC
LIMIT 5;

-- 4. CHECK MEMORY CONTEXT LOGS
-- Should show memory retrieval operations with layer counts
SELECT 
    log_id,
    request_id,
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
    datetime(created_at, 'localtime') as local_time
FROM memory_context_logs
ORDER BY created_at DESC
LIMIT 5;

-- 5. CHECK CONTEXT EXTRACTION
-- Should show patterns and topics identified
SELECT 
    log_id,
    substr(context_summary, 1, 100) as summary_preview,
    patterns_identified,
    topics_extracted,
    routing_decision
FROM memory_context_logs
WHERE context_summary IS NOT NULL
ORDER BY created_at DESC
LIMIT 5;

-- 6. VERIFY MEMORY SERVICE STATISTICS
-- Should match what Memory Service dashboard shows
SELECT 
    'Total Memories' as metric,
    COUNT(*) as value
FROM messages
WHERE role IN ('user', 'assistant')
UNION ALL
SELECT 
    'Messages Today' as metric,
    COUNT(*) as value
FROM messages 
WHERE date(timestamp) = date('now')
AND role IN ('user', 'assistant')
UNION ALL
SELECT 
    'User Messages Today' as metric,
    COUNT(*) as value
FROM messages 
WHERE date(timestamp) = date('now')
AND role = 'user'
UNION ALL
SELECT 
    'Assistant Messages Today' as metric,
    COUNT(*) as value
FROM messages 
WHERE date(timestamp) = date('now')
AND role = 'assistant';

-- 7. CHECK MEMORY LAYER DISTRIBUTION
-- Shows how many memories retrieved from each time layer
SELECT 
    'Recent (2h)' as layer,
    AVG(memories_retrieved_recent) as avg_retrieved,
    MAX(memories_retrieved_recent) as max_retrieved
FROM memory_context_logs
WHERE created_at > datetime('now', '-1 hour')
UNION ALL
SELECT 
    'Today (24h)' as layer,
    AVG(memories_retrieved_today) as avg_retrieved,
    MAX(memories_retrieved_today) as max_retrieved
FROM memory_context_logs
WHERE created_at > datetime('now', '-1 hour')
UNION ALL
SELECT 
    'Week (7d)' as layer,
    AVG(memories_retrieved_week) as avg_retrieved,
    MAX(memories_retrieved_week) as max_retrieved
FROM memory_context_logs
WHERE created_at > datetime('now', '-1 hour')
UNION ALL
SELECT 
    'Semantic' as layer,
    AVG(memories_retrieved_semantic) as avg_retrieved,
    MAX(memories_retrieved_semantic) as max_retrieved
FROM memory_context_logs
WHERE created_at > datetime('now', '-1 hour');

-- 8. CHECK ROUTING DECISIONS
-- Should show mix of SIMPLE and COMPLEX based on questions
SELECT 
    routing_decision,
    COUNT(*) as count,
    ROUND(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM memory_context_logs), 2) as percentage
FROM memory_context_logs
WHERE routing_decision IS NOT NULL
GROUP BY routing_decision;

-- 9. PERFORMANCE METRICS
-- Check memory and context stage performance
SELECT 
    'Memory Retrieval' as operation,
    MIN(memory_stage_duration_ms) as min_ms,
    AVG(memory_stage_duration_ms) as avg_ms,
    MAX(memory_stage_duration_ms) as max_ms
FROM memory_context_logs
WHERE memory_stage_duration_ms > 0
UNION ALL
SELECT 
    'Context Building' as operation,
    MIN(context_stage_duration_ms) as min_ms,
    AVG(context_stage_duration_ms) as avg_ms,
    MAX(context_stage_duration_ms) as max_ms
FROM memory_context_logs
WHERE context_stage_duration_ms > 0;

-- 10. CHECK CONVERSATION CONTINUITY
-- Verify conversations are properly linked
SELECT 
    conversation_id,
    COUNT(*) as message_count,
    MIN(timestamp) as conversation_start,
    MAX(timestamp) as last_message,
    ROUND((julianday(MAX(timestamp)) - julianday(MIN(timestamp))) * 24 * 60, 2) as duration_minutes
FROM messages
GROUP BY conversation_id
ORDER BY MAX(timestamp) DESC
LIMIT 5;

-- 11. VERIFY COST TRACKING
-- Should show costs accumulating from consensus operations
SELECT 
    DATE(timestamp) as date,
    COUNT(CASE WHEN role = 'user' THEN 1 END) as user_messages,
    COUNT(CASE WHEN role = 'assistant' THEN 1 END) as assistant_messages,
    SUM(CASE WHEN role = 'assistant' THEN tokens_used ELSE 0 END) as total_tokens,
    ROUND(SUM(CASE WHEN role = 'assistant' THEN cost ELSE 0 END), 4) as total_cost
FROM messages
WHERE timestamp > datetime('now', '-7 days')
GROUP BY DATE(timestamp)
ORDER BY date DESC;

-- 12. CHECK IF MEMORIES ARE BEING RETRIEVED
-- Look for recent memories that would be retrieved for context
SELECT 
    id,
    substr(content, 1, 80) as content_preview,
    timestamp,
    CASE 
        WHEN datetime(timestamp) > datetime('now', '-2 hours') THEN 'RECENT'
        WHEN datetime(timestamp) > datetime('now', '-24 hours') THEN 'TODAY'
        WHEN datetime(timestamp) > datetime('now', '-7 days') THEN 'WEEK'
        ELSE 'SEMANTIC'
    END as memory_layer
FROM messages
WHERE role IN ('user', 'assistant')
    AND datetime(timestamp) > datetime('now', '-2 hours')
ORDER BY timestamp DESC
LIMIT 10;

-- 13. VERIFY ANALYTICS INTEGRATION
-- Should match Analytics Dashboard display
SELECT 
    'Conversations Table' as source,
    COUNT(*) as total_conversations,
    SUM(total_cost) as total_cost,
    AVG(total_tokens_input + total_tokens_output) as avg_tokens
FROM conversations
WHERE created_at > datetime('now', '-7 days');

-- 14. CHECK CONVERSATION_USAGE TABLE (Analytics Source)
-- This drives the Analytics Dashboard metrics
SELECT 
    DATE(timestamp) as date,
    COUNT(*) as queries_count,
    COUNT(DISTINCT conversation_id) as unique_conversations,
    COUNT(DISTINCT user_id) as unique_users
FROM conversation_usage
WHERE timestamp > datetime('now', '-7 days')
GROUP BY DATE(timestamp)
ORDER BY date DESC;

-- 15. VERIFY HOURLY ANALYTICS STATS
-- Should show activity distribution for Analytics charts
SELECT 
    strftime('%H:00', timestamp) as hour,
    COUNT(*) as queries,
    COUNT(DISTINCT conversation_id) as conversations
FROM conversation_usage
WHERE date(timestamp) = date('now')
GROUP BY strftime('%H', timestamp)
ORDER BY hour DESC;

-- 16. CHECK MODEL USAGE FOR ANALYTICS
-- Should populate model usage charts
SELECT 
    CASE 
        WHEN model_used LIKE '%/%' THEN substr(model_used, instr(model_used, '/') + 1)
        ELSE model_used
    END as model_name,
    COUNT(*) as usage_count,
    SUM(tokens_used) as total_tokens,
    ROUND(SUM(cost), 4) as total_cost
FROM messages
WHERE role = 'assistant' 
    AND model_used IS NOT NULL
    AND timestamp > datetime('now', '-7 days')
GROUP BY model_name
ORDER BY usage_count DESC;

-- 17. VERIFY REAL-TIME ANALYTICS METRICS
-- These should auto-update every 5 seconds in Analytics Dashboard
SELECT 
    'Today Queries' as metric,
    COUNT(*) as value
FROM conversation_usage
WHERE date(timestamp, 'localtime') = date('now', 'localtime')
UNION ALL
SELECT 
    'Today Cost' as metric,
    ROUND(SUM(c.total_cost), 4) as value
FROM conversations c
INNER JOIN conversation_usage cu ON c.id = cu.conversation_id
WHERE date(cu.timestamp, 'localtime') = date('now', 'localtime')
UNION ALL
SELECT 
    'All-Time Queries' as metric,
    COUNT(*) as value
FROM conversation_usage
UNION ALL
SELECT 
    'All-Time Cost' as metric,
    ROUND(SUM(total_cost), 4) as value
FROM conversations;

-- 18. CHECK RECENT ACTIVITY FOR ANALYTICS
-- Should populate Recent Queries table in dashboard
SELECT 
    cu.timestamp,
    substr(kc.question, 1, 60) as question_preview,
    c.total_cost as cost,
    (c.total_tokens_input + c.total_tokens_output) as tokens,
    pm.total_duration / 1000.0 as duration_seconds
FROM conversation_usage cu
INNER JOIN conversations c ON c.id = cu.conversation_id
LEFT JOIN knowledge_conversations kc ON c.id = kc.conversation_id
LEFT JOIN performance_metrics pm ON c.id = pm.conversation_id
ORDER BY cu.timestamp DESC
LIMIT 10;

-- 19. VERIFY MEMORY SERVICE CONTRIBUTION TO ANALYTICS
-- Memory Service stats should align with Analytics
SELECT 
    'Memory Service View' as source,
    (SELECT COUNT(*) FROM messages WHERE role IN ('user', 'assistant')) as total_memories,
    (SELECT COUNT(*) FROM messages WHERE date(timestamp) = date('now') AND role = 'user') as queries_today,
    (SELECT COUNT(*) FROM messages WHERE date(timestamp) = date('now') AND role = 'assistant') as contributions_today,
    '---vs---' as separator,
    'Analytics View' as source2,
    (SELECT COUNT(*) FROM conversation_usage WHERE date(timestamp, 'localtime') = date('now', 'localtime')) as analytics_queries_today;

-- 20. CHECK COST_ANALYTICS TABLE
-- Should have entries for cost tracking
SELECT 
    conversation_id,
    total_cost,
    cost_per_token,
    optimization_potential,
    datetime(created_at, 'localtime') as created_time
FROM cost_analytics
ORDER BY created_at DESC
LIMIT 5;

-- ============================================
-- EXPECTED RESULTS AFTER TESTING:
-- ============================================
-- 1. User messages should appear immediately after asking questions
-- 2. Assistant messages should appear with consensus_path filled
-- 3. Parent-child relationships should link Q&A pairs
-- 4. Memory context logs should show retrieval counts
-- 5. Context summary and patterns should be extracted
-- 6. Statistics should match Memory Service dashboard
-- 7. Memory layers should show appropriate distribution
-- 8. Routing decisions should reflect question complexity
-- 9. Performance should be <50ms for memory, <20ms for context
-- 10. Conversations should maintain continuity
-- 11. Costs should accumulate per response
-- 12. Recent messages should be available for retrieval