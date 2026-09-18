//! Phase 6: Live Streaming Diff & Multi-Buffer Checkpoints Example
//!
//! Demonstrates how Zenthree computes live streaming diffs and manages turn checkpoints:
//! - O(M) memory dynamic programming matrix line diff with column swapping
//! - Live streaming diff accumulation as token chunks stream from an LLM
//! - In-memory BufferEditSession with search/replace and atomic commit
//! - Multi-buffer turn snapshots enabling atomic single-click rollback (`Undo Turn`)
//!
//! Run with: `cargo run --example phase6_diff_checkpoints_demo -p agent`

use agent::diff::{
    BufferEditSession, CheckpointTimeline, MatrixDiff, StreamingDiff,
};
use std::fs;
use tempfile::tempdir;

fn main() {
    println!("============================================================");
    println!("   ZENTHREE AGENT: PHASE 6 STREAMING DIFF & CHECKPOINTS     ");
    println!("============================================================\n");

    let temp_workspace = tempdir().expect("Failed to create temporary workspace");
    let ws = temp_workspace.path();

    // ---------------------------------------------------------------------
    // 1. O(M) Memory Matrix Line Diff & Edit Distance
    // ---------------------------------------------------------------------
    println!("--- 1. Dynamic Programming Line Diff & O(M) Edit Distance ---");
    let original_code = "\
fn calculate_total(items: &[Item]) -> u64 {
    let mut total = 0;
    for item in items {
        total += item.price;
    }
    total
}";

    let modified_code = "\
fn calculate_total(items: &[Item]) -> u64 {
    // Fast path: use iterator sum with overflow protection
    items.iter().map(|i| i.price).sum()
}";

    let distance = MatrixDiff::edit_distance(original_code, modified_code);
    println!("Line-level edit distance: {} operation(s)", distance);

    let ops = MatrixDiff::compute_diff(original_code, modified_code);
    println!("Total Diff Operations: {}", ops.len());

    let unified_diff = MatrixDiff::format_unified(&ops);
    println!("\nUnified Line Diff Output:\n------------------------------------------------------------");
    print!("{}", unified_diff);
    println!("------------------------------------------------------------");

    // ---------------------------------------------------------------------
    // 2. Live Streaming Token Accumulation
    // ---------------------------------------------------------------------
    println!("\n--- 2. Live Streaming Diff (Token Stream Emulation) ---");
    let base_code = "pub fn serve() {\n    listen(3000);\n}\n";
    let mut stream = StreamingDiff::new(base_code);

    let chunks = vec![
        "pub fn serve() {\n",
        "    // Initializing metrics\n",
        "    init_telemetry();\n",
        "    listen(8080);\n",
        "}\n",
    ];

    println!("Original Code:\n{}", base_code);
    println!("Emulating 120 FPS token streaming chunks from LLM:");
    for (i, chunk) in chunks.iter().enumerate() {
        let current_ops = stream.push_chunk(chunk);
        println!("  [Chunk {}] Received {:?} -> {} active diff op(s)", i + 1, chunk.trim(), current_ops.len());
    }

    println!("\nFinal Accumulated Streaming Diff:\n------------------------------------------------------------");
    print!("{}", stream.finalize());
    println!("------------------------------------------------------------");

    // ---------------------------------------------------------------------
    // 3. In-Memory BufferEditSession
    // ---------------------------------------------------------------------
    println!("\n--- 3. In-Memory BufferEditSession ---");
    let file_path = ws.join("src/network.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    let initial_network_rs = "pub const RETRIES: u32 = 3;\npub const TIMEOUT: u64 = 10;\n";
    fs::write(&file_path, initial_network_rs).unwrap();

    let mut session = BufferEditSession::new(&file_path, initial_network_rs);
    println!("Buffer opened for: {:?}", session.file_path());
    println!("Initial is_dirty: {}", session.is_dirty());

    // Apply patch block
    println!("\nApplying surgical patch in-memory...");
    let status = session
        .apply_patch_block("pub const TIMEOUT: u64 = 10;", "pub const TIMEOUT: u64 = 60;")
        .expect("Patch must apply cleanly");

    println!("Post-patch is_dirty: {}", status.is_dirty);
    println!("Working Buffer Content:\n{}", session.working_content());

    // Test atomic commit
    println!("Committing buffer atomically to filesystem...");
    let bytes_written = session.commit(ws, None).expect("Commit must succeed");
    println!("✓ Committed {} bytes. Post-commit is_dirty: {}", bytes_written, session.is_dirty());

    let disk_content = fs::read_to_string(&file_path).unwrap();
    println!("Disk Content Verification:\n{}", disk_content);

    // ---------------------------------------------------------------------
    // 4. Multi-Buffer Turn Checkpoints & Atomic Rollback (Undo Turn)
    // ---------------------------------------------------------------------
    println!("\n--- 4. Multi-Buffer Turn Checkpoints (Atomic Rollback) ---");
    let mut timeline = CheckpointTimeline::new();

    let file_existing = ws.join("src/service.rs");
    let file_created = ws.join("src/handler.rs");

    // Existing file before turn
    fs::write(&file_existing, "pub fn handle_v1() -> &'static str { \"v1\" }\n").unwrap();
    assert!(!file_created.exists(), "handler.rs must not exist prior to turn");

    // Capture pre-turn snapshot
    let turn_id = 42;
    println!("Capturing Turn #{} pre-modification snapshots...", turn_id);
    timeline.capture_turn(
        turn_id,
        "Upgrade service to v2 and add handler",
        &[&file_existing, &file_created],
        ws,
    );
    println!("Timeline active checkpoints: {}", timeline.len());

    // Turn modifications: modify service.rs and create handler.rs
    fs::write(&file_existing, "pub fn handle_v2() -> &'static str { \"v2\" }\n").unwrap();
    fs::write(&file_created, "pub struct Handler;\n").unwrap();

    println!("\n[During Turn #{}]", turn_id);
    println!("  - service.rs content: {}", fs::read_to_string(&file_existing).unwrap().trim());
    println!("  - handler.rs exists:  {}", file_created.exists());

    // User executes "Undo Turn"
    println!("\nExecuting `undo_latest` (Reverting Turn #{})...", turn_id);
    let revert_report = timeline.undo_latest(ws).unwrap().expect("Revert report expected");

    println!("Revert Report:");
    println!("  - Restored files ({}):", revert_report.restored_files.len());
    for f in &revert_report.restored_files {
        println!("    ✓ {:?}", f);
    }
    println!("  - Deleted files ({}):", revert_report.deleted_files.len());
    for f in &revert_report.deleted_files {
        println!("    ✓ {:?}", f);
    }

    println!("\nPost-Revert State Verification:");
    println!("  - service.rs content: {}", fs::read_to_string(&file_existing).unwrap().trim());
    println!("  - handler.rs exists:  {}", file_created.exists());
    assert!(!file_created.exists(), "Created file must be deleted on turn revert");
    assert_eq!(
        fs::read_to_string(&file_existing).unwrap(),
        "pub fn handle_v1() -> &'static str { \"v1\" }\n"
    );

    println!("\n✓ PHASE 6 STREAMING DIFF & CHECKPOINTS DEMO COMPLETED SUCCESSFULLY!");
}
