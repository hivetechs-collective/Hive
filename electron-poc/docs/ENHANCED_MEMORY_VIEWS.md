# Enhanced Memory Views for AI CLI Tools

## Overview
These enhanced views replicate the Hive Consensus engine's sophisticated Memory-Context pipeline, giving CLI tools access to the same intelligent context retrieval capabilities.

## 🧠 Core Memory Layers (Temporal Retrieval)

### 1. `memory_recent` - Conversation Continuity (Last 2 Hours)
**Purpose**: Maintain conversation context and immediate continuity
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_recent"
```
- **Recency Score**: 4 (highest priority)
- **Use Case**: Continue ongoing discussions, reference just-mentioned topics
- **Example Query**: "What were we just discussing about authentication?"

### 2. `memory_today` - Recent Context (Last 24 Hours)
**Purpose**: Provide recent working context and daily patterns
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_today"
```
- **Recency Score**: 3
- **Use Case**: Reference today's work, recent problems and solutions
- **Example Query**: "What errors did we fix today?"

### 3. `memory_week` - Pattern Recognition (Last 7 Days)
**Purpose**: Identify patterns and recurring themes
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_week"
```
- **Recency Score**: 2
- **Use Case**: Understand weekly development patterns, common issues
- **Example Query**: "What patterns have emerged this week?"

### 4. `memory_semantic` - Thematic Relevance (All Time)
**Purpose**: Deep knowledge base for semantic matching
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_semantic"
```
- **Recency Score**: 1
- **Use Case**: Find similar past solutions, historical context
- **Example Query**: "How have we handled authentication before?"

## 🎯 Context Building Views

### 5. `memory_patterns` - Coding Pattern Recognition
**Purpose**: Identify recurring solutions and approaches
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns"
```
- Shows frequently used code patterns
- Tracks when patterns were first seen and last used
- Helps maintain consistency in solutions

### 6. `memory_preferences` - User Technology Preferences
**Purpose**: Understand user's preferred technologies and approaches
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_preferences"
```
- Tracks language preferences (React, TypeScript, Python, etc.)
- Shows usage frequency and last mention
- Helps tailor responses to user's stack

### 7. `memory_themes` - Thematic Clustering
**Purpose**: Group conversations by topic for context
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes"
```
- Categories: Authentication, Database, API, React, Testing, etc.
- Shows message count per theme
- Tracks first and last discussion dates

### 8. `memory_solutions_enhanced` - Problem-Solution Pairs
**Purpose**: Track what solutions worked for specific problems
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_solutions_enhanced LIMIT 5"
```
- Links user problems to assistant solutions
- Focuses on successful resolutions
- Helps avoid repeating failed approaches

## 📊 Master Context Views

### 9. `memory_context_full` - Complete Layered Memory
**Purpose**: Combined view of all memory layers with scoring
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_context_full WHERE content LIKE '%[search_term]%' LIMIT 20"
```
- Combines all 4 temporal layers
- Includes recency scores and relevance weights
- Provides comprehensive context

### 10. `memory_context_summary` - Quick Overview
**Purpose**: High-level statistics for context assessment
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_context_summary"
```
Returns:
- Count of messages per memory layer
- Total patterns and preferences
- Last activity timestamp

## 🚀 Optimal Query Strategy for CLI Tools

### Step 1: Assess Context Needs
```sql
-- Quick overview of available context
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_context_summary"
```

### Step 2: Layer-Based Retrieval
```sql
-- For continuing conversation
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_recent WHERE content LIKE '%topic%'"

-- For today's context
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_today WHERE content LIKE '%error%'"

-- For patterns
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns WHERE pattern_snippet LIKE '%async%'"
```

### Step 3: Build Rich Context
```sql
-- Get user preferences
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_preferences"

-- Get relevant themes
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes WHERE theme IN ('Authentication', 'Database')"

-- Get successful solutions
sqlite3 ~/.hive-ai.db "SELECT solution FROM memory_solutions_enhanced WHERE problem LIKE '%similar_issue%'"
```

## 💡 Example: Full Context Retrieval Flow

When a user asks: "How should I implement authentication?"

1. **Check recent context** (maybe they just mentioned specific requirements)
```sql
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_recent WHERE content LIKE '%auth%'"
```

2. **Check user preferences** (do they prefer JWT, OAuth, etc?)
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_preferences WHERE preference LIKE '%auth%'"
```

3. **Find past solutions** (what worked before?)
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_solutions_enhanced WHERE solution LIKE '%auth%' LIMIT 5"
```

4. **Identify patterns** (common authentication patterns used)
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns WHERE pattern_snippet LIKE '%auth%' OR pattern_snippet LIKE '%jwt%'"
```

5. **Get thematic context** (how much auth work has been done?)
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes WHERE theme = 'Authentication'"
```

## 🎯 Best Practices for AI CLI Tools

1. **Always check `memory_recent` first** - Maintains conversation continuity
2. **Use `memory_preferences` to tailor responses** - Match user's tech stack
3. **Reference `memory_patterns` for consistency** - Use established patterns
4. **Query `memory_solutions_enhanced` to avoid past failures** - Learn from history
5. **Use `memory_themes` to understand project focus** - Align with user's priorities

## 📈 Performance Considerations

- Views are optimized with LIMIT clauses to prevent large result sets
- Recency scores enable weighted relevance ranking
- Temporal layering reduces query scope progressively
- Pattern matching uses indexed columns for speed

## 🔄 Replicating Consensus Context Building

To replicate the full Consensus experience:

1. **Memory Retrieval** (Parallel queries to all 4 layers)
2. **Pattern Extraction** (Query memory_patterns)
3. **Preference Detection** (Query memory_preferences)
4. **Thematic Clustering** (Query memory_themes)
5. **Solution Matching** (Query memory_solutions_enhanced)
6. **Context Synthesis** (Combine all results with weights)

This gives CLI tools ~80% of Consensus's context awareness using only SQL views!