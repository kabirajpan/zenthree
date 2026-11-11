# Changelog v0.1.3: Modular Sidebar Panels, Full Git Source Control, File Tree Git Badges (M/U), Native Outline-Free Inputs & Event-Loop Lag Elimination

> **Release Version:** `v0.1.3`  
> **Release Date:** September 2026  
> **Components:** `zenthree-ide`, `zenthra-state`, `ui_layout`, `git`, `sidebar`  
> **Scope:** Modular Sidebar Panel Architecture, Full Git Source Control Integration, Native Outline-Free Inputs, File Tree Git Status Badges (`M`/`U`), Infinite Redraw Lag Elimination (`ArcSignal::with_silent_mut`), and Header Highlight Calibration  

---

## 1. Modular Sidebar Panel Architecture & Updated Iconography

Decoupled the monolithic sidebar into distinct, maintainable panel modules while upgrading the primary Activity Bar navigation iconography.

### 1.1 Dedicated Panel Modules
* **Extracted Renderers:** Separated sidebar rendering into isolated components:
  * [`crates/zenthree/src/ui_layout/git_panel.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/git_panel.rs): Comprehensive source control panel with staging, committing, branch management, and commit history graph.
  * [`crates/zenthree/src/ui_layout/notes_panel.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/notes_panel.rs): Dedicated notes & scratchpad panel.
  * [`crates/zenthree/src/ui_layout/extensions_panel.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/extensions_panel.rs): Language tools, marketplace, and installed extensions manager.
  * [`crates/zenthree/src/ui_layout/sidebar.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/sidebar.rs): Focused exclusively on workspace file exploration, presets, tree navigation, and context operations.
* **Module Registration:** Exported and connected all panel renderers through [`crates/zenthree/src/ui_layout/mod.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/ui_layout/mod.rs).

### 1.2 Activity Bar Iconography Overhaul
* **Files / Explorer:** Updated icon to `\u{f0645}` (File tree) for modern code editor styling.
* **Git Source Control:** Configured with `\u{ea68}` and dynamic real-time badge overlay indicating pending change counts (`staged.len() + unstaged.len()`).
* **Notes Panel:** Configured with `\u{f1782}` icon and custom active indicator bar.
* **Extensions / Plugins:** Updated icon to `\u{f0431}` (Puzzle piece).
* **Settings:** Configured with `\u{f013}` (Gear).

---

## 2. Full Git Source Control Integration & Native Panel Styling

Imported and consolidated comprehensive Git version control capabilities into `ZenthreeState` and `git_panel.rs`.

### 2.1 Git Operations State Machine (`crates/zenthree/src/state/git.rs`)
* **State Structs:** Defined complete data types for `GitFileStatusCode`, `GitChangeItem`, `GitCommitLogItem`, and `GitRepoState` in [`crates/zenthree/src/state/types.rs`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/state/types.rs).
* **Core Git Operations:**
  * `refresh_git_status()`: Detects repository status, active branch (or detached HEAD), parsed porcelain changes, and commit log graph.
  * `git_stage_file()`, `git_unstage_file()`, `git_stage_all()`, `git_unstage_all()`.
  * `git_discard_file()`: Reverts modified files or cleans untracked files safely.
  * `git_commit()`: Commits staged changes with user commit message.
  * `git_init()`: Initializes new Git repositories in current workspace.
* **Interactive Commit Graph:** Renders commit nodes, lanes, hashes, authors, relative timestamps, and branch ref badges with adjustable vertical split-pane resizing.

### 2.2 Native Commit Message Box & Highlight Calibration
* **Native Input (No Outlines):** Stripped all outlines and borders from the commit message text area (`border(Color::TRANSPARENT, 0.0)` and `focus_border(Color::TRANSPARENT, 0.0)`), backed by clean native background and padding.
* **Accordion Header Highlight Fix:** Fixed overly bright header hover styling on `CHANGES`, `STAGED CHANGES`, and `Graph` titles. Replaced `colors.highlight.with_alpha(0.6)` (which forced 60% solid white) with standard `colors.highlight` (`rgba(255, 255, 255, 0.04)`), matching the subtle dark aesthetic of the editor.

---

## 3. Native Extension Search Input Box

Modernized the Extension panel search bar into a native, outline-free input matching the editor's design system.

### 3.1 Outline-Free Search Container (`crates/zenthree/src/ui_layout/extensions_panel.rs`)
* Replaced boxed 1px border container (`.border(colors.border, 1.0)`) with flat native styling: `.border(Color::TRANSPARENT, 0.0)` and `.bg(colors.bg_base)`.
* Applied `.no_bg()` and `.no_border()` to `ui.input`, eliminating all widget borders and focus outlines.
* Added interactive clear button (`\u{f00d}`) that smoothly toggles when search query is populated.

---

## 4. File Tree Git Status Indicators (`M` & `U`)

Restored two-column aligned Git status letters directly in the sidebar file explorer tree.

### 4.1 `SpaceBetween` File Tree Row Layout (`crates/zenthree/src/ui_layout/sidebar.rs`)
* **Two-Column Distribution:** Configured `item_container` with `.halign(Align::SpaceBetween)` and `.no_wrap()`.
* **Left Content Block:** Wrapped indentation, tree guides, folder toggle arrows, file/folder icons, and filename in a single `.no_wrap()` container.
* **Right Badge Container:** Positioned Git status badges cleanly against the right margin with `.padding_right(2.0)`.

### 4.2 Git Status Badges & Colors
* **Status Badges:**
  * **`M`** (Yellow `#eeb42e`): Modified files.
  * **`U`** (Green `#b8bb26`): Untracked / new files.
  * **`A`** (Light Green): Staged additions.
  * **`D`** (Red): Deleted files.
  * **`R`** (Cyan): Renamed files.
  * **`!`** (Orange): Merge conflicts.
  * Directories containing modified contents display a subtle indicator dot.
* **Dynamic Filename Coloring:** Filename text color dynamically reflects active Git status (yellow for modified, green for untracked).
* **Canonical Path Matching:** Enhanced [`get_git_statuses`](file:///home/kabir/Desktop/projects/personal/framworks/zenthra-v2/apps/zenthree/crates/zenthree/src/state/workspace.rs) to insert both workspace-relative and canonicalized filesystem paths, ensuring status badges never miss on symlinks or deep directories.

---

## 5. Performance Optimization: Infinite Redraw Lag Elimination

Eliminated an issue where the IDE ran at 1,000+ FPS in an infinite re-render loop at 100% CPU usage.

### 5.1 Root Cause Analysis
* On every frame, `main.rs` executed `state.drain_ai_updates()` and `state.poll_file_tree_changes()`.
* Both functions invoked `ArcSignal::with_mut(...)` even when the queues were empty or no files had changed.
* `with_mut(...)` unconditionally invoked `self.notify()`, calling `trigger_redraw_hook()`.
* `trigger_redraw_hook()` queued a `UserEvent(())` via `winit`'s event proxy, causing `winit` to immediately schedule another frame.
* This generated a continuous, runaway redraw loop that pegged the CPU at 100% and starved the event queue of keyboard and mouse events, causing noticeable UI stutter and lag.

### 5.2 Resolution & Silent Mutation (`crates/zenthra-state/src/arc_signal.rs`)
* **`ArcSignal::with_silent_mut`:** Added a thread-safe mutation method that modifies signal values without firing notifications or triggering redraw hooks.
* **Optimized Consumers:**
  * Updated `drain_ai_updates`: Checks if the queue is empty first before locking, and drains using `with_silent_mut`.
  * Updated `take_pending`: Returns `false` immediately if the pending flag is unset, and clears using `with_silent_mut`.
* **Results:** When idle, no spurious events are dispatched, CPU drops to 0%, and mouse/keyboard latency is completely eliminated.
