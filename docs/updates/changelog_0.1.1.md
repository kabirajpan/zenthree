# Changelog v0.1.1: Neuromorphic Fly Brain, Tool Consolidation, Agent Upgrades & Unified IDE UI

> **Release Version:** `v0.1.1`  
> **Release Date:** September 2026  
> **Components:** `fly-decision-net`, `zenthree-agent`, `zenthree-ide`, `zenthra-widgets`  
> **Scope:** Biological Connectome Scaling, Tool Consolidation & Duplicate Removal, Agentic Intent Provisioning, Nerd Font Iconography, Panel Theme & Divider Standardization  

---

## 1. Tool Consolidation, Duplicate Removal & Execution Aliasing

Based on `git diff` audit across `crates/agent/src/tools/`:

### 1.1 Redundant Tool Deletion & Consolidation
* **Unified Search Tool:** Deleted duplicate legacy search implementations (`crates/agent/src/tools/search/files.rs` and `crates/agent/src/tools/search/codebase.rs`). Replaced with a single, high-performance `search` tool (`SearchTool` in `unified.rs`) that natively handles:
  * File path queries with wildcard glob matching (e.g. `*.rs`, `test*`, prefix/suffix splits).
  * Fast ripgrep-style content searches across the workspace.
  * Regex and case-sensitive querying.
* **Unified Directory Tree:** Deleted redundant `get_project_structure` (`crates/agent/src/tools/fs/structure.rs`). Replaced with an enhanced `list_directory` tool (`crates/agent/src/tools/fs/list.rs`) supporting:
  * Recursive `depth` parameter (depth $\ge 2$ renders clean ASCII branch trees: `├── `, `└── `).
  * Summary metrics for subdirectories and file counts (e.g., `(3 subdirs, 14 files)`).
  * Formatted file sizes (`B`, `KB`, `MB`) and automatic filtering of noisy directories (`target`, `node_modules`, `dist`, `.git`).
* **Tool Schema Deduplication:** Added `HashSet` tracking in `ToolRegistry::openai_schemas_for` and `ToolRegistry::gemini_schemas_for` to guarantee that duplicate schemas are never passed to remote provider APIs.

### 1.2 Automatic Namespace Stripping & Backward Compatibility
* **Namespace Hallucination Stripping:** Models (e.g. Groq/Llama or GPT-OSS) frequently hallucinate tool namespaces such as `repo_browser.get_project_structure` or `tools.read_file`. `ToolRegistry::find()` now strips namespaces up to the final dot (`.`), routing calls cleanly to their registered tools.
* **Transparent Backward-Compatibility Argument Translation:**
  * When a model emits `find_files { pattern: "..." }`, `ToolRegistry::execute()` automatically maps it to `search { query: "...", in: "files" }`.
  * When a model emits `search_codebase { ... }`, it automatically maps to `search { ..., in: "content" }`.
  * When a model emits `get_project_structure { max_depth: N }`, it automatically maps to `list_directory { depth: N }`.

### 1.3 Read File Deduplication & Overlap Trimming (`fs/read.rs`)
* **Full Read Deduplication:** If a file has already been read in full during the conversation, subsequent `read_file` calls return a lightweight context hint instead of re-emitting hundreds of lines of code.
* **Overlap Trimming:** If the agent reads lines `1..20` and later requests lines `10..30`, lines `10..20` are automatically trimmed out, presenting lines `21..30` with an explicit notice (`lines 10..20 were already provided in prior context`).
* **Covered Range Suppression:** If an exact range has already been inspected, `read_file` alerts the agent that the lines are already in context.
* **Workspace Boundary Security:** Added strict checks preventing `..` parent directory traversal outside the active workspace directory.

### 1.4 Structured Plan Generation & Live Disk Sync (`plan/planner.rs`)
* **Rich Markdown Implementation Plans:** Upgraded `CreatePlanTool` to generate comprehensive markdown plans containing:
  * User Intent & Requirements summary.
  * Files to Change table (`File Path | Action | Purpose`).
  * Before & After code diff blocks with syntax highlighting.
  * Sequential Execution Phases with progress checkboxes.
  * Testing & Verification Suite (`Test Target | Command | Expected Result`).
  * Explicit Approval Gate (`[ ▶ Proceed with Plan ]` button & chat prompt).
* **Live Disk Sync:** Persists `implementation_plan.md` to the workspace root and updates checkmarks (`- [ ]` $\to$ `- [>]` $\to$ `- [x]`) in real-time as steps are updated via `UpdatePlanStepTool`.

### 1.5 Context Manager & Working Memory Preservation (`context/pruner.rs`)
* **Active Turn Memory Protection:** Updated `ContextManager::compact_prior_tool_results` so that under normal token limits ($< 20,000$ chars), it only compacts tool outputs from previous turns *before* the active user message. This preserves working memory of inspected files for the active task.
* **File Header Summarization:** Compacted tool outputs retain the file name and line count header (`[file: src/main.rs (120 lines) (full content held in memory from prior step)]`).
* **Emergency Compaction Fallback:** On `413 Payload Too Large` errors, aggressively compacts older tools to recover under provider TPM limits immediately.

---

## 2. Fly Brain, Neuromorphic Connectome & Training Datasets

### 2.1 Sensory Routing & Intent Disambiguation
* **Object-Targeted Sensory Filtering:** Fixed a sensory pattern collision in `extract_sensory` where queries like *"explain this project"* collapsed into pure conversational theory (`Channel 0: DISCUSSION`) instead of activating codebase inspection (`Channel 1: INSPECTION`).
* **Compound Anchor Cues:** Introduced regex patterns that pair informational verbs (`explain`, `summarize`, `walkthrough`) with concrete project targets (`project`, `codebase`, `repo`, `directory`, `files`).
* **Dynamic Tool Provisioning:** Prevented provider API 400 errors (`"Tool choice is none, but model called a tool"`) by ensuring the orchestrator dynamically provisions read/search tools whenever workspace context is implicated.

### 2.2 Connectome Scaling & FlyWire Dataset Pipeline
* **500-Neuron to Whole-Brain Drosophila Architecture:** Integrated synaptic adjacency matrices and connectome topology from the FlyWire dataset into the sensory-reflex engine.
* **Mushroom Body & Central Complex Dynamics:** Modeled Kenyon cell sparse representations and dopamine-modulated synaptic plasticity for task-conditioned memory retention.
* **Multimodal Behavior Trajectories:** Standardized training pipelines using behavioral trajectories, action logs, and state transition graphs for reflex-level intent prediction.
* **Benchmarking Alignment:** Anchored agent performance metrics against CodeEditorBench and real-world multi-turn editing workflows.

---

## 3. AI Panel UI & Nerd Font Iconography

### 3.1 Refined AI Assistant Interface
* **Docked & Expanded Modes:** Full support for both a docked side-panel (with continuous resizing splitter) and an expanded full-width native drawer mode with side-by-side chat and session history.
* **Calibrated Floating Prompt Container:** 
  * Bottom floating margin increased to `24.0px` with reserved container headroom, preventing overlap with panel edges.
  * Idle border width calibrated to `0.65px` for a crisp, razor-sharp outline.
  * Active focus outline set to `0.75px` with an ambient glow (`accent.with_alpha(0.24)`).

### 3.2 Dedicated Nerd Font Glyphs
Replaced ambiguous symbols and unrendered box placeholders with standardized, high-clarity Nerd Font glyphs:
* **Terminal:** `\u{f489}` (Nerd Font terminal icon)
* **File:** `\u{f0219}` (Document / file glyph)
* **Inbox:** `\u{f48d}` (Inbox / tray glyph)
* **Web:** `\u{f059f}` (Globe / web search glyph)

All glyphs render natively through the cosmic-text provider without fallback artifacts or rendering boxes.

---

## 4. File Tree Iconography & Folder Glyphs

### 4.1 Minimalist Icon Theme Folder Update
Updated folder glyphs in `mini_icons.rs` for the Minimalist icon theme to match the user's preferred Nerd Font symbols:
* **Closed Folder:** `\u{f024b}`
* **Open / Expanded Folder:** `\u{f0770}`

### 4.2 Icon Theme Synchronization
* Synchronized folder icon rendering across all tree items in the Explorer sidebar.
* Aligned Title Bar theme dropdown and settings selector with live previewing of folder glyphs across presets (Minimalist, Catppuccin, Gruvbox, Material).

---

## 5. Panel Themes, Colors & Divider Lines ("Gaps as Lines")

### 5.1 Elimination of Doubled and Tripled Borders
* Audited all adjacent panels (Title Bar, Activity Bar, Sidebar, Editor Buffer, Terminal Panel, AI Panel, Status Bar).
* Removed redundant 4-sided container borders (`.border(colors.border, 1.0)`) that previously stacked into 2px or 3px thick divider lines at panel junctions.
* Replaced container borders with dedicated, single **1.0px dividing lines** using `colors.border`.

### 5.2 Comprehensive Boundary Standardization
* **Horizontal Dividers:**
  * **Title Bar Bottom:** Exact 1.0px line (`pos: [0.0, BAR_H - 1.0], size: [ui.width, 1.0]`) separating title bar from workspace.
  * **Editor Tab Bar Bottom:** Continuous 1.0px line across the entire editor width separating tabs from code buffer.
  * **Explorer Header Bottom:** Uniform 1.0px divider line beneath the folder title row.
  * **Terminal Panel Header Bottom:** Crisp 1.0px divider line separating terminal header from terminal output screen.
  * **Status Bar Top:** Single 1.0px line across the full window width separating middle workspace from status bar.
* **Vertical Dividers & Resizing Splitters:**
  * **Activity Bar $\leftrightarrow$ Sidebar:** Exact 1.0px vertical line at `x = ACTIVITY_BAR_WIDTH - 1.0`.
  * **Sidebar $\leftrightarrow$ Editor Splitter:** Pixel-aligned 1.0px line in `colors.border` (idle), expanding to 2.0px in `colors.accent` when hovered or actively dragged, with a centered 36px pill grip handle.
  * **Editor $\leftrightarrow$ AI Panel Splitter:** Aligned with sidebar splitter styling: exact same `colors.border` idle color (eliminating previous `with_alpha(0.7)` opacity mismatch), `colors.accent.with_alpha(0.8)` hover, and 36px pill handle.
  * **AI Chat $\leftrightarrow$ Session History:** Standardized full-height 1.0px vertical line using `colors.border`.

### 5.3 Color Token Harmonization
* Eliminated hardcoded dark colors (`#121214`, `#0e0e11`, `#1f1f23`) in the AI panel.
* Replaced with cohesive reactive theme tokens:
  * **Base Background (`colors.bg_base`):** Window backdrop, Activity bar, Terminal body.
  * **Sidebar Background (`colors.bg_sidebar`):** Title bar, Sidebar, Panel headers, Status bar.
  * **Editor Background (`colors.bg_editor`):** Code buffer, active tab, breadcrumb path bar.
  * **Dividers & Splitters (`colors.border`):** Uniform boundary lines across the IDE.
* Guaranteed seamless switching and visual harmony between Dark Mode, Light Mode, and Glassmorphism.

---

## 6. Additional Bug Fixes & Stability Improvements

* **Terminal Height Clamping:** Adjusted terminal body height calculation (`height - 28.0 - 1.0`) to account for the header divider, eliminating bottom overflow.
* **Breadcrumb & Popup Alignment:** Pixel-aligned breadcrumb segment popup dropdowns with the bottom of the path bar.
* **Redraw & Event Loop Optimization:** Ensured all splitter drag handles and hover states trigger efficient redraw notifications without polling loops.
* **Comprehensive Test Suite:** All 122 unit tests, agent workflow tests, and workspace integration tests pass with 0 failures (`cargo test --workspace`).
