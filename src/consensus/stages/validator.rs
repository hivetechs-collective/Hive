// Validator Stage - Check accuracy and correctness
// Third stage of the consensus pipeline

use crate::consensus::stages::ConsensusStage;
use crate::consensus::types::{Message, Stage, StagePrompts};
use anyhow::Result;

pub struct ValidatorStage;

impl ConsensusStage for ValidatorStage {
    fn stage(&self) -> Stage {
        Stage::Validator
    }

    fn system_prompt(&self) -> &'static str {
        StagePrompts::validator_system()
    }

    fn build_messages(
        &self,
        question: &str,
        previous_answer: Option<&str>,
        context: Option<&str>,
    ) -> Result<Vec<Message>> {
        let refined_response = previous_answer.unwrap_or("No response to validate");

        // Use intelligent context type if available, otherwise analyze content
        let content_type = self.extract_intelligent_context_type(context)
            .unwrap_or_else(|| self.analyze_content_type(refined_response));

        let mut messages = vec![Message {
            role: "system".to_string(),
            content: self
                .build_enhanced_validation_system_prompt(question, refined_response, content_type)
                .to_string(),
        }];

        // Add validation criteria
        messages.push(Message {
            role: "system".to_string(),
            content: self.get_validation_criteria().to_string(),
        });

        // Add enhanced context for validation
        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: self.structure_validation_context(ctx, question),
            });
        }

        // Perform automated checks first (but skip for academic/general knowledge questions)
        if !matches!(content_type, "academic" | "general_knowledge" | "computer_science") {
            let validation_report = self.perform_automated_validation(refined_response);
            if !validation_report.is_empty() {
                messages.push(Message {
                    role: "system".to_string(),
                    content: format!("AUTOMATED VALIDATION REPORT:\n{}", validation_report),
                });
            }
        }

        // Add the question and refined response to validate
        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "ORIGINAL QUESTION:\n{}\n\nEnhanced analysis from Refiner:\n{}\n\nVALIDATION TASKS:\n{}",
                question,
                refined_response,
                self.get_validation_tasks_for_content_type(content_type)
            ),
        });

        Ok(messages)
    }
}

impl ValidatorStage {
    pub fn new() -> Self {
        Self
    }

    /// Build enhanced system prompt for validation with intelligent context awareness
    pub fn build_enhanced_validation_system_prompt(&self, question: &str, response: &str, content_type: &str) -> String {
        let base_prompt = StagePrompts::validator_system();

        format!(
            "{}\n\nVALIDATION FOCUS: This is a {} response. Apply specialized validation appropriate for this content type.\n\nVALIDATION OBJECTIVES:\n{}",
            base_prompt,
            content_type,
            self.get_validation_objectives_for_type(content_type)
        )
    }
    
    /// Build enhanced system prompt for validation (legacy method for compatibility)
    pub fn build_validation_system_prompt(&self, question: &str, response: &str) -> String {
        let content_type = self.analyze_content_type(response);
        self.build_enhanced_validation_system_prompt(question, response, content_type)
    }
    
    /// Extract intelligent context guidance from system messages
    fn extract_intelligent_context_type(&self, context: Option<&str>) -> Option<&'static str> {
        if let Some(ctx) = context {
            if ctx.contains("This is an academic/scientific question") {
                Some("academic")
            } else if ctx.contains("This is a general knowledge question") {
                Some("general_knowledge")
            } else if ctx.contains("This is a general programming question") {
                Some("programming")
            } else if ctx.contains("This question is repository-specific") {
                Some("repository")
            } else if ctx.contains("This is a computer science theory question") {
                Some("computer_science")
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Structure context specifically for validation
    pub fn structure_validation_context(&self, context: &str, question: &str) -> String {
        let mut structured = String::new();

        // Check if this is repository context
        if context.contains("CRITICAL REPOSITORY CONTEXT") {
            structured.push_str("⚠️ CRITICAL VALIDATION REQUIREMENT:\n");
            structured.push_str("The previous stages analyzed a SPECIFIC repository. During validation:\n");
            structured.push_str("1. VALIDATE information against the SAME repository mentioned in the context\n");
            structured.push_str("2. DO NOT validate against assumptions from other projects\n");
            structured.push_str("3. ENSURE all file paths, code references, and features match THIS repository\n");
            structured.push_str("4. FLAG any information that doesn't match the actual repository structure\n\n");
        }

        // Check if this is memory context (authoritative knowledge from curator)
        if context.contains("## Memory Context") || context.contains("## Recent Context") {
            structured.push_str("🧠 AUTHORITATIVE MEMORY CONTEXT:\n");
            structured.push_str(context);
            structured.push_str("\n\n⚡ CRITICAL: The above memory context contains VALIDATED CURATOR ANSWERS from previous conversations. ");
            structured.push_str("These are the SOURCE OF TRUTH. Validate consistency with this authoritative knowledge. ");
            structured
                .push_str("Flag any contradictions while respecting the established facts.\n");
        } else {
            structured.push_str("🔍 VALIDATION REFERENCE CONTEXT:\n");
            structured.push_str(context);
        }

        structured.push_str("\n\n🎯 VALIDATION INSTRUCTIONS:\n");
        structured.push_str("- Cross-reference information against provided context\n");
        structured.push_str("- Verify technical accuracy using repository data\n");
        structured.push_str("- Check temporal accuracy for current information\n");
        structured.push_str("- Ensure consistency with project patterns and conventions\n");
        structured.push_str("- Validate security and safety considerations\n");

        if context.contains("## Memory Context") || context.contains("## Recent Context") {
            structured.push_str("- Ensure consistency with authoritative curator knowledge\n");
            structured.push_str("- Validate new information aligns with established facts\n");
        }

        if context.contains("symbols:") || context.contains("dependencies:") || context.contains("Repository Path:") {
            structured.push_str("- Verify code suggestions match actual repository structure\n");
            structured.push_str("- Check that referenced symbols and imports are valid\n");
            structured.push_str("- VALIDATE that all technical details match the actual repository\n");
            structured.push_str("- DO NOT accept generic assumptions about the codebase\n");
        }

        if context.contains("TEMPORAL CONTEXT") {
            structured.push_str("- Validate currency of information and version references\n");
            structured.push_str("- Ensure dates and timing information are accurate\n");
        }

        structured
    }

    /// Perform automated validation checks
    pub fn perform_automated_validation(&self, response: &str) -> String {
        let mut issues = Vec::new();

        // Basic validation checks
        let basic_issues = self.basic_validation_checks(response);
        issues.extend(basic_issues);

        // Security validation
        let security_issues = self.security_validation_checks(response);
        issues.extend(security_issues);

        // Code validation if applicable
        if response.contains("```") {
            let code_issues = self.code_validation_checks(response);
            issues.extend(code_issues);
        }

        // Format validation
        let format_issues = self.format_validation_checks(response);
        issues.extend(format_issues);

        if issues.is_empty() {
            "✅ Automated validation passed - no obvious issues detected".to_string()
        } else {
            format!("⚠️ ISSUES DETECTED:\n{}", issues.join("\n"))
        }
    }

    /// Get validation tasks specific to content type
    pub fn get_validation_tasks_for_content(&self, response: &str) -> String {
        let content_type = self.analyze_content_type(response);

        match content_type {
            "academic" => {
                "1. Verify scientific accuracy and theoretical concepts\n2. Check mathematical formulations and equations\n3. Validate examples against established scientific principles\n4. Ensure proper academic terminology is used correctly\n5. Check for misconceptions or oversimplifications\n6. Verify that pseudocode/examples support theoretical concepts without code validation\n7. Ensure no contamination from programming or repository contexts"
            }
            "general_knowledge" => {
                "1. Verify factual accuracy of all general information\n2. Check for balanced and unbiased presentation\n3. Validate examples are appropriate and correct\n4. Ensure information is current and relevant\n5. Check for potential misconceptions\n6. Verify accessibility for general audience\n7. Ensure no technical or repository contamination"
            }
            "programming" => {
                "1. Verify programming concepts and best practices\n2. Check code examples for general applicability\n3. Validate security considerations in code\n4. Ensure examples are language-appropriate\n5. Check for proper error handling patterns\n6. Verify concepts apply broadly, not to specific projects\n7. Avoid repository-specific assumptions"
            }
            "repository" => {
                "1. Verify information matches the specific repository\n2. Check file paths and references are accurate\n3. Validate code suggestions align with project structure\n4. Ensure recommendations fit project architecture\n5. Cross-reference with actual repository contents\n6. Verify dependencies and imports are valid\n7. Check project-specific conventions are followed"
            }
            "computer_science" => {
                "1. Verify algorithmic and complexity analysis accuracy\n2. Check theoretical concepts and mathematical proofs\n3. Validate data structure explanations\n4. Ensure computational theory is correctly presented\n5. Check for proper CS terminology usage\n6. Verify examples support theoretical understanding\n7. Avoid implementation-specific contamination"
            }
            "code" => {
                "1. Verify code syntax and functionality\n2. Check for security vulnerabilities\n3. Validate best practices and patterns\n4. Ensure error handling is present\n5. Verify documentation and comments\n6. Check for performance considerations"
            }
            "explanation" => {
                "1. Verify factual accuracy of all statements\n2. Check logical flow and reasoning\n3. Ensure examples are correct and relevant\n4. Validate that all concepts are properly explained\n5. Check for potential misconceptions\n6. Ensure accessibility and clarity"
            }
            "analysis" => {
                "1. Verify data accuracy and sources\n2. Check reasoning and logical consistency\n3. Validate conclusions against evidence\n4. Ensure multiple perspectives are considered\n5. Check for bias or incomplete analysis\n6. Verify actionability of recommendations"
            }
            _ => {
                "1. Verify all factual claims\n2. Check for completeness and accuracy\n3. Ensure clarity and consistency\n4. Validate helpfulness and relevance\n5. Check for potential issues or errors\n6. Ensure appropriate tone and formatting"
            }
        }.to_string()
    }
    
    /// Get validation tasks for a specific content type (intelligent context aware)
    pub fn get_validation_tasks_for_content_type(&self, content_type: &str) -> String {
        match content_type {
            "academic" => {
                "1. Verify scientific accuracy and theoretical concepts\n2. Check mathematical formulations and equations\n3. Validate examples against established scientific principles\n4. Ensure proper academic terminology is used correctly\n5. Check for misconceptions or oversimplifications\n6. Verify that pseudocode/examples support theoretical concepts without code validation\n7. Ensure no contamination from programming or repository contexts"
            }
            "general_knowledge" => {
                "1. Verify factual accuracy of all general information\n2. Check for balanced and unbiased presentation\n3. Validate examples are appropriate and correct\n4. Ensure information is current and relevant\n5. Check for potential misconceptions\n6. Verify accessibility for general audience\n7. Ensure no technical or repository contamination"
            }
            "programming" => {
                "1. Verify programming concepts and best practices\n2. Check code examples for general applicability\n3. Validate security considerations in code\n4. Ensure examples are language-appropriate\n5. Check for proper error handling patterns\n6. Verify concepts apply broadly, not to specific projects\n7. Avoid repository-specific assumptions"
            }
            "repository" => {
                "1. Verify information matches the specific repository\n2. Check file paths and references are accurate\n3. Validate code suggestions align with project structure\n4. Ensure recommendations fit project architecture\n5. Cross-reference with actual repository contents\n6. Verify dependencies and imports are valid\n7. Check project-specific conventions are followed"
            }
            "computer_science" => {
                "1. Verify algorithmic and complexity analysis accuracy\n2. Check theoretical concepts and mathematical proofs\n3. Validate data structure explanations\n4. Ensure computational theory is correctly presented\n5. Check for proper CS terminology usage\n6. Verify examples support theoretical understanding\n7. Avoid implementation-specific contamination"
            }
            "code" => {
                "1. Verify code syntax and functionality\n2. Check for security vulnerabilities\n3. Validate best practices and patterns\n4. Ensure error handling is present\n5. Verify documentation and comments\n6. Check for performance considerations"
            }
            "explanation" => {
                "1. Verify factual accuracy of all statements\n2. Check logical flow and reasoning\n3. Ensure examples are correct and relevant\n4. Validate that all concepts are properly explained\n5. Check for potential misconceptions\n6. Ensure accessibility and clarity"
            }
            "analysis" => {
                "1. Verify data accuracy and sources\n2. Check reasoning and logical consistency\n3. Validate conclusions against evidence\n4. Ensure multiple perspectives are considered\n5. Check for bias or incomplete analysis\n6. Verify actionability of recommendations"
            }
            _ => {
                "1. Verify all factual claims\n2. Check for completeness and accuracy\n3. Ensure clarity and consistency\n4. Validate helpfulness and relevance\n5. Check for potential issues or errors\n6. Ensure appropriate tone and formatting"
            }
        }.to_string()
    }

    /// Analyze content type for targeted validation using intelligent context guidance
    fn analyze_content_type(&self, response: &str) -> &'static str {
        // First check if we have intelligent context guidance in the response
        if response.contains("🧠 INTELLIGENT CONTEXT GUIDANCE:") {
            // Extract the intelligent context decision from the system message
            if response.contains("This is an academic/scientific question") {
                return "academic";
            } else if response.contains("This is a general knowledge question") {
                return "general_knowledge";
            } else if response.contains("This is a general programming question") {
                return "programming";
            } else if response.contains("This question is repository-specific") {
                return "repository";
            } else if response.contains("This is a computer science theory question") {
                return "computer_science";
            }
        }
        
        // Fallback to content-based analysis only if no intelligent guidance
        if response.contains("```") || response.contains("function") || response.contains("class ")
        {
            "code"
        } else if response.contains("explanation")
            || response.contains("because")
            || response.contains("therefore")
        {
            "explanation"
        } else if response.contains("analysis")
            || response.contains("data")
            || response.contains("metrics")
        {
            "analysis"
        } else {
            "general"
        }
    }

    /// Get validation objectives for different content types
    fn get_validation_objectives_for_type(&self, content_type: &str) -> &'static str {
        match content_type {
            "academic" => {
                "- Verify scientific accuracy and theoretical concepts\n- Check mathematical formulations and equations\n- Validate examples against established scientific principles\n- Ensure proper citation of scientific concepts\n- Avoid contamination from programming or repository contexts\n- Focus on academic rigor and theoretical understanding"
            }
            "general_knowledge" => {
                "- Verify factual accuracy of general information\n- Check for balanced and unbiased presentation\n- Validate examples are appropriate and correct\n- Ensure information is current and relevant\n- Avoid technical programming or repository contamination\n- Focus on broad educational value"
            }
            "programming" => {
                "- Verify programming concepts and best practices\n- Check code examples for correctness\n- Validate general applicability across projects\n- Ensure security considerations are addressed\n- Avoid repository-specific assumptions\n- Focus on widely applicable programming knowledge"
            }
            "repository" => {
                "- Verify information matches the specific repository context\n- Check code suggestions align with project structure\n- Validate file paths and references are accurate\n- Ensure recommendations fit the project's architecture\n- Cross-reference with actual repository contents\n- Focus on project-specific accuracy"
            }
            "computer_science" => {
                "- Verify algorithmic and theoretical accuracy\n- Check complexity analysis and mathematical proofs\n- Validate data structure explanations\n- Ensure theoretical concepts are correctly presented\n- Avoid implementation-specific details\n- Focus on computer science theory and principles"
            }
            "code" => {
                "- Ensure code compiles and runs correctly\n- Verify security best practices\n- Check for proper error handling\n- Validate performance considerations\n- Ensure code follows project conventions"
            }
            "explanation" => {
                "- Verify factual accuracy of all statements\n- Ensure logical consistency throughout\n- Check that examples are correct and helpful\n- Validate completeness of the explanation\n- Ensure accessibility for the target audience"
            }
            "analysis" => {
                "- Verify data accuracy and methodology\n- Check reasoning and conclusions\n- Ensure balanced perspective\n- Validate actionability of insights\n- Check for potential biases or gaps"
            }
            _ => {
                "- Verify factual accuracy\n- Ensure completeness and relevance\n- Check for clarity and consistency\n- Validate helpfulness to the user\n- Ensure appropriate tone and format"
            }
        }
    }

    /// Perform security validation checks
    fn security_validation_checks(&self, response: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for potential security issues in code
        if response.contains("```") {
            let security_patterns = [
                ("eval(", "Use of eval() function - potential security risk"),
                ("exec(", "Use of exec() function - potential security risk"),
                (
                    "system(",
                    "Use of system() function - validate security implications",
                ),
                (
                    "unsafe {",
                    "Unsafe Rust code block - ensure necessity and safety",
                ),
                (
                    "password",
                    "Password mentioned - ensure no hardcoded credentials",
                ),
                ("api_key", "API key mentioned - ensure no hardcoded secrets"),
                ("token", "Token mentioned - ensure no hardcoded credentials"),
            ];

            for (pattern, warning) in &security_patterns {
                if response.to_lowercase().contains(&pattern.to_lowercase()) {
                    issues.push(format!("🔒 Security concern: {}", warning));
                }
            }
        }

        // Check for sensitive information exposure
        let sensitive_patterns = [
            (r"\b\d{16}\b", "Potential credit card number"),
            (r"\b\d{3}-\d{2}-\d{4}\b", "Potential SSN pattern"),
            (
                r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}",
                "Email address exposed",
            ),
        ];

        for (pattern, warning) in &sensitive_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(response) {
                issues.push(format!("🔐 Privacy concern: {}", warning));
            }
        }

        issues
    }

    /// Perform code-specific validation
    fn code_validation_checks(&self, response: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Extract code blocks and validate
        let code_blocks: Vec<&str> = response.split("```").collect();
        for (i, block) in code_blocks.iter().enumerate() {
            if i % 2 == 1 {
                // Odd indices are code blocks
                let lines: Vec<&str> = block.lines().collect();
                if lines.is_empty() {
                    continue;
                }

                let language = lines[0].trim();
                let code = lines[1..].join("\n");

                if let Err(e) = self.validate_code_syntax(&code, language) {
                    issues.push(format!("💻 Code syntax issue: {}", e));
                }
            }
        }

        issues
    }

    /// Perform format validation
    fn format_validation_checks(&self, response: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Check markdown formatting
        let code_block_count = response.matches("```").count();
        if code_block_count % 2 != 0 {
            issues.push("📝 Unclosed code block detected".to_string());
        }

        // Check for broken links (basic check)
        if response.contains("](") && !response.contains("](http") && !response.contains("](#") {
            issues.push("🔗 Potentially broken markdown link detected".to_string());
        }

        // Check for proper heading structure
        if response.contains("##") && !response.contains("# ") {
            issues.push("📋 Consider adding a main heading (# ) for better structure".to_string());
        }

        issues
    }

    /// Get validation criteria based on content type
    fn get_validation_criteria(&self) -> &'static str {
        "Validation Criteria:
1. FACTUAL ACCURACY: Verify all facts, figures, and claims
2. TECHNICAL CORRECTNESS: Ensure code examples work and follow best practices
3. COMPLETENESS: Check if the response fully addresses the question
4. CLARITY: Ensure explanations are clear and unambiguous
5. SAFETY: Verify no harmful, biased, or inappropriate content
6. CONSISTENCY: Ensure internal consistency throughout the response
7. SECURITY: Check for potential security vulnerabilities or sensitive data exposure
8. REPOSITORY ACCURACY: Ensure code suggestions match actual project structure

If you find errors, provide corrections. If the response is accurate, enhance it with validation notes."
    }

    /// Perform basic validation checks
    pub fn basic_validation_checks(&self, content: &str) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for incomplete code blocks
        let code_blocks = content.matches("```").count();
        if code_blocks % 2 != 0 {
            issues.push("Unclosed code block detected".to_string());
        }

        // Check for placeholder text
        let placeholders = ["TODO", "FIXME", "XXX", "[INSERT", "[PLACEHOLDER"];
        for placeholder in &placeholders {
            if content.contains(placeholder) {
                issues.push(format!("Placeholder text '{}' found", placeholder));
            }
        }

        // Check for broken markdown links
        if content.contains("](") && !content.contains("](http") && !content.contains("](#") {
            issues.push("Potentially broken markdown link detected".to_string());
        }

        issues
    }

    /// Validate code snippets for basic syntax
    pub fn validate_code_syntax(&self, code: &str, language: &str) -> Result<()> {
        // Basic validation - can be extended with actual parsers
        match language {
            "rust" => {
                if code.contains("fn ") && !code.contains("{") {
                    anyhow::bail!("Rust function missing opening brace");
                }
            }
            "python" => {
                if code.contains("def ") && !code.contains(":") {
                    anyhow::bail!("Python function missing colon");
                }
            }
            _ => {}
        }
        Ok(())
    }
}
