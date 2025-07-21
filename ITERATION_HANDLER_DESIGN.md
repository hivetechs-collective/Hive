# Iteration Handler Design

## Overview

The Iteration Handler is a key component of the Claude Code integration that manages multi-step development workflows. It enables the AI to work iteratively on complex tasks, similar to how Claude Code handles development projects that require multiple steps, testing, and refinement.

## Purpose

The Iteration Handler allows Hive Consensus to:
- Execute operations in steps
- Test results after each step
- Fix errors when they occur
- Continue with subsequent operations
- Maintain context across iterations
- Request user feedback when needed

## Architecture

### Core Components

```rust
pub struct IterationHandler {
    context: IterationContext,
    ai_helpers: Arc<AIHelperEcosystem>,
    executor: Arc<StreamingOperationExecutor>,
    direct_handler: Arc<DirectExecutionHandler>,
    max_iterations: usize,
}
```

### Iteration Context

The `IterationContext` maintains state across iterations:

```rust
pub struct IterationContext {
    pub original_request: String,
    pub iteration: usize,
    pub executed_operations: Vec<ExecutedOperation>,
    pub test_results: Vec<TestResult>,
    pub current_goal: String,
    pub remaining_tasks: Vec<String>,
    pub affected_files: Vec<PathBuf>,
    pub errors: Vec<IterationError>,
    pub user_feedback: Vec<String>,
}
```

## Workflow

### 1. Initial Request Analysis

When a complex request comes in, the system:
1. Detects it requires multiple steps (via ModeDetector)
2. Creates an IterationHandler instance
3. Breaks down the request into tasks
4. Begins the iteration loop

### 2. Iteration Loop

Each iteration follows this pattern:

```
┌─────────────────┐
│ Analyze State   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│ Determine Next  │
│     Action      │
└────────┬────────┘
         │
    ┌────┴────┬────────┬──────────┬─────────┐
    │         │        │          │         │
    v         v        v          v         v
┌───────┐ ┌──────┐ ┌────────┐ ┌──────┐ ┌──────┐
│Execute│ │ Test │ │Request │ │ Fix  │ │Done! │
│ Ops   │ │      │ │Feedback│ │Errors│ │      │
└───────┘ └──────┘ └────────┘ └──────┘ └──────┘
```

### 3. Decision Making

The handler uses AI helpers to analyze:
- What operations succeeded/failed
- What tests passed/failed
- What errors were encountered
- What tasks remain

Based on this analysis, it recommends:
- `Continue`: Execute next operations
- `TestRequired`: Run tests first
- `FixRequired`: Fix errors before continuing
- `NeedsFeedback`: Ask user for clarification
- `Complete`: All tasks done

### 4. Error Handling

When errors occur:
1. Capture error details (type, message, location)
2. Use AI to analyze the error
3. Generate fix operations
4. Execute fixes
5. Re-test to verify

## Example Flow

### Simple Web Server Implementation

```
User: "Create a simple web server with a health endpoint"

Iteration 1:
- Task: Create main server file
- Execute: Create src/main.rs with basic server
- Result: Success

Iteration 2:
- Task: Add health endpoint
- Execute: Update src/main.rs with /health route
- Result: Success

Iteration 3:
- Task: Test the implementation
- Execute: cargo build
- Result: Error - missing dependencies

Iteration 4:
- Task: Fix missing dependencies
- Execute: Update Cargo.toml with actix-web
- Result: Success

Iteration 5:
- Task: Test again
- Execute: cargo build && cargo test
- Result: Success

Complete: Web server with health endpoint ready!
```

## Integration with Claude Code Features

### 1. Inline Execution
- Operations execute as they're generated
- Results show inline in the conversation
- Errors appear immediately for quick fixes

### 2. Auto-Accept Mode
- When enabled, iterations proceed automatically
- When disabled, user confirms each major step
- Critical operations always require confirmation

### 3. Context Preservation
- File modifications tracked across iterations
- Test results remembered
- Error patterns learned

### 4. Smart Recovery
- If an iteration fails, previous state preserved
- Can rollback operations if needed
- Learn from failures for future iterations

## Configuration

### Max Iterations
Default: 10 iterations
- Prevents infinite loops
- Configurable per request type
- User can extend if needed

### Iteration Triggers
- Error detection
- Test completion
- User feedback
- Task completion

### Context Limits
- Track up to 50 operations
- Store last 20 test results
- Keep 10 most recent errors

## Future Enhancements

### 1. Learning System
- Track successful iteration patterns
- Predict likely next steps
- Optimize task ordering

### 2. Parallel Iterations
- Execute independent tasks simultaneously
- Merge results intelligently
- Reduce total execution time

### 3. Checkpoint System
- Save iteration state periodically
- Resume from checkpoints
- Share contexts between sessions

### 4. Advanced Testing
- Property-based testing integration
- Fuzzing for edge cases
- Performance benchmarking

## Usage Example

```rust
// Create iteration handler
let handler = IterationHandler::new(
    original_request,
    ai_helpers,
    executor,
    direct_handler,
);

// Add initial tasks
handler.add_task("Create server structure");
handler.add_task("Implement endpoints");
handler.add_task("Add tests");
handler.add_task("Setup CI/CD");

// Run iteration loop
loop {
    match handler.iterate(previous_result, user_feedback).await? {
        NextIteration::Execute(operations) => {
            // Execute operations
            let result = executor.execute_batch(operations).await?;
            previous_result = ExecutionResult::Success { operations, message };
        }
        NextIteration::RunTests(commands) => {
            // Run test commands
            for cmd in commands {
                let output = run_command(&cmd).await?;
                previous_result = ExecutionResult::TestResult { 
                    result: TestResult { ... } 
                };
            }
        }
        NextIteration::RequestFeedback(questions) => {
            // Show questions to user
            let feedback = get_user_feedback(questions).await?;
            user_feedback = Some(feedback);
        }
        NextIteration::FixErrors(errors) => {
            // Generate and execute fixes
            let fix_ops = generate_fixes(errors).await?;
            previous_result = ExecutionResult::Success { 
                operations: fix_ops, 
                message: "Fixes applied" 
            };
        }
        NextIteration::Done(summary) => {
            // Show completion summary
            println!("{}", summary);
            break;
        }
    }
}
```

## Benefits

1. **Handles Complexity**: Breaks down complex requests into manageable steps
2. **Error Resilience**: Automatically recovers from errors
3. **User Control**: Allows intervention when needed
4. **Learning**: Improves over time through pattern recognition
5. **Transparency**: Shows all steps and decisions

The Iteration Handler brings Claude Code's iterative development capabilities to Hive Consensus, enabling it to handle complex, multi-step development tasks with the same fluency as simple file operations.