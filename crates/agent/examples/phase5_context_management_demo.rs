//! Phase 5: Context Resilience & Large Output Spiller Example
//!
//! Demonstrates how Zenthree manages context limits and prevents API errors:
//! - Spilling oversized tool outputs (>150KB) to `.zenthree/tmp/tool_spill_<uuid>.txt`
//!   with actionable navigation instructions (`read_file` / `grep`)
//! - Historical tool result pruning: compresses older verbose tool outputs
//!   from completed turns while keeping active turn outputs 100% intact
//! - Sliding window conversation compaction when exceeding 75% context capacity,
//!   producing structured `<conversation_summary>` handoffs
//! - Token Accountant tracking input, output, and cached tokens across turns
//!
//! Run with: `cargo run --example phase5_context_management_demo -p agent`

use agent::context_management::{
    ContextManager, ConversationCompactor, LargeOutputSpiller,
    ToolPairPruner,
};
use agent::types::ChatMessage;
use tempfile::tempdir;

fn main() {
    println!("============================================================");
    println!("   ZENTHREE AGENT: PHASE 5 CONTEXT RESILIENCE DEMO          ");
    println!("============================================================\n");

    let temp_workspace = tempdir().expect("Failed to create temporary workspace");
    let ws = temp_workspace.path().to_path_buf();

    // ---------------------------------------------------------------------
    // 1. Large Output Spiller
    // ---------------------------------------------------------------------
    println!("--- 1. Large Output Spiller (Goose/Zenthree Standard) ---");
    // Use threshold of 2,000 characters for demo clarity
    let spiller = LargeOutputSpiller::with_threshold(2_000, Some(ws.clone()));

    println!("Active Spiller Threshold: {} chars (~500 tokens)", spiller.threshold());

    // A. Small tool output (passes through inline)
    let normal_output = "Finished release [optimized] target(s) in 2.15s\nRunning target/release/zenthree";
    let small_res = spiller.process_output("terminal", normal_output);
    println!("\n[Small Output] Spilled: {} | Chars: {}", small_res.is_spilled, small_res.original_char_count);
    println!("Inline Result:\n{}", small_res.rendered_content);

    // B. Oversized tool output (e.g. huge build log or cat of large file)
    let huge_payload = "TRACE 2026-09-18T14:50:00Z [lsp::server] indexing file components/widget.rs...\n".repeat(60);
    let large_res = spiller.process_output("read_file", &huge_payload);
    println!("\n[Large Output] Spilled: {} | Chars: {}", large_res.is_spilled, large_res.original_char_count);
    if let Some(ref path) = large_res.spill_path {
        println!("Spill File Written to: {}", path.display());
        assert!(path.exists(), "Spill file must exist on disk");
    }
    println!("\nRendered Model Pointer Prompt:\n------------------------------------------------------------");
    println!("{}", large_res.rendered_content);
    println!("------------------------------------------------------------");

    // ---------------------------------------------------------------------
    // 2. Historical Tool Pair Pruner (VSCode/Zed Pattern)
    // ---------------------------------------------------------------------
    println!("\n--- 2. Historical Tool Result Pruner ---");
    let pruner = ToolPairPruner::with_char_budget(150);

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are Zenthree Autonomous Agent.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Please build and run tests for the project.".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Running cargo test via terminal...".to_string(),
        },
        // Turn 1 Tool Result: Very verbose cargo test output
        ChatMessage {
            role: "tool".to_string(),
            content: "running 120 tests\ntest test_init ... ok\ntest test_db ... ok\n".to_string()
                + &"test verbose_log_line ... ok\n".repeat(30),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Tests passed. Now let's check git status.".to_string(),
        },
        // Turn 2: Active turn (most recent exchange)
        ChatMessage {
            role: "user".to_string(),
            content: "Go ahead with git status.".to_string(),
        },
        ChatMessage {
            role: "tool".to_string(),
            content: "On branch main\nChanges not staged for commit:\n  modified: Cargo.toml\n".to_string(),
        },
    ];

    println!("Original message count: {}", messages.len());
    println!("Historical Turn 1 tool result length: {} chars", messages[3].content.len());

    // Prune history, declaring the last 2 messages as the active turn
    let (pruned_messages, pruning_report) = pruner.prune_history(&messages, 2);

    println!("\nPruning Report: {} tool output(s) pruned | {} chars saved (~{} tokens)",
        pruning_report.pruned_count,
        pruning_report.characters_saved,
        pruning_report.estimated_tokens_saved,
    );

    println!("\nPruned Historical Tool Message (Turn 1):");
    println!("  -> {}", pruned_messages[3].content);

    println!("\nActive Turn Tool Message (Turn 2 - UNTOUCHED):");
    println!("  -> {}", pruned_messages[6].content.trim());

    // ---------------------------------------------------------------------
    // 3. Sliding Window Conversation Compactor (Zed Auto-Compaction)
    // ---------------------------------------------------------------------
    println!("\n--- 3. Sliding Window Conversation Compactor ---");
    // Set 500 token capacity and 75% threshold (triggers at 375 tokens)
    let compactor = ConversationCompactor::with_settings(0.75, 500, 2);

    let mut long_conversation = Vec::new();
    long_conversation.push(ChatMessage {
        role: "system".to_string(),
        content: "System prompt: Maintain clean architecture and strict security.".to_string(),
    });
    long_conversation.push(ChatMessage {
        role: "user".to_string(),
        content: "Refactor error handling across all core crates.".to_string(),
    });

    for step in 1..=5 {
        long_conversation.push(ChatMessage {
            role: "assistant".to_string(),
            content: format!("Step {}: Checked errors in crate module and updated trait.", step),
        });
        long_conversation.push(ChatMessage {
            role: "tool".to_string(),
            content: format!("Action: updated file crate_{}.rs\n{}", step, "OK: replacement hunk applied.\n".repeat(15)),
        });
    }

    // Active tail turns
    long_conversation.push(ChatMessage {
        role: "assistant".to_string(),
        content: "All crates refactored. Ready for final cargo check.".to_string(),
    });
    long_conversation.push(ChatMessage {
        role: "user".to_string(),
        content: "Run cargo check now.".to_string(),
    });

    let (compacted_messages, compaction_report) = compactor.compact(&long_conversation);

    println!("Compacted: {} | Messages: {} -> {} | Tokens: {} -> {} (saved {})",
        compaction_report.was_compacted,
        compaction_report.original_message_count,
        compaction_report.compacted_message_count,
        compaction_report.original_estimated_tokens,
        compaction_report.compacted_estimated_tokens,
        compaction_report.tokens_saved,
    );

    println!("\nInjected Handoff Summary Structure:");
    println!("------------------------------------------------------------");
    for msg in &compacted_messages {
        if msg.content.contains("<conversation_summary>") {
            println!("{}", msg.content);
        }
    }
    println!("------------------------------------------------------------");

    // ---------------------------------------------------------------------
    // 4. Token Accountant & Unified ContextManager
    // ---------------------------------------------------------------------
    println!("\n--- 4. Token Accountant & ContextManager Pipeline ---");
    let mut manager = ContextManager::new(Some(ws), 128_000);

    // Record turn usage
    manager.accountant_mut().record_usage(14_500, 1_200, 8_000);
    println!("Total Input Tokens:   {}", manager.accountant().total_input_tokens);
    println!("Total Output Tokens:  {}", manager.accountant().total_output_tokens);
    println!("Total Cached Tokens:  {}", manager.accountant().total_cached_tokens);
    println!("Last Turn Tokens:     {}", manager.accountant().last_turn_tokens);
    println!("Context Window Limit: {}", manager.accountant().max_context_tokens);
    println!("Capacity Ratio:       {:.2}%", manager.accountant().current_capacity_ratio() * 100.0);
    println!("Approaching 75% limit: {}", manager.accountant().is_approaching_limit(0.75));

    println!("\n✓ PHASE 5 CONTEXT MANAGEMENT DEMO COMPLETED SUCCESSFULLY!");
}
