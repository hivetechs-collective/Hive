# Consensus Memory Implementation Analysis

## Overview

The Hive AI Rust implementation features a sophisticated memory system that integrates deeply with the 4-stage consensus pipeline (Generator → Refiner → Validator → Curator). This system provides **multi-conversation intelligence** by storing and retrieving authoritative knowledge from previous curator results.

## Key Components

### 1. Memory Context Retrieval (Pipeline Integration)

**Location**: `src/consensus/pipeline.rs` (lines 1370-1635)

The memory context retrieval happens at the beginning of each consensus run:

```rust
// Get memory context (recent + thematic) - CRITICAL for multi-conversation intelligence
let memory_context = self.get_memory_context(question).await?;
```

The system uses a **two-tier priority system**:

1. **Temporal Priority (24 hours)**: Recent conversations searched first, newest to oldest
2. **Thematic Search**: All-time memory searched with keyword matching

### 2. Database Storage Structure

**Location**: `migrations/002_knowledge_tables.sql`

The memory system uses several key tables:

#### `knowledge_conversations`
- Stores the complete conversation with question, answer, and source of truth
- The `source_of_truth` field contains the final curator output (authoritative answer)
- Links to consensus profiles for context

#### `curator_truths`
- Stores curator outputs with confidence scores
- Topic summaries for thematic clustering
- One entry per conversation

#### `conversation_context`
- Tracks relationships between conversations
- Three types: `recent_24h`, `thematic`, `direct_reference`
- Relevance scoring for retrieval optimization

### 3. Memory Context Building Algorithm

**Location**: `src/consensus/pipeline.rs::build_memory_context`

The algorithm follows this priority order:

1. **Follow-up Detection**: Identifies if the query is a follow-up using patterns like "tell me more", "examples", etc.
2. **Recent Memory Search (24h)**:
   - For follow-ups: Most recent conversation is always relevant
   - For new queries: Searches for keyword matches in recent conversations
3. **Broader Window (72h)**: If follow-up but no 24h context found
4. **Thematic Search**: Searches all memory with no time limits, ordered by recency

### 4. Generator Stage Memory Integration

**Location**: `src/consensus/stages/generator.rs`

The Generator stage receives memory context and treats it as **authoritative**:

```rust
if context.contains("## Memory Context") {
    structured.push_str("🧠 AUTHORITATIVE MEMORY CONTEXT:\n");
    structured.push_str("⚡ IMPORTANT: The above memory context contains AUTHORITATIVE ANSWERS...");
}
```

Key instructions for the Generator:
- Prioritize memory context over general knowledge
- Build upon previous answers rather than starting from scratch
- Reference specific details from memory context

### 5. Curator Stage Storage

**Location**: `src/consensus/pipeline.rs::store_consensus_result`

After consensus completes, the Curator result is stored as the new authoritative knowledge:

1. **Conversation Record**: Basic conversation data in `conversations` table
2. **Stage Results**: All 4 stage outputs stored for analysis
3. **Knowledge Conversation**: Extended format with source of truth
4. **Curator Truth**: Flagged as authoritative with confidence score

### 6. Enhanced Memory System (Phase 5.1)

**Location**: `src/core/memory.rs` and `src/memory/`

Additional advanced features:

#### Vector Embeddings
- 768-dimensional embeddings for semantic search
- Cosine similarity for finding related memories
- Caching for performance

#### Knowledge Graph
- Entities: Concepts, Technologies, Patterns, Solutions, etc.
- Relationships: RelatedTo, DependsOn, Implements, etc.
- Graph traversal for discovering connections

#### Pattern Learning
- Identifies recurring patterns in Q&A
- Learns from conversation history
- Provides pattern-based suggestions

## Memory Flow in Consensus

1. **Query Received**: User asks a question
2. **Memory Retrieval**: System searches for relevant past curator results
3. **Context Building**: Combines memory, temporal, and repository context
4. **Generator Stage**: Uses memory as authoritative foundation
5. **Refiner/Validator**: Build upon memory-informed generator output
6. **Curator Stage**: Creates final authoritative answer
7. **Storage**: Curator result becomes new memory for future queries

## Key Features

### Temporal Awareness
- Recent conversations (24h) have highest priority
- Follow-up questions automatically use most recent context
- Older memories accessed through thematic search

### Authoritative Knowledge
- Curator outputs are treated as "source of truth"
- Future conversations build upon this established knowledge
- Prevents contradictions across conversations

### Thematic Clustering
- Keywords extracted from questions and answers
- Topic summaries generated for each conversation
- Enables finding related conversations across time

### Context Types
- **Memory Context**: Previous curator results
- **Repository Context**: Current codebase information
- **Temporal Context**: Current date/time awareness
- **Semantic Context**: User-provided additional context

## Implementation Details

### Memory Context Format
```
🧠 AUTHORITATIVE MEMORY CONTEXT:
Here is the most relevant recent conversation:

From 2024-01-15 10:30: Question: How to implement authentication in Rust?

Verified Answer:
[Curator output with detailed implementation guide]

---

⚡ Use this context to inform your response.
```

### Follow-up Detection Patterns
- "give me", "tell me more", "can you explain"
- "show me", "what about", "examples"
- "how does", " it ", " that ", " this "

### Database Cost Tracking
- Each conversation tracks token usage and costs
- Stage-level analytics stored
- Enables usage optimization

## Benefits

1. **Continuity**: Conversations build upon each other naturally
2. **Consistency**: Prevents contradictory answers
3. **Learning**: System improves with each conversation
4. **Efficiency**: Reuses validated knowledge
5. **Intelligence**: Multi-conversation awareness

## Future Enhancements

1. **Embedding Quality**: Move from placeholder to real sentence transformers
2. **Entity Recognition**: Use proper NER models
3. **Relationship Extraction**: ML-based relationship discovery
4. **Temporal Decay**: Weight recent memories more heavily
5. **User Personalization**: Track per-user conversation threads

## Conclusion

The consensus memory implementation creates a powerful system where each conversation contributes to a growing knowledge base. The Curator stage produces authoritative answers that become the foundation for future responses, enabling true multi-conversation intelligence that learns and improves over time.