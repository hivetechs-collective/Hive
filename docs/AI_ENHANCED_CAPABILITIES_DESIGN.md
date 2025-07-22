# AI Enhanced Capabilities Design (Future Implementation)

## Overview

This document outlines the design for future AI Helper capabilities that maintain our architecture principle: Consensus THINKS → AI Helpers DO.

## Architecture-Compliant Design

### Code Translation (Future)

**Consensus Layer (THINKING)**:
```rust
// In consensus pipeline
let translation_plan = curator.analyze_translation_requirements(
    source_code,
    from_language,
    to_language
)?;
```

**AI Helper Layer (DOING)**:
```rust
// In AI helpers - only execution
let translated = ai_helpers.execute_translation_plan(translation_plan)?;
```

### Semantic Retrieval (Current)

**Consensus Layer (THINKING)**:
```rust
// Consensus decides what to search for
let search_strategy = curator.determine_search_strategy(query)?;
```

**AI Helper Layer (DOING)**:
```rust
// AI Helpers execute the search
let results = knowledge_indexer.search_similar(
    &search_strategy.query,
    search_strategy.limit
).await?;
```

## Current Working Capabilities

### 1. Semantic Search (Available Now)
```rust
// AI Helpers can search for similar knowledge
let similar_docs = ai_helpers.knowledge_indexer
    .search_similar(query, limit)
    .await?;
```

### 2. Operation Prediction (Available Now)
```rust
// AI Helpers predict success based on history
let prediction = ai_helpers.knowledge_indexer
    .predict_operation_success(&operation, &context)
    .await?;
```

### 3. Quality Analysis (Available Now)
```rust
// AI Helpers assess quality
let metrics = ai_helpers.quality_analyzer
    .analyze_text_quality(text, "code_quality")
    .await?;
```

### 4. Context Preparation (Available Now)
```rust
// AI Helpers prepare context for consensus
let stage_context = ai_helpers
    .prepare_stage_context(question, stage)
    .await?;
```

## Implementation Guidelines

### For New Features

1. **Consensus decides WHAT**:
   - Analyze requirements
   - Create execution plan
   - Make architectural decisions

2. **AI Helpers execute HOW**:
   - Follow the plan
   - Use their intelligence for execution
   - Learn from outcomes

### Example: Adding Code Translation

```rust
// WRONG - AI Helper making decisions
async fn translate_code(&self, code: &str, to: &str) {
    let strategy = self.decide_translation_approach(); // ❌ Decision!
}

// RIGHT - AI Helper executing plan
async fn execute_translation(&self, plan: TranslationPlan) {
    let result = self.apply_translation_rules(plan); // ✅ Execution!
}
```

## Available AI Helper Methods

### KnowledgeIndexer
- `index_output` - Index curator outputs
- `search_similar` - Semantic search
- `find_similar_operations` - Find similar file operations
- `predict_operation_success` - Predict based on history

### ContextRetriever
- `get_generator_context` - Context for generator stage
- `analyze_operation_context` - Analyze file operations
- `get_success_rate_analysis` - Historical success rates

### QualityAnalyzer
- `evaluate_quality` - Evaluate indexed knowledge
- `analyze_text_quality` - Analyze code/text quality
- `assess_operation_risk` - Risk assessment

### IntelligentContextOrchestrator
- `make_intelligent_context_decision` - Route to appropriate helper

## Conclusion

The AI Helpers have powerful capabilities, but they must be used correctly:
- ✅ Execute plans from Consensus
- ✅ Search and retrieve information
- ✅ Analyze quality and risk
- ✅ Learn from outcomes
- ❌ Make architectural decisions
- ❌ Choose strategies
- ❌ Decide what to do

This maintains our architecture while fully utilizing AI Helper intelligence!