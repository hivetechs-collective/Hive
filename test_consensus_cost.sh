#!/bin/bash

# Count conversations before
BEFORE=$(sqlite3 ~/.hive/hive-ai.db "SELECT COUNT(*) FROM conversations;")
echo "Conversations before: $BEFORE"

# Run consensus with a short timeout
echo "Running consensus..."
timeout 60 ./target/debug/hive-consensus --prompt "What is 2+2?" 2>&1 | grep -E "Conversation ID|conversation_id|💰|Total cost" &

# Wait a bit for it to start
sleep 30

# Count conversations after
AFTER=$(sqlite3 ~/.hive/hive-ai.db "SELECT COUNT(*) FROM conversations;")
echo "Conversations after: $AFTER"

# Show the latest conversation
echo "Latest conversation:"
sqlite3 ~/.hive/hive-ai.db "SELECT id, total_cost, total_tokens_input, total_tokens_output, datetime(created_at) FROM conversations ORDER BY created_at DESC LIMIT 1;"