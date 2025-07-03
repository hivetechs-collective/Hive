//! MCP prompt management system
//!
//! Provides centralized prompt templates and context injection

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde_json::Value;
use chrono::{DateTime, Utc};

/// Prompt template manager
pub struct PromptManager {
    templates: HashMap<String, PromptTemplate>,
    context_injectors: Vec<Box<dyn ContextInjector + Send + Sync>>,
}

/// Prompt template definition
#[derive(Clone)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template: String,
    pub required_params: Vec<String>,
    pub optional_params: Vec<String>,
    pub language_specific: bool,
    pub context_aware: bool,
}

/// Context injection trait
pub trait ContextInjector {
    fn inject_context(&self, prompt: &str, context: &PromptContext) -> Result<String>;
    fn get_name(&self) -> &str;
}

/// Prompt context for injection
#[derive(Clone)]
pub struct PromptContext {
    pub language: Option<String>,
    pub project_path: Option<String>,
    pub user_preferences: HashMap<String, Value>,
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub tool_name: String,
    pub additional_context: HashMap<String, Value>,
}

impl PromptManager {
    /// Create new prompt manager
    pub fn new() -> Self {
        let mut manager = Self {
            templates: HashMap::new(),
            context_injectors: Vec::new(),
        };

        // Register default templates
        manager.register_default_templates();
        
        // Register default context injectors
        manager.register_default_injectors();

        manager
    }

    /// Register default prompt templates
    fn register_default_templates(&mut self) {
        // Code analysis template
        self.register_template(PromptTemplate {
            id: "analyze_code".to_string(),
            name: "Code Analysis".to_string(),
            description: "Comprehensive code analysis template".to_string(),
            template: r#"Analyze this {{language}} code with focus on {{focus}}:

```{{language}}
{{code}}
```

{{#if context}}
Context: {{context}}
{{/if}}

Please provide:
1. Code structure and organization
2. Quality assessment and metrics
3. Potential issues and improvements
4. Best practices compliance
5. Maintainability recommendations

{{#if language_specific_notes}}
Language-specific considerations for {{language}}:
{{language_specific_notes}}
{{/if}}

Current timestamp: {{timestamp}}
Analysis requested by: {{session_id}}"#.to_string(),
            required_params: vec!["code".to_string()],
            optional_params: vec!["focus".to_string(), "context".to_string()],
            language_specific: true,
            context_aware: true,
        });

        // Debug assistance template
        self.register_template(PromptTemplate {
            id: "debug_code".to_string(),
            name: "Debug Assistant".to_string(),
            description: "Systematic debugging assistance template".to_string(),
            template: r#"Debug this {{language}} code systematically:

**Code with Issue:**
```{{language}}
{{code}}
```

**Error/Issue:**
{{error_message}}

{{#if stack_trace}}
**Stack Trace:**
```
{{stack_trace}}
```
{{/if}}

{{#if context}}
**Additional Context:**
{{context}}
{{/if}}

**Debugging Analysis Required:**
1. Root cause identification
2. Step-by-step troubleshooting approach
3. Potential fixes with explanations
4. Prevention strategies
5. Testing recommendations

{{temporal_context}}

Please be thorough and systematic in your analysis."#.to_string(),
            required_params: vec!["code".to_string(), "error_message".to_string()],
            optional_params: vec!["stack_trace".to_string(), "context".to_string()],
            language_specific: true,
            context_aware: true,
        });

        // Code generation template
        self.register_template(PromptTemplate {
            id: "generate_code".to_string(),
            name: "Code Generator".to_string(),
            description: "AI-powered code generation template".to_string(),
            template: r#"Generate {{language}} code for the following requirements:

**Requirements:**
{{requirements}}

{{#if existing_code}}
**Existing Code Context:**
```{{language}}
{{existing_code}}
```
{{/if}}

{{#if constraints}}
**Constraints:**
{{#each constraints}}
- {{this}}
{{/each}}
{{/if}}

**Generation Guidelines:**
- Follow {{language}} best practices and idioms
- Include comprehensive error handling
- Add appropriate comments and documentation
- Ensure code is testable and maintainable
- Consider performance implications

{{#if style_preferences}}
**Style Preferences:**
{{style_preferences}}
{{/if}}

{{temporal_context}}

Please provide complete, production-ready code with explanations."#.to_string(),
            required_params: vec!["requirements".to_string()],
            optional_params: vec!["existing_code".to_string(), "constraints".to_string(), "style_preferences".to_string()],
            language_specific: true,
            context_aware: true,
        });

        // Security analysis template
        self.register_template(PromptTemplate {
            id: "security_scan".to_string(),
            name: "Security Scanner".to_string(),
            description: "Security vulnerability analysis template".to_string(),
            template: r#"Perform comprehensive security analysis of this {{language}} code:

```{{language}}
{{code}}
```

**Scan Depth:** {{scan_depth}}

{{#if framework}}
**Framework Context:** {{framework}}
{{/if}}

**Security Analysis Required:**
1. **Vulnerability Assessment:**
   - Input validation issues
   - Authentication/authorization flaws
   - Data exposure risks
   - Injection vulnerabilities
   - Cryptographic weaknesses

2. **Risk Categorization:**
   - Critical (immediate action required)
   - High (urgent attention needed)
   - Medium (should be addressed)
   - Low (minor improvements)

3. **Remediation Recommendations:**
   - Specific fixes for each issue
   - Code examples for secure implementations
   - Security best practices
   - Framework-specific security guidelines

4. **Compliance Check:**
   - OWASP Top 10 compliance
   - Industry standard adherence
   - Regulatory requirements (if applicable)

{{security_context}}

Please be thorough and provide actionable security recommendations."#.to_string(),
            required_params: vec!["code".to_string()],
            optional_params: vec!["scan_depth".to_string(), "framework".to_string()],
            language_specific: true,
            context_aware: true,
        });

        // Refactoring template
        self.register_template(PromptTemplate {
            id: "refactor_code".to_string(),
            name: "Code Refactorer".to_string(),
            description: "Smart refactoring guidance template".to_string(),
            template: r#"Refactor this {{language}} code using {{refactor_type}} approach:

**Original Code:**
```{{language}}
{{code}}
```

{{#if target}}
**Refactoring Target:** {{target}}
{{/if}}

**Refactoring Objectives:**
1. Improve code maintainability
2. Enhance readability and clarity
3. Optimize performance where applicable
4. Follow {{language}} best practices
5. Maintain existing functionality

**Required Deliverables:**
1. **Refactored Code:** Complete implementation with improvements
2. **Change Analysis:** Detailed explanation of modifications
3. **Benefits Assessment:** How refactoring improves the code
4. **Risk Evaluation:** Potential issues and mitigation strategies
5. **Testing Strategy:** Approach to verify refactored code

{{#if complexity_constraints}}
**Complexity Constraints:**
{{complexity_constraints}}
{{/if}}

{{refactoring_context}}

Please ensure the refactored code maintains compatibility while improving quality."#.to_string(),
            required_params: vec!["code".to_string(), "refactor_type".to_string()],
            optional_params: vec!["target".to_string(), "complexity_constraints".to_string()],
            language_specific: true,
            context_aware: true,
        });

        // Test generation template
        self.register_template(PromptTemplate {
            id: "generate_tests".to_string(),
            name: "Test Generator".to_string(),
            description: "Comprehensive test suite generation template".to_string(),
            template: r#"Generate comprehensive tests for this {{language}} code:

**Code to Test:**
```{{language}}
{{code}}
```

**Testing Framework:** {{test_framework}}
**Coverage Target:** {{coverage_target}}%

{{#if test_types}}
**Test Types Required:**
{{#each test_types}}
- {{this}}
{{/each}}
{{/if}}

**Test Generation Requirements:**
1. **Unit Tests:**
   - Test all public methods/functions
   - Cover edge cases and boundary conditions
   - Include error scenarios and exception handling
   - Mock external dependencies appropriately

2. **Test Structure:**
   - Clear test organization and naming
   - Setup and teardown methods
   - Descriptive test descriptions
   - Parameterized tests where applicable

3. **Coverage Analysis:**
   - Identify untested code paths
   - Suggest additional test scenarios
   - Assess overall test effectiveness

4. **Test Data:**
   - Provide appropriate test fixtures
   - Include both valid and invalid inputs
   - Consider performance test data

{{testing_context}}

Please generate production-ready tests with comprehensive coverage."#.to_string(),
            required_params: vec!["code".to_string()],
            optional_params: vec!["test_framework".to_string(), "coverage_target".to_string(), "test_types".to_string()],
            language_specific: true,
            context_aware: true,
        });
    }

    /// Register default context injectors
    fn register_default_injectors(&mut self) {
        self.context_injectors.push(Box::new(TemporalContextInjector));
        self.context_injectors.push(Box::new(LanguageContextInjector));
        self.context_injectors.push(Box::new(SecurityContextInjector));
        self.context_injectors.push(Box::new(RefactoringContextInjector));
        self.context_injectors.push(Box::new(TestingContextInjector));
    }

    /// Register a prompt template
    pub fn register_template(&mut self, template: PromptTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get a prompt template by ID
    pub fn get_template(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    /// Generate prompt from template
    pub fn generate_prompt(
        &self,
        template_id: &str,
        params: HashMap<String, Value>,
        context: PromptContext,
    ) -> Result<String> {
        let template = self.get_template(template_id)
            .ok_or_else(|| anyhow!("Template not found: {}", template_id))?;

        // Validate required parameters
        for required_param in &template.required_params {
            if !params.contains_key(required_param) {
                return Err(anyhow!("Missing required parameter: {}", required_param));
            }
        }

        // Start with base template
        let mut prompt = template.template.clone();

        // Replace template variables
        for (key, value) in &params {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                _ => value.to_string(),
            };
            prompt = prompt.replace(&placeholder, &value_str);
        }

        // Apply context injectors
        for injector in &self.context_injectors {
            prompt = injector.inject_context(&prompt, &context)?;
        }

        // Clean up any remaining placeholders
        prompt = self.clean_template_placeholders(prompt);

        Ok(prompt)
    }

    /// Clean up template placeholders
    fn clean_template_placeholders(&self, prompt: String) -> String {
        // Remove conditional blocks that weren't processed
        let mut cleaned = prompt;
        
        // Simple cleanup - in a full implementation, you'd use a proper template engine
        cleaned = cleaned.replace("{{#if context}}", "");
        cleaned = cleaned.replace("{{/if}}", "");
        cleaned = cleaned.replace("{{#each constraints}}", "");
        cleaned = cleaned.replace("{{/each}}", "");
        
        // Remove empty template variables
        let re = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
        cleaned = re.replace_all(&cleaned, "").to_string();

        cleaned
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }
}

/// Temporal context injector
struct TemporalContextInjector;

impl ContextInjector for TemporalContextInjector {
    fn inject_context(&self, prompt: &str, context: &PromptContext) -> Result<String> {
        let temporal_info = format!(
            "**Current Context:**\n\
            - Timestamp: {}\n\
            - Session: {}\n\
            - Tool: {}",
            context.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            context.session_id,
            context.tool_name
        );

        Ok(prompt.replace("{{temporal_context}}", &temporal_info))
    }

    fn get_name(&self) -> &str {
        "temporal"
    }
}

/// Language-specific context injector
struct LanguageContextInjector;

impl ContextInjector for LanguageContextInjector {
    fn inject_context(&self, prompt: &str, context: &PromptContext) -> Result<String> {
        if let Some(language) = &context.language {
            let language_notes = get_language_specific_notes(language);
            let updated = prompt.replace("{{language_specific_notes}}", &language_notes);
            Ok(updated)
        } else {
            Ok(prompt.replace("{{language_specific_notes}}", ""))
        }
    }

    fn get_name(&self) -> &str {
        "language"
    }
}

/// Security context injector
struct SecurityContextInjector;

impl ContextInjector for SecurityContextInjector {
    fn inject_context(&self, prompt: &str, context: &PromptContext) -> Result<String> {
        let security_context = if let Some(language) = &context.language {
            format!(
                "**Security Context for {}:**\n\
                - Common vulnerabilities in {}\n\
                - Framework-specific security considerations\n\
                - Industry security standards compliance\n\
                - Recent security advisories and CVEs",
                language, language
            )
        } else {
            "**General Security Context:**\n\
            - OWASP Top 10 vulnerabilities\n\
            - Common security anti-patterns\n\
            - Secure coding practices".to_string()
        };

        Ok(prompt.replace("{{security_context}}", &security_context))
    }

    fn get_name(&self) -> &str {
        "security"
    }
}

/// Refactoring context injector
struct RefactoringContextInjector;

impl ContextInjector for RefactoringContextInjector {
    fn inject_context(&self, prompt: &str, context: &PromptContext) -> Result<String> {
        let refactoring_context = if let Some(language) = &context.language {
            format!(
                "**Refactoring Context for {}:**\n\
                - Language-specific refactoring patterns\n\
                - Modern {} idioms and best practices\n\
                - Performance optimization opportunities\n\
                - Tooling support for {} refactoring",
                language, language, language
            )
        } else {
            "**General Refactoring Context:**\n\
            - Universal refactoring principles\n\
            - Code smell identification\n\
            - Design pattern opportunities".to_string()
        };

        Ok(prompt.replace("{{refactoring_context}}", &refactoring_context))
    }

    fn get_name(&self) -> &str {
        "refactoring"
    }
}

/// Testing context injector
struct TestingContextInjector;

impl ContextInjector for TestingContextInjector {
    fn inject_context(&self, prompt: &str, context: &PromptContext) -> Result<String> {
        let testing_context = if let Some(language) = &context.language {
            format!(
                "**Testing Context for {}:**\n\
                - Popular {} testing frameworks\n\
                - Language-specific testing patterns\n\
                - Mocking and stubbing strategies\n\
                - Performance and integration testing approaches",
                language, language
            )
        } else {
            "**General Testing Context:**\n\
            - Universal testing principles\n\
            - Test-driven development practices\n\
            - Testing pyramid concepts".to_string()
        };

        Ok(prompt.replace("{{testing_context}}", &testing_context))
    }

    fn get_name(&self) -> &str {
        "testing"
    }
}

/// Get language-specific notes
fn get_language_specific_notes(language: &str) -> String {
    match language.to_lowercase().as_str() {
        "rust" => "Rust-specific considerations: memory safety, ownership, lifetimes, error handling with Result, async/await patterns".to_string(),
        "python" => "Python-specific considerations: PEP 8 compliance, duck typing, context managers, list comprehensions, decorators".to_string(),
        "javascript" | "typescript" => "JavaScript/TypeScript considerations: ES6+ features, async/await, closures, prototypal inheritance, type safety".to_string(),
        "java" => "Java considerations: object-oriented design, exception handling, generics, stream API, memory management".to_string(),
        "c++" => "C++ considerations: RAII, smart pointers, const correctness, template metaprogramming, performance optimization".to_string(),
        "go" => "Go considerations: goroutines, channels, interfaces, error handling patterns, simplicity principles".to_string(),
        _ => "General programming considerations: code clarity, maintainability, performance, security, testability".to_string(),
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}