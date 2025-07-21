# Claude Code Output Format Guide

## 🎯 Overview

This document defines the exact output format that Hive Consensus should use to match Claude Code's style. The goal is to provide clear, concise, and scannable output that shows file operations inline with the conversation flow.

## 📐 Core Formatting Principles

### 1. **Operation Indicators**
- Use `⏺` for operations and actions
- Use `✳` for status updates
- Clear visual hierarchy with consistent prefixes

### 2. **Concise Output**
- Show file paths and line numbers clearly
- Use collapsible sections for long outputs (ctrl+r to expand)
- Display exact diffs with line numbers and +/- indicators

### 3. **Progressive Workflow**
- Show investigation steps clearly
- Explain what's being done and why
- Build understanding progressively

## 📋 Output Format Examples

### File Update Operations

```
⏺ Update(src/consensus/direct_executor.rs)
  ⎿  Updated src/consensus/direct_executor.rs with 2 additions and 3 removals
       221                model: model.id.clone(),
       222                conversation_id: "direct".to_string(),
       223                timestamp: chrono::Utc::now(),
       224 -              token_count: 0, // Would need to track this
       225 -              cost: 0.0, // Would need to calculate this
       226 -              duration: 0.0, // Would need to track this
       224 +              usage: None, // Would need to track this
       225 +              analytics: None, // Would need to track this
       226            })?;
       227
       228            Ok(())
```

### Search Operations

```
⏺ Search(pattern: "pub fn.*get.*model", path: "src/consensus/models.rs", output_mode: "content")
  ⎿  Found 0 lines

⏺ Let me check what methods are available on ModelManager:

⏺ Search(pattern: "impl ModelManager", path: "src/consensus/models.rs", output_mode: "content")
  ⎿  Found 27 lines (ctrl+r to expand)
```

### File Creation

```
⏺ Creating `src/models/user.rs`:
```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```
  ⎿  Created src/models/user.rs (12 lines)
```

### Progressive Investigation

```
⏺ Let me check what methods the SafetyGuardrailSystem has:

⏺ Search(pattern: "pub fn validate|pub async fn validate", path: "src/consensus/safety_guardrails.rs", output_mode: "content")
  ⎿  Found 4 lines (ctrl+r to expand)

⏺ I see it takes EnhancedFileOperation. Let me check the safety context structure:

⏺ Search(pattern: "struct SafetyContext", path: "src/consensus/safety_guardrails.rs", output_mode: "content")
  ⎿  Found 11 lines (ctrl+r to expand)

⏺ The SafetyContext structure is different. Let me create a proper wrapper for safety validation:
```

### Build and Test Operations

```
⏺ Build(release mode)
  ⎿  Building hive-ai v2.0.2
     Compiling consensus module...
     Compiling UI components...
     ✅ Build successful in 45.2s

⏺ Test(consensus integration tests)
  ⎿  Running 127 tests
     test consensus::tests::test_pipeline_flow ... ok
     test consensus::tests::test_streaming ... ok
     ...
     ✅ All tests passed (12.3s)
```

### Error Handling

```
⏺ Build(release mode)
  ⎿  Building hive-ai v2.0.2
     ❌ Build failed with 3 errors:
     
     error[E0412]: cannot find type `SafetyGuardrailSystem` in this scope
       --> src/consensus/direct_executor.rs:164:60
        |
     164 |     pub fn with_safety_system(mut self, safety_system: Arc<SafetyGuardrailSystem>) -> Self {
        |                                                            ^^^^^^^^^^^^^^^^^^^^^ not found
        |
     help: consider importing this struct
        |
     6   + use crate::consensus::SafetyGuardrailSystem;
        |

⏺ Let me fix this import issue:
```

## 🔄 Implementation in Hive Consensus

### 1. **Curator Stage Output**
The curator should format its responses to include these operation markers:

```rust
// Instead of:
"I'll create a file at src/main.rs with the following content..."

// Use:
"⏺ Creating `src/main.rs`:"
```

### 2. **Streaming Display**
Operations should be displayed as they execute:

```
⏺ Update(src/auth/handler.rs)
  ⎿  Updating... [spinner]
  ⎿  Updated src/auth/handler.rs with 15 additions and 8 removals
```

### 3. **Search Integration**
When the AI needs to understand code:

```
⏺ Let me check the existing authentication implementation:

⏺ Search(pattern: "fn authenticate", path: "src/auth", output_mode: "content")
  ⎿  Found 3 matches (ctrl+r to expand)
```

## 📊 Status Indicators

### Operation Status
- `⎿  Updating...` - In progress
- `⎿  Updated` - Completed successfully
- `⎿  Failed` - Operation failed
- `⎿  Skipped` - Operation skipped (already exists, etc.)

### Progress Indicators
- `[spinner]` - Operation in progress
- `✅` - Success
- `❌` - Failure
- `⚠️` - Warning
- `🔍` - Searching/analyzing

## 🎨 Formatting Guidelines

### 1. **Line Diffs**
- Always show context (3 lines before/after)
- Use line numbers from the actual file
- Align +/- indicators properly
- Use consistent spacing

### 2. **File Paths**
- Always use relative paths from project root
- Highlight the filename when possible
- Show file stats (lines added/removed)

### 3. **Output Collapsing**
- Long outputs (>10 lines) should be collapsible
- Show summary with "(ctrl+r to expand)"
- Critical information should be visible without expanding

### 4. **Explanatory Text**
- Keep explanations concise
- Use progressive disclosure
- Explain the "why" when investigating
- Show clear reasoning for decisions

## 🚀 Benefits

1. **Scannable**: Users can quickly see what operations occurred
2. **Traceable**: Clear progression of investigation and execution
3. **Actionable**: Shows exactly what changed with precise diffs
4. **Professional**: Clean, consistent formatting
5. **Efficient**: Minimal noise, maximum information

## 📝 Examples in Context

### Complete Workflow Example

```
User: Fix the authentication error in the login handler

⏺ Let me investigate the authentication error:

⏺ Search(pattern: "login.*handler|auth.*error", path: "src/auth", output_mode: "files_with_matches")
  ⎿  Found 3 files:
     - src/auth/handlers/login.rs
     - src/auth/errors.rs
     - src/auth/middleware.rs

⏺ Let me check the login handler implementation:

⏺ Read(src/auth/handlers/login.rs)
  ⎿  Read 145 lines (ctrl+r to expand)

⏺ I found the issue. The password verification is using an outdated method. Let me fix it:

⏺ Update(src/auth/handlers/login.rs)
  ⎿  Updated src/auth/handlers/login.rs with 3 additions and 2 removals
       42     // Verify password
       43 -   let is_valid = bcrypt::verify(&req.password, &user.password_hash);
       44 -   if !is_valid {
       43 +   let is_valid = bcrypt::verify(&req.password, &user.password_hash)
       44 +       .map_err(|e| AuthError::PasswordVerification(e.to_string()))?;
       45 +   if !is_valid {
       46         return Err(AuthError::InvalidCredentials);
       47     }

⏺ Let me also add proper error handling to the error type:

⏺ Update(src/auth/errors.rs)
  ⎿  Updated src/auth/errors.rs with 3 additions
       15     InvalidCredentials,
       16     TokenExpired,
       17     TokenInvalid,
       18 +   #[error("Password verification failed: {0}")]
       19 +   PasswordVerification(String),
       20 }

⏺ Build(check mode)
  ⎿  ✅ Build successful - no errors

The authentication error has been fixed! The issue was that password verification errors weren't being properly handled, causing a panic instead of returning a proper error response.
```

This format ensures that Hive Consensus provides the same high-quality, scannable output that users expect from Claude Code.