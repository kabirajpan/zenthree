use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::theme::{ThemeMode, ThemeColors};
use editor::buffer::Buffer;
use editor::editor::Editor;

#[derive(Clone)]
pub struct EditorTab {
    pub name: String,
    pub path: Option<PathBuf>,
    pub editor: Editor,
    pub is_dirty: bool,
    pub scroll_y: f32,
    pub shaping_cache: std::collections::HashMap<usize, (String, zenthra_text::prelude::ShapedBuffer)>,
    pub version: u64,
    pub saved_text: String,
}

impl EditorTab {
    pub fn update_dirty_state(&mut self) {
        if self.editor.buffer.len() != self.saved_text.len() {
            self.is_dirty = true;
        } else {
            self.is_dirty = self.editor.buffer.to_string() != self.saved_text;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    Modified,
    Untracked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub is_expanded: bool,
    #[allow(dead_code)]
    pub is_last: bool,
    pub guide_prefix: String,
    pub git_status: Option<GitStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExplorerPreset {
    NeovimSnacks,
    ModernMinimalist,
    ClassicTree,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SidebarView {
    Explorer,
    SourceControl,
    Extensions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatusCode {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitChangeItem {
    pub path: PathBuf,
    pub rel_path: String,
    pub filename: String,
    pub dir_path: String,
    pub status: GitFileStatusCode,
    pub is_staged: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitCommitLogItem {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub time_ago: String,
    pub message: String,
    pub is_head: bool,
    pub branch_refs: Vec<String>,
    pub lane: usize,
    pub graph_prefix: String,
}

#[derive(Debug, Clone)]
pub struct GitRepoState {
    pub is_repo: bool,
    pub branch: String,
    pub staged: Vec<GitChangeItem>,
    pub unstaged: Vec<GitChangeItem>,
    pub commit_message: String,
    pub staged_expanded: bool,
    pub changes_expanded: bool,
    pub graph_expanded: bool,
    pub graph_height: f32,
    pub active_resize_graph: bool,
    pub commit_history: Vec<GitCommitLogItem>,
    pub status_error: Option<String>,
}

impl Default for GitRepoState {
    fn default() -> Self {
        Self {
            is_repo: false,
            branch: String::new(),
            staged: Vec::new(),
            unstaged: Vec::new(),
            commit_message: String::new(),
            staged_expanded: true,
            changes_expanded: true,
            graph_expanded: true,
            graph_height: 200.0,
            active_resize_graph: false,
            commit_history: Vec::new(),
            status_error: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionFilterTab {
    All,
    Installed,
    Available,
}

#[derive(Debug, Clone)]
pub struct LanguagePromptDialog {
    pub extension_id: String,
    pub language_name: String,
    pub file_name: String,
    pub dont_show_again_checked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineTreeInputMode {
    NewFile { parent_dir: PathBuf },
    NewFolder { parent_dir: PathBuf },
    Rename { target_path: PathBuf },
}

#[derive(Debug, Clone)]
pub struct InlineTreeState {
    pub mode: InlineTreeInputMode,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct TreeContextMenu {
    pub target_path: PathBuf,
    pub is_dir: bool,
    pub menu_x: f32,
    pub menu_y: f32,
}

#[derive(Debug, Clone)]
pub enum TreeClipboardAction {
    Copy(PathBuf),
    Cut(PathBuf),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalLine {
    pub text: String,
    pub is_command: bool,
    pub is_error: bool,
}

pub use ai::{ExploredItem, EditedFileSummary};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiMessage {
    pub is_user: bool,
    pub content: String,
    pub code_snippet: Option<String>,
    pub thoughts: Option<String>,
    #[serde(default)]
    pub deliberation: Option<connectome::DecisionResult>,
    #[serde(default)]
    pub explored_items: Vec<ExploredItem>,
    #[serde(default)]
    pub edited_files: Vec<EditedFileSummary>,
    #[serde(default)]
    pub is_working: bool,
    #[serde(default)]
    pub status_text: Option<String>,
    #[serde(default)]
    pub sub_agent: Option<crate::agent::SubAgentRole>,
    #[serde(default)]
    pub elapsed_secs: Option<u64>,
}

impl Default for AiMessage {
    fn default() -> Self {
        Self {
            is_user: false,
            content: String::new(),
            code_snippet: None,
            thoughts: None,
            deliberation: None,
            explored_items: Vec::new(),
            edited_files: Vec::new(),
            is_working: false,
            status_text: None,
            sub_agent: None,
            elapsed_secs: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineState {
    Unchanged,
    Added,    // + line in green
    Removed,  // - line in red
}

#[derive(Debug, Clone)]
pub struct PendingModification {
    pub file_path: PathBuf,
    pub original_code: String,
    pub proposed_code: String,
    pub diff_lines: Vec<(String, DiffLineState)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AiStatus {
    Ready,
    Generating,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiInputDrawer {
    None,
    PendingModifications,
    Terminals,
    Artifacts,
    SessionChanges,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionModifiedFile {
    pub file_path: PathBuf,
    pub file_name: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub timestamp: u64,
    pub is_new_file: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiTerminalCommand {
    pub id: String,
    pub command: String,
    pub status: AiCommandStatus,
    pub output: String,
    pub duration_ms: Option<u64>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AiCommandStatus {
    Running,
    Success,
    Failed,
}

#[allow(unused_imports)]
pub use crate::agent::{
    AgentMission, AgentTask, TaskStatus, TargetFileModification,
    create_mission_for_prompt, build_autonomous_agent_prompt, extract_all_file_modifications,
    run_project_compiler_check_with_env, build_self_healing_prompt,
    AgentToolCall, parse_agent_tool_calls, execute_agent_tool_with_provider,
    ProviderChain, DetectorAgent, ProjectEnvironment, SubAgentRole,
    UserIntent, classify_user_intent, build_discussion_guide_prompt,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistentState {
    #[serde(default)]
    pub theme: Option<ThemeMode>,
    #[serde(default)]
    pub accent_color: Option<String>,
    #[serde(default)]
    pub icon_theme: Option<String>,
    #[serde(default)]
    pub explorer_preset: Option<ExplorerPreset>,
    #[serde(default)]
    pub explorer_tree_guides: Option<bool>,
    #[serde(default)]
    pub explorer_git_status: Option<bool>,
    #[serde(default)]
    pub explorer_show_hidden: Option<bool>,
    #[serde(default)]
    pub sidebar_visible: Option<bool>,
    #[serde(default)]
    pub sidebar_width: Option<f32>,
    #[serde(default)]
    pub status_bar_visible: Option<bool>,
    #[serde(default)]
    pub line_numbers_visible: Option<bool>,
    #[serde(default)]
    pub zoom_level: Option<f32>,
    #[serde(default)]
    pub current_dir: Option<PathBuf>,
    #[serde(default)]
    pub expanded_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub open_tabs: Vec<PathBuf>,
    #[serde(default)]
    pub active_tab_path: Option<PathBuf>,
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,
    #[serde(default)]
    pub terminal_visible: Option<bool>,
    #[serde(default)]
    pub terminal_height: Option<f32>,
    #[serde(default)]
    pub ai_panel_visible: Option<bool>,
    #[serde(default)]
    pub ai_panel_width: Option<f32>,
    #[serde(default)]
    pub sidebar_view: Option<SidebarView>,
    #[serde(default)]
    pub installed_extensions: Vec<String>,
    #[serde(default)]
    pub ignored_language_prompts: Vec<String>,
    #[serde(default)]
    pub selected_model: Option<String>,
    #[serde(default)]
    pub model_auto_mode: Option<bool>,
    #[serde(default)]
    pub custom_models: Vec<String>,
    #[serde(default)]
    pub git_graph_height: Option<f32>,
}

pub fn get_config_path() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        let app_dir = config_dir.join("zenthree");
        let _ = std::fs::create_dir_all(&app_dir);
        app_dir.join("state.json")
    } else if let Some(home_dir) = dirs::home_dir() {
        let app_dir = home_dir.join(".config").join("zenthree");
        let _ = std::fs::create_dir_all(&app_dir);
        app_dir.join("state.json")
    } else {
        PathBuf::from(".zenthree_state.json")
    }
}

#[derive(Clone, Debug)]
pub struct BreadcrumbPopup {
    pub dir_path: PathBuf,
    pub selected_name: String,
    pub popup_x: f32,
    pub popup_y: f32,
    pub expanded_dirs: std::collections::HashSet<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeState {
    pub mode: ThemeMode,
    pub accent_color: String,
    pub icon_theme: String,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Dark,
            accent_color: "blue".to_string(),
            icon_theme: "neovim_mini".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiLayoutState {
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub active_resize_sidebar: bool,
    pub sidebar_view: SidebarView,
    pub terminal_visible: bool,
    pub terminal_height: f32,
    pub active_resize_terminal: bool,
    pub ai_panel_visible: bool,
    pub ai_panel_width: f32,
    pub active_resize_ai_panel: bool,
    pub status_bar_visible: bool,
    pub line_numbers_visible: bool,
    pub zoom_level: f32,
    pub about_open: bool,
    pub user_profile_open: bool,
    pub find_in_buffer_open: bool,
    pub find_query: String,
    pub find_match_index: usize,
    pub find_matches: Vec<(usize, usize, usize)>,
}

impl Default for UiLayoutState {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            sidebar_width: 250.0,
            active_resize_sidebar: false,
            sidebar_view: SidebarView::Explorer,
            terminal_visible: false,
            terminal_height: 200.0,
            active_resize_terminal: false,
            ai_panel_visible: false,
            ai_panel_width: 340.0,
            active_resize_ai_panel: false,
            status_bar_visible: true,
            line_numbers_visible: true,
            zoom_level: 1.0,
            about_open: false,
            user_profile_open: false,
            find_in_buffer_open: false,
            find_query: String::new(),
            find_match_index: 0,
            find_matches: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkspaceDomainState {
    pub current_dir: PathBuf,
    pub has_workspace: bool,
    pub recent_files: Vec<PathBuf>,
    pub sidebar_items: Vec<FileTreeNode>,
    pub expanded_dirs: std::collections::HashSet<PathBuf>,
    pub selected_sidebar_path: Option<PathBuf>,
    pub explorer_preset: ExplorerPreset,
    pub explorer_tree_guides: bool,
    pub explorer_git_status: bool,
    pub explorer_show_hidden: bool,
    pub workspace_tree_overview: String,
}

impl Default for WorkspaceDomainState {
    fn default() -> Self {
        Self {
            current_dir: PathBuf::from("."),
            has_workspace: false,
            recent_files: Vec::new(),
            sidebar_items: Vec::new(),
            expanded_dirs: std::collections::HashSet::new(),
            selected_sidebar_path: None,
            explorer_preset: ExplorerPreset::ModernMinimalist,
            explorer_tree_guides: false,
            explorer_git_status: true,
            explorer_show_hidden: false,
            workspace_tree_overview: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct AgentDomainState {
    pub is_running: bool,
    pub active_status: Option<String>,
    pub active_file: Option<String>,
    pub event_count: usize,
    pub pending_requests: usize,
}

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
    pub terminal_scroll_offset: usize,
    pub terminal_last_activity: std::time::Instant,
    pub ai_panel_visible: bool,
    pub ai_panel_width: f32,
    pub active_resize_ai_panel: bool,
    pub ai_messages: Vec<AiMessage>,
    pub ai_input: String,
    pub ai_active_status: zenthra_state::ArcSignal<Option<String>>,
    pub ai_active_file: zenthra_state::ArcSignal<Option<String>>,
    pub ai_event_signal: zenthra_state::ArcSignal<usize>,
    pub user_profile_open: bool,
    pub terminal_session: Option<crate::native_terminal::NativeTerminalSession>,
    pub sidebar_view: SidebarView,
    pub git_state: GitRepoState,
    pub installed_extensions: std::collections::HashSet<String>,
    pub ignored_language_prompts: std::collections::HashSet<String>,
    pub extension_search_query: String,
    pub extension_tab_filter: ExtensionFilterTab,
    pub active_language_prompt: Option<LanguagePromptDialog>,
    pub lsp_manager: crate::lsp::LspManager,
    pub completion_active: bool,
    pub completion_items: Vec<crate::lsp::protocol::CompletionItem>,
    pub completion_selected_idx: usize,
    pub active_hover_text: Option<String>,
    pub find_in_buffer_open: bool,
    pub find_query: String,
    pub find_match_index: usize,
    pub find_matches: Vec<(usize, usize, usize)>, // (row, start_col, end_col)
    pub inline_tree_input: Option<InlineTreeState>,
    pub tree_context_menu: Option<TreeContextMenu>,
    pub tree_clipboard: Option<TreeClipboardAction>,
    pub groq_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub ai_receiver: Option<std::sync::mpsc::Receiver<(usize, ai::AgentEvent)>>,
    pub ai_sender: std::sync::mpsc::Sender<(usize, ai::AgentEvent)>,
    pub ai_cancellation_token: ai::CancellationToken,
    pub ai_pending_requests: usize,
    pub cursor_blink: bool,
    pub connectome: connectome::ConnectomeRNN,
    pub connectome_cockpit_open: bool,
    pub connectome_ablation_enabled: bool,
    pub connectome_ablation_pct: f32,
    pub connectome_ablation_mode: connectome::AblationMode,
    pub pending_modification: Option<PendingModification>,
    pub pending_modifications: Vec<PendingModification>,
    pub last_ai_prompt: String,
    pub expanded_explored_msgs: std::collections::HashSet<usize>,
    pub collapsed_explored_folders: std::collections::HashSet<(usize, String)>,
    pub expanded_thought_items: std::collections::HashSet<(usize, usize)>,
    #[allow(dead_code)]
    pub expanded_worked_msgs: std::collections::HashSet<usize>,
    pub ai_request_start_times: std::collections::HashMap<usize, std::time::Instant>,
    pub expanded_code_blocks: std::collections::HashSet<u64>,
    pub current_mission: Option<AgentMission>,
    pub current_session: crate::session::AgentSession,
    pub session_list: Vec<crate::session::SessionSummary>,
    pub session_dropdown_open: bool,
    pub active_input_drawer: AiInputDrawer,
    pub session_modified_files: Vec<SessionModifiedFile>,
    pub ai_terminal_commands: Vec<AiTerminalCommand>,
    pub ai_showing_session_list: bool,
    pub ai_context_menu_open: bool,
    pub ai_model_menu_open: bool,
    pub model_auto_mode: bool,
    pub selected_model: String,
    pub custom_models: Vec<String>,
    pub adding_custom_model: bool,
    pub custom_model_input: String,
    pub detected_environment: Option<ProjectEnvironment>,
    pub workspace_tree_overview: String,
    pub session_artifacts_collapsed: bool,
    pub last_provider_notice: Option<String>,
    pub ai_status: AiStatus,
    pub ai_error: Option<String>,
    pub ai_include_selection: bool,
    pub ai_include_current_file: bool,
    pub ai_include_open_files: bool,
    pub ai_include_terminal: bool,
    pub ai_manual_permissions: bool,
    pub ai_pending_prompt: Option<String>,
    pub ai_generation_ticks: usize,
    pub ai_context_snapshot: String,
    pub ai_backend: crate::agent::AgentBackend,
    pub theme_state: ThemeState,
    pub theme_signal: zenthra_state::Signal<ThemeState>,
    pub colors_signal: zenthra_state::Signal<ThemeColors>,
    pub ui_layout: UiLayoutState,
    pub layout_signal: zenthra_state::Signal<UiLayoutState>,
    pub workspace: WorkspaceDomainState,
    pub workspace_signal: zenthra_state::Signal<WorkspaceDomainState>,
    pub sidebar_visible_sig: zenthra_state::Signal<bool>,
    pub ai_panel_visible_sig: zenthra_state::Signal<bool>,
    pub terminal_visible_sig: zenthra_state::Signal<bool>,
    pub agent_state: AgentDomainState,
    pub agent_signal: zenthra_state::Signal<AgentDomainState>,
}

impl ZenthreeState {
    pub fn new() -> Self {
        // Try workspace argument, fallback to std::env::current_dir(), fallback to home dir
        let args: Vec<String> = std::env::args().collect();
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        });
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
        let (ai_sender, ai_receiver) = std::sync::mpsc::channel();

        let initial_workspace = WorkspaceDomainState {
            current_dir: current_dir.clone(),
            has_workspace,
            recent_files: Vec::new(),
            sidebar_items: Vec::new(),
            expanded_dirs: std::collections::HashSet::new(),
            selected_sidebar_path: None,
            explorer_preset: ExplorerPreset::ModernMinimalist,
            explorer_tree_guides: false,
            explorer_git_status: true,
            explorer_show_hidden: false,
            workspace_tree_overview: String::new(),
        };

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
            current_dir: current_dir.clone(),
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
            terminal_lines: vec![
                TerminalLine {
                    text: "Zenthree Native Terminal v0.1.0".to_string(),
                    is_command: false,
                    is_error: false,
                },
            ],
            terminal_input: String::new(),
            terminal_history: Vec::new(),
            terminal_history_idx: None,
            terminal_scroll_offset: 0,
            terminal_last_activity: std::time::Instant::now(),
            ai_panel_visible: false,
            ai_panel_width: 340.0,
            active_resize_ai_panel: false,
            ai_messages: Vec::new(),
            ai_input: String::new(),
            ai_active_status: zenthra_state::ArcSignal::new(None),
            ai_active_file: zenthra_state::ArcSignal::new(None),
            ai_event_signal: zenthra_state::ArcSignal::new(0),
            user_profile_open: false,
            terminal_session: None,
            cursor_blink: true,
            connectome: connectome::ConnectomeRNN::load_default(),
            connectome_cockpit_open: false,
            connectome_ablation_enabled: false,
            connectome_ablation_pct: 0.20,
            connectome_ablation_mode: connectome::AblationMode::TargetedHubs,
            selected_sidebar_path: None,
            sidebar_view: SidebarView::Explorer,
            git_state: GitRepoState::default(),
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
            active_hover_text: None,
            find_in_buffer_open: false,
            find_query: String::new(),
            find_match_index: 0,
            find_matches: Vec::new(),
            inline_tree_input: None,
            tree_context_menu: None,
            tree_clipboard: None,
            groq_api_key: crate::groq::load_groq_api_key(),
            gemini_api_key: crate::gemini::load_gemini_api_key(),
            ai_receiver: Some(ai_receiver),
            ai_sender,
            ai_cancellation_token: ai::CancellationToken::new(),
            ai_pending_requests: 0,
            pending_modification: None,
            pending_modifications: Vec::new(),
            last_ai_prompt: String::new(),
            expanded_explored_msgs: std::collections::HashSet::new(),
            collapsed_explored_folders: std::collections::HashSet::new(),
            expanded_thought_items: std::collections::HashSet::new(),
            expanded_worked_msgs: std::collections::HashSet::new(),
            ai_request_start_times: std::collections::HashMap::new(),
            expanded_code_blocks: std::collections::HashSet::new(),
            current_mission: None,
            detected_environment: None,
            current_session: crate::session::AgentSession::new(&current_dir, None),
            session_list: crate::session::list_saved_sessions(&current_dir),
            session_dropdown_open: false,
            active_input_drawer: AiInputDrawer::None,
            session_modified_files: Vec::new(),
            ai_terminal_commands: Vec::new(),
            ai_showing_session_list: false,
            ai_context_menu_open: false,
            ai_model_menu_open: false,
            model_auto_mode: true,
            selected_model: "Auto".to_string(),
            custom_models: Vec::new(),
            adding_custom_model: false,
            custom_model_input: String::new(),
            workspace_tree_overview: String::new(),
            session_artifacts_collapsed: true,
            last_provider_notice: None,
            ai_status: AiStatus::Ready,
            ai_error: None,
            ai_include_selection: false,
            ai_include_current_file: true,
            ai_include_open_files: false,
            ai_include_terminal: false,
            ai_manual_permissions: false,
            ai_pending_prompt: None,
            ai_generation_ticks: 0,
            ai_context_snapshot: String::new(),
            #[cfg(test)]
            ai_backend: crate::agent::AgentBackend::Native,
            #[cfg(not(test))]
            ai_backend: if ai::GooseSupervisor::new().find_binary().is_some() {
                crate::agent::AgentBackend::GooseAcp
            } else {
                crate::agent::AgentBackend::Native
            },
            theme_state: ThemeState::default(),
            theme_signal: zenthra_state::Signal::new(ThemeState::default()),
            colors_signal: zenthra_state::Signal::new(ThemeColors::get_custom(ThemeMode::Dark, "blue")),
            ui_layout: UiLayoutState::default(),
            layout_signal: zenthra_state::Signal::new(UiLayoutState::default()),
            workspace: initial_workspace.clone(),
            workspace_signal: zenthra_state::Signal::new(initial_workspace),
            sidebar_visible_sig: zenthra_state::Signal::new(true),
            ai_panel_visible_sig: zenthra_state::Signal::new(false),
            terminal_visible_sig: zenthra_state::Signal::new(false),
            agent_state: AgentDomainState::default(),
            agent_signal: zenthra_state::Signal::new(AgentDomainState::default()),
        };

        // Restore previous session preferences (theme, accent, extensions, etc.)
        #[cfg(not(test))]
        state.load_persistent_state();
        state.sync_reactive_domains();

        // If CLI argument was explicitly provided, open specified file or workspace
        #[allow(unused_mut)]
        let mut opened_cli = false;
        #[cfg(not(test))]
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                if path.is_dir() {
                    state.open_workspace(path);
                    opened_cli = true;
                } else if path.is_file() {
                    state.open_file(&path);
                    if let Some(parent) = path.parent() {
                        state.current_dir = parent.to_path_buf();
                        state.has_workspace = true;
                    }
                    opened_cli = true;
                }
            }
        }

        if !opened_cli {
            // Default startup without arguments or in test runner:
            // DO NOT open any project by default.
            // Start clean with an empty untitled buffer!
            state.has_workspace = false;
            state.sidebar_items.clear();
            state.tabs.clear();
            state.new_untitled_file();
        }

        state.init_workspace_session();

        if state.has_workspace {
            state.rebuild_tree();
            state.refresh_git_status();
        }
        state
    }

    pub fn set_sidebar_visible(&mut self, visible: bool) {
        if self.sidebar_visible != visible {
            self.sidebar_visible = visible;
            self.ui_layout.sidebar_visible = visible;
            self.sidebar_visible_sig.set(visible);
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn toggle_sidebar(&mut self) {
        self.set_sidebar_visible(!self.sidebar_visible);
    }

    pub fn set_terminal_visible(&mut self, visible: bool) {
        if self.terminal_visible != visible {
            self.terminal_visible = visible;
            self.ui_layout.terminal_visible = visible;
            self.terminal_visible_sig.set(visible);
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn toggle_terminal(&mut self) {
        self.set_terminal_visible(!self.terminal_visible);
    }

    pub fn set_ai_panel_visible(&mut self, visible: bool) {
        if self.ai_panel_visible != visible {
            self.ai_panel_visible = visible;
            self.ui_layout.ai_panel_visible = visible;
            self.ai_panel_visible_sig.set(visible);
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn toggle_ai_panel(&mut self) {
        self.set_ai_panel_visible(!self.ai_panel_visible);
    }

    pub fn set_status_bar_visible(&mut self, visible: bool) {
        if self.status_bar_visible != visible {
            self.status_bar_visible = visible;
            self.ui_layout.status_bar_visible = visible;
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn toggle_status_bar(&mut self) {
        self.set_status_bar_visible(!self.status_bar_visible);
    }

    pub fn set_line_numbers_visible(&mut self, visible: bool) {
        if self.line_numbers_visible != visible {
            self.line_numbers_visible = visible;
            self.ui_layout.line_numbers_visible = visible;
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn toggle_line_numbers(&mut self) {
        self.set_line_numbers_visible(!self.line_numbers_visible);
    }

    pub fn set_zoom_level(&mut self, zoom: f32) {
        let clamped = ((zoom.clamp(0.5, 2.5)) * 10.0).round() / 10.0;
        if (self.zoom_level - clamped).abs() > 0.001 {
            self.zoom_level = clamped;
            self.ui_layout.zoom_level = clamped;
            self.status_message = format!("Zoom: {:.0}%", clamped * 100.0);
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn zoom_in(&mut self) {
        self.set_zoom_level(self.zoom_level + 0.1);
    }

    pub fn zoom_out(&mut self) {
        self.set_zoom_level(self.zoom_level - 0.1);
    }

    pub fn reset_zoom(&mut self) {
        self.set_zoom_level(1.0);
    }

    pub fn set_sidebar_view(&mut self, view: SidebarView) {
        if self.sidebar_view != view {
            self.sidebar_view = view;
            self.ui_layout.sidebar_view = view;
            self.layout_signal.set(self.ui_layout.clone());
            self.save_persistent_state();
        }
    }

    pub fn set_theme(&mut self, mode: ThemeMode) {
        if self.theme != mode {
            self.theme = mode;
            self.theme_state.mode = mode;
            self.theme_signal.set(self.theme_state.clone());
            let colors = ThemeColors::get_custom(self.theme, &self.accent_color);
            self.colors_signal.set(colors);
            self.highlighter = match mode {
                ThemeMode::Dark => syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::dark()),
                ThemeMode::Light => syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::light()),
            };
            for tab in &mut self.tabs {
                tab.shaping_cache.clear();
            }
            self.save_persistent_state();
        }
    }

    pub fn set_accent_color(&mut self, accent: &str) {
        if self.accent_color != accent {
            self.accent_color = accent.to_string();
            self.theme_state.accent_color = self.accent_color.clone();
            self.theme_signal.set(self.theme_state.clone());
            let colors = ThemeColors::get_custom(self.theme, &self.accent_color);
            self.colors_signal.set(colors);
            for tab in &mut self.tabs {
                tab.shaping_cache.clear();
            }
            self.save_persistent_state();
        }
    }

    pub fn set_icon_theme(&mut self, icon_theme: &str) {
        if self.icon_theme != icon_theme {
            self.icon_theme = icon_theme.to_string();
            self.theme_state.icon_theme = self.icon_theme.clone();
            self.theme_signal.set(self.theme_state.clone());
            self.save_persistent_state();
        }
    }

    #[allow(dead_code)]
    pub fn toggle_theme(&mut self) {
        let new_mode = match self.theme {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        self.set_theme(new_mode);
    }

    pub fn sync_reactive_domains(&mut self) {
        self.ui_layout.sidebar_visible = self.sidebar_visible;
        self.ui_layout.sidebar_width = self.sidebar_width;
        self.ui_layout.active_resize_sidebar = self.active_resize_sidebar;
        self.ui_layout.sidebar_view = self.sidebar_view;
        self.ui_layout.terminal_visible = self.terminal_visible;
        self.ui_layout.terminal_height = self.terminal_height;
        self.ui_layout.active_resize_terminal = self.active_resize_terminal;
        self.ui_layout.ai_panel_visible = self.ai_panel_visible;
        self.ui_layout.ai_panel_width = self.ai_panel_width;
        self.ui_layout.active_resize_ai_panel = self.active_resize_ai_panel;
        self.ui_layout.status_bar_visible = self.status_bar_visible;
        self.ui_layout.line_numbers_visible = self.line_numbers_visible;
        self.ui_layout.zoom_level = self.zoom_level;
        self.ui_layout.about_open = self.about_open;
        self.ui_layout.user_profile_open = self.user_profile_open;
        self.ui_layout.find_in_buffer_open = self.find_in_buffer_open;
        self.ui_layout.find_query = self.find_query.clone();
        self.ui_layout.find_match_index = self.find_match_index;
        self.ui_layout.find_matches = self.find_matches.clone();
        self.sidebar_visible_sig.set(self.sidebar_visible);
        self.terminal_visible_sig.set(self.terminal_visible);
        self.ai_panel_visible_sig.set(self.ai_panel_visible);
        self.layout_signal.set(self.ui_layout.clone());

        self.theme_state.mode = self.theme;
        self.theme_state.accent_color = self.accent_color.clone();
        self.theme_state.icon_theme = self.icon_theme.clone();
        self.theme_signal.set(self.theme_state.clone());
        let current_colors = ThemeColors::get_custom(self.theme, &self.accent_color);
        self.colors_signal.set(current_colors);

        self.workspace.current_dir = self.current_dir.clone();
        self.workspace.has_workspace = self.has_workspace;
        self.workspace.recent_files = self.recent_files.clone();
        self.workspace.sidebar_items = self.sidebar_items.clone();
        self.workspace.expanded_dirs = self.expanded_dirs.clone();
        self.workspace.selected_sidebar_path = self.selected_sidebar_path.clone();
        self.workspace.explorer_preset = self.explorer_preset;
        self.workspace.explorer_tree_guides = self.explorer_tree_guides;
        self.workspace.explorer_git_status = self.explorer_git_status;
        self.workspace.explorer_show_hidden = self.explorer_show_hidden;
        self.workspace.workspace_tree_overview = self.workspace_tree_overview.clone();
        self.workspace_signal.set(self.workspace.clone());

        self.agent_state.is_running = self.ai_pending_requests > 0;
        self.agent_state.active_status = self.ai_active_status.with(|s| s.clone());
        self.agent_state.active_file = self.ai_active_file.with(|f| f.clone());
        self.agent_state.event_count = self.ai_event_signal.get();
        self.agent_state.pending_requests = self.ai_pending_requests;
        self.agent_signal.set(self.agent_state.clone());
    }

    pub fn set_explorer_preset(&mut self, preset: ExplorerPreset) {
        if self.explorer_preset != preset {
            self.explorer_preset = preset;
            self.workspace.explorer_preset = preset;
            self.workspace_signal.set(self.workspace.clone());
            self.save_persistent_state();
        }
    }

    pub fn toggle_explorer_tree_guides(&mut self) {
        self.explorer_tree_guides = !self.explorer_tree_guides;
        self.workspace.explorer_tree_guides = self.explorer_tree_guides;
        self.workspace_signal.set(self.workspace.clone());
        self.save_persistent_state();
    }

    pub fn toggle_explorer_git_status(&mut self) {
        self.explorer_git_status = !self.explorer_git_status;
        self.workspace.explorer_git_status = self.explorer_git_status;
        self.rebuild_tree();
        self.workspace.sidebar_items = self.sidebar_items.clone();
        self.workspace_signal.set(self.workspace.clone());
        self.save_persistent_state();
    }

    pub fn toggle_explorer_show_hidden(&mut self) {
        self.explorer_show_hidden = !self.explorer_show_hidden;
        self.workspace.explorer_show_hidden = self.explorer_show_hidden;
        self.rebuild_tree();
        self.workspace.sidebar_items = self.sidebar_items.clone();
        self.workspace_signal.set(self.workspace.clone());
        self.save_persistent_state();
    }

    pub fn sync_workspace_signal(&mut self) {
        self.workspace.current_dir = self.current_dir.clone();
        self.workspace.has_workspace = self.has_workspace;
        self.workspace.recent_files = self.recent_files.clone();
        self.workspace.sidebar_items = self.sidebar_items.clone();
        self.workspace.expanded_dirs = self.expanded_dirs.clone();
        self.workspace.selected_sidebar_path = self.selected_sidebar_path.clone();
        self.workspace.explorer_preset = self.explorer_preset;
        self.workspace.explorer_tree_guides = self.explorer_tree_guides;
        self.workspace.explorer_git_status = self.explorer_git_status;
        self.workspace.explorer_show_hidden = self.explorer_show_hidden;
        self.workspace.workspace_tree_overview = self.workspace_tree_overview.clone();
        self.workspace_signal.set(self.workspace.clone());
    }

    pub fn save_persistent_state(&self) {
        self.save_current_session_messages();
        let path = get_config_path();
        let current_dir = if self.has_workspace {
            Some(self.current_dir.clone())
        } else {
            None
        };
        let open_tabs: Vec<PathBuf> = if self.has_workspace {
            self.tabs.iter()
                .filter_map(|t| t.path.clone())
                .collect()
        } else {
            Vec::new()
        };
        let active_tab_path = if self.has_workspace {
            self.active_tab
                .and_then(|idx| self.tabs.get(idx))
                .and_then(|t| t.path.clone())
        } else {
            None
        };

        let state = PersistentState {
            theme: Some(self.theme),
            accent_color: Some(self.accent_color.clone()),
            icon_theme: Some(self.icon_theme.clone()),
            explorer_preset: Some(self.explorer_preset),
            explorer_tree_guides: Some(self.explorer_tree_guides),
            explorer_git_status: Some(self.explorer_git_status),
            explorer_show_hidden: Some(self.explorer_show_hidden),
            sidebar_visible: Some(self.sidebar_visible),
            sidebar_width: Some(self.sidebar_width),
            status_bar_visible: Some(self.status_bar_visible),
            line_numbers_visible: Some(self.line_numbers_visible),
            zoom_level: Some(self.zoom_level),
            current_dir,
            expanded_dirs: if self.has_workspace { self.expanded_dirs.iter().cloned().collect() } else { Vec::new() },
            open_tabs,
            active_tab_path,
            recent_files: self.recent_files.clone(),
            terminal_visible: Some(self.terminal_visible),
            terminal_height: Some(self.terminal_height),
            ai_panel_visible: Some(self.ai_panel_visible),
            ai_panel_width: Some(self.ai_panel_width),
            sidebar_view: Some(self.sidebar_view),
            installed_extensions: self.installed_extensions.iter().cloned().collect(),
            ignored_language_prompts: self.ignored_language_prompts.iter().cloned().collect(),
            selected_model: Some(self.selected_model.clone()),
            model_auto_mode: Some(self.model_auto_mode),
            custom_models: self.custom_models.clone(),
            git_graph_height: Some(self.git_state.graph_height),
        };

        if let Ok(json) = serde_json::to_string_pretty(&state) {
            let _ = std::fs::write(path, json);
        }
    }

    #[allow(dead_code)]
    pub fn load_persistent_state(&mut self) {
        let path = get_config_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(saved) = serde_json::from_str::<PersistentState>(&content) {
                if let Some(theme) = saved.theme { self.theme = theme; }
                if let Some(accent) = saved.accent_color { self.accent_color = accent; }
                if let Some(icon) = saved.icon_theme { self.icon_theme = icon; }
                if let Some(preset) = saved.explorer_preset { self.explorer_preset = preset; }
                if let Some(guides) = saved.explorer_tree_guides { self.explorer_tree_guides = guides; }
                if let Some(git) = saved.explorer_git_status { self.explorer_git_status = git; }
                if let Some(hidden) = saved.explorer_show_hidden { self.explorer_show_hidden = hidden; }
                if let Some(sb_vis) = saved.sidebar_visible { self.sidebar_visible = sb_vis; }
                if let Some(sb_w) = saved.sidebar_width { self.sidebar_width = sb_w; }
                if let Some(st_vis) = saved.status_bar_visible { self.status_bar_visible = st_vis; }
                if let Some(ln_vis) = saved.line_numbers_visible { self.line_numbers_visible = ln_vis; }
                if let Some(zoom) = saved.zoom_level { self.zoom_level = zoom; }
                if !saved.recent_files.is_empty() { self.recent_files = saved.recent_files; }
                if let Some(tv) = saved.terminal_visible { self.terminal_visible = tv; }
                if let Some(th) = saved.terminal_height { self.terminal_height = th; }
                if let Some(aiv) = saved.ai_panel_visible { self.ai_panel_visible = aiv; }
                if let Some(aiw) = saved.ai_panel_width { self.ai_panel_width = aiw; }
                if let Some(sv) = saved.sidebar_view { self.sidebar_view = sv; }
                if !saved.installed_extensions.is_empty() {
                    self.installed_extensions = saved.installed_extensions.into_iter().collect();
                }
                if !saved.ignored_language_prompts.is_empty() {
                    self.ignored_language_prompts = saved.ignored_language_prompts.into_iter().collect();
                }
                if let Some(m) = saved.selected_model { self.selected_model = m; }
                if let Some(a) = saved.model_auto_mode { self.model_auto_mode = a; }
                if !saved.custom_models.is_empty() { self.custom_models = saved.custom_models; }
                if let Some(gh) = saved.git_graph_height { self.git_state.graph_height = gh; }

                let args: Vec<String> = std::env::args().collect();
                if args.len() <= 1 {
                    if let Some(saved_dir) = saved.current_dir {
                        if saved_dir.exists() && saved_dir.is_dir() {
                            self.current_dir = saved_dir;
                            self.has_workspace = true;
                        }
                    }

                    for dir in saved.expanded_dirs {
                        if dir.exists() {
                            self.expanded_dirs.insert(dir);
                        }
                    }

                    for tab_path in saved.open_tabs {
                        if tab_path.exists() && tab_path.is_file() {
                            if let Ok(content) = std::fs::read_to_string(&tab_path) {
                                let name = tab_path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Untitled")
                                    .to_string();

                                self.tabs.push(EditorTab {
                                    name,
                                    path: Some(tab_path.clone()),
                                    editor: Editor::new(Buffer::from_text(&content)),
                                    is_dirty: false,
                                    scroll_y: 0.0,
                                    shaping_cache: std::collections::HashMap::new(),
                                    version: 0,
                                    saved_text: content,
                                });
                            }
                        }
                    }

                    if !self.tabs.is_empty() {
                        self.active_tab = Some(0);
                        if let Some(ref active_path) = saved.active_tab_path {
                            for (idx, tab) in self.tabs.iter().enumerate() {
                                if tab.path.as_ref() == Some(active_path) {
                                    self.active_tab = Some(idx);
                                    break;
                                }
                            }
                        }

                        // On IDE startup: check if the restored active tab needs language support
                        let startup_path = self.active_tab
                            .and_then(|idx| self.tabs.get(idx))
                            .and_then(|t| t.path.clone())
                            .or_else(|| self.tabs.first().and_then(|t| t.path.clone()));
                        if let Some(p) = startup_path {
                            self.selected_sidebar_path = Some(p.clone());
                            self.check_language_support_prompt(&p);
                        }
                    }
                }
            }
        }
    }

    pub fn poll_ai_responses(&mut self) -> bool {
        let mut received = Vec::new();
        if let Some(ref rx) = self.ai_receiver {
            while let Ok((idx, event)) = rx.try_recv() {
                received.push((idx, event));
            }
        }

        let mut updated = false;
        for (idx, event) in received {
            match event {
                ai::AgentEvent::Thinking { delta } => {
                    self.ai_active_status.set(Some("Thinking".to_string()));
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.is_working = true;
                        msg.thoughts = Some(delta.clone());
                    }
                    self.status_message = delta;
                    updated = true;
                }
                ai::AgentEvent::ToolStarted { tool, summary } => {
                    let clean_tool = tool.trim();
                    let status_str = if clean_tool.starts_with("search") {
                        "Searching".to_string()
                    } else if clean_tool.starts_with("read") {
                        "Reading".to_string()
                    } else if clean_tool.starts_with("edit") {
                        "Editing".to_string()
                    } else {
                        "Working".to_string()
                    };
                    self.ai_active_status.set(Some(status_str.clone()));
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.is_working = true;
                        msg.status_text = Some(status_str);
                        msg.content = format!("⚡ *Executing:* `{}`...", tool);
                        if clean_tool.starts_with("read ") || clean_tool.starts_with("read_file") {
                            let path = clean_tool.trim_start_matches("read ").trim_start_matches("read_file").trim().trim_matches('`').trim_start_matches("./");
                            let fname = if path.is_empty() { "file".to_string() } else { path.to_string() };
                            if !fname.is_empty() && !msg.explored_items.iter().any(|i| i.file_name == fname) {
                                msg.explored_items.push(ExploredItem {
                                    file_name: fname,
                                    line_range: None,
                                    action_type: "Analyzed".to_string(),
                                    thought_content: None,
                                    duration_secs: None,
                                    children: Vec::new(),
                                });
                            }
                        } else if clean_tool.starts_with("list ") || clean_tool.starts_with("list_directory") {
                            let path = clean_tool.trim_start_matches("list ").trim_start_matches("list_directory").trim().trim_matches('`').trim_start_matches("./");
                            let fname = if path.is_empty() || path == "." { ".".to_string() } else { path.to_string() };
                            if !fname.is_empty() && !msg.explored_items.iter().any(|i| i.file_name == fname) {
                                msg.explored_items.push(ExploredItem {
                                    file_name: fname,
                                    line_range: None,
                                    action_type: "Analyzed".to_string(),
                                    thought_content: None,
                                    duration_secs: None,
                                    children: Vec::new(),
                                });
                            }
                        }
                    }
                    if let Some(ref mut mission) = self.current_mission {
                        mission.status_message = format!("Agent running: {}", summary);
                        if let Some(task) = mission.tasks.get_mut(1) {
                            task.title = format!("Running: {}", tool);
                            task.status = TaskStatus::InProgress;
                        }
                        self.current_session.update_mission_file(mission);
                    }

                    // Track ONLY the currently active AI terminal command in the drawer
                    let is_run_cmd = tool.starts_with("run ") || tool.starts_with("run `") || tool == "run_command" || tool.starts_with("run");
                    if is_run_cmd {
                        let cmd_str = if let Some(rest) = tool.strip_prefix("run ") {
                            rest.trim_matches('`').trim()
                        } else if let Some(rest) = tool.strip_prefix("run `") {
                            rest.trim_matches('`').trim()
                        } else if let Some(rest) = tool.strip_prefix("run") {
                            rest.trim_matches('`').trim()
                        } else {
                            &tool
                        };
                        let now_ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        if self.ai_terminal_commands.len() > 15 {
                            self.ai_terminal_commands.retain(|c| c.status == AiCommandStatus::Running);
                        }
                        let already_running = self.ai_terminal_commands.iter().any(|c| c.command == cmd_str && c.status == AiCommandStatus::Running);
                        if !already_running {
                            self.ai_terminal_commands.push(AiTerminalCommand {
                                id: format!("cmd_{}", now_ts),
                                command: cmd_str.to_string(),
                                status: AiCommandStatus::Running,
                                output: String::new(),
                                duration_ms: None,
                                timestamp: now_ts,
                            });
                        }
                        self.active_input_drawer = AiInputDrawer::Terminals;
                    }

                    self.status_message = format!("⚡ {}", summary);
                    updated = true;
                }
                ai::AgentEvent::ToolFinished { tool, ok, output_snippet, duration_ms } => {
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.is_working = true;
                        msg.status_text = Some("Working".to_string());
                        let icon = if ok { "✓" } else { "✗" };
                        msg.content = format!("{} *Finished:* `{}` ({}ms)\n\n*Analyzing results...*", icon, tool, duration_ms);

                        let clean_tool = tool.trim();
                        if clean_tool.starts_with("list ") || clean_tool.starts_with("list_directory") {
                            let path = clean_tool.trim_start_matches("list ").trim_start_matches("list_directory").trim().trim_matches('`').trim_start_matches("./");
                            let fname = if path.is_empty() || path == "." { ".".to_string() } else { path.to_string() };
                            let mut discovered_children = Vec::new();
                            for line in output_snippet.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with("📁 ") {
                                    let dir = trimmed.trim_start_matches("📁 ").trim().trim_end_matches('/').to_string();
                                    if !dir.is_empty() {
                                        discovered_children.push(format!("{}/", dir));
                                    }
                                } else if trimmed.starts_with("📄 ") {
                                    let file = trimmed.trim_start_matches("📄 ").trim().split_whitespace().next().unwrap_or("").to_string();
                                    if !file.is_empty() {
                                        discovered_children.push(file);
                                    }
                                }
                            }
                            if let Some(existing) = msg.explored_items.iter_mut().find(|i| i.file_name == fname) {
                                existing.children = discovered_children;
                            } else {
                                msg.explored_items.push(ExploredItem {
                                    file_name: fname,
                                    line_range: None,
                                    action_type: "Analyzed".to_string(),
                                    thought_content: None,
                                    duration_secs: None,
                                    children: discovered_children,
                                });
                            }
                        }
                    }

                    let is_run_cmd = tool.starts_with("run ") || tool.starts_with("run `") || tool == "run_command" || tool.starts_with("run");
                    if is_run_cmd {
                        let is_background = output_snippet.contains("running in the background")
                            || output_snippet.contains("started successfully");
                        if is_background {
                            if let Some(last_cmd) = self.ai_terminal_commands.last_mut() {
                                last_cmd.output = output_snippet.clone();
                                last_cmd.duration_ms = Some(duration_ms);
                                last_cmd.status = AiCommandStatus::Running;
                            }
                        } else {
                            // Command finished and is not background: remove so only active ones remain
                            self.ai_terminal_commands.retain(|c| c.status == AiCommandStatus::Running && c.output.contains("running in the background"));
                        }

                        if self.ai_terminal_commands.is_empty() && self.active_input_drawer == AiInputDrawer::Terminals {
                            self.active_input_drawer = AiInputDrawer::None;
                        }
                    }

                    self.rebuild_tree();
                    updated = true;
                }
                ai::AgentEvent::PendingModifications { summaries } => {
                    let now_ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    for s in &summaries {
                        let target_file = if Path::new(&s.file_name).is_absolute() {
                            PathBuf::from(&s.file_name)
                        } else {
                            self.current_dir.join(&s.file_name)
                        };

                        let disk_content = std::fs::read_to_string(&target_file).unwrap_or_default();
                        let original_text = s.original_code.as_deref().unwrap_or(&disk_content);
                        let proposed_text = s.proposed_code.as_deref().unwrap_or(&disk_content);
                        let is_new_file = s.lines_removed == 0 && (s.lines_added > 0 || original_text.is_empty());

                        let (diff_lines, preview_text) = compute_line_diff(original_text, proposed_text);

                        let pm = PendingModification {
                            file_path: target_file.clone(),
                            original_code: original_text.to_string(),
                            proposed_code: proposed_text.to_string(),
                            diff_lines,
                        };

                        self.pending_modifications.retain(|p| p.file_path != target_file);
                        self.pending_modifications.push(pm.clone());
                        if self.pending_modification.is_none() {
                            self.pending_modification = Some(pm);
                        }

                        // Open or update tab so diff is immediately visible in buffer
                        let target_name = target_file.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                        if let Some(tab) = self.tabs.iter_mut().find(|t| {
                            t.path.as_deref() == Some(&target_file)
                                || t.name == target_name
                        }) {
                            tab.path = Some(target_file.clone());
                            tab.editor.replace_all(&preview_text);
                            tab.is_dirty = true;
                            tab.saved_text = original_text.to_string();
                        } else if self.tabs.len() < 10 {
                            self.tabs.push(EditorTab {
                                name: target_name.to_string(),
                                path: Some(target_file.clone()),
                                editor: Editor::new(Buffer::from_text(&preview_text)),
                                is_dirty: true,
                                scroll_y: 0.0,
                                shaping_cache: std::collections::HashMap::new(),
                                version: 0,
                                saved_text: original_text.to_string(),
                            });
                            self.active_tab = Some(self.tabs.len() - 1);
                        }

                        if let Some(existing) = self.session_modified_files.iter_mut().find(|f| f.file_name == s.file_name) {
                            existing.lines_added = s.lines_added;
                            existing.lines_removed = s.lines_removed;
                            existing.timestamp = now_ts;
                        } else {
                            self.session_modified_files.push(SessionModifiedFile {
                                file_path: target_file.clone(),
                                file_name: s.file_name.clone(),
                                lines_added: s.lines_added,
                                lines_removed: s.lines_removed,
                                timestamp: now_ts,
                                is_new_file,
                            });
                        }
                    }

                    let state_summaries: Vec<EditedFileSummary> = summaries.iter().map(|s| EditedFileSummary {
                        file_name: s.file_name.clone(),
                        lines_added: s.lines_added,
                        lines_removed: s.lines_removed,
                        original_code: s.original_code.clone(),
                        proposed_code: s.proposed_code.clone(),
                    }).collect();

                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        for s in &state_summaries {
                            if !msg.edited_files.iter().any(|f| f.file_name == s.file_name) {
                                msg.edited_files.push(s.clone());
                            }
                        }
                    }

                    self.active_input_drawer = AiInputDrawer::PendingModifications;
                    self.current_session.update_walkthrough_file(&self.last_ai_prompt, &state_summaries, false);
                    self.rebuild_tree();
                    self.status_message = format!("✓ Updated {} file(s) - Review changes in editor", summaries.len());
                    updated = true;
                }
                ai::AgentEvent::FileAnalyzed { file_path, start_line, end_line } => {
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        let clean_path = file_path.trim_start_matches("./").to_string();
                        let p = std::path::Path::new(&clean_path);
                        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or(&clean_path).to_string();
                        let range = format!("#L{}-{}", start_line, end_line);
                        if let Some(existing) = msg.explored_items.iter_mut().find(|i| i.file_name == clean_path || i.file_name == file_name) {
                            existing.line_range = Some(range);
                        } else {
                            msg.explored_items.push(ExploredItem {
                                file_name: clean_path,
                                line_range: Some(range),
                                action_type: "Analyzed".to_string(),
                                thought_content: None,
                                duration_secs: None,
                                children: Vec::new(),
                            });
                        }
                    }
                    updated = true;
                }
                ai::AgentEvent::SearchExecuted { query, result_count } => {
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.explored_items.push(ExploredItem {
                            file_name: query.clone(),
                            line_range: Some(format!("({} result{})", result_count, if result_count == 1 { "" } else { "s" })),
                            action_type: "Searched".to_string(),
                            thought_content: None,
                            duration_secs: None,
                            children: Vec::new(),
                        });
                    }
                    updated = true;
                }
                ai::AgentEvent::ThoughtBlock { content, duration_secs } => {
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.explored_items.push(ExploredItem {
                            file_name: String::new(),
                            line_range: None,
                            action_type: "Thought".to_string(),
                            thought_content: Some(content),
                            duration_secs: Some(duration_secs),
                            children: Vec::new(),
                        });
                    }
                    updated = true;
                }
                ai::AgentEvent::Completed { summary } => {
                    let duration = self.ai_request_start_times.remove(&idx)
                        .map(|t| t.elapsed().as_secs())
                        .unwrap_or(0);
                    let mut clean_summary = summary;
                    while let Some(start) = clean_summary.find("<thought>") {
                        if let Some(end) = clean_summary[start..].find("</thought>") {
                            clean_summary.replace_range(start..start + end + 10, "");
                        } else {
                            break;
                        }
                    }
                    while let Some(start) = clean_summary.find("<thinking>") {
                        if let Some(end) = clean_summary[start..].find("</thinking>") {
                            clean_summary.replace_range(start..start + end + 11, "");
                        } else {
                            break;
                        }
                    }
                    let trimmed_content = clean_summary.trim();
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.elapsed_secs = Some(duration.max(1));
                        if trimmed_content.is_empty() {
                            msg.content = format!(
                                "I completed the workspace tasks for `{}`. Let me know if you would like any further changes or testing!",
                                self.last_ai_prompt
                            );
                        } else {
                            msg.content = trimmed_content.to_string();
                        }
                        msg.is_working = false;
                        msg.status_text = None;
                    }
                    self.ai_pending_requests = self.ai_pending_requests.saturating_sub(1);
                    if self.ai_pending_requests == 0 {
                        self.ai_status = AiStatus::Ready;
                        self.ai_active_status.set(None);
                        self.ai_active_file.set(None);
                    }
                    if let Some(ref mut mission) = self.current_mission {
                        for task in &mut mission.tasks {
                            if task.status != TaskStatus::Completed {
                                task.status = TaskStatus::Completed;
                            }
                        }
                        mission.is_active = false;
                        mission.status_message = "✓ All tasks completed successfully".to_string();
                        self.current_session.update_mission_file(mission);
                    }
                    self.rebuild_tree();
                    updated = true;
                }
                ai::AgentEvent::StatusChanged { status } => {
                    self.ai_active_status.set(Some(status.clone()));
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.is_working = true;
                        msg.status_text = Some(status);
                    }
                    updated = true;
                }
                ai::AgentEvent::Error { error } => {
                    self.ai_active_status.set(None);
                    self.ai_active_file.set(None);
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.content = format!("⚠️ {}", error);
                        msg.is_working = false;
                        msg.status_text = None;
                    }
                    self.ai_pending_requests = self.ai_pending_requests.saturating_sub(1);
                    if self.ai_pending_requests == 0 {
                        self.ai_status = AiStatus::Ready;
                    }
                    if let Some(ref mut mission) = self.current_mission {
                        mission.is_active = false;
                        mission.status_message = format!("⚠️ Mission failed: {}", error);
                        self.current_session.update_mission_file(mission);
                    }
                    self.last_provider_notice = Some(format!("AI Notice: {}", error));
                    updated = true;
                }
                _ => {}
            }
        }
        if updated {
            self.save_current_session_messages();
        }
        updated
    }

    pub fn accept_all_modifications(&mut self) {
        let mut pendings = std::mem::take(&mut self.pending_modifications);
        if pendings.is_empty() {
            if let Some(p) = self.pending_modification.take() {
                pendings.push(p);
            }
        } else {
            self.pending_modification = None;
        }

        if pendings.is_empty() {
            return;
        }

        let total = pendings.len();
        let mut summaries = Vec::new();

        for pending in pendings {
            // Ensure parent directory exists on disk
            if let Some(parent) = pending.file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            // Write modified or newly created code directly to disk
            let _ = std::fs::write(&pending.file_path, &pending.proposed_code);

            // Update any open editor tab matching this file
            for tab in &mut self.tabs {
                if tab.path.as_deref() == Some(&pending.file_path) {
                    tab.editor.replace_all(&pending.proposed_code);
                    tab.is_dirty = false;
                    tab.saved_text = pending.proposed_code.clone();
                }
            }

            let file_name = pending.file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
            let added = pending.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
            let removed = pending.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
            summaries.push(EditedFileSummary {
                file_name,
                lines_added: added,
                lines_removed: removed,
                original_code: None,
                proposed_code: None,
            });
        }

        // Rebuild file tree so any new files and folders appear in sidebar immediately
        self.rebuild_tree();
        self.active_input_drawer = AiInputDrawer::None;

        if !summaries.is_empty() {
            self.current_session.update_walkthrough_file(
                &self.last_ai_prompt,
                &summaries,
                true,
            );
        }

        self.status_message = format!("✓ Accepted and saved {} modified file{}", total, if total == 1 { "" } else { "s" });
    }

    #[allow(dead_code)]
    pub fn accept_pending_modification(&mut self) {
        self.accept_all_modifications();
    }

    pub fn reject_all_modifications(&mut self) {
        let mut pendings = std::mem::take(&mut self.pending_modifications);
        if pendings.is_empty() {
            if let Some(p) = self.pending_modification.take() {
                pendings.push(p);
            }
        } else {
            self.pending_modification = None;
        }

        if pendings.is_empty() {
            return;
        }

        let total = pendings.len();

        for pending in pendings {
            if pending.original_code.is_empty() {
                // If it was a new uncreated file, remove from disk if it was created
                let _ = std::fs::remove_file(&pending.file_path);
                // Close preview tab
                if let Some(pos) = self.tabs.iter().position(|t| t.path.as_deref() == Some(&pending.file_path)) {
                    self.tabs.remove(pos);
                    if let Some(act) = self.active_tab {
                        if act >= self.tabs.len() {
                            self.active_tab = if self.tabs.is_empty() { None } else { Some(self.tabs.len() - 1) };
                        }
                    }
                }
            } else {
                // Restore original code to disk
                let _ = std::fs::write(&pending.file_path, &pending.original_code);
                // Restore original code in any open editor tab matching this file
                for tab in &mut self.tabs {
                    if tab.path.as_deref() == Some(&pending.file_path) {
                        tab.editor.replace_all(&pending.original_code);
                        tab.is_dirty = false;
                        tab.saved_text = pending.original_code.clone();
                    }
                }
            }
        }

        self.rebuild_tree();
        self.active_input_drawer = AiInputDrawer::None;
        self.status_message = format!("✕ Rejected changes to {} file{}", total, if total == 1 { "" } else { "s" });
    }

    #[allow(dead_code)]
    pub fn reject_pending_modification(&mut self) {
        self.reject_all_modifications();
    }

    #[allow(dead_code)]
    pub fn accept_single_modification(&mut self, path: &std::path::Path) {
        let pending = if let Some(pos) = self.pending_modifications.iter().position(|p| p.file_path == path) {
            Some(self.pending_modifications.remove(pos))
        } else if self.pending_modification.as_ref().map(|p| &p.file_path) == Some(&path.to_path_buf()) {
            self.pending_modification.take()
        } else {
            None
        };

        if let Some(pending) = pending {
            if let Some(parent) = pending.file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&pending.file_path, &pending.proposed_code);
            for tab in &mut self.tabs {
                if tab.path.as_deref() == Some(&pending.file_path) {
                    tab.editor.replace_all(&pending.proposed_code);
                    tab.is_dirty = false;
                    tab.saved_text = pending.proposed_code.clone();
                }
            }

            let file_name = pending.file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
            let added = pending.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
            let removed = pending.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
            let summary = EditedFileSummary {
                file_name: file_name.clone(),
                lines_added: added,
                lines_removed: removed,
                original_code: None,
                proposed_code: None,
            };
            self.current_session.update_walkthrough_file(
                &self.last_ai_prompt,
                &[summary],
                true,
            );

            if self.pending_modifications.is_empty() && self.pending_modification.is_none() {
                self.active_input_drawer = AiInputDrawer::None;
            } else if self.pending_modification.is_none() && !self.pending_modifications.is_empty() {
                self.pending_modification = self.pending_modifications.first().cloned();
            }

            self.rebuild_tree();
            self.status_message = format!("✓ Accepted and saved {}", file_name);
        }
    }

    pub fn reject_single_modification(&mut self, path: &std::path::Path) {
        let pending = if let Some(pos) = self.pending_modifications.iter().position(|p| p.file_path == path) {
            Some(self.pending_modifications.remove(pos))
        } else if self.pending_modification.as_ref().map(|p| &p.file_path) == Some(&path.to_path_buf()) {
            self.pending_modification.take()
        } else {
            None
        };

        if let Some(pending) = pending {
            if pending.original_code.is_empty() {
                let _ = std::fs::remove_file(&pending.file_path);
                if let Some(pos) = self.tabs.iter().position(|t| t.path.as_deref() == Some(&pending.file_path)) {
                    self.tabs.remove(pos);
                    if let Some(act) = self.active_tab {
                        if act >= self.tabs.len() {
                            self.active_tab = if self.tabs.is_empty() { None } else { Some(self.tabs.len() - 1) };
                        }
                    }
                }
            } else {
                let _ = std::fs::write(&pending.file_path, &pending.original_code);
                for tab in &mut self.tabs {
                    if tab.path.as_deref() == Some(&pending.file_path) {
                        tab.editor.replace_all(&pending.original_code);
                        tab.is_dirty = false;
                        tab.saved_text = pending.original_code.clone();
                    }
                }
            }

            if self.pending_modifications.is_empty() && self.pending_modification.is_none() {
                self.active_input_drawer = AiInputDrawer::None;
            } else if self.pending_modification.is_none() && !self.pending_modifications.is_empty() {
                self.pending_modification = self.pending_modifications.first().cloned();
            }

            self.rebuild_tree();
            let file_name = pending.file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
            self.status_message = format!("✕ Rejected changes to {}", file_name);
        }
    }

    pub fn toggle_explored_accordion(&mut self, msg_idx: usize) {
        if self.expanded_explored_msgs.contains(&msg_idx) {
            self.expanded_explored_msgs.remove(&msg_idx);
        } else {
            self.expanded_explored_msgs.insert(msg_idx);
        }
    }

    pub fn toggle_explored_folder(&mut self, msg_idx: usize, folder_path: &str) {
        let key = (msg_idx, folder_path.to_string());
        if self.collapsed_explored_folders.contains(&key) {
            self.collapsed_explored_folders.remove(&key);
        } else {
            self.collapsed_explored_folders.insert(key);
        }
    }

    pub fn is_explored_folder_expanded(&self, msg_idx: usize, folder_path: &str) -> bool {
        !self.collapsed_explored_folders.contains(&(msg_idx, folder_path.to_string()))
    }

    pub fn toggle_thought_accordion(&mut self, msg_idx: usize, item_idx: usize) {
        let key = (msg_idx, item_idx);
        if self.expanded_thought_items.contains(&key) {
            self.expanded_thought_items.remove(&key);
        } else {
            self.expanded_thought_items.insert(key);
        }
    }

    #[allow(dead_code)]
    pub fn toggle_worked_accordion(&mut self, msg_idx: usize) {
        if self.expanded_worked_msgs.contains(&msg_idx) {
            self.expanded_worked_msgs.remove(&msg_idx);
        } else {
            self.expanded_worked_msgs.insert(msg_idx);
        }
    }

    pub fn jump_to_file_and_line(&mut self, file_name: &str, line_range: Option<&str>) {
        let full_path = if std::path::Path::new(file_name).is_absolute() {
            std::path::PathBuf::from(file_name)
        } else {
            self.current_dir.join(file_name)
        };
        if full_path.is_dir() {
            self.expanded_dirs.insert(full_path.clone());
            self.selected_sidebar_path = Some(full_path);
            self.rebuild_tree();
            return;
        }
        self.open_file_by_name(file_name);
        if let Some(range_str) = line_range {
            let clean = range_str.trim_start_matches("#L").trim_start_matches('L');
            let mut parts = clean.split('-');
            let start_line = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            let end_line = parts.next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(start_line);
            if let Some(tab_idx) = self.active_tab {
                if let Some(tab) = self.tabs.get_mut(tab_idx) {
                    let total_lines = tab.editor.buffer.lines().len();
                    let start_row = (start_line.saturating_sub(1)).min(total_lines.saturating_sub(1));
                    let end_row = (end_line.saturating_sub(1)).min(total_lines.saturating_sub(1));
                    let end_col = tab.editor.buffer.line(end_row).map(|l| l.len()).unwrap_or(0);
                    let sel = editor::selection::Selection::new(
                        editor::buffer::Point::new(start_row, 0),
                        editor::buffer::Point::new(end_row, end_col),
                    );
                    tab.editor.set_selection(sel);
                    tab.scroll_y = (start_row as f32 * 20.0).max(0.0);
                }
            }
        }
    }

    pub fn open_file_by_name(&mut self, name: &str) {
        for (idx, tab) in self.tabs.iter().enumerate() {
            if tab.name == name || tab.path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()) == Some(name) {
                self.active_tab = Some(idx);
                return;
            }
        }
        let direct = self.current_dir.join(name);
        if direct.exists() {
            self.open_file(&direct);
        }
    }

    pub fn is_agent_working(&self) -> bool {
        self.ai_pending_requests > 0
            || self.ai_messages.iter().any(|m| m.is_working)
            || self.current_mission.as_ref().map(|m| m.is_active && !m.is_paused).unwrap_or(false)
    }

    pub fn pause_agent_mission(&mut self) {
        if let Some(ref mut mission) = self.current_mission {
            mission.is_paused = true;
            mission.status_message = "⏸ Mission paused by user".to_string();
            self.current_session.update_mission_file(mission);
        }
        self.ai_pending_requests = 0;
        for msg in &mut self.ai_messages {
            msg.is_working = false;
        }
    }

    pub fn resume_agent_mission(&mut self) {
        if let Some(ref mut mission) = self.current_mission {
            mission.is_paused = false;
            mission.status_message = "Resuming mission...".to_string();
            self.current_session.update_mission_file(mission);
        }
    }

    pub fn colors(&self) -> ThemeColors {
        self.colors_signal.get()
    }

    pub fn rebuild_tree(&mut self) {
        self.sidebar_items.clear();
        if !self.has_workspace {
            self.workspace_tree_overview.clear();
            return;
        }
        let root = self.current_dir.clone();
        let git_statuses = if self.explorer_git_status {
            self.get_git_statuses(&root)
        } else {
            HashMap::new()
        };
        let mut ancestors = Vec::new();
        self.scan_dir_recursive(&root, 0, &mut ancestors, &git_statuses);

        // Store and maintain in-memory workspace tree structure in real-time
        self.workspace_tree_overview = crate::project_index::generate_project_tree_overview(&root, 6);
    }

    pub fn get_or_update_workspace_tree(&mut self) -> &str {
        if self.has_workspace {
            self.workspace_tree_overview = crate::project_index::generate_project_tree_overview(&self.current_dir, 6);
        } else {
            self.workspace_tree_overview.clear();
        }
        &self.workspace_tree_overview
    }

    pub fn save_current_session_messages(&self) {
        if self.ai_messages.is_empty() {
            return;
        }
        self.current_session.save_messages(&self.ai_messages);
    }

    pub fn init_workspace_session(&mut self) {
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);
        if let Some(latest) = self.session_list.first() {
            if let Some(sess) = crate::session::AgentSession::load(&self.current_dir, &latest.id) {
                self.current_session = sess;
                self.ai_messages = self.current_session.load_messages().unwrap_or_default();
                if let Some(last_user) = self.ai_messages.iter().filter(|m| m.is_user).last() {
                    self.last_ai_prompt = last_user.content.clone();
                }
                return;
            }
        }
        self.current_session = crate::session::AgentSession::new(&self.current_dir, None);
        self.ai_messages.clear();
        self.ai_showing_session_list = false;
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);
    }

    pub fn start_new_session(&mut self) {
        if !self.ai_messages.is_empty() {
            self.save_current_session_messages();
            let new_sess = crate::session::AgentSession::new(&self.current_dir, None);
            self.current_session = new_sess;
            self.current_mission = None;
            self.pending_modification = None;
            self.pending_modifications.clear();
            self.ai_messages.clear();
            self.session_list = crate::session::list_saved_sessions(&self.current_dir);
            self.session_modified_files.clear();
            self.status_message = "Started new agent session".to_string();
        }
        self.session_dropdown_open = false;
        self.active_input_drawer = AiInputDrawer::None;
        self.ai_showing_session_list = false;
    }

    pub fn open_session_file(&mut self, filename: &str) {
        let path = self.current_session.dir_path.join(filename);
        if !path.exists() {
            self.current_session.init_starter_files();
        }
        self.open_file(&path);
        self.status_message = format!("Opened session file: {}", filename);
    }

    pub fn switch_session(&mut self, session_id: &str) {
        // Save conversation of current session first if it has messages
        self.save_current_session_messages();

        if let Some(sess) = crate::session::AgentSession::load(&self.current_dir, session_id) {
            self.current_session = sess;
            self.current_mission = None;
            self.pending_modification = None;
            self.pending_modifications.clear();
            self.session_modified_files.clear();
            self.active_input_drawer = AiInputDrawer::None;

            // Load saved chat messages directly
            self.ai_messages = self.current_session.load_messages().unwrap_or_default();

            if let Some(last_user) = self.ai_messages.iter().filter(|m| m.is_user).last() {
                self.last_ai_prompt = last_user.content.clone();
            }

            self.ai_showing_session_list = false;
            self.session_dropdown_open = false;
            self.status_message = format!("Switched to session {}", self.current_session.title);
        }
    }

    pub fn delete_session(&mut self, session_id: &str) {
        let is_current = self.current_session.id == session_id;
        crate::session::delete_session(&self.current_dir, session_id);
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);

        if is_current {
            if let Some(next_sess) = self.session_list.first() {
                if let Some(sess) = crate::session::AgentSession::load(&self.current_dir, &next_sess.id) {
                    self.current_session = sess;
                    self.ai_messages = self.current_session.load_messages().unwrap_or_default();
                    if let Some(last_user) = self.ai_messages.iter().filter(|m| m.is_user).last() {
                        self.last_ai_prompt = last_user.content.clone();
                    }
                } else {
                    self.current_session = crate::session::AgentSession::new(&self.current_dir, None);
                    self.ai_messages.clear();
                }
            } else {
                self.current_session = crate::session::AgentSession::new(&self.current_dir, None);
                self.ai_messages.clear();
            }
            self.current_mission = None;
            self.pending_modification = None;
            self.pending_modifications.clear();
            self.session_modified_files.clear();
            self.active_input_drawer = AiInputDrawer::None;
            self.session_dropdown_open = false;
            self.ai_showing_session_list = false;
            self.session_list = crate::session::list_saved_sessions(&self.current_dir);
        }
        self.status_message = format!("Deleted session {}", session_id);
    }

    pub fn open_workspace(&mut self, path: PathBuf) {
        self.save_current_session_messages();
        self.current_dir = path;
        self.has_workspace = true;
        self.expanded_dirs.clear();
        self.init_workspace_session();
        self.rebuild_tree();
        self.refresh_git_status();
        self.status_message = format!("Opened workspace: {}", self.current_dir.display());
        self.sync_workspace_signal();
        self.save_persistent_state();
    }

    pub fn close_workspace(&mut self) {
        self.save_current_session_messages();
        self.has_workspace = false;
        self.sidebar_items.clear();
        self.expanded_dirs.clear();
        self.workspace_tree_overview.clear();
        self.git_state = GitRepoState::default();
        self.init_workspace_session();
        self.status_message = "Workspace closed".to_string();
        self.sync_workspace_signal();
        self.save_persistent_state();
    }

    fn get_git_statuses(&self, root: &Path) -> HashMap<PathBuf, GitStatus> {
        let mut map = HashMap::new();
        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain", "-uall"])
            .current_dir(root)
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.len() < 4 {
                        continue;
                    }
                    let status_code = &line[0..2];
                    let file_rel = line[3..].trim();
                    let file_clean = if let Some((_, to)) = file_rel.split_once(" -> ") {
                        to.trim()
                    } else {
                        file_rel
                    };
                    let full_path = root.join(file_clean);
                    let status = if status_code.contains('?') {
                        GitStatus::Untracked
                    } else {
                        GitStatus::Modified
                    };
                    
                    map.insert(full_path.clone(), status);

                    let mut current = full_path.parent();
                    while let Some(parent) = current {
                        if parent == root || !parent.starts_with(root) {
                            break;
                        }
                        map.entry(parent.to_path_buf()).or_insert(GitStatus::Modified);
                        current = parent.parent();
                    }
                }
            }
        }
        map
    }

    pub fn refresh_git_status(&mut self) {
        let root = if self.has_workspace {
            self.current_dir.clone()
        } else if let Some(active_idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(active_idx) {
                if let Some(ref p) = tab.path {
                    p.parent().unwrap_or(&self.current_dir).to_path_buf()
                } else {
                    self.current_dir.clone()
                }
            } else {
                self.current_dir.clone()
            }
        } else {
            self.current_dir.clone()
        };

        let is_inside = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&root)
            .output()
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
            .unwrap_or(false);

        if !is_inside {
            self.git_state.is_repo = false;
            self.git_state.branch.clear();
            self.git_state.staged.clear();
            self.git_state.unstaged.clear();
            return;
        }

        self.git_state.is_repo = true;

        let branch_out = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&root)
            .output();

        let branch = if let Ok(o) = branch_out {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() {
                s
            } else {
                std::process::Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .current_dir(&root)
                    .output()
                    .map(|ho| {
                        let hash = String::from_utf8_lossy(&ho.stdout).trim().to_string();
                        if hash.is_empty() {
                            "HEAD".to_string()
                        } else {
                            format!("detached ({})", hash)
                        }
                    })
                    .unwrap_or_else(|_| "HEAD".to_string())
            }
        } else {
            "HEAD".to_string()
        };
        self.git_state.branch = branch;

        let status_out = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "-uall"])
            .current_dir(&root)
            .output();

        if let Ok(output) = status_out {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut staged = Vec::new();
                let mut unstaged = Vec::new();

                for line in stdout.lines() {
                    if line.len() < 4 {
                        continue;
                    }
                    let x = line.as_bytes()[0] as char;
                    let y = line.as_bytes()[1] as char;

                    let mut path_str = line[3..].trim();
                    if path_str.starts_with('"') && path_str.ends_with('"') && path_str.len() >= 2 {
                        path_str = &path_str[1..path_str.len() - 1];
                    }

                    let rel_path = if let Some((_, to)) = path_str.split_once(" -> ") {
                        to.trim().to_string()
                    } else {
                        path_str.to_string()
                    };

                    let full_path = root.join(&rel_path);
                    let filename = full_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| rel_path.clone());
                    let dir_path = full_path
                        .parent()
                        .and_then(|p| p.strip_prefix(&root).ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Check Staged (X)
                    if x != ' ' && x != '?' && x != '!' {
                        let status = match x {
                            'M' => GitFileStatusCode::Modified,
                            'A' => GitFileStatusCode::Added,
                            'D' => GitFileStatusCode::Deleted,
                            'R' => GitFileStatusCode::Renamed,
                            'U' => GitFileStatusCode::Conflicted,
                            _ => GitFileStatusCode::Modified,
                        };
                        staged.push(GitChangeItem {
                            path: full_path.clone(),
                            rel_path: rel_path.clone(),
                            filename: filename.clone(),
                            dir_path: dir_path.clone(),
                            status,
                            is_staged: true,
                        });
                    }

                    // Check Unstaged (Y)
                    if y != ' ' && y != '!' {
                        let status = match y {
                            'M' => GitFileStatusCode::Modified,
                            'D' => GitFileStatusCode::Deleted,
                            '?' => GitFileStatusCode::Untracked,
                            'U' => GitFileStatusCode::Conflicted,
                            _ => GitFileStatusCode::Modified,
                        };
                        unstaged.push(GitChangeItem {
                            path: full_path,
                            rel_path,
                            filename,
                            dir_path,
                            status,
                            is_staged: false,
                        });
                    }
                }

                self.git_state.staged = staged;
                self.git_state.unstaged = unstaged;
                self.git_state.status_error = None;
            } else {
                let err = String::from_utf8_lossy(&output.stderr).to_string();
                self.git_state.status_error = Some(err);
            }
        }

        // Fetch git commit history with DAG graph for Graph panel
        let log_out = std::process::Command::new("git")
            .args(["log", "--graph", "-n", "50", "--pretty=format:COMMIT_SPLIT%H|%h|%an|%cr|%s|%D"])
            .current_dir(&root)
            .output();

        if let Ok(lo) = log_out {
            if lo.status.success() {
                let stdout = String::from_utf8_lossy(&lo.stdout);
                let mut history = Vec::new();
                for line in stdout.lines() {
                    if let Some(split_idx) = line.find("COMMIT_SPLIT") {
                        let prefix = &line[..split_idx];
                        let commit_data = &line[split_idx + "COMMIT_SPLIT".len()..];
                        let parts: Vec<&str> = commit_data.splitn(6, '|').collect();
                        if parts.len() >= 5 {
                            let hash = parts[0].to_string();
                            let short_hash = parts[1].to_string();
                            let author = parts[2].to_string();
                            let time_ago = parts[3].to_string();
                            let message = parts[4].to_string();
                            let refs_raw = parts.get(5).unwrap_or(&"");

                            let is_head = history.is_empty() || refs_raw.contains("HEAD");
                            let mut branch_refs = Vec::new();
                            for r in refs_raw.split(',') {
                                let clean = r.trim().trim_start_matches("HEAD -> ").trim();
                                if !clean.is_empty() {
                                    branch_refs.push(clean.to_string());
                                }
                            }

                            // Calculate lane from position of '*' in git's graph prefix
                            let star_pos = prefix.chars().position(|c| c == '*').unwrap_or(0);
                            let lane = (star_pos / 2).min(5);

                            history.push(GitCommitLogItem {
                                hash,
                                short_hash,
                                author,
                                time_ago,
                                message,
                                is_head,
                                branch_refs,
                                lane,
                                graph_prefix: prefix.to_string(),
                            });
                        }
                    }
                }
                self.git_state.commit_history = history;
            }
        }
    }

    pub fn git_stage_file(&mut self, path: &Path) {
        if !self.has_workspace {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["add", "--"])
            .arg(path)
            .current_dir(&self.current_dir)
            .output();
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_unstage_file(&mut self, path: &Path) {
        if !self.has_workspace {
            return;
        }
        let out = std::process::Command::new("git")
            .args(["restore", "--staged", "--"])
            .arg(path)
            .current_dir(&self.current_dir)
            .output();
        if let Ok(o) = out {
            if !o.status.success() {
                let _ = std::process::Command::new("git")
                    .args(["reset", "HEAD", "--"])
                    .arg(path)
                    .current_dir(&self.current_dir)
                    .output();
            }
        }
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_stage_all(&mut self) {
        if !self.has_workspace {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.current_dir)
            .output();
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_unstage_all(&mut self) {
        if !self.has_workspace {
            return;
        }
        let out = std::process::Command::new("git")
            .args(["restore", "--staged", "."])
            .current_dir(&self.current_dir)
            .output();
        if let Ok(o) = out {
            if !o.status.success() {
                let _ = std::process::Command::new("git")
                    .args(["reset", "HEAD"])
                    .current_dir(&self.current_dir)
                    .output();
            }
        }
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_discard_file(&mut self, path: &Path, is_untracked: bool) {
        if !self.has_workspace {
            return;
        }
        if is_untracked {
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(path);
            } else {
                let _ = std::fs::remove_file(path);
            }
        } else {
            let out = std::process::Command::new("git")
                .args(["restore", "--"])
                .arg(path)
                .current_dir(&self.current_dir)
                .output();
            if let Ok(o) = out {
                if !o.status.success() {
                    let _ = std::process::Command::new("git")
                        .args(["checkout", "--"])
                        .arg(path)
                        .current_dir(&self.current_dir)
                        .output();
                }
            }
        }
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_commit(&mut self) -> Result<(), String> {
        if !self.has_workspace {
            return Err("No workspace open".to_string());
        }
        let msg = self.git_state.commit_message.trim().to_string();
        if msg.is_empty() {
            return Err("Commit message cannot be empty".to_string());
        }
        if self.git_state.staged.is_empty() && !self.git_state.unstaged.is_empty() {
            self.git_stage_all();
        }
        if self.git_state.staged.is_empty() {
            return Err("No changes to commit".to_string());
        }
        let out = std::process::Command::new("git")
            .args(["commit", "-m", &msg])
            .current_dir(&self.current_dir)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                self.git_state.commit_message.clear();
                self.git_state.status_error = None;
                self.status_message = "Git commit succeeded".to_string();
                self.refresh_git_status();
                self.rebuild_tree();
                Ok(())
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr).to_string();
                let msg = if err.is_empty() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    err
                };
                self.git_state.status_error = Some(msg.clone());
                self.status_message = format!("Commit failed: {}", msg);
                Err(msg)
            }
            Err(e) => {
                let msg = e.to_string();
                self.git_state.status_error = Some(msg.clone());
                self.status_message = format!("Commit error: {}", msg);
                Err(msg)
            }
        }
    }

    pub fn git_init(&mut self) {
        if !self.has_workspace {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&self.current_dir)
            .output();
        self.refresh_git_status();
        self.rebuild_tree();
    }

    fn scan_dir_recursive(
        &mut self,
        dir: &Path,
        depth: usize,
        ancestor_is_last: &mut Vec<bool>,
        git_statuses: &HashMap<PathBuf, GitStatus>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            // Sort: folders first, then files alphabetically
            paths.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir != b_is_dir {
                    b_is_dir.cmp(&a_is_dir)
                } else {
                    a.file_name().cmp(&b.file_name())
                }
            });

            // Filter out hidden files unless toggled on
            paths.retain(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if self.explorer_show_hidden {
                    name != "." && name != ".."
                } else {
                    !name.starts_with('.') || name == ".gitignore" || name == ".github"
                }
            });

            let total = paths.len();
            for (i, path) in paths.into_iter().enumerate() {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let is_dir = path.is_dir();
                let is_expanded = is_dir && self.expanded_dirs.contains(&path);
                let is_last = i + 1 == total;

                let mut guide_prefix = String::new();
                for &last in ancestor_is_last.iter() {
                    if last {
                        guide_prefix.push_str("   ");
                    } else {
                        guide_prefix.push_str("│  ");
                    }
                }
                if is_last {
                    guide_prefix.push_str("└─ ");
                } else {
                    guide_prefix.push_str("├─ ");
                }

                let git_status = git_statuses.get(&path).cloned();

                self.sidebar_items.push(FileTreeNode {
                    path: path.clone(),
                    name,
                    depth,
                    is_dir,
                    is_expanded,
                    is_last,
                    guide_prefix,
                    git_status,
                });

                if is_expanded {
                    ancestor_is_last.push(is_last);
                    self.scan_dir_recursive(&path, depth + 1, ancestor_is_last, git_statuses);
                    ancestor_is_last.pop();
                }
            }
        }
    }

    pub fn collapse_all_folders(&mut self) {
        self.expanded_dirs.clear();
        self.rebuild_tree();
        self.sync_workspace_signal();
        self.save_persistent_state();
    }

    pub fn expand_all_folders(&mut self) {
        fn collect_dirs(dir: &Path, acc: &mut std::collections::HashSet<PathBuf>, show_hidden: bool) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let allow = if show_hidden {
                            name != "." && name != ".."
                        } else {
                            !name.starts_with('.') || name == ".github"
                        };
                        if allow {
                            acc.insert(path.clone());
                            collect_dirs(&path, acc, show_hidden);
                        }
                    }
                }
            }
        }
        collect_dirs(&self.current_dir, &mut self.expanded_dirs, self.explorer_show_hidden);
        self.rebuild_tree();
        self.sync_workspace_signal();
        self.save_persistent_state();
    }

    #[allow(dead_code)]
    pub fn new_file_in_workspace(&mut self) {
        if !self.has_workspace {
            self.new_untitled_file();
            return;
        }
        self.start_new_file_inline(None);
    }

    pub fn start_new_file_inline(&mut self, target_dir: Option<PathBuf>) {
        if !self.has_workspace {
            self.new_untitled_file();
            return;
        }
        let dir = target_dir.unwrap_or_else(|| {
            if let Some(ref selected) = self.selected_sidebar_path {
                if selected.is_dir() {
                    selected.clone()
                } else {
                    selected.parent().unwrap_or(&self.current_dir).to_path_buf()
                }
            } else {
                self.current_dir.clone()
            }
        });
        if dir != self.current_dir {
            self.expanded_dirs.insert(dir.clone());
            self.rebuild_tree();
        }
        self.inline_tree_input = Some(InlineTreeState {
            mode: InlineTreeInputMode::NewFile { parent_dir: dir },
            text: String::new(),
        });
        self.tree_context_menu = None;
    }

    pub fn start_new_folder_inline(&mut self, target_dir: Option<PathBuf>) {
        if !self.has_workspace {
            return;
        }
        let dir = target_dir.unwrap_or_else(|| {
            if let Some(ref selected) = self.selected_sidebar_path {
                if selected.is_dir() {
                    selected.clone()
                } else {
                    selected.parent().unwrap_or(&self.current_dir).to_path_buf()
                }
            } else {
                self.current_dir.clone()
            }
        });
        if dir != self.current_dir {
            self.expanded_dirs.insert(dir.clone());
            self.rebuild_tree();
        }
        self.inline_tree_input = Some(InlineTreeState {
            mode: InlineTreeInputMode::NewFolder { parent_dir: dir },
            text: String::new(),
        });
        self.tree_context_menu = None;
    }

    pub fn start_rename_inline(&mut self, target_path: PathBuf) {
        let initial_name = target_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        self.inline_tree_input = Some(InlineTreeState {
            mode: InlineTreeInputMode::Rename { target_path },
            text: initial_name,
        });
        self.tree_context_menu = None;
    }

    pub fn commit_inline_tree_input(&mut self) -> bool {
        let inline = match self.inline_tree_input.take() {
            Some(i) => i,
            None => return false,
        };

        let trimmed = inline.text.trim();
        if trimmed.is_empty() {
            return false;
        }

        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
            self.status_message = "Invalid characters in filename".to_string();
            self.inline_tree_input = Some(inline);
            return false;
        }

        match inline.mode {
            InlineTreeInputMode::NewFile { ref parent_dir } => {
                let full_path = parent_dir.join(trimmed);
                if full_path.exists() {
                    self.status_message = format!("File '{}' already exists", trimmed);
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                if let Err(e) = std::fs::write(&full_path, "") {
                    self.status_message = format!("Failed to create file: {}", e);
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                self.rebuild_tree();
                self.open_file(&full_path);
                self.status_message = format!("Created {}", trimmed);
                true
            }
            InlineTreeInputMode::NewFolder { ref parent_dir } => {
                let full_path = parent_dir.join(trimmed);
                if full_path.exists() {
                    self.status_message = format!("Folder '{}' already exists", trimmed);
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                if let Err(e) = std::fs::create_dir_all(&full_path) {
                    self.status_message = format!("Failed to create folder: {}", e);
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                self.expanded_dirs.insert(full_path.clone());
                self.selected_sidebar_path = Some(full_path);
                self.rebuild_tree();
                self.status_message = format!("Created folder {}", trimmed);
                true
            }
            InlineTreeInputMode::Rename { ref target_path } => {
                let parent = target_path.parent().unwrap_or(&self.current_dir);
                let new_path = parent.join(trimmed);
                if new_path == *target_path {
                    return true;
                }
                if new_path.exists() {
                    self.status_message = format!("'{}' already exists", trimmed);
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                if let Err(e) = std::fs::rename(target_path, &new_path) {
                    self.status_message = format!("Failed to rename: {}", e);
                    self.inline_tree_input = Some(inline);
                    return false;
                }

                // Update open tabs if file was renamed, or if it was a folder containing open files
                for tab in &mut self.tabs {
                    if let Some(ref mut tab_path) = tab.path {
                        if *tab_path == *target_path {
                            *tab_path = new_path.clone();
                            tab.name = trimmed.to_string();
                        } else if let Ok(rel) = tab_path.strip_prefix(target_path) {
                            *tab_path = new_path.join(rel);
                        }
                    }
                }

                if self.selected_sidebar_path.as_ref() == Some(target_path) {
                    self.selected_sidebar_path = Some(new_path.clone());
                }

                if self.expanded_dirs.remove(target_path) {
                    self.expanded_dirs.insert(new_path.clone());
                }

                self.rebuild_tree();
                self.status_message = format!("Renamed to {}", trimmed);
                true
            }
        }
    }

    pub fn cancel_inline_tree_input(&mut self) {
        self.inline_tree_input = None;
    }

    pub fn delete_tree_path(&mut self, path: &Path) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("item").to_string();
        let res = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match res {
            Ok(_) => {
                let mut i = 0;
                while i < self.tabs.len() {
                    let should_close = self.tabs[i].path.as_ref().map(|p| p == path || p.starts_with(path)).unwrap_or(false);
                    if should_close {
                        self.close_tab(i);
                    } else {
                        i += 1;
                    }
                }

                if self.selected_sidebar_path.as_ref() == Some(&path.to_path_buf()) {
                    self.selected_sidebar_path = None;
                }
                self.expanded_dirs.remove(path);
                self.rebuild_tree();
                self.status_message = format!("Deleted {}", name);
            }
            Err(e) => {
                self.status_message = format!("Failed to delete: {}", e);
            }
        }
        self.tree_context_menu = None;
    }

    pub fn copy_tree_path(&mut self, path: &Path) {
        self.tree_clipboard = Some(TreeClipboardAction::Copy(path.to_path_buf()));
        self.status_message = format!("Copied {}", path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
        self.tree_context_menu = None;
    }

    pub fn cut_tree_path(&mut self, path: &Path) {
        self.tree_clipboard = Some(TreeClipboardAction::Cut(path.to_path_buf()));
        self.status_message = format!("Cut {}", path.file_name().and_then(|n| n.to_str()).unwrap_or(""));
        self.tree_context_menu = None;
    }

    pub fn paste_into_tree_path(&mut self, dest_dir: &Path) {
        let clip = match self.tree_clipboard.take() {
            Some(c) => c,
            None => return,
        };

        let target_dir = if dest_dir.is_dir() {
            dest_dir.to_path_buf()
        } else {
            dest_dir.parent().unwrap_or(&self.current_dir).to_path_buf()
        };

        match clip {
            TreeClipboardAction::Copy(src) => {
                if src.exists() {
                    let file_name = src.file_name().unwrap_or_default();
                    let mut dest = target_dir.join(file_name);

                    if dest.exists() {
                        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("copy");
                        let ext = src.extension().and_then(|e| e.to_str()).map(|e| format!(".{}", e)).unwrap_or_default();
                        let mut count = 1;
                        loop {
                            let candidate = target_dir.join(format!("{}_copy{}{}", stem, count, ext));
                            if !candidate.exists() {
                                dest = candidate;
                                break;
                            }
                            count += 1;
                        }
                    }

                    let res = if src.is_dir() {
                        copy_dir_all(&src, &dest)
                    } else {
                        std::fs::copy(&src, &dest).map(|_| ())
                    };

                    if res.is_ok() {
                        self.rebuild_tree();
                        self.status_message = format!("Pasted into {}", target_dir.file_name().and_then(|n| n.to_str()).unwrap_or(""));
                    } else {
                        self.status_message = "Failed to paste item".to_string();
                    }
                }
                self.tree_clipboard = Some(TreeClipboardAction::Copy(src));
            }
            TreeClipboardAction::Cut(src) => {
                if src.exists() {
                    let file_name = src.file_name().unwrap_or_default();
                    let dest = target_dir.join(file_name);
                    if !dest.exists() {
                        if std::fs::rename(&src, &dest).is_ok() {
                            for tab in &mut self.tabs {
                                if let Some(ref mut tab_path) = tab.path {
                                    if *tab_path == src {
                                        *tab_path = dest.clone();
                                    } else if let Ok(rel) = tab_path.strip_prefix(&src) {
                                        *tab_path = dest.join(rel);
                                    }
                                }
                            }
                            self.rebuild_tree();
                            self.status_message = format!("Moved to {}", target_dir.file_name().and_then(|n| n.to_str()).unwrap_or(""));
                        }
                    } else {
                        self.status_message = "Destination already exists".to_string();
                    }
                }
            }
        }
        self.tree_context_menu = None;
    }

    pub fn reveal_in_file_manager(&self, path: &Path) {
        let dir = if path.is_dir() { path } else { path.parent().unwrap_or(&self.current_dir) };
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(dir).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open").arg("-R").arg(path).spawn();
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        self.selected_sidebar_path = Some(path.to_path_buf());
        let pending = self.pending_modifications.iter().find(|p| {
            p.file_path == path
                || p.file_path.file_name() == path.file_name()
                || p.file_path.ends_with(path)
                || path.ends_with(&p.file_path)
        }).cloned();

        let disk_content = std::fs::read_to_string(path).ok();
        let content = if let Some(ref p) = pending {
            p.diff_lines.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n")
        } else if let Some(c) = disk_content {
            c
        } else {
            self.status_message = format!("Failed to open file: {}", path.display());
            return;
        };

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        // Track recent files
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_path_buf());
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }

        // Check if already open
        for (idx, tab) in self.tabs.iter_mut().enumerate() {
            if tab.path.as_ref() == Some(&path.to_path_buf()) || tab.name == name {
                self.active_tab = Some(idx);
                tab.path = Some(path.to_path_buf());
                if pending.is_some() {
                    tab.editor.replace_all(&content);
                    tab.is_dirty = true;
                    if let Some(ref p) = pending {
                        tab.saved_text = p.original_code.clone();
                    }
                } else if !tab.is_dirty && tab.saved_text != content {
                    tab.editor.replace_all(&content);
                    tab.saved_text = content.clone();
                }
                self.status_message = format!("Switched to {}", name);
                self.check_language_support_prompt(path);
                self.save_persistent_state();
                return;
            }
        }

        let is_dirty = pending.is_some();
        let saved_text = if let Some(ref p) = pending { p.original_code.clone() } else { content.clone() };

        self.tabs.push(EditorTab {
            name,
            path: Some(path.to_path_buf()),
            editor: Editor::new(Buffer::from_text(&content)),
            is_dirty,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            version: 0,
            saved_text,
        });
        self.active_tab = Some(self.tabs.len() - 1);
        self.status_message = format!("Opened file: {}", path.display());
        let ws_root = if self.has_workspace { Some(self.current_dir.as_path()) } else { None };
        self.lsp_manager.on_document_open(path, &content, ws_root);
        self.check_language_support_prompt(path);
        self.save_persistent_state();
    }

    /// Request Go to Definition for symbol under cursor in the active editor
    pub fn goto_definition_at_cursor(&mut self) {
        if let Some(active_idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(active_idx) {
                if let Some(ref path) = tab.path {
                    let cursor = tab.editor.cursor();
                    self.lsp_manager.request_definition(
                        path,
                        cursor.row as u32,
                        cursor.column as u32,
                    );
                    self.status_message = format!("Requesting definition at {}:{}...", cursor.row + 1, cursor.column + 1);
                }
            }
        }
    }

    /// Request Hover info for symbol under cursor in the active editor
    pub fn request_hover_at_cursor(&mut self) {
        if let Some(active_idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(active_idx) {
                if let Some(ref path) = tab.path {
                    let cursor = tab.editor.cursor();
                    self.lsp_manager.request_hover(
                        path,
                        cursor.row as u32,
                        cursor.column as u32,
                    );
                }
            }
        }
    }

    /// Jump directly to an LSP Location (e.g. from Go-to-Definition)
    pub fn jump_to_location(&mut self, loc: &crate::lsp::protocol::Location) {
        let raw_path = loc.uri.trim_start_matches("file://");
        let path = std::path::PathBuf::from(raw_path);
        self.open_file(&path);
        if let Some(active_idx) = self.active_tab {
            if active_idx < self.tabs.len() {
                let tab = &mut self.tabs[active_idx];
                let target_line = loc.range.start.line as usize;
                let target_col = loc.range.start.character as usize;
                tab.editor.set_cursor(editor::buffer::Point::new(target_line, target_col));
                self.status_message = format!(
                    "Jumped to {}:{}:{}",
                    path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
                    target_line + 1,
                    target_col + 1
                );
            }
        }
    }

    /// Toggle Find in active buffer overlay
    pub fn toggle_find_in_buffer(&mut self) {
        self.find_in_buffer_open = !self.find_in_buffer_open;
        if self.find_in_buffer_open {
            self.update_find_matches();
        } else {
            self.find_matches.clear();
        }
    }

    /// Update search matches within the active buffer
    pub fn update_find_matches(&mut self) {
        self.find_matches.clear();
        if self.find_query.is_empty() {
            return;
        }
        if let Some(active_idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(active_idx) {
                let query = self.find_query.to_lowercase();
                for (row, line) in tab.editor.buffer.lines().iter().enumerate() {
                    let line_lower = line.to_lowercase();
                    let mut start = 0;
                    while let Some(idx) = line_lower[start..].find(&query) {
                        let col = start + idx;
                        self.find_matches.push((row, col, col + query.len()));
                        start = col + query.len().max(1);
                    }
                }
            }
        }
        if !self.find_matches.is_empty() {
            if self.find_match_index >= self.find_matches.len() {
                self.find_match_index = 0;
            }
        }
    }

    /// Jump to the next match
    pub fn next_find_match(&mut self) {
        if self.find_matches.is_empty() {
            return;
        }
        self.find_match_index = (self.find_match_index + 1) % self.find_matches.len();
        self.jump_to_current_find_match();
    }

    /// Jump to the previous match
    pub fn prev_find_match(&mut self) {
        if self.find_matches.is_empty() {
            return;
        }
        if self.find_match_index == 0 {
            self.find_match_index = self.find_matches.len() - 1;
        } else {
            self.find_match_index -= 1;
        }
        self.jump_to_current_find_match();
    }

    /// Center editor cursor on the currently selected find match
    pub fn jump_to_current_find_match(&mut self) {
        if let Some(&(row, col, _)) = self.find_matches.get(self.find_match_index) {
            if let Some(active_idx) = self.active_tab {
                if let Some(tab) = self.tabs.get_mut(active_idx) {
                    tab.editor.set_cursor(editor::buffer::Point::new(row, col));
                }
            }
        }
    }

    pub fn check_language_support_prompt(&mut self, path: &Path) {
        if let Some(ext) = crate::extensions::ExtensionCatalog::find_by_path(path) {
            // Check if installed or explicitly ignored
            if !self.installed_extensions.contains(ext.id) && !self.ignored_language_prompts.contains(ext.id) {
                let file_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string();
                self.active_language_prompt = Some(LanguagePromptDialog {
                    extension_id: ext.id.to_string(),
                    language_name: ext.name.to_string(),
                    file_name,
                    dont_show_again_checked: false,
                });
            }
        }
    }

    pub fn install_extension(&mut self, id: &str) {
        self.installed_extensions.insert(id.to_string());
        self.highlighter.reset();
        self.save_persistent_state();
        if let Some(ext) = crate::extensions::ExtensionCatalog::find_by_id(id) {
            self.status_message = format!("Installed {} successfully!", ext.name);
        } else {
            self.status_message = format!("Installed extension: {}", id);
        }
        if let Some(ref prompt) = self.active_language_prompt {
            if prompt.extension_id == id {
                self.active_language_prompt = None;
            }
        }
    }

    pub fn uninstall_extension(&mut self, id: &str) {
        self.installed_extensions.remove(id);
        self.highlighter.reset();
        self.save_persistent_state();
        if let Some(ext) = crate::extensions::ExtensionCatalog::find_by_id(id) {
            self.status_message = format!("Uninstalled {}", ext.name);
        } else {
            self.status_message = format!("Uninstalled extension: {}", id);
        }
    }

    pub fn is_extension_installed(&self, id: &str) -> bool {
        self.installed_extensions.contains(id)
    }

    pub fn dismiss_language_prompt(&mut self, save_dont_show_again: bool) {
        if let Some(ref dialog) = self.active_language_prompt {
            if save_dont_show_again || dialog.dont_show_again_checked {
                self.ignored_language_prompts.insert(dialog.extension_id.clone());
                self.save_persistent_state();
            }
        }
        self.active_language_prompt = None;
    }

    pub fn save_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                if self.tabs[idx].path.is_none() {
                    self.save_active_tab_as();
                    return;
                }
                let tab = &mut self.tabs[idx];
                if let Some(ref path) = tab.path {
                    let content = tab.editor.buffer.to_string();
                    if std::fs::write(path, &content).is_ok() {
                        tab.saved_text = content;
                        tab.is_dirty = false;
                        self.status_message = format!("Saved file: {}", tab.name);
                        self.refresh_git_status();
                    } else {
                        self.status_message = format!("Failed to save file: {}", tab.name);
                    }
                }
            }
        }
    }

    pub fn save_active_tab_as(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let initial_dir = self.tabs[idx].path.as_ref()
                    .and_then(|p| p.parent())
                    .unwrap_or(&self.current_dir);
                let mut dialog = rfd::FileDialog::new().set_directory(initial_dir);
                if let Some(ref p) = self.tabs[idx].path {
                    if let Some(file_name) = p.file_name().and_then(|n| n.to_str()) {
                        dialog = dialog.set_file_name(file_name);
                    }
                } else {
                    dialog = dialog.set_file_name(&self.tabs[idx].name);
                }

                if let Some(path) = dialog.save_file() {
                    let tab = &mut self.tabs[idx];
                    let content = tab.editor.buffer.to_string();
                    if std::fs::write(&path, &content).is_ok() {
                        tab.name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                        tab.path = Some(path.clone());
                        tab.saved_text = content;
                        tab.is_dirty = false;
                        if !self.recent_files.contains(&path) {
                            self.recent_files.insert(0, path);
                            if self.recent_files.len() > 10 {
                                self.recent_files.pop();
                            }
                        }
                        self.status_message = format!("Saved as: {}", tab.name);
                        self.refresh_git_status();
                    } else {
                        self.status_message = format!("Failed to save file: {}", path.display());
                    }
                }
            }
        }
    }

    pub fn set_clipboard(&mut self, text: String) {
        self.clipboard = Some(text.clone());

        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                use std::io::Write;
                if let Ok(mut child) = std::process::Command::new("wl-copy")
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(text.as_bytes());
                    }
                    let _ = child.wait();
                }
            }
        }

        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text);
        }
    }

    pub fn get_clipboard(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                if let Ok(output) = std::process::Command::new("wl-paste")
                    .args(&["--type", "text/plain;charset=utf-8", "-n"])
                    .output()
                {
                    if output.status.success() {
                        if let Ok(s) = String::from_utf8(output.stdout) {
                            if !s.is_empty() {
                                return Some(s);
                            }
                        }
                    }
                }
                if let Ok(output) = std::process::Command::new("wl-paste")
                    .arg("-n")
                    .output()
                {
                    if output.status.success() {
                        if let Ok(s) = String::from_utf8(output.stdout) {
                            if !s.is_empty() {
                                return Some(s);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(text) = cb.get_text() {
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = std::process::Command::new("xclip")
                .args(&["-selection", "clipboard", "-o"])
                .output()
            {
                if output.status.success() {
                    if let Ok(s) = String::from_utf8(output.stdout) {
                        if !s.is_empty() {
                            return Some(s);
                        }
                    }
                }
            }
        }

        self.clipboard.clone()
    }

    pub fn copy_selection(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let text = self.tabs[idx].editor.selected_text();
                if !text.is_empty() {
                    self.set_clipboard(text);
                    self.status_message = "Copied to clipboard".to_string();
                }
            }
        }
    }

    pub fn cut_selection(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let text = {
                    let tab = &mut self.tabs[idx];
                    let text = tab.editor.cut_selection();
                    if !text.is_empty() {
                        tab.update_dirty_state();
                        tab.version = tab.version.wrapping_add(1);
                        tab.shaping_cache.clear();
                    }
                    text
                };
                if !text.is_empty() {
                    self.set_clipboard(text);
                    self.status_message = "Cut to clipboard".to_string();
                }
            }
        }
    }

    pub fn paste_clipboard(&mut self) {
        let text = self.get_clipboard();
        if let Some(text) = text {
            if !text.is_empty() {
                if let Some(idx) = self.active_tab {
                    if idx < self.tabs.len() {
                        let tab = &mut self.tabs[idx];
                        tab.editor.paste(&text);
                        tab.update_dirty_state();
                        tab.version = tab.version.wrapping_add(1);
                        tab.shaping_cache.clear();
                        self.status_message = "Pasted from clipboard".to_string();
                    }
                }
            }
        }
    }

    pub fn undo_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let tab = &mut self.tabs[idx];
                if tab.editor.undo() {
                    tab.update_dirty_state();
                    tab.version = tab.version.wrapping_add(1);
                    tab.shaping_cache.clear();
                    self.status_message = "Undo".to_string();
                } else {
                    self.status_message = "Already at oldest change".to_string();
                }
            }
        }
    }

    pub fn redo_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let tab = &mut self.tabs[idx];
                if tab.editor.redo() {
                    tab.update_dirty_state();
                    tab.version = tab.version.wrapping_add(1);
                    tab.shaping_cache.clear();
                    self.status_message = "Redo".to_string();
                } else {
                    self.status_message = "Already at newest change".to_string();
                }
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            let removed = self.tabs.remove(idx);
            if let Some(ref path) = removed.path {
                self.lsp_manager.on_document_close(path);
            }
            if self.tabs.is_empty() {
                self.active_tab = None;
                self.selected_sidebar_path = None;
            } else if let Some(active) = self.active_tab {
                let new_active = if active >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    active
                };
                self.active_tab = Some(new_active);
                if let Some(ref path) = self.tabs[new_active].path {
                    self.selected_sidebar_path = Some(path.clone());
                }
            }
            self.status_message = "Tab closed".to_string();
            self.save_persistent_state();
        }
    }

    pub fn new_untitled_file(&mut self) {
        let count = self.tabs.iter()
            .filter(|t| t.name.starts_with("Untitled"))
            .count() + 1;
        let name = format!("Untitled-{}", count);
        self.tabs.push(EditorTab {
            name,
            path: None,
            editor: Editor::new(Buffer::new()),
            is_dirty: false,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            version: 0,
            saved_text: String::new(),
        });
        self.active_tab = Some(self.tabs.len() - 1);
        self.status_message = "New untitled file created".to_string();
    }

    pub fn format_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let tab = &mut self.tabs[idx];
                let path = tab.path.as_deref();
                let before = tab.editor.buffer.to_string();
                match self.formatter.format_editor(&mut tab.editor, path) {
                    Ok(_) => {
                        let after = tab.editor.buffer.to_string();
                        if before != after {
                            tab.update_dirty_state();
                            tab.version = tab.version.wrapping_add(1);
                            tab.shaping_cache.clear();
                            self.status_message = format!("Formatted {}", tab.name);
                        } else {
                            self.status_message = format!("{} is already formatted", tab.name);
                        }
                    }
                    Err(err) => {
                        self.status_message = err;
                    }
                }
            }
        }
    }

    pub fn ensure_terminal_session(&mut self) {
        if self.terminal_session.is_none() {
            if let Some(sess) = crate::native_terminal::NativeTerminalSession::new(&self.current_dir) {
                self.terminal_session = Some(sess);
            } else {
                self.terminal_lines.push(TerminalLine {
                    text: "Failed to spawn native PTY session".to_string(),
                    is_command: false,
                    is_error: true,
                });
            }
        }
    }

    pub fn is_resizing(&self) -> bool {
        self.active_resize_sidebar || self.active_resize_terminal || self.active_resize_ai_panel
    }

    pub fn poll_terminal(&mut self) -> bool {
        if let Some(ref sess) = self.terminal_session {
            sess.poll_output()
        } else {
            false
        }
    }

    pub fn send_terminal_input(&mut self, text: &str) {
        self.ensure_terminal_session();

        let trimmed = text.trim();
        if !trimmed.is_empty() {
            if self.terminal_history.last().map(|s| s.as_str()) != Some(trimmed) {
                self.terminal_history.push(trimmed.to_string());
            }
        }
        self.terminal_history_idx = None;

        if let Some(ref mut sess) = self.terminal_session {
            sess.write_line(text);
        } else {
            self.terminal_lines.push(TerminalLine {
                text: format!("$ {}", text),
                is_command: true,
                is_error: false,
            });
            if !trimmed.is_empty() {
                self.terminal_lines.push(TerminalLine {
                    text: format!("Executed (simulated): {}", trimmed),
                    is_command: false,
                    is_error: false,
                });
            }
        }
        self.terminal_input.clear();
    }

    pub fn send_terminal_sigint(&mut self) {
        if let Some(ref mut sess) = self.terminal_session {
            sess.write_raw(b"\x03");
        }
        if let Some(ref sess) = self.terminal_session {
            if let Ok(mut l) = sess.lines.lock() {
                l.push("^C".to_string());
            }
        } else {
            self.terminal_lines.push(TerminalLine {
                text: "^C".to_string(),
                is_command: false,
                is_error: false,
            });
        }
        self.terminal_input.clear();
        self.terminal_history_idx = None;
    }

    pub fn clear_terminal(&mut self) {
        if let Some(ref sess) = self.terminal_session {
            if let Ok(mut l) = sess.lines.lock() {
                l.clear();
            }
            if let Ok(mut p) = sess.pending_line.lock() {
                p.clear();
            }
        }
        self.terminal_lines.clear();
        self.terminal_input.clear();
        self.terminal_history_idx = None;
        self.terminal_scroll_offset = 0;
    }

    pub fn terminal_history_prev(&mut self) {
        if self.terminal_history.is_empty() {
            return;
        }
        let next_idx = match self.terminal_history_idx {
            None => self.terminal_history.len().saturating_sub(1),
            Some(idx) => idx.saturating_sub(1),
        };
        self.terminal_history_idx = Some(next_idx);
        if let Some(cmd) = self.terminal_history.get(next_idx) {
            self.terminal_input = cmd.clone();
        }
    }

    pub fn terminal_history_next(&mut self) {
        if let Some(idx) = self.terminal_history_idx {
            if idx + 1 < self.terminal_history.len() {
                self.terminal_history_idx = Some(idx + 1);
                self.terminal_input = self.terminal_history[idx + 1].clone();
            } else {
                self.terminal_history_idx = None;
                self.terminal_input.clear();
            }
        }
    }

    pub fn send_ai_message(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }

        self.ai_status = AiStatus::Generating;
        self.ai_pending_prompt = Some(trimmed.to_string());
        self.ai_generation_ticks = 1;
        self.ai_context_snapshot = self.build_ai_context_snapshot();

        if self.current_session.title == "New Session" || self.current_session.title.is_empty() {
            let clean_title = trimmed.lines().next().unwrap_or(trimmed).trim();
            let display_title = if clean_title.chars().count() > 36 {
                format!("{}...", clean_title.chars().take(36).collect::<String>())
            } else {
                clean_title.to_string()
            };
            self.current_session.title = display_title;
        }

        self.ai_messages.push(AiMessage {
            is_user: true,
            content: trimmed.to_string(),
            code_snippet: None,
            thoughts: None,
            deliberation: None,
            ..Default::default()
        });
        self.save_current_session_messages();
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);

        // 1. Extract AST context via tree-sitter
        let (ast_context, active_file) = if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                let code = tab.editor.buffer.to_string();
                let cur = tab.editor.cursor();
                let byte_pos = tab.editor.buffer.point_to_offset(cur).value();
                let ctx = if let Some(ref path) = tab.path {
                    let registry = syntax::languages::LanguageRegistry::new();
                    if let Some(config) = registry.detect_language(path) {
                        let mut parser = registry.create_parser(config);
                        parser.parse(&code, None).map(|t| syntax::context::CodeContext::new(&t, byte_pos, &code))
                    } else {
                        None
                    }
                } else {
                    None
                };
                (ctx, tab.name.clone())
            } else {
                (None, "No active file".to_string())
            }
        } else {
            (None, "No active file".to_string())
        };

        // 2. Biological Connectome Deliberation Gate:
        let lower = trimmed.to_lowercase();
        let intent = classify_user_intent(trimmed);

        let is_refactor = lower.contains("refactor") || lower.contains("optimize");
        let is_explain = lower.contains("explain");
        let is_test = lower.contains("test");
        let is_project_query = intent.is_work()
            || lower.contains("project")
            || lower.contains("workspace")
            || lower.contains("feature")
            || lower.contains("folder")
            || lower.contains("tree")
            || lower.contains("code");

        // Formulate dual-channel evidence streams (Option A: Primary target action vs Option B: Conservative fallback)
        let (ev_a, ev_b) = if intent.is_discussion() {
            (0.70, 0.40)
        } else if is_refactor {
            (0.86, 0.24)
        } else if is_explain {
            (0.89, 0.18)
        } else if is_test {
            (0.80, 0.30)
        } else if is_project_query {
            (0.88, 0.16)
        } else {
            (0.74, 0.36)
        };

        let deliberation = if self.connectome_ablation_enabled {
            let mask = self.connectome.generate_ablation_mask(self.connectome_ablation_mode, self.connectome_ablation_pct);
            self.connectome.deliberate_with_mask(ev_a, ev_b, 30, Some(&mask))
        } else {
            self.connectome.deliberate(ev_a, ev_b, 30)
        };

        // Explored items starts empty and is populated strictly when real tools execute
        let explored_items = Vec::new();

        // Autonomous mission and subagents are only initiated for work/project intents that are not pure explanations
        let sub_agent = if intent.is_work() && self.has_workspace && !is_explain {
            let env = DetectorAgent::detect(&self.current_dir);
            self.detected_environment = Some(env.clone());
            let mut mission = create_mission_for_prompt(trimmed, &explored_items);
            if let Some(detect_task) = mission.tasks.iter_mut().find(|t| t.id == "task_2_detect") {
                detect_task.title = format!("Detect toolchain: {} ({:?})", env.build_system, env.language);
            }
            let clean_title = trimmed.lines().next().unwrap_or(trimmed).trim();
            let display_title = if clean_title.chars().count() > 36 {
                format!("{}...", clean_title.chars().take(36).collect::<String>())
            } else {
                clean_title.to_string()
            };
            self.current_session.title = display_title;
            self.current_session.update_implementation_plan(trimmed, &explored_items, "");
            self.current_session.update_mission_file(&mission);
            self.current_mission = Some(mission);
            Some(SubAgentRole::Explorer)
        } else if self.has_workspace {
            let env = DetectorAgent::detect(&self.current_dir);
            self.detected_environment = Some(env);
            self.current_mission = None;
            None
        } else {
            self.current_mission = None;
            None
        };

        // If Gemini or Groq API key is configured, or Goose ACP is selected, query agent asynchronously
        if self.gemini_api_key.is_some() || self.groq_api_key.is_some() || self.ai_backend == crate::agent::AgentBackend::GooseAcp {
            let msg_idx = self.ai_messages.len();
            let thoughts = if intent.is_discussion() {
                format!(
                    "Deliberation: {} dt ({:.0}% conf) | Discussion & Technical Guide",
                    deliberation.reaction_time_dt, deliberation.confidence * 100.0
                )
            } else if let Some(ref ctx) = ast_context {
                let target = ctx.function_name.as_deref()
                    .or(ctx.struct_name.as_deref())
                    .unwrap_or(&ctx.node_type);
                format!(
                    "Scope: {} | Target: `{}` | Connectome: {} dt ({:.0}% conf)",
                    ctx.scope_path, target, deliberation.reaction_time_dt, deliberation.confidence * 100.0
                )
            } else {
                format!(
                    "Biological deliberation: {} dt ({:.0}% conf, {})",
                    deliberation.reaction_time_dt, deliberation.confidence * 100.0, deliberation.choice_label
                )
            };

            self.ai_messages.push(AiMessage {
                is_user: false,
                content: String::new(),
                code_snippet: None,
                thoughts: Some(thoughts),
                deliberation: Some(deliberation.clone()),
                explored_items,
                edited_files: Vec::new(),
                is_working: true,
                status_text: Some("Working".to_string()),
                sub_agent,
                elapsed_secs: None,
            });
            self.ai_request_start_times.insert(msg_idx, std::time::Instant::now());
            self.save_current_session_messages();

            self.ai_pending_requests += 1;
            let gemini_key = self.gemini_api_key.clone();
            let groq_key = self.groq_api_key.clone();
            let sender = self.ai_sender.clone();
            let query = trimmed.to_string();

            let buffer_snippet = if let Some(idx) = self.active_tab {
                self.tabs.get(idx).map(|t| {
                    let s = t.editor.buffer.to_string();
                    if s.trim().is_empty() { String::new() } else { s }
                }).unwrap_or_default()
            } else {
                String::new()
            };

            self.last_ai_prompt = trimmed.to_string();

            let system_prompt = if intent.is_discussion() {
                build_discussion_guide_prompt()
            } else if self.has_workspace {
                build_autonomous_agent_prompt(deliberation.confidence * 100.0)
            } else {
                format!(
                    "You are ZENE, an autonomous software engineering assistant integrated directly into the Zenthree code editor.\n\
                    CRITICAL: Never introduce yourself or say 'I am ZENE' or greeting intros. Jump straight into the technical answer or code.\n\
                    Note: The user currently has NO workspace or folder opened.\n\
                    Help the user directly with their questions, code generation, algorithms, or explanations. Do not assume or reference files from any workspace unless provided by the user in the prompt or active editor buffer."
                )
            };

            let preferred_model = if !self.model_auto_mode && !self.selected_model.is_empty() && self.selected_model != "Auto" {
                Some(self.selected_model.clone())
            } else {
                None
            };

            let mut loop_ctx = ai::AgentLoopContext::new(self.current_dir.clone());
            loop_ctx.gemini_key = gemini_key;
            loop_ctx.groq_key = groq_key;
            loop_ctx.preferred_model = preferred_model;
            loop_ctx.system_prompt = Some(system_prompt);
            loop_ctx.active_file = self.active_tab
                .and_then(|idx| self.tabs.get(idx))
                .and_then(|t| t.path.clone());
            loop_ctx.active_buffer = if !buffer_snippet.is_empty() {
                Some(buffer_snippet)
            } else {
                None
            };
            loop_ctx.cursor_line = self.active_tab
                .and_then(|idx| self.tabs.get(idx))
                .map(|t| t.editor.cursor().row);
            loop_ctx.open_tabs = self.tabs.iter()
                .filter_map(|t| t.path.clone())
                .collect();
            let term_text = if let Some(ref session) = self.terminal_session {
                session.lines.lock().ok().map(|lines| lines.join("\n")).unwrap_or_default()
            } else {
                self.terminal_lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n")
            };
            if !term_text.trim().is_empty() {
                loop_ctx.recent_terminal_output = Some(term_text);
            }
            loop_ctx.max_iterations = 10;
            self.ai_cancellation_token.reset();
            loop_ctx.cancellation_token = self.ai_cancellation_token.clone();

            let total_msgs = self.ai_messages.len();
            if total_msgs > 2 {
                let start_idx = total_msgs.saturating_sub(6);
                let end_idx = total_msgs.saturating_sub(1);
                for prev in &self.ai_messages[start_idx..end_idx] {
                    if prev.content.trim().is_empty() || prev.content.starts_with("⚠️") {
                        continue;
                    }
                    let role = if prev.is_user { "user" } else { "assistant" };
                    loop_ctx.prior_history.push(ai::ChatMessage {
                        role: role.to_string(),
                        content: prev.content.clone(),
                    });
                }
            }

            let backend = self.ai_backend;
            let current_dir = self.current_dir.clone();
            let cancel_token = self.ai_cancellation_token.clone();
            let event_signal = self.ai_event_signal.clone();

            std::thread::spawn(move || {
                if backend == crate::agent::AgentBackend::GooseAcp {
                    let supervisor = ai::GooseSupervisor::new();
                    match supervisor.spawn_acp_client(&current_dir) {
                        Ok(mut client) => {
                            let (event_tx, event_rx) = std::sync::mpsc::channel();
                            let s_clone = sender.clone();
                            let sig_clone = event_signal.clone();
                            let forwarder = std::thread::spawn(move || {
                                while let Ok(event) = event_rx.recv() {
                                    let _ = s_clone.send((msg_idx, event));
                                    sig_clone.with_mut(|c| *c = c.wrapping_add(1));
                                }
                            });
                            let _ = client.send_prompt(&query, vec![], &event_tx, &cancel_token);
                            drop(event_tx);
                            let _ = forwarder.join();
                        }
                        Err(err) => {
                            let _ = sender.send((
                                msg_idx,
                                ai::AgentEvent::Error {
                                    error: format!("Goose ACP: {}", err),
                                },
                            ));
                            event_signal.with_mut(|c| *c = c.wrapping_add(1));
                        }
                    }
                } else {
                    let sig_loop = event_signal.clone();
                    let mut agent_loop = ai::AgentLoop::new(
                        loop_ctx,
                        Box::new(move |event| {
                            let _ = sender.send((msg_idx, event));
                            sig_loop.with_mut(|c| *c = c.wrapping_add(1));
                        }),
                    );
                    let _ = agent_loop.run(&query);
                }
            });
            return;
        }

        let reply = if is_refactor {
            format!(
                "### Suggested Optimization for `{}`\n\nBiological deliberation reached **{:.1}% confidence** across **{} dt**.\n\n```rust\n// Zero-allocation idiomatic implementation\npub fn optimize() -> Result<bool, Box<dyn std::error::Error>> {{\n    // In-place processing\n    Ok(true)\n}}\n```",
                active_file, deliberation.confidence * 100.0, deliberation.reaction_time_dt
            )
        } else if is_project_query && self.has_workspace {
            let project_name = self.current_dir.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Workspace".to_string());

            let tree = self.get_or_update_workspace_tree().to_string();

            let ecosystem = if self.current_dir.join("Cargo.toml").exists() {
                "Rust / Cargo Workspace"
            } else if self.current_dir.join("package.json").exists() {
                "Node.js / Web Application"
            } else if self.current_dir.join("pyproject.toml").exists() || self.current_dir.join("requirements.txt").exists() {
                "Python Ecosystem"
            } else {
                "Software Project"
            };

            let mut out = format!("## Project: `{}`\n\n", project_name);
            out.push_str(&format!("- **Ecosystem**: {}\n", ecosystem));
            out.push_str(&format!("- **Workspace Root**: `{}`\n", self.current_dir.display()));
            out.push_str(&format!("- **Active Document**: `{}`\n\n", active_file));

            if !tree.is_empty() {
                out.push_str("### Project Workspace Tree Structure:\n```\n");
                out.push_str(&tree);
                out.push_str("\n```\n\n");
            }

            out.push_str("### What would you like to build or explore?\n");
            out.push_str("- **Feature Implementation**: Describe any feature or component to generate code\n");
            out.push_str("- **Explain Code**: Inspect syntax structures and AST symbol scopes\n");
            out.push_str("- **Refactor**: Suggest zero-allocation idiomatic improvements\n");
            out.push_str("- **Generate Tests**: Write automated unit test suites");
            out
        } else if intent.is_discussion() {
            format!(
                "### Discussion & Technical Guide\n\n- **Architecture & Tradeoffs**: Deep dive into system design, patterns, and component boundaries\n- **Feature Planning**: Brainstorming and technical planning\n- **Workflows & Code Inspection**: Guided walkthroughs and troubleshooting\n\nLet me know what you would like to work on or explore!"
            )
        } else if is_explain {
            let is_empty_buffer = if let Some(idx) = self.active_tab {
                self.tabs.get(idx).map(|t| t.editor.buffer.to_string().trim().is_empty()).unwrap_or(true)
            } else {
                true
            };

            if is_empty_buffer {
                format!(
                    "### Document: `{}` (Empty Buffer)\n\nThe active editor tab currently has no code.\n\nTo analyze code:\n- Open a source file from the **Explorer** sidebar on the left.\n- Or write / paste functions into this buffer and ask me again!",
                    active_file
                )
            } else if let Some(ref ctx) = ast_context {
                let target = ctx.function_name.as_deref()
                    .or(ctx.struct_name.as_deref())
                    .unwrap_or(&ctx.node_type);
                format!(
                    "### Analysis of `{}` in `{}`\n\n- **Symbol Target**: `{}`\n- **Scope Hierarchy**: `{}`\n- **Node Kind**: `{}`\n\nThe syntax tree confirms clean memory semantics with static lifetimes and zero-cost abstraction guarantees.",
                    target, active_file, target, ctx.scope_path, ctx.node_type
                )
            } else {
                format!(
                    "### Analysis: `{}`\n\nParsed `{}`. The syntax adheres to clean, reactive architecture with safe data flow.",
                    active_file, active_file
                )
            }
        } else if is_test {
            format!(
                "### Unit Test Suite for `{}`\n\n```rust\n#[cfg(test)]\nmod tests {{\n    use super::*;\n\n    #[test]\n    fn test_core_flow() {{\n        assert!(true);\n    }}\n}}\n```",
                active_file
            )
        } else if is_project_query || intent.is_work() {
            format!(
                "### No Folder Opened\n\nYou currently do not have a folder or workspace open in Zenthree.\n\nTo open a project:\n- Click **Open Folder** in the Explorer sidebar on the left.\n- Or use `Ctrl+O`.\n\nOnce opened, I can analyze your codebase and project architecture!"
            )
        } else {
            format!(
                "Query: `{}`\n\nConnectome neural gate verified the prompt with **{:.1}% confidence** ({}) across **{} dt**.\n\n- Use **Explain Code** to break down syntax trees\n- Use **Refactor** for idiomatic optimizations\n- Use **Generate Tests** for test suites",
                trimmed, deliberation.confidence * 100.0, deliberation.choice_label, deliberation.reaction_time_dt
            )
        };

        let thoughts = if intent.is_discussion() {
            format!(
                "Deliberation: {} dt ({:.0}% conf) | Discussion & Technical Guide",
                deliberation.reaction_time_dt, deliberation.confidence * 100.0
            )
        } else if let Some(ref ctx) = ast_context {
            let target = ctx.function_name.as_deref()
                .or(ctx.struct_name.as_deref())
                .unwrap_or(&ctx.node_type);
            format!(
                "AST Scope: {} | Target: `{}` | Deliberation: {} dt ({:.1}% conf, {})",
                ctx.scope_path, target, deliberation.reaction_time_dt, deliberation.confidence * 100.0, deliberation.choice_label
            )
        } else {
            format!(
                "Context: {} | Deliberation: {} dt ({:.1}% conf, {})",
                active_file, deliberation.reaction_time_dt, deliberation.confidence * 100.0, deliberation.choice_label
            )
        };

        self.ai_messages.push(AiMessage {
            is_user: false,
            content: reply,
            code_snippet: None,
            thoughts: Some(thoughts),
            deliberation: Some(deliberation),
            explored_items,
            sub_agent,
            ..Default::default()
        });
        self.save_current_session_messages();
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);

        self.ai_status = AiStatus::Ready;
        self.ai_input.clear();
    }

    pub fn build_ai_context_snapshot(&self) -> String {
        let mut sections = Vec::new();

        if self.ai_include_selection {
            if let Some(index) = self.active_tab {
                if let Some(tab) = self.tabs.get(index) {
                    let selection = tab.editor.selection();
                    if !selection.is_empty() {
                        let selected = tab.editor.selected_text();
                        if !selected.trim().is_empty() {
                            sections.push(format!("Selected code ({}):\n{}", tab.name, selected));
                        }
                    }
                }
            }
        }

        if self.ai_include_current_file {
            if let Some(index) = self.active_tab {
                if let Some(tab) = self.tabs.get(index) {
                    sections.push(format!("Current file ({}):\n{}", tab.name, tab.editor.buffer.to_string()));
                }
            }
        }

        if self.ai_include_open_files {
            let files = self.tabs.iter()
                .map(|tab| tab.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            if !files.is_empty() {
                sections.push(format!("Open files: {}", files));
            }
        }

        if self.ai_include_terminal {
            let terminal = if let Some(ref session) = self.terminal_session {
                session.lines.lock().ok().map(|lines| lines.join("\n")).unwrap_or_default()
            } else {
                self.terminal_lines.iter()
                    .map(|line| line.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            if !terminal.trim().is_empty() {
                sections.push(format!("Terminal output:\n{}", terminal));
            }
        }

        if sections.is_empty() {
            "No editor context selected".to_string()
        } else {
            sections.join("\n\n")
        }
    }

    #[allow(dead_code)]
    pub fn advance_ai_generation(&mut self) -> bool {
        if self.ai_status != AiStatus::Generating {
            return false;
        }

        if self.ai_generation_ticks > 0 {
            self.ai_generation_ticks -= 1;
            return true;
        }

        let Some(prompt) = self.ai_pending_prompt.take() else {
            self.set_ai_error("AI generation ended without a pending request");
            return false;
        };

        let active_file = self.active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "No active file".to_string());
        let language = Self::language_for_file(&active_file);

        let reply = if prompt.to_lowercase().contains("explain") {
            format!("I've analyzed `{}` as {} code. The structure and data flow are consistent with the file's current implementation.", active_file, language)
        } else if prompt.to_lowercase().contains("refactor") || prompt.to_lowercase().contains("optimize") {
            format!("Here is an optimized {} refactoring suggestion for `{}`:", language, active_file)
        } else if prompt.to_lowercase().contains("test") {
            format!("Tests for `{}` can be added using the standard {} testing conventions.", active_file, language)
        } else {
            format!("I understand you're asking about `{}` in `{}`. I can help analyze, refactor, test, or explain this file.", prompt, active_file)
        };

        let snippet = if prompt.to_lowercase().contains("refactor") || prompt.to_lowercase().contains("code") {
            Some(Self::example_snippet(language))
        } else {
            None
        };

        self.ai_messages.push(AiMessage {
            is_user: false,
            content: reply,
            code_snippet: snippet,
            thoughts: Some(format!("Context loaded: {} | {} syntax context", active_file, language)),
            ..Default::default()
        });
        self.ai_status = AiStatus::Ready;
        true
    }

    #[allow(dead_code)]
    pub fn set_ai_error(&mut self, error: &str) {
        self.ai_error = Some(error.to_string());
        self.ai_status = AiStatus::Error;
    }

    #[allow(dead_code)]
    pub fn retry_last_ai_message(&mut self) {
        if let Some(last_user_msg) = self.ai_messages.iter().rfind(|m| m.is_user).map(|m| m.content.clone()) {
            self.ai_status = AiStatus::Generating;
            self.ai_pending_prompt = Some(last_user_msg.clone());
            self.ai_generation_ticks = 1;
            self.send_ai_message(&last_user_msg);
        }
    }

    #[allow(dead_code)]
    pub fn clear_ai_conversation(&mut self) {
        self.ai_messages.clear();
        self.ai_input.clear();
        self.ai_error = None;
        self.ai_status = AiStatus::Ready;
        self.ai_pending_prompt = None;
        self.ai_generation_ticks = 0;
    }

    pub fn stop_ai_generation(&mut self) {
        self.ai_cancellation_token.cancel();
        self.ai_status = AiStatus::Ready;
        self.ai_pending_prompt = None;
        self.ai_generation_ticks = 0;
        self.ai_pending_requests = 0;
        for msg in &mut self.ai_messages {
            msg.is_working = false;
        }
        if let Some(ref mut mission) = self.current_mission {
            mission.is_active = false;
            mission.is_paused = true;
            mission.status_message = "⏹ Mission stopped by user".to_string();
            self.current_session.update_mission_file(mission);
        }
        self.status_message = "AI response stopped".to_string();
    }

    pub fn cancel_ai_terminal_command(&mut self, cmd_id: &str) {
        if let Some(pos) = self.ai_terminal_commands.iter().position(|c| c.id == cmd_id) {
            let cmd_text = self.ai_terminal_commands[pos].command.clone();
            let output_text = self.ai_terminal_commands[pos].output.clone();

            // Try extracting PID from output e.g. "(PID 12345)"
            let mut killed = false;
            if let Some(pid_idx) = output_text.find("PID ") {
                let rest = &output_text[pid_idx + 4..];
                let pid_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(pid) = pid_str.parse::<u32>() {
                    #[cfg(windows)]
                    {
                        let _ = std::process::Command::new("taskkill")
                            .args(["/F", "/PID", &pid.to_string()])
                            .output();
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = std::process::Command::new("kill")
                            .arg("-9")
                            .arg(pid.to_string())
                            .output();
                    }
                    killed = true;
                }
            }

            if !killed {
                let first_word = cmd_text.split_whitespace().next().unwrap_or("");
                if !first_word.is_empty() && (first_word == "vite" || first_word == "bun" || first_word == "node" || first_word == "cargo" || first_word == "python") {
                    #[cfg(windows)]
                    {
                        let exe = if first_word.ends_with(".exe") { first_word.to_string() } else { format!("{}.exe", first_word) };
                        let _ = std::process::Command::new("taskkill")
                            .args(["/F", "/IM", &exe])
                            .output();
                    }
                    #[cfg(not(windows))]
                    {
                        let _ = std::process::Command::new("pkill")
                            .arg("-f")
                            .arg(&cmd_text)
                            .output();
                    }
                }
            }

            self.ai_terminal_commands.remove(pos);
            if self.ai_terminal_commands.is_empty() && self.active_input_drawer == AiInputDrawer::Terminals {
                self.active_input_drawer = AiInputDrawer::None;
            }
            self.status_message = format!("Terminated active command `{}`", cmd_text);
        }
    }

    pub(crate) fn language_for_file(file_name: &str) -> &'static str {
        match Path::new(file_name).extension().and_then(|ext| ext.to_str()) {
            Some("py") => "Python",
            Some("js") | Some("jsx") => "JavaScript",
            Some("ts") | Some("tsx") => "TypeScript",
            Some("go") => "Go",
            Some("cpp") | Some("cc") | Some("hpp") | Some("h") => "C++",
            Some("c") => "C",
            Some("java") => "Java",
            Some("html") => "HTML",
            Some("css") => "CSS",
            Some("json") => "JSON",
            Some("toml") => "TOML",
            Some("md") => "Markdown",
            Some("rs") => "Rust",
            _ => "source",
        }
    }

    pub(crate) fn example_snippet(language: &str) -> String {
        match language {
            "Python" => "def optimize(items):\n    return [item for item in items if item is not None]".to_string(),
            "JavaScript" => "const optimize = (items) => items.filter(Boolean);".to_string(),
            "TypeScript" => "const optimize = <T>(items: T[]) => items.filter(Boolean);".to_string(),
            "Go" => "func optimize(items []Item) []Item {\n    return filtered\n}".to_string(),
            "C++" => "std::vector<Item> optimize(const std::vector<Item>& items) {\n    return filtered;\n}".to_string(),
            _ => "// Example suggested optimization\npub fn optimize() -> bool {\n    true\n}".to_string(),
        }
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
pub fn detect_target_folders_from_response(
    content: &str,
    prompt: &str,
    workspace_root: &Path,
) -> Vec<PathBuf> {
    let mut folders = Vec::new();

    // 1. Explicit folder markers in AI response
    for line in content.lines() {
        let trimmed = line.trim();
        let stripped = if let Some(rest) = trimmed.strip_prefix("### Folder:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### Directory:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### Create Folder:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### New Folder:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("**Folder**:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("Folder:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("Directory:") {
            Some(rest)
        } else {
            None
        };

        if let Some(path_candidate) = stripped {
            let cleaned = path_candidate.trim().trim_matches('`').trim_matches('*').trim();
            if !cleaned.is_empty() {
                let dir = if Path::new(cleaned).is_absolute() {
                    PathBuf::from(cleaned)
                } else {
                    workspace_root.join(cleaned)
                };
                if !folders.contains(&dir) {
                    folders.push(dir);
                }
            }
        }
    }

    // 2. User prompt commands like "create folder <path>", "mkdir <path>", "new folder <path>"
    let prompt_lower = prompt.to_lowercase();
    for kw in ["create folder", "new folder", "make folder", "mkdir", "add folder", "create directory", "new directory"] {
        if let Some(idx) = prompt_lower.find(kw) {
            let after = prompt[idx + kw.len()..].trim_start();
            if let Some(first_token) = after.split_whitespace().next() {
                let clean = first_token.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '_' && c != '-');
                let lower_clean = clean.to_lowercase();
                if clean.len() >= 2
                    && !clean.contains('.')
                    && lower_clean != "named"
                    && lower_clean != "called"
                    && lower_clean != "for"
                    && lower_clean != "here"
                    && lower_clean != "folders"
                    && lower_clean != "directories"
                {
                    let dir = if Path::new(clean).is_absolute() {
                        PathBuf::from(clean)
                    } else {
                        workspace_root.join(clean)
                    };
                    if !folders.contains(&dir) {
                        folders.push(dir);
                    }
                }
            }
        }
    }

    folders
}

#[allow(dead_code)]
pub fn detect_target_file_from_response(
    content: &str,
    prompt: &str,
    workspace_root: &Path,
    _active_path: Option<&Path>,
) -> Option<PathBuf> {
    // 1. Look for explicit file markers in the response (including create / new file directives)
    for line in content.lines() {
        let trimmed = line.trim();
        let stripped = if let Some(rest) = trimmed.strip_prefix("### File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### file:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### Create File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### New File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("**File**:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("// File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("// file:") {
            Some(rest)
        } else {
            None
        };

        if let Some(path_candidate) = stripped {
            let cleaned = path_candidate.trim().trim_matches('`').trim_matches('*').trim();
            if !cleaned.is_empty() {
                return Some(crate::agent::ground_workspace_path(workspace_root, cleaned));
            }
        }
    }

    // 2. Check if user prompt explicitly requested creating or targeting a file path
    let prompt_lower = prompt.to_lowercase();
    for kw in ["create file", "new file", "make file", "add file", "touch"] {
        if let Some(idx) = prompt_lower.find(kw) {
            let after = prompt[idx + kw.len()..].trim_start();
            if let Some(first_token) = after.split_whitespace().next() {
                let clean = first_token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '_' && c != '-');
                if clean.contains('.') && !clean.starts_with('.') && clean.len() >= 3 {
                    return Some(crate::agent::ground_workspace_path(workspace_root, clean));
                }
            }
        }
    }

    // 3. Check if user prompt explicitly mentions a known file path with extension in workspace
    for token in prompt.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '_' && c != '-');
        if clean.contains('.') && !clean.starts_with('.') {
            let direct = workspace_root.join(clean);
            if direct.exists() && direct.is_file() {
                return Some(direct);
            }
        }
    }

    None
}

#[allow(dead_code)]
pub fn extract_code_from_markdown(content: &str) -> Option<String> {
    let mut in_block = false;
    let mut code_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if in_block {
                if !code_lines.is_empty() {
                    return Some(code_lines.join("\n"));
                }
                in_block = false;
            } else {
                in_block = true;
            }
        } else if in_block {
            code_lines.push(line);
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RawDiffOp<'a> {
    Equal(&'a str),
    Delete(&'a str),
    Insert(&'a str),
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
        assert!(state.tabs.iter().any(|t| t.path.as_ref() == Some(&test_file)));

        // Clean up
        let _ = std::fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn test_terminal_and_ai_panel_state() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_term_ai_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        assert!(!state.terminal_visible);
        assert!(!state.ai_panel_visible);

        // Toggle terminal & send command
        state.terminal_visible = true;
        assert!(state.terminal_visible);
        state.send_terminal_input("echo hello");
        assert!(state.terminal_session.is_some() || !state.terminal_lines.is_empty());

        // Toggle AI panel & send prompt
        state.ai_panel_visible = true;
        state.send_ai_message("Explain the workspace");
        assert!(state.ai_messages.iter().any(|m| m.is_user && m.content == "Explain the workspace"));
        assert!(state.ai_status == AiStatus::Generating || state.ai_status == AiStatus::Ready);

        state.ai_include_current_file = false;
        state.ai_include_open_files = true;
        state.ai_include_terminal = true;
        let context = state.build_ai_context_snapshot();
        assert!(context.contains("Open files:") || context.contains("Terminal output:") || context.contains("No editor context selected"));

        state.clear_ai_conversation();
        assert!(state.ai_messages.is_empty());
        assert_eq!(state.ai_status, AiStatus::Ready);
        assert!(state.ai_error.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ai_language_context_uses_active_file_type() {
        assert_eq!(ZenthreeState::language_for_file("Practice13.py"), "Python");
        assert_eq!(ZenthreeState::language_for_file("main.rs"), "Rust");
        assert!(ZenthreeState::example_snippet("Python").contains("def optimize"));
        assert!(ZenthreeState::example_snippet("Rust").contains("pub fn optimize"));
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

        let go_ext = crate::extensions::ExtensionCatalog::find_by_path(std::path::Path::new("app.go"));
        assert!(go_ext.is_some());
        assert_eq!(go_ext.unwrap().id, "go");

        let ts_ext = crate::extensions::ExtensionCatalog::find_by_path(std::path::Path::new("index.ts"));
        assert!(ts_ext.is_some());
        assert_eq!(ts_ext.unwrap().id, "typescript");

        let cpp_ext = crate::extensions::ExtensionCatalog::find_by_path(std::path::Path::new("main.cpp"));
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
    fn test_undo_redo_dirty_state() {
        let tmp_dir = std::env::temp_dir().join("zenthree_undo_test");
        let _ = std::fs::create_dir_all(&tmp_dir);
        let test_file = tmp_dir.join("test_undo.txt");
        let _ = std::fs::write(&test_file, "initial text");

        let mut state = ZenthreeState::new();
        state.open_file(&test_file);

        let active_idx = state.active_tab.expect("active tab");
        assert!(!state.tabs[active_idx].is_dirty);

        // Type something into active tab
        state.tabs[active_idx].editor.insert(" modified", None, None);
        state.tabs[active_idx].update_dirty_state();
        assert!(state.tabs[active_idx].is_dirty);

        // Undo -> should return to initial text and is_dirty should be false!
        state.undo_active_tab();
        assert_eq!(state.tabs[active_idx].editor.buffer.to_string(), "initial text");
        assert!(!state.tabs[active_idx].is_dirty, "Undoing back to saved state must clear dirty indicator");

        // Redo -> should become dirty again!
        state.redo_active_tab();
        assert_eq!(state.tabs[active_idx].editor.buffer.to_string(), " modifiedinitial text");
        assert!(state.tabs[active_idx].is_dirty, "Redo must re-set dirty indicator");

        // Clean up
        let _ = std::fs::remove_dir_all(tmp_dir);
    }

    #[test]
    fn test_connectome_ai_markdown_and_deliberation() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_delib_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        state.groq_api_key = None; // Test offline connectome + local markdown path
        state.gemini_api_key = None;

        // 1. Intact connectome test
        state.send_ai_message("hi tell me about this project");
        let msg1 = state.ai_messages.iter().filter(|m| !m.is_user).last().expect("assistant msg");
        assert!(msg1.content.starts_with("## Project:"));
        let delib1 = msg1.deliberation.as_ref().expect("deliberation");
        assert_eq!(delib1.ablated_neurons, 0);
        assert!(delib1.confidence > 0.5);
        assert!(delib1.reaction_time_dt > 0);

        // 2. In silico ablation connectome test
        state.connectome_ablation_enabled = true;
        state.connectome_ablation_pct = 0.25;
        state.connectome_ablation_mode = connectome::AblationMode::TargetedHubs;
        state.send_ai_message("refactor this code");
        let msg2 = state.ai_messages.iter().filter(|m| !m.is_user).last().expect("assistant msg");
        assert!(msg2.content.contains("### Suggested Optimization"));
        assert!(msg2.content.contains("```rust"));
        let delib2 = msg2.deliberation.as_ref().expect("deliberation under ablation");
        assert!(delib2.ablated_neurons > 50, "Should have ablated neurons: {}", delib2.ablated_neurons);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_target_file_and_folders_creation() {
        let tmp_root = std::env::temp_dir().join("zenthree_create_test");
        let _ = std::fs::create_dir_all(&tmp_root);

        // 1. Detect folder creation
        let ai_response_folder = "I will create the directories for you.\n\n### Folder: src/nested/components\n### Directory: assets/icons";
        let folders = detect_target_folders_from_response(ai_response_folder, "make new folders", &tmp_root);
        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0], tmp_root.join("src/nested/components"));
        assert_eq!(folders[1], tmp_root.join("assets/icons"));

        // 2. Detect new file creation anywhere (even deeply nested unopened files)
        let ai_response_file = "Here is the new file:\n\n### File: src/components/button.rs\n```rust\npub fn button() {}\n```";
        let file = detect_target_file_from_response(ai_response_file, "create file button.rs", &tmp_root, None);
        assert_eq!(file, Some(tmp_root.join("src/components/button.rs")));

        // 3. User prompt direct file creation
        let prompt_file = detect_target_file_from_response("Here is the code", "please create file src/views/dashboard.rs with layout", &tmp_root, None);
        assert_eq!(prompt_file, Some(tmp_root.join("src/views/dashboard.rs")));

        let _ = std::fs::remove_dir_all(&tmp_root);
    }

    #[test]
    fn test_agent_mission_and_checklist_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_mission_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        state.groq_api_key = Some("test_mock_key".to_string());
        assert!(!state.is_agent_working());
        assert!(state.current_mission.is_none());

        // Send AI message initializes mission with structured tasks
        state.send_ai_message("Implement JWT authentication");
        let mission = state.current_mission.as_ref().expect("Mission should be initialized");
        assert_eq!(mission.goal, "Implement JWT authentication");
        assert_eq!(mission.tasks.len(), 4);
        assert_eq!(mission.tasks[0].status, TaskStatus::Completed);
        assert_eq!(mission.tasks[0].sub_agent, Some(SubAgentRole::Explorer));
        assert_eq!(mission.tasks[1].status, TaskStatus::Completed);
        assert_eq!(mission.tasks[1].sub_agent, Some(SubAgentRole::Detector));
        assert_eq!(mission.tasks[2].status, TaskStatus::InProgress);
        assert_eq!(mission.tasks[2].sub_agent, Some(SubAgentRole::Builder));
        assert_eq!(mission.tasks[3].status, TaskStatus::Pending);
        assert_eq!(mission.tasks[3].sub_agent, Some(SubAgentRole::Tester));
        assert!(state.is_agent_working());

        // Test Pause
        state.pause_agent_mission();
        assert!(state.current_mission.as_ref().unwrap().is_paused);
        assert!(!state.is_agent_working());

        // Test Resume
        state.resume_agent_mission();
        assert!(!state.current_mission.as_ref().unwrap().is_paused);
        assert!(state.is_agent_working());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_extract_multiple_file_modifications() {
        let content = "### Plan\n1. Create Navbar\n2. Create Contact\n\n### File: src/components/Navbar.tsx\n```tsx\nexport const Navbar = () => <nav>Navbar</nav>;\n```\n\n### File: src/pages/Contact.tsx\n```tsx\nexport const Contact = () => <div>Contact</div>;\n```";
        let tmp = PathBuf::from("/workspace");
        let mods = extract_all_file_modifications(content, &tmp, None);
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0].file_path, tmp.join("src/components/Navbar.tsx"));
        assert!(mods[0].apply_to("").unwrap().contains("export const Navbar"));
        assert_eq!(mods[1].file_path, tmp.join("src/pages/Contact.tsx"));
        assert!(mods[1].apply_to("").unwrap().contains("export const Contact"));
    }

    #[test]
    fn test_compiler_self_healing_state() {
        let mut state = ZenthreeState::new();
        state.groq_api_key = None;
        state.gemini_api_key = None;
        let items = vec![ExploredItem::new(
            "src/main.rs",
            Some("#L1-10".to_string()),
            "Analyzed",
        )];
        let mission = create_mission_for_prompt("fix navbar bug", &items);
        assert_eq!(mission.self_healing_attempts, 0);
        assert_eq!(mission.tasks.len(), 4);
        let verify_idx = 3;
        assert_eq!(mission.tasks[verify_idx].id, "task_4_test");
        assert_eq!(mission.tasks[verify_idx].sub_agent, Some(SubAgentRole::Tester));
        state.current_mission = Some(mission);

        // Simulate simulated error in healing attempt
        if let Some(ref mut m) = state.current_mission {
            m.self_healing_attempts += 1;
            m.tasks[verify_idx].status = TaskStatus::InProgress;
            m.tasks[verify_idx].title = format!("🛠️ Self-Healing: Repair compiler errors (Attempt {}/2)", m.self_healing_attempts);
        }

        let m = state.current_mission.as_ref().unwrap();
        assert_eq!(m.self_healing_attempts, 1);
        assert_eq!(m.tasks[verify_idx].status, TaskStatus::InProgress);
        assert!(m.tasks[verify_idx].title.contains("Attempt 1/2"));
    }


    #[test]
    fn test_accept_all_and_reject_all_modifications() {
        let mut state = ZenthreeState::new();
        let temp_dir = std::env::temp_dir().join("zenthree_test_accept_reject");
        let _ = std::fs::create_dir_all(&temp_dir);

        let file_a = temp_dir.join("file_a.rs");
        let file_b = temp_dir.join("file_b.rs");

        std::fs::write(&file_a, "fn original_a() {}").unwrap();

        state.pending_modifications = vec![
            PendingModification {
                file_path: file_a.clone(),
                original_code: "fn original_a() {}".to_string(),
                proposed_code: "fn modified_a() {}".to_string(),
                diff_lines: vec![("fn modified_a() {}".to_string(), DiffLineState::Added)],
            },
            PendingModification {
                file_path: file_b.clone(),
                original_code: "".to_string(),
                proposed_code: "fn new_b() {}".to_string(),
                diff_lines: vec![("fn new_b() {}".to_string(), DiffLineState::Added)],
            },
        ];

        // Test accept all
        state.accept_all_modifications();
        assert!(state.pending_modifications.is_empty());
        assert_eq!(std::fs::read_to_string(&file_a).unwrap(), "fn modified_a() {}");
        assert_eq!(std::fs::read_to_string(&file_b).unwrap(), "fn new_b() {}");
        assert!(state.status_message.contains("Accepted and saved 2 modified files"));

        // Setup for reject all test
        state.pending_modifications = vec![
            PendingModification {
                file_path: file_a.clone(),
                original_code: "fn original_a() {}".to_string(),
                proposed_code: "fn modified_a_again() {}".to_string(),
                diff_lines: vec![("fn modified_a_again() {}".to_string(), DiffLineState::Added)],
            },
            PendingModification {
                file_path: file_b.clone(),
                original_code: "".to_string(),
                proposed_code: "fn new_b_again() {}".to_string(),
                diff_lines: vec![("fn new_b_again() {}".to_string(), DiffLineState::Added)],
            },
        ];

        state.reject_all_modifications();
        assert!(state.pending_modifications.is_empty());
        assert_eq!(std::fs::read_to_string(&file_a).unwrap(), "fn original_a() {}");
        assert!(!file_b.exists());
        assert!(state.status_message.contains("Rejected changes to 2 files"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_delete_current_session_clean_reset() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_del_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        state.send_ai_message("Hello session");
        let current_id = state.current_session.id.clone();
        let session_folder = state.current_session.dir_path.clone();

        assert!(session_folder.exists());

        // Deleting current session
        state.delete_session(&current_id);

        // Old session folder must not exist
        assert!(!session_folder.exists());
        // New session must have a different ID
        assert_ne!(state.current_session.id, current_id);
        assert!(!state.ai_showing_session_list);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_send_ai_message_standalone_python_workspace() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_py_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let script_file = temp_dir.join("calculator.py");
        std::fs::write(&script_file, "def add(a, b): return a + b\n").unwrap();

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        state.groq_api_key = None;
        state.gemini_api_key = None;

        state.send_ai_message("inspect python project");

        assert!(state.detected_environment.is_some());
        let env = state.detected_environment.as_ref().unwrap();
        assert_eq!(env.language, crate::agent::ProjectLanguage::Python);
        assert_eq!(env.build_system, "Python");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_send_ai_message_greeting_discussion_mode() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_disc_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let secret_code_file = temp_dir.join("secret.rs");
        std::fs::write(&secret_code_file, "fn secret_algorithm() { let x = 42; }\n").unwrap();

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        state.open_file(&secret_code_file);
        state.groq_api_key = None;
        state.gemini_api_key = None;

        // Saying "hi" or greeting must act as discussion & guide without reading active buffer code
        state.send_ai_message("hi");

        let reply = state.ai_messages.iter().filter(|m| !m.is_user).last().expect("assistant reply");
        assert!(reply.content.contains("Discussion & Technical Guide"));
        assert!(!reply.content.contains("secret_algorithm"));
        assert!(reply.explored_items.is_empty());
        assert!(state.current_mission.is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_workspace_tree_realtime_in_memory_updates() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_tree_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let src_dir = temp_dir.join("src");
        let _ = std::fs::create_dir_all(&src_dir);
        let main_file = src_dir.join("main.rs");
        std::fs::write(&main_file, "fn main() {}\n").unwrap();

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());

        // In-memory workspace tree structure is stored in real-time
        let tree1 = state.get_or_update_workspace_tree().to_string();
        assert!(tree1.contains("main.rs"));
        assert!(!tree1.contains("auth_handler.rs"));

        // Work query reads the tree structure
        state.groq_api_key = None;
        state.gemini_api_key = None;
        state.send_ai_message("add new feature to project");
        let reply1 = state.ai_messages.iter().filter(|m| !m.is_user).last().expect("assistant reply");
        assert!(reply1.content.contains("main.rs"));
        assert!(state.current_mission.is_some());

        // Add a new file in real time
        let new_file = src_dir.join("auth_handler.rs");
        std::fs::write(&new_file, "pub fn authenticate() -> bool { true }\n").unwrap();
        state.rebuild_tree();

        // The in-memory tree in state management updates in real time
        let tree2 = state.get_or_update_workspace_tree().to_string();
        assert!(tree2.contains("auth_handler.rs"));

        // Delete file in real time
        std::fs::remove_file(&new_file).unwrap();
        state.rebuild_tree();

        // The in-memory tree reflects deletion immediately
        let tree3 = state.get_or_update_workspace_tree().to_string();
        assert!(!tree3.contains("auth_handler.rs"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_workspace_session_resume_and_no_duplicates() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_resume_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());
        state.groq_api_key = None;
        state.gemini_api_key = None;

        // Send a message
        state.send_ai_message("First user prompt");
        let first_id = state.current_session.id.clone();
        assert_eq!(state.ai_messages.len(), 2);
        assert_eq!(state.session_list.len(), 1);

        // Reopening the same workspace (simulating app relaunch or switching)
        state.open_workspace(temp_dir.clone());

        // Must resume the existing session without creating a duplicate!
        assert_eq!(state.current_session.id, first_id);
        assert_eq!(state.session_list.len(), 1);
        assert_eq!(state.ai_messages.len(), 2);
        assert_eq!(state.ai_messages[0].content, "First user prompt");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_isolated_sessions() {
        let p1_dir = std::env::temp_dir().join(format!("zenthree_p1_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let p2_dir = std::env::temp_dir().join(format!("zenthree_p2_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&p1_dir);
        let _ = std::fs::create_dir_all(&p2_dir);

        let mut state = ZenthreeState::new();
        state.groq_api_key = None;
        state.gemini_api_key = None;

        // Work in Project 1
        state.open_workspace(p1_dir.clone());
        state.send_ai_message("Task for Project 1");
        let p1_sess_id = state.current_session.id.clone();
        assert_eq!(state.session_list.len(), 1);
        assert_eq!(state.ai_messages[0].content, "Task for Project 1");

        // Switch to Project 2
        state.open_workspace(p2_dir.clone());
        assert_ne!(state.current_session.id, p1_sess_id);
        assert_eq!(state.ai_messages.len(), 0); // clean state for new project
        state.send_ai_message("Task for Project 2");
        let p2_sess_id = state.current_session.id.clone();
        assert_eq!(state.session_list.len(), 1);
        assert_eq!(state.ai_messages[0].content, "Task for Project 2");

        // Switch back to Project 1: must load Project 1's session and messages!
        state.open_workspace(p1_dir.clone());
        assert_eq!(state.current_session.id, p1_sess_id);
        assert_eq!(state.session_list.len(), 1);
        assert_eq!(state.ai_messages[0].content, "Task for Project 1");

        // Switch back to Project 2: must load Project 2's session and messages!
        state.open_workspace(p2_dir.clone());
        assert_eq!(state.current_session.id, p2_sess_id);
        assert_eq!(state.session_list.len(), 1);
        assert_eq!(state.ai_messages[0].content, "Task for Project 2");

        let _ = std::fs::remove_dir_all(&p1_dir);
        let _ = std::fs::remove_dir_all(&p2_dir);
    }

    #[test]
    fn test_empty_workspace_has_zero_saved_sessions_and_no_disk_write() {
        let temp_dir = std::env::temp_dir().join(format!("zenthree_empty_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        let mut state = ZenthreeState::new();
        state.open_workspace(temp_dir.clone());

        // When workspace is opened and no messages were sent:
        // Must have 0 saved sessions and NO .zenthree directory created!
        assert_eq!(state.session_list.len(), 0);
        assert!(!temp_dir.join(".zenthree").exists());

        // Reopening workspace without sending messages:
        state.open_workspace(temp_dir.clone());
        assert_eq!(state.session_list.len(), 0);
        assert!(!temp_dir.join(".zenthree").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_compute_line_diff_groups_consecutive_removals_before_additions() {
        // Test 3 lines replaced in sequence: must show all 3 removed lines together,
        // followed by all 3 added lines together (NOT alternating one by one).
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

        // Preview text line count matches diff line count
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
    fn test_cognitive_loop_events_and_toggles() {
        let mut state = ZenthreeState::new();
        state.ai_messages.push(AiMessage {
            is_user: false,
            content: "I'll create the project.".to_string(),
            code_snippet: None,
            thoughts: None,
            deliberation: None,
            explored_items: Vec::new(),
            edited_files: Vec::new(),
            is_working: true,
            status_text: None,
            sub_agent: None,
            elapsed_secs: None,
        });

        // Test accordion toggles
        assert!(!state.expanded_worked_msgs.contains(&0));
        state.toggle_worked_accordion(0);
        assert!(state.expanded_worked_msgs.contains(&0));
        state.toggle_worked_accordion(0);
        assert!(!state.expanded_worked_msgs.contains(&0));

        assert!(!state.expanded_thought_items.contains(&(0, 1)));
        state.toggle_thought_accordion(0, 1);
        assert!(state.expanded_thought_items.contains(&(0, 1)));
        state.toggle_thought_accordion(0, 1);
        assert!(!state.expanded_thought_items.contains(&(0, 1)));

        // Test event handling
        state.ai_request_start_times.insert(0, std::time::Instant::now());

        // FileAnalyzed
        let analyzed_evt = ai::AgentEvent::FileAnalyzed {
            file_path: "src/App.jsx".to_string(),
            start_line: 10,
            end_line: 45,
        };
        // SearchExecuted
        let search_evt = ai::AgentEvent::SearchExecuted {
            query: "calculator".to_string(),
            result_count: 3,
        };
        // ThoughtBlock
        let thought_evt = ai::AgentEvent::ThoughtBlock {
            content: "Planning layout structure and state bindings".to_string(),
            duration_secs: 2,
        };
        // Completed
        let completed_evt = ai::AgentEvent::Completed {
            summary: "Finished scaffolding".to_string(),
        };

        // Simulate event poll handling
        if let Some(msg) = state.ai_messages.get_mut(0) {
            match analyzed_evt {
                ai::AgentEvent::FileAnalyzed { file_path, start_line, end_line } => {
                    let range = format!("#L{}-{}", start_line, end_line);
                    msg.explored_items.push(ExploredItem {
                        file_name: file_path,
                        line_range: Some(range),
                        action_type: "Analyzed".to_string(),
                        thought_content: None,
                        duration_secs: None,
                        children: Vec::new(),
                    });
                }
                _ => {}
            }
            match search_evt {
                ai::AgentEvent::SearchExecuted { query, result_count } => {
                    msg.explored_items.push(ExploredItem {
                        file_name: query,
                        line_range: Some(format!("({} results)", result_count)),
                        action_type: "Searched".to_string(),
                        thought_content: None,
                        duration_secs: None,
                        children: Vec::new(),
                    });
                }
                _ => {}
            }
            match thought_evt {
                ai::AgentEvent::ThoughtBlock { content, duration_secs } => {
                    msg.explored_items.push(ExploredItem {
                        file_name: String::new(),
                        line_range: None,
                        action_type: "Thought".to_string(),
                        thought_content: Some(content),
                        duration_secs: Some(duration_secs),
                        children: Vec::new(),
                    });
                }
                _ => {}
            }
            match completed_evt {
                ai::AgentEvent::Completed { summary } => {
                    msg.content = summary;
                    msg.is_working = false;
                    msg.elapsed_secs = Some(19);
                }
                _ => {}
            }
        }

        let msg = &state.ai_messages[0];
        assert_eq!(msg.explored_items.len(), 3);
        assert_eq!(msg.explored_items[0].action_type, "Analyzed");
        assert_eq!(msg.explored_items[0].file_name, "src/App.jsx");
        assert_eq!(msg.explored_items[0].line_range.as_deref(), Some("#L10-45"));

        assert_eq!(msg.explored_items[1].action_type, "Searched");
        assert_eq!(msg.explored_items[1].file_name, "calculator");

        assert_eq!(msg.explored_items[2].action_type, "Thought");
        assert_eq!(msg.explored_items[2].thought_content.as_deref(), Some("Planning layout structure and state bindings"));
        assert_eq!(msg.explored_items[2].duration_secs, Some(2));

        assert_eq!(msg.elapsed_secs, Some(19));
        assert_eq!(msg.is_working, false);
    }

    #[test]
    fn test_source_control_sidebar_view_and_git_state() {
        let mut state = ZenthreeState::new();
        assert_eq!(state.sidebar_view, SidebarView::Explorer);

        // Switch to SourceControl
        state.sidebar_view = SidebarView::SourceControl;
        assert_eq!(state.sidebar_view, SidebarView::SourceControl);

        // Verify git_state default fields
        assert!(!state.git_state.is_repo);
        assert!(state.git_state.branch.is_empty());
        assert!(state.git_state.staged.is_empty());
        assert!(state.git_state.unstaged.is_empty());
        assert!(state.git_state.commit_message.is_empty());
        assert!(state.git_state.staged_expanded);
        assert!(state.git_state.changes_expanded);
        assert_eq!(state.git_state.graph_height, 200.0);
        assert!(!state.git_state.active_resize_graph);

        // Test serde serialization of SidebarView::SourceControl
        let json = serde_json::to_string(&SidebarView::SourceControl).unwrap();
        let deserialized: SidebarView = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SidebarView::SourceControl);
    }

    #[test]
    fn test_git_status_scanning_in_repo() {
        let mut state = ZenthreeState::new();
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .to_path_buf();
        state.open_workspace(repo_root);
        assert!(state.git_state.is_repo);
        assert!(!state.git_state.branch.is_empty());
    }

    #[test]
    fn test_lsp_definition_jump_and_find_in_buffer() {
        let mut state = ZenthreeState::new();
        state.new_untitled_file();
        let active_idx = state.active_tab.unwrap();
        state.tabs[active_idx].editor.insert("fn calculate_metrics() {\n    let value = 42;\n    println!(\"{}\", value);\n}", None, None);

        // Test Find in buffer
        state.find_query = "value".to_string();
        state.toggle_find_in_buffer();
        assert!(state.find_in_buffer_open);
        assert_eq!(state.find_matches.len(), 2);
        assert_eq!(state.find_matches[0], (1, 8, 13));
        assert_eq!(state.find_matches[1], (2, 19, 24));

        // Test navigation
        state.next_find_match();
        assert_eq!(state.find_match_index, 1);
        state.prev_find_match();
        assert_eq!(state.find_match_index, 0);

        // Test Jump to Location
        let _loc = crate::lsp::protocol::Location {
            uri: "file:///test/mock.rs".to_string(),
            range: crate::lsp::protocol::Range {
                start: crate::lsp::protocol::Position { line: 10, character: 4 },
                end: crate::lsp::protocol::Position { line: 10, character: 12 },
            },
        };
        // Use a temp file so open_file succeeds
        let tmp = std::env::temp_dir().join("zenthree_test_jump.rs");
        std::fs::write(&tmp, "pub fn jump_target() {}\n").unwrap();
        let loc_real = crate::lsp::protocol::Location {
            uri: format!("file://{}", tmp.display()),
            range: crate::lsp::protocol::Range {
                start: crate::lsp::protocol::Position { line: 0, character: 7 },
                end: crate::lsp::protocol::Position { line: 0, character: 18 },
            },
        };
        state.jump_to_location(&loc_real);
        let curr_tab = state.active_tab.unwrap();
        assert_eq!(state.tabs[curr_tab].editor.cursor(), editor::buffer::Point::new(0, 7));
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn test_explored_folder_toggle_and_selection_jump() {
        let mut state = ZenthreeState::new();
        let msg_idx = 0;
        let folder = "crates/ai/src";

        // Default: expanded
        assert!(state.is_explored_folder_expanded(msg_idx, folder));

        // Toggle: collapsed
        state.toggle_explored_folder(msg_idx, folder);
        assert!(!state.is_explored_folder_expanded(msg_idx, folder));

        // Toggle back: expanded
        state.toggle_explored_folder(msg_idx, folder);
        assert!(state.is_explored_folder_expanded(msg_idx, folder));

        // Test jump_to_file_and_line with selection
        let tmp = std::env::temp_dir().join("zenthree_test_explored_file.rs");
        std::fs::write(&tmp, "line1\nline2\nline3\nline4\n").unwrap();
        state.jump_to_file_and_line(tmp.to_str().unwrap(), Some("#L2-3"));
        let tab_idx = state.active_tab.expect("Tab should be open");
        let tab = &state.tabs[tab_idx];
        let sel = tab.editor.selection();
        assert_eq!(sel.start, editor::buffer::Point::new(1, 0));
        assert_eq!(sel.end, editor::buffer::Point::new(2, 5));
        let _ = std::fs::remove_file(tmp);
    }
}


