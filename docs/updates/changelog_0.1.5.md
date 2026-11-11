# Changelog v0.1.5: Collapsible Trajectory Card Restoration, Real-Time Braille Spinners, Workspace Isolation & Non-Intrusive Assistant Identity

> **Release Version:** `v0.1.5`  
> **Release Date:** September 2026  
> **Components:** `agent`, `zenthree`, `ui_layout`, `state`, `acp`, `external_provider`, `fs`  
> **Scope:** Collapsible exploration trajectory card restoration, real-time animated braille spinner for active tool runs, strict `.zenthree` internal workspace isolation with selective notes/skills whitelist, and conditional assistant identity calibration to eliminate unprompted self-branding.

---

## 1. Collapsible Exploration & Tool Trajectory Architecture

Restored and refined the collapsible trajectory grouping in the AI chat panel, ensuring all tool calls (file reading, directory listing, searching, and command executions) are consolidated into a clean, collapsible card rather than flooding the conversation stream with standalone items.

### 1.1 Tool Call Routing Fix (`crates/agent/src/external_provider/antigravity_cli/provider.rs`)
* **Root Cause:** Antigravity CLI ACP events format tool titles as `"Running <tool_name>"` (e.g. `Running list_directory`, `Running client_view_file`). A naive prefix check `title.starts_with("Running ")` was intercepting all tool events and misclassifying them as `run_terminal` shell executions.
* **Resolution:** Removed the overly broad title check. ACP tool calls now preserve their genuine tool identifiers (`read_text_file`, `list_directory`, `grep_search`, `client_view_file`), routing them into standard `AcpEvent::ToolCallStarted` and `ToolCallCompleted` events.

### 1.2 Unified Trajectory Card Grouping (`crates/zenthree/src/state/ai.rs`)
* Removed top-level interception of terminal executions that previously bypassed trajectory aggregation.
* Terminal runs and tool calls are now recorded as `ExplorationItem::ExecutedCommand`, `ExplorationItem::ReadFile`, and `ExplorationItem::ListDir` within `ExplorationGroup`.
* The card renders a clean summary header (e.g., `Explored 3 files, 2 folders, 1 command`) that users can expand (`▾`) or collapse (`▸`) on demand.

---

## 2. Real-Time Animated Braille Spinner for Active Trajectory Items

Replaced static placeholder glyphs on actively running trajectory items with a smooth, non-blocking braille spinner animation.

### 2.1 Animation Loop & Frame Calculation (`crates/zenthree/src/ui_layout/ai_panel.rs`)
* **10-Frame Braille Sequence:** Added standard braille spinner frames:
  ```rust
  const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
  ```
* **Time-Based Frame Selection:** Implemented `get_thinking_spinner_frame()` derived from UNIX timestamp milliseconds (`(elapsed_millis / 80) % 10`), ensuring steady 80ms frame pacing.
* **Reactive Redraw Scheduling:** Wired `ui.request_redraw_after(Duration::from_millis(80))` whenever any exploration item is in an active state (`Running`, `Reading`, `Searching`, `Exploring`), providing silky smooth animation without blocking the UI thread or pinning CPU cores.
* **Terminal Status Glyph:** Once completed, active spinners transition immediately to a green checkmark (`✓`) or failure indicator.

---

## 3. Workspace Isolation & Privacy Guardrails for `.zenthree`

Established strict security and privacy boundaries to prevent autonomous AI agents from inspecting internal IDE state while preserving note-taking and user skill workflows.

### 3.1 Filesystem Read Guardrails (`crates/agent/src/tools/fs/read.rs`)
* **Protected Internal State:** Blocked AI tool access to sensitive IDE state files and internal directories:
  * `.zenthree_sessions/` (chat and session history)
  * `.zenthree_state.json` (window state, workspace cache)
  * Arbitrary root `.zenthree/*` files.
* **Whitelisted User Directories:** Maintained explicit exceptions for legitimate user workflow directories:
  * `.zenthree/notes/` remains fully readable and writable for notes editing and documentation tasks.
  * `.zenthree/skills/` remains accessible for custom assistant skill workflows.

### 3.2 System Prompt Context Separation (`crates/zenthree/src/state/ai.rs`)
* Added explicit instructions in the base prompt:
  * *"Keep `.zenthree/notes/` strictly for notes workflows; do not scan `.zenthree` internals or sessions during general repository tasks."*

---

## 4. Non-Intrusive Assistant Identity & Conditional Self-Branding

Calibrated system prompts across the IDE and CLI agent binary to eliminate unsolicited branding and keep responses focused purely on user code.

### 4.1 Neutral Foundation & Conditional Identity (`crates/zenthree/src/state/ai.rs`, `crates/agent/src/bin/main.rs`)
* **Issue:** Prepending `"You are ZENE, an advanced AI coding assistant embedded in the Zenthree IDE"` caused LLMs to aggressively volunteer IDE identity and assistant branding even when asked standard questions (e.g. *"what API am I using?"*).
* **Calibrated Base Prompt:** Replaced intrusive self-introductions with a neutral instruction:
  > *"You are a helpful, accurate, and concise AI coding assistant."*
* **Environment Context Header:** Encapsulated IDE details into an environment block:
  ```markdown
  Environment: Zenthree IDE (Assistant: ZENE)
  ```
* **Strict Conditional Identity Guideline:** Added explicit behavioral guidance:
  * *"CONDITIONAL IDENTITY: Do NOT talk about Zenthree IDE or introduce yourself as ZENE unless the user explicitly asks about your name, your identity, or the IDE environment."*

---

## 5. Verification & Test Suite

* **Zenthree Workspace Tests:** All 44 unit and integration tests passed cleanly (`cargo test -p zenthree`).
* **Agent Engine Tests:** All 67 unit and protocol tests passed cleanly (`cargo test -p agent`).
* **Manual Verification:** Confirmed that trajectory cards collapse properly, braille spinners animate actively during tool execution, `.zenthree` internal sessions remain shielded, and model responses stay concise without unsolicited branding.
