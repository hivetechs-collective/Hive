//! MCP Server Demo
//!
//! This example demonstrates the MCP server functionality

use hive_ai::integration::mcp::protocol::*;
use serde_json::json;

fn main() {
    println!("🚀 Hive AI MCP Server Demo\n");

    // Show protocol version
    println!("📡 MCP Protocol Version: {}", MCP_VERSION);
    println!();

    // Demo: Initialize request
    println!("1️⃣  Initialize Request Example:");
    let init_request = McpMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        content: McpMessageContent::Request(McpRequest {
            method: "initialize".to_string(),
            params: json!({
                "protocolVersion": MCP_VERSION,
                "capabilities": {
                    "experimental": {}
                },
                "clientInfo": {
                    "name": "VS Code MCP Client",
                    "version": "1.0.0"
                }
            }),
        }),
    };

    println!("{}", serde_json::to_string_pretty(&init_request).unwrap());
    println!();

    // Demo: Initialize response
    println!("2️⃣  Initialize Response Example:");
    let init_response = McpMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        content: McpMessageContent::Response(McpResponse::Success {
            result: json!({
                "protocolVersion": MCP_VERSION,
                "capabilities": {
                    "tools": {},
                    "resources": {},
                    "logging": {}
                },
                "serverInfo": {
                    "name": "Hive AI MCP Server",
                    "version": "0.1.0"
                }
            }),
        }),
    };

    println!("{}", serde_json::to_string_pretty(&init_response).unwrap());
    println!();

    // Demo: List tools
    println!("3️⃣  Available Tools:");
    let tools = vec![
        Tool {
            name: "ask_hive".to_string(),
            description: "Ask Hive AI a question using multi-model consensus".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "question": {
                        "type": "string",
                        "description": "The question to ask"
                    }
                },
                "required": ["question"]
            }),
        },
        Tool {
            name: "analyze_code".to_string(),
            description: "Analyze code files or directories".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to analyze"
                    }
                },
                "required": ["path"]
            }),
        },
        Tool {
            name: "plan_project".to_string(),
            description: "Create a strategic implementation plan".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "description": {
                        "type": "string",
                        "description": "What to plan"
                    }
                },
                "required": ["description"]
            }),
        },
    ];

    for tool in &tools {
        println!("  🛠️  {} - {}", tool.name, tool.description);
    }
    println!();

    // Demo: Call tool request
    println!("4️⃣  Tool Call Example:");
    let tool_call = McpMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        content: McpMessageContent::Request(McpRequest {
            method: "tools/call".to_string(),
            params: json!({
                "name": "ask_hive",
                "arguments": {
                    "question": "What is the best way to implement error handling in Rust?"
                }
            }),
        }),
    };

    println!("{}", serde_json::to_string_pretty(&tool_call).unwrap());
    println!();

    // Demo: Resources
    println!("5️⃣  Available Resources:");
    let resources = vec![
        Resource {
            uri: "hive://config".to_string(),
            name: "Hive Configuration".to_string(),
            description: Some("Current Hive AI configuration".to_string()),
            mime_type: Some("application/toml".to_string()),
        },
        Resource {
            uri: "hive://memory/conversations".to_string(),
            name: "Conversation History".to_string(),
            description: Some("Recent conversation summaries".to_string()),
            mime_type: Some("application/json".to_string()),
        },
        Resource {
            uri: "file:///src/main.rs".to_string(),
            name: "main.rs".to_string(),
            description: Some("Main source file".to_string()),
            mime_type: Some("text/x-rust".to_string()),
        },
    ];

    for resource in &resources {
        println!("  📁 {} - {}", resource.uri, resource.name);
    }
    println!();

    // Demo: Error handling
    println!("6️⃣  Error Response Example:");
    let error_response = McpMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        content: McpMessageContent::Response(McpResponse::Error {
            error: McpError::custom(-32000, "Tool not found: unknown_tool".to_string()),
        }),
    };

    println!("{}", serde_json::to_string_pretty(&error_response).unwrap());
    println!();

    println!("✅ MCP Protocol Implementation Complete!");
    println!("💡 Use 'hive serve' to start the MCP server on port 7777");
}
