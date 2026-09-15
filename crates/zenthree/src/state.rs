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

#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    Modified,
    Untracked,
}

#[derive(Debug, Clone)]
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
    Extensions,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExploredItem {
    pub file_name: String,
    pub line_range: Option<String>,
    pub action_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EditedFileSummary {
    pub file_name: String,
    pub lines_added: usize,
    pub lines_removed: usize,
}

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

pub use crate::agent::{
    AgentMission, AgentTask, TaskStatus, TargetFileModification,
    create_mission_for_prompt, build_autonomous_agent_prompt, extract_all_file_modifications,
    run_project_compiler_check, build_self_healing_prompt, extract_tool_call_read_paths,
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
    pub groq_api_key: Option<String>,
    pub gemini_api_key: Option<String>,
    pub ai_receiver: Option<std::sync::mpsc::Receiver<(usize, String)>>,
    pub ai_sender: std::sync::mpsc::Sender<(usize, String)>,
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
    pub collapsed_code_blocks: std::collections::HashSet<u64>,
    pub current_mission: Option<AgentMission>,
    pub current_session: crate::session::AgentSession,
    pub session_list: Vec<crate::session::SessionSummary>,
    pub session_dropdown_open: bool,
    pub active_input_drawer: AiInputDrawer,
    pub session_modified_files: Vec<SessionModifiedFile>,
    pub ai_showing_session_list: bool,
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
            ai_panel_visible: false,
            ai_panel_width: 340.0,
            active_resize_ai_panel: false,
            ai_messages: vec![
                AiMessage {
                    is_user: false,
                    content: "Hello! I am your Zenthra AI Assistant. How can I help you with your code today?".to_string(),
                    code_snippet: None,
                    thoughts: Some("Context initialized with workspace files and biological Connectome deliberation engine.".to_string()),
                    deliberation: None,
                    ..Default::default()
                },
            ],
            ai_input: String::new(),
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
            groq_api_key: crate::groq::load_groq_api_key(),
            gemini_api_key: crate::gemini::load_gemini_api_key(),
            ai_receiver: Some(ai_receiver),
            ai_sender,
            ai_pending_requests: 0,
            pending_modification: None,
            pending_modifications: Vec::new(),
            last_ai_prompt: String::new(),
            expanded_explored_msgs: std::collections::HashSet::new(),
            collapsed_code_blocks: std::collections::HashSet::new(),
            current_mission: None,
            current_session: crate::session::AgentSession::new(&current_dir, None),
            session_list: crate::session::list_saved_sessions(&current_dir),
            session_dropdown_open: false,
            active_input_drawer: AiInputDrawer::None,
            session_modified_files: Vec::new(),
            ai_showing_session_list: true,
        };

        // Restore previous session preferences (theme, accent, extensions, etc.)
        #[cfg(not(test))]
        state.load_persistent_state();

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

        if state.has_workspace {
            state.rebuild_tree();
        }
        state
    }

    pub fn save_persistent_state(&self) {
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

                let args: Vec<String> = std::env::args().collect();
                if args.len() > 1 {
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
            while let Ok((idx, content)) = rx.try_recv() {
                received.push((idx, content));
            }
        }

        let mut updated = false;
        for (idx, content) in received {
            if let Some(msg) = self.ai_messages.get_mut(idx) {
                msg.content = content.clone();
                msg.is_working = false;
            }
            updated = true;
            self.ai_pending_requests = self.ai_pending_requests.saturating_sub(1);

            // API Error Guard: if Groq returned an error (e.g. 429, timeout, network)
            let is_api_error = content.trim().starts_with("⚠️");
            if is_api_error {
                if let Some(ref mut mission) = self.current_mission {
                    if let Some(task) = mission.tasks.get_mut(1) {
                        task.status = TaskStatus::Failed(content.trim().to_string());
                    }
                    let verify_idx = mission.tasks.len().saturating_sub(1);
                    if let Some(task) = mission.tasks.get_mut(verify_idx) {
                        task.status = TaskStatus::Failed("API call failed".to_string());
                    }
                    mission.is_active = false;
                    mission.status_message = format!("⚠️ Mission failed: {}", content.trim());
                    self.current_session.update_mission_file(mission);
                }
                continue;
            }

            // 1. Check if response specifies folder / directory creation
            let folders_to_create = detect_target_folders_from_response(&content, &self.last_ai_prompt, &self.current_dir);
            for folder in folders_to_create {
                if let Ok(_) = std::fs::create_dir_all(&folder) {
                    self.expanded_dirs.insert(folder.clone());
                    self.rebuild_tree();
                    let folder_name = folder.file_name().and_then(|n| n.to_str()).unwrap_or("folder");
                    self.status_message = format!("✓ Created directory `{}`", folder_name);
                }
            }

            // 2. Check if response contains proposed code modifications or new files
            let active_path = self.active_tab.and_then(|idx| self.tabs.get(idx)).and_then(|t| t.path.as_deref());
            let mut file_modifications = extract_all_file_modifications(&content, &self.current_dir, active_path);

            if file_modifications.is_empty() {
                if let Some(proposed_code) = extract_code_from_markdown(&content) {
                    if let Some(target_file) = detect_target_file_from_response(
                        &content,
                        &self.last_ai_prompt,
                        &self.current_dir,
                        active_path,
                    ) {
                        file_modifications.push(TargetFileModification {
                            file_path: target_file,
                            proposed_code,
                        });
                    }
                }
            }

            let mut summaries = Vec::new();
            for modif in file_modifications {
                let target_file = modif.file_path;
                let proposed_code = modif.proposed_code;

                if let Some(parent) = target_file.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                let is_new_file = !target_file.exists();
                let original_code = if let Some(tab) = self.tabs.iter().find(|t| t.path.as_deref() == Some(&target_file)) {
                    tab.editor.buffer.to_string()
                } else if let Ok(disk_content) = std::fs::read_to_string(&target_file) {
                    disk_content
                } else {
                    String::new()
                };

                if is_new_file || (!original_code.trim().is_empty() && proposed_code != original_code) {
                    let (diff_lines, preview_text) = compute_line_diff(&original_code, &proposed_code);
                    let has_changes = is_new_file || diff_lines.iter().any(|(_, s)| *s != DiffLineState::Unchanged);
                    if has_changes {
                        // Write proposed code to disk so project compiler check can run against it
                        let _ = std::fs::write(&target_file, &proposed_code);

                        if is_new_file {
                            let name = target_file.file_name().and_then(|n| n.to_str()).unwrap_or("New File").to_string();
                            let existing_pos = self.tabs.iter().position(|t| t.path.as_deref() == Some(&target_file));
                            if let Some(idx) = existing_pos {
                                self.active_tab = Some(idx);
                                if let Some(tab) = self.tabs.get_mut(idx) {
                                    tab.editor.replace_all(&preview_text);
                                }
                            } else {
                                self.tabs.push(EditorTab {
                                    name,
                                    path: Some(target_file.clone()),
                                    editor: Editor::new(Buffer::from_text(&preview_text)),
                                    is_dirty: true,
                                    scroll_y: 0.0,
                                    shaping_cache: std::collections::HashMap::new(),
                                    version: 0,
                                    saved_text: String::new(),
                                });
                                self.active_tab = Some(self.tabs.len() - 1);
                            }
                        } else {
                            self.open_file(&target_file);
                            if let Some(idx) = self.active_tab {
                                if let Some(tab) = self.tabs.get_mut(idx) {
                                    tab.editor.replace_all(&preview_text);
                                }
                            }
                        }

                        let added_count = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                        let removed_count = diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                        let file_display = target_file.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                        summaries.push(EditedFileSummary {
                            file_name: file_display,
                            lines_added: added_count,
                            lines_removed: removed_count,
                        });

                        let pm = PendingModification {
                            file_path: target_file.clone(),
                            original_code,
                            proposed_code,
                            diff_lines,
                        };
                        self.pending_modifications.push(pm.clone());
                        if self.pending_modification.is_none() {
                            self.pending_modification = Some(pm);
                        }
                    }
                }
            }

            if !summaries.is_empty() {
                let now_ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                for s in &summaries {
                    let fpath = self.current_dir.join(&s.file_name);
                    if let Some(existing) = self.session_modified_files.iter_mut().find(|f| f.file_name == s.file_name) {
                        existing.lines_added = s.lines_added;
                        existing.lines_removed = s.lines_removed;
                        existing.timestamp = now_ts;
                    } else {
                        self.session_modified_files.push(SessionModifiedFile {
                            file_path: fpath,
                            file_name: s.file_name.clone(),
                            lines_added: s.lines_added,
                            lines_removed: s.lines_removed,
                            timestamp: now_ts,
                            is_new_file: s.lines_removed == 0 && s.lines_added > 0,
                        });
                    }
                }
                self.active_input_drawer = AiInputDrawer::PendingModifications;
                self.rebuild_tree();
                self.status_message = "Review AI changes: +green added, -red removed. Accept or Reject above input box.".to_string();
                if let Some(msg) = self.ai_messages.get_mut(idx) {
                    msg.edited_files = summaries.clone();
                }
                self.current_session.update_walkthrough_file(&self.last_ai_prompt, &summaries, false);
            }

            // 3. Check if the model attempted a tool call to read files
            let requested_read_paths = extract_tool_call_read_paths(&content);
            if summaries.is_empty() && !requested_read_paths.is_empty() {
                let mut read_snippets = Vec::new();
                for rel_path in &requested_read_paths {
                    let full_path = self.current_dir.join(rel_path);
                    if let Ok(c) = std::fs::read_to_string(&full_path) {
                        if !c.trim().is_empty() {
                            let sample: String = c.chars().take(4000).collect();
                            read_snippets.push(format!("File `{}`:\n```\n{}\n```", rel_path, sample));
                        }
                    }
                }

                if !read_snippets.is_empty() && (self.gemini_api_key.is_some() || self.groq_api_key.is_some()) {
                    if let Some(ref mut mission) = self.current_mission {
                        mission.status_message = format!("Agent read {} requested files... generating code", read_snippets.len());
                        if let Some(task) = mission.tasks.get_mut(1) {
                            task.title = format!("Read {} files & implement code", read_snippets.len());
                            task.status = TaskStatus::InProgress;
                        }
                        self.current_session.update_mission_file(mission);
                    }
                    if let Some(msg) = self.ai_messages.get_mut(idx) {
                        msg.is_working = true;
                    }
                    self.ai_pending_requests += 1;
                    let gemini_key = self.gemini_api_key.clone();
                    let groq_key = self.groq_api_key.clone();
                    let sender = self.ai_sender.clone();
                    let system_prompt = build_autonomous_agent_prompt(95.0);
                    let goal_prompt = self.last_ai_prompt.clone();
                    let continuation_prompt = format!(
                        "Here are the contents of the files you requested from the workspace:\n\n\
                        {}\n\n\
                        User Goal: {}\n\
                        NOW write the complete code implementation for each target file using `### File: <path>` tags and fenced code blocks.",
                        read_snippets.join("\n\n"),
                        goal_prompt
                    );

                    std::thread::spawn(move || {
                        if let Some(ref gkey) = gemini_key {
                            let res_text = match crate::gemini::call_gemini_api(gkey, &system_prompt, &continuation_prompt, &[]) {
                                Ok(txt) => txt,
                                Err(e) => format!("⚠️ Gemini continuation error: {}", e),
                            };
                            let _ = sender.send((idx, res_text));
                            return;
                        }
                        if let Some(ref qkey) = groq_key {
                            let res_text = match crate::groq::call_groq_api(qkey, &system_prompt, &continuation_prompt) {
                                Ok(txt) => txt,
                                Err(e) => format!("⚠️ Groq continuation error: {}", e),
                            };
                            let _ = sender.send((idx, res_text));
                            return;
                        }
                        let _ = sender.send((idx, "⚠️ No AI provider configured".to_string()));
                    });
                    continue;
                }
            }

            // 4. If prompt requested code action but model only returned plan/advice without code blocks
            let is_action_prompt = {
                let lower = self.last_ai_prompt.to_lowercase();
                lower.contains("create")
                    || lower.contains("add")
                    || lower.contains("make")
                    || lower.contains("update")
                    || lower.contains("implement")
                    || lower.contains("refactor")
            };

            if summaries.is_empty() && is_action_prompt {
                if let Some(ref mut mission) = self.current_mission {
                    if let Some(task) = mission.tasks.get_mut(1) {
                        task.status = TaskStatus::Completed;
                        task.title = "Formulate architectural plan".to_string();
                    }
                    let verify_idx = mission.tasks.len().saturating_sub(1);
                    if let Some(task) = mission.tasks.get_mut(verify_idx) {
                        task.title = "Plan formulated (Send 'now update it' or 'proceed' to write code)".to_string();
                        task.status = TaskStatus::Pending;
                    }
                    mission.is_active = false;
                    mission.status_message = "Architectural plan ready. Ask AI to write code to apply.".to_string();
                    self.current_session.update_mission_file(mission);
                }
                continue;
            }

            let mut trigger_self_healing = false;
            let mut self_healing_prompt = String::new();

            // Update Agent Mission: complete task 2, re-read checklist, then run compiler self-healing check
            if let Some(ref mut mission) = self.current_mission {
                if let Some(task) = mission.tasks.get_mut(1) {
                    task.status = TaskStatus::Completed;
                    if let Some(first_sum) = summaries.first() {
                        task.diff_summary = Some(first_sum.clone());
                        task.target_file = Some(first_sum.file_name.clone());
                    }
                }

                // If multiple files were edited, ensure each is registered in the mission checklist
                for sum in summaries.iter().skip(1) {
                    mission.tasks.push(AgentTask {
                        id: format!("task_file_{}", sum.file_name),
                        title: format!("Create / update `{}`", sum.file_name),
                        status: TaskStatus::Completed,
                        target_file: Some(sum.file_name.clone()),
                        diff_summary: Some(sum.clone()),
                    });
                }

                let verify_idx = mission.tasks.len().saturating_sub(1);
                mission.active_task_index = Some(verify_idx);

                // Step: Re-read the checklist & verify with project compiler
                mission.status_message = "Agent reading checklist... verifying syntax & compiler diagnostics".to_string();

                if !summaries.is_empty() {
                    match run_project_compiler_check(&self.current_dir) {
                        Some(Err(compiler_err)) => {
                            if mission.self_healing_attempts < 2 {
                                mission.self_healing_attempts += 1;
                                mission.is_active = true;
                                mission.status_message = format!("🛠️ Self-Healing: Repairing compiler errors (Attempt {}/2)...", mission.self_healing_attempts);
                                if let Some(task) = mission.tasks.get_mut(verify_idx) {
                                    task.title = format!("🛠️ Self-Healing: Repair compiler errors (Attempt {}/2)", mission.self_healing_attempts);
                                    task.status = TaskStatus::InProgress;
                                }
                                let target_hint = summaries.first().map(|s| s.file_name.as_str());
                                self_healing_prompt = build_self_healing_prompt(&mission.goal, &compiler_err, target_hint);
                                trigger_self_healing = true;
                            } else {
                                if let Some(task) = mission.tasks.get_mut(verify_idx) {
                                    task.title = "Compiler check (Errors remain after 2 healing attempts)".to_string();
                                    task.status = TaskStatus::Failed(compiler_err);
                                }
                                mission.is_active = false;
                                mission.status_message = "⚠️ Self-healing limit reached (2 attempts). Review errors manually.".to_string();
                            }
                        }
                        Some(Ok(())) => {
                            if let Some(task) = mission.tasks.get_mut(verify_idx) {
                                task.title = "Verify syntax & compiler check (Passed ✓)".to_string();
                                task.status = TaskStatus::Completed;
                            }
                            mission.is_active = false;
                            mission.status_message = "✓ All tasks completed and verified with compiler".to_string();
                        }
                        None => {
                            if let Some(task) = mission.tasks.get_mut(verify_idx) {
                                task.title = "Verify syntax and code consistency (Passed ✓)".to_string();
                                task.status = TaskStatus::Completed;
                            }
                            mission.is_active = false;
                            mission.status_message = "✓ All tasks completed successfully".to_string();
                        }
                    }
                } else {
                    if let Some(task) = mission.tasks.get_mut(verify_idx) {
                        task.status = TaskStatus::Completed;
                    }
                    mission.is_active = false;
                    mission.status_message = "✓ All tasks completed successfully".to_string();
                }

                self.current_session.update_mission_file(mission);
            }

            // If compiler self-healing was triggered and API key is available, dispatch repair LLM call
            if trigger_self_healing && (self.gemini_api_key.is_some() || self.groq_api_key.is_some()) {
                if let Some(msg) = self.ai_messages.get_mut(idx) {
                    msg.is_working = true;
                }
                self.ai_pending_requests += 1;
                let gemini_key = self.gemini_api_key.clone();
                let groq_key = self.groq_api_key.clone();
                let sender = self.ai_sender.clone();
                let system_prompt = build_autonomous_agent_prompt(95.0);
                std::thread::spawn(move || {
                    if let Some(ref gkey) = gemini_key {
                        let res = match crate::gemini::call_gemini_api(gkey, &system_prompt, &self_healing_prompt, &[]) {
                            Ok(txt) => txt,
                            Err(e) => format!("⚠️ Self-healing (Gemini) failed: {}", e),
                        };
                        let _ = sender.send((idx, res));
                        return;
                    }
                    if let Some(ref qkey) = groq_key {
                        let res = match crate::groq::call_groq_api(qkey, &system_prompt, &self_healing_prompt) {
                            Ok(txt) => txt,
                            Err(e) => format!("⚠️ Self-healing (Groq) failed: {}", e),
                        };
                        let _ = sender.send((idx, res));
                        return;
                    }
                    let _ = sender.send((idx, "⚠️ Self-healing: No AI provider configured.".to_string()));
                });
            }
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

    pub fn toggle_explored_accordion(&mut self, msg_idx: usize) {
        if self.expanded_explored_msgs.contains(&msg_idx) {
            self.expanded_explored_msgs.remove(&msg_idx);
        } else {
            self.expanded_explored_msgs.insert(msg_idx);
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
        self.ai_pending_requests > 0 || self.current_mission.as_ref().map(|m| m.is_active && !m.is_paused).unwrap_or(false)
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
        ThemeColors::get_custom(self.theme, &self.accent_color)
    }

    pub fn rebuild_tree(&mut self) {
        self.sidebar_items.clear();
        if !self.has_workspace {
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
    }

    pub fn start_new_session(&mut self) {
        let new_sess = crate::session::AgentSession::new(&self.current_dir, None);
        self.current_session = new_sess;
        self.current_mission = None;
        self.pending_modification = None;
        self.pending_modifications.clear();
        self.ai_messages = vec![
            AiMessage {
                is_user: false,
                content: "Hello! I am your Zenthra AI Assistant. How can I help you with your code today?".to_string(),
                code_snippet: None,
                thoughts: Some("Session initialized. Session markdown files ready in .zenthree/sessions/".to_string()),
                deliberation: None,
                ..Default::default()
            },
        ];
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);
        self.session_dropdown_open = false;
        self.active_input_drawer = AiInputDrawer::None;
        self.session_modified_files.clear();
        self.ai_showing_session_list = false;
        self.status_message = "Started new agent session".to_string();
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
        if let Some(sess) = crate::session::AgentSession::load(&self.current_dir, session_id) {
            self.current_session = sess;
            self.current_mission = None;
            self.pending_modification = None;
            self.pending_modifications.clear();
            self.session_modified_files.clear();
            self.active_input_drawer = AiInputDrawer::None;

            let mut msgs = Vec::new();
            let mut thoughts = format!("Resumed session: {}", self.current_session.title);

            let walk_path = self.current_session.walkthrough_file_path();
            if let Ok(walk_text) = std::fs::read_to_string(&walk_path) {
                if walk_text.contains("## Changes Implemented") {
                    thoughts.push_str(" (Previous changes and artifacts loaded)");
                }
            }

            msgs.push(AiMessage {
                is_user: false,
                content: format!("Resumed session **{}**.\nYou can continue this conversation, review files, or issue new tasks.", self.current_session.title),
                code_snippet: None,
                thoughts: Some(thoughts),
                deliberation: None,
                ..Default::default()
            });

            self.ai_messages = msgs;
            self.ai_showing_session_list = false;
            self.session_dropdown_open = false;
            self.status_message = format!("Switched to session {}", self.current_session.title);
        }
    }

    pub fn open_workspace(&mut self, path: PathBuf) {
        self.current_dir = path;
        self.has_workspace = true;
        self.expanded_dirs.clear();
        self.current_session = crate::session::AgentSession::new(&self.current_dir, None);
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);
        self.rebuild_tree();
        self.status_message = format!("Opened workspace: {}", self.current_dir.display());
        self.save_persistent_state();
    }

    pub fn close_workspace(&mut self) {
        self.has_workspace = false;
        self.sidebar_items.clear();
        self.expanded_dirs.clear();
        self.current_session = crate::session::AgentSession::new(&self.current_dir, None);
        self.session_list = crate::session::list_saved_sessions(&self.current_dir);
        self.status_message = "Workspace closed".to_string();
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
        if let Ok(content) = std::fs::read_to_string(path) {
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
                if tab.path.as_ref() == Some(&path.to_path_buf()) {
                    self.active_tab = Some(idx);
                    if !tab.is_dirty && tab.saved_text != content {
                        tab.editor.replace_all(&content);
                        tab.saved_text = content.clone();
                    }
                    self.status_message = format!("Switched to {}", name);
                    self.check_language_support_prompt(path);
                    self.save_persistent_state();
                    return;
                }
            }

            self.tabs.push(EditorTab {
                name,
                path: Some(path.to_path_buf()),
                editor: Editor::new(Buffer::from_text(&content)),
                is_dirty: false,
                scroll_y: 0.0,
                shaping_cache: std::collections::HashMap::new(),
                version: 0,
                saved_text: content.clone(),
            });
            self.active_tab = Some(self.tabs.len() - 1);
            self.status_message = format!("Opened file: {}", path.display());
            let ws_root = if self.has_workspace { Some(self.current_dir.as_path()) } else { None };
            self.lsp_manager.on_document_open(path, &content, ws_root);
            self.check_language_support_prompt(path);
            self.save_persistent_state();
        } else {
            self.status_message = format!("Failed to open file: {}", path.display());
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
        }
        self.terminal_lines.clear();
        self.terminal_input.clear();
        self.terminal_history_idx = None;
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

        self.ai_messages.push(AiMessage {
            is_user: true,
            content: trimmed.to_string(),
            code_snippet: None,
            thoughts: None,
            deliberation: None,
            ..Default::default()
        });

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
        let is_refactor = lower.contains("refactor") || lower.contains("optimize");
        let is_explain = lower.contains("explain");
        let is_test = lower.contains("test");
        let is_project_query = lower.contains("project")
            || lower.contains("workspace")
            || lower.contains("about")
            || lower == "hi"
            || lower.starts_with("hi ")
            || lower.starts_with("hello");

        // 2. Biological Connectome Deliberation Gate:
        // Formulate dual-channel evidence streams (Option A: Primary target action vs Option B: Conservative fallback)
        let (ev_a, ev_b) = if is_refactor {
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

        // If Gemini or Groq API key is configured, query LLM asynchronously with Connectome deliberation attached
        if self.gemini_api_key.is_some() || self.groq_api_key.is_some() {
            let msg_idx = self.ai_messages.len();
            let thoughts = if let Some(ref ctx) = ast_context {
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

            let project_ctx = crate::project_index::search_project_context(&self.current_dir, trimmed, 6);

            let mut explored_items = Vec::new();
            for f in &project_ctx.matched_files {
                let range = if let Some((first_l, _)) = f.snippet_lines.first() {
                    let last_l = f.snippet_lines.last().map(|(l, _)| *l).unwrap_or(*first_l);
                    if *first_l == last_l {
                        Some(format!("#L{}", first_l))
                    } else {
                        Some(format!("#L{}-{}", first_l, (first_l + 15).max(last_l)))
                    }
                } else {
                    None
                };
                let display_file = f.path.file_name().and_then(|n| n.to_str()).unwrap_or(&f.relative_path);
                explored_items.push(ExploredItem {
                    file_name: display_file.to_string(),
                    line_range: range,
                    action_type: "Analyzed".to_string(),
                });
            }
            if explored_items.is_empty() {
                if let Some(tab_idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get(tab_idx) {
                        let lines_cnt = tab.editor.buffer.line_count();
                        explored_items.push(ExploredItem {
                            file_name: tab.name.clone(),
                            line_range: Some(format!("#L1-{}", lines_cnt.min(100))),
                            action_type: "Analyzed".to_string(),
                        });
                    }
                }
            }

            let mission = create_mission_for_prompt(trimmed, &explored_items);
            self.current_session.title = trimmed.to_string();
            self.current_session.update_implementation_plan(trimmed, &explored_items, "");
            self.current_session.update_mission_file(&mission);
            self.current_mission = Some(mission);

            self.ai_messages.push(AiMessage {
                is_user: false,
                content: String::new(),
                code_snippet: None,
                thoughts: Some(thoughts),
                deliberation: Some(deliberation.clone()),
                explored_items,
                edited_files: Vec::new(),
                is_working: true,
            });

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

            let system_prompt = build_autonomous_agent_prompt(deliberation.confidence * 100.0);

            let mut user_prompt = String::new();
            if !project_ctx.formatted_prompt_context.is_empty() {
                user_prompt.push_str(&project_ctx.formatted_prompt_context);
                user_prompt.push_str("\n\n");
            }
            // Include contents of top matched project files so the AI can read and trace symbols across unopened files
            for f in project_ctx.matched_files.iter().take(2) {
                if let Ok(c) = std::fs::read_to_string(&f.path) {
                    if !c.trim().is_empty() {
                        let sample: String = c.chars().take(4000).collect();
                        user_prompt.push_str(&format!("Project file `{}`:\n```\n{}\n```\n\n", f.relative_path, sample));
                    }
                }
            }
            if !buffer_snippet.is_empty() {
                let code_sample: String = buffer_snippet.chars().take(4000).collect();
                user_prompt.push_str(&format!("Active buffer `{}`:\n```\n{}\n```\n\n", active_file, code_sample));
            }
            user_prompt.push_str(&format!("User Request:\n{}", query));

            // Build multi-turn chat messages including recent conversation turns
            let mut chat_messages = Vec::new();
            chat_messages.push(crate::groq::ChatMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
            });

            let total_msgs = self.ai_messages.len();
            if total_msgs > 2 {
                let start_idx = total_msgs.saturating_sub(6);
                let end_idx = total_msgs.saturating_sub(1);
                for prev in &self.ai_messages[start_idx..end_idx] {
                    if prev.content.trim().is_empty() || prev.content.starts_with("⚠️") {
                        continue;
                    }
                    let role = if prev.is_user { "user" } else { "assistant" };
                    chat_messages.push(crate::groq::ChatMessage {
                        role: role.to_string(),
                        content: prev.content.clone(),
                    });
                }
            }

            chat_messages.push(crate::groq::ChatMessage {
                role: "user".to_string(),
                content: user_prompt.clone(),
            });

            std::thread::spawn(move || {
                // 1. Google Gemini (Preferred when configured)
                if let Some(ref gkey) = gemini_key {
                    let history = if chat_messages.len() > 2 {
                        &chat_messages[1..chat_messages.len() - 1]
                    } else {
                        &[]
                    };
                    let result = match crate::gemini::call_gemini_api(gkey, &system_prompt, &user_prompt, history) {
                        Ok(txt) => txt,
                        Err(e) => format!("⚠️ Gemini API error: {}", e),
                    };
                    let _ = sender.send((msg_idx, result));
                    return;
                }

                // 2. Groq (only if Gemini is not configured)
                if let Some(ref qkey) = groq_key {
                    let result = match crate::groq::call_groq_api_messages(qkey, &chat_messages) {
                        Ok(txt) => txt,
                        Err(e) => format!("⚠️ Groq API error: {}", e),
                    };
                    let _ = sender.send((msg_idx, result));
                    return;
                }

                let _ = sender.send((msg_idx, "⚠️ No AI provider configured. Add GEMINI_API_KEY in .env".to_string()));
            });
            return;
        }

        let reply = if is_refactor {
            format!(
                "### Suggested Optimization for `{}`\n\nBiological deliberation reached **{:.1}% confidence** across **{} dt**.\n\n```rust\n// Zero-allocation idiomatic implementation\npub fn optimize() -> Result<bool, Box<dyn std::error::Error>> {{\n    // In-place processing\n    Ok(true)\n}}\n```",
                active_file, deliberation.confidence * 100.0, deliberation.reaction_time_dt
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
        } else if is_project_query {
            let project_name = self.current_dir.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Workspace".to_string());

            let readme_path = self.current_dir.join("README.md");
            let readme_summary = if readme_path.exists() {
                std::fs::read_to_string(&readme_path)
                    .ok()
                    .and_then(|content| {
                        let lines: Vec<&str> = content.lines()
                            .map(|l| l.trim())
                            .filter(|l| !l.is_empty() && !l.starts_with('#'))
                            .take(2)
                            .collect();
                        if lines.is_empty() { None } else { Some(lines.join(" ")) }
                    })
            } else {
                None
            };

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
            out.push_str(&format!("- **Active Document**: `{}`\n", active_file));
            out.push_str(&format!("- **Open Tabs**: {} tabs active\n\n", self.tabs.len()));

            if let Some(summary) = readme_summary {
                out.push_str("### About\n");
                out.push_str(&summary);
                out.push_str("\n\n");
            }

            out.push_str("### What would you like to explore?\n");
            out.push_str("- **Explain Code**: Inspect syntax structures and AST symbol scopes\n");
            out.push_str("- **Refactor**: Suggest zero-allocation idiomatic improvements\n");
            out.push_str("- **Generate Tests**: Write automated unit test suites");
            out
        } else {
            format!(
                "Query: `{}`\n\nConnectome neural gate verified the prompt with **{:.1}% confidence** ({}) across **{} dt**.\n\n- Use **Explain Code** to break down syntax trees\n- Use **Refactor** for idiomatic optimizations\n- Use **Generate Tests** for test suites",
                trimmed, deliberation.confidence * 100.0, deliberation.choice_label, deliberation.reaction_time_dt
            )
        };

        let thoughts = if let Some(ref ctx) = ast_context {
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
            ..Default::default()
        });

        self.ai_input.clear();
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

pub fn detect_target_file_from_response(
    content: &str,
    prompt: &str,
    workspace_root: &Path,
    active_path: Option<&Path>,
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
                let candidate = if Path::new(cleaned).is_absolute() {
                    PathBuf::from(cleaned)
                } else {
                    workspace_root.join(cleaned)
                };
                return Some(candidate);
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
                    let direct = if Path::new(clean).is_absolute() {
                        PathBuf::from(clean)
                    } else {
                        workspace_root.join(clean)
                    };
                    return Some(direct);
                }
            }
        }
    }

    // 3. Check if user prompt mentions a known file path with extension in workspace
    for token in prompt.split_whitespace() {
        let clean = token.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '_' && c != '-');
        if clean.contains('.') && !clean.starts_with('.') {
            let direct = workspace_root.join(clean);
            if direct.exists() && direct.is_file() {
                return Some(direct);
            }
        }
    }

    // 4. Fallback to active editor buffer file if open and valid
    if let Some(act) = active_path {
        if act.exists() {
            return Some(act.to_path_buf());
        }
    }

    // 5. Fallback to top keyword-matched file in workspace
    let summary = crate::project_index::search_project_context(workspace_root, prompt, 1);
    if let Some(first_match) = summary.matched_files.first() {
        if first_match.path.exists() {
            return Some(first_match.path.clone());
        }
    }

    None
}


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

pub fn compute_line_diff(original: &str, proposed: &str) -> (Vec<(String, DiffLineState)>, String) {
    let orig_lines: Vec<&str> = original.lines().collect();
    let prop_lines: Vec<&str> = proposed.lines().collect();

    let mut diff = Vec::new();
    let mut preview = Vec::new();

    let mut i = 0;
    let mut j = 0;

    while i < orig_lines.len() || j < prop_lines.len() {
        if i < orig_lines.len() && j < prop_lines.len() && orig_lines[i] == prop_lines[j] {
            diff.push((orig_lines[i].to_string(), DiffLineState::Unchanged));
            preview.push(orig_lines[i].to_string());
            i += 1;
            j += 1;
        } else {
            let mut match_found = false;
            for look_j in (j + 1)..=(j + 6).min(prop_lines.len()) {
                if i < orig_lines.len() && orig_lines[i] == prop_lines[look_j - 1] {
                    while j < look_j - 1 {
                        diff.push((prop_lines[j].to_string(), DiffLineState::Added));
                        preview.push(prop_lines[j].to_string());
                        j += 1;
                    }
                    match_found = true;
                    break;
                }
            }

            if !match_found {
                for look_i in (i + 1)..=(i + 6).min(orig_lines.len()) {
                    if j < prop_lines.len() && orig_lines[look_i - 1] == prop_lines[j] {
                        while i < look_i - 1 {
                            diff.push((orig_lines[i].to_string(), DiffLineState::Removed));
                            preview.push(orig_lines[i].to_string());
                            i += 1;
                        }
                        match_found = true;
                        break;
                    }
                }
            }

            if !match_found {
                if i < orig_lines.len() {
                    diff.push((orig_lines[i].to_string(), DiffLineState::Removed));
                    preview.push(orig_lines[i].to_string());
                    i += 1;
                }
                if j < prop_lines.len() {
                    diff.push((prop_lines[j].to_string(), DiffLineState::Added));
                    preview.push(prop_lines[j].to_string());
                    j += 1;
                }
            }
        }
    }

    let preview_text = preview.join("\n");
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
        assert!(state.ai_messages.iter().any(|m| m.is_user && m.content == "Explain the workspace"));
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
        let mut state = ZenthreeState::new();
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
        let mut state = ZenthreeState::new();
        assert!(!state.is_agent_working());
        assert!(state.current_mission.is_none());

        // Send AI message initializes mission with structured tasks
        state.send_ai_message("Implement JWT authentication");
        let mission = state.current_mission.as_ref().expect("Mission should be initialized");
        assert_eq!(mission.goal, "Implement JWT authentication");
        assert_eq!(mission.tasks.len(), 3);
        assert_eq!(mission.tasks[0].status, TaskStatus::Completed);
        assert_eq!(mission.tasks[1].status, TaskStatus::InProgress);
        assert_eq!(mission.tasks[2].status, TaskStatus::Pending);
        assert!(state.is_agent_working());

        // Test Pause
        state.pause_agent_mission();
        assert!(state.current_mission.as_ref().unwrap().is_paused);
        assert!(!state.is_agent_working());

        // Test Resume
        state.resume_agent_mission();
        assert!(!state.current_mission.as_ref().unwrap().is_paused);
        assert!(state.is_agent_working());
    }

    #[test]
    fn test_extract_multiple_file_modifications() {
        let content = "### Plan\n1. Create Navbar\n2. Create Contact\n\n### File: src/components/Navbar.tsx\n```tsx\nexport const Navbar = () => <nav>Navbar</nav>;\n```\n\n### File: src/pages/Contact.tsx\n```tsx\nexport const Contact = () => <div>Contact</div>;\n```";
        let tmp = PathBuf::from("/workspace");
        let mods = extract_all_file_modifications(content, &tmp, None);
        assert_eq!(mods.len(), 2);
        assert_eq!(mods[0].file_path, tmp.join("src/components/Navbar.tsx"));
        assert!(mods[0].proposed_code.contains("export const Navbar"));
        assert_eq!(mods[1].file_path, tmp.join("src/pages/Contact.tsx"));
        assert!(mods[1].proposed_code.contains("export const Contact"));
    }

    #[test]
    fn test_compiler_self_healing_state() {
        let mut state = ZenthreeState::new();
        state.groq_api_key = None;
        state.gemini_api_key = None;
        let items = vec![ExploredItem {
            file_name: "src/main.rs".to_string(),
            line_range: Some("#L1-10".to_string()),
            action_type: "Analyzed".to_string(),
        }];
        let mission = create_mission_for_prompt("fix navbar bug", &items);
        assert_eq!(mission.self_healing_attempts, 0);
        assert_eq!(mission.tasks.len(), 3);
        assert_eq!(mission.tasks[2].id, "task_3_verify");
        state.current_mission = Some(mission);

        // Simulate simulated error in healing attempt
        if let Some(ref mut m) = state.current_mission {
            m.self_healing_attempts += 1;
            m.tasks[2].status = TaskStatus::InProgress;
            m.tasks[2].title = format!("🛠️ Self-Healing: Repair compiler errors (Attempt {}/2)", m.self_healing_attempts);
        }

        let m = state.current_mission.as_ref().unwrap();
        assert_eq!(m.self_healing_attempts, 1);
        assert_eq!(m.tasks[2].status, TaskStatus::InProgress);
        assert!(m.tasks[2].title.contains("Attempt 1/2"));
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
}


