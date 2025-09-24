// Refiner Stage - Improve and enhance the initial response
// Second stage of the consensus pipeline

use crate::consensus::stages::ConsensusStage;
use crate::consensus::types::{Message, Stage, StagePrompts};
use anyhow::Result;

pub struct RefinerStage;

impl ConsensusStage for RefinerStage {
    fn stage(&self) -> Stage {
        Stage::Refiner
    }

    fn system_prompt(&self) -> &'static str {
        StagePrompts::refiner_system()
    }

    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        let mut messages = vec![Message {
            role: "system".to_string(),
            content: self
                .build_enhanced_system_prompt(question, previous_answer)
                .to_string(),
        }];

        // Add context if provided with quality analysis
        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: self.structure_refiner_context(ctx, question),
            });
        }

        // Analyze the generator's response and provide targeted refinement instructions
        let generator_response = previous_answer.unwrap_or("No previous response available");
        let response_analysis = self.analyze_response_quality(generator_response);

        messages.push(Message {
            role: "system".to_string(),
            content: format!("RESPONSE ANALYSIS:\n{}", response_analysis),
        });

        // Add the original question and generator's response with improvement guidance
        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "ORIGINAL QUESTION:\n{}\n\nInitial analysis from Generator:\n{}\n\nREFINEMENT INSTRUCTIONS:\n{}",
                question,
                generator_response,
                self.build_improvement_instructions(&self.detect_response_type(generator_response))
            ),
        });

        Ok(messages)
    }
}

impl RefinerStage {
    pub fn new() -> Self {
        Self
    }

    /// Build enhanced system prompt based on question context
    pub fn build_enhanced_system_prompt(
        &self,
        question: &str,
        previous_answer: Option<&str>,
    ) -> String {
        let base_prompt = StagePrompts::refiner_system();
        let response_type = previous_answer
            .map(|r| self.detect_response_type(r))
            .unwrap_or("general");

        format!(
            "{}\n\nREFINEMENT FOCUS: This is a {} response. Apply targeted improvements specific to this content type.\n\nQUALITY STANDARDS:\n{}",
            base_prompt,
            response_type,
            self.get_quality_standards_for_type(response_type)
        )
    }

    /// Structure context specifically for refinement
    pub fn structure_refiner_context(&self, context: &str, question: &str) -> String {
        let mut structured = String::new();

        structured.push_str("🔧 REFINEMENT CONTEXT:\n");
        structured.push_str(context);

        // If repository context is present, emphasize maintaining focus
        if context.contains("CRITICAL REPOSITORY CONTEXT")
            || context.contains("ACTUAL FILE CONTENTS")
        {
            structured.push_str("\n\n⚠️ CRITICAL REFINEMENT REQUIREMENT:\n");
            structured.push_str("The Generator has analyzed a SPECIFIC repository with ACTUAL FILE CONTENTS. You MUST:\n");
            structured
                .push_str("1. Continue analyzing the SAME repository and files mentioned above\n");
            structured.push_str("2. NOT introduce information about other projects\n");
            structured.push_str(
                "3. Base your refinements on the ACTUAL CODE shown, not generic examples\n",
            );
            structured
                .push_str("4. Reference specific files and features from THIS repository only\n");
            structured.push_str("5. Quote actual code snippets when making improvements\n");
        }

        structured.push_str("\n\n🎯 REFINEMENT OBJECTIVES:\n");
        structured.push_str("- Improve clarity and readability\n");
        structured.push_str("- Fix any inaccuracies or ambiguities\n");
        structured.push_str("- Add missing important information\n");
        structured.push_str("- Enhance structure and flow\n");
        structured.push_str("- Optimize for user's specific needs\n");

        if context.contains("symbols:")
            || context.contains("dependencies:")
            || context.contains("Repository Path:")
        {
            structured.push_str("- Leverage repository context for technical accuracy\n");
            structured.push_str(
                "- Ensure all file paths and code references match the actual repository\n",
            );
        }

        if context.contains("TEMPORAL CONTEXT") {
            structured.push_str("- Ensure temporal accuracy and currency of information\n");
        }

        structured
    }

    /// Analyze response quality and identify improvement areas
    pub fn analyze_response_quality(&self, response: &str) -> String {
        let mut analysis = Vec::new();

        // Check length appropriateness
        let word_count = response.split_whitespace().count();
        if word_count < 20 {
            analysis.push("⚠️ Response may be too brief - consider adding more detail".to_string());
        } else if word_count > 1000 {
            analysis
                .push("📝 Response is comprehensive - ensure clarity and structure".to_string());
        }

        // Check for code quality
        if response.contains("```") {
            let code_blocks = response.matches("```").count() / 2;
            let code_analysis = format!(
                "💻 Contains {} code block(s) - verify syntax and best practices",
                code_blocks
            );
            analysis.push(code_analysis);

            if !response.contains("//") && !response.contains("#") && response.contains("fn ") {
                analysis.push("📖 Consider adding code comments for clarity".to_string());
            }
        }

        // Check structure
        if !response.contains('\n') && word_count > 50 {
            analysis
                .push("📋 Consider breaking into paragraphs for better readability".to_string());
        }

        // Check for examples
        if response.len() > 500
            && !response.contains("example")
            && !response.contains("for instance")
        {
            analysis
                .push("💡 Consider adding concrete examples to illustrate concepts".to_string());
        }

        // Check for actionability
        if response.contains("you should") || response.contains("you can") {
            analysis.push("✅ Response provides actionable guidance".to_string());
        } else if word_count > 100 {
            analysis.push("🎯 Consider adding actionable steps or recommendations".to_string());
        }

        if analysis.is_empty() {
            "✅ Response quality appears good - focus on polish and enhancement".to_string()
        } else {
            analysis.join("\n")
        }
    }

    /// Get quality standards for different response types
    fn get_quality_standards_for_type(&self, response_type: &str) -> &'static str {
        match response_type {
            "code" => {
                "- Code must be syntactically correct and follow best practices\n- Include proper error handling and edge cases\n- Add meaningful comments and documentation\n- Optimize for readability and maintainability\n- Suggest alternative approaches when appropriate"
            }
            "explanation" => {
                "- Use clear, accessible language appropriate for the audience\n- Provide logical flow from basic to advanced concepts\n- Include concrete examples and analogies\n- Address potential misconceptions\n- Ensure completeness without overwhelming detail"
            }
            "analysis" => {
                "- Present multiple perspectives and considerations\n- Support claims with evidence or reasoning\n- Identify potential risks, limitations, or trade-offs\n- Provide actionable insights and recommendations\n- Maintain objectivity while being thorough"
            }
            _ => {
                "- Ensure accuracy and completeness\n- Use clear, well-structured language\n- Address the user's specific needs\n- Provide practical, actionable information\n- Maintain helpful and professional tone"
            }
        }
    }

    /// Build improvement instructions based on response type
    pub fn build_improvement_instructions(&self, response_type: &str) -> String {
        match response_type {
            "code" => {
                "Focus on: code clarity, best practices, performance, error handling, and documentation"
            }
            "explanation" => {
                "Focus on: clarity, structure, examples, completeness, and accessibility"
            }
            "analysis" => {
                "Focus on: depth, accuracy, evidence, multiple perspectives, and actionable insights"
            }
            _ => {
                "Focus on: clarity, accuracy, completeness, structure, and helpfulness"
            }
        }.to_string()
    }

    /// Detect response type for targeted refinement
    pub fn detect_response_type(&self, content: &str) -> &'static str {
        if content.contains("```")
            || content.contains("fn ")
            || content.contains("def ")
            || content.contains("class ")
        {
            "code"
        } else if content.contains("because")
            || content.contains("therefore")
            || content.contains("explanation")
        {
            "explanation"
        } else if content.contains("analysis")
            || content.contains("examine")
            || content.contains("compare")
        {
            "analysis"
        } else {
            "general"
        }
    }

    /// Check for specific quality issues
    pub fn identify_quality_issues(&self, content: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for vague language
        let vague_terms = ["might", "could", "possibly", "maybe", "perhaps"];
        let vague_count = vague_terms
            .iter()
            .map(|term| content.matches(term).count())
            .sum::<usize>();

        if vague_count > 3 {
            issues.push("Too much uncertain language - consider more definitive statements where appropriate".to_string());
        }

        // Check for repetition
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut word_freq = std::collections::HashMap::new();
        for word in &words {
            *word_freq.entry(word.to_lowercase()).or_insert(0) += 1;
        }

        let repeated_words: Vec<_> = word_freq
            .iter()
            .filter(|(word, count)| **count > 5 && word.len() > 4)
            .collect();

        if !repeated_words.is_empty() {
            issues.push("Potential repetition detected - consider varying word choice".to_string());
        }

        issues
    }
}
