# Consensus GUI Freezing Fix - Implementation Tracking

## 🚨 Critical Issue
**Problem**: GUI freezes when running consensus, system overheats, app becomes unresponsive
**Root Cause**: Consensus running on Dioxus's single-threaded runtime instead of Tokio

## 📍 Current State
- **Current Branch**: `fix/restore-lazygit-with-improvements`
- **Safe Rollback Commit**: `b3fa9f0` - "fix(terminal): resolve display bug by starting with no terminals and improving PTY sizing"
- **Date**: 2025-08-07
- **Status**: Starting implementation

## 🎯 Implementation Plan

### Phase 1: Immediate Fix (PARTIALLY COMPLETE)
- [x] Attempted to replace `dioxus::prelude::spawn` with `tokio::spawn` at line 4983
- [x] Added ConsensusMessage enum for future communication
- [x] Discovered blocker: DesktopConsensusManager is not Send due to Dioxus Signal<AppState>
- [ ] Need to refactor DesktopConsensusManager to separate Send-able engine from UI state
- [ ] Test that UI remains responsive during consensus
- [ ] Verify CPU usage is distributed across cores

**Blocker Found**: Cannot move consensus to Tokio thread because DesktopConsensusManager contains non-Send Dioxus signals. Need deeper refactoring.

### Phase 2: Thread Isolation (PARTIALLY COMPLETE)
- [x] Wrap AI helper initialization in `tokio::task::spawn_blocking`
  - Modified AIHelperEcosystem::new() to use spawn_blocking for:
    - Model downloading
    - Python service initialization
    - Vector store creation
    - All helper component initialization
- [ ] Add progress reporting from background threads
- [ ] Implement proper cancellation tokens
- [ ] Cache AI helpers between consensus runs
- [ ] Add timeout handling for long operations

### Phase 3: Lazy Initialization
- [ ] Make AI helper creation lazy (on-demand)
- [ ] Move Python model loading to background
- [ ] Add initialization state tracking
- [ ] Implement progress UI for initialization
- [ ] Add error recovery for failed initialization

## 📝 Implementation Details

### File Modifications Required

#### 1. `src/bin/hive-consensus.rs` (Primary Fix Location)
**Line 4983**: Change spawn implementation
```rust
// BEFORE (BROKEN):
let task = spawn(async move { ... });  // This is dioxus::prelude::spawn

// AFTER (FIXED):
// Create channel for communication
let (tx, mut rx) = tokio::sync::mpsc::channel(100);

// Spawn on Tokio runtime
let task = tokio::spawn(async move {
    // Heavy consensus work here
    let result = consensus.process_query_streaming(&enhanced_query).await;
    let _ = tx.send(result).await;
});

// Handle results on UI thread
spawn(async move {
    while let Some(result) = rx.recv().await {
        // Update UI safely
    }
});
```

#### 2. `src/desktop/consensus_integration.rs`
- Wrap ConsensusPipeline creation in spawn_blocking
- Add lazy initialization flag
- Cache pipeline instance

#### 3. `src/ai_helpers/mod.rs`
- Line 245: Make `AIHelperEcosystem::new()` use spawn_blocking
- Line 341: Move model downloading to background
- Add progress callbacks

## 🧪 Testing Checklist

### Immediate Tests (After Phase 1)
- [ ] Build successfully with `cargo build --bin hive-consensus`
- [ ] Launch GUI without errors
- [ ] Run consensus query
- [ ] Verify UI remains clickable during consensus
- [ ] Check CPU usage with Activity Monitor
- [ ] Test cancellation with Ctrl+C
- [ ] Verify no system overheating

### Performance Metrics to Track
- UI responsiveness during consensus (should be <16ms frame time)
- CPU usage distribution (should use multiple cores)
- Memory usage (should not spike on UI thread)
- Consensus completion time (should be similar or better)
- System temperature (should not cause thermal throttling)

## 🔍 Key Code Locations

1. **Spawn Call**: `/src/bin/hive-consensus.rs:4983`
   - This is where `dioxus::prelude::spawn` needs to be replaced

2. **Consensus Creation**: `/src/desktop/consensus_integration.rs:28-89`
   - Heavy initialization happening here

3. **AI Helper Init**: `/src/ai_helpers/mod.rs:245-339`
   - Multiple blocking operations during initialization

4. **Python Service**: `/src/ai_helpers/python_models.rs`
   - Synchronous Python initialization

## ⚠️ Rollback Plan

If implementation fails or causes new issues:

```bash
# Rollback to last known good commit
git reset --hard b3fa9f0

# Or if we've committed, revert the changes
git revert HEAD
```

## 📊 Progress Tracking

### Session 1 (Current)
- [x] Identified root cause: wrong async runtime
- [x] Created tracking document
- [ ] Implementing Phase 1 fix
- [ ] Testing basic functionality

### Success Criteria
1. ✅ GUI remains responsive during consensus
2. ✅ CPU usage distributed across cores
3. ✅ No system overheating
4. ✅ Cancellation works properly
5. ✅ No regression in consensus quality

## 🚀 Next Steps After Fix

Once immediate fix is verified:
1. Implement Phase 2 (thread isolation)
2. Implement Phase 3 (lazy initialization)
3. Continue with Dioxus copy plan for long-term solution
4. Add resource governor to copied Dioxus

## 📝 Notes

- The root issue is that Dioxus uses a WASM-compatible single-threaded runtime
- Tokio is already in our dependencies and runs on a separate thread pool
- This fix is temporary until we copy and modify Dioxus source
- Keep commits small and semantic for easy rollback

## 🔗 Related Documentation

- Original issue discussion: GUI freezing during consensus
- Dioxus runtime limitations: Single-threaded for WASM compatibility
- Tokio documentation: Multi-threaded async runtime
- AI Helper architecture: Heavy initialization requirements

---

Last Updated: 2025-08-07
Status: IN PROGRESS - Implementing Phase 1