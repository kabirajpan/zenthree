pub mod ai;
pub mod editor;
pub mod persistent;
pub mod terminal;
pub mod types;
pub mod workspace;

use crate::theme::{ThemeColors, ThemeMode};
use std::path::PathBuf;

#[allow(unused_imports)]
pub use persistent::{get_config_path, PersistentState};
pub use types::*;

pub struct ZenthreeState {
    pub theme: ThemeMode,
    pub accent_color: String,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub status_bar_visible: bool,
    pub line_numbers_visible: bool,
    pub zoom_level: f32,
    pub active_resize_sidebar: bool,
    pub tabs: Vec<EditorTab>,
    pub active_tab: Option<usize>,
    pub current_dir: PathBuf,
    pub has_workspace: bool,
    pub recent_files: Vec<PathBuf>,
    pub sidebar_items: Vec<FileTreeNode>,
    pub expanded_dirs: std::collections::HashSet<PathBuf>,
    pub status_message: String,
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
    /// Channel to receive status updates and final reply from the background agent thread.
    pub ai_response_rx: Option<std::sync::mpsc::Receiver<AiUiUpdate>>,
    pub copied_ai_msg: Option<(usize, std::time::Instant)>,
    pub copied_ai_code_id: Option<(u64, std::time::Instant)>,
    /// Render-side markdown parse cache: maps a FNV hash of message content → pre-parsed blocks.
    /// Stored as `Box<dyn Any>` so this module stays free of the `MarkdownBlock` type.
    pub ai_block_cache: std::collections::HashMap<u64, Box<dyn std::any::Any + Send + Sync>>,

    pub selected_ai_model: AiModel,
    pub ai_model_menu_open: bool,
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
            current_dir,
            has_workspace,
            recent_files: Vec::new(),
            sidebar_items: Vec::new(),
            expanded_dirs: std::collections::HashSet::new(),
            status_message: "Welcome to Zenthree!".to_string(),
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
            ai_panel_width: 340.0,
            active_resize_ai_panel: false,
            ai_messages: vec![AiMessage {
                is_user: false,
                content: "Hello! I am your assistant. How can I help you today?".to_string(),
                code_snippet: None,
                thoughts: None,
                expanded: false,
            }],
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
            ai_response_rx: None,
            copied_ai_msg: None,
            copied_ai_code_id: None,
            ai_block_cache: std::collections::HashMap::new(),
            selected_ai_model: AiModel::default(),
            ai_model_menu_open: false,
        };

        // Restore previous session preferences (theme, accent, extensions, etc.)
        #[cfg(not(test))]
        state.load_persistent_state();

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
        }
        state
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
        #[cfg(not(test))]
        self.save_persistent_state();
    }
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
    fn test_terminal_and_ai_panel_state() {
        let mut state = ZenthreeState::new();
        assert!(!state.terminal_visible);
        assert!(!state.ai_panel_visible);

        // Toggle terminal & send command
        state.terminal_visible = true;
        state.send_terminal_input("echo hello");
        assert!(state.terminal_session.is_some() || !state.terminal_lines.is_empty());

        // Toggle AI panel & send prompt
        state.ai_panel_visible = true;
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
}
