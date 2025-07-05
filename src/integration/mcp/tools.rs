//! MCP tool registry and execution
//!
//! Provides AI-powered tools for IDE integration

use super::protocol::{Tool, ToolResult, ToolContent};
use super::tools_advanced::AdvancedToolRegistry;
use super::prompts::{PromptManager, PromptContext};
use super::performance::{PerformanceManager, PerformanceConfig};
use crate::core::config::Config;
use crate::consensus::engine::ConsensusEngine;
use crate::analysis::repository_intelligence::RepositoryAnalyzer;
use crate::commands::analyze::analyze_codebase;

use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use tracing::{info, error};
use chrono::Utc;
use uuid::Uuid;

/// Tool registry for MCP server
pub struct ToolRegistry {
    consensus_engine: Arc<RwLock<ConsensusEngine>>,
    config: Arc<Config>,
    tools: HashMap<String, ToolDefinition>,
    advanced_tools: AdvancedToolRegistry,
    prompt_manager: PromptManager,
    performance_manager: PerformanceManager,
}

/// Tool definition
struct ToolDefinition {
    pub tool: Tool,
    pub handler: ToolHandler,
}

/// Tool handler type
type ToolHandler = Box<dyn Fn(&ToolRegistry, Value) -> tokio::task::JoinHandle<Result<ToolResult>> + Send + Sync>;

impl ToolRegistry {
    /// Create new tool registry
    pub async fn new(
        consensus_engine: Arc<RwLock<ConsensusEngine>>,
        config: Arc<Config>,
    ) -> Result<Self> {
        let advanced_tools = AdvancedToolRegistry::new(
            consensus_engine.clone(),
            config.clone(),
        );

        let prompt_manager = PromptManager::new();
        
        let performance_manager = PerformanceManager::new(PerformanceConfig::default());
        performance_manager.start_monitoring().await?;

        let mut registry = Self {
            consensus_engine,
            config,
            tools: HashMap::new(),
            advanced_tools,
            prompt_manager,
            performance_manager,
        };

        // Register built-in tools
        registry.register_builtin_tools().await?;

        // Register advanced tools
        registry.register_advanced_tools().await?;

        Ok(registry)
    }

    /// Register built-in tools
    async fn register_builtin_tools(&mut self) -> Result<()> {
        // Consensus tools
        self.register_tool(
            "ask_hive",
            "Ask Hive AI a question using multi-model consensus",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question to ask Hive AI"
                    },
                    "context": {
                        "type": "string",
                        "description": "Additional context for the question"
                    }
                },
                "required": ["question"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_ask_hive(args).await
                })
            }),
        );

        self.register_tool(
            "analyze_code",
            "Analyze code files or directories using AI consensus",
            serde_json::json!({
                "type": "object", 
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to file or directory to analyze"
                    },
                    "focus": {
                        "type": "string",
                        "description": "Specific aspect to focus on (security, performance, architecture, etc.)"
                    }
                },
                "required": ["path"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_analyze_code(args).await
                })
            }),
        );

        self.register_tool(
            "explain_code",
            "Explain what code does using AI consensus",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to explain"
                    },
                    "language": {
                        "type": "string", 
                        "description": "Programming language"
                    }
                },
                "required": ["code"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_explain_code(args).await
                })
            }),
        );

        self.register_tool(
            "improve_code",
            "Suggest improvements for code using AI consensus",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to improve"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "focus": {
                        "type": "string",
                        "description": "Area of improvement (performance, readability, security, etc.)"
                    }
                },
                "required": ["code"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_improve_code(args).await
                })
            }),
        );

        self.register_tool(
            "generate_tests",
            "Generate unit tests for code using AI consensus", 
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to generate tests for"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "test_framework": {
                        "type": "string",
                        "description": "Testing framework to use"
                    }
                },
                "required": ["code"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_generate_tests(args).await
                })
            }),
        );

        self.register_tool(
            "repository_summary",
            "Generate a comprehensive summary of the repository",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Repository path"
                    }
                },
                "required": ["path"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_repository_summary(args).await
                })
            }),
        );

        // Planning tools
        self.register_tool(
            "plan_project",
            "Create a strategic plan for implementing features or changes",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "Description of what needs to be planned"
                    },
                    "scope": {
                        "type": "string",
                        "description": "Scope of the plan (feature, refactor, migration, etc.)"
                    },
                    "constraints": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Any constraints or requirements"
                    }
                },
                "required": ["description"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_plan_project(args).await
                })
            }),
        );

        // Code transformation tools
        self.register_tool(
            "transform_code",
            "Apply AI-powered code transformations",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to transform"
                    },
                    "transformation": {
                        "type": "string",
                        "description": "Type of transformation to apply"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    }
                },
                "required": ["code", "transformation"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_transform_code(args).await
                })
            }),
        );

        // Memory access tools
        self.register_tool(
            "search_memory",
            "Search through conversation history and knowledge base",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results",
                        "default": 10
                    },
                    "type": {
                        "type": "string",
                        "description": "Type of memory to search (conversations, knowledge, all)",
                        "default": "all"
                    }
                },
                "required": ["query"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_search_memory(args).await
                })
            }),
        );

        // Analytics tools
        self.register_tool(
            "generate_analytics",
            "Generate analytics reports and insights",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "report_type": {
                        "type": "string",
                        "description": "Type of report (performance, cost, usage, executive)"
                    },
                    "timeframe": {
                        "type": "string",
                        "description": "Time period for the report (day, week, month, quarter)"
                    },
                    "format": {
                        "type": "string",
                        "description": "Output format (summary, detailed, dashboard)",
                        "default": "summary"
                    }
                },
                "required": ["report_type"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_generate_analytics(args).await
                })
            }),
        );

        // Documentation tools
        self.register_tool(
            "generate_docs",
            "Generate documentation for code using AI consensus",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Code to document"
                    },
                    "language": {
                        "type": "string",
                        "description": "Programming language"
                    },
                    "style": {
                        "type": "string",
                        "description": "Documentation style (javadoc, sphinx, markdown, etc.)",
                        "default": "markdown"
                    }
                },
                "required": ["code"]
            }),
            Box::new(|registry, args| {
                let registry = registry.clone_for_handler();
                tokio::spawn(async move {
                    registry.handle_generate_docs(args).await
                })
            }),
        );

        info!("Registered {} basic MCP tools", self.tools.len());
        Ok(())
    }

    /// Register advanced tools
    async fn register_advanced_tools(&mut self) -> Result<()> {
        let advanced_tool_definitions = self.advanced_tools.get_advanced_tools();
        
        for (name, tool) in advanced_tool_definitions {
            self.register_tool(
                &name,
                &tool.description,
                tool.input_schema.clone(),
                Box::new(move |registry, args| {
                    let name_clone = name.clone();
                    let registry_clone = registry.clone_for_handler();
                    tokio::spawn(async move {
                        registry_clone.handle_advanced_tool(&name_clone, args).await
                    })
                }),
            );
        }

        info!("Registered {} advanced MCP tools", advanced_tool_definitions.len());
        info!("Total MCP tools available: {}", self.tools.len());
        Ok(())
    }

    /// Register a tool
    fn register_tool(
        &mut self,
        name: &str,
        description: &str,
        input_schema: Value,
        handler: ToolHandler,
    ) {
        let tool = Tool {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        };

        self.tools.insert(name.to_string(), ToolDefinition {
            tool,
            handler,
        });
    }

    /// List all available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        Ok(self.tools.values().map(|def| def.tool.clone()).collect())
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<ToolResult> {
        let tool_def = self.tools.get(name)
            .ok_or_else(|| anyhow!("Tool not found: {}", name))?;

        info!("Executing tool: {}", name);
        
        // Use performance optimizations
        let result = self.performance_manager.execute_tool_optimized(
            name,
            &arguments,
            || async {
                let handle = (tool_def.handler)(self, arguments.clone());
                let tool_result = handle.await?;
                
                // Convert ToolResult to JSON for caching
                Ok(serde_json::to_value(tool_result)?)
            }
        ).await?;
        
        // Convert back to ToolResult
        Ok(serde_json::from_value(result)?)
    }

    /// Clone for handler (simplified version for async closures)
    fn clone_for_handler(&self) -> ToolRegistryHandler {
        ToolRegistryHandler {
            consensus_engine: self.consensus_engine.clone(),
            config: self.config.clone(),
        }
    }

    /// Handle ask_hive tool
    async fn handle_ask_hive(&self, args: Value) -> Result<ToolResult> {
        let question = args.get("question")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: question"))?;

        let context = args.get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut prompt = question.to_string();
        if !context.is_empty() {
            prompt = format!("Context: {}\n\nQuestion: {}", context, question);
        }

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle analyze_code tool
    async fn handle_analyze_code(&self, args: Value) -> Result<ToolResult> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: path"))?;

        let focus = args.get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        // Use existing analyze command
        let analysis = analyze_codebase(
            path,
            true, // detailed
            Some(focus.to_string()),
            &self.config,
        ).await
            .map_err(|e| anyhow!("Analysis failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: analysis 
            }],
            is_error: None,
        })
    }

    /// Handle explain_code tool
    async fn handle_explain_code(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let prompt = format!(
            "Please explain what this {} code does:\n\n```{}\n{}\n```",
            language, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle improve_code tool
    async fn handle_improve_code(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let focus = args.get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let prompt = format!(
            "Please suggest improvements for this {} code, focusing on {}:\n\n```{}\n{}\n```\n\nProvide specific, actionable suggestions.",
            language, focus, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle generate_tests tool
    async fn handle_generate_tests(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let test_framework = args.get("test_framework")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let prompt = format!(
            "Generate comprehensive unit tests for this {} code using {}:\n\n```{}\n{}\n```\n\nInclude edge cases and error conditions.",
            language, test_framework, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle repository_summary tool
    async fn handle_repository_summary(&self, args: Value) -> Result<ToolResult> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: path"))?;

        // Use repository analyzer
        let analyzer = RepositoryAnalyzer::new(self.config.clone());
        let summary = analyzer.analyze_repository(path).await
            .map_err(|e| anyhow!("Repository analysis failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: summary.to_string() 
            }],
            is_error: None,
        })
    }

    /// Handle plan_project tool
    async fn handle_plan_project(&self, args: Value) -> Result<ToolResult> {
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: description"))?;

        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("feature");

        let constraints = args.get("constraints")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n- "))
            .unwrap_or_default();

        let prompt = format!(
            "Create a detailed implementation plan for the following project:\n\n\
            Description: {}\n\
            Scope: {}\n\
            {}\n\n\
            Please provide:\n\
            1. Project overview and goals\n\
            2. Task breakdown with dependencies\n\
            3. Timeline and milestones\n\
            4. Risk analysis and mitigation strategies\n\
            5. Resource requirements\n\
            6. Success criteria",
            description, scope,
            if constraints.is_empty() { String::new() } else { format!("Constraints:\n- {}", constraints) }
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Planning failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle transform_code tool
    async fn handle_transform_code(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let transformation = args.get("transformation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: transformation"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let prompt = format!(
            "Apply the following transformation to this {} code:\n\n\
            Transformation: {}\n\n\
            Original code:\n```{}\n{}\n```\n\n\
            Please provide the transformed code with explanations of changes made.",
            language, transformation, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Transformation failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle search_memory tool
    async fn handle_search_memory(&self, args: Value) -> Result<ToolResult> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;

        let limit = args.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10) as usize;

        let search_type = args.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        // TODO: Implement actual memory search
        // For now, return a placeholder response
        let placeholder = format!(
            "Memory search results for query '{}' (type: {}, limit: {}):\n\n\
            [Memory search integration pending - this will search through:\n\
            - Conversation history\n\
            - Knowledge base entries\n\
            - Code analysis results\n\
            - Planning documents]",
            query, search_type, limit
        );

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: placeholder 
            }],
            is_error: None,
        })
    }

    /// Handle generate_analytics tool
    async fn handle_generate_analytics(&self, args: Value) -> Result<ToolResult> {
        let report_type = args.get("report_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: report_type"))?;

        let timeframe = args.get("timeframe")
            .and_then(|v| v.as_str())
            .unwrap_or("week");

        let format = args.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("summary");

        // TODO: Implement actual analytics generation
        // For now, return a placeholder response
        let placeholder = format!(
            "Analytics Report: {} ({})\n\
            Timeframe: Past {}\n\n\
            [Analytics integration pending - this will provide:\n\
            - Performance metrics\n\
            - Cost analysis\n\
            - Usage patterns\n\
            - Trend analysis\n\
            - Predictive insights]",
            report_type, format, timeframe
        );

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: placeholder 
            }],
            is_error: None,
        })
    }

    /// Handle generate_docs tool
    async fn handle_generate_docs(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let style = args.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("markdown");

        let prompt = format!(
            "Generate comprehensive {} documentation for this {} code:\n\n\
            ```{}\n{}\n```\n\n\
            Include:\n\
            - Function/class descriptions\n\
            - Parameter documentation\n\
            - Return value descriptions\n\
            - Usage examples\n\
            - Any important notes or warnings",
            style, language, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Documentation generation failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }
}

/// Simplified tool registry for handlers
#[derive(Clone)]
struct ToolRegistryHandler {
    consensus_engine: Arc<RwLock<ConsensusEngine>>,
    config: Arc<Config>,
}

impl ToolRegistryHandler {
    /// Handle ask_hive tool
    async fn handle_ask_hive(&self, args: Value) -> Result<ToolResult> {
        let question = args.get("question")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: question"))?;

        let context = args.get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut prompt = question.to_string();
        if !context.is_empty() {
            prompt = format!("Context: {}\n\nQuestion: {}", context, question);
        }

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle analyze_code tool
    async fn handle_analyze_code(&self, args: Value) -> Result<ToolResult> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: path"))?;

        let focus = args.get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        // Use existing analyze command
        let analysis = analyze_codebase(
            path,
            true, // detailed
            Some(focus.to_string()),
            &self.config,
        ).await
            .map_err(|e| anyhow!("Analysis failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: analysis 
            }],
            is_error: None,
        })
    }

    /// Handle explain_code tool
    async fn handle_explain_code(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let prompt = format!(
            "Please explain what this {} code does:\n\n```{}\n{}\n```",
            language, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle improve_code tool
    async fn handle_improve_code(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let focus = args.get("focus")
            .and_then(|v| v.as_str())
            .unwrap_or("general");

        let prompt = format!(
            "Please suggest improvements for this {} code, focusing on {}:\n\n```{}\n{}\n```\n\nProvide specific, actionable suggestions.",
            language, focus, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle generate_tests tool
    async fn handle_generate_tests(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let test_framework = args.get("test_framework")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let prompt = format!(
            "Generate comprehensive unit tests for this {} code using {}:\n\n```{}\n{}\n```\n\nInclude edge cases and error conditions.",
            language, test_framework, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Consensus failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle repository_summary tool
    async fn handle_repository_summary(&self, args: Value) -> Result<ToolResult> {
        let path = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: path"))?;

        // Use repository analyzer
        let analyzer = RepositoryAnalyzer::new(self.config.clone());
        let summary = analyzer.analyze_repository(path).await
            .map_err(|e| anyhow!("Repository analysis failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: summary.to_string() 
            }],
            is_error: None,
        })
    }

    /// Handle plan_project tool
    async fn handle_plan_project(&self, args: Value) -> Result<ToolResult> {
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: description"))?;

        let scope = args.get("scope")
            .and_then(|v| v.as_str())
            .unwrap_or("feature");

        let constraints = args.get("constraints")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join("\n- "))
            .unwrap_or_default();

        let prompt = format!(
            "Create a detailed implementation plan for the following project:\n\n\
            Description: {}\n\
            Scope: {}\n\
            {}\n\n\
            Please provide:\n\
            1. Project overview and goals\n\
            2. Task breakdown with dependencies\n\
            3. Timeline and milestones\n\
            4. Risk analysis and mitigation strategies\n\
            5. Resource requirements\n\
            6. Success criteria",
            description, scope,
            if constraints.is_empty() { String::new() } else { format!("Constraints:\n- {}", constraints) }
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Planning failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle transform_code tool
    async fn handle_transform_code(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let transformation = args.get("transformation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: transformation"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let prompt = format!(
            "Apply the following transformation to this {} code:\n\n\
            Transformation: {}\n\n\
            Original code:\n```{}\n{}\n```\n\n\
            Please provide the transformed code with explanations of changes made.",
            language, transformation, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Transformation failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle search_memory tool
    async fn handle_search_memory(&self, args: Value) -> Result<ToolResult> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: query"))?;

        let limit = args.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(10) as usize;

        let search_type = args.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        // TODO: Implement actual memory search
        // For now, return a placeholder response
        let placeholder = format!(
            "Memory search results for query '{}' (type: {}, limit: {}):\n\n\
            [Memory search integration pending - this will search through:\n\
            - Conversation history\n\
            - Knowledge base entries\n\
            - Code analysis results\n\
            - Planning documents]",
            query, search_type, limit
        );

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: placeholder 
            }],
            is_error: None,
        })
    }

    /// Handle generate_analytics tool
    async fn handle_generate_analytics(&self, args: Value) -> Result<ToolResult> {
        let report_type = args.get("report_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: report_type"))?;

        let timeframe = args.get("timeframe")
            .and_then(|v| v.as_str())
            .unwrap_or("week");

        let format = args.get("format")
            .and_then(|v| v.as_str())
            .unwrap_or("summary");

        // TODO: Implement actual analytics generation
        // For now, return a placeholder response
        let placeholder = format!(
            "Analytics Report: {} ({})\n\
            Timeframe: Past {}\n\n\
            [Analytics integration pending - this will provide:\n\
            - Performance metrics\n\
            - Cost analysis\n\
            - Usage patterns\n\
            - Trend analysis\n\
            - Predictive insights]",
            report_type, format, timeframe
        );

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: placeholder 
            }],
            is_error: None,
        })
    }

    /// Handle generate_docs tool
    async fn handle_generate_docs(&self, args: Value) -> Result<ToolResult> {
        let code = args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing required parameter: code"))?;

        let language = args.get("language")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let style = args.get("style")
            .and_then(|v| v.as_str())
            .unwrap_or("markdown");

        let prompt = format!(
            "Generate comprehensive {} documentation for this {} code:\n\n\
            ```{}\n{}\n```\n\n\
            Include:\n\
            - Function/class descriptions\n\
            - Parameter documentation\n\
            - Return value descriptions\n\
            - Usage examples\n\
            - Any important notes or warnings",
            style, language, language, code
        );

        let engine = self.consensus_engine.read().await;
        let response = engine.process(&prompt).await
            .map_err(|e| anyhow!("Documentation generation failed: {}", e))?;

        Ok(ToolResult {
            content: vec![ToolContent::Text { 
                text: response.result.unwrap_or_default() 
            }],
            is_error: None,
        })
    }

    /// Handle advanced tool call
    async fn handle_advanced_tool(&self, tool_name: &str, args: Value) -> Result<ToolResult> {
        // Create advanced tools registry for this call
        let advanced_tools = AdvancedToolRegistry::new(
            self.consensus_engine.clone(),
            self.config.clone(),
        );

        // Delegate to advanced tools handler
        advanced_tools.handle_advanced_tool(tool_name, args).await
    }
}
