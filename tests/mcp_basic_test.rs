//! Basic MCP protocol tests

use crate::integration::mcp::protocol::*;

#[test]
fn test_mcp_protocol_constants() {
    // Test that the protocol version is defined
    assert_eq!(MCP_VERSION, "2024-11-05");
}

#[test]
fn test_mcp_error_codes() {
    // Test error code generation
    let parse_error = McpError::parse_error();
    assert_eq!(parse_error.code, -32700);
    assert_eq!(parse_error.message, "Parse error");

    let invalid_request = McpError::invalid_request();
    assert_eq!(invalid_request.code, -32600);

    let method_not_found = McpError::method_not_found();
    assert_eq!(method_not_found.code, -32601);

    let invalid_params = McpError::invalid_params();
    assert_eq!(invalid_params.code, -32602);

    let internal_error = McpError::internal_error();
    assert_eq!(internal_error.code, -32603);
}

#[test]
fn test_tool_serialization() {
    let tool = Tool {
        name: "test_tool".to_string(),
        description: "A test tool".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string"
                }
            },
            "required": ["input"]
        }),
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&tool).unwrap();
    let deserialized: Tool = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.name, tool.name);
    assert_eq!(deserialized.description, tool.description);
}

#[test]
fn test_resource_serialization() {
    let resource = Resource {
        uri: "file:///test/file.rs".to_string(),
        name: "file.rs".to_string(),
        description: Some("A Rust file".to_string()),
        mime_type: Some("text/x-rust".to_string()),
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&resource).unwrap();
    let deserialized: Resource = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.uri, resource.uri);
    assert_eq!(deserialized.name, resource.name);
    assert_eq!(deserialized.description, resource.description);
    assert_eq!(deserialized.mime_type, resource.mime_type);
}

#[test]
fn test_mcp_message_structure() {
    let request = McpRequest {
        method: "tools/list".to_string(),
        params: serde_json::json!({}),
    };

    let message = McpMessage {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(1)),
        content: McpMessageContent::Request(request),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&message).unwrap();

    // Check that JSON contains expected fields
    assert!(json.contains("jsonrpc"));
    assert!(json.contains("\"2.0\""));
    assert!(json.contains("tools/list"));
}

#[test]
fn test_initialize_params() {
    let params = InitializeParams {
        protocol_version: MCP_VERSION.to_string(),
        capabilities: ClientCapabilities {
            experimental: None,
            sampling: None,
        },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    // Should serialize correctly
    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["protocolVersion"], MCP_VERSION);
    assert_eq!(json["clientInfo"]["name"], "test-client");
}

#[test]
fn test_server_capabilities() {
    let capabilities = ServerCapabilities {
        logging: Some(McpCapability { experimental: None }),
        prompts: None,
        resources: Some(McpCapability { experimental: None }),
        tools: Some(McpCapability { experimental: None }),
    };

    // Serialize and check structure
    let json = serde_json::to_value(&capabilities).unwrap();
    assert!(json["logging"].is_object());
    assert!(json["prompts"].is_null());
    assert!(json["resources"].is_object());
    assert!(json["tools"].is_object());
}
