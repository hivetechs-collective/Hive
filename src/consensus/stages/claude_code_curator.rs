//! Claude Code-Style Curator Stage
//!
//! Enhanced curator that formats responses with inline file operations,
//! similar to how Claude Code presents code changes within the conversation flow.

use crate::consensus::stages::ConsensusStage;
use crate::consensus::types::{Message, Stage, StagePrompts};
use anyhow::Result;

pub struct ClaudeCodeCuratorStage {
    inline_operations_enabled: bool,
}

impl ConsensusStage for ClaudeCodeCuratorStage {
    fn stage(&self) -> Stage {
        Stage::Curator
    }

    fn system_prompt(&self) -> &'static str {
        if self.inline_operations_enabled {
            CLAUDE_CODE_CURATOR_SYSTEM_PROMPT
        } else {
            StagePrompts::curator_system()
        }
    }

    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        let validated_response = previous_answer.unwrap_or("No response to curate");

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: self
                .build_claude_code_system_prompt(question, validated_response)
                .to_string(),
        }];

        // Add inline operation guidelines
        if self.inline_operations_enabled {
            messages.push(Message {
                role: "system".to_string(),
                content: self.get_inline_operation_guidelines().to_string(),
            });
        }

        // Add context with curation-specific instructions
        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: self.structure_claude_code_context(ctx, question),
            });
        }

        // Add the question and validated response with specific curation tasks
        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "ORIGINAL QUESTION:\n{}\n\nValidated analysis from Validator:\n{}\n\nCURATION OBJECTIVES:\n{}",
                question,
                validated_response,
                self.get_claude_code_objectives()
            ),
        });

        Ok(messages)
    }
}

impl ClaudeCodeCuratorStage {
    pub fn new(inline_operations_enabled: bool) -> Self {
        Self {
            inline_operations_enabled,
        }
    }

    /// Build Claude Code-style system prompt
    pub fn build_claude_code_system_prompt(&self, question: &str, response: &str) -> String {
        format!(
            "{}\n\nCLAUDE CODE INTEGRATION:\n{}\n\nFINAL STAGE OBJECTIVES:\n{}",
            CLAUDE_CODE_CURATOR_SYSTEM_PROMPT,
            self.analyze_for_file_operations(question, response),
            self.get_claude_code_objectives()
        )
    }

    /// Analyze if the response should include file operations
    fn analyze_for_file_operations(&self, question: &str, _response: &str) -> &'static str {
        let q_lower = question.to_lowercase();

        if q_lower.contains("create")
            || q_lower.contains("write")
            || q_lower.contains("add")
            || q_lower.contains("implement")
            || q_lower.contains("build")
            || q_lower.contains("make")
            || q_lower.contains("update")
            || q_lower.contains("modify")
            || q_lower.contains("refactor")
            || q_lower.contains("fix")
        {
            "This request involves file operations. Format your response with inline code blocks showing file creation/modification operations as they happen, similar to Claude Code."
        } else {
            "This appears to be an informational request. Focus on clear explanation without file operations."
        }
    }

    /// Structure context for Claude Code style
    pub fn structure_claude_code_context(&self, context: &str, _question: &str) -> String {
        let mut structured = String::new();

        if context.contains("CRITICAL REPOSITORY CONTEXT")
            || context.contains("ACTUAL FILE CONTENTS")
        {
            structured.push_str("⚠️ CLAUDE CODE CURATION REQUIREMENT:\n");
            structured.push_str("Format file operations inline within your response:\n");
            structured.push_str("1. Show operations as they happen (Creating `filename`...)\n");
            structured.push_str("2. Include full file content in code blocks\n");
            structured.push_str("3. Add execution status indicators (✅ Created, ✅ Updated)\n");
            structured.push_str("4. Make operations feel like natural conversation flow\n");
            structured.push_str("5. Test code and show results inline when appropriate\n\n");
        }

        structured.push_str(context);
        structured.push_str("\n\n🎯 INLINE OPERATION FORMATTING:\n");
        structured.push_str("- Present file operations conversationally\n");
        structured.push_str("- Show progress as you work through the task\n");
        structured.push_str("- Include status updates after each operation\n");
        structured.push_str("- Make the response feel interactive and live\n");

        structured
    }

    /// Get inline operation guidelines
    fn get_inline_operation_guidelines(&self) -> &'static str {
        "INLINE OPERATION FORMATTING GUIDELINES:

1. FILE CREATION:
   Text: \"Creating `path/to/file.ext`:\"\n
   Code block with full content
   Status: \"✅ Created path/to/file.ext\"

2. FILE UPDATES:
   Text: \"Updating `path/to/file.ext`:\"\n
   Code block with updated content or diff
   Status: \"✅ Updated path/to/file.ext\"

3. MULTIPLE OPERATIONS:
   Present operations in logical order
   Show dependencies being resolved
   Include brief explanations between operations

4. TESTING/VERIFICATION:
   When appropriate, show test execution:
   ```bash
   cargo test module_name
   ```
   ✅ All tests passed (5/5)

5. NATURAL FLOW:
   - Make it conversational: \"Let me create...\", \"Now I'll update...\", \"Next, let's add...\"
   - Explain what you're doing and why
   - Show the thought process, not just the result
   - Ask follow-up questions when appropriate

6. ERROR HANDLING:
   If an operation might fail, acknowledge it:
   \"This will create... Note: ensure the directory exists\"
   Or show error recovery inline

IMPORTANT: Make the entire response feel like you're actively working on the code WITH the user, not just presenting a plan."
    }

    /// Get Claude Code-specific objectives
    fn get_claude_code_objectives(&self) -> &'static str {
        "CLAUDE CODE CURATOR OBJECTIVES:
1. Transform validated analysis into executable inline operations
2. Present file operations as they would appear in Claude Code
3. Maintain conversational flow while showing concrete actions
4. Include progress indicators and status updates
5. Make complex operations feel simple and approachable
6. Show testing and verification inline when relevant
7. Create an experience of collaborative coding
8. Ensure all operations are clear and unambiguous
9. Provide natural transitions between operations
10. End with clear next steps or follow-up questions"
    }
}

/// Claude Code-style curator system prompt
const CLAUDE_CODE_CURATOR_SYSTEM_PROMPT: &str = "You are the Curator in a 4-stage AI consensus pipeline, enhanced with Claude Code capabilities. Your role is to synthesize all previous analyses into a polished, comprehensive final answer that includes inline file operations when appropriate.

When the user's request involves creating, modifying, or working with code:
1. Present file operations inline as you work through the solution
2. Show the actual code being created or modified
3. Include status indicators (✅) after operations complete
4. Make the response feel like live coding assistance
5. Test and verify changes inline when appropriate

Focus on clarity, completeness, and making the user feel like you're actively helping them code, not just explaining what could be done.";

#[cfg(all(test, feature = "legacy-tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_claude_code_curator_creation() {
        let curator = ClaudeCodeCuratorStage::new(true);
        assert_eq!(curator.stage(), Stage::Curator);
    }

    #[test]
    fn test_inline_operations_detection() {
        let curator = ClaudeCodeCuratorStage::new(true);
        let messages = curator
            .build_messages(
                "Create a new authentication module",
                Some("Previous analysis about auth"),
                None,
            )
            .unwrap();

        // Should include inline operation guidelines
        assert!(messages
            .iter()
            .any(|m| m.content.contains("INLINE OPERATION")));
    }

    #[test]
    fn test_standard_mode() {
        let curator = ClaudeCodeCuratorStage::new(false);
        assert_eq!(curator.system_prompt(), StagePrompts::curator_system());
    }
}
