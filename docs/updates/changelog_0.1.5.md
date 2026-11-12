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

## 5. Native Tree-sitter In-Memory Formatter & Generic Fallback

Eliminated all external Prettier dependencies, replacing them with a native, zero-latency in-process Tree-sitter formatting architecture.

### 5.1 In-Process Syntactic Formatting (`crates/formatter/src/providers/treesitter.rs`)
* **Zero External Dependencies:** Tree-sitter grammars are compiled directly into the binary, executing in ~0ms with no external runtime, Node.js, or CLI prerequisites.
* **14 Built-In Languages:** Full syntactic formatting across Java, C, C++, Rust, Python, Go, JavaScript, TypeScript, HTML, CSS, JSON, YAML, Bash, and Markdown.
* **AST Literal & Comment Protection:** Uses the Tree-sitter concrete syntax tree to distinguish actual code blocks from braces and delimiters inside comments (`// }`) and string literals (`String s = "}";`), preventing invalid formatting shifts.
* **Python Block Engine:** Handles colon-delimited suites (`def`, `class`, `if`, `for`, `while`, `with`, `try`) and dedent keywords (`elif`, `else`, `except`, `finally`).

### 5.2 Generic Fallback Formatter (`crates/formatter/src/providers/generic.rs`)
* **Universal Language Fallback:** Handles any custom, new, or unknown programming language without crashing or failing.
* **Block & Bracket Depth:** Automatically tracks structural block depths (`{`, `}`, `[`, `]`, `(`, `)`), strips trailing whitespace, and normalizes consecutive blank lines and EOF trailing newlines.

### 5.3 Instant GPU Cache Invalidation & Shortcuts
* **Glyph Cache Clearing:** Inactive shaping cache (`shaping_cache.clear()`) is triggered upon buffer formatting, ensuring updated code immediately re-renders on screen.
* **Standard Keybindings:** Mapped to `Shift+Alt+F`, `Ctrl+Shift+F`, and `Ctrl+Alt+L`.
* **Format on Save:** Integrated into document save flows.

---

## 6. Autocomplete & Delimiter Gate Fixes

### 6.1 Overlap & Suffix Normalization (`crates/zenthree/src/completion/mod.rs`)
* **Case-Insensitive Prefix Overlap:** Computes overlap with `eq_ignore_ascii_case`, ensuring typed prefixes like `scan` cleanly replace with `Scanner` rather than duplicating into `scanScanner`.
* **Import Path Overlaps:** Resolved package segment overlaps such as `import ja` + `java` and `import java.ut` + `java.util`.
* **Detail Suffix Trimming:** Cleans LSP completion labels (e.g. `Scanner - java.util` -> `Scanner`) before buffer insertion.

### 6.2 Semicolon & Delimiter Suppression
* **Statement Delimiter Gate:** Autocompletion popups and keyword suggestions are suppressed immediately following statement delimiters (`;`) when prefix is empty and not following an accessor dot.
* **Naked Dot Postfix Suppression:** Postfix snippet templates (`if~`, `ret~`) and generic buffer tokens are suppressed on bare dots (`import java.`), restricting them to valid identifier prefixes.

### 6.3 Brace Splitting (`{}`)
* **Enter Between Braces:** Pressing Enter between `{}` splits into an indented block line rather than accepting an autocomplete candidate.

---

## 7. Verification & Test Suite

* **Formatter Unit Tests:** Verified in-memory Tree-sitter and generic fallback across Java, Python, JavaScript, and custom languages (`5 passed, 0 failed`).
* **Zenthree Autocomplete Tests:** All 25 completion and fuzzy matching tests passed cleanly (`cargo test --bin zenthree completion`).
* **Workspace Compilation:** Verified clean zero-warning build (`cargo check --bin zenthree`).
