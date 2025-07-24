# Claude Code Integration Status - Comprehensive Plan & Progress

## Overview
We are integrating Claude Code as a stateless execution engine into the hive-consensus GUI to replace the 5 AI Helper models, solve conversation compacting issues, and create a powerful hybrid system.

## Key Architectural Decisions

### 1. **Stateless Claude Code Execution**
- Each request to Claude Code is independent (no conversation history)
- Full context loaded from our database for each request
- Results stored back to knowledge_conversations table
- Avoids auto-compact issues and reduces token costs

### 2. **Replace DirectExecutionHandler**
- Create `ClaudeCodeExecutor` to replace `DirectExecutionHandler`
- Use Claude Code SDK in non-interactive mode
- Feed context from thematic memory and recent knowledge
- Store outputs as KnowledgeConversation entries

### 3. **Execution Modes**
- **Claude Autonomous**: Direct execution for all tasks
- **Claude Assisted**: Smart consensus validation for complex decisions
- **Consensus Required**: All plans validated through 4-stage pipeline
- Toggle in GUI (like plan mode toggle)

## Current Implementation Status

### ✅ Completed Tasks
1. **API Key Manager Updates** (`src/core/api_keys.rs`)
   - Added `anthropic_key` field to `ApiKeyConfig` struct
   - Created `validate_anthropic_format()` method
   - Implemented `test_anthropic_key()` with live validation
   - Added `get_anthropic_key()` method
   - Updated `save_to_database()` to handle Anthropic keys
   - Updated `load_from_database()` to retrieve Anthropic keys
   - Updated all validation methods to use specific format validators

### 🚧 In Progress
2. **Settings UI Updates**
   - ✅ Updated `SettingsDialog` component to include Anthropic API key field
   - ✅ Updated `OnboardingDialog` to include Anthropic API key field
   - ✅ Updated `hive-consensus.rs` to add `anthropic_key` signal
   - ✅ Updated API config initialization to include anthropic_key
   - ✅ Updated all effect hooks to clone anthropic_key
   - ✅ Updated SettingsDialog usage to pass anthropic_key prop
   - 🔄 **NEXT**: Need to find OnboardingDialog usage and update it
   - 🔄 **NEXT**: Need to implement save functionality when Settings/Onboarding dialogs close

### 📋 Pending Tasks

#### Phase 1: Complete UI Integration
1. Find and update OnboardingDialog usage to pass anthropic_key
2. Implement save_api_keys function that calls `ApiKeyManager::save_to_database()`
3. Hook up save functionality in Settings and Onboarding dialogs
4. Test API key persistence

#### Phase 2: Create ClaudeCodeExecutor
```rust
// src/consensus/claude_code_executor.rs
pub struct ClaudeCodeExecutor {
    claude_process: Option<ClaudeCodeProcess>,
    db: Arc<DatabaseManager>,
    thematic_cluster: Arc<ThematicCluster>,
    consensus_pipeline: Arc<ConsensusPipeline>,
    execution_mode: Arc<RwLock<ExecutionMode>>,
}
```

#### Phase 3: Implement Claude Code SDK Process
- Use TypeScript/Python SDK for subprocess management
- Implement streaming JSON output parsing
- Handle abort controllers for cancellation
- Create stateless request builder

#### Phase 4: Build Memory Context Loading
- Load recent curator knowledge (24h window)
- Get thematic matches from database
- Retrieve learned patterns from ContinuousLearner
- Build comprehensive context under token limit

#### Phase 5: GUI Integration
- Add execution mode toggle (Claude/Consensus/Hybrid)
- Show Claude execution status in UI
- Stream Claude responses to existing output window
- Handle cancellation properly

#### Phase 6: Smart Consensus Integration
- Detect uncertainty in Claude responses
- Implement consensus validation triggers
- Store both Claude and Curator outputs
- Create hooks for enterprise control

## Key Files Modified/To Modify

### Modified:
- `/src/core/api_keys.rs` - Complete API key management for Anthropic
- `/src/desktop/dialogs/mod.rs` - Added Anthropic key fields to dialogs
- `/src/bin/hive-consensus.rs` - Added anthropic_key signal and partially updated

### To Create:
- `/src/consensus/claude_code_executor.rs` - Main Claude integration
- `/src/consensus/claude_process.rs` - SDK subprocess management
- `/src/consensus/claude_hooks.rs` - Hook system for consensus

### To Modify:
- `/src/consensus/engine.rs` - Add Claude executor option
- `/src/consensus/mode_detector.rs` - Route to Claude instead of Direct
- `/src/core/database.rs` - Ensure knowledge storage works

## Critical Implementation Notes

### API Key Handling
- API keys stored in database only (not config.toml)
- Same pattern as OpenRouter keys
- Validation before saving
- Environment variable fallback

### Stateless Context Building
```rust
async fn build_stateless_context(&self, request: &str) -> Result<StatelessContext> {
    // Get recent curator knowledge (24h window)
    let recent_knowledge = self.thematic_cluster
        .get_recent_curator_knowledge(request).await?;
    
    // Get thematic matches
    let thematic_knowledge = self.thematic_cluster
        .get_thematic_knowledge(request, "direct").await?;
    
    // Get learned patterns from ContinuousLearner
    let learned_context = self.ai_helpers.continuous_learner
        .get_learned_context(request, Stage::Generator, 5000).await?;
    
    Ok(StatelessContext {
        recent_knowledge,
        thematic_knowledge,
        learned_patterns: learned_context,
        repository_context: repo_context,
    })
}
```

### System Prompt
```rust
const HYBRID_SYSTEM_PROMPT: &str = r#"
You are Claude Code integrated with Hive Consensus, operating in stateless mode.

Key principles:
1. Each request is independent - you have no conversation history
2. The context provided contains relevant past knowledge from the memory system
3. When uncertain about complex decisions, explicitly state "I would benefit from consensus validation"
4. All your outputs will be stored as knowledge for future requests
5. You have full access to all Claude Code capabilities and commands

Available context includes:
- Recent curator knowledge (authoritative answers from past 24h)
- Thematically similar past conversations
- Learned patterns and best practices
- Repository context when applicable
"#;
```

## Next Immediate Steps

1. **Find OnboardingDialog usage**:
   ```bash
   grep -r "OnboardingDialog" src/bin/hive-consensus.rs
   ```

2. **Implement save_api_keys function**:
   ```rust
   async fn save_api_keys(
       openrouter_key: &str,
       hive_key: &str,
       anthropic_key: &str,
   ) -> Result<()> {
       ApiKeyManager::save_to_database(
           Some(openrouter_key),
           Some(hive_key),
           Some(anthropic_key),
       ).await
   }
   ```

3. **Hook up save in dialogs**:
   - In SettingsDialog "Save" button onclick
   - In OnboardingDialog "Continue" button onclick

4. **Test the flow**:
   - Open settings
   - Enter Anthropic API key
   - Save
   - Restart app
   - Verify key persists

## Todo List Summary
- [IN PROGRESS] Add Anthropic API key configuration to Settings UI
- [PENDING] Complete save functionality for API keys
- [PENDING] Create ClaudeCodeExecutor module
- [PENDING] Implement Claude Code SDK process management
- [PENDING] Build stateless memory context loading
- [PENDING] Integrate Claude streaming with existing UI
- [PENDING] Add execution mode toggle to GUI
- [PENDING] Implement smart consensus invocation
- [PENDING] Store Claude outputs in knowledge_conversations
- [PENDING] Create hooks for consensus validation
- [PENDING] Test complete integration

## Key Features to Preserve
1. **Stateless execution** - No conversation history
2. **Memory integration** - Use our powerful database
3. **Consensus validation** - Multi-model perspectives
4. **Streaming UI** - Real-time updates
5. **Cancellation support** - Proper cleanup
6. **Knowledge storage** - Every output saved
7. **Smart routing** - Claude decides when to use consensus