# 🤖 Agent Orchestration Guide - Terminal Migration

## ⚠️ IMPORTANT: SDK Integration Available

**This guide describes manual agent coordination. For automated, SDK-powered agents, see:**
- **[CLAUDE_AGENT_SDK_INTEGRATION.md](./CLAUDE_AGENT_SDK_INTEGRATION.md)** - Official SDK approach (recommended)

The Claude Agent SDK provides:
- ✅ Automated agent coordination
- ✅ Type-safe tool definitions
- ✅ Built-in verification hooks
- ✅ Automatic parallel execution
- ✅ Context management

**Use this manual guide when:**
- Learning agent patterns
- SDK not available/installed
- Requiring manual control

**Use SDK approach when:**
- Automating complex multi-week migrations
- Requiring type safety and verification
- Coordinating 3+ agents

---

## Overview (Manual Approach)
This document defines how Claude Code agents should be used manually to systematically migrate from TTYD to xterm.js + PTY architecture. Agents work in parallel to maximize efficiency while maintaining code quality.

## Agent Roles & Responsibilities

### 1. Architecture Agent
**Purpose**: Design decisions, pattern analysis, system design

**Responsibilities**:
- Analyze VS Code terminal architecture patterns
- Design data flow and IPC architecture
- Review integration points
- Validate architectural decisions
- Create technical specifications

**When to Use**:
- Before implementing new components
- When making architectural decisions
- For pattern discovery in large codebases
- Validating approach against industry standards

**Example Invocation**:
```
Analyze VS Code's ptyService.ts implementation and create a technical specification
for our PtyService.ts that follows their patterns while integrating with our existing
PtyManager.ts for AI CLI tools.
```

### 2. Implementation Agent
**Purpose**: Code generation, refactoring, feature implementation

**Responsibilities**:
- Write new TypeScript/JavaScript code
- Refactor existing code
- Implement IPC handlers
- Create React components
- Follow established patterns

**When to Use**:
- Creating new files from specifications
- Refactoring existing code
- Implementing features with clear requirements
- Converting designs to code

**Example Invocation**:
```
Create src/main/terminal/PtyService.ts following the specification in TERMINAL_ARCHITECTURE.md.
Integrate with existing PtyManager.ts patterns and ensure compatibility with our IPC API.
Include error handling, process cleanup, and TypeScript types.
```

### 3. Testing Agent
**Purpose**: Test creation, validation, quality assurance

**Responsibilities**:
- Write unit tests
- Create integration tests
- Validate functionality
- Test edge cases
- Performance benchmarking

**When to Use**:
- After implementing new features
- Before refactoring critical code
- Validating migration steps
- Performance testing

**Example Invocation**:
```
Create comprehensive tests for PtyService.ts covering:
- Terminal spawn/destroy lifecycle
- Data flow (input/output)
- Error handling and cleanup
- Multiple terminal instances
- Process management
Include both unit tests and integration tests.
```

### 4. Migration Agent
**Purpose**: Code removal, cleanup, deprecation

**Responsibilities**:
- Safely remove deprecated code
- Update references
- Clean up unused dependencies
- Verify no breaking changes
- Update documentation

**When to Use**:
- Removing TTYD-related code
- Cleaning up after migration
- Deprecating old patterns
- Ensuring clean codebase

**Example Invocation**:
```
Safely remove TTYDManager.ts and all TTYD-related code:
1. Find all references to TTYDManager
2. Verify they're replaced with PtyService
3. Remove TTYD binary bundling from build scripts
4. Update imports and dependencies
5. Clean up unused code
Report any remaining references before deletion.
```

### 5. Documentation Agent
**Purpose**: Documentation, guides, architecture docs

**Responsibilities**:
- Update technical documentation
- Create user guides
- Document APIs
- Update architecture diagrams
- Maintain migration logs

**When to Use**:
- After completing implementation
- Updating architecture docs
- Creating developer guides
- Recording decisions

**Example Invocation**:
```
Update MASTER_ARCHITECTURE_DESKTOP.md to reflect the new xterm.js + PTY terminal
architecture. Document:
- Data flow diagrams
- IPC API surface
- Component responsibilities
- Integration points
- Migration from TTYD
```

### 6. Integration Agent
**Purpose**: Component integration, end-to-end flow

**Responsibilities**:
- Connect components together
- Ensure data flows correctly
- Validate integration points
- Fix integration issues
- Coordinate between layers

**When to Use**:
- Connecting new components
- Debugging integration issues
- Validating end-to-end flows
- Coordinating renderer ↔ main process

**Example Invocation**:
```
Integrate xterm.js in TerminalManager.ts with the new PtyService via IPC:
1. Set up IPC listeners in renderer
2. Connect xterm.onData to IPC send
3. Connect IPC receive to xterm.write
4. Handle resize events
5. Test bidirectional data flow
Ensure AI CLI tools continue working during integration.
```

## Parallel Agent Workflow Patterns

### Pattern 1: Research + Implementation
**Use When**: Starting new features

```
Agent 1 (Architecture): Research VS Code patterns for feature X
Agent 2 (Documentation): Review existing documentation for related features
→ Wait for completion
→ Agent 3 (Implementation): Implement feature X using findings
→ Agent 4 (Testing): Create tests for feature X
```

### Pattern 2: Parallel Implementation
**Use When**: Multiple independent components

```
Agent 1: Create PtyService.ts
Agent 2: Refactor terminal-ipc-handlers.ts
Agent 3: Update TerminalManager.ts
Agent 4: Create test suite
→ All agents run in parallel
→ Agent 5 (Integration): Connect all components
```

### Pattern 3: Phased Migration
**Use When**: Migrating large systems

```
Phase 1:
  Agent 1: Analyze current TTYD implementation
  Agent 2: Design PTY replacement architecture

Phase 2:
  Agent 3: Implement PtyService (parallel)
  Agent 4: Implement xterm integration (parallel)
  Agent 5: Create tests (parallel)

Phase 3:
  Agent 6: Integration testing
  Agent 7: Migration cleanup

Phase 4:
  Agent 8: Documentation updates
```

## Terminal Migration Execution Plan

### Week 1: Foundation
**Day 1-2: Architecture & Design**
```
Agent (Architecture): Analyze VS Code terminal architecture
→ Create PtyService specification
→ Design IPC data flow

Agent (Documentation): Document current TTYD architecture
→ Create migration plan
→ Document rollback strategy
```

**Day 3-4: Core Implementation**
```
Agent (Implementation) #1: Create PtyService.ts
Agent (Implementation) #2: Refactor terminal-ipc-handlers.ts
Agent (Implementation) #3: Update TerminalManager.ts
Agent (Testing): Create test suite for all above

→ All run in parallel
```

**Day 5: Integration**
```
Agent (Integration): Connect all components
→ Verify data flow
→ Test with real shell commands
→ Validate AI CLI tools still work
```

### Week 2: Enhancement & Migration
**Day 1-2: Advanced Features**
```
Agent (Implementation) #1: Add resize handling
Agent (Implementation) #2: Add copy/paste support
Agent (Implementation) #3: Add terminal themes
Agent (Testing): Integration tests
```

**Day 3-4: TTYD Removal**
```
Agent (Migration): Remove TTYD code
→ Find all references
→ Update build scripts
→ Remove dependencies
→ Clean up bundling

Agent (Testing): Regression testing
→ Verify nothing broke
→ Test all terminal features
```

**Day 5: Polish**
```
Agent (Implementation): Fix any issues
Agent (Testing): Full test suite
Agent (Documentation): Update all docs
```

### Week 3: Production Ready
**Day 1-2: Performance & Stability**
```
Agent (Testing): Performance benchmarks
→ Memory usage
→ Latency measurements
→ Multi-terminal stress tests

Agent (Implementation): Optimize based on findings
```

**Day 3-4: User Experience**
```
Agent (Implementation): UI polish
→ Loading states
→ Error handling
→ User feedback

Agent (Documentation): User guide
```

**Day 5: Release Prep**
```
Agent (Testing): Full QA pass
Agent (Documentation): Release notes
Agent (Migration): Final cleanup
```

## Agent Coordination Rules

### 1. File-Level Locking
- Only one agent modifies a file at a time
- Declare file intentions upfront
- Wait for completion before dependent work

### 2. Communication Protocol
Agents report status in this format:
```
Agent [Role]: [Task Name]
Status: [In Progress | Completed | Blocked]
Files Modified: [list]
Dependencies: [what's needed to proceed]
Next: [what comes after this]
```

### 3. Dependency Management
```
Agent A creates PtyService.ts
  ↓ (blocks)
Agent B refactors IPC handlers (needs PtyService interface)
  ↓ (blocks)
Agent C integrates components (needs both A and B)
```

### 4. Quality Gates
Before marking complete, agents must:
- ✅ Code compiles without errors
- ✅ No TypeScript type errors
- ✅ Tests pass (if applicable)
- ✅ Documentation updated
- ✅ No breaking changes to existing features

## Agent Invocation Examples

### Example 1: Create New Component
```
Task: Create PtyService.ts

Agent Type: Implementation
Context: We're migrating from TTYD to node-pty. PtyService should manage
terminal processes using node-pty, following VS Code's ptyService pattern.

Requirements:
- Location: src/main/terminal/PtyService.ts
- Follow patterns from existing PtyManager.ts for AI CLI tools
- Expose methods: spawn, write, resize, kill
- Emit events: onData, onExit
- Handle multiple terminal instances
- Proper error handling and cleanup

Reference Files:
- electron-poc/src/main/terminal/PtyManager.ts (existing patterns)
- VS Code: src/vs/platform/terminal/node/ptyService.ts (industry pattern)

Return: Complete TypeScript implementation with comments
```

### Example 2: Research Task
```
Task: Analyze xterm.js integration patterns

Agent Type: Architecture
Context: Need to understand how VS Code integrates xterm.js in their renderer
process to replicate their proven approach.

Tasks:
1. Search VS Code codebase for xterm.js usage
2. Identify initialization patterns
3. Find data flow (IPC → xterm)
4. Document event handling
5. Note performance optimizations

Return: Technical specification with code examples showing:
- How xterm Terminal is created
- How data flows from IPC to xterm
- How resize is handled
- How addons are used
```

### Example 3: Integration Task
```
Task: Connect xterm.js to PtyService via IPC

Agent Type: Integration
Context: PtyService exists in main process, xterm.js in renderer.
Need bidirectional data flow.

Steps:
1. Update TerminalManager.ts to listen for IPC messages
2. Connect xterm.onData → IPC send to main process
3. Connect IPC receive from main → xterm.write
4. Handle terminal lifecycle (spawn, destroy)
5. Test with basic shell commands

Files to Modify:
- src/components/terminal/TerminalManager.ts
- src/preload.ts (if needed)

Return: Working integration with test results
```

## Success Criteria

### For Each Agent Task:
- ✅ Deliverable is complete and functional
- ✅ Code follows existing patterns
- ✅ TypeScript types are correct
- ✅ No console errors or warnings
- ✅ Documentation updated (if applicable)
- ✅ Tests pass (if applicable)

### For Migration Overall:
- ✅ All TTYD code removed
- ✅ xterm.js + PTY working perfectly
- ✅ AI CLI tools still functional
- ✅ Performance equal or better than TTYD
- ✅ No quarantine/signing issues
- ✅ Homebrew distribution works
- ✅ User experience smooth and professional

## Rollback Strategy

If migration encounters critical issues:

```
Agent (Migration): Rollback to TTYD
1. Restore TTYDManager.ts from git history
2. Revert IPC handler changes
3. Restore webview-based terminal panel
4. Remove xterm.js integration
5. Verify TTYD works locally (but note: still has distribution issues)
```

## Notes for Claude Code

- **Use agents liberally** - they're designed for parallel work
- **Be specific** - detailed prompts get better results
- **Coordinate carefully** - respect file-level locking
- **Verify quality** - check agent work before proceeding
- **Document decisions** - maintain audit trail
- **Think in phases** - don't try to do everything at once

This migration is **perfect for agents** because:
1. Clear requirements and specifications
2. Well-defined component boundaries
3. Proven patterns to follow (VS Code)
4. Multiple independent tasks can run in parallel
5. Quality gates are objective and measurable

---

*This document should be updated as we learn better agent coordination patterns during the migration.*
