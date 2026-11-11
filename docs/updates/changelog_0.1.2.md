# Changelog v0.1.2: Standalone ZENE Agent CLI, Quota & Rate-Limit Resilience, Gemini 3.x Expansion, UI Popovers & Zero-Approval Direct Execution

> **Release Version:** `v0.1.2`  
> **Release Date:** September 2026  
> **Components:** `zenthree-agent`, `zenthree-ide`, `agent-cli`, `zenthra-widgets`  
> **Scope:** Standalone Agent CLI, API Quota & 429 Hang Protection, Expanded Gemini 3.x Models, AI Panel Popover UX Fixes, Zero-Gate Autonomous Coding, Workspace Relative Path Normalization, Markdown Smooth Scrolling  

---

## 1. Standalone Open-Source ZENE Agent CLI (`crates/agent/src/bin/main.rs`)

Decoupled the core `agent` crate from the GUI workspace into an independent, open-source-ready terminal binary (`zene-agent`) while maintaining full architectural parity and shared logic with the Zenthree IDE.

### 1.1 Dedicated CLI Binary & REPL Architecture
* **Binary Target:** Configured `[[bin]]` target `name = "zene-agent"` in [`crates/agent/Cargo.toml`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/Cargo.toml) backed by [`crates/agent/src/bin/main.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/src/bin/main.rs).
* **CLI Arguments & Options:**
  * `--workspace <DIR>`: Sets target project workspace (defaults to current working directory).
  * `--model <ID>`: Selects model (e.g. `gemini-3.5-flash-lite`, `gemini-3.5-flash`, `openai/gpt-oss-120b`).
  * `--provider <NAME>`: Selects provider (`gemini` or `groq`).
  * `--verbose` / `-v`: Activates deep diagnostic mode, streaming step counts, tool arguments, full return payloads, and token timing.
* **Interactive Developer REPL:**
  * Clean ASCII banner displaying active provider, model, workspace, and verbosity status.
  * Real-time tool lifecycle indicators:
    * ⚙ `tool_name (args)` when invoked.
    * ✓ `tool_name finished (N lines output)` on completion.
  * Multi-line input mode (`"""` delimiter) for pasting complex instructions, schemas, or stack traces.
  * REPL Meta Commands:
    * `:help` – Displays command help.
    * `:model <name>` – Live hot-swapping of AI model without restarting the process.
    * `:models` – Lists recommended tested models across Gemini and Groq.
    * `:provider <gemini|groq>` – Switches active provider engine.
    * `:workspace <dir>` – Changes target project directory on the fly.
    * `:verbose` – Toggles verbose telemetry mode.
    * `:clear` – Resets terminal screen.
    * `:quit` / `:exit` – Gracefully exits session.

### 1.2 Dedicated Launch Scripts & Aliases
Created executable helper scripts inside `crates/agent/` and root `scripts/`:
* [`crates/agent/run-normal.sh`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/run-normal.sh): Runs the CLI in clean interactive mode.
* [`crates/agent/run-verbose.sh`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/run-verbose.sh): Runs the CLI with full step-by-step telemetry and raw tool outputs.
* [`crates/agent/test-models.sh`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/test-models.sh): Automated diagnostic script testing active connectivity across Gemini and Groq models.
* Root-level convenience shortcuts in [`scripts/`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/scripts): `run-agent.sh`, `run-agent-verbose.sh`, and `test-agent-models.sh`.
* Comprehensive documentation and usage guide authored in [`crates/agent/README.md`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/agent/README.md).

---

## 2. API Quota Resilience & 429 Rate-Limit Freeze Protection

Diagnosed and eliminated the critical background thread hang that caused the IDE and CLI to stay stuck on `"Working..."` indefinitely.

### 2.1 Groq Rate-Limit Hang Fix (`crates/agent/src/provider/groq.rs`)
* **Root Cause:** Free-tier Groq accounts hit the 200,000 Tokens-Per-Day (TPD) ceiling or 1,000 On-Demand Tokens-Per-Minute (OTPM) rate limit. Groq returned `429 Too Many Requests` with `retry-after: 301` (seconds). The provider loop previously ran `std::thread::sleep(301s)` 3 times in a row, freezing the worker thread for 15 minutes without notifying the UI.
* **Enforced Default Token Ceiling:** Explicitly injected `body["max_tokens"] = json!(1000)` whenever unassigned, ensuring single turns never exceed the free-tier 1,000 OTPM limit.
* **Immediate Quota Exhaustion Break:** Added explicit detection for daily quota messages (`"tokens per day"` and `"TPD"`). The provider aborts immediately with a clear error rather than sleeping.
* **Capped Retry Backoff:** Capped transient retry sleep to a maximum of 5 seconds (`retry_secs.min(5)`), and reduced `MAX_RETRIES` to 2. If `retry_after > 8s`, the provider fails immediately and surfaces actionable advice to switch models.

### 2.2 Gemini Quota Exhaustion Handling (`crates/agent/src/provider/gemini.rs`)
* **Daily Quota Detection:** Added pattern matching for `"Quota exceeded"` and `"RESOURCE_EXHAUSTED"` on HTTP 429 responses.
* **Immediate Error Surfacing:** Breaks immediately on daily limit exhaustion instead of burning retries on a locked key.
* **Capped Sleep Intervals:** Capped backoff delays to `retry_secs.min(5)` with maximum 8s retry threshold.

---

## 3. Gemini 3.x Expansion & Default Model Overhaul

Expanded model enums and configuration across both the UI and backend to expose next-generation Gemini 2.x and 3.x models.

### 3.1 Model Registry Expansion (`crates/zenthree/src/state/types.rs`)
Expanded `AiModel` from 6 to 13 models:
* **Gemini 3.x Series:**
  * `Gemini 3.5 Flash` (`gemini-3.5-flash`)
  * `Gemini 3.5 Flash Lite` (`gemini-3.5-flash-lite`) — High-speed, high-quota default
  * `Gemini 3.6 Flash` (`gemini-3.6-flash`)
  * `Gemini 3.7 Flash` (`gemini-3.7-flash`)
  * `Gemini 3.8 Flash` (`gemini-3.8-flash`)
* **Gemini 2.x Series:**
  * `Gemini 2.5 Flash` (`gemini-2.5-flash`)
  * `Gemini 2.5 Pro` (`gemini-2.5-pro`)
  * `Gemini 2.5 Flash Lite` (`gemini-2.5-flash-lite`)
* **Gemini Aliases:** `Gemini Flash (Latest)`, `Gemini Pro (Latest)`
* **Groq Models:** `GPT-OSS 120B`, `GPT-OSS 20B`, `Qwen 3.8 27B`

### 3.2 Reliable Default Model
* Updated default model from Groq's rate-limited `qwen3.8-27b` to `gemini-3.5-flash` / `gemini-3.5-flash-lite`, delivering instant < 2s agentic responses.

---

## 4. AI Panel Popover UX & Event Handling Fixes

Resolved issues where clicking the Agent Mode button or Model Picker button closed popups instantly.

### 4.1 Outside-Click Event Dismissal Fix (`crates/zenthree/src/ui_layout/ai_panel.rs`)
* **Event Race Condition:** Clicking a pill button or menu item triggered both button click handler and the global outside-click dismiss check in the same frame.
* **Resolution:** Added `ui.clicked = false;` immediately upon pill button and menu row selection, cleanly consuming the click event and preventing dismissal.

### 4.2 Scrollable Menu with Clamped Headroom
* Replaced hardcoded popup bounds with `scroll_y(true)` on model lists with heights up to `340.0px`.
* Adjusted popover anchor geometry (`menu_h + 8.0`) to prevent overlapping with the bottom toolbar or getting truncated at window edges.
* Refined pill button dimensions (`height: 26.0px`, `padding: 3.5px 8.0px`, `radius: 6.0px`, `gap: 5.0px`).

---

## 5. Autonomous Direct Execution (Zero-Approval Gate Removal)

Streamlined the agent from an approval-bound planner into an autonomous pair-programmer.

### 5.1 Skill Instruction Update (`assets/agent/zene/skills/implement/SKILL.md`)
* Removed the mandatory implementation plan approval gate that blocked code execution until the user manually clicked *"Proceed with Plan"*.
* Replaced with direct execution instructions:
  * Inspect the workspace.
  * Direct file creation and code edits via `write_file` / `edit_file`.
  * Validate compilation or tests via `run_terminal`.
  * Provide clean, concise final summaries of all changed files.

### 5.2 Casual Greeting & Intent Disambiguation
* **Greeting Over-Exploration Fix:** Simple user greetings (e.g. `"hi"`, `"hello"`, `"who are you?"`) previously caused models without injected system prompts to list 14,000 files in the workspace.
* Injected explicit system prompt guidelines ensuring conversational prompts receive warm, direct answers without initiating file searches.
* Provisioned safe read-only tools to `AgentIntent::Discussion` to maintain capability without triggering noisy disk scans.

---

## 6. Workspace Path Normalization & Tool Robustness

Hardened filesystem and execution tools against model hallucination and relative path mismatches.

### 6.1 Relative Workspace Root Normalization (`fs/list.rs`, `fs/read.rs`)
* When models generated paths prefixed with the current workspace directory name (e.g. passing `snake_game/Cargo.toml` while already in `/path/to/snake_game`), the tools previously joined them into `/path/to/snake_game/snake_game/Cargo.toml`, resulting in `FileNotFound`.
* Added automatic `strip_prefix(ws_name)` normalization in both `ListDirectoryTool` and `ReadFileTool`.

### 6.2 Directory Inspection Fallback in `read_file` (`fs/read.rs`)
* If an LLM calls `read_file` on a folder path instead of `list_directory`, `ReadFileTool` now inspects the directory entries and returns a formatted directory summary (`[directory: <path> contains N items: ...]`) with an actionable hint to read specific files, eliminating fatal errors.

### 6.3 Workspace-Aware Terminal Execution (`tools/terminal/exec.rs`)
* Updated `RunTerminalTool` with `RunTerminalTool::with_workspace(path)`: terminal commands automatically execute inside the active workspace directory when no explicit `cwd` parameter is provided.

### 6.4 Risk-Aware Tool Filtering (`tools/registry.rs`)
* Added `ToolRegistry::list_names_by_risk(ToolRisk)` and `ToolRegistry::safe_names()` to dynamically filter safe `ReadOnly` tools from mutating tools.

---

## 7. Editor Buffer & Markdown Preview Polish

### 7.1 Native Smooth Scrolling in Markdown Preview (`markdown_preview.rs`)
* Migrated markdown preview from manual mouse-wheel event interception to native container-level scrolling (`.scroll_y(true)`).
* Added persistent scroll offset synchronization across editor tab switches using unique per-tab container IDs (`Id::from_u64(999_888_000 + tab_idx)`).

### 7.2 Breadcrumbs & Preview Toggle Header (`main_buffer.rs`)
* Fixed breadcrumbs layout to split navigation segments to the left and the Markdown Edit/Preview toggle button to the far right.
* Pixel-aligned breadcrumb icons and chevron dividers.

---

## 8. Verification & Test Suite

* **Unit & Integration Tests:** All 68 core agent tests and 8 workflow test suites pass with 0 errors (`cargo test -p agent`).
* **Live Model Diagnostics:** Validated real-time streaming and tool calls across `gemini-3.5-flash-lite`, `gemini-3.5-flash`, and `gemini-2.5-flash`.
* **Workspace Integrity:** `cargo check --workspace` compiles cleanly with zero warnings.
