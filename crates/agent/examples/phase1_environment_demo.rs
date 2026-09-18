//! Phase 1: Agent Environment & Sandboxing Example
//!
//! Demonstrates how Zenthree secures the agent's physical execution boundary:
//! - WorkspaceJail: path confinement and traversal blocking
//! - ProtectedPathGuard: .git metadata & .env secret protection
//! - SecretScrubber: pattern-based credential redaction
//! - ProcessRunner: non-hanging execution with timeouts and output clipping
//! - HostEnvironment: toolchain discovery for prompt context
//!
//! Run with: `cargo run --example phase1_environment_demo -p agent`

use agent::environment::{
    EnvironmentSanitizer, HostEnvironment, ProcessExecutionOptions,
    ProcessRunner, ProtectedPathGuard, SecretScrubber, WorkspaceJail,
};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

fn main() {
    println!("============================================================");
    println!("   ZENTHREE AGENT: PHASE 1 ENVIRONMENT & SANDBOX DEMO       ");
    println!("============================================================\n");

    // ---------------------------------------------------------------------
    // 1. Workspace Jail Confinement
    // ---------------------------------------------------------------------
    println!("--- 1. Workspace Jail Confinement ---");
    let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let jail = WorkspaceJail::new(vec![&current_dir]);

    println!("Primary Workspace Root: {:?}", jail.primary_root());

    // Valid path inside jail
    let valid_path = Path::new("src/lib.rs");
    match jail.validate_read(valid_path) {
        Ok(resolved) => println!("✓ Safe Read Path Allowed: {}", resolved.display()),
        Err(e) => println!("✗ Unexpected Read Failure: {}", e),
    }

    // Path traversal attack attempt
    let traversal_attack = Path::new("../../etc/passwd");
    match jail.validate_read(traversal_attack) {
        Ok(_) => println!("✗ Traversal Attack Succeeded (BAD!)"),
        Err(e) => println!("✓ Traversal Attack Blocked by Jail: {}", e),
    }

    // ---------------------------------------------------------------------
    // 2. Protected Invariant Paths (.git and secrets)
    // ---------------------------------------------------------------------
    println!("\n--- 2. Invariant Protected Paths Guard ---");
    let test_git_hook = Path::new(".git/hooks/pre-commit");
    let test_env_file = Path::new(".env.production");
    let test_source = Path::new("src/main.rs");

    println!("Is '.git/hooks/pre-commit' write-blocked? {}", ProtectedPathGuard::is_write_blocked(test_git_hook));
    println!("Is '.env.production' write-blocked? {}", ProtectedPathGuard::is_write_blocked(test_env_file));
    println!("Is 'src/main.rs' write-blocked? {}", ProtectedPathGuard::is_write_blocked(test_source));

    // ---------------------------------------------------------------------
    // 3. Secret Scrubber & Pattern Redaction
    // ---------------------------------------------------------------------
    println!("\n--- 3. Secret Scrubber & Pattern Redaction ---");
    let scrubber = SecretScrubber::new();
    let raw_log = "Connected to remote API using sk-1234567890abcdef1234567890abcdef and ghp_1234567890abcdef1234567890abcdef1234 with git clone https://user:super_secret_password@github.com/repo.git";

    println!("Raw Payload With Secrets:\n  {}", raw_log);
    let sanitized = scrubber.scrub(raw_log);
    println!("\nScrubbed Output (Zero Leakage):\n  {}", sanitized);

    // ---------------------------------------------------------------------
    // 4. Sanitized Subprocess Execution & Line Clipping
    // ---------------------------------------------------------------------
    println!("\n--- 4. Process Runner With Anti-Hang & Output Clipping ---");
    let runner = ProcessRunner::new();

    // Demonstrate git command sanitization
    let raw_git_cmd = "git log -n 5";
    let sanitized_git_cmd = EnvironmentSanitizer::sanitize_git_command(raw_git_cmd);
    println!("Original Command:  {}", raw_git_cmd);
    println!("Sanitized Command: {} (automatically inserted --no-pager)", sanitized_git_cmd);

    // Run safe command
    let opts = ProcessExecutionOptions {
        command: "cargo --version".to_string(),
        cwd: current_dir.clone(),
        timeout: Duration::from_secs(5),
        head_lines: Some(2),
        tail_lines: Some(2),
        env_overrides: HashMap::new(),
    };

    match runner.execute(opts, None, None) {
        Ok(res) => {
            println!("Process Exit Code: {:?}", res.exit_code);
            println!("Process Output:    {}", res.clipped_output.trim());
        }
        Err(e) => println!("Process error: {}", e),
    }

    // ---------------------------------------------------------------------
    // 5. Host Toolchain Discovery
    // ---------------------------------------------------------------------
    println!("\n--- 5. Host Environment Discovery & Context Generation ---");
    let host = HostEnvironment::detect();
    println!("{}", host.format_prompt_block());

    println!("============================================================");
    println!("   PHASE 1 ENVIRONMENT DEMO COMPLETED SUCCESSFULLY!         ");
    println!("============================================================");
}
