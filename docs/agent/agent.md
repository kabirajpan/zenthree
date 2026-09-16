# Z.E.N.E. Agent - Comprehensive System Specification

This document details the architecture, internal subsystems, and operational workflows of **Z.E.N.E.** (**Z**enthra **E**ngineering & **N**eural **E**xecutor), the autonomous software engineering agent built directly into the editor.

---

## 1. System Overview

**Z.E.N.E.** is an autonomous, in-editor software engineering agent. Unlike typical chat assistants that generate code snippets for manual copy-pasting, Z.E.N.E. operates directly inside the editor runtime:
- **Autonomous Reasoning & Execution**: Analyzes workspace structure, inspects source files, deduces architectural conventions, and directly writes/patches files.
- **Multi-Agent Coordination (Z.E.N.E.)**: Coordinates four specialized sub-agents (**Explorer**, **Detector**, **Builder**, **Tester**) to modularize exploration, toolchain detection, code editing, and verification without context bloat.
- **Surgical Diff Engine**: Replaces specific slices of code using search-and-replace blocks rather than rewriting whole files.
- **Biological Deliberation**: Integrates a simulated biological spiking neural connectome to weigh trade-offs and calculate confidence scores.
- **Verification Gate**: Executes non-destructive compiler checks (`cargo check`, `npx tsc`, `mypy`, `go vet`) and initiates self-healing loops to repair diagnostics automatically.
- **Resilient Provider Chain**: Uses automated error classification, exponential backoff, and seamless fallback across Google Gemini and Groq Cloud models.


---

## 2. Layered Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              IDE FRONTEND                                   │
│  Main Buffer · Visual Line Diff Viewer · AI Panel · Session Drawer · Tabs   │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ user prompt / UI controls
┌──────────────────────────────────────▼──────────────────────────────────────┐
│                            ZENTHREE STATE                                   │
│  State Machine · Mission Queue · Pending Modifications · Buffer Synchronization │
└───┬───────────────────┬───────────────────┬───────────────────┬─────────────┘
    │                   │                   │                   │
┌───▼───────────┐ ┌─────▼─────────────┐ ┌───▼─────────────┐ ┌───▼─────────────┐
│CONTEXT ENGINE │ │   CONNECTOME      │ │  TOOL REGISTRY  │ │ PROVIDER CHAIN  │
│Tree-Sitter AST│ │Spiking Neural Sim │ │search_code,     │ │Gemini 3.5-Flash │
│Language Deduc.│ │Deliberation dt    │ │read_file, ls,   │ │Groq GPT-OSS-120b│
│Workspace Tree │ │Ablation Testing   │ │run_command, edit│ │Auto-Fallback    │
└───┬───────────┘ └─────┬─────────────┘ └───┬─────────────┘ └───┬─────────────┘
    │                   │                   │                   │
┌───▼───────────────────▼───────────────────▼───────────────────▼─────────────┐
│                           SANDBOX & FILESYSTEM                              │
│   Surgical 3-Tier Matcher · Disk IO · Visual Diffs · Compiler Checks        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Biological Connectome Deliberation Engine (`crates/connectome`)

Zenthree features a biologically-inspired spiking neural deliberation engine modeled on neuronal connectome dynamics.

### How it Works:
- **Evidence Accumulation**: When a user submits a prompt, the engine calculates evidence vectors ($ev_a, ev_b$) based on intent (e.g. refactoring, file creation, code explanation).
- **Deliberation Dynamics**: Runs in-silico simulation across neural layers over simulated discrete time steps (`reaction_time_dt`).
- **Confidence & Decision**: Outputs a continuous deliberation confidence score (0.0 to 1.0), reaction latency, and decision label (e.g. `ActionConfirmed`, `RefactorRecommended`).
- **In Silico Ablation Testing**: Supports simulated neural ablation modes (`TargetedHubs`, `Random`) with variable ablation percentages to research neural network degradation and agent resilience.
- **Prompt Grounding**: The connectome's deliberation metrics (`reaction_time_dt`, `confidence`) are injected into the agent's prompt headers and user-visible thought badges.

---

## 4. Context Management: 3-Tier Hierarchy

To handle large codebases without overflowing context windows or causing token degradation:

| Tier | Component | Location | Role |
|---|---|---|---|
| **Tier 1: Working Memory** | Chat History & Tool Observations | RAM (`ai_messages`) | Holds the active multi-turn conversation and real workspace tool observation results. |
| **Tier 2: Task & Session State** | Agent Mission & Session Artifacts | `.zenthree/sessions/<id>/` | Machine-parseable task checklist (`mission.json` / `mission.md`), architectural plans (`implementation_plan.md`), and summary logs (`walkthrough.md`). |
| **Tier 3: Long-Term Index** | Tree-Sitter AST & Workspace Symbols | `project_index.rs` | Project directory tree overview, framework convention detection (React, Qwik City, Next.js, Vite, Vue, Svelte, Rust, Python, Go), and top-matched symbol snippets. |

---

## 5. Surgical Diff Editing & 3-Tier Matcher

### The Problem
Full-file code generation by LLMs regularly drops helper functions, clobbers comments, or truncates large files (`// ... rest of code`).

### The Solution: Surgical SEARCH / REPLACE Blocks
Z.E.N.E. edits existing files by outputting precise diff slices:

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

### The 3-Tier Matcher (`apply_search_replace`)
1. **Tier 1 (Exact Substring Match)**: Zero-allocation direct substring match. Fast path.
2. **Tier 2 (CRLF / LF Normalization)**: Strips carriage return variations (`\r\n` vs `\n`) to prevent Windows/Unix line-ending mismatches.
3. **Tier 3 (Line-Trimmed & Indentation-Tolerant Match)**:
   - Compares lines with trimmed whitespace.
   - Computes indentation delta between existing code and LLM search block.
   - Adjusts indentation of replacement lines so inserted code matches the surrounding code's indentation.

### Visual Diff Integration in Editor
- Proposed code is compared against the active buffer/disk using `compute_line_diff`.
- Injects colored visual diff lines (`+green added`, `-red removed`) into the active editor tab.
- Activates the `AiInputDrawer::PendingModifications` drawer with **Accept All** and **Reject All** buttons.

---

## 6. Typed Tool Registry & Native Sandbox

The agent has access to a typed tool suite for workspace exploration and inspection before code generation.

### Tool Catalog (`AgentToolCall`)

1. **`search_code`**
   - **Signature**: `{"tool": "search_code", "query": "<pattern>"}`
   - **Behavior**: Recursively walks project source files. Safely skips noisy/binary dirs (`.git`, `node_modules`, `target`, `dist`, `build`, `.gemini`). Returns up to 30 line-numbered matches (`path/to/file.rs:42: let token = ...`).
2. **`read_file`**
   - **Signature**: `{"tool": "read_file", "path": "<file>", "start_line": 10, "end_line": 40}`
   - **Behavior**: Reads file with 1-indexed line numbers. Canonicalizes paths to prevent directory traversal outside the workspace.
3. **`list_directory`**
   - **Signature**: `{"tool": "list_directory", "path": "<optional_subdir>"}`
   - **Behavior**: Lists directories (with `/` suffix) and files (with human-readable byte sizes `KB`/`MB`).
4. **`run_command`**
   - **Signature**: `{"tool": "run_command", "cmd": "<command>"}`
   - **Behavior**: Executes shell commands within the workspace directory. Includes security guards blocking destructive commands (`rm -rf`, `sudo`, `mkfs`, fork bombs).
5. **`edit_file`**
   - **Signature**: `{"tool": "edit_file", "path": "<file>", "search": "<old_str>", "replace": "<new_str>"}`
   - **Behavior**: Programmatically applies surgical patches to disk.
6. **`detect_environment`**
   - **Signature**: `{"tool": "detect_environment", "subpath": "<optional_subdir>"}`
   - **Behavior**: Inspects project configuration files (`Cargo.toml`, `package.json`, `tsconfig.json`, `pyproject.toml`, `go.mod`, `CMakeLists.txt`, `Makefile`) to identify language, build tool, compiler check command, and test runner.


### Multi-Turn Tool Observation Flow
```
Model emits Tool Call ──► Rust Sandbox executes Tool ──► Real Output formatted
                                                                │
                                                                ▼
Model receives Observation Context ◄────────────────────────────┘
```

---

## 7. Resilient Provider Chain & Auto-Fallback

Zenthree incorporates a multi-provider fallback engine (`ProviderChain`) in [`agent.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/agent.rs):

### Error Classification
- **`ProviderError::Transient`**:
  - Detection: HTTP 429, rate limits, 500/503 server errors, timeouts.
  - Strategy: Exponential backoff delay with automatic retry.
- **`ProviderError::Fatal`**:
  - Detection: HTTP 401 (Invalid Key), HTTP 404 (Model Gone / Deprecated), quota exhausted.
  - Strategy: Immediate seamless failover to secondary provider.

### Configured Models
- **Google Gemini**:
  - `gemini-3.5-flash` (Primary high-speed reasoning)
  - `gemini-3.1-flash-lite` (Ultra-low latency fallback)
  - `gemini-3.6-flash`, `gemini-flash-latest`, `gemma-4-31b-it`
- **Groq Cloud**:
  - `openai/gpt-oss-120b` (Primary high-throughput coding)
  - `openai/gpt-oss-20b` (Secondary fallback)
  - `qwen/qwen3.8-27b`

---

## 8. Non-Halting Orchestrator State Machine

In contrast to single-prompt agents that fail permanently upon the first error, Zenthree's `AgentMission` manages a resilient task state machine with sub-agent role assignments:

### Task State Structure (`AgentTask`)
```rust
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub target_file: Option<String>,
    pub diff_summary: Option<EditedFileSummary>,
    pub sub_agent: Option<SubAgentRole>,
}

pub enum TaskStatus {
    Pending,
    ReadingChecklist,
    InProgress,
    Completed,
    Failed(String),
}
```

### Orchestrator Lifecycle:
1. **`next_runnable_task_index()`**: Scans task list for the next `Pending` task.
2. **`complete_task_and_advance(idx)`**: Marks the current task as `Completed`, advances `active_task_index`, and sets status to `InProgress`.
3. **`mark_task_failed_and_continue(idx, error_msg)`**:
   - When compiler verification or tool execution encounters persistent failure, records the diagnostic error on that task.
   - **Crucially: Does not abort the mission.** Advances to the next runnable task so independent feature steps continue executing.

---

## 9. Verification Gate & Self-Healing Loop

1. **Automatic Environment & Compiler Detection (`DetectorAgent`)**:
   - Inspects workspace manifests and dynamically selects the correct toolchain and test runner:
     - **Rust**: `Cargo.toml` $\rightarrow$ `cargo check --message-format=short`, `cargo test`
     - **TypeScript / JavaScript**: `package.json`, `tsconfig.json`, lockfiles (`bun.lockb`, `pnpm-lock.yaml`, `yarn.lock`, `package-lock.json`, `deno.json`) $\rightarrow$ `npx tsc --noEmit` / `bun run check`, `npm test` / `bun test`
     - **Python**: `pyproject.toml`, `requirements.txt`, `setup.py`, `Pipfile` $\rightarrow$ `mypy .`, `pytest`
     - **Go**: `go.mod` $\rightarrow$ `go vet ./...`, `go test ./...`
     - **C / C++**: `CMakeLists.txt`, `Makefile` $\rightarrow$ `cmake --build .` / `make -q`, `ctest` / `make test`
2. **Error Filtering (`filter_compiler_errors`)**:
   - Filters raw compiler noise down to actionable error diagnostics (`error[E0425]`, file coordinates, compiler suggestions `help:`).
3. **Self-Healing Loop (`build_self_healing_prompt`)**:
   - Generates a targeted compiler repair prompt containing the isolated diagnostics.
   - Allows up to 2 automated healing attempts.
   - If resolved, marks the verification task passed; if persistent, records diagnostics and continues the mission.

---

## 10. Multi-Agent Architecture: Z.E.N.E. & Sub-Agents

Zenthree employs a cooperative multi-agent architecture where the lead agent coordinates specialized sub-agents to achieve end-to-end task completion without token bloat or context degradation.

```
                     ┌─────────────────────────────────────────┐
                     │                  ZENE                   │
                     │  Zenthree Engineering & Neural Executor │
                     │  (User Dialog · Planner · Orchestrator) │
                     └────────────────────┬────────────────────┘
                                          │
         ┌──────────────────┬─────────────┴────────────┬──────────────────┐
         │                  │                          │                  │
         ▼                  ▼                          ▼                  ▼
┌─────────────────┐┌─────────────────┐        ┌─────────────────┐┌─────────────────┐
│   🔍 EXPLORER   ││   🧠 DETECTOR   │        │   ⚡ BUILDER    ││   🛡️ TESTER     │
│ Reconnaissance  ││ Toolchain & Env │        │ Code Synthesis  ││ Verification    │
│ Files & Symbols ││ Compiler & Tests│        │ Surgical Diffs  ││ Self-Healing    │
└─────────────────┘└─────────────────┘        └─────────────────┘└─────────────────┘
```

### Lead Agent: ZENE
- **Full Form**: **Z**enthree **E**ngineering & **N**eural **E**xecutor (displayed simply as **ZENE** in the AI panel and editor)
- **Role**: Primary interface with the user. Decomposes high-level requests into structured missions, assigns tasks to sub-agents, aggregates their findings, and renders formatted previews and chat responses.

### Specialized Sub-Agents:

1. 🔍 **Explorer** (`SubAgentRole::Explorer`)
   - **Mission**: Scans project directories, traces symbol definitions, and gathers AST context without cluttering the primary prompt.
   - **Tools Used**: `search_code`, `read_file`, `list_directory`.

2. 🧠 **Detector** (`SubAgentRole::Detector`)
   - **Mission**: Analyzes project files and toolchains to determine the programming language, build system, non-destructive compiler check command, and test runner.
   - **Static Manifest Heuristics**:
     - **Rust**: Detects `Cargo.toml` $\rightarrow$ Compiler: `cargo check --message-format=short`, Tests: `cargo test`.
     - **TypeScript / JavaScript**: Detects `tsconfig.json`, `package.json`, lockfiles (`bun.lockb`, `pnpm-lock.yaml`, `yarn.lock`, `deno.json`) $\rightarrow$ Compiler: `npx tsc --noEmit` / `bun run check` / `deno check`, Tests: `npm test` / `bun test` / `deno test`.
     - **Python**: Detects `pyproject.toml`, `requirements.txt`, `setup.py`, `Pipfile` $\rightarrow$ Linter/Checker: `mypy .`, Tests: `pytest`.
     - **Go**: Detects `go.mod` $\rightarrow$ Compiler: `go vet ./...`, Tests: `go test ./...`.
     - **C / C++**: Detects `CMakeLists.txt`, `Makefile` $\rightarrow$ Compiler: `cmake --build .` / `make -q`, Tests: `ctest` / `make test`.
     - **Zig**: Detects `build.zig`, `build.zig.zon` $\rightarrow$ Compiler: `zig build`, Tests: `zig build test`.
     - **Java / Kotlin**: Detects `build.gradle`, `build.gradle.kts`, `pom.xml` $\rightarrow$ Compiler: `./gradlew check` / `mvn test-compile`, Tests: `./gradlew test` / `mvn test`.
     - **Swift**: Detects `Package.swift` $\rightarrow$ Compiler: `swift build`, Tests: `swift test`.
     - **Ruby**: Detects `Gemfile` $\rightarrow$ Tests: `bundle exec rspec`.
     - **PHP**: Detects `composer.json` $\rightarrow$ Tests: `composer test`.
     - **Elixir**: Detects `mix.exs` $\rightarrow$ Compiler: `mix compile`, Tests: `mix test`.
     - **Dart / Flutter**: Detects `pubspec.yaml` $\rightarrow$ Compiler: `dart analyze` / `flutter analyze`, Tests: `dart test` / `flutter test`.
   - **Autonomous LLM Fallback (`detect_with_llm`)**:
     - When static heuristics **find nothing** or **cannot identify the codebase** (`is_unidentified()` evaluates to true):
       1. **Deep Context Inspection (`inspect_workspace_context`)**: Recursively crawls the repository up to 3 levels deep (filtering noise like `.git`, `node_modules`, `target`, `build`, `dist`, `venv`, `.cache`), collecting up to 60 sample file paths.
       2. **Extension Frequency Analysis**: Tallies and ranks dominant file extensions (e.g. `.zig: 18`, `.kt: 12`, `.ex: 20`).
       3. **Manifest Candidate Extraction**: Flags non-standard or nested build files (`build.zig`, `mix.exs`, `Taskfile.yml`, `justfile`, `Makefile`, etc.).
       4. **Documentation Ingestion**: Ingests up to 20 lines of `README.md` or `README` for explicit build/run instructions.
       5. **Structured LLM Inference**: Queries Gemini / Groq with the assembled context requesting a strict JSON payload (`language`, `build_system`, `compiler_check_cmd`, `test_cmd`, `details`).
       6. **Safe Parsing**: Parses the response with markdown fence recovery and validates command tokens before returning the typed `ProjectEnvironment`.
   - **Output**: Generates a typed `ProjectEnvironment` profile cached on `ZenthreeState` and consumed by **Builder** and **Tester**.
   - **Tool Call**: Exposes `detect_environment` (`{"tool": "detect_environment", "subpath": "<path>"}`).


3. ⚡ **Builder** (`SubAgentRole::Builder`)
   - **Mission**: Writes the required changes as precise `SEARCH/REPLACE` blocks.
   - **Tools Used**: `edit_file`, diff generation engine.

4. 🛡️ **Tester** (`SubAgentRole::Tester`)
   - **Mission**: Consumes the commands identified by **Detector** to execute non-destructive compiler checks and unit tests (`run_project_compiler_check_with_env`, `run_project_tests`), driving automated self-healing loops when errors occur.

### 4-Stage Mission Decomposition:
When a prompt is received, `create_mission_for_prompt` decomposes the mission into 4 coordinated tasks:
1. `task_1_explore` (🔍 Explorer: Analyze workspace context & symbols)
2. `task_2_detect` (🧠 Detector: Detect project toolchain & compiler commands)
3. `task_3_build` (⚡ Builder: Implement changes / surgical diffs)
4. `task_4_test` (🛡️ Tester: Verify syntax, compiler checks & test suite)

### Live Visual Feedback in Editor UI:
- **Message Headers**: Assistant messages display the coordinating badge `Z.E.N.E. » [Sub-Agent Badge]` (e.g. `Z.E.N.E. » 🔍 Explorer`).
- **Mission Status Header**: Real-time pill badge shows active sub-agent next to task completion count (e.g. `2/4 tasks [⚡ Builder]`).
- **Markdown Session Artifacts**: `mission.md` task checklists display assigned sub-agent badges next to task items (e.g. `- [/] **Implement changes** '`⚡ Builder`'`).

---

## 11. File Map & Code Locations

| Component | File Path | Responsibilities |
|---|---|---|
| **Agent Engine** | [`crates/zenthree/src/agent.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/agent.rs) | `AgentMission`, `AgentTask`, `SubAgentRole`, `DetectorAgent`, `ProjectEnvironment`, `AgentToolCall`, `ProviderChain`, `apply_search_replace`, `execute_agent_tool`, `execute_detect_environment`, `run_project_compiler_check_with_env`, `run_project_tests`. |
| **State Orchestrator** | [`crates/zenthree/src/state.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/state.rs) | Event loop, multi-turn AI message handling, editor tab patching, visual diff management, detected environment caching, and mission progress. |
| **AI UI Panel** | [`crates/zenthree/src/ui_layout/ai_panel.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/ai_panel.rs) | Live mission task checklist rendering with sub-agent badges, prompt input text area, model selector, pill-shaped user bubbles, and `Z.E.N.E. » [Sub-Agent]` message tags. |
| **Session Store** | [`crates/zenthree/src/session.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/session.rs) | Session persistence, sub-agent formatted `mission.md`, `implementation_plan.md`, and `walkthrough.md` generation. |
| **Connectome** | [`crates/connectome/src/lib.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/connectome/src/lib.rs) | Biological spiking neural simulation, evidence accumulation, and in-silico ablation models. |
| **Gemini Client** | [`crates/zenthree/src/gemini.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/gemini.rs) | Google Gemini API communication, IPv4 DNS routing, multi-model fallback. |
| **Groq Client** | [`crates/zenthree/src/groq.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/groq.rs) | Groq Cloud API communication and chat message payload structuring. |
| **Project Index** | [`crates/zenthree/src/project_index.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/project_index.rs) | AST symbol exploration, workspace tree synthesis, framework convention discovery. |


