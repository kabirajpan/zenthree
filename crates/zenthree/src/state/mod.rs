pub mod ai;
pub mod ai_settings;
pub mod editor;
pub mod persistent;
pub mod terminal;
pub mod types;
pub mod workspace;

use crate::theme::{ThemeColors, ThemeMode};
use std::path::{Path, PathBuf};

#[allow(unused_imports)]
pub use persistent::{get_config_path, PersistentState};
pub use types::*;
#[allow(unused_imports)]
pub use ai_settings::*;

pub struct ZenthreeState {
    pub theme: ThemeMode,
    /// Reactive signal for the current theme — subscribing components re-derive colors on change.
    pub theme_signal: zenthra_state::Signal<ThemeMode>,
    pub accent_color: String,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub status_bar_visible: bool,
    pub line_numbers_visible: bool,
    pub zoom_level: f32,
    pub active_resize_sidebar: bool,
    pub tabs: Vec<EditorTab>,
    pub active_tab: Option<usize>,
    pub active_tab_signal: zenthra_state::Signal<Option<usize>>,
    pub current_dir: PathBuf,
    pub has_workspace: bool,
    pub recent_files: Vec<PathBuf>,
    pub sidebar_items: Vec<FileTreeNode>,
    pub expanded_dirs: std::collections::HashSet<PathBuf>,
    pub status_message: String,
    pub status_signal: zenthra_state::Signal<String>,
    /// Timestamp of the last set_status() call — used by tick_status() to auto-clear after 3s.
    pub status_set_at: Option<std::time::Instant>,
    /// Auto-clear effect: fires when status_signal changes, schedules a 3-second clear.
    /// Kept alive for the lifetime of ZenthreeState.
    pub _status_clear_effect: zenthra_state::Effect,
    pub about_open: bool,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
    pub alt_pressed: bool,
    pub highlighter: syntax::SyntaxHighlighter,
    pub formatter: formatter::Formatter,
    pub indent_calc: syntax::IndentCalculator,
    pub clipboard: Option<String>,
    pub last_cursor_activity: std::time::Instant,
    pub active_breadcrumb_popup: Option<BreadcrumbPopup>,
    pub selected_sidebar_path: Option<PathBuf>,
    pub icon_theme: String,
    pub explorer_preset: ExplorerPreset,
    pub explorer_tree_guides: bool,
    #[allow(dead_code)]
    pub explorer_filter_query: String,
    #[allow(dead_code)]
    pub explorer_search_active: bool,
    pub explorer_git_status: bool,
    pub explorer_show_hidden: bool,
    pub terminal_visible: bool,
    pub terminal_height: f32,
    pub active_resize_terminal: bool,
    pub terminal_lines: Vec<TerminalLine>,
    pub terminal_input: String,
    pub terminal_history: Vec<String>,
    pub terminal_history_idx: Option<usize>,
    pub ai_panel_visible: bool,
    pub ai_panel_expanded: bool,
    pub ai_panel_width: f32,
    pub active_resize_ai_panel: bool,
    pub ai_messages: Vec<AiMessage>,
    pub ai_input: String,
    pub user_profile_open: bool,
    pub terminal_session: Option<crate::native_terminal::NativeTerminalSession>,
    pub sidebar_view: SidebarView,
    pub installed_extensions: std::collections::HashSet<String>,
    pub ignored_language_prompts: std::collections::HashSet<String>,
    pub extension_search_query: String,
    pub extension_tab_filter: ExtensionFilterTab,
    pub active_language_prompt: Option<LanguagePromptDialog>,
    pub lsp_manager: crate::lsp::LspManager,
    pub completion_active: bool,
    pub completion_items: Vec<crate::lsp::protocol::CompletionItem>,
    pub completion_selected_idx: usize,
    pub inline_tree_input: Option<InlineTreeState>,
    pub tree_context_menu: Option<TreeContextMenu>,
    pub tree_clipboard: Option<TreeClipboardAction>,
    #[allow(dead_code)]
    pub groq_api_key: Option<String>,
    /// Serialized agent conversation history (JSON). Persisted across messages so the agent
    /// remembers previous turns. Reset when the user clears chat.
    pub ai_session_json: Option<String>,
    /// Thread-safe signal queue for AI updates from the background agent thread.
    /// Background thread pushes `AiUiUpdate` items; the signal auto-triggers a redraw.
    /// The UI thread drains the queue on the next frame via `drain_ai_updates()`.
    pub ai_update_signal: zenthra_state::ArcSignal<Vec<AiUiUpdate>>,
    pub copied_ai_msg: Option<(usize, std::time::Instant)>,
    pub copied_ai_code_id: Option<(u64, std::time::Instant)>,
    /// Render-side markdown parse cache: maps a FNV hash of message content → pre-parsed blocks.
    /// Stored as `Box<dyn Any>` so this module stays free of the `MarkdownBlock` type.
    pub ai_block_cache: std::collections::HashMap<u64, Box<dyn std::any::Any + Send + Sync>>,

    pub selected_ai_model: AiModel,
    pub ai_model_menu_open: bool,
    pub agent_mode: AgentMode,
    pub agent_mode_menu_open: bool,
    pub pending_reviews: Vec<FileDiffReview>,
    pub active_review_idx: Option<usize>,
    pub show_modified_files_popup: bool,
    pub font_adapter: Option<zenthra_text::prelude::CosmicFontProvider>,
    /// Cached OS clipboard handle — lazily initialized to avoid opening a new
    /// Wayland/X11 socket on every copy/paste operation.
    pub os_clipboard: Option<arboard::Clipboard>,
    pub fs_watcher: crate::watcher::WorkspaceWatcher,

    /// Session history panel state
    pub ai_sessions_visible: bool,
    pub ai_session_id: Option<String>,
    pub ai_sessions: Vec<SessionSummary>,
    pub ai_sessions_search_query: String,
    pub ai_expanded_projects: std::collections::HashSet<String>,
    pub ai_cancel_token: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// AI 3-dot dropdown menu open state
    pub ai_menu_open: bool,
    /// AI & Agent settings state (appearance, general, model, customization)
    pub ai_settings: AiSettingsState,
}

impl ZenthreeState {
    pub fn new() -> Self {
        // Try workspace argument, fallback to std::env::current_dir(), fallback to home dir
        let args: Vec<String> = std::env::args().collect();
        let mut current_dir = std::env::current_dir()
            .unwrap_or_else(|_| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")));
        let mut has_workspace = false;

        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                if path.is_dir() {
                    current_dir = path;
                    has_workspace = true;
                } else if path.is_file() {
                    if let Some(parent) = path.parent() {
                        current_dir = parent.to_path_buf();
                        has_workspace = true;
                    }
                }
            }
        }

        let highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::dark());
        let formatter = formatter::Formatter::new();
        let indent_calc = syntax::IndentCalculator::new();

        let mut state = Self {
            theme: ThemeMode::Dark,
            theme_signal: zenthra_state::Signal::new(ThemeMode::Dark),
            accent_color: "blue".to_string(),
            icon_theme: "neovim_mini".to_string(),
            explorer_preset: ExplorerPreset::ModernMinimalist,
            explorer_tree_guides: false, // Default: Zed style without guidelines
            explorer_filter_query: String::new(),
            explorer_search_active: false,
            explorer_git_status: true,
            explorer_show_hidden: false,
            sidebar_visible: true,
            sidebar_width: 250.0,
            status_bar_visible: true,
            line_numbers_visible: true,
            zoom_level: 1.0,
            active_resize_sidebar: false,
            tabs: Vec::new(),
            active_tab: None,
            active_tab_signal: zenthra_state::Signal::new(None),
            current_dir,
            has_workspace,
            recent_files: Vec::new(),
            sidebar_items: Vec::new(),
            expanded_dirs: std::collections::HashSet::new(),
            status_message: "Welcome to Zenthree!".to_string(),
            status_signal: zenthra_state::Signal::new("Welcome to Zenthree!".to_string()),
            status_set_at: Some(std::time::Instant::now()),
            // Placeholder — will be replaced with the real effect after construction.
            _status_clear_effect: zenthra_state::Effect::run(|| {}),
            about_open: false,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            highlighter,
            formatter,
            indent_calc,
            clipboard: None,
            last_cursor_activity: std::time::Instant::now(),
            active_breadcrumb_popup: None,
            terminal_visible: false,
            terminal_height: 200.0,
            active_resize_terminal: false,
            terminal_lines: vec![TerminalLine {
                text: "Zenthree Native Terminal v0.1.0".to_string(),
                is_command: false,
                is_error: false,
            }],
            terminal_input: String::new(),
            terminal_history: Vec::new(),
            terminal_history_idx: None,
            ai_panel_visible: false,
            ai_panel_expanded: false,
            ai_panel_width: 340.0,
            active_resize_ai_panel: false,
            ai_messages: Vec::new(),
            ai_input: String::new(),
            user_profile_open: false,
            terminal_session: None,
            selected_sidebar_path: None,
            sidebar_view: SidebarView::Explorer,
            installed_extensions: {
                let mut set = std::collections::HashSet::new();
                set.insert("rust".to_string());
                set.insert("python".to_string());
                set.insert("javascript".to_string());
                set
            },
            ignored_language_prompts: std::collections::HashSet::new(),
            extension_search_query: String::new(),
            extension_tab_filter: ExtensionFilterTab::All,
            active_language_prompt: None,
            lsp_manager: crate::lsp::LspManager::new(),
            completion_active: false,
            completion_items: Vec::new(),
            completion_selected_idx: 0,
            inline_tree_input: None,
            tree_context_menu: None,
            tree_clipboard: None,
            groq_api_key: std::env::var("GROQ_API_KEY").ok(),
            ai_session_json: None,
            ai_update_signal: zenthra_state::ArcSignal::new(Vec::new()),
            copied_ai_msg: None,
            copied_ai_code_id: None,
            ai_block_cache: std::collections::HashMap::new(),
            selected_ai_model: AiModel::default(),
            ai_model_menu_open: false,
            agent_mode: AgentMode::default(),
            agent_mode_menu_open: false,
            pending_reviews: Vec::new(),
            active_review_idx: None,
            show_modified_files_popup: false,
            font_adapter: None,
            os_clipboard: None,
            fs_watcher: crate::watcher::WorkspaceWatcher::new(),
            ai_sessions_visible: false,
            ai_session_id: None,
            ai_sessions: Vec::new(),
            ai_sessions_search_query: String::new(),
            ai_expanded_projects: std::collections::HashSet::new(),
            ai_cancel_token: None,
            ai_menu_open: false,
            ai_settings: AiSettingsState::default(),
        };

        // Restore previous session preferences (theme, accent, extensions, etc.)
        #[cfg(not(test))]
        state.load_persistent_state();

        // Refresh saved AI session history
        state.refresh_ai_sessions();

        // If CLI argument was explicitly provided, open specified file or workspace
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                if path.is_dir() {
                    state.open_workspace(path);
                } else if path.is_file() {
                    state.open_file(&path);
                    if let Some(parent) = path.parent() {
                        state.current_dir = parent.to_path_buf();
                        state.has_workspace = true;
                    }
                }
            }
        } else {
            // Default startup without arguments:
            // DO NOT open any project by default.
            // Start clean with an empty untitled buffer!
            state.has_workspace = false;
            state.sidebar_items.clear();
            state.tabs.clear();
            state.new_untitled_file();
        }

        if state.has_workspace {
            state.rebuild_tree();
            state.fs_watcher.watch(&state.current_dir);
        }

        // Wire the status auto-clear Effect to the real status_signal so it
        // tracks changes. The actual clearing is done by tick_status() in the
        // main frame loop (woken by the existing 250ms terminal redraw timer).
        {
            let sig = state.status_signal.clone();
            state._status_clear_effect = zenthra_state::Effect::new(&sig, move |_msg| {
                let _ = _msg; // subscription keeps the effect alive
            });
        }

        state
    }

    pub fn set_active_tab(&mut self, idx: Option<usize>) {
        self.active_tab = idx;
        self.active_tab_signal.set(idx);
    }

    /// Set the active theme and fire the reactive theme_signal.
    /// All subscribers (e.g. Effects watching theme changes) are notified.
    pub fn set_theme(&mut self, mode: ThemeMode) {
        self.theme = mode;
        self.theme_signal.set(mode);
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        let text = msg.into();
        self.status_message = text.clone();
        self.status_set_at = Some(std::time::Instant::now());
        self.status_signal.set(text);
    }

    /// Called once per frame. Clears the status message if it has been shown for >= 3 seconds.
    /// Returns true if the status was cleared (so caller can request a redraw).
    pub fn tick_status(&mut self) -> bool {
        if let Some(set_at) = self.status_set_at {
            if !self.status_message.is_empty() && set_at.elapsed() >= std::time::Duration::from_secs(3) {
                self.status_message.clear();
                self.status_set_at = None;
                // Use set_if_changed to avoid firing a spurious redraw if the signal
                // is already empty (guards against tick_status → redraw → tick_status loops).
                self.status_signal.set_if_changed(String::new());
                return true;
            }
        }
        false
    }

    pub fn active_review(&self) -> Option<&FileDiffReview> {
        self.active_review_idx.and_then(|idx| self.pending_reviews.get(idx))
    }

    pub fn review_idx_for_path(&self, path: &Path) -> Option<usize> {
        self.pending_reviews.iter().position(|r| path_matches(&r.path, path))
    }

    pub fn sync_active_review_for_tab(&mut self, tab_idx: usize) {
        if let Some(tab) = self.tabs.get(tab_idx) {
            if let Some(ref p) = tab.path {
                if let Some(rev_idx) = self.review_idx_for_path(p) {
                    self.active_review_idx = Some(rev_idx);
                }
            }
        }
    }

    /// Call after ANY buffer mutation (typing, paste, cut, undo, redo, format, completion insert).
    /// Atomically:
    ///   1. marks tab dirty
    ///   2. bumps version (invalidates render caches)
    ///   3. clears shaping cache (forces text re-layout)
    ///   4. notifies LSP so the language server sees the updated buffer
    ///
    /// Previously steps 1-3 were scattered across 6+ call sites,
    /// and LSP was NEVER notified from undo/redo/cut/paste/format — a real bug.
    pub fn on_buffer_changed(&mut self, tab_idx: usize) {
        if tab_idx >= self.tabs.len() {
            return;
        }
        self.tabs[tab_idx].is_dirty = true;
        self.tabs[tab_idx].version = self.tabs[tab_idx].version.wrapping_add(1);

        // Only invalidate lines near the cursor rather than wiping the entire shaping cache.
        // A single-char insert/delete only changes the cursor line (and possibly the next line
        // if it wraps). Clearing ALL cached shapes was forcing a full re-shape of every visible
        // line (~50+) on every keypress.
        let cursor_row = self.tabs[tab_idx].editor.cursor().row;
        self.tabs[tab_idx].shaping_cache.remove(&cursor_row);
        if cursor_row > 0 {
            self.tabs[tab_idx].shaping_cache.remove(&(cursor_row - 1));
        }
        self.tabs[tab_idx].shaping_cache.remove(&(cursor_row + 1));

        if let Some(path) = self.tabs[tab_idx].path.clone() {
            let content = self.tabs[tab_idx].editor.buffer.to_string();
            self.lsp_manager.on_document_change(&path, &content);
        }
    }

    #[allow(dead_code)]
    pub fn accept_hunk(&mut self, _hunk_idx: usize) {
        self.accept_current_review();
    }

    #[allow(dead_code)]
    pub fn reject_hunk(&mut self, _hunk_idx: usize) {
        self.reject_current_review();
    }

    pub fn accept_current_review(&mut self) {
        if let Some(idx) = self.active_review_idx {
            if idx < self.pending_reviews.len() {
                let rev = self.pending_reviews.remove(idx);
                if let Some(parent) = rev.path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&rev.path, &rev.proposed_content);
                for tab in &mut self.tabs {
                    if tab.path.as_ref().map(|p| path_matches(p, &rev.path)).unwrap_or(false) {
                        tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&rev.proposed_content));
                        tab.shaping_cache.clear();
                        tab.lines_added = 0;
                        tab.lines_removed = 0;
                        tab.is_dirty = false;
                        tab.version += 1;
                    }
                }
                if self.pending_reviews.is_empty() {
                    self.active_review_idx = None;
                } else {
                    let next_idx = idx.min(self.pending_reviews.len() - 1);
                    self.active_review_idx = Some(next_idx);
                    let next_path = self.pending_reviews[next_idx].path.clone();
                    self.open_file(&next_path);
                }
            }
        }
    }

    pub fn reject_current_review(&mut self) {
        if let Some(idx) = self.active_review_idx {
            if idx < self.pending_reviews.len() {
                let rev = self.pending_reviews.remove(idx);
                if rev.is_new_file {
                    let _ = std::fs::remove_file(&rev.path);
                    if let Some(tab_idx) = self.tabs.iter().position(|t| t.path.as_ref().map(|p| path_matches(p, &rev.path)).unwrap_or(false)) {
                        self.close_tab(tab_idx);
                    }
                } else {
                    let _ = std::fs::write(&rev.path, &rev.original_content);
                    for tab in &mut self.tabs {
                        if tab.path.as_ref().map(|p| path_matches(p, &rev.path)).unwrap_or(false) {
                            tab.editor = ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&rev.original_content));
                            tab.shaping_cache.clear();
                            tab.lines_added = 0;
                            tab.lines_removed = 0;
                            tab.is_dirty = false;
                            tab.version += 1;
                        }
                    }
                }
                if self.pending_reviews.is_empty() {
                    self.active_review_idx = None;
                } else {
                    let next_idx = idx.min(self.pending_reviews.len() - 1);
                    self.active_review_idx = Some(next_idx);
                    let next_path = self.pending_reviews[next_idx].path.clone();
                    self.open_file(&next_path);
                }
            }
        }
    }


    #[allow(dead_code)]
    pub fn next_review_file(&mut self) {
        if !self.pending_reviews.is_empty() {
            let next = match self.active_review_idx {
                Some(idx) => (idx + 1) % self.pending_reviews.len(),
                None => 0,
            };
            self.active_review_idx = Some(next);
            let path = self.pending_reviews[next].path.clone();
            self.open_file(&path);
        }
    }

    #[allow(dead_code)]
    pub fn prev_review_file(&mut self) {
        if !self.pending_reviews.is_empty() {
            let prev = match self.active_review_idx {
                Some(0) => self.pending_reviews.len() - 1,
                Some(idx) => idx - 1,
                None => 0,
            };
            self.active_review_idx = Some(prev);
            let path = self.pending_reviews[prev].path.clone();
            self.open_file(&path);
        }
    }

    pub fn accept_all_reviews(&mut self) {
        while !self.pending_reviews.is_empty() {
            self.active_review_idx = Some(0);
            self.accept_current_review();
        }
        self.show_modified_files_popup = false;
    }

    pub fn reject_all_reviews(&mut self) {
        while !self.pending_reviews.is_empty() {
            self.active_review_idx = Some(0);
            self.reject_current_review();
        }
        self.show_modified_files_popup = false;
    }

    #[allow(dead_code)]
    pub fn reject_review_for_path(&mut self, path: &std::path::Path) {
        if let Some(pos) = self.pending_reviews.iter().position(|r| path_matches(&r.path, path)) {
            self.active_review_idx = Some(pos);
            self.reject_current_review();
        }
    }

    pub fn colors(&self) -> ThemeColors {
        ThemeColors::get_custom(self.theme, &self.accent_color)
    }

    pub fn is_resizing(&self) -> bool {
        self.active_resize_sidebar || self.active_resize_terminal || self.active_resize_ai_panel
    }
}

impl Default for ZenthreeState {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ZenthreeState {
    fn drop(&mut self) {
        self.cancel_active_ai();
        #[cfg(not(test))]
        self.save_persistent_state();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawDiffOp<T> {
    Equal(T),
    Delete(T),
    Insert(T),
}

pub fn compute_line_diff(original: &str, proposed: &str) -> (Vec<(String, DiffLineState)>, String) {
    if original.is_empty() && proposed.is_empty() {
        return (Vec::new(), String::new());
    }
    if original.is_empty() {
        let diff: Vec<(String, DiffLineState)> = proposed
            .lines()
            .map(|l| (l.to_string(), DiffLineState::Added))
            .collect();
        let preview = diff.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n");
        return (diff, preview);
    }
    if proposed.is_empty() {
        let diff: Vec<(String, DiffLineState)> = original
            .lines()
            .map(|l| (l.to_string(), DiffLineState::Removed))
            .collect();
        let preview = diff.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n");
        return (diff, preview);
    }

    let orig_lines: Vec<&str> = original.lines().collect();
    let prop_lines: Vec<&str> = proposed.lines().collect();

    // 1. Trim common prefix
    let mut prefix_len = 0;
    while prefix_len < orig_lines.len()
        && prefix_len < prop_lines.len()
        && orig_lines[prefix_len] == prop_lines[prefix_len]
    {
        prefix_len += 1;
    }

    // 2. Trim common suffix
    let mut orig_end = orig_lines.len();
    let mut prop_end = prop_lines.len();
    while orig_end > prefix_len
        && prop_end > prefix_len
        && orig_lines[orig_end - 1] == prop_lines[prop_end - 1]
    {
        orig_end -= 1;
        prop_end -= 1;
    }

    let orig_mid = &orig_lines[prefix_len..orig_end];
    let prop_mid = &prop_lines[prefix_len..prop_end];

    let n = orig_mid.len();
    let m = prop_mid.len();

    let mut raw_ops = Vec::new();

    if n == 0 && m == 0 {
        // Only identical lines exist
    } else if n == 0 {
        for s in prop_mid {
            raw_ops.push(RawDiffOp::Insert(*s));
        }
    } else if m == 0 {
        for s in orig_mid {
            raw_ops.push(RawDiffOp::Delete(*s));
        }
    } else {
        // Myers diff algorithm with maximum search cap
        let max = n + m;
        let max_d = max.min(2000);
        let offset = max_d as isize;
        let mut v = vec![0isize; 2 * max_d + 1];
        let mut trace: Vec<Vec<isize>> = Vec::new();
        let mut found = false;

        for d in 0..=max_d {
            trace.push(v.clone());
            let d_isize = d as isize;
            for k in (-d_isize..=d_isize).step_by(2) {
                let k_idx = (k + offset) as usize;
                let mut x = if k == -d_isize || (k != d_isize && v[k_idx - 1] < v[k_idx + 1]) {
                    v[k_idx + 1]
                } else {
                    v[k_idx - 1] + 1
                };
                let mut y = x - k;
                while x < n as isize && y < m as isize && orig_mid[x as usize] == prop_mid[y as usize] {
                    x += 1;
                    y += 1;
                }
                v[k_idx] = x;
                if x >= n as isize && y >= m as isize {
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
        }

        if !found {
            // Fallback for massive divergent replacements: all removed, then all added
            for s in orig_mid {
                raw_ops.push(RawDiffOp::Delete(*s));
            }
            for s in prop_mid {
                raw_ops.push(RawDiffOp::Insert(*s));
            }
        } else {
            let mut x = n as isize;
            let mut y = m as isize;

            for d in (1..trace.len()).rev() {
                let v_prev = &trace[d];
                let k = x - y;
                let d_isize = d as isize;
                let prev_k = if k == -d_isize || (k != d_isize && v_prev[(k - 1 + offset) as usize] < v_prev[(k + 1 + offset) as usize]) {
                    k + 1
                } else {
                    k - 1
                };
                let prev_x = v_prev[(prev_k + offset) as usize];
                let prev_y = prev_x - prev_k;

                while x > prev_x && y > prev_y {
                    x -= 1;
                    y -= 1;
                    raw_ops.push(RawDiffOp::Equal(orig_mid[x as usize]));
                }
                if x == prev_x {
                    y -= 1;
                    raw_ops.push(RawDiffOp::Insert(prop_mid[y as usize]));
                } else {
                    x -= 1;
                    raw_ops.push(RawDiffOp::Delete(orig_mid[x as usize]));
                }
            }
            while x > 0 && y > 0 {
                x -= 1;
                y -= 1;
                raw_ops.push(RawDiffOp::Equal(orig_mid[x as usize]));
            }
            raw_ops.reverse();
        }
    }

    let mut diff = Vec::new();

    // 1. Emit common prefix
    for line in &orig_lines[..prefix_len] {
        diff.push((line.to_string(), DiffLineState::Unchanged));
    }

    // 2. Emit middle changes grouped by hunk:
    // Any sequence of deletions and insertions is grouped so that ALL deletions (-)
    // appear first in red sequence, followed by ALL additions (+) in green sequence.
    let mut hunk_removed = Vec::new();
    let mut hunk_added = Vec::new();

    let flush_hunk = |diff: &mut Vec<(String, DiffLineState)>,
                      removed: &mut Vec<&str>,
                      added: &mut Vec<&str>| {
        for line in removed.drain(..) {
            diff.push((line.to_string(), DiffLineState::Removed));
        }
        for line in added.drain(..) {
            diff.push((line.to_string(), DiffLineState::Added));
        }
    };

    for op in raw_ops {
        match op {
            RawDiffOp::Equal(line) => {
                flush_hunk(&mut diff, &mut hunk_removed, &mut hunk_added);
                diff.push((line.to_string(), DiffLineState::Unchanged));
            }
            RawDiffOp::Delete(line) => {
                hunk_removed.push(line);
            }
            RawDiffOp::Insert(line) => {
                hunk_added.push(line);
            }
        }
    }
    flush_hunk(&mut diff, &mut hunk_removed, &mut hunk_added);

    // 3. Emit common suffix
    for line in &orig_lines[orig_end..] {
        diff.push((line.to_string(), DiffLineState::Unchanged));
    }

    let preview_text = diff.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n");
    (diff, preview_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explorer_preset_defaults_and_toggles() {
        let mut state = ZenthreeState::new();
        assert_eq!(state.explorer_preset, ExplorerPreset::ModernMinimalist);
        // Default: Zed style without guidelines
        assert!(!state.explorer_tree_guides);
        assert!(state.explorer_git_status);
        assert!(!state.explorer_show_hidden);

        state.explorer_preset = ExplorerPreset::NeovimSnacks;
        assert_eq!(state.explorer_preset, ExplorerPreset::NeovimSnacks);

        state.explorer_preset = ExplorerPreset::ClassicTree;
        assert_eq!(state.explorer_preset, ExplorerPreset::ClassicTree);

        state.explorer_show_hidden = true;
        assert!(state.explorer_show_hidden);
    }

    #[test]
    fn test_collapse_all_folders() {
        let mut state = ZenthreeState::new();
        state.expanded_dirs.insert(PathBuf::from("/tmp/test_dir"));
        assert!(!state.expanded_dirs.is_empty());

        state.collapse_all_folders();
        assert!(state.expanded_dirs.is_empty());
    }

    #[test]
    fn test_persistent_state_roundtrip() {
        let tmp_dir = std::env::temp_dir().join("zenthree_test_state");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let test_file = tmp_dir.join("test_file.txt");
        let _ = std::fs::write(&test_file, "hello persistence");

        let mut state = ZenthreeState::new();
        state.icon_theme = "catppuccin".to_string();
        state.explorer_preset = ExplorerPreset::ClassicTree;
        state.open_file(&test_file);

        assert_eq!(state.icon_theme, "catppuccin");
        assert_eq!(state.explorer_preset, ExplorerPreset::ClassicTree);
        assert!(state
            .tabs
            .iter()
            .any(|t| t.path.as_ref() == Some(&test_file)));

        // Clean up
        let _ = std::fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn test_ai_settings_persistence_and_defaults() {
        let mut state = ZenthreeState::new();
        // Check initial appearance defaults
        assert_eq!(state.ai_settings.preset_mode, AiPresetMode::SystemDefault);
        assert!(!state.ai_settings.effective_input_outline());
        assert!(state.ai_settings.effective_input_shadow());
        assert_eq!(state.ai_settings.effective_input_tone(), AiInputTone::Charcoal);
        assert_eq!(state.ai_settings.effective_message_bg(), AiMessageBg::DeepDark);
        assert_eq!(state.ai_settings.effective_text_size(), AiTextSize::Regular);
        assert_eq!(state.ai_settings.effective_text_size().pt(), 11.5);

        // Switch to custom preset mode and mutate settings
        state.ai_settings.preset_mode = AiPresetMode::Custom;
        state.ai_settings.input_outline = true;
        state.ai_settings.input_shadow = false;
        state.ai_settings.input_tone = AiInputTone::Bright;
        state.ai_settings.message_bg = AiMessageBg::Sidebar;
        state.ai_settings.text_size = AiTextSize::Large;
        assert_eq!(state.ai_settings.effective_text_size().pt(), 12.5);

        // Serialize to PersistentState JSON
        let persistent: PersistentState = (&state).into();
        let json = serde_json::to_string(&persistent).expect("serialize persistent state");
        let restored: PersistentState = serde_json::from_str(&json).expect("deserialize persistent state");

        let saved_ai = restored.ai_settings.expect("saved ai_settings");
        assert_eq!(saved_ai.preset_mode, AiPresetMode::Custom);
        assert!(saved_ai.input_outline);
        assert!(!saved_ai.input_shadow);
        assert_eq!(saved_ai.input_tone, AiInputTone::Bright);
        assert_eq!(saved_ai.message_bg, AiMessageBg::Sidebar);
        assert_eq!(saved_ai.text_size, AiTextSize::Large);
    }

    #[test]
    fn test_terminal_and_ai_panel_state() {
        let mut state = ZenthreeState::new();
        assert!(!state.terminal_visible);
        assert!(!state.ai_panel_visible);

        // Toggle terminal & send command
        state.terminal_visible = true;
        state.send_terminal_input("echo hello");
        assert!(state.terminal_session.is_some() || !state.terminal_lines.is_empty());

        // Toggle AI panel & send prompt
        assert!(!state.ai_panel_expanded);
        state.ai_panel_visible = true;
        state.ai_panel_expanded = true;
        assert!(state.ai_panel_expanded);
        state.send_ai_message("Explain the workspace");
        assert!(state
            .ai_messages
            .iter()
            .any(|m| m.is_user && m.content == "Explain the workspace"));
        assert!(state.ai_messages.iter().any(|m| !m.is_user));
    }

    #[test]
    fn test_language_detection_and_prompt_lifecycle() {
        let mut state = ZenthreeState::new();
        // Go is not installed initially
        assert!(!state.is_extension_installed("go"));

        // Opening a .go file triggers the language prompt
        let go_path = std::path::Path::new("/tmp/test_main.go");
        state.check_language_support_prompt(go_path);

        assert!(state.active_language_prompt.is_some());
        let prompt = state.active_language_prompt.as_ref().unwrap();
        assert_eq!(prompt.extension_id, "go");
        assert_eq!(prompt.language_name, "Go");
        assert!(!prompt.dont_show_again_checked);

        // User dismisses without checking "Don't ask again"
        state.dismiss_language_prompt(false);
        assert!(state.active_language_prompt.is_none());
        assert!(!state.ignored_language_prompts.contains("go"));

        // Since it wasn't ignored, opening again (e.g. next time IDE opens) triggers it again!
        state.check_language_support_prompt(go_path);
        assert!(state.active_language_prompt.is_some());

        // Now user checks "Don't ask again" and dismisses
        if let Some(ref mut p) = state.active_language_prompt {
            p.dont_show_again_checked = true;
        }
        state.dismiss_language_prompt(false);
        assert!(state.active_language_prompt.is_none());
        assert!(state.ignored_language_prompts.contains("go"));

        // Subsequent file open does NOT show prompt
        state.check_language_support_prompt(go_path);
        assert!(state.active_language_prompt.is_none());

        // Now install the extension
        state.install_extension("go");
        assert!(state.is_extension_installed("go"));

        // Uninstall works
        state.uninstall_extension("go");
        assert!(!state.is_extension_installed("go"));
    }

    #[test]
    fn test_extension_catalog_and_filtering() {
        let catalog = crate::extensions::ExtensionCatalog::all();
        assert!(!catalog.is_empty());

        let go_ext =
            crate::extensions::ExtensionCatalog::find_by_path(std::path::Path::new("app.go"));
        assert!(go_ext.is_some());
        assert_eq!(go_ext.unwrap().id, "go");

        let ts_ext =
            crate::extensions::ExtensionCatalog::find_by_path(std::path::Path::new("index.ts"));
        assert!(ts_ext.is_some());
        assert_eq!(ts_ext.unwrap().id, "typescript");

        let cpp_ext =
            crate::extensions::ExtensionCatalog::find_by_path(std::path::Path::new("main.cpp"));
        assert!(cpp_ext.is_some());
        assert_eq!(cpp_ext.unwrap().id, "cpp");
    }

    #[test]
    fn test_default_startup_empty_and_no_workspace() {
        let state = ZenthreeState::new();
        assert!(!state.has_workspace);
        assert!(state.sidebar_items.is_empty());
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.tabs[0].name, "Untitled-1");
        assert!(state.tabs[0].path.is_none());
        assert_eq!(state.tabs[0].editor.buffer.to_string(), "");
    }

    #[test]
    fn test_trajectory_exploration_summary() {
        let mut grp = ExplorationGroup {
            expanded: false,
            items: Vec::new(),
            is_active: false,
        };

        // 1. Single file
        grp.items.push(ExplorationItem::AnalyzedFile {
            path: "crates/zenthree/src/theme.rs".to_string(),
            start_line: Some(150),
            end_line: Some(300),
            symbol: None,
            is_peek: false,
        });
        assert_eq!(grp.summary_label(), "Explored 1 file");
        grp.is_active = true;
        assert_eq!(grp.summary_label(), "Exploring 1 file");
        grp.is_active = false;

        // 2. Add folders and searches
        for i in 0..12 {
            grp.items.push(ExplorationItem::AnalyzedFile {
                path: format!("file_{}.rs", i),
                start_line: None,
                end_line: None,
                symbol: None,
                is_peek: false,
            });
        }
        grp.items.push(ExplorationItem::ExploredDirectory { path: "src".into() });
        grp.items.push(ExplorationItem::ExploredDirectory { path: "crates".into() });
        for i in 0..11 {
            grp.items.push(ExplorationItem::Searched { query: format!("q{}", i), scope: None });
        }
        assert_eq!(grp.summary_label(), "Explored 13 files, 2 folders, 11 searches");
        grp.is_active = true;
        assert_eq!(grp.summary_label(), "Exploring 13 files, 2 folders, 11 searches");

        // 3. Thought only
        let mut thought_grp = ExplorationGroup {
            expanded: false,
            items: vec![ExplorationItem::Thought {
                summary: "Planning approach...".to_string(),
                duration_secs: Some(1),
            }],
            is_active: false,
        };
        assert_eq!(thought_grp.summary_label(), "Thought for 1s");
        thought_grp.is_active = true;
        assert_eq!(thought_grp.summary_label(), "Thinking...");
    }

    #[test]
    fn test_ai_message_trajectory_serialization_roundtrip() {
        let msg = AiMessage {
            is_user: false,
            content: "## Done\nFixed the bug.".to_string(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: vec![
                TrajectoryStep::Exploration(ExplorationGroup {
                    expanded: false,
                    items: vec![
                        ExplorationItem::AnalyzedFile {
                            path: "crates/zenthree/src/theme.rs".into(),
                            start_line: Some(1),
                            end_line: Some(50),
                            symbol: None,
                            is_peek: false,
                        }
                    ],
                    is_active: false,
                }),
                TrajectoryStep::FileEdit {
                    path: "crates/zenthree/src/theme.rs".into(),
                    lines_added: 32,
                    lines_removed: 1,
                },
                TrajectoryStep::Command {
                    command: "cargo check".into(),
                },
            ],
            is_working: false,
        };

        let json = serde_json::to_string(&msg).expect("serialization failed");
        let restored: AiMessage = serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(restored.trajectory.len(), 3);
        match &restored.trajectory[1] {
            TrajectoryStep::FileEdit { path, lines_added, lines_removed } => {
                assert_eq!(path, "crates/zenthree/src/theme.rs");
                assert_eq!(*lines_added, 32);
                assert_eq!(*lines_removed, 1);
            }
            _ => panic!("Expected FileEdit"),
        }
    }

    #[test]
    fn test_editor_tab_diff_line_tracking() {
        let mut state = ZenthreeState::default();
        state.tabs.clear();
        state.new_untitled_file();
        assert_eq!(state.tabs.len(), 1);
        assert_eq!(state.tabs[0].lines_added, 0);
        assert_eq!(state.tabs[0].lines_removed, 0);

        // Simulate file edit with grouped diff
        let diff = agent::context::GroupedDiff::from_replacement(
            "old line 1\nold line 2\nold line 3",
            "new line 1\nnew line 2\nnew line 3",
        );
        assert_eq!(diff.lines_removed, 3);
        assert_eq!(diff.lines_added, 3);
        assert_eq!(
            diff.formatted,
            "- old line 1\n- old line 2\n- old line 3\n+ new line 1\n+ new line 2\n+ new line 3\n"
        );

        state.tabs[0].lines_added += diff.lines_added;
        state.tabs[0].lines_removed += diff.lines_removed;

        assert_eq!(state.tabs[0].lines_added, 3);
        assert_eq!(state.tabs[0].lines_removed, 3);
    }

    #[test]
    fn test_file_diff_review_lifecycle() {
        let mut state = ZenthreeState::default();
        state.tabs.clear();

        let path1 = std::path::PathBuf::from("/tmp/test1.rs");
        let path2 = std::path::PathBuf::from("/tmp/test2.rs");

        // Set up tab 1 with original content
        state.tabs.push(crate::state::EditorTab {
            name: "test1.rs".to_string(),
            path: Some(path1.clone()),
            editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text("line 1\nnew edited line 2\nline 3")),
            is_dirty: false,
            version: 1,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            lines_added: 1,
            lines_removed: 1,
        });
        state.active_tab = Some(0);

        // Add pending review 1
        let (diff_lines1, _) = crate::state::compute_line_diff("line 1\nold line 2\nline 3", "line 1\nnew line 2\nline 3");
        state.pending_reviews.push(crate::state::FileDiffReview {
            path: path1.clone(),
            original_content: "line 1\nold line 2\nline 3".to_string(),
            proposed_content: "line 1\nnew line 2\nline 3".to_string(),
            diff_lines: diff_lines1,
            is_new_file: false,
            active_hunk_idx: 0,
        });

        // Add pending review 2 (new file)
        let (diff_lines2, _) = crate::state::compute_line_diff("", "line A\nline B");
        state.pending_reviews.push(crate::state::FileDiffReview {
            path: path2.clone(),
            original_content: String::new(),
            proposed_content: "line A\nline B".to_string(),
            diff_lines: diff_lines2,
            is_new_file: true,
            active_hunk_idx: 0,
        });
        state.active_review_idx = Some(0);

        assert_eq!(state.pending_reviews.len(), 2);
        assert_eq!(state.active_review().unwrap().path, path1);

        // Test review navigation
        state.next_review_file();
        assert_eq!(state.active_review_idx, Some(1));
        assert_eq!(state.active_review().unwrap().path, path2);

        state.prev_review_file();
        assert_eq!(state.active_review_idx, Some(0));
        assert_eq!(state.active_review().unwrap().path, path1);

        // Test Reject on review 1: restores original content
        state.reject_current_review();
        assert_eq!(state.tabs[0].editor.buffer.to_string(), "line 1\nold line 2\nline 3");
        assert_eq!(state.pending_reviews.len(), 1);
        assert_eq!(state.pending_reviews[0].path, path2);

        // Test Accept on remaining review
        state.accept_current_review();
        assert_eq!(state.pending_reviews.len(), 0);
        assert_eq!(state.active_review_idx, None);
    }

    #[test]
    fn test_accept_and_reject_all_reviews_and_path_rejection() {
        let mut state = ZenthreeState::default();
        state.tabs.clear();

        let path1 = std::path::PathBuf::from("/tmp/bulk1.rs");
        let path2 = std::path::PathBuf::from("/tmp/bulk2.rs");

        state.tabs.push(crate::state::EditorTab {
            name: "bulk1.rs".to_string(),
            path: Some(path1.clone()),
            editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text("new 1")),
            is_dirty: true,
            version: 1,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            lines_added: 1,
            lines_removed: 1,
        });
        state.tabs.push(crate::state::EditorTab {
            name: "bulk2.rs".to_string(),
            path: Some(path2.clone()),
            editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text("new 2")),
            is_dirty: true,
            version: 1,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            lines_added: 1,
            lines_removed: 1,
        });

        let (d1, _) = crate::state::compute_line_diff("orig 1", "new 1");
        state.pending_reviews.push(crate::state::FileDiffReview {
            path: path1.clone(),
            original_content: "orig 1".to_string(),
            proposed_content: "new 1".to_string(),
            diff_lines: d1,
            is_new_file: false,
            active_hunk_idx: 0,
        });

        let (d2, _) = crate::state::compute_line_diff("orig 2", "new 2");
        state.pending_reviews.push(crate::state::FileDiffReview {
            path: path2.clone(),
            original_content: "orig 2".to_string(),
            proposed_content: "new 2".to_string(),
            diff_lines: d2,
            is_new_file: false,
            active_hunk_idx: 0,
        });

        assert_eq!(state.pending_reviews.len(), 2);

        // Reject review for specific path (path1)
        state.reject_review_for_path(&path1);
        assert_eq!(state.pending_reviews.len(), 1);
        assert_eq!(state.tabs[0].editor.buffer.to_string(), "orig 1");
        assert_eq!(state.tabs[0].lines_added, 0);
        assert_eq!(state.tabs[0].lines_removed, 0);

        // Reject all remaining
        state.reject_all_reviews();
        assert_eq!(state.pending_reviews.len(), 0);
        assert_eq!(state.tabs[1].editor.buffer.to_string(), "orig 2");
        assert_eq!(state.tabs[1].lines_added, 0);
        assert_eq!(state.tabs[1].lines_removed, 0);
    }

    #[test]
    fn test_hunk_accept_reject_lifecycle() {
        let mut state = ZenthreeState::default();
        state.tabs.clear();
        let path = std::path::PathBuf::from("/tmp/test_hunks_file.rs");

        let orig = "line 1\nold line 2\nline 3\nold line 4\nline 5";
        let prop = "line 1\nnew line 2\nline 3\nnew line 4\nline 5";
        let (diff_lines, preview_text) = compute_line_diff(orig, prop);

        state.tabs.push(EditorTab {
            name: "test_hunks_file.rs".to_string(),
            path: Some(path.clone()),
            editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text(&preview_text)),
            is_dirty: true,
            version: 1,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            lines_added: 2,
            lines_removed: 2,
        });
        state.active_tab = Some(0);

        state.pending_reviews.push(crate::state::FileDiffReview {
            path: path.clone(),
            original_content: orig.to_string(),
            proposed_content: prop.to_string(),
            diff_lines,
            is_new_file: false,
            active_hunk_idx: 0,
        });
        state.active_review_idx = Some(0);

        assert_eq!(state.pending_reviews[0].total_removed_lines(), 2);
        assert_eq!(state.pending_reviews[0].total_added_lines(), 2);

        // Accept review
        state.accept_current_review();
        assert_eq!(state.pending_reviews.len(), 0);
        assert_eq!(state.active_review_idx, None);
        assert_eq!(state.tabs[0].lines_added, 0);
        assert_eq!(state.tabs[0].lines_removed, 0);
    }

    #[test]
    fn test_compute_line_diff_groups_consecutive_removals_before_additions() {
        let original = "fn test() {\n    let a = 1;\n    let b = 2;\n    let c = 3;\n    println!(\"done\");\n}";
        let proposed = "fn test() {\n    let x = 10;\n    let y = 20;\n    let z = 30;\n    println!(\"done\");\n}";

        let (diff, preview) = compute_line_diff(original, proposed);

        // Header: Unchanged "fn test() {"
        assert_eq!(diff[0], ("fn test() {".to_string(), DiffLineState::Unchanged));

        // 3 consecutive removed lines in sequence (red)
        assert_eq!(diff[1], ("    let a = 1;".to_string(), DiffLineState::Removed));
        assert_eq!(diff[2], ("    let b = 2;".to_string(), DiffLineState::Removed));
        assert_eq!(diff[3], ("    let c = 3;".to_string(), DiffLineState::Removed));

        // 3 consecutive added lines in sequence (green)
        assert_eq!(diff[4], ("    let x = 10;".to_string(), DiffLineState::Added));
        assert_eq!(diff[5], ("    let y = 20;".to_string(), DiffLineState::Added));
        assert_eq!(diff[6], ("    let z = 30;".to_string(), DiffLineState::Added));

        // Footer: Unchanged
        assert_eq!(diff[7], ("    println!(\"done\");".to_string(), DiffLineState::Unchanged));
        assert_eq!(diff[8], ("}".to_string(), DiffLineState::Unchanged));

        assert_eq!(preview.lines().count(), diff.len());
    }

    #[test]
    fn test_compute_line_diff_pure_addition_and_removal() {
        let original = "line 1\nline 2";
        let proposed = "line 1\ninserted\nline 2";
        let (diff, _) = compute_line_diff(original, proposed);
        assert_eq!(diff.len(), 3);
        assert_eq!(diff[0].1, DiffLineState::Unchanged);
        assert_eq!(diff[1], ("inserted".to_string(), DiffLineState::Added));
        assert_eq!(diff[2].1, DiffLineState::Unchanged);

        let original_del = "line 1\ndeleted 1\ndeleted 2\nline 2";
        let proposed_del = "line 1\nline 2";
        let (diff_del, _) = compute_line_diff(original_del, proposed_del);
        assert_eq!(diff_del.len(), 4);
        assert_eq!(diff_del[0].1, DiffLineState::Unchanged);
        assert_eq!(diff_del[1], ("deleted 1".to_string(), DiffLineState::Removed));
        assert_eq!(diff_del[2], ("deleted 2".to_string(), DiffLineState::Removed));
        assert_eq!(diff_del[3].1, DiffLineState::Unchanged);
    }

    #[test]
    fn test_compute_line_diff_keeps_gap_lines_unchanged() {
        let original = "fn main() {\n    let a = 1;\n    // middle comment\n    let b = 2;\n}";
        let proposed = "fn main() {\n    let a = 10;\n    // middle comment\n    let b = 20;\n}";
        let (diff, _) = compute_line_diff(original, proposed);
        // Prefix
        assert_eq!(diff[0].1, DiffLineState::Unchanged);
        // Hunk 1: a
        assert_eq!(diff[1], ("    let a = 1;".to_string(), DiffLineState::Removed));
        assert_eq!(diff[2], ("    let a = 10;".to_string(), DiffLineState::Added));
        // Gap: middle comment unchanged!
        assert_eq!(diff[3], ("    // middle comment".to_string(), DiffLineState::Unchanged));
        // Hunk 2: b
        assert_eq!(diff[4], ("    let b = 2;".to_string(), DiffLineState::Removed));
        assert_eq!(diff[5], ("    let b = 20;".to_string(), DiffLineState::Added));
        // Suffix
        assert_eq!(diff[6].1, DiffLineState::Unchanged);
    }

    #[test]
    fn test_already_open_tab_path_matching_and_review_sync() {
        use crate::state::types::path_matches;
        use std::path::Path;

        // Path matching test
        assert!(path_matches(Path::new("/workspace/src/lib.rs"), Path::new("/workspace/src/lib.rs")));
        assert!(path_matches(Path::new("/workspace/src/lib.rs"), Path::new("src/lib.rs")));
        assert!(path_matches(Path::new("/workspace/src/lib.rs"), Path::new("./src/lib.rs")));
        assert!(path_matches(Path::new("src/lib.rs"), Path::new("./src/lib.rs")));

        let mut state = ZenthreeState::default();
        state.tabs.clear();
        let path_a = std::path::PathBuf::from("/workspace/src/a.rs");
        let path_b = std::path::PathBuf::from("/workspace/src/b.rs");

        state.tabs.push(crate::state::EditorTab {
            name: "a.rs".to_string(),
            path: Some(path_a.clone()),
            editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text("fn a() {}")),
            is_dirty: false,
            version: 0,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            lines_added: 0,
            lines_removed: 0,
        });
        state.tabs.push(crate::state::EditorTab {
            name: "b.rs".to_string(),
            path: Some(path_b.clone()),
            editor: ::editor::editor::Editor::new(::editor::buffer::Buffer::from_text("fn b() {}")),
            is_dirty: false,
            version: 0,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            lines_added: 0,
            lines_removed: 0,
        });

        // Add reviews for both files (using relative paths as AI tools would report)
        state.pending_reviews.push(crate::state::FileDiffReview {
            path: std::path::PathBuf::from("src/a.rs"),
            original_content: "fn a() {}".to_string(),
            proposed_content: "fn a() { 1 }".to_string(),
            diff_lines: vec![("fn a() { 1 }".to_string(), DiffLineState::Added)],
            is_new_file: false,
            active_hunk_idx: 0,
        });
        state.pending_reviews.push(crate::state::FileDiffReview {
            path: std::path::PathBuf::from("./src/b.rs"),
            original_content: "fn b() {}".to_string(),
            proposed_content: "fn b() { 2 }".to_string(),
            diff_lines: vec![("fn b() { 2 }".to_string(), DiffLineState::Added)],
            is_new_file: false,
            active_hunk_idx: 0,
        });

        // Review lookup via review_idx_for_path
        assert_eq!(state.review_idx_for_path(&path_a), Some(0));
        assert_eq!(state.review_idx_for_path(&path_b), Some(1));

        // Syncing active review for tab 1
        state.sync_active_review_for_tab(1);
        assert_eq!(state.active_review_idx, Some(1));

        // Syncing active review for tab 0
        state.sync_active_review_for_tab(0);
        assert_eq!(state.active_review_idx, Some(0));
    }

    #[test]
    fn test_session_summary_relative_time() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let s1 = SessionSummary {
            id: "1".into(),
            title: "Test".into(),
            project_path: "/tmp/proj".into(),
            project_name: "proj".into(),
            created_at: now,
            updated_at: now - 10,
            message_count: 2,
        };
        assert_eq!(s1.relative_time_string(), "just now");

        let s2 = SessionSummary {
            updated_at: now - 300,
            ..s1.clone()
        };
        assert_eq!(s2.relative_time_string(), "5m");

        let s3 = SessionSummary {
            updated_at: now - 7200,
            ..s1.clone()
        };
        assert_eq!(s3.relative_time_string(), "2h");

        let s4 = SessionSummary {
            updated_at: now - 172800,
            ..s1.clone()
        };
        assert_eq!(s4.relative_time_string(), "2d");
    }

    #[test]
    fn test_open_workspace_switches_to_blank_session_and_preserves_general() {
        let mut state = ZenthreeState::default();
        assert!(!state.has_workspace);

        // 1. User is chatting in General session
        state.ai_messages.push(AiMessage {
            is_user: true,
            content: "General chat query".into(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });

        // 2. Open workspace
        let tmp = std::env::temp_dir().join(format!("zenthree_test_proj_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&tmp).unwrap();
        state.open_workspace(tmp.clone());
        assert!(state.has_workspace);

        // 3. Verify general session was cleanly saved under "general"
        assert!(state.ai_sessions.iter().any(|s| s.project_path == "general" && s.title == "General chat query"));

        // 4. Verify AI messages and session ID were cleared for a fresh blank session in the project
        assert!(state.ai_messages.is_empty(), "Opening workspace must start a fresh blank session");
        assert!(state.ai_session_id.is_none(), "Opening workspace must clear active session ID");
        assert!(!state.ai_sessions_visible, "Opening workspace must show the fresh start screen, not history drawer");

        // 5. Send message in project
        state.ai_messages.push(AiMessage {
            is_user: true,
            content: "Project chat query".into(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });
        state.save_current_ai_session();

        let canon = tmp.canonicalize().unwrap();
        let canon_str = canon.to_string_lossy().to_string();
        assert!(state.ai_sessions.iter().any(|s| s.project_path == canon_str && s.title == "Project chat query"));

        // Cleanup
        state.close_workspace();
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ai_session_save_and_start_new() {
        let mut state = ZenthreeState::default();
        // By default, no messages exist so start screen is displayed
        assert!(state.ai_messages.is_empty(), "AI messages must be empty by default for start screen");

        // Saving with no messages must NOT create or save any session
        state.save_current_ai_session();
        assert!(state.ai_session_id.is_none(), "Session ID must not be generated when no user message was sent");

        // Even with an assistant-only message, no session should be created
        state.ai_messages.push(AiMessage {
            is_user: false,
            content: "Welcome!".into(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });
        state.save_current_ai_session();
        assert!(state.ai_session_id.is_none(), "Session ID must not be generated when only assistant message exists");

        // When user sends a message, session is created
        state.ai_messages.push(AiMessage {
            is_user: true,
            content: "Hello AI session test".into(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });

        // Save session
        state.save_current_ai_session();
        let session_id = state.ai_session_id.clone();
        assert!(session_id.is_some(), "Session ID should be generated on save once user sends a message");

        // Start new session
        state.start_new_ai_session();
        assert!(state.ai_messages.is_empty(), "Starting new session must clear messages");
        assert!(state.ai_session_id.is_none(), "Starting new session must clear session ID");
        assert!(!state.ai_sessions_visible, "Starting new session must switch to chat view");

        // Cleanup
        if let Some(id) = session_id {
            state.delete_ai_session(&id);
        }
    }

    #[test]
    fn test_workspace_file_tree_auto_updates() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp = std::env::temp_dir().join(format!("zenthree_watch_test_{}", unique_id));
        std::fs::create_dir_all(&tmp).unwrap();

        let mut state = ZenthreeState::default();
        state.open_workspace(tmp.clone());

        assert!(state.sidebar_items.is_empty());

        // Create a new file externally (as another process or tool would)
        let test_file = tmp.join("created_by_other.txt");
        std::fs::write(&test_file, "hello external world").unwrap();

        // Allow debounce window (~200ms) to trigger
        let start = std::time::Instant::now();
        let mut detected = false;
        while start.elapsed() < std::time::Duration::from_millis(1500) {
            if state.poll_file_tree_changes() {
                detected = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        assert!(detected, "Watcher should automatically detect newly created file");
        assert!(
            state.sidebar_items.iter().any(|item| item.name == "created_by_other.txt"),
            "sidebar_items must contain the newly created file after auto-update"
        );

        // Remove the file externally
        std::fs::remove_file(&test_file).unwrap();

        let start2 = std::time::Instant::now();
        let mut removed_detected = false;
        while start2.elapsed() < std::time::Duration::from_millis(1500) {
            if state.poll_file_tree_changes() {
                removed_detected = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        assert!(removed_detected, "Watcher should automatically detect removed file");
        assert!(
            !state.sidebar_items.iter().any(|item| item.name == "created_by_other.txt"),
            "sidebar_items must not contain the removed file after auto-update"
        );

        state.close_workspace();
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_general_vs_workspace_ai_session_categorization() {
        let mut state = ZenthreeState::default();
        assert!(!state.has_workspace);

        // Add a message in no-workspace mode
        state.ai_messages.push(crate::state::AiMessage {
            is_user: true,
            content: "What is 2 + 2?".to_string(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });

        state.save_current_ai_session();
        let first_session = state.ai_sessions.first().expect("Session should be saved");
        assert_eq!(first_session.project_path, "general");
        assert_eq!(first_session.project_name, "General");

        // Now open a workspace and verify sessions are saved under that project
        let tmp = std::env::temp_dir().join(format!("zenthree_test_proj_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&tmp).unwrap();
        state.open_workspace(tmp.clone());
        assert!(state.has_workspace);

        state.ai_session_id = None; // start new session
        state.ai_messages.clear();
        state.ai_messages.push(crate::state::AiMessage {
            is_user: true,
            content: "Explain this project".to_string(),
            code_snippet: None,
            thoughts: None,
            expanded: false,
            trajectory: Vec::new(),
            is_working: false,
        });

        state.save_current_ai_session();
        let proj_session = state.ai_sessions.first().expect("Project session should be saved");
        let canon = tmp.canonicalize().unwrap();
        assert_eq!(proj_session.project_path, canon.to_string_lossy().to_string());

        // Clean up
        state.close_workspace();
        let _ = std::fs::remove_dir_all(&tmp);
    }
}



