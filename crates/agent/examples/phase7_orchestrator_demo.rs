//! Phase 7: FSM Orchestrator & Execution Loop Example
//!
//! Demonstrates the complete deterministic agent turn loop in Zenthree:
//! - Dynamic SystemPromptBuilder assembling all architectural layers into the master prompt
//! - AgentStateMachine coordinating: Init -> Context -> LLM -> Permission Gate -> Tools -> Spill -> Compact
//! - Security permission gating with HardcodedSecurityRules
//! - Mid-flight user steering (cancellation & redirection) via SteeringController
//! - Turn session history with DAG of turns, tool calls, and results
//!
//! Run with: `cargo run --example phase7_orchestrator_demo -p agent`

use agent::orchestrator::{
    AgentStateMachine, AgentTurnState, SteeringSignal, SystemPromptBuilder,
};
use agent::tools::AgentToolCall;
use std::fs;
use tempfile::tempdir;

fn main() {
    println!("============================================================");
    println!("    ZENTHREE AGENT: PHASE 7 FSM ORCHESTRATOR DEMO          ");
    println!("============================================================\n");

    let temp_workspace = tempdir().expect("Failed to create temporary workspace");
    let ws = temp_workspace.path().to_path_buf();

    // Setup dummy files in workspace
    let sample_file = ws.join("src/lib.rs");
    fs::create_dir_all(sample_file.parent().unwrap()).unwrap();
    fs::write(&sample_file, "pub fn greeting() -> &'static str { \"hello world\" }\n").unwrap();

    // ---------------------------------------------------------------------
    // 1. Dynamic Master System Prompt Builder
    // ---------------------------------------------------------------------
    println!("--- 1. Dynamic Master System Prompt Generation ---");
    let builder = SystemPromptBuilder::new(Some(ws.clone()))
        .with_custom_instructions("Always ensure zero clippy warnings.")
        .with_active_file(&sample_file, "pub fn greeting() -> &'static str { \"hello world\" }");

    let master_prompt = builder.build();
    println!("Master System Prompt Length: {} chars (~{} tokens)", master_prompt.len(), master_prompt.len() / 4);
    println!("Prompt Subsystems Verified:");
    println!("  ✓ Base Persona & Operating Principles: {}", master_prompt.contains("Zenthree autonomous software engineering agent"));
    println!("  ✓ Host Environment Detection (Phase 1):  {}", master_prompt.contains("## System Environment"));
    println!("  ✓ Few-Shot Tool Examples (Phase 4):      {}", master_prompt.contains("<tool_call_examples>"));
    println!("  ✓ Active Working Buffers:                {}", master_prompt.contains("<active_file path="));

    // ---------------------------------------------------------------------
    // 2. End-to-End Deterministic Turn Execution Loop
    // ---------------------------------------------------------------------
    println!("\n--- 2. End-to-End Deterministic Turn Loop ---");
    let mut fsm = AgentStateMachine::new(Some(ws.clone()));

    // Step A: Prepare Context
    println!("State: {:?}", fsm.state);
    fsm.prepare_context();
    println!("State after prepare_context: {:?}", fsm.state);

    // Step B: Start User Turn
    let user_prompt = "Examine src/lib.rs and check git status";
    let turn_id = fsm.session.start_user_turn(user_prompt);
    println!("\nTurn #{} Started: \"{}\"", turn_id, user_prompt);

    // Step C: Simulated LLM Tool Call Generation
    let tool_calls = vec![
        AgentToolCall::ReadFile {
            path: "src/lib.rs".to_string(),
            start_line: None,
            end_line: None,
        },
        AgentToolCall::RunCommand {
            cmd: "rm -rf /".to_string(),
        },
    ];

    println!("Model requested {} tool call(s)", tool_calls.len());

    // Step D: Permission Gate Evaluation
    let (approved, denied) = fsm.evaluate_permissions(tool_calls);
    println!("State in Permission Gate: {:?}", fsm.state);
    println!("  ✓ Approved calls: {} ({})", approved.len(), approved.first().map(|c| c.tool_name()).unwrap_or("none"));
    println!("  ✓ Denied calls:   {} ({})", denied.len(), denied.first().map(|c| c.tool_name()).unwrap_or("none"));

    // Step E: Tool Execution & Spilling
    let results = fsm.execute_and_spill_tools(turn_id, approved);
    println!("\nExecuted {} tool(s):", results.len());
    for r in &results {
        println!("  - Tool '{}' -> {} lines output", r.tool_name, r.output.lines().count());
    }

    // Step F: Context Compaction & Turn Completion
    fsm.finalize_turn(turn_id, "src/lib.rs contains `greeting()`. No uncommitted changes.");
    println!("\nState after finalize_turn: {:?}", fsm.state);
    println!("Total Session Messages: {}", fsm.session.messages.len());
    println!("FSM Telemetry Transitions: {}", fsm.events.len());

    // ---------------------------------------------------------------------
    // 3. Mid-Flight User Steering (Cancellation & Guidance)
    // ---------------------------------------------------------------------
    println!("\n--- 3. Mid-Flight User Steering ---");

    // Scenario A: User Redirection
    let turn_2 = fsm.session.start_user_turn("Run full test suite");
    println!("Turn #{} Started: \"Run full test suite\"", turn_2);

    fsm.steering.emit(SteeringSignal::Redirect {
        user_feedback: "Don't run integration tests yet, run unit tests only.".to_string(),
    });

    let interrupted = fsm.check_steering(turn_2);
    println!("Was turn cancelled? {}", interrupted);
    println!("Redirect injected into message history: {}", fsm.session.messages.last().unwrap().content);

    // Scenario B: User Immediate Stop / Cancellation
    let turn_3 = fsm.session.start_user_turn("Compile massive debug binary");
    println!("\nTurn #{} Started: \"Compile massive debug binary\"", turn_3);

    fsm.steering.emit(SteeringSignal::Cancel {
        reason: "User clicked Stop in UI".to_string(),
    });

    let was_cancelled = fsm.check_steering(turn_3);
    println!("Was turn cancelled? {}", was_cancelled);
    println!("Final State after Stop: {:?}", fsm.state);
    assert!(matches!(fsm.state, AgentTurnState::Cancelled { .. }));

    println!("\n✓ PHASE 7 FSM ORCHESTRATOR DEMO COMPLETED SUCCESSFULLY!");
}
