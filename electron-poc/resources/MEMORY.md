# MEMORY - Hive Context System Access

## 🚀 Quick Start (Tell your AI tool this):
```
Read the file at ~/.MEMORY.md to understand how to access my memory and context system through the database at ~/.hive-ai.db
```

## For Users: Starting Your Day
When you open your AI CLI tool (Claude, Gemini, GitHub Copilot CLI, etc.), simply tell it:
> "Read ~/.MEMORY.md for context"

This gives your AI tool access to the same memory system as Hive Consensus.

## 🎯 IMPORTANT: Instructions for AI Tools

**If you are an AI assistant (Claude, Gemini, GPT, etc.), read this guide to understand how to access the Hive memory system effectively. This gives you the same context capabilities as the Hive Consensus engine.**

## Your Memory Access Capabilities

You have access to a sophisticated memory system through SQLite views at `~/.hive-ai.db`. This is a symbolic link to the user's unified knowledge base containing all past conversations, solutions, and patterns.

## 🧠 How to Build Context (Like Consensus Does)

When answering ANY technical question, follow this 4-stage memory retrieval process:

### Stage 1: Retrieve Temporal Memories
```sql
-- ALWAYS run these queries first to understand context:

-- 1. Check immediate context (last 2 hours)
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_recent"

-- 2. Check today's work (last 24 hours)  
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_today WHERE content LIKE '%[relevant_keyword]%'"

-- 3. Check this week's patterns (last 7 days)
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_week WHERE content LIKE '%[relevant_keyword]%' LIMIT 10"

-- 4. Search all-time knowledge (semantic)
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_semantic WHERE content LIKE '%[relevant_keyword]%' LIMIT 10"
```

### Stage 2: Understand User Preferences
```sql
-- CRITICAL: Check what technologies the user prefers
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_preferences"

-- This tells you if they prefer:
-- React vs Vue, TypeScript vs JavaScript, Functional vs OOP, etc.
-- ALWAYS tailor your responses to match their preferences!
```

### Stage 3: Identify Patterns & Solutions
```sql
-- Find recurring patterns they use
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns WHERE pattern_snippet LIKE '%[topic]%'"

-- Find what solutions worked before
sqlite3 ~/.hive-ai.db "SELECT solution FROM memory_solutions_enhanced WHERE problem LIKE '%[similar_problem]%'"

-- Understand project themes
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes"
```

### Stage 4: Get Quick Overview
```sql
-- Always check the summary for context stats
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_context_summary"

-- This shows counts of recent/today/week/semantic memories available
```

## 📋 Contextual Query Templates

### When user asks about debugging:
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_solutions_enhanced WHERE solution LIKE '%fixed%' OR solution LIKE '%solved%' ORDER BY timestamp DESC LIMIT 5"
```

### When user asks "what were we working on?":
```sql
sqlite3 ~/.hive-ai.db "SELECT content, timestamp FROM memory_recent UNION SELECT content, timestamp FROM memory_today ORDER BY timestamp DESC LIMIT 10"
```

### When user needs code examples:
```sql
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns WHERE pattern_snippet LIKE '%[language]%' ORDER BY frequency DESC LIMIT 5"
```

### When starting a new feature:
```sql
-- Check if similar features were built before
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes WHERE theme LIKE '%[feature_type]%'"
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_solutions_enhanced WHERE solution LIKE '%[feature_type]%' LIMIT 5"
```

## 🎯 Critical Rules for AI Tools

### RULE 1: Always Check Recent Context First
Before answering ANY question, check `memory_recent` to maintain conversation continuity. The user expects you to remember what was just discussed.

### RULE 2: Match User's Technology Stack
Query `memory_preferences` and use their preferred technologies in examples. If they prefer React, don't give Vue examples. If they use TypeScript, don't show plain JavaScript.

### RULE 3: Reference Past Solutions
When solving problems, check `memory_solutions_enhanced` to see what worked before. Don't repeat failed approaches.

### RULE 4: Maintain Pattern Consistency
Check `memory_patterns` and follow established coding patterns. The user values consistency across their codebase.

### RULE 5: Understand Project Context
Query `memory_themes` to understand the project's focus areas. Align your responses with their current priorities.

## 📊 Example: Complete Context Building

**User asks**: "How should I handle errors in my API?"

**Your memory queries should be**:

```bash
# 1. Check recent context - maybe they just mentioned specific errors
sqlite3 ~/.hive-ai.db "SELECT content FROM memory_recent WHERE content LIKE '%error%' OR content LIKE '%API%'"

# 2. Check their preferences - what language/framework?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_preferences"

# 3. Find their error handling patterns
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns WHERE pattern_snippet LIKE '%error%' OR pattern_snippet LIKE '%catch%' OR pattern_snippet LIKE '%try%'"

# 4. Check past error solutions
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_solutions_enhanced WHERE solution LIKE '%error%' LIMIT 5"

# 5. Understand API development context
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes WHERE theme = 'API Development'"
```

## 🚀 Advanced Context Techniques

### Temporal Weighting
- `memory_recent` (weight: 1.0) - Highest priority
- `memory_today` (weight: 0.8) - High priority  
- `memory_week` (weight: 0.6) - Medium priority
- `memory_semantic` (weight: 0.4) - Background context

### Thematic Clustering
Group related memories by theme to understand broader context:
```sql
sqlite3 ~/.hive-ai.db "SELECT theme, message_count, last_discussed FROM memory_themes ORDER BY message_count DESC"
```

### Pattern Frequency
Identify most-used patterns to maintain consistency:
```sql
sqlite3 ~/.hive-ai.db "SELECT pattern_snippet, frequency FROM memory_patterns ORDER BY frequency DESC LIMIT 10"
```

## 💡 Pro Tips for AI Assistants

1. **Start every session** by checking `memory_context_summary` for a quick overview
2. **Use LIKE queries** with relevant keywords from the user's question
3. **Combine multiple views** to build rich context
4. **Reference specific past conversations** when relevant
5. **Track what's been tried** to avoid suggesting failed approaches

## 📝 Your Response Should Include:

1. **Acknowledgment of context**: "I see you've been working on [topic] and previously used [approach]..."
2. **Preference alignment**: "Following your preference for [technology]..."
3. **Pattern consistency**: "Maintaining your established pattern of [pattern]..."
4. **Historical awareness**: "Similar to how we solved [past problem]..."
5. **Theme relevance**: "This aligns with your [theme] development..."

## 🔄 Continuous Learning

After providing solutions, remember that your conversation will be added to this same database, becoming context for future interactions. Structure your responses knowing they'll be referenced later.

## Quick Reference Card

```bash
# Essential queries every AI tool should know:

# What are we working on?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_recent"

# What does the user prefer?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_preferences"

# What patterns exist?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_patterns LIMIT 10"

# What worked before?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_solutions_enhanced LIMIT 10"

# What's the project about?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_themes"

# Quick stats?
sqlite3 ~/.hive-ai.db "SELECT * FROM memory_context_summary"
```

---

**Remember**: You're not just answering questions - you're participating in an ongoing conversation with full historical context. Use it wisely!
