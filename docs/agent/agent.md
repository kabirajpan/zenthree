# Zenthree AI Agent - Comprehensive System Specification

This document details the architecture, internal subsystems, and operational workflows of **Zenthree AI**, the autonomous software engineering agent built directly into the Zenthree code editor.

---

## 1. System Overview

Zenthree AI is an autonomous, in-editor software engineering agent. Unlike typical chat assistants that generate code snippets for manual copy-pasting, Zenthree AI operates directly inside the editor runtime:
- **Autonomous Reasoning & Execution**: Analyzes workspace structure, inspects source files, deduces architectural conventions, and directly writes/patches files.
- **Surgical Diff Engine**: Replaces specific slices of code using search-and-replace blocks rather than rewriting whole files.
- **Biological Deliberation**: Integrates a simulated biological spiking neural connectome to weigh trade-offs and calculate confidence scores.
- **Verification Gate**: Executes non-destructive compiler checks (`cargo check`, `npx tsc`) and initiates self-healing loops to repair diagnostics automatically.
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
Zenthree AI edits existing files by outputting precise diff slices:

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

In contrast to single-prompt agents that fail permanently upon the first error, Zenthree's `AgentMission` manages a resilient task state machine:

### Task State Structure (`AgentTask`)
```rust
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

1. **Automatic Compiler Check**:
   - Inspects workspace for `Cargo.toml` (Rust) or `tsconfig.json` (TypeScript).
   - Executes `cargo check --message-format=short` or `npx tsc --noEmit`.
2. **Error Filtering (`filter_compiler_errors`)**:
   - Filters raw compiler noise down to actionable error diagnostics (`error[E0425]`, file coordinates, compiler suggestions `help:`).
3. **Self-Healing Loop (`build_self_healing_prompt`)**:
   - Generates a targeted compiler repair prompt containing the isolated diagnostics.
   - Allows up to 2 automated healing attempts.
   - If resolved, marks the verification task passed; if persistent, records diagnostics and continues the mission.

---

## 10. File Map & Code Locations

| Component | File Path | Responsibilities |
|---|---|---|
| **Agent Engine** | [`crates/zenthree/src/agent.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/agent.rs) | `AgentMission`, `AgentTask`, `AgentToolCall`, `ProviderChain`, `apply_search_replace`, `execute_agent_tool`, `run_project_compiler_check`. |
| **State Orchestrator** | [`crates/zenthree/src/state.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/state.rs) | Event loop, multi-turn AI message handling, editor tab patching, visual diff management, and mission progress. |
| **AI UI Panel** | [`crates/zenthree/src/ui_layout/ai_panel.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/ai_panel.rs) | Live mission task checklist rendering, prompt input text area, model selector, and pill-shaped user bubbles. |
| **Connectome** | [`crates/connectome/src/lib.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/connectome/src/lib.rs) | Biological spiking neural simulation, evidence accumulation, and in-silico ablation models. |
| **Gemini Client** | [`crates/zenthree/src/gemini.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/gemini.rs) | Google Gemini API communication, IPv4 DNS routing, multi-model fallback. |
| **Groq Client** | [`crates/zenthree/src/groq.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/groq.rs) | Groq Cloud API communication and chat message payload structuring. |
| **Session Store** | [`crates/zenthree/src/session.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/session.rs) | Session persistence, `implementation_plan.md`, `mission.md`, and `walkthrough.md` generation. |
| **Project Index** | [`crates/zenthree/src/project_index.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/project_index.rs) | AST symbol exploration, workspace tree synthesis, framework convention discovery. |
