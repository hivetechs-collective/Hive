# Claude Agent SDK Integration Plan

## Executive Summary

The **Claude Agent SDK** (released October 2025) provides an official framework for building autonomous AI agents with Claude. This document outlines how to integrate the SDK into our Hive Consensus IDE to leverage its capabilities for our terminal migration and AI tool orchestration.

## What is the Claude Agent SDK?

### Official Definition
From Anthropic: "A collection of tools that helps developers build powerful agents on top of Claude Code" that can perform complex, iterative tasks across various domains.

### Key Capabilities
- **Autonomous Task Execution**: Agents can gather context, take action, verify work, and iterate
- **Tool Ecosystem**: Built-in file operations, code execution, web search
- **Subagent Support**: Launch specialized agents for parallel processing
- **Context Management**: Automatic context compaction to prevent context overflow
- **MCP Integration**: Model Context Protocol for external service connections
- **Permission System**: Fine-grained control over tool access

## SDK Architecture

### Core Components

```typescript
import { query, tool, createSdkMcpServer, AgentDefinition } from '@anthropic-ai/claude-agent-sdk';

// 1. Query Function - Primary interaction method
const result = await query({
  prompt: "Analyze this codebase and suggest improvements",
  options: {
    agents: { /* subagent definitions */ },
    tools: ['Read', 'Grep', 'Write'],
    hooks: { /* event handlers */ }
  }
});

// 2. Tool Definition - Create custom tools
const myTool = tool({
  name: "analyze_terminal",
  description: "Analyze terminal implementation",
  schema: z.object({
    path: z.string()
  }),
  handler: async ({ path }) => {
    // Tool logic
    return { result: "analysis" };
  }
});

// 3. Subagent Definition - Specialized agents
const codeReviewAgent: AgentDefinition = {
  description: "Reviews code for quality and security",
  prompt: "You are an expert code reviewer...",
  tools: ['Read', 'Grep'],
  permissionMode: 'allow-listed-tools'
};
```

### Agent Loop Pattern

```
1. Gather Context → Read files, search codebase, analyze
2. Take Action    → Write code, run commands, make changes
3. Verify Work    → Check results, run tests, validate
4. Iterate        → Repeat until task complete
```

## Integration Strategy for Hive Consensus

### Phase 1: SDK Installation & Setup

```bash
# Install the SDK
npm install @anthropic-ai/claude-agent-sdk

# Verify installation
npm list @anthropic-ai/claude-agent-sdk
```

**Files to Create:**
- `src/agents/AgentOrchestrator.ts` - Central agent coordination
- `src/agents/types.ts` - TypeScript interfaces
- `src/agents/tools/` - Custom tool definitions

### Phase 2: Rewrite AGENTS.md Using SDK Patterns

Our current `AGENTS.md` defines manual agent roles. We should restructure it to leverage the SDK:

**Current Approach** (Manual):
```
Agent 1: Research terminal architecture
Agent 2: Implement PtyService
Agent 3: Write tests
```

**SDK-Powered Approach**:
```typescript
const terminalMigrationOrchestrator = query({
  prompt: "Migrate terminal from TTYD to xterm.js + PTY",
  options: {
    agents: {
      architecture_analyst: {
        description: "Analyzes VS Code terminal patterns",
        tools: ['Read', 'Grep', 'WebFetch']
      },
      implementation_specialist: {
        description: "Implements PtyService following patterns",
        tools: ['Read', 'Write', 'Edit', 'Bash']
      },
      test_engineer: {
        description: "Creates comprehensive test suites",
        tools: ['Write', 'Bash', 'Read']
      }
    }
  }
});
```

### Phase 3: Custom Tools for Terminal Migration

Create specialized tools for our terminal work:

```typescript
// src/agents/tools/terminal-tools.ts
import { tool } from '@anthropic-ai/claude-agent-sdk';
import { z } from 'zod';

export const analyzePtyImplementation = tool({
  name: "analyze_pty_implementation",
  description: "Analyzes existing PTY implementation in the codebase",
  schema: z.object({
    filePath: z.string()
  }),
  handler: async ({ filePath }) => {
    // Read PtyManager.ts, extract patterns
    const code = await fs.readFile(filePath, 'utf-8');

    return {
      patterns: extractPatterns(code),
      dependencies: extractDependencies(code),
      interfaces: extractInterfaces(code)
    };
  }
});

export const validateTerminalIntegration = tool({
  name: "validate_terminal_integration",
  description: "Validates that xterm.js and PTY are properly integrated",
  schema: z.object({
    terminalManagerPath: z.string(),
    ptyServicePath: z.string()
  }),
  handler: async ({ terminalManagerPath, ptyServicePath }) => {
    // Check IPC handlers, data flow, event bindings
    const validation = await runIntegrationChecks();

    return {
      isValid: validation.passed,
      errors: validation.errors,
      warnings: validation.warnings
    };
  }
});

export const removeDeprecatedCode = tool({
  name: "remove_deprecated_code",
  description: "Safely removes TTYD code after verifying replacement",
  schema: z.object({
    deprecatedFiles: z.array(z.string()),
    verifyReplacement: z.boolean()
  }),
  handler: async ({ deprecatedFiles, verifyReplacement }) => {
    if (verifyReplacement) {
      // Check that PTY replacement is working
      const testResults = await runTerminalTests();
      if (!testResults.allPassed) {
        return { error: "PTY replacement not fully functional" };
      }
    }

    // Safe to delete TTYD code
    for (const file of deprecatedFiles) {
      await fs.unlink(file);
    }

    return { removed: deprecatedFiles.length };
  }
});
```

### Phase 4: Agent Orchestration for Terminal Migration

Create a master orchestrator that coordinates all migration work:

```typescript
// src/agents/TerminalMigrationOrchestrator.ts
import { query, AgentDefinition } from '@anthropic-ai/claude-agent-sdk';
import * as terminalTools from './tools/terminal-tools';

export class TerminalMigrationOrchestrator {
  private agents: Record<string, AgentDefinition> = {
    architecture_analyst: {
      description: "Expert in terminal architecture and VS Code patterns",
      prompt: `You are an expert in terminal architecture. Your job is to:
      1. Analyze VS Code's xterm.js + node-pty integration
      2. Study our existing PtyManager.ts implementation
      3. Design PtyService.ts that unifies both patterns
      4. Document data flow and IPC architecture`,
      tools: ['Read', 'Grep', 'WebFetch', 'analyze_pty_implementation']
    },

    pty_service_implementer: {
      description: "Implements PtyService.ts following specifications",
      prompt: `You are a TypeScript/Electron expert. Your job is to:
      1. Read the PtyService specification
      2. Study existing PtyManager.ts for patterns
      3. Implement PtyService.ts with proper TypeScript types
      4. Ensure compatibility with existing IPC API`,
      tools: ['Read', 'Write', 'Edit', 'Bash']
    },

    ipc_refactorer: {
      description: "Refactors IPC handlers to use PTY instead of TTYD",
      prompt: `You are an IPC expert. Your job is to:
      1. Read terminal-ipc-handlers.ts
      2. Replace TTYDManager calls with PtyService calls
      3. Maintain backward compatibility
      4. Add proper error handling`,
      tools: ['Read', 'Edit', 'Grep']
    },

    xterm_integrator: {
      description: "Integrates xterm.js directly in renderer",
      prompt: `You are a frontend expert. Your job is to:
      1. Remove webview code from TTYDTerminalPanel
      2. Use xterm.js directly in renderer
      3. Connect xterm events to IPC
      4. Handle resize, input, output correctly`,
      tools: ['Read', 'Write', 'Edit']
    },

    test_engineer: {
      description: "Creates comprehensive test suite",
      prompt: `You are a testing expert. Your job is to:
      1. Create unit tests for PtyService
      2. Create integration tests for IPC ↔ PTY ↔ xterm flow
      3. Test AI CLI tools still work
      4. Validate terminal features (resize, copy/paste, themes)`,
      tools: ['Write', 'Bash', 'Read']
    },

    cleanup_specialist: {
      description: "Removes deprecated TTYD code safely",
      prompt: `You are a code cleanup expert. Your job is to:
      1. Verify PTY implementation is fully working
      2. Find all TTYD references
      3. Safely remove TTYD code
      4. Update build scripts to not bundle ttyd binary`,
      tools: ['Read', 'Grep', 'Edit', 'Bash', 'remove_deprecated_code']
    }
  };

  async executeMigration() {
    // Week 1: Architecture & Implementation
    console.log('📐 Week 1: Architecture & Core Implementation');

    const week1Result = await query({
      prompt: `Execute terminal migration - Week 1:

      1. Architecture Analysis (architecture_analyst):
         - Analyze VS Code terminal architecture
         - Study our PtyManager.ts
         - Design PtyService.ts specification
         - Document IPC data flow

      2. Core Implementation (pty_service_implementer, ipc_refactorer in parallel):
         - Implement PtyService.ts
         - Refactor terminal-ipc-handlers.ts
         - Ensure type safety and error handling

      3. Initial Testing (test_engineer):
         - Create unit tests for PtyService
         - Test basic spawn/write/destroy operations

      Report progress and any blockers.`,

      options: {
        agents: {
          architecture_analyst: this.agents.architecture_analyst,
          pty_service_implementer: this.agents.pty_service_implementer,
          ipc_refactorer: this.agents.ipc_refactorer,
          test_engineer: this.agents.test_engineer
        },
        tools: Object.values(terminalTools),
        hooks: {
          PreToolUse: [this.validateToolUse.bind(this)],
          PostToolUse: [this.logToolUse.bind(this)]
        }
      }
    });

    // Week 2: Integration & Migration
    console.log('🔗 Week 2: Integration & TTYD Removal');

    const week2Result = await query({
      prompt: `Execute terminal migration - Week 2:

      1. xterm.js Integration (xterm_integrator):
         - Update TerminalManager.ts
         - Remove webview code
         - Connect xterm ↔ IPC ↔ PTY

      2. Validation (test_engineer):
         - Integration tests
         - AI CLI tools still work
         - All terminal features functional

      3. TTYD Removal (cleanup_specialist):
         - Verify PTY fully working
         - Remove TTYDManager.ts
         - Update build scripts
         - Delete TTYD binary

      Report final status and test results.`,

      options: {
        agents: {
          xterm_integrator: this.agents.xterm_integrator,
          test_engineer: this.agents.test_engineer,
          cleanup_specialist: this.agents.cleanup_specialist
        },
        tools: Object.values(terminalTools)
      }
    });

    return {
      week1: week1Result,
      week2: week2Result
    };
  }

  private async validateToolUse(input: any) {
    // Prevent accidental deletion of critical files
    if (input.tool === 'remove_deprecated_code') {
      console.warn('⚠️  Code deletion requested - verifying safety');
    }
    return { continue: true };
  }

  private async logToolUse(output: any) {
    // Log all tool usage for audit trail
    console.log(`✓ Tool used: ${output.tool}`);
  }
}
```

### Phase 5: Project-Level Agent Configuration

Update `CLAUDE.md` to define our agent ecosystem:

```markdown
# Claude Code Agent Configuration

## Terminal Migration Agents

We use the Claude Agent SDK for systematic terminal migration from TTYD to xterm.js + PTY.

### Available Agents

- **architecture_analyst**: Analyzes patterns and designs architecture
- **pty_service_implementer**: Implements PtyService.ts
- **ipc_refactorer**: Updates IPC handlers
- **xterm_integrator**: Integrates xterm.js in renderer
- **test_engineer**: Creates comprehensive tests
- **cleanup_specialist**: Removes deprecated code

### Custom Tools

- `analyze_pty_implementation`: Extracts patterns from existing code
- `validate_terminal_integration`: Checks IPC ↔ PTY ↔ xterm flow
- `remove_deprecated_code`: Safely removes TTYD code

### Orchestration

Run the migration via:
```typescript
import { TerminalMigrationOrchestrator } from './src/agents/TerminalMigrationOrchestrator';

const orchestrator = new TerminalMigrationOrchestrator();
await orchestrator.executeMigration();
```
```

## Benefits of SDK Integration

### 1. Automated Coordination
- No manual agent invocation needed
- SDK handles subagent lifecycle
- Automatic context management

### 2. Type Safety
```typescript
// Before (manual)
Task: "Create PtyService.ts"
// No types, no validation

// After (SDK)
const result = await query({
  prompt: "Create PtyService.ts",
  options: {
    agents: {
      implementer: {
        tools: ['Write', 'Read'] // Type-checked
      }
    }
  }
});
```

### 3. Verification Built-In
```typescript
hooks: {
  PostToolUse: [async (output) => {
    // Automatically verify every action
    if (output.tool === 'Write') {
      await runTypeCheck(output.filePath);
    }
  }]
}
```

### 4. Parallel Execution
SDK automatically manages parallel subagents:
```typescript
agents: {
  agent1: { /* runs in parallel */ },
  agent2: { /* runs in parallel */ },
  agent3: { /* runs in parallel */ }
}
// SDK coordinates automatically
```

## Integration Timeline

### Immediate (This Week)
- [x] Install `@anthropic-ai/claude-agent-sdk`
- [ ] Create `src/agents/` directory structure
- [ ] Define custom tools in `src/agents/tools/terminal-tools.ts`
- [ ] Create `TerminalMigrationOrchestrator.ts`

### Week 1 (Architecture & Implementation)
- [ ] Run architecture analysis via SDK
- [ ] Implement PtyService.ts using SDK agents
- [ ] Refactor IPC handlers using SDK agents
- [ ] Create initial test suite

### Week 2 (Integration & Cleanup)
- [ ] Integrate xterm.js via SDK agents
- [ ] Run comprehensive testing
- [ ] Remove TTYD code safely
- [ ] Update build scripts

### Week 3 (Polish & Documentation)
- [ ] Performance optimization
- [ ] Documentation updates
- [ ] Final QA
- [ ] Homebrew release

## Code Examples

### Quick Start - Terminal Migration

```typescript
// src/scripts/migrate-terminal.ts
import { TerminalMigrationOrchestrator } from '../agents/TerminalMigrationOrchestrator';

async function main() {
  const orchestrator = new TerminalMigrationOrchestrator();

  console.log('🚀 Starting terminal migration...');
  console.log('📊 This will take ~2 weeks with autonomous agents');

  const result = await orchestrator.executeMigration();

  console.log('✅ Migration complete!');
  console.log(`Week 1 Result: ${result.week1.summary}`);
  console.log(`Week 2 Result: ${result.week2.summary}`);
}

main().catch(console.error);
```

### Run Migration

```bash
# Install SDK
npm install @anthropic-ai/claude-agent-sdk

# Set API key
export ANTHROPIC_API_KEY=your_key_here

# Run migration
npm run migrate:terminal
```

## Key Differences from Manual Agent Approach

| Aspect | Manual (AGENTS.md) | SDK-Powered |
|--------|-------------------|-------------|
| **Invocation** | Human launches each agent | SDK orchestrates automatically |
| **Coordination** | Human coordinates | SDK manages dependencies |
| **Verification** | Manual checking | Hooks validate every action |
| **Type Safety** | No types | Full TypeScript types |
| **Context** | Manual management | Automatic compaction |
| **Parallel Execution** | Manual coordination | SDK handles automatically |
| **Error Handling** | Ad-hoc | Built-in retry and recovery |

## Next Steps

1. **Install SDK**: `npm install @anthropic-ai/claude-agent-sdk`
2. **Create agent structure**: `mkdir -p src/agents/tools`
3. **Define custom tools**: Implement terminal-specific tools
4. **Create orchestrator**: Build TerminalMigrationOrchestrator
5. **Run migration**: Execute via SDK
6. **Monitor progress**: SDK provides real-time updates
7. **Verify results**: Hooks ensure quality

## Resources

- **Official SDK Docs**: https://docs.claude.com/en/api/agent-sdk/overview
- **TypeScript Reference**: https://docs.claude.com/en/docs/claude-code/sdk/sdk-typescript
- **GitHub Repo**: https://github.com/anthropics/claude-agent-sdk-typescript
- **Examples**: https://github.com/anthropics/claude-code-sdk-demos
- **Community Agents**: https://github.com/VoltAgent/awesome-claude-code-subagents

---

*This integration transforms our manual agent approach into a fully automated, SDK-powered system with type safety, verification, and autonomous execution.*
