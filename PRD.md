# Product Requirements Document: Hive AI Professional Workspace
## Modular CLI Orchestration Layer

**Version**: 1.0  
**Date**: August 3, 2025  
**Status**: Active Development

---

## Executive Summary

Hive AI will evolve from a standalone consensus engine into a **Professional AI Workspace** that orchestrates best-in-class CLI tools while providing unique value through its consensus pipeline. This PRD defines the implementation of a modular orchestration layer that hosts multiple AI CLI tools (Claude Code, Gemini, Qwen, OpenAI) and Git management (lazygit) as permanent terminal tabs, with intelligent routing between tools and the consensus pipeline.

### Core Philosophy
- **Be an orchestrator, not a reimplementor**
- **Zero maintenance when CLI tools update**
- **Maximum modular separation**
- **Consensus pipeline as the unique value proposition**

---

## Product Vision

### What We're Building
A desktop application that provides:
1. **Unified Workspace**: Single interface for multiple AI coding assistants and development tools
2. **Intelligent Orchestration**: Route outputs between tools and consensus pipeline
3. **Premium Consensus**: Subscription-gated 4-stage consensus pipeline for authoritative answers
4. **Zero Configuration**: Each CLI tool manages its own authentication and settings
5. **Future-Proof Architecture**: New tools can be added without code changes

### What We're NOT Building
- API gateway or authentication manager for CLI tools
- Git implementation (using lazygit instead)
- Custom UI for each tool (pure terminal hosting)
- Parsing or understanding of CLI outputs
- Offline capabilities

---

## User Personas

### Primary: Professional Developer
- Uses multiple AI coding assistants daily
- Values integrated workflows over switching between tools
- Willing to pay for consensus features that provide authoritative answers
- Expects tools to work exactly as they do standalone

### Secondary: Development Team Lead
- Needs to synthesize information from multiple sources
- Values the consensus pipeline for decision-making
- Requires audit trail of AI interactions
- Manages team subscriptions

---

## Core Features

### 1. Modular CLI Hosting

#### 1.1 Permanent Terminal Tabs
- **Claude Code CLI**: Anthropic's coding assistant
- **Gemini CLI**: Google's AI assistant
- **Qwen CLI**: Alibaba's AI assistant
- **OpenAI CLI**: OpenAI's assistant
- **lazygit**: Git management tool

Each terminal:
- Cannot be closed (permanent tabs)
- Auto-restarts if process dies
- Maintains its own authentication
- Preserves full native functionality

#### 1.2 Terminal Infrastructure
- Built on existing `terminal_xterm_simple.rs`
- PTY-based for full compatibility
- Preserves ANSI colors and formatting
- Supports interactive features

### 2. Selection and Routing System

#### 2.1 Text Selection
- Simple text selection within any terminal
- Visual highlight of selected text
- Keyboard shortcuts (Ctrl+Shift+C to copy)
- Selection history for recent selections

#### 2.2 Routing Actions
For any selected text:
- **"Send to Consensus"** - Routes to consensus pipeline (requires subscription)
- **"Send to [Tool]"** - Routes to another CLI tool's stdin
- **"Copy"** - Standard clipboard operation

#### 2.3 Visual Feedback
- Arrow indicators showing data flow direction
- Brief toast notifications for routing confirmations
- Status indicators on terminal tabs

### 3. Consensus Pipeline Integration

#### 3.1 Subscription Gating
- Validates Hive Key before allowing consensus access
- Shows remaining conversation count
- Clear messaging when limits reached
- Graceful degradation to tool-only mode

#### 3.2 Context Preservation
- Adds source context to routed content
- Maintains conversation history
- Stores authoritative answers in unified database

#### 3.3 Result Distribution
- Consensus results can be sent back to any CLI tool
- Preserves formatting for tool consumption
- Optional broadcast to multiple tools

### 4. Settings Simplification

#### 4.1 Remove
- Anthropic API key field and validation
- Any provider-specific configuration

#### 4.2 Keep
- Hive License Key (required for consensus)
- OpenRouter API Key (required for consensus)
- General application settings

### 5. Tool Management

#### 5.1 Health Monitoring
- Simple process alive check every 30 seconds
- Automatic restart on crash (max 3 attempts)
- Visual indicator of tool status

#### 5.2 Resource Management
- Rely on OS-level process management
- No custom resource limits
- Monitor for hung processes

---

## Technical Architecture

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Hive AI Desktop Application              │
├─────────────────────────────────────────────────────────────┤
│                         Menu Bar                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────┐  ┌──────────────────────────────────────────┐ │
│  │         │  │          Terminal Orchestrator            │ │
│  │ Sidebar │  │  ┌────────────────────────────────────┐  │ │
│  │         │  │  │          Terminal Tabs              │  │ │
│  │ - Files │  │  │ Claude | Gemini | Qwen | AI | Git  │  │ │
│  │ - Chat  │  │  └────────────────────────────────────┘  │ │
│  │ - Analytics  │  ┌────────────────────────────────────┐  │ │
│  │ - Settings│  │  │       Active Terminal View         │  │ │
│  │         │  │  │  ┌──────────────────────────────┐   │  │ │
│  │         │  │  │  │  $ claude --chat              │   │  │ │
│  │         │  │  │  │  > How do I implement...      │   │  │ │
│  │         │  │  │  │  [Selected Text]              │   │  │ │
│  │         │  │  │  └──────────────────────────────┘   │  │ │
│  │         │  │  │  ┌──────────────────────────────┐   │  │ │
│  │         │  │  │  │ → Consensus │ → Gemini │ Copy │   │  │ │
│  │         │  │  │  └──────────────────────────────┘   │  │ │
│  │         │  │  └────────────────────────────────────┘  │ │
│  └─────────┘  └──────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                        Status Bar                           │
└─────────────────────────────────────────────────────────────┘
```

### Component Architecture

```rust
// Core Components
pub struct TerminalOrchestrator {
    terminals: HashMap<CLIProvider, ManagedTerminal>,
    selection_router: SelectionRouter,
    consensus_bridge: ConsensusBridge,
    health_monitor: HealthMonitor,
}

pub struct ManagedTerminal {
    provider: CLIProvider,
    terminal: SimpleTerminal,
    status: TerminalStatus,
    restart_count: u32,
}

pub struct SelectionRouter {
    active_selection: Option<Selection>,
    routing_history: VecDeque<RoutingEvent>,
}

pub struct ConsensusBridge {
    subscription_manager: Arc<SubscriptionManager>,
    conversation_gateway: Arc<ConversationGateway>,
    consensus_engine: Arc<ConsensusEngine>,
}
```

### Data Flow

```
User Selects Text in Terminal A
           ↓
   Selection Router
           ↓
   [Route Decision]
     ↓          ↓
Consensus    Terminal B
Pipeline      (stdin)
     ↓
Unified DB
     ↓
Terminal C
 (optional)
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1)

#### 1.1 Settings Cleanup
- [ ] Remove Anthropic API key field from settings UI
- [ ] Remove Anthropic key validation logic
- [ ] Remove Anthropic key storage/retrieval
- [ ] Update settings UI layout
- [ ] Add help text explaining CLI auth model

#### 1.2 CLI Provider Framework
- [ ] Create `CLIProvider` enum with all tools
- [ ] Implement `CLIConfig` for each provider
- [ ] Create `ManagedTerminal` wrapper struct
- [ ] Add terminal lifecycle management

#### 1.3 MVP Terminal Orchestrator
- [ ] Create `TerminalOrchestrator` struct
- [ ] Implement spawn methods for each CLI
- [ ] Add basic health checking (process alive)
- [ ] Implement simple restart logic

### Phase 2: Core Features (Week 2)

#### 2.1 Permanent Terminal Tabs
- [ ] Modify `TerminalTab` to support permanent flag
- [ ] Update UI to hide close button for permanent tabs
- [ ] Add provider icons to tabs
- [ ] Implement tab switching logic
- [ ] Auto-spawn all CLI terminals on startup

#### 2.2 Selection System
- [ ] Add text selection to terminal renderer
- [ ] Create selection state management
- [ ] Implement selection highlighting
- [ ] Add keyboard shortcuts
- [ ] Create selection context menu

#### 2.3 Basic Routing
- [ ] Implement `SelectionRouter`
- [ ] Add "Send to Consensus" action
- [ ] Add "Send to [Tool]" actions
- [ ] Implement stdin writing to terminals
- [ ] Add visual feedback for routing

### Phase 3: Consensus Integration (Week 3)

#### 3.1 Subscription Checking
- [ ] Integrate subscription validation
- [ ] Add conversation count display
- [ ] Implement limit messaging
- [ ] Create graceful degradation

#### 3.2 Consensus Bridge
- [ ] Create `ConsensusBridge` component
- [ ] Add context to routed content
- [ ] Implement D1 authorization flow
- [ ] Store results in unified database

#### 3.3 Result Distribution
- [ ] Add consensus result callbacks
- [ ] Implement result routing to terminals
- [ ] Add result formatting options
- [ ] Create broadcast functionality

### Phase 4: Polish and Launch (Week 4)

#### 4.1 UI/UX Polish
- [ ] Add routing animations
- [ ] Implement toast notifications
- [ ] Create keyboard shortcut overlay
- [ ] Add selection history viewer
- [ ] Polish terminal status indicators

#### 4.2 Error Handling
- [ ] Handle CLI tool not found
- [ ] Manage authentication failures gracefully
- [ ] Add retry mechanisms
- [ ] Implement user-friendly error messages

#### 4.3 Testing and Documentation
- [ ] Test all CLI tool integrations
- [ ] Verify cross-platform compatibility
- [ ] Create user documentation
- [ ] Record demo videos
- [ ] Update marketing materials

---

## Success Metrics

### Technical Metrics
- All 5 CLI tools spawn successfully on all platforms
- < 50ms latency for text routing operations
- < 5% CPU overhead from orchestration
- 99.9% crash recovery success rate

### User Metrics
- 80% of users use 2+ CLI tools in a session
- 60% of selections routed to consensus
- < 2% abandonment due to technical issues
- 4.5+ star average user satisfaction

### Business Metrics
- 40% conversion from free to paid (consensus access)
- 70% monthly retention rate
- 25% of users upgrade to higher tiers
- 90% subscription renewal rate

---

## Technical Decisions

### Use Existing Infrastructure
- Leverage current `terminal_xterm_simple.rs` for PTY management
- Utilize existing Dioxus signals for state management
- Keep current subscription/database systems

### Avoid Complexity
- No custom resource management (cgroups, etc.)
- No high-performance IPC (simple channels suffice)
- No plugin architecture (hardcoded tools)
- No CLI output parsing

### Tool Choices
- **lazygit** for Git operations (most popular, best UX)
- **pty-process** for improved PTY handling (if needed)
- Standard `tokio` channels for IPC
- Existing Dioxus components for UI

---

## Security Considerations

### Process Isolation
- Each CLI runs in its own process
- No shared memory between tools
- Standard OS-level process isolation

### Data Protection
- No storage of CLI tool credentials
- Consensus results encrypted in database
- Audit logging for compliance

### Subscription Security
- D1 validation cannot be bypassed locally
- Conversation tokens expire after use
- Rate limiting prevents abuse

---

## Future Enhancements (Post-Launch)

### Phase 5: Advanced Features
- Tool-specific workspace layouts
- Collaborative features (share selections)
- AI-powered routing suggestions
- Custom tool registration

### Phase 6: Enterprise Features
- Team management dashboard
- Centralized audit logging
- Custom consensus models
- SSO integration

### Phase 7: Ecosystem
- Public tool registry
- Community-contributed tools
- Marketplace for consensus models
- Integration with popular IDEs

---

## Risks and Mitigations

### Risk: CLI Tool Breaking Changes
**Mitigation**: Our wrapper is thin enough that changes won't break integration

### Risk: Performance with Multiple Tools
**Mitigation**: Simple architecture with minimal overhead, rely on OS scheduling

### Risk: User Confusion with Auth
**Mitigation**: Clear documentation and in-app guidance about CLI authentication

### Risk: Subscription Bypass Attempts
**Mitigation**: Server-side validation with D1, no local credit storage

---

## Appendix A: CLI Tool Commands

### Required CLI Tools
```bash
# Claude Code
npm install -g @anthropic-ai/claude-cli

# Gemini
pip install google-gemini-cli

# Qwen
npm install -g @alibaba/qwen-cli

# OpenAI
pip install openai-cli

# lazygit
brew install lazygit  # macOS
# or download from GitHub releases
```

### Health Check Commands
- Claude: `claude --version`
- Gemini: `gemini --version`
- Qwen: `qwen version`
- OpenAI: `openai --version`
- lazygit: `lazygit --version`

---

## Appendix B: Code Examples

### Basic Terminal Spawning
```rust
impl TerminalOrchestrator {
    pub async fn spawn_cli(&mut self, provider: CLIProvider) -> Result<()> {
        let config = provider.get_config();
        let terminal = SimpleTerminal::new(
            &config.command,
            &config.args,
            config.env.clone(),
        ).await?;
        
        self.terminals.insert(provider, ManagedTerminal {
            provider: provider.clone(),
            terminal,
            status: TerminalStatus::Running,
            restart_count: 0,
        });
        
        Ok(())
    }
}
```

### Selection Routing
```rust
impl SelectionRouter {
    pub async fn route(&mut self, selection: Selection, target: RouteTarget) -> Result<()> {
        match target {
            RouteTarget::Consensus => {
                self.consensus_bridge.process(selection).await?;
            }
            RouteTarget::Tool(provider) => {
                if let Some(terminal) = self.terminals.get_mut(&provider) {
                    terminal.write_stdin(&selection.text).await?;
                }
            }
        }
        Ok(())
    }
}
```

---

## Conclusion

This PRD defines a clear path to transform Hive AI into a Professional AI Workspace that leverages best-in-class CLI tools while maintaining its unique consensus pipeline value proposition. By focusing on orchestration rather than reimplementation, we create a maintainable, scalable, and user-friendly platform that can evolve with the AI tooling ecosystem.

The phased approach ensures we can deliver value quickly while maintaining quality and stability throughout the development process.