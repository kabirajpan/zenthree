# Zenthree Autonomous Agent Architecture & Engineering Specification

> **Status:** Definitive Architectural Specification  
> **Target Subsystem:** `apps/zenthree/crates/agent`  
> **Workspace Members:** `crates/agent`, `crates/editor`, `crates/zenthree`, `crates/syntax`, `crates/formatter`  
> **References:** Zed Editor (`references/zed`), Goose Agent (`references/goose`), VS Code (`references/vscode`)  
> **Standards:** `agentskills.io` (Skills Open Standard), Model Context Protocol (MCP 2.0), Agent Client Protocol (ACP)

---

# Table of Contents
1. [Executive Summary & Core Principles](#1-executive-summary--core-principles)
2. [Reference Systems Deep-Dive & Comparative Synthesis](#2-reference-systems-deep-dive--comparative-synthesis)
3. [Exhaustive Modular Structure for `crates/agent`](#3-exhaustive-modular-structure-for-cratesai)
4. [Deep Technical Specifications](#4-deep-technical-specifications)
   - [4.1 Module: `environment/` (Workspace Jail, Protected Paths, Process Runner)](#41-module-environment)
   - [4.2 Module: `permissions/` (Security Guardrails & Hardcoded Denials)](#42-module-permissions)
   - [4.3 Module: `skills/` (agentskills.io Open Standard & Catalog)](#43-module-skills)
   - [4.4 Module: `rules/` (Multi-Standard Rule Ingestion)](#44-module-rules)
   - [4.5 Module: `tools/` (Unified Tool Registry & Native Suites)](#45-module-tools)
   - [4.6 Module: `context_management/` (Large Output Spiller & Compactor)](#46-module-context_management)
   - [4.7 Module: `diff/` (Matrix Rope Diff Engine & Multi-File Checkpoints)](#47-module-diff)
   - [4.8 Module: `orchestrator/` (Finite State Machine & Turn Coordinator)](#48-module-orchestrator)
   - [4.9 Module: `protocols/` (MCP 2.0 & ACP Bridges)](#49-module-protocols)
   - [4.10 Module: `providers/` (Streaming Multi-Provider Drivers)](#410-module-providers)
   - [4.11 Module: `types/` (Domain Types & UI Event Contracts)](#411-module-types)
5. [Cross-Crate Integration Architecture (`editor` and `zenthree`)](#5-cross-crate-integration-architecture)
6. [Phased Implementation Roadmap & Verification Test Plans](#6-phased-implementation-roadmap--verification-test-plans)

---

# 1. Executive Summary & Core Principles

The Zenthree Agent System is an autonomous, high-performance, sandboxed software engineering agent runtime built natively in Rust. It lives as a headless library within `crates/agent` and coordinates tightly with `crates/editor` (buffer text ropes, cursor selections) and `crates/zenthree` (GPUI/Zenthra-render graphical user interface).

### Core Architectural Axioms
1. **Strict Modular Decoupling**: Every functional subsystem (`environment`, `permissions`, `skills`, `rules`, `tools`, `context_management`, `diff`, `orchestrator`, `protocols`, `providers`) is completely decoupled and self-contained with well-defined trait interfaces.
2. **Deterministic Safety-First Design**: The agent cannot be tricked or instructed to destroy the host environment, delete git histories, or read out-of-bounds private secrets. Destructive commands are blocked at the lowest engine layer via unbypassable security rules.
3. **Progressive Disclosure of Context**: Rather than bloating prompt context with every file, skill, or rule, context is loaded progressively. Catalogs fit within strict byte budgets; detailed contents are fetched on-demand.
4. **Resilient Context Management**: Large outputs (>150KB) never overflow LLM context limits; they are automatically spilled to temporary storage with indexed search pointers. Old conversation turns are continuously compacted.
5. **Zero-Latency In-Buffer Feedback**: In-editor file modifications stream directly to the editor buffer at 120 FPS via an $O(N)$ dynamic programming matrix diff algorithm, giving immediate visual feedback before the LLM finishes generating.

---

# 2. Reference Systems Deep-Dive & Comparative Synthesis

### A. Zed Editor (`references/zed`)
* **`agent::thread` & `agent::agent`**: Thread conversation state represented as a Directed Acyclic Graph (DAG) with tree-branching, model switching, and subagent depth boundaries (`MAX_SUBAGENT_DEPTH = 1`).
* **`agent::sandboxing`**:
  * Write access strictly jailed to workspace roots (`sandbox_worktree_writable_paths`).
  * Critical git paths protected (`sandbox_git_dirs`): `.git/`, `.git/hooks/`, `.git/config`, git common directories.
  * System-level sandboxing via Apple Seatbelt (`sandbox-exec`) and Linux Bubblewrap (`bwrap`).
* **`agent::tool_permissions`**:
  * Unbypassable hardcoded denials (`HARDCODED_SECURITY_RULES`) blocking destructive commands (`rm -rf /`, `rm -rf ~`, `rm -rf .`, traversal escapes like `rm -rf /tmp/../../`).
  * POSIX command chaining validation using shell AST parsing to prevent smuggled commands.
  * Three-tiered decisions: `Allow`, `Confirm`, `Deny`.
* **`agent_skills`**:
  * Flat discovery across Builtin, Global (`~/.agents/skills/`), and Workspace (`<root>/.agents/skills/`).
  * `MAX_SKILL_FILE_SIZE` = 100KB, `MAX_SKILL_DESCRIPTIONS_SIZE` = 50KB total catalog budget.
* **`streaming_diff`**: Dynamic programming matrix rope diff using column-swapping memory management, emitting character and line hunk operations as tokens stream in.
* **`context_server`**: Model Context Protocol (MCP) client and server supporting Stdio, SSE, and full OAuth2 flows.

### B. Goose Agent (`references/goose`)
* **Finite State Machine Architecture (`state_machine/`)**:
  * Turns modeled as explicit atomic operators: `ops_skills` ➔ `ops_llm` ➔ `ops_steer` ➔ `ops_tool_approval` ➔ `ops_toolcalling` ➔ `ops_compaction`.
* **`large_response_handler.rs`**:
  * Large tool output spill-to-disk: When output > 200,000 characters, spills payload to `.tmp/goose_mcp_response_*.txt` and tells the model to inspect it using `read_file` or `grep`.
* **`ops_compaction.rs` & `ops_tool_pair_compaction.rs`**:
  * Token-aware compaction that prunes raw verbose tool payloads once processed and summarizes older dialogue turns into persistent context blocks.
* **`ops_steer.rs`**:
  * Real-time steering allowing the user to inject mid-flight guidance or cancel running commands without dropping thread context.

### C. VS Code (`references/vscode`)
* **`chatEditingCheckpointTimelineImpl.ts`**:
  * Checkpoint timelines across all modified buffers in a turn, allowing instant multi-file rollback ("Undo Turn" / "Revert Checkpoint").
* **`chatEditingSession.ts` & `chatEditingModifiedFileEntry.ts`**:
  * Multi-file working sets tracking accepted/rejected hunks per file during code generation.
* **`chatEditingEditorOverlay.ts`**:
  * Floating in-editor review bar ("Accept All", "Reject All", "Keep", "Next Change").

---

# 3. Exhaustive Modular Structure for `crates/agent`

```
apps/zenthree/crates/agent/
├── Cargo.toml
└── src/
    ├── lib.rs
    │
    ├── environment/                   # 1. RUNTIME ENVIRONMENT & SANDBOXING
    │   ├── mod.rs                     # AgentEnvironment trait & EnvironmentContext
    │   ├── workspace_jail.rs          # Canonicalization, path jailing, traversal blocker
    │   ├── protected_paths.rs         # Invariants: .git, .env*, .ssh, system paths
    │   ├── secret_scrubber.rs         # Pattern-based secret masking (API keys, tokens, auth)
    │   ├── process_runner.rs          # Subprocess & PTY runner, timeouts, cancellation
    │   ├── env_sanitizer.rs           # Sanitized variables (GIT_EDITOR=true, PAGER=cat, etc.)
    │   └── host_detection.rs          # Toolchain detection (cargo, rustc, node, python, git)
    │
    ├── permissions/                   # 2. SECURITY GUARDRAILS & POLICY ENGINE
    │   ├── mod.rs                     # PermissionEngine, PermissionRule, ToolPermissionDecision
    │   ├── hardcoded_denials.rs       # Static unbypassable denial patterns (rm -rf, dd, mkfs)
    │   ├── command_parser.rs          # POSIX shell chain parser (&&, ||, ;, pipes, subshells)
    │   ├── rules.rs                   # Tool permission policies (always_allow, always_confirm, always_deny)
    │   └── decision.rs                # Allow, Confirm { reason }, Deny { reason }
    │
    ├── skills/                        # 3. AGENT SKILLS SYSTEM (agentskills.io)
    │   ├── mod.rs                     # SkillRegistry, SkillLoader, SkillSummary
    │   ├── spec.rs                    # SKILL.md YAML frontmatter parser & schema
    │   ├── discovery.rs               # Flat discovery: Builtin -> Global -> Workspace
    │   ├── catalog.rs                 # Progressive disclosure catalog (token budget capped)
    │   └── executor.rs                # Skill runner & script invoker
    │
    ├── rules/                         # 4. WORKSPACE RULES INGESTION
    │   ├── mod.rs                     # RuleRegistry, RuleSource, RuleContext
    │   ├── scanner.rs                 # Multi-standard scanner (.cursorrules, AGENTS.md, etc.)
    │   ├── parser.rs                  # Extracts instructions, priority levels, glob triggers
    │   └── context_injection.rs       # Injects matched rules into system prompts
    │
    ├── tools/                         # 5. UNIFIED TOOL ENGINE & NATIVE SUITE
    │   ├── mod.rs                     # AgentTool trait, ToolRegistry, ToolCallResult
    │   ├── fs/                        # File system tools
    │   │   ├── mod.rs
    │   │   ├── read_file.rs           # Line-numbered chunked & offset file reader
    │   │   ├── write_file.rs          # Atomic file creator and overwriter
    │   │   ├── edit_file.rs           # In-buffer search-and-replace with fuzzy matching
    │   │   ├── list_directory.rs      # Directory tree crawler with gitignore filtering
    │   │   ├── delete_path.rs         # Permission-guarded file/dir deletion
    │   │   └── move_copy_path.rs      # Move, rename, and copy operations
    │   ├── search/                    # Codebase intelligence
    │   │   ├── mod.rs
    │   │   ├── grep.rs                # Ripgrep project regex search with exclusion filters
    │   │   └── find_path.rs           # Fast fuzzy path finder
    │   ├── terminal/                  # Terminal runner
    │   │   ├── mod.rs
    │   │   └── terminal_tool.rs       # PTY executor with head/tail output limit
    │   ├── lsp/                       # IDE & Compiler intelligence
    │   │   ├── mod.rs
    │   │   ├── diagnostics.rs         # Real-time LSP compiler/linter error inspector
    │   │   ├── definition.rs          # Jump-to-definition resolver
    │   │   └── references.rs          # Symbol usage & references locator
    │   ├── agent/                     # Meta & Collaboration
    │   │   ├── mod.rs
    │   │   ├── ask_user.rs            # Interactive human-in-the-loop questionnaire
    │   │   ├── load_skill.rs          # On-demand skill body loader
    │   │   └── spawn_subagent.rs      # Child agent delegation with scoped task limits
    │   └── mcp_bridge.rs              # Dynamic MCP tool proxy
    │
    ├── context_management/            # 6. RESILIENCE & CONTEXT WINDOW MANAGEMENT
    │   ├── mod.rs                     # ContextManager, TokenAccountant
    │   ├── large_output_spiller.rs    # Disk spiller (>150KB) returning compact pointers
    │   ├── compaction.rs              # Sliding window & semantic summarizer for old turns
    │   └── tool_pair_pruner.rs        # Prunes verbose raw tool results after model answers
    │
    ├── diff/                          # 7. LIVE STREAMING DIFF & CHECKPOINTS
    │   ├── mod.rs                     # StreamingDiff, BufferEditSession, Checkpoint
    │   ├── matrix_diff.rs             # O(N) memory dynamic programming matrix rope diff
    │   ├── edit_session.rs            # In-buffer partial JSON streaming parser & patcher
    │   └── checkpoint.rs              # Multi-file turn snapshot timeline (Undo/Revert Turn)
    │
    ├── orchestrator/                  # 8. FINITE STATE MACHINE & EXECUTION LOOP
    │   ├── mod.rs                     # AgentLoop, TurnState, AgentSession
    │   ├── state_machine.rs           # Explicit FSM: Init -> Catalog -> LLM -> Gate -> Tool -> Compact
    │   ├── session.rs                 # Thread conversation history (DAG of turns & messages)
    │   ├── steer.rs                   # Mid-flight user guidance & cancellation tokens
    │   └── prompt_builder.rs          # Dynamic system prompt generator
    │
    ├── protocols/                     # 9. EXTERNAL PROTOCOLS (MCP & ACP)
    │   ├── mod.rs
    │   ├── mcp/                       # Model Context Protocol (MCP 2.0)
    │   │   ├── mod.rs
    │   │   ├── client.rs              # MCP client manager
    │   │   ├── transport_stdio.rs     # Child process stdio JSON-RPC transport
    │   │   ├── transport_sse.rs       # HTTP Server-Sent Events transport
    │   │   └── types.rs               # MCP 2.0 schemas (tools, resources, prompts)
    │   └── acp/                       # Agent Client Protocol
    │       ├── mod.rs
    │       └── adapter.rs             # Editor-to-agent JSON-RPC communication bridge
    │
    ├── providers/                     # 10. MULTI-MODEL STREAMING DRIVERS
    │   ├── mod.rs                     # ModelProvider trait & ProviderRegistry
    │   ├── anthropic.rs               # Claude 3.7 Sonnet / Thinking / 3.5 Sonnet
    │   ├── openai.rs                  # GPT-4o / o1 / o3-mini
    │   ├── gemini.rs                  # Gemini 2.5 Flash / 2.5 Pro
    │   ├── groq.rs                    # Fast inference (Llama 3.3 70B, DeepSeek R1)
    │   ├── local.rs                   # Ollama / Llama.cpp local model runners
    │   └── types.rs                   # Unified request/response/stream data structures
    │
    └── types/                         # 11. CORE SHARED DOMAIN TYPES
        ├── mod.rs
        ├── message.rs                 # Message, Role, ContentPart, ThinkingBlock
        ├── events.rs                  # Streaming AgentEvent definitions for UI
        └── config.rs                  # Agent settings & user configurations
```

---

# 4. Deep Technical Specifications

## 4.1 Module: `environment/`

The Agent Environment establishes the physical boundary between the autonomous agent and the host operating system.

### `workspace_jail.rs`
* **Purpose**: Guarantees that all file operations remain confined within authorized project directories.
* **Mechanism**:
  * Resolves symlinks using `std::fs::canonicalize`.
  * Verifies path containment: `target_path.starts_with(&canonical_workspace_root)`.
  * Explicitly intercepts path traversal sequences (`..`, `.` components, URL-encoded variations `%2e%2e%2f`).
  * Exception: Whitelists `~/.agents/skills/` for global agent skills.
* **Primary Interface**:
  ```rust
  pub struct WorkspaceJail {
      roots: Vec<PathBuf>,
      allowed_globals: Vec<PathBuf>,
  }

  impl WorkspaceJail {
      pub fn new(roots: Vec<PathBuf>) -> Result<Self>;
      pub fn validate_read(&self, path: &Path) -> Result<PathBuf, SecurityViolation>;
      pub fn validate_write(&self, path: &Path) -> Result<PathBuf, SecurityViolation>;
      pub fn is_confined(&self, path: &Path) -> bool;
  }
  ```

### `protected_paths.rs`
* **Purpose**: Protects internal repository structures and sensitive user credentials from modification or unauthorized exfiltration.
* **Invariants**:
  * Git directories: `<root>/.git`, `.git/hooks/*`, `.git/config`, `git-common-dir`.
  * Environment secrets: `.env`, `.env.*`, `*.env`, `*.secret`.
  * User credentials: `~/.ssh/*`, `id_rsa*`, `~/.aws/*`, `~/.gnupg/*`, `~/.config/gh/*`.
* **Primary Interface**:
  ```rust
  pub struct ProtectedPathGuard;

  impl ProtectedPathGuard {
      pub fn is_write_blocked(path: &Path) -> bool;
      pub fn is_read_sensitive(path: &Path) -> bool;
      pub fn get_git_paths(workspace_root: &Path) -> Vec<PathBuf>;
  }
  ```

### `secret_scrubber.rs`
* **Purpose**: Prevents accidental leakage of credentials into agent logs, tool outputs, or model context windows.
* **Patterns Scrubbed**:
  * OpenAI / Anthropic / Gemini API keys (`sk-[a-zA-Z0-9_-]{20,}`, `ghp_[a-zA-Z0-9]{36}`, `xox[baprs]-[a-zA-Z0-9]{10,}`).
  * Private SSH keys (`-----BEGIN [A-Z ]+ PRIVATE KEY-----`).
  * Basic Authentication credentials in URLs (`https://user:password@host`).
  * AWS Secret Access Keys and bearer tokens.
* **Primary Interface**:
  ```rust
  pub struct SecretScrubber {
      patterns: Vec<regex::Regex>,
  }

  impl SecretScrubber {
      pub fn scrub(&self, input: &str) -> String;
      pub fn contains_secrets(&self, input: &str) -> bool;
  }
  ```

### `process_runner.rs` & `env_sanitizer.rs`
* **Purpose**: Spawns terminal processes safely in an isolated sub-shell.
* **Sanitization Invariants**:
  * Injects `PAGER=cat` and `GIT_EDITOR=true` to prevent child processes from hanging on pagers or text editors.
  * Prepends `--no-pager` and `--no-optional-locks` to read-only git calls.
  * Enforces runtime timeout (`timeout_ms`, default 60s) with active `tokio::select!` cancellation.
  * Employs `head_lines` and `tail_lines` output clipping: live output is streamed to the UI in full, but the return payload to the LLM is clipped to avoid blowing token budgets.
* **Primary Interface**:
  ```rust
  pub struct ProcessRunner;

  pub struct ProcessExecutionOptions {
      pub command: String,
      pub cwd: PathBuf,
      pub timeout: Duration,
      pub head_lines: Option<usize>,
      pub tail_lines: Option<usize>,
  }

  pub struct ProcessExecutionResult {
      pub exit_code: Option<i32>,
      pub stdout: String,
      pub stderr: String,
      pub combined_output: String,
      pub duration: Duration,
      pub timed_out: bool,
  }

  impl ProcessRunner {
      pub async fn execute(
          &self,
          opts: ProcessExecutionOptions,
          cancel: CancellationToken,
          on_chunk: impl FnMut(String) + Send,
      ) -> Result<ProcessExecutionResult>;
  }
  ```

### `host_detection.rs`
* **Purpose**: Automatically discovers the local development environment to provide context-aware prompt instructions.
* **Capabilities**:
  * Host OS (`linux`, `macos`, `windows`), Architecture (`x86_64`, `aarch64`), Shell (`bash`, `zsh`, `fish`, `powershell`).
  * Installed toolchains: Rust (`cargo`, `rustc`), Node.js (`node`, `npm`, `pnpm`, `yarn`), Python (`python3`, `pip`, `uv`), Go (`go`), Git (`git`), Docker (`docker`).
* **Primary Interface**:
  ```rust
  pub struct HostEnvironment {
      pub os: String,
      pub arch: String,
      pub shell: String,
      pub toolchains: Vec<ToolchainInfo>,
  }

  pub struct ToolchainInfo {
      pub name: String,
      pub binary_path: PathBuf,
      pub version: Option<String>,
  }

  impl HostEnvironment {
      pub fn detect() -> Self;
  }
  ```

---

## 4.2 Module: `permissions/`

The Permissions Engine is the security barrier that evaluates every tool invocation against hardcoded safety rules, user-defined preferences, and interactive prompts.

### `hardcoded_denials.rs`
* **Purpose**: Provides absolute, unbypassable security rules that cannot be overridden by settings or user confirmation prompts.
* **Rules**:
  1. `rm -rf /` or any flag permutation (`rm -rfv /`, `rm / -rf`, `rm -rf /*`).
  2. `rm -rf ~` or `rm -rf $HOME` or `rm -rf ${HOME}`.
  3. `rm -rf .` or `rm -rf ..` and all traversal bypasses (`rm -rf /tmp/../../`).
  4. Block-level filesystem destruction (`mkfs.*`, `dd if=... of=/dev/sd*`).
  5. Shell fork bombs (`:(){ :|:& };:`).
* **Primary Interface**:
  ```rust
  pub struct HardcodedSecurityRules;

  impl HardcodedSecurityRules {
      pub fn evaluate_terminal(command: &str) -> Option<String>;
      pub fn evaluate_filesystem(path: &Path, is_delete: bool) -> Option<String>;
  }
  ```

### `command_parser.rs`
* **Purpose**: Prevents malicious command injection and chaining bypasses.
* **Mechanism**:
  * Parses POSIX command lines into discrete sub-commands separated by `&&`, `||`, `;`, and `|`.
  * Forbids dangerous shell substitutions in auto-approved terminal runs (`$(...)`, backticks, `$((...))`, `<(...)`).
* **Primary Interface**:
  ```rust
  pub struct ShellCommandParser;

  impl ShellCommandParser {
      pub fn extract_subcommands(command_line: &str) -> Vec<String>;
      pub fn contains_substitutions(command_line: &str) -> bool;
  }
  ```

### `decision.rs` & `rules.rs`
* **Decision Enum**:
  ```rust
  #[derive(Debug, Clone, PartialEq, Eq)]
  pub enum ToolPermissionDecision {
      Allow,
      Confirm { reason: String, suggested_action: String },
      Deny { reason: String },
  }
  ```
* **Evaluation Hierarchy (Strict Priority)**:
  1. Hardcoded security denials ➔ `Deny`.
  2. Workspace jail / protected path violation ➔ `Deny`.
  3. User `always_deny` regex pattern ➔ `Deny`.
  4. User `always_confirm` regex pattern ➔ `Confirm`.
  5. User `always_allow` regex pattern ➔ `Allow`.
  6. Tool default setting (e.g. read tools allow by default, write/terminal confirm by default).

---

## 4.3 Module: `skills/`

Implements the open standard specification from `agentskills.io` for modular agent skills.

### `spec.rs` & Format
Each skill is defined in a `SKILL.md` file:
```markdown
---
name: high-performance-rust
description: Best practices, profiling, and idioms for high-throughput async Rust
disable_model_invocation: false
---
# Instructions
...
```
* Validation rules:
  * `name`: Regex `^[a-z0-9-]{1,64}$`.
  * `description`: 1 to 1024 characters.
  * Maximum `SKILL.md` file size: 100KB.

### `discovery.rs`
* Scans skills in flat priority order:
  1. Built-in skills (compiled statically into binary).
  2. Global skills (`~/.agents/skills/<name>/SKILL.md`).
  3. Workspace skills (`<workspace_root>/.agents/skills/<name>/SKILL.md`).
  * Workspace skills shadow global skills with the same name.

### `catalog.rs` & Progressive Disclosure
* **Catalog Budget**: Total size of skill descriptions injected into the model system prompt is capped at 50KB.
* **Mechanism**:
  * The system prompt only receives the **Catalog** (`name` and `description`).
  * Full skill bodies are loaded dynamically when the model calls `load_skill` or the user enters `/skill-name`.

---

## 4.4 Module: `rules/`

### Multi-Standard Rule Ingestion
* Scans workspace root for existing rule configurations:
  * `.rules`
  * `.cursorrules`
  * `.windsurfrules`
  * `.clinerules`
  * `.github/copilot-instructions.md`
  * `AGENT.md` / `AGENTS.md`
  * `CLAUDE.md`
  * `GEMINI.md`
* Aggregates active rules and formats them into a dedicated `<workspace_rules>` block in the system prompt.

---

## 4.5 Module: `tools/`

Unified tool registry providing standardized schema generation and execution.

### Tool Suite Specification
```rust
#[async_trait::async_trait]
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(
        &self,
        args: serde_json::Value,
        ctx: &AgentEnvironment,
    ) -> Result<ToolCallResult, ToolExecutionError>;
}
```

### Complete Native Tools:
1. **`read_file`**: Reads file content with 1-indexed line numbers. Accepts `start_line` and `end_line` for offset chunking.
2. **`write_file`**: Writes full content atomically using temporary file rename.
3. **`edit_file`**: In-buffer search-and-replace using fuzzy matching to tolerate minor indentation/whitespace variance. Preserves file indentation.
4. **`list_directory`**: Lists files and subdirectories with `.gitignore` awareness.
5. **`delete_path`**: Safely deletes a file or directory (permission guarded).
6. **`move_copy_path`**: Moves, renames, or copies files and directories.
7. **`grep`**: Fast ripgrep regex searching across workspace with file inclusion/exclusion glob filters.
8. **`find_path`**: Fuzzy file path finder across the workspace.
9. **`terminal`**: Executes commands in an interactive PTY with timeout and head/tail line limits.
10. **`diagnostics`**: Directly inspects compiler and linter errors/warnings from the LSP.
11. **`go_to_definition`**: Queries LSP for symbol definitions.
12. **`find_references`**: Queries LSP for symbol usages across project.
13. **`ask_user`**: Queries user with questions or multiple-choice prompts.
14. **`load_skill`**: Dynamically loads full markdown instructions of an agent skill.
15. **`spawn_subagent`**: Creates an isolated child agent session to resolve a sub-problem.
16. **`mcp_bridge`**: Dynamic proxy forwarding tool calls to connected external MCP servers.

---

## 4.6 Module: `context_management/`

### `large_output_spiller.rs`
* **Problem**: A tool call (e.g. `cat huge_file.json` or `cargo build` outputting 5MB) overflows context limits and causes API errors.
* **Solution**:
  * Threshold: `DEFAULT_LARGE_OUTPUT_THRESHOLD = 150_000` characters (~37,500 tokens).
  * If output > threshold:
    1. Writes full payload to `.zenthree/tmp/tool_spill_<uuid>.txt`.
    2. Returns compact message:
       > *"Output exceeded 150KB (returned 421,090 chars). Output was saved to `.zenthree/tmp/tool_spill_<uuid>.txt`. Use `read_file` with line bounds or `grep` to inspect specific sections."*

### `compaction.rs` & `tool_pair_pruner.rs`
* **Compaction Trigger**: When total message tokens exceed 75% of model context window.
* **Process**:
  1. Older turns (excluding system prompt and active files) are compressed into a semantic summary.
  2. Prunes raw verbose tool results from earlier turns once the model has generated its response.

---

## 4.7 Module: `diff/`

### `matrix_diff.rs`
* Computes dynamic programming matrix diff over rope text buffers with column swapping.
* Memory complexity: $O(N)$ where $N$ is line count.
* Emits added (`+`) and removed (`-`) hunk events on streaming token chunks, enabling 120 FPS live highlighting in the editor buffer.

### `checkpoint.rs`
* **Multi-Buffer Turn Checkpoints**:
  * Before applying changes during a turn, captures snapshots of all affected buffers.
  * Allows user to execute `Revert Turn` or `Undo Checkpoint` with a single click, cleanly restoring all modified files.

---

## 4.8 Module: `orchestrator/`

### Finite State Machine (FSM)
The agent executes an explicit deterministic turn loop:

```
                  ┌──────────────────────┐
                  │    User Prompt       │
                  └──────────┬───────────┘
                             │
                             ▼
┌────────────────────────────────────────────────────────┐
│               ZENTHREE STATE MACHINE                   │
│                                                        │
│  [1. Prepare Context]  Inject active files, rules,     │
│                        and skill catalog.              │
│                                                        │
│  [2. Stream LLM]       Stream tokens, thinking blocks  │
│                        and tool calls. Check steering. │
│                                                        │
│  [3. Tool Approval]    Evaluate permissions.           │
│                        Prompt UI if Confirm required.  │
│                                                        │
│  [4. Execute Tools]    Parallel/Sequential execution   │
│                        inside Workspace Jail.          │
│                                                        │
│  [5. Resilience]       Spill outputs > 150KB to disk.  │
│                        Scrub secrets from outputs.     │
│                                                        │
│  [6. Compaction]       Check token budget. Compact if  │
│                        exceeding threshold.            │
│                                                        │
│  [7. Terminal Check]   If more tool calls, repeat [2]. │
│                        Else, commit checkpoint & done. │
└────────────────────────────────────────────────────────┘
```

### Mid-Flight Steerability (`steer.rs`)
* Allows the user to send a follow-up message while the agent is executing.
* Cancels pending unexecuted tool calls and adjusts the next LLM prompt with the user's updated guidance.

---

## 4.9 Module: `protocols/`

### Model Context Protocol (MCP 2.0)
* Implements client support for external MCP servers.
* Transports:
  * `transport_stdio.rs`: Spawns external processes (e.g. `npx -y @modelcontextprotocol/server-postgres`) and speaks JSON-RPC 2.0 over stdin/stdout.
  * `transport_sse.rs`: Connects to remote MCP servers over HTTP Server-Sent Events.
* Discovered tools are dynamically registered in `tools/mcp_bridge.rs`.

### Agent Client Protocol (ACP)
* Provides standardized JSON-RPC interface for headless testing, CI evaluation, and editor harnesses.

---

## 4.10 Module: `providers/`

Unified streaming model drivers with automatic retry, backoff, and error categorization:
* **Anthropic**: Claude 3.7 Sonnet, Claude 3.5 Sonnet, Claude Thinking blocks.
* **OpenAI**: GPT-4o, o1, o3-mini.
* **Google Gemini**: Gemini 2.5 Flash, Gemini 2.5 Pro.
* **Groq**: Llama 3.3 70B, DeepSeek R1.
* **Local**: Ollama and Llama.cpp.

---

## 4.11 Module: `types/`

### Streaming UI Events (`events.rs`)
Emits strongly typed events to the UI layer:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    TurnStarted { turn_id: String },
    ThinkingChunk { delta: String },
    TokenChunk { delta: String },
    ToolCallStarted { call_id: String, tool_name: String, args: serde_json::Value },
    PermissionPrompt { call_id: String, tool_name: String, reason: String },
    ToolCallOutput { call_id: String, output: String, duration_ms: u64 },
    DiffHunk { path: PathBuf, line: usize, old: String, new: String },
    TurnCompleted { turn_id: String, token_usage: TokenUsage },
    TurnFailed { turn_id: String, error: String },
}
```

---

# 5. Cross-Crate Integration Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           CRATES/ZENTHREE (UI)                          │
│                                                                         │
│  - AI Panel View (Markdown, Thinking accordions, Tool invocation cards) │
│  - Interactive Permission Modals (Allow Once, Always Allow, Deny)       │
│  - Multi-File Checkpoint Review Bar (Accept All, Reject All)            │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │ Events & State
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                             CRATES/AI                                   │
│                                                                         │
│  [environment]  [permissions]  [skills]  [rules]  [tools]               │
│  [context_mgmt] [diff]         [orchestrator]    [providers]            │
└────────────────────────────────────┬────────────────────────────────────┘
                                     │ Hunk Updates & Snapshots
                                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           CRATES/EDITOR                                 │
│                                                                         │
│  - Text Buffer Rope & Undo/Redo Timeline                                │
│  - 120 FPS Live Streaming Diff Decorations (+ / - line highlights)      │
│  - Multi-Buffer Snapshot Restore on Checkpoint Rollback                 │
└─────────────────────────────────────────────────────────────────────────┘
```

---

# 6. Phased Implementation Roadmap & Verification Test Plans

### Phase 1: Agent Environment & Sandboxing (`environment/`)
* **Deliverables**: `workspace_jail.rs`, `protected_paths.rs`, `secret_scrubber.rs`, `env_sanitizer.rs`, `process_runner.rs`, `host_detection.rs`.
* **Automated Tests**:
  * Canonical path jailing rejecting `../../etc/passwd` and symlink escapes.
  * Protected path guard rejecting write attempts to `.git/hooks/pre-commit`.
  * Secret scrubber replacing API keys and SSH headers with `[REDACTED_SECRET]`.
  * Process runner enforcing timeout cancellation and `head_lines`/`tail_lines` clipping.

### Phase 2: Security & Permissions (`permissions/`)
* **Deliverables**: `hardcoded_denials.rs`, `command_parser.rs`, `rules.rs`, `decision.rs`.
* **Automated Tests**:
  * Unconditional rejection of `rm -rf /`, `rm -rf ~`, `rm -rf .`, and `rm -rf /tmp/../../`.
  * Command chaining parser detecting hidden subcommands (`echo ok && rm -rf /`).
  * Tri-state decision evaluation (Allow, Confirm, Deny).

### Phase 3: Skills & Rules System (`skills/` & `rules/`)
* **Deliverables**: `agentskills.io` parser, multi-tier discovery, catalog budget management, multi-standard rule scanner.
* **Automated Tests**:
  * Parsing standard `SKILL.md` frontmatter and markdown body.
  * Discovery precedence: Workspace shadows Global shadows Builtin.
  * Rule ingestion parsing `.cursorrules` and `AGENTS.md`.

### Phase 4: Unified Tool Suite (`tools/`)
* **Deliverables**: Complete 16-tool suite (`fs`, `search`, `terminal`, `lsp`, `agent`).
* **Automated Tests**:
  * Safe chunked reads, atomic writes, in-buffer fuzzy edit replacements.
  * Ripgrep search filtering by glob patterns.
  * Real-time LSP diagnostics retrieval.

### Phase 5: Context Resilience & Large Output Spiller (`context_management/`)
* **Deliverables**: `large_output_spiller.rs`, `compaction.rs`, `tool_pair_pruner.rs`.
* **Automated Tests**:
  * Tool payloads > 150KB automatically spilled to `.zenthree/tmp/` with valid file pointers.
  * Sliding window compaction generating structured conversation summaries.

### Phase 6: Streaming Diff & Multi-Buffer Checkpoints (`diff/`)
* **Deliverables**: Dynamic programming matrix rope diff, multi-file turn checkpoint timelines.
* **Automated Tests**:
  * O(N) memory diff correctly calculating added and removed hunks.
  * Checkpoint timeline rolling back multi-file edits simultaneously.

### Phase 7: FSM Orchestrator & Providers (`orchestrator/` & `providers/`)
* **Deliverables**: Complete deterministic state machine turn loop, mid-flight steerability, multi-model streaming providers.
* **Automated Tests**:
  * End-to-end simulated turn with tool execution, permission gate, and token streaming.
  * Mid-flight user steering cancelling pending steps.

### Phase 8: Protocols (MCP & ACP) (`protocols/`)
* **Deliverables**: Native MCP 2.0 client (Stdio & SSE), ACP adapter.
* **Automated Tests**:
  * Spawning sample MCP stdio server and calling external tools.
