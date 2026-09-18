//! Phase 8 Protocols Demo
//!
//! Demonstrates the native Model Context Protocol (MCP 2.0) client and the
//! Agent Client Protocol (ACP) communication bridge:
//! 1. MCP 2.0 Protocol Types & Schemas (tools, prompts, resources)
//! 2. Live MCP Stdio Server Handshake & Tool Execution
//! 3. Bidirectional ACP JSON-RPC Adapter (Inbound Commands & Outbound Streaming Events)
//!
//! Run with:
//! cargo run --example phase8_protocols_demo -p agent

use std::fs;
use std::time::Duration;

use agent::protocols::{
    AcpAdapter, AcpInboundCommand, McpClient, McpPrompt, McpPromptArgument, McpTool,
};
use agent::types::AgentEvent;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("============================================================");
    println!("⚡ ZENTHREE AGENT PROTOCOLS DEMO (PHASE 8: MCP 2.0 & ACP)");
    println!("============================================================\n");

    // -----------------------------------------------------------------------
    // Section 1: MCP 2.0 Typed Schemas & Prompt Templates
    // -----------------------------------------------------------------------
    println!("--- 1. Model Context Protocol 2.0 Types & Prompt Templates ---");

    let prompt_spec = McpPrompt {
        name: "review_code".to_string(),
        description: Some("Automated code review and security audit".to_string()),
        arguments: Some(vec![
            McpPromptArgument {
                name: "file_path".to_string(),
                description: Some("Target source code file".to_string()),
                required: Some(true),
            },
            McpPromptArgument {
                name: "severity".to_string(),
                description: Some("Minimum alert severity".to_string()),
                required: Some(false),
            },
        ]),
    };
    println!("Serialized MCP Prompt Template:");
    println!("  {}\n", serde_json::to_string_pretty(&prompt_spec)?);

    let sample_tool = McpTool {
        name: "execute_sql_query".to_string(),
        description: Some("Runs read-only SQL queries against local SQLite database".to_string()),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "SQL statement" }
            },
            "required": ["query"]
        }),
    };
    println!("Registered MCP Tool Specification:");
    println!("  Tool Name: {}", sample_tool.name);
    println!("  Description: {}", sample_tool.description.as_deref().unwrap_or(""));
    println!("  Input Schema: {}\n", sample_tool.input_schema);

    // -----------------------------------------------------------------------
    // Section 2: Mock MCP Stdio Server & Live McpClient Handshake
    // -----------------------------------------------------------------------
    println!("--- 2. Live MCP Stdio Process Interop ---");

    // Create a temporary mock MCP server script in python
    let temp_dir = tempfile::tempdir()?;
    let mock_server_script = temp_dir.path().join("mock_mcp_server.py");
    let script_code = r#"
import sys, json

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
    except Exception:
        continue

    method = req.get("method")
    req_id = req.get("id")

    if method == "initialize":
        resp = {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2025-11-25",
                "capabilities": {
                    "tools": {"listChanged": True},
                    "resources": {"subscribe": False},
                    "prompts": {"listChanged": False}
                },
                "serverInfo": {
                    "name": "mock-sqlite-mcp-server",
                    "version": "2.0.0"
                },
                "instructions": "Execute SQL queries carefully"
            }
        }
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()

    elif method == "tools/list":
        resp = {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": [
                    {
                        "name": "sqlite_query",
                        "description": "Run analytical SQL queries",
                        "inputSchema": {
                            "type": "object",
                            "properties": {"sql": {"type": "string"}},
                            "required": ["sql"]
                        }
                    }
                ]
            }
        }
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()

    elif method == "tools/call":
        params = req.get("params", {})
        sql = params.get("arguments", {}).get("sql", "SELECT 1")
        resp = {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": "Query '" + str(sql) + "' returned: [{\"id\": 1, \"status\": \"ACTIVE\"}]"
                    }
                ]
            }
        }
        sys.stdout.write(json.dumps(resp) + "\n")
        sys.stdout.flush()

    elif method == "notifications/initialized":
        pass
"#;
    fs::write(&mock_server_script, script_code)?;

    // Spawn McpClient over Stdio
    println!("Spawning local MCP server process via Stdio...");
    let script_path_str = mock_server_script.to_string_lossy();
    let mut mcp_client = McpClient::connect_stdio(
        "python3",
        &[&script_path_str],
        Some(temp_dir.path()),
        &[],
    )?;

    // Step A: Initialize Handshake
    let init_result = mcp_client.initialize(Duration::from_secs(5))?;
    println!("✓ Initialized MCP Connection:");
    println!("  Server Name: {}", init_result.server_info.name);
    println!("  Server Version: {}", init_result.server_info.version);
    println!("  Protocol Version: {}", init_result.protocol_version);
    println!("  Server Instructions: {:?}", init_result.instructions);

    // Step B: Register Server into McpToolBridge for the LLM
    println!("\n--- 2B. Bridging MCP Tools into LLM Runtime (tools/mcp_bridge.rs) ---");
    let mut mcp_bridge = agent::tools::McpToolBridge::new();
    let registered_count = mcp_bridge.register_server("sqlite_db", mcp_client, Duration::from_secs(5))?;
    println!("✓ Successfully registered {} external tools into McpToolBridge", registered_count);

    // Step C: Schema Conversion for LLM Function Calling
    let llm_schemas = mcp_bridge.to_llm_tool_schemas();
    println!("\n✓ Generated Function Declaration Schemas for LLM (OpenAI/Anthropic/Gemini compatible):");
    for schema in &llm_schemas {
        println!("  {}", serde_json::to_string(schema)?);
    }

    // Step D: LLM System Prompt Injection
    let prompt_block = mcp_bridge.format_mcp_prompt_block();
    let system_prompt = agent::orchestrator::SystemPromptBuilder::new(Some(temp_dir.path().to_path_buf()))
        .with_mcp_tools(&prompt_block)
        .build();
    println!("\n✓ Injected MCP Tools Prompt Block into LLM Master System Prompt:");
    println!("  Master prompt contains <mcp_tools>: {}", system_prompt.contains("<mcp_tools>"));

    // Step E: LLM Emits Tool Call -> Dispatched through McpToolBridge
    println!("\nSimulating LLM executing tool call: `AgentToolCall::Mcp`...");
    let llm_tool_call = agent::tools::AgentToolCall::Mcp {
        tool_name: "sqlite_query".to_string(),
        arguments: json!({ "sql": "SELECT id, status FROM agents LIMIT 1" }),
    };

    let execution_result = agent::tools::execute_agent_tool_with_bridge(
        temp_dir.path(),
        &llm_tool_call,
        Some(&mcp_bridge),
    );
    println!("✓ Result returned to LLM from external MCP Server:");
    println!("  Tool: {}", execution_result.tool_name);
    println!("  Success: {}", execution_result.ok);
    println!("  Output: {}", execution_result.output);

    // -----------------------------------------------------------------------
    // Section 3: Agent Client Protocol (ACP) Adapter
    // -----------------------------------------------------------------------
    println!("\n--- 3. Bidirectional ACP JSON-RPC Adapter ---");

    // Simulating inbound editor requests
    let raw_acp_prompt_request = r#"{
        "jsonrpc": "2.0",
        "id": 101,
        "method": "session/prompt",
        "params": {
            "sessionId": "zenthree_session_889",
            "prompt": "Refactor auth handler to support OAuth2 Bearer tokens"
        }
    }"#;

    let (id, inbound_cmd) = AcpAdapter::parse_inbound_request(raw_acp_prompt_request)?;
    println!("✓ Inbound ACP JSON-RPC Parsed:");
    println!("  ID: {}", id);
    match inbound_cmd {
        AcpInboundCommand::Prompt { session_id, prompt } => {
            println!("  Command: Session Prompt");
            println!("  Session ID: {}", session_id);
            println!("  Prompt: {}", prompt);
        }
        _ => panic!("Expected Prompt command"),
    }

    // Simulating outbound AgentEvents streaming to the editor
    println!("\n✓ Outbound AgentEvents Translated to ACP JSON-RPC Notifications:");
    let events = vec![
        AgentEvent::Thinking {
            delta: "Planning authentication refactor with JWT validation...".to_string(),
        },
        AgentEvent::ToolStarted {
            tool: "read_file".to_string(),
            summary: "Reading crates/agent/src/auth.rs".to_string(),
        },
        AgentEvent::ToolFinished {
            tool: "read_file".to_string(),
            ok: true,
            output_snippet: "pub fn verify_token() { ... }".to_string(),
            duration_ms: 18,
        },
        AgentEvent::StreamChunk {
            delta: "I have reviewed the authentication handler and propose...".to_string(),
        },
        AgentEvent::Completed {
            summary: "OAuth2 authentication handler refactored cleanly.".to_string(),
        },
    ];

    for event in &events {
        let notif = AcpAdapter::format_outbound_event("zenthree_session_889", event);
        println!("  [ACP NOTIFICATION] method: '{}', params: {}", notif.method, notif.params.unwrap());
    }

    println!("\n============================================================");
    println!("🎉 PHASE 8 PROTOCOLS DEMO COMPLETED SUCCESSFULLY!");
    println!("============================================================");

    Ok(())
}
