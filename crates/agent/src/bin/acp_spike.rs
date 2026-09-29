use std::path::PathBuf;
use std::time::Duration;

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::schema::v1::{
    ContentBlock, Implementation, InitializeRequest, NewSessionRequest,
    PromptRequest, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome, SessionNotification, TextContent,
};
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Agent, Client, ConnectionTo, LineDirection};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("============================================================");
    eprintln!("                   TASK 1: ACP SPIKE                        ");
    eprintln!("============================================================");

    // 1. Exact command & args from Zed registry
    // Command: ./agy_acp_server.par
    // Args: ["--uid="]
    let binary_path = std::env::args().nth(1).unwrap_or_else(|| {
        "/home/kabir/.local/share/zed/external_agents/registry/antigravity-acp/v_1.2.1_206620a657f1ccc8_54ab3757f38ebeb7/agy_acp_server.par".to_string()
    });

    let binary = PathBuf::from(&binary_path);
    if !binary.exists() {
        eprintln!("[FATAL] Binary not found at: {:?}", binary);
        return Ok(());
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree"));

    eprintln!("Command: {:?}", binary);
    eprintln!("Args: [\"--uid=\"]");
    eprintln!("Cwd: {:?}", cwd);

    let config = AcpAgentConfig::new(binary).arg("--uid=");

    // Separate stderr logging from raw stdin/stdout protocol lines
    let agent = AcpAgent::new(config).with_debug(|line, dir| {
        match dir {
            LineDirection::Stdin => println!("[CLIENT -> AGENT RAW]: {}", line),
            LineDirection::Stdout => println!("[AGENT -> CLIENT RAW]: {}", line),
            LineDirection::Stderr => eprintln!("[AGENT STDERR]: {}", line),
        }
    });

    let client_builder = Client.builder();

    let client_builder = client_builder
        .on_receive_notification(
            async move |notification: SessionNotification, _cx| {
                println!("[NOTIFICATION PARSED]: {:?}", notification.update);
                Ok(())
            },
            agent_client_protocol::on_receive_notification!(),
        )
        .on_receive_request(
            async move |req: RequestPermissionRequest, responder, _cx| {
                println!("[PERMISSION REQUEST]: {:?}", req);
                let first_opt = req.options.first().map(|o| o.option_id.clone());
                if let Some(opt_id) = first_opt {
                    let _ = responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(opt_id)),
                    ));
                } else {
                    let _ = responder.respond(RequestPermissionResponse::new(
                        RequestPermissionOutcome::Cancelled,
                    ));
                }
                Ok(())
            },
            agent_client_protocol::on_receive_request!(),
        );

    let result = client_builder
        .connect_with(agent, |cx: ConnectionTo<Agent>| async move {
            // STEP 2: Initialize
            println!("\n>>> SENDING INITIALIZE REQUEST...");
            let mut init_req = InitializeRequest::new(ProtocolVersion::V1);
            init_req.client_info = Some(Implementation::new("zenthree", "0.1.0"));

            let init_resp = cx.send_request(init_req).block_task().await?;

            println!("\n=== INITIALIZE RESPONSE SUMMARY ===");
            println!("Protocol Version: {:?}", init_resp.protocol_version);
            println!("Agent Info: {:?}", init_resp.agent_info);
            println!("Capabilities: {:?}", init_resp.agent_capabilities);
            println!("Auth Methods ({} found):", init_resp.auth_methods.len());
            for (i, m) in init_resp.auth_methods.iter().enumerate() {
                println!("  [{}] id='{:?}' name='{}' desc='{:?}'", i, m.id(), m.name(), m.description());
            }
            println!("====================================\n");

            // STEP 3: Test session/new without authenticate first to see if auth is required
            println!(">>> SENDING SESSION/NEW (testing if auth is required)...");
            let new_session_req = NewSessionRequest::new(cwd.clone());

            let session_resp = match cx.send_request(new_session_req).block_task().await {
                Ok(resp) => {
                    println!("\n=== SESSION/NEW SUCCEEDED ===");
                    println!("Session ID: {:?}", resp.session_id);
                    println!("Auth was NOT required before session/new (existing credentials accepted)!");
                    println!("=============================\n");
                    resp
                }
                Err(e) => {
                    println!("\n=== SESSION/NEW FAILED ===");
                    println!("Error: {:?}", e);
                    println!("Auth WAS required before session/new!");
                    println!("==========================\n");
                    return Err(e);
                }
            };

            let session_id = session_resp.session_id;

            // Wait 200ms for initial notifications
            tokio::time::sleep(Duration::from_millis(200)).await;

            // STEP 4: Send prompt with "hi"
            println!(">>> SENDING PROMPT WITH 'hi'...");
            let prompt_req = PromptRequest::new(
                session_id.clone(),
                vec![ContentBlock::Text(TextContent::new("hi"))],
            );

            let _prompt_resp = match cx.send_request(prompt_req).block_task().await {
                Ok(resp) => {
                    println!("\n=== PROMPT RESPONSE RECEIVED ===");
                    println!("Stop reason: {:?}", resp.stop_reason);
                    println!("================================\n");
                    resp
                }
                Err(e) => {
                    println!("\n=== PROMPT FAILED ===");
                    println!("Error: {:?}", e);
                    println!("=====================\n");
                    return Err(e);
                }
            };

            // Allow trailing messages to flush
            tokio::time::sleep(Duration::from_millis(300)).await;
            Ok(())
        })
        .await;

    if let Err(e) = result {
        eprintln!("[ERROR] Spike run failed: {:?}", e);
    }

    Ok(())
}
