// Curator Stage - Final polish and formatting
// Fourth and final stage of the consensus pipeline

use crate::consensus::stages::ConsensusStage;
use crate::consensus::types::{Message, Stage, StagePrompts};
use anyhow::Result;

pub struct CuratorStage;

impl ConsensusStage for CuratorStage {
    fn stage(&self) -> Stage {
        Stage::Curator
    }

    fn system_prompt(&self) -> &'static str {
        StagePrompts::curator_system()
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
                .build_curation_system_prompt(question, validated_response)
                .to_string(),
        }];

        // Add enhanced curation guidelines
        messages.push(Message {
            role: "system".to_string(),
            content: self.get_curation_guidelines().to_string(),
        });

        // Add context with curation-specific instructions
        if let Some(ctx) = context {
            messages.push(Message {
                role: "system".to_string(),
                content: self.structure_curation_context(ctx, question),
            });
        }

        // Analyze the validated response for curation opportunities
        let curation_analysis = self.analyze_curation_opportunities(validated_response);
        messages.push(Message {
            role: "system".to_string(),
            content: format!("CURATION ANALYSIS:\n{}", curation_analysis),
        });

        // Add the question and validated response with specific curation tasks
        messages.push(Message {
            role: "user".to_string(),
            content: format!(
                "ORIGINAL QUESTION:\n{}\n\nValidated analysis from Validator:\n{}\n\nCURATION OBJECTIVES:\n{}",
                question,
                validated_response,
                self.get_curation_objectives_for_content(validated_response)
            ),
        });

        Ok(messages)
    }
}

impl CuratorStage {
    pub fn new() -> Self {
        Self
    }

    /// Build enhanced system prompt for curation
    pub fn build_curation_system_prompt(&self, question: &str, response: &str) -> String {
        let base_prompt = StagePrompts::curator_system();
        let content_type = self.analyze_content_type(response);
        let response_length = response.len();

        let length_guidance = if response_length < 500 {
            "Focus on clarity and completeness without over-expanding."
        } else if response_length > 2000 {
            "Focus on organization, structure, and executive summary."
        } else {
            "Focus on polish, flow, and presentation optimization."
        };

        format!(
            "{}\n\nCURATION FOCUS: This is a {} response ({} characters). {}\n\nFINAL STAGE OBJECTIVES:\n{}",
            base_prompt,
            content_type,
            response_length,
            length_guidance,
            self.get_final_stage_objectives()
        )
    }

    /// Structure context specifically for curation
    pub fn structure_curation_context(&self, context: &str, question: &str) -> String {
        let mut structured = String::new();

        // Check if this is repository context
        if context.contains("CRITICAL REPOSITORY CONTEXT")
            || context.contains("ACTUAL FILE CONTENTS")
        {
            structured.push_str("⚠️ CRITICAL CURATION REQUIREMENT:\n");
            structured.push_str("All previous stages analyzed a SPECIFIC repository with ACTUAL FILE CONTENTS. In your final curation:\n");
            structured
                .push_str("1. MAINTAIN focus on the SAME repository throughout your answer\n");
            structured.push_str("2. DO NOT introduce information from other projects\n");
            structured
                .push_str("3. ENSURE your final answer accurately describes THIS repository\n");
            structured
                .push_str("4. Use ONLY the actual code and files shown - NO GENERIC EXAMPLES\n");
            structured
                .push_str("5. Create an authoritative answer about THIS specific codebase\n\n");
        }

        // Check if this is memory context (authoritative knowledge from previous curator)
        if context.contains("## Memory Context") || context.contains("## Recent Context") {
            structured.push_str("🧠 AUTHORITATIVE MEMORY CONTEXT:\n");
            structured.push_str(context);
            structured.push_str(
                "\n\n⚡ NOTE: All previous stages have been informed by this curator knowledge. ",
            );
            structured.push_str("Build upon and synthesize their enhanced analyses while maintaining consistency with established facts. ");
            structured.push_str("Your final answer becomes the new authoritative source.\n");
        } else {
            structured.push_str("✨ CURATION CONTEXT:\n");
            structured.push_str(context);
        }

        structured.push_str("\n\n🎯 FINAL POLISH INSTRUCTIONS:\n");
        structured.push_str("- Apply perfect formatting and visual hierarchy\n");
        structured.push_str("- Ensure professional and accessible tone\n");
        structured.push_str("- Optimize for maximum helpfulness and clarity\n");
        structured.push_str("- Add executive summary if content is substantial\n");
        structured.push_str("- Include actionable next steps where appropriate\n");

        if context.contains("## Memory Context") || context.contains("## Recent Context") {
            structured
                .push_str("- Synthesize insights from all stages informed by curator knowledge\n");
            structured.push_str(
                "- Create a comprehensive answer that becomes the new authoritative source\n",
            );
        }

        if context.contains("symbols:")
            || context.contains("dependencies:")
            || context.contains("Repository Path:")
        {
            structured.push_str("- Reference repository context with clear technical guidance\n");
            structured.push_str("- Ensure all technical details match the actual repository\n");
            structured.push_str(
                "- Create an authoritative article about THIS codebase for future reference\n",
            );
        }

        if context.contains("TEMPORAL CONTEXT") {
            structured.push_str("- Highlight currency and recency of information appropriately\n");
        }

        structured
    }

    /// Analyze curation opportunities in validated response
    pub fn analyze_curation_opportunities(&self, response: &str) -> String {
        let mut opportunities = Vec::new();

        // Check structure and organization
        if response.len() > 1000 && !response.contains("## ") {
            opportunities.push("📋 Add section headers for better organization");
        }

        if response.len() > 2000
            && !response.contains("Summary")
            && !response.contains("## Key Points")
        {
            opportunities.push("📝 Consider adding an executive summary");
        }

        // Check formatting
        if response.contains("```") {
            let code_blocks = response.matches("```").count() / 2;
            if code_blocks > 1 {
                opportunities.push("💻 Ensure consistent code block formatting and explanations");
            }
        }

        // Check for action items
        if !response.contains("you can")
            && !response.contains("next steps")
            && !response.contains("recommended")
        {
            opportunities.push("🎯 Consider adding actionable recommendations");
        }

        // Check for examples
        if response.len() > 800
            && !response.contains("example")
            && !response.contains("for instance")
        {
            opportunities.push("💡 Consider adding concrete examples");
        }

        // Check flow and transitions
        let paragraph_count = response.split("\n\n").count();
        if paragraph_count > 5
            && !response.contains("Additionally")
            && !response.contains("Furthermore")
        {
            opportunities.push("🔗 Improve transitions between sections");
        }

        if opportunities.is_empty() {
            "✅ Response structure is excellent - focus on final polish and tone".to_string()
        } else {
            opportunities.join("\n")
        }
    }

    /// Get curation objectives specific to content type
    pub fn get_curation_objectives_for_content(&self, response: &str) -> String {
        let content_type = self.analyze_content_type(response);

        match content_type {
            "code" => {
                "1. Perfect code formatting with syntax highlighting\n2. Clear explanations for each code section\n3. Proper documentation and comments\n4. Usage examples and best practices\n5. Error handling and edge cases\n6. Performance considerations and alternatives"
            }
            "explanation" => {
                "1. Logical flow from basic to advanced concepts\n2. Clear, accessible language for the audience\n3. Relevant examples and analogies\n4. Visual hierarchy with proper headings\n5. Summary of key takeaways\n6. Next steps or further reading"
            }
            "analysis" => {
                "1. Executive summary of key findings\n2. Well-organized presentation of evidence\n3. Clear conclusions and recommendations\n4. Visual separation of different aspects\n5. Actionable insights highlighted\n6. Balanced perspective maintained"
            }
            _ => {
                "1. Perfect formatting and visual appeal\n2. Clear, professional, and helpful tone\n3. Logical organization and flow\n4. Actionable information highlighted\n5. Summary or key points if appropriate\n6. Optimized for user's specific needs"
            }
        }.to_string()
    }

    /// Analyze content type for targeted curation
    fn analyze_content_type(&self, response: &str) -> &'static str {
        if response.contains("```") || response.contains("function") || response.contains("class ")
        {
            "code"
        } else if response.contains("explanation")
            || response.contains("understand")
            || response.contains("concept")
        {
            "explanation"
        } else if response.contains("analysis")
            || response.contains("findings")
            || response.contains("conclusion")
        {
            "analysis"
        } else {
            "general"
        }
    }

    /// Get final stage objectives
    fn get_final_stage_objectives(&self) -> &'static str {
        "As the final stage in the consensus pipeline:\n- Synthesize insights from Generator, Refiner, and Validator\n- Create the most helpful possible response for the user\n- Ensure perfect presentation and accessibility\n- Optimize for immediate actionability\n- Maintain the highest quality standards"
    }

    /// Get curation guidelines
    fn get_curation_guidelines(&self) -> &'static str {
        "Curation Guidelines:
1. FORMATTING: Ensure perfect markdown formatting, proper headings, and clear structure
2. TONE: Maintain a helpful, professional, and accessible tone throughout
3. FLOW: Ensure smooth transitions and logical flow between sections
4. EXAMPLES: Include clear, practical examples where appropriate
5. SUMMARY: Add a brief summary or key takeaways if the response is long
6. ACTIONABLE: Make the response as actionable and practical as possible
7. VISUAL HIERARCHY: Use headers, lists, and formatting to enhance readability
8. CONSISTENCY: Ensure consistent style and terminology throughout

Create a response that directly addresses the user's needs in the most helpful way possible."
    }

    /// Apply comprehensive formatting improvements
    pub fn apply_comprehensive_formatting(&self, content: &str) -> String {
        let mut formatted = content.to_string();

        // Apply all formatting methods
        formatted = self.format_code_blocks(&formatted);
        formatted = self.add_structure(&formatted, "");
        formatted = self.ensure_consistency(&formatted);
        formatted = self.improve_visual_hierarchy(&formatted);
        formatted = self.enhance_readability(&formatted);

        formatted
    }

    /// Improve visual hierarchy
    pub fn improve_visual_hierarchy(&self, content: &str) -> String {
        let mut improved = content.to_string();

        // Ensure proper spacing around headers
        improved = improved.replace("##", "\n## ");
        improved = improved.replace("###", "\n### ");
        improved = improved.replace("\n\n## ", "\n## ");
        improved = improved.replace("\n\n### ", "\n### ");

        // Add proper spacing around lists
        improved = improved.replace("\n-", "\n\n-");
        improved = improved.replace("\n\n\n-", "\n\n-");

        // Ensure proper spacing around code blocks
        improved = improved.replace("```", "\n```");
        improved = improved.replace("\n\n```", "\n```");

        improved
    }

    /// Enhance readability
    pub fn enhance_readability(&self, content: &str) -> String {
        let mut enhanced = content.to_string();

        // Break up long paragraphs
        let lines: Vec<&str> = enhanced.split('\n').collect();
        let mut result = Vec::new();

        for line in lines {
            if line.len() > 500 && !line.starts_with('#') && !line.starts_with('`') {
                // Try to break at sentence boundaries
                let sentences: Vec<&str> = line.split(". ").collect();
                if sentences.len() > 2 {
                    let mid = sentences.len() / 2;
                    let first_half = sentences[..mid].join(". ") + ".";
                    let second_half = sentences[mid..].join(". ");
                    result.push(first_half);
                    result.push("".to_string());
                    result.push(second_half);
                } else {
                    result.push(line.to_string());
                }
            } else {
                result.push(line.to_string());
            }
        }

        result.join("\n")
    }

    /// Format code blocks properly
    pub fn format_code_blocks(&self, content: &str) -> String {
        // Ensure all code blocks have language identifiers
        let mut formatted = content.to_string();

        // Replace generic code blocks with language-specific ones
        formatted = formatted.replace("```\n", "```text\n");

        // Ensure consistent spacing around code blocks
        formatted = formatted.replace("```", "\n```");
        formatted = formatted.replace("\n\n```", "\n```");
        formatted = formatted.replace("```\n\n", "```\n");

        formatted
    }

    /// Add section headers if missing
    pub fn add_structure(&self, content: &str, question: &str) -> String {
        let mut structured = String::new();

        // Add a brief introduction if the response starts abruptly
        if !content.starts_with('#') && !content.starts_with("##") {
            if content.len() > 500 {
                structured.push_str("## Overview\n\n");
            }
        }

        structured.push_str(content);

        // Add a summary section for long responses
        if content.len() > 2000 && !content.contains("## Summary") {
            structured.push_str("\n\n## Summary\n\n");
            structured.push_str("The key points from this response are highlighted above. ");
        }

        structured
    }

    /// Ensure consistent formatting
    pub fn ensure_consistency(&self, content: &str) -> String {
        let mut consistent = content.to_string();

        // Normalize bullet points
        consistent = consistent.replace("•", "-");
        consistent = consistent.replace("*", "-");

        // Ensure consistent header formatting
        consistent = consistent.replace("###", "### ");
        consistent = consistent.replace("##", "## ");
        consistent = consistent.replace("#", "# ");

        // Fix multiple # or spaces
        consistent = consistent.replace("#  ", "# ");
        consistent = consistent.replace("##  ", "## ");
        consistent = consistent.replace("###  ", "### ");

        consistent
    }
}
