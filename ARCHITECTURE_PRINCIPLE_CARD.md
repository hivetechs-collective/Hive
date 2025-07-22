# 🏗️ Architecture Principle Quick Reference

## Core Principle: Separation of Concerns

### THINKING (Consensus Pipeline)
**Purpose**: Analysis, reasoning, decision-making  
**Models**: OpenRouter (expensive, powerful)  
**Responsibilities**:
- 4-stage analysis (Generator → Refiner → Validator → Curator)
- Complex reasoning and validation
- Architectural decisions
- Understanding user intent
- Creating execution plans

**NEVER**: Execute files, make changes, retrieve data

### DOING (AI Helpers)
**Purpose**: Execution, retrieval, translation  
**Models**: Local AI (CodeBERT, GraphCodeBERT, etc.)  
**Responsibilities**:
- File operations (create, update, delete)
- Code translation between languages
- Semantic search and retrieval
- Pattern recognition
- Learning from outcomes

**NEVER**: Make architectural decisions, choose strategies

## Quick Check Before ANY Code

Ask yourself:
1. Is this THINKING or DOING?
2. Am I in the right layer?
3. Am I violating the separation?

## Red Flags 🚩

- `fs::write()` in consensus code
- Decision logic in AI helpers  
- Direct file manipulation without AI helpers
- Consensus doing retrieval/search

## Correct Patterns ✅

```rust
// Consensus creates plan
let operations = analyze_and_plan();

// AI Helpers execute plan  
ai_file_executor.execute(operations).await?;
```

## Remember

**Consensus THINKS** → **AI Helpers DO**

This separation enables:
- Cost efficiency (expensive models think, cheap models do)
- Better quality (specialized models for each task)
- Scalability (can run many AI helpers in parallel)
- Safety (AI helpers validate before executing)