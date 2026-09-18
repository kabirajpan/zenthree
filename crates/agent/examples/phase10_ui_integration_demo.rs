//! Phase 10 Cross-Crate UI Integration Demo
//!
//! Demonstrates the complete integration between the autonomous agent engine and UI:
//! 1. Dynamic System Prompt Generation via SystemPromptBuilder
//! 2. Streaming UI Event Loop (Thinking deltas, Token chunks, Tool events)
//! 3. Interactive Tool Permission Gating (Allow Once, Always Allow, Deny)
//! 4. Multi-File Checkpoint Snapshots & Live Matrix Diff Rollback
//!
//! Run with:
//! cargo run --example phase10_ui_integration_demo -p agent

use agent::diff::{CheckpointTimeline, MatrixDiff};
use agent::orchestrator::SystemPromptBuilder;
use agent::permissions::{
    PermissionEngine, ToolPermissionDecision, UserPermissionResponse,
};
use agent::types::{AgentEvent, PendingPermissionRequest};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===============================================================================");
    println!("       PHASE 10 DEMO: CROSS-CRATE UI INTEGRATION & EVENT ORCHESTRATION        ");
    println!("===============================================================================\n");

    let temp_dir = tempfile::tempdir()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // -----------------------------------------------------------------------
    // STEP 1: Dynamic System Prompt Generation for UI
    // -----------------------------------------------------------------------
    println!("--- Step 1: Dynamic System Prompt Generation for UI ---");
    // Create sample project files
    let src_dir = workspace_root.join("src");
    fs::create_dir_all(&src_dir)?;
    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, "fn main() {\n    println!(\"Hello World\");\n}\n")?;

    let prompt_builder = SystemPromptBuilder::new(Some(workspace_root.clone()))
        .with_custom_instructions("User requested performance refactoring.")
        .with_active_file(&main_rs, "fn main() {\n    println!(\"Hello World\");\n}\n");

    let master_prompt = prompt_builder.build();
    println!("Generated Dynamic System Prompt Length: {} chars", master_prompt.len());
    println!("Contains Persona: {}", master_prompt.contains("Zenthree autonomous software engineering agent"));
    println!("Contains Host Detection: {}", master_prompt.contains("<host_environment>"));
    println!("Contains Active File Context: {}\n", master_prompt.contains("active_documents"));

    // -----------------------------------------------------------------------
    // STEP 2: Streaming UI Events Pipeline Simulation
    // -----------------------------------------------------------------------
    println!("--- Step 2: Streaming UI Events Pipeline ---");
    let mut ui_event_log = Vec::new();
    let mut record_event = |event: AgentEvent| {
        println!("  [UI_EVENT] {:?}", event);
        ui_event_log.push(event);
    };

    record_event(AgentEvent::Thinking {
        delta: "Analyzing project structure and compiler diagnostics...".to_string(),
    });
    record_event(AgentEvent::StreamChunk {
        delta: "I will now refactor `main.rs` to use buffered I/O.".to_string(),
    });
    record_event(AgentEvent::ToolStarted {
        tool: "edit_file".to_string(),
        summary: "Applying buffered I/O optimization to src/main.rs".to_string(),
    });
    record_event(AgentEvent::ToolFinished {
        tool: "edit_file".to_string(),
        ok: true,
        output_snippet: "✓ Successfully edited src/main.rs".to_string(),
        duration_ms: 42,
    });
    println!("Emitted {} events cleanly into UI stream.\n", ui_event_log.len());

    // -----------------------------------------------------------------------
    // STEP 3: Interactive Tool Permission Gating (Modal / Card)
    // -----------------------------------------------------------------------
    println!("--- Step 3: Interactive Tool Permission Gating ---");
    let perm_engine = PermissionEngine::new();

    // Terminal calls confirm by default
    let decision = perm_engine.evaluate("terminal", "cargo build --release");
    println!("Initial Engine Decision: {:?}", decision);

    if let ToolPermissionDecision::Confirm { reason, suggested_action } = decision {
        let perm_req = PendingPermissionRequest {
            call_id: "call_9981".to_string(),
            tool_name: "terminal".to_string(),
            reason,
            suggested_action: Some(suggested_action),
        };
        println!("  -> UI Displays Permission Card: [Allow Once] [Always Allow] [Deny]");
        println!("     Tool: {}", perm_req.tool_name);
        println!("     Reason: {}", perm_req.reason);

        // User clicks "Always Allow" in UI
        println!("  -> User Clicks: [Always Allow]");
        let outcome = perm_engine.handle_user_response(
            "terminal",
            "cargo build --release",
            UserPermissionResponse::AlwaysAllow,
        );
        println!("  -> Permission Outcome: {:?}", outcome);
        assert_eq!(outcome, ToolPermissionDecision::Allow);

        // Subsequent invocations of the same command are now auto-approved
        let next_decision = perm_engine.evaluate("terminal", "cargo build --release");
        println!("  -> Subsequent Call Decision: {:?}", next_decision);
        assert_eq!(next_decision, ToolPermissionDecision::Allow);
    }
    println!("Permission Gating successfully verified.\n");

    // -----------------------------------------------------------------------
    // STEP 4: Multi-File Checkpoint Snapshots & Live Matrix Diff Rollback
    // -----------------------------------------------------------------------
    println!("--- Step 4: Multi-File Checkpoint Snapshots & Matrix Diff ---");
    let mut timeline = CheckpointTimeline::new();
    let turn_id = 101u64;
    timeline.capture_turn(
        turn_id,
        "Refactor main.rs for buffered I/O",
        &[&main_rs],
        &workspace_root,
    );

    let original_code = fs::read_to_string(&main_rs)?;
    let modified_code = "use std::io::{self, Write};\n\nfn main() -> io::Result<()> {\n    let mut stdout = io::stdout().lock();\n    writeln!(stdout, \"Hello World\")?;\n    Ok(())\n}\n";
    fs::write(&main_rs, modified_code)?;

    let hunks = MatrixDiff::compute_diff(&original_code, modified_code);
    println!("Live Matrix Diff Calculated: {} hunks", hunks.len());
    for h in &hunks {
        println!("  {:?}", h);
    }

    println!("Checkpoint timeline tracking: {} checkpoints", timeline.len());

    // Simulate User Clicking [Reject / Revert All] in UI
    println!("User clicks [Reject / Revert All] in UI Review Bar...");
    let report = timeline.revert_turn(turn_id, &workspace_root)?;
    println!("Revert Report: {} files restored cleanly", report.restored_files.len());

    let restored_code = fs::read_to_string(&main_rs)?;
    assert_eq!(restored_code, original_code);
    println!("File on disk successfully matched pre-turn state:\n{}", restored_code);

    println!("===============================================================================");
    println!("   PHASE 10 CROSS-CRATE UI INTEGRATION COMPLETE & FULLY OPERATIONAL!           ");
    println!("===============================================================================\n");

    Ok(())
}
