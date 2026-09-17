# Z.E.N.E. Agent - Comprehensive System Specification & Architecture

This document details the complete production architecture, internal subsystems, loop engineering mechanics, and operational workflows of **Z.E.N.E.** (**Z**enthree **E**ngineering & **N**eural **E**xecutor), the autonomous software engineering agent built natively into the Zenthree code editor.

---

## 1. System Overview & Philosophy

**Z.E.N.E.** is an autonomous, in-editor software engineering agent designed on modern **Loop Engineering** principles. Rather than acting as a simple chatbot that expects human copy-pasting, Z.E.N.E. functions as a proactive, autonomous systems engineer directly inside the editor runtime:

- **Loop Engineering over Prompt Engineering**: Instead of relying on a single, brittle prompt, Z.E.N.E. is governed by an automated control harness that iteratively plans, reasons, acts via tools, observes real execution outputs, self-heals compiler diagnostics, and terminates cleanly when verification succeeds.
- **Dual-Loop Architecture**: Strictly separates the **Outer Loop** (strategic multi-agent task orchestration, mission queues, dependency ordering, and non-halting failure handling) from the **Inner Loop** (tactical, fast ReAct tool cycles, surgical patching, and compiler feedback).
- **Specialized Multi-Agent Coordination**: Coordinates four dedicated sub-agents (**Planner**, **Explorer**, **Builder**, **Tester**) under the lead orchestrator (**ZENE**), enforcing the *Principle of Least Privilege* for tool access to eliminate hallucination and context bloat.
- **Surgical Diff Engine & 3-Tier Matcher**: Replaces exact slices of code using search-and-replace blocks rather than rewriting whole files, with zero-allocation exact matching, CRLF normalization, and indentation tolerance.
- **Biological Deliberation Engine**: Connects to an in-silico spiking neural connectome (`crates/connectome`) to calculate evidence accumulation vectors, reaction latency, and continuous deliberation confidence metrics.
- **Verification Gate & Self-Healing**: Enforces non-destructive compiler checks (`cargo check`, `npx tsc`, `mypy`, `go vet`) and automated test suites, feeding isolated compiler diagnostics straight back into a bounded repair loop.
- **Pluggable & Resilient Provider Chain**: Uses trait-based model abstractions with automatic error classification (`Transient` vs `Fatal`), exponential backoff, and seamless failover across Google Gemini, Groq Cloud, Claude, and local endpoints.

---

## 2. Modular Crate Structure (`crates/ai`)

All AI capabilities, sub-agents, tool registries, ACP/Goose protocol adapters, providers, and session persistence are decoupled from the UI into an independent, modular crate: `crates/ai` (workspace package `ai`).

```
crates/ai/
├── Cargo.toml
├── src/
│   ├── lib.rs                       # Crate root, high-level Zene facade & public re-exports
│   │
│   ├── acp/                         # 🔌 AGENT CLIENT PROTOCOL (ACP) SUBSYSTEM
│   │   ├── mod.rs                   # ACP client adapter, process supervisor & connection lifecycle
│   │   ├── protocol.rs              # Typed JSON-RPC ACP schemas (SessionUpdate, Notifications, Prompts)
│   │   ├── transport.rs             # Stdio transport, async JSON-RPC framing & stream parsing
│   │   └── adapter.rs               # Event adapter converting ACP notifications into AgentEvents
│   │
│   ├── goose/                       # 🪿 GOOSE CLI INTEGRATION SUBSYSTEM
│   │   ├── mod.rs                   # Goose backend manager & runner
│   │   ├── config.rs                # Goose configuration parser, extensions & profiles
│   │   └── supervisor.rs            # Child process lifecycle, health check & IPC supervisor
│   │
│   ├── orchestrator/                # 🔄 DUAL-LOOP CONTROL SYSTEM & AGENT BACKENDS
│   │   ├── mod.rs                   # Orchestrator orchestration facade & backend selector
│   │   ├── backend.rs               # AgentBackend enum: Native vs GooseAcp vs Custom
│   │   └── loop_engine.rs           # Native autonomous ReAct loop driver with surgical diff capture
│   │
│   ├── tools/                       # 🛠️ TYPED TOOL REGISTRY & SURGICAL DIFF ENGINE
│   │   ├── mod.rs                   # AgentToolCall enum, AgentToolResult & execute_agent_tool router
│   │   ├── patch.rs                 # SearchReplaceBlock, FileEditAction, TargetFileModification
│   │   ├── command.rs               # run_command (sandboxed shell runner with background support)
│   │   ├── environment.rs           # detect_environment (heuristics + manifest analysis)
│   │   ├── read.rs                  # read_file (surgical line-bounded reading)
│   │   ├── search.rs                # search_code (fast ripgrep across project tree)
│   │   ├── list.rs                  # list_directory (bounded depth directory walk)
│   │   ├── git.rs                   # git_status, git_diff (repository awareness & rollback)
│   │   └── parser.rs                # parse_agent_tool_calls, parse_search_replace_blocks
│   │
│   ├── providers/                   # 🧠 MODEL PROVIDERS & RESILIENT FALLBACK CHAIN
│   │   ├── mod.rs                   # ModelProvider trait (complete, stream, count_tokens)
│   │   ├── chain.rs                 # ProviderChain: multi-provider failover & retry engine
│   │   ├── errors.rs                # ProviderError (Transient vs Fatal), classify_provider_error
│   │   ├── gemini.rs                # Google Gemini client (Gemini 2.5/3.5/Flash, IPv4 DNS routing)
│   │   ├── groq.rs                  # Groq Cloud client (Llama 3, DeepSeek R1, GPT-OSS-120b)
│   │   ├── claude.rs                # Anthropic Claude client (Claude 3.5 Sonnet)
│   │   ├── openai.rs                # OpenAI-compatible client (Local LLMs, Ollama, vLLM)
│   │   └── keys.rs                  # API key loaders from process environment and local .env
│   │
│   ├── context/                     # 📚 3-TIER CONTEXT ENGINE & TOKEN BUDGETING
│   │   ├── mod.rs                   # ContextEngine: unifies Working Memory, Session, and AST
│   │   ├── prompts.rs               # Base system prompts, ZENE identity, autonomous coding rules
│   │   ├── truncation.rs            # Output truncation guards (caps tool outputs to prevent context blowup)
│   │   └── self_healing.rs          # filter_compiler_errors & build_self_healing_prompt
│   │
│   └── types/                       # 📦 SHARED CORE TYPES & UI STREAMING EVENTS
│       ├── mod.rs
│       ├── message.rs               # ChatMessage, Role, EditedFileSummary (original/proposed code)
│       ├── env.rs                   # ProjectLanguage, ProjectEnvironment, WorkspaceDetectionContext
│       └── events.rs                # AgentEvent stream for 120 FPS non-blocking UI rendering
```

---

## 3. Dual-Loop Engineering Architecture

The heart of Z.E.N.E. is the separation of **Strategic Orchestration (Outer Loop)** from **Mechanical Execution (Inner Loop)**:

```
                      ┌─────────────────────────────────────────────────────────┐
                      │              OUTER LOOP (Strategic Orchestration)       │
                      │  User Goal ──► Planner ──► Task Queue ──► Verify ──► End│
                      └────────────────────────────┬────────────────────────────┘
                                                   │ assigns next runnable task
                   ┌───────────────────────────────┴───────────────────────────────┐
                   │                                                               │
                   ▼                                                               ▼
       ┌───────────────────────────────┐                               ┌───────────────────────────────┐
       │   INNER LOOP: Explorer        │                               │   INNER LOOP: Builder/Tester  │
       │   Reason ──► read_file        │                               │   Reason ──► patch_file       │
       │     ▲           │             │                               │     ▲             │           │
       │     │           ▼             │                               │     │             ▼           │
       │   Observe ◄── Tool Output     │                               │   Self-Heal ◄── cargo check   │
       └───────────────────────────────┘                               └───────────────────────────────┘
```

### The Outer Loop (Strategic Governance & Non-Halting State Machine)
1. **Goal Ingestion**: The user prompt is ingested, and intent is classified (`Refactor`, `NewFeature`, `Debug`, `Explain`).
2. **Deliberation**: Connectome calculates evidence vectors and attaches deliberation confidence to the mission.
3. **Plan Formulation**: 📋 **Planner** maps out the dependency graph and populates `AgentMission` tasks.
4. **Task Dispatch**: Pulls the first `Pending` task and assigns it to the designated sub-agent.
5. **Non-Halting Resilience (`mark_task_failed_and_continue`)**:
   - If a task encounters a permanent failure (e.g. external API or unsolvable semantic error), the orchestrator logs the diagnostic, marks that specific task `Failed(reason)`, and **advances to the next runnable task**.
   - The mission is never prematurely halted by a single isolated failure.

### The Inner Loop (Surgical ReAct & Verification)
1. **Context Assembly**: Only the necessary context for the active task is assembled (avoiding full-repo dumps).
2. **Proposal**: The assigned sub-agent emits a typed tool call (`AgentToolCall`).
3. **Execution**: The Rust sandbox executes the tool against the workspace filesystem or compiler.
4. **Observation**: Real output (or filtered compiler diagnostic) is returned as `AgentToolResult`.
5. **Verification & Self-Correction**: 🛡️ **Tester** validates the change with `cargo check` / `tsc`. If diagnostics occur, the error is fed straight back into the prompt for up to 2 automated self-healing turns.

---

## 4. Multi-Agent System: Z.E.N.E. & The 4 Sub-Agents

```
                     ┌─────────────────────────────────────────┐
                     │                  ZENE                   │
                     │  Zenthree Engineering & Neural Executor │
                     │  (User Dialogue · Master Orchestrator)  │
                     └────────────────────┬────────────────────┘
                                          │ delegates
         ┌──────────────────┬─────────────┴────────────┬──────────────────┐
         │                  │                          │                  │
         ▼                  ▼                          ▼                  ▼
┌─────────────────┐┌─────────────────┐        ┌─────────────────┐┌─────────────────┐
│   📋 PLANNER    ││   🔍 EXPLORER   │        │   ⚡ BUILDER    ││   🛡️ TESTER     │
│ Architecture    ││ Reconnaissance  │        │ Code Synthesis  ││ Verification    │
│ Task Breakdown  ││ Files & Symbols │        │ Surgical Diffs  ││ Self-Healing    │
└─────────────────┘└─────────────────┘        └─────────────────┘└─────────────────┘
```

### 1. 👑 Lead Agent: ZENE
- **Identity**: Master autonomous engineering coordinator.
- **Responsibility**: Direct dialogue with user, intent classification, mission initialization, sub-agent delegation, and presentation of final changes and walkthroughs.
- **Allowed Tools**: `delegate_to_subagent`, `ask_user`, `finalize_mission`.

### 2. 📋 Sub-Agent: Planner
- **Badge**: `📋 Planner`
- **Responsibility**: Decomposes high-level requirements into clear, ordered tasks with dependency tracking. Evaluates project conventions and build files before implementation begins.
- **Allowed Tools**: `list_directory`, `detect_environment`, `read_file` (manifests/configs only), `create_task_plan`.
- **Safety Restriction**: **Read-only**. Cannot modify source code or run arbitrary shell commands.

### 3. 🔍 Sub-Agent: Explorer
- **Badge**: `🔍 Explorer`
- **Responsibility**: Rapid workspace reconnaissance, AST symbol discovery, tracing type definitions and function signatures, and gathering targeted line context.
- **Allowed Tools**: `search_code`, `read_file`, `find_symbol`, `list_directory`, `git_status`.
- **Safety Restriction**: **Read-only**. Cannot modify source code or run arbitrary shell commands.

### 4. ⚡ Sub-Agent: Builder
- **Badge**: `⚡ Builder`
- **Responsibility**: Formulates surgical `SEARCH/REPLACE` blocks and creates new files. Ensures edits preserve existing comments, formatting, and surrounding architecture.
- **Allowed Tools**: `edit_file`, `write_file`, `multi_file_patch`, `read_file` (surrounding line checks).
- **Safety Restriction**: Confined strictly to project workspace paths. Cannot run shell commands or delete directories.

### 5. 🛡️ Sub-Agent: Tester
- **Badge**: `🛡️ Tester`
- **Responsibility**: Verification gatekeeper. Executes non-destructive compiler checks, runs targeted unit tests, extracts clean compiler diagnostics, and drives the self-healing repair loop.
- **Allowed Tools**: `run_compiler_check`, `run_tests`, `self_heal_patch`, `git_diff`, `run_command` (sandboxed build scripts only).
- **Safety Restriction**: Destructive commands (`rm -rf`, `sudo`, `mkfs`) are hard-blocked by the sandbox security guard.

---

## 5. Comprehensive Tool Registry & Permission Matrix

| Tool | Signature | 📋 Plan | 🔍 Exp | ⚡ Build | 🛡️ Test | 👑 Zene |
|---|---|:---:|:---:|:---:|:---:|:---:|
| `list_directory` | `{"path": "<dir>", "max_depth": 2}` | ✅ | ✅ | ❌ | ❌ | ❌ |
| `detect_environment` | `{"subpath": "<dir>"}` | ✅ | ❌ | ❌ | ❌ | ❌ |
| `search_code` | `{"query": "<str>", "pattern": "*.rs"}` | ❌ | ✅ | ❌ | ❌ | ❌ |
| `find_symbol` | `{"name": "<Symbol>", "kind": "fn"}` | ❌ | ✅ | ❌ | ❌ | ❌ |
| `read_file` | `{"path": "<file>", "start_line": 1, "end_line": 50}` | ✅ | ✅ | ✅ | ❌ | ❌ |
| `git_status` / `git_diff` | `{"staged_only": false}` | ❌ | ✅ | ❌ | ✅ | ❌ |
| `edit_file` | `{"path": "<file>", "search": "...", "replace": "..."}` | ❌ | ❌ | ✅ | ❌ | ❌ |
| `write_file` | `{"path": "<file>", "content": "..."}` | ❌ | ❌ | ✅ | ❌ | ❌ |
| `multi_file_patch` | `{"patches": [{"path": "...", "blocks": [...]}]}` | ❌ | ❌ | ✅ | ❌ | ❌ |
| `run_command` | `{"cmd": "<command>", "timeout_secs": 30}` | ❌ | ❌ | ❌ | ✅ | ❌ |
| `run_compiler_check` | `{"env": "..."}` | ❌ | ❌ | ❌ | ✅ | ❌ |
| `run_tests` | `{"test_filter": "<optional_name>"}` | ❌ | ❌ | ❌ | ✅ | ❌ |
| `self_heal_patch` | `{"path": "<file>", "search": "...", "replace": "..."}` | ❌ | ❌ | ❌ | ✅ | ❌ |
| `delegate_to_subagent` | `{"role": "Builder", "task": "..."}` | ❌ | ❌ | ❌ | ❌ | ✅ |

---

## 6. Surgical Diff Engine & 3-Tier Matcher

Full-file rewrites suffer from code truncation and helper clobbering. Z.E.N.E. exclusively performs surgical modifications:

```markdown
### File: src/ui_layout/sidebar.rs
<<<<<<< SEARCH
fn render_tab(&self, tab: &Tab) -> Element {
    let title = tab.name.clone();
=======
fn render_tab(&self, tab: &Tab, is_active: bool) -> Element {
    let title = if is_active { format!("* {}", tab.name) } else { tab.name.clone() };
>>>>>>> REPLACE
```

### The 3-Tier Matcher Pipeline (`apply_search_replace`)
1. **Tier 1 (Exact Substring Match)**: Direct zero-allocation substring search. Executes in microseconds.
2. **Tier 2 (CRLF / LF Normalization)**: Strips Windows carriage returns (`\r\n` $\rightarrow$ `\n`) across both the target file and the search block, eliminating platform line-ending mismatches.
3. **Tier 3 (Line-Trimmed & Indentation-Tolerant Match)**:
   - Compares trimmed lines to locate code even when LLMs slightly alter leading indentation.
   - Computes indentation delta: $\Delta_{\text{indent}} = \text{indent}_{\text{actual}} - \text{indent}_{\text{search}}$.
   - Re-indents replacement lines dynamically to preserve perfect code formatting.

---

## 7. 3-Tier Context Engine & Token Budgeting

To scale to multi-million line codebases without context degradation:

| Tier | Component | Location | Role |
|---|---|---|---|
| **Tier 1: Working Memory** | Sliding message window + Tool observations | RAM (`ai_messages`) | Holds active conversational turns and surgical tool outputs. Older turns are rolling-compacted. |
| **Tier 2: Task State Store** | Machine-parseable missions & markdown artifacts | `.zenthree/sessions/<id>/` | Atomic disk store (`task.md`, `plan.md`, `walkthrough.md`, `messages.json`). |
| **Tier 3: Long-Term Index** | Tree-Sitter AST & Symbol Index | `crates/syntax` & `project_index.rs` | Symbol definitions, call graphs, manifest convention maps, and cached environment profiles. |

### Output Truncation Guard
- Tool outputs (e.g. large grep results, verbose cargo outputs) are capped at **40 lines / 4,000 characters**.
- Excess lines are summarized: `"... [412 additional lines truncated - refine query with file_pattern] ..."`.
- Prevents context window exhaustion and prompt blowup.

---

## 8. Resilient Model Provider Chain (`providers/`)

```rust
pub trait ModelProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn complete(&self, system: &str, user: &str, history: &[ChatMessage]) -> Result<String, ProviderError>;
    fn stream(&self, system: &str, user: &str, history: &[ChatMessage], tx: &Sender<AgentEvent>) -> Result<(), ProviderError>;
}
```

### Automatic Error Classification & Backoff
- **`ProviderError::Transient`**:
  - Detection: HTTP 429 (Rate Limit), HTTP 500/503 (Server Busy), timeouts.
  - Strategy: Exponential backoff ($200\text{ms} \rightarrow 800\text{ms} \rightarrow 2000\text{ms}$) up to 3 attempts.
- **`ProviderError::Fatal`**:
  - Detection: HTTP 401 (Invalid Key), HTTP 404 (Model Deprecated), Quota Exhausted.
  - Strategy: Immediate, seamless failover to next provider in the chain.

### Provider Failover Ladder:
1. **Google Gemini (Primary)**: `gemini-3.5-flash` $\rightarrow$ `gemini-3.1-flash-lite` $\rightarrow$ `gemini-2.5-pro` (routed via IPv4 DNS to bypass IPv6 socket drops).
2. **Groq Cloud (High-Throughput Secondary)**: `openai/gpt-oss-120b` $\rightarrow$ `qwen/qwen3.8-27b` $\rightarrow$ `llama-3.3-70b-versatile`.
3. **Anthropic Claude (Complex Architectural Fallback)**: `claude-3-5-sonnet-latest`.
4. **OpenAI / Local LLM**: Generic endpoint (`http://localhost:11434/v1` for Ollama / vLLM).

---

## 9. Biological Deliberation Bridge (`crates/connectome`)

Z.E.N.E. integrates with the spiking neural network engine in `crates/connectome`:
- **Evidence Accumulation**: Pre-computes dual evidence vectors ($ev_a, ev_b$) based on intent analysis.
- **Neural Simulation**: Runs discrete time-step in-silico simulation across simulated cortical columns.
- **Metrics Injected**: Deliberation reaction time ($dt$) and confidence score ($0.0 \dots 1.0$) are injected into ZENE's prompt headers and displayed as real-time thought badges in the UI:
  `"Deliberation Confidence: 94.8% (dt: 18ms) — Plan Approved"`.

---

## 10. Runtime Reliability, Cancellation & 120 FPS UI Pipeline

### 1. Instant User Cancellation (`Arc<AtomicBool>`)
- The user "Stop" button or `Escape` key sets an atomic cancellation flag.
- The inner tool loop and network reader check this flag at every tick, terminating tasks cleanly in $< 50\text{ms}$ without leaving orphaned background processes.

### 2. Real-Time Streaming UI Event Pipeline (`AgentEvent`)
- The agent loop executes entirely on a background thread pool, communicating with the UI via high-speed crossbeam channels:
  ```rust
  pub enum AgentEvent {
      SubAgentActivated { role: SubAgentRole, name: String },
      Thinking { delta: String },
      StreamChunk { delta: String },
      ToolStarted { tool: String, summary: String },
      ToolFinished { tool: String, ok: bool, duration_ms: u64 },
      PendingModifications { summaries: Vec<EditedFileSummary> },
      TaskUpdated { task_id: String, status: TaskStatus },
      Completed { summary: String },
      Error { error: String },
  }
  ```
- Ensures the Zenthree editor UI remains fluid at **120 FPS** while tools execute.

### 3. Visual Diff Staging, Legible Buffer Highlighting & Drawer Rollback Engine
- **Editor Buffer Inline Diffs**:
  - Live diff lines are loaded directly into the active editor buffer via `compute_line_diff(original, proposed)`:
    - **Added Lines (`+`)**: Highlighted with a green gutter bar, green background tint, and `+` indicator.
    - **Removed Lines (`-`)**: Highlighted with a red gutter bar, red background tint, and `-` indicator.
    - **High Legibility**: Strikethrough cut lines across deleted text are intentionally avoided so removed code remains completely legible.
- **In-Chat Edited File Cards**:
  - Displays clean file name with `+added` and `-removed` line counts.
  - Features an individual **Reject** (`✕`) button to immediately revert a single file's modifications.
- **Floating Input Drawers (`AiInputDrawer`) with Deep Drop Shadows**:
  - Positioned directly above the prompt inbox with elevated drop shadows (`.shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, -4.0, 14.0)`).
  - **Modified Files List**: Shows file name, relative project path, +/- line count badges, per-file **Reject** (`✕`) button, and a bottom action bar with **Reject all** and **Accept all**.
  - **Running Terminals List**: Real-time display of running background AI commands with cancel (`✕`) buttons.
  - **Artifacts List**: Quick clickable navigation to AI generated artifacts (`plan.md`, `walkthrough.md`).

---

## 11. Dual Backend Execution Engine: Native & ACP/Goose

Z.E.N.E. supports swappable execution backends via `AgentBackend`:

```rust
pub enum AgentBackend {
    Native,
    GooseAcp,
}
```

### 1. Native Execution Loop (`crates/ai/src/orchestrator/loop_engine.rs`)
- In-process ReAct agent loop powered directly by configured model providers (Gemini, Groq, Claude, OpenAI).
- Executes typed tool calls (`edit_file`, `read_file`, `run_command`, `search_code`, `list_directory`, `detect_environment`, `git_status`, `git_diff`).
- Extracts search-and-replace blocks and code file updates, captures `original_code` and `proposed_code`, and dispatches `AgentEvent::PendingModifications`.

### 2. Agent Client Protocol (ACP) & Goose CLI (`crates/ai/src/acp/`, `crates/ai/src/goose/`)
- Integrates external autonomous agents conforming to the Agent Client Protocol (ACP) standard over JSON-RPC stdio.
- Bridges Goose CLI supervisor processes with real-time session streaming, converting notifications into native `AgentEvent`s (`ToolStarted`, `ToolFinished`, `PendingModifications`, `TaskUpdated`, `Completed`).

---

---

## 13. Granular Cognitive Loop & Visual Deliberation ("See → Think → Edit → Reflect")

*(For full specification, see [cognitive_loop_architecture.md](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/docs/agent/cognitive_loop_architecture.md))*

Rather than blind full-file overwrites, Z.E.N.E. operates on a disciplined cognitive cycle for multi-location edits:
1. **See Code Slice**: The agent issues bounded `read_file` calls (e.g. `#L1058-1085`) before generating any modification.
2. **Deep Deliberation**: The internal chain of reasoning is captured in `<thought>` blocks and rendered in the UI as collapsible `Thought for Xs > / v` elements.
3. **Surgical Multi-Location Editing**: Edits are applied location-by-location using search/replace patches without altering untouched sections.
4. **Hierarchical UI Transparency**:
   - `Worked for Xs`: Top-level completion timer.
   - `Explored N files, N searches`: Pre-flight reconnaissance accordion.
   - `Analyzed [icon] file #Lstart-end`: Direct clickable link to active lines in editor.
   - `Thought for Xs >`: Expandable internal monologue disclosure.
   - `Edited [icon] file +added -removed [✕]`: Surgical patch summary with instant rollback.

---

## 14. File Map & Code Locations

| Subsystem | File Path | Responsibilities |
|---|---|---|
| **Crate Root** | `crates/ai/src/lib.rs` | Public facade, crate exports, and convenience constructors. |
| **ACP Protocol** | `crates/ai/src/acp/protocol.rs` | Typed JSON-RPC 2.0 ACP notification schemas and request models. |
| **ACP Transport** | `crates/ai/src/acp/transport.rs` | Stdio transport, async JSON-RPC framing, and line parsing. |
| **ACP Adapter** | `crates/ai/src/acp/adapter.rs` | Bridges ACP session updates into internal `AgentEvent` streaming. |
| **Goose Supervisor**| `crates/ai/src/goose/supervisor.rs` | Goose CLI child process spawning, health monitoring, and IPC. |
| **Agent Backend** | `crates/ai/src/orchestrator/backend.rs` | Swappable backend dispatcher (`Native` vs `GooseAcp`). |
| **Loop Engine** | `crates/ai/src/orchestrator/loop_engine.rs` | Autonomous ReAct inner loop with observation feedback & patch capture. |
| **Tool Registry** | `crates/ai/src/tools/mod.rs` | Typed `AgentToolCall` execution and tool routing. |
| **Surgical Patch** | `crates/ai/src/tools/patch.rs` | `SearchReplaceBlock` formulation and non-destructive code splicing. |
| **Provider Chain** | `crates/ai/src/providers/chain.rs` | Multi-model fallback, retry backoff, and error classification. |
| **UI Streaming** | `crates/ai/src/types/events.rs` | Real-time `AgentEvent` definitions for 120 FPS UI feedback. |
| **Buffer Diffing** | `crates/zenthree/src/ui_layout/main_buffer.rs` | Inline `+`/`-` line diff rendering with legible text & gutter bars. |
| **AI Panel UI** | `crates/zenthree/src/ui_layout/ai_panel.rs` | Clean chat bubbles, shadowed drawers, and per-file reject actions. |
| **Cognitive Spec** | `docs/agent/cognitive_loop_architecture.md` | Deep specification for See → Think → Edit → Reflect loop and UI components. |

