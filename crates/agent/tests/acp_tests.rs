use agent::acp::protocol::*;
use agent::acp::adapter::AcpAdapter;
use agent::types::AgentEvent;
use serde_json::json;

#[test]
fn test_acp_request_serialization() {
    let req = AcpRequest::new(
        42,
        "session/prompt",
        Some(json!({
            "sessionId": "sess-123",
            "prompt": "Fix bug in main.rs"
        })),
    );

    let serialized = serde_json::to_string(&req).expect("Failed to serialize request");
    assert!(serialized.contains("\"id\":42"));
    assert!(serialized.contains("\"method\":\"session/prompt\""));
    assert!(serialized.contains("\"sessionId\":\"sess-123\""));
}

#[test]
fn test_acp_response_deserialization() {
    let raw = r#"{
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "protocolVersion": "1.0",
            "agentInfo": {
                "name": "goose",
                "version": "1.0.0",
                "capabilities": ["tools", "recipes", "streaming"]
            }
        }
    }"#;

    let incoming: AcpIncomingMessage = serde_json::from_str(raw).expect("Failed to parse response");
    match incoming {
        AcpIncomingMessage::Response(resp) => {
            assert_eq!(resp.id, 1);
            let init_res: InitializeResult = serde_json::from_value(resp.result.unwrap()).unwrap();
            assert_eq!(init_res.agent_info.unwrap().name, "goose");
        }
        _ => panic!("Expected response message"),
    }
}

#[test]
fn test_acp_adapter_converts_notifications_to_agent_events() {
    // 1. Thinking notification
    let thinking_notif = AcpNotification {
        jsonrpc: "2.0".to_string(),
        method: "session/update".to_string(),
        params: Some(json!({
            "sessionId": "sess-1",
            "eventType": "thinking",
            "content": "Analyzing AST of Cargo.toml..."
        })),
    };
    let events = AcpAdapter::convert_notification(&thinking_notif);
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::Thinking { delta } => {
            assert_eq!(delta, "Analyzing AST of Cargo.toml...");
        }
        other => panic!("Expected Thinking, got {:?}", other),
    }

    // 2. Tool call notification
    let tool_notif = AcpNotification {
        jsonrpc: "2.0".to_string(),
        method: "session/update".to_string(),
        params: Some(json!({
            "sessionId": "sess-1",
            "eventType": "tool_call",
            "toolName": "read_file",
            "toolArgs": { "path": "src/main.rs" }
        })),
    };
    let events = AcpAdapter::convert_notification(&tool_notif);
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::ToolStarted { tool, summary } => {
            assert_eq!(tool, "read_file");
            assert!(summary.contains("src/main.rs"));
        }
        other => panic!("Expected ToolStarted, got {:?}", other),
    }

    // 3. Diff notification
    let diff_notif = AcpNotification {
        jsonrpc: "2.0".to_string(),
        method: "session/update".to_string(),
        params: Some(json!({
            "sessionId": "sess-1",
            "eventType": "diff",
            "filePath": "src/lib.rs"
        })),
    };
    let events = AcpAdapter::convert_notification(&diff_notif);
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::PendingModifications { summaries } => {
            assert_eq!(summaries[0].file_name, "src/lib.rs");
        }
        other => panic!("Expected PendingModifications, got {:?}", other),
    }

    // 4. Completed notification
    let completed_notif = AcpNotification {
        jsonrpc: "2.0".to_string(),
        method: "session/completed".to_string(),
        params: Some(json!({
            "summary": "Updated dependencies successfully"
        })),
    };
    let events = AcpAdapter::convert_notification(&completed_notif);
    assert_eq!(events.len(), 1);
    match &events[0] {
        AgentEvent::Completed { summary } => {
            assert_eq!(summary, "Updated dependencies successfully");
        }
        other => panic!("Expected Completed, got {:?}", other),
    }
}

#[test]
fn test_goose_supervisor_status_check() {
    let supervisor = agent::GooseSupervisor::new();
    match supervisor.check_status() {
        agent::GooseBinaryStatus::Installed { binary_path, version } => {
            println!("Goose binary detected: {} (v{})", binary_path.display(), version);
        }
        agent::GooseBinaryStatus::NotFound { install_command } => {
            assert!(install_command.contains("download_cli.sh"));
        }
    }
}
