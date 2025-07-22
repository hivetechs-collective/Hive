# 🎯 Essential Development Reminders

## Core Architecture Principle
**THINKING (Consensus) → DOING (AI Helpers)**
- Consensus: Analysis & decisions (OpenRouter models)
- AI Helpers: Execution & retrieval (Local AI models)
- NEVER mix: No file ops in consensus, no decisions in helpers

## Pre-Task Checklist (MANDATORY)
Before ANY code:
1. Check architecture: Is this thinking or doing?
2. Search existing patterns first
3. Verify APIs exist (don't assume)
4. Separate RSX logic from rendering
5. Use AI Helpers for ALL file operations

## Key Achievements
- ✅ AI Helpers are real AI models (125M+ parameters)
- ✅ Direct Execution uses profile.generator_model
- ✅ AI file operations integrated via AIConsensusFileExecutor
- ✅ Markdown rendering restored
- ✅ Pre-Task Checklist prevents errors

## Red Flags to Stop For
- 🚩 fs::write() in consensus code
- 🚩 Decision logic in AI helpers
- 🚩 Mixed Rust logic in RSX blocks
- 🚩 Assumed APIs that don't exist

## Quick Verification
```bash
# Before implementing:
grep -r "similar_pattern" src/
# Check actual API:
grep "method_name" src/module/file.rs
# Verify no violations:
grep -r "fs::write" src/consensus/
```

Remember: 10 minutes research saves hours debugging!