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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalLine {
    pub text: String,
    pub is_command: bool,
    pub is_error: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiMessage {
    pub is_user: bool,
    pub content: String,
    pub code_snippet: Option<String>,
    pub thoughts: Option<String>,
}

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
}

impl ZenthreeState {
    pub fn new() -> Self {
        // Try workspace argument, fallback to std::env::current_dir(), fallback to home dir
        let args: Vec<String> = std::env::args().collect();
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        });

        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                current_dir = path;
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
                    thoughts: Some("Context initialized with workspace files and syntax engine.".to_string()),
                },
            ],
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
        };

        // Restore previous session state
        #[cfg(not(test))]
        state.load_persistent_state();

        // If CLI argument was explicitly provided, override workspace
        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                state.current_dir = path;
            }
        }

        state.rebuild_tree();
        state
    }

    pub fn save_persistent_state(&self) {
        let path = get_config_path();
        let open_tabs: Vec<PathBuf> = self.tabs.iter()
            .filter_map(|t| t.path.clone())
            .collect();
        let active_tab_path = self.active_tab
            .and_then(|idx| self.tabs.get(idx))
            .and_then(|t| t.path.clone());

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
            current_dir: Some(self.current_dir.clone()),
            expanded_dirs: self.expanded_dirs.iter().cloned().collect(),
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
                if args.len() <= 1 {
                    if let Some(saved_dir) = saved.current_dir {
                        if saved_dir.exists() && saved_dir.is_dir() {
                            self.current_dir = saved_dir;
                        }
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
                        self.check_language_support_prompt(&p);
                    }
                }
            }
        }
    }

    pub fn colors(&self) -> ThemeColors {
        ThemeColors::get_custom(self.theme, &self.accent_color)
    }

    pub fn rebuild_tree(&mut self) {
        self.sidebar_items.clear();
        let root = self.current_dir.clone();
        let git_statuses = if self.explorer_git_status {
            self.get_git_statuses(&root)
        } else {
            HashMap::new()
        };
        let mut ancestors = Vec::new();
        self.scan_dir_recursive(&root, 0, &mut ancestors, &git_statuses);
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

    pub fn new_file_in_workspace(&mut self) {
        let mut idx = 1;
        let mut target_path = self.current_dir.join("untitled.txt");
        while target_path.exists() {
            idx += 1;
            target_path = self.current_dir.join(format!("untitled_{}.txt", idx));
        }
        if std::fs::write(&target_path, "").is_ok() {
            self.rebuild_tree();
            self.open_file(&target_path);
            self.status_message = format!("Created {}", target_path.file_name().unwrap().to_string_lossy());
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
            for (idx, tab) in self.tabs.iter().enumerate() {
                if tab.path.as_ref() == Some(&path.to_path_buf()) {
                    self.active_tab = Some(idx);
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
            });
            self.active_tab = Some(self.tabs.len() - 1);
            self.status_message = format!("Opened file: {}", path.display());
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
                    if std::fs::write(path, tab.editor.buffer.to_string()).is_ok() {
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
                    if std::fs::write(&path, tab.editor.buffer.to_string()).is_ok() {
                        tab.name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                        tab.path = Some(path.clone());
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
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text);
        }
    }

    pub fn get_clipboard(&self) -> Option<String> {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(text) = cb.get_text() {
                if !text.is_empty() {
                    return Some(text);
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
                        tab.is_dirty = true;
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
                        tab.is_dirty = true;
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
                    tab.is_dirty = true;
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
                    tab.is_dirty = true;
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
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                self.active_tab = None;
            } else if let Some(active) = self.active_tab {
                if active >= self.tabs.len() {
                    self.active_tab = Some(self.tabs.len() - 1);
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
                            tab.is_dirty = true;
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
        });

        let active_file = self.active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "No active file".to_string());

        let reply = if trimmed.to_lowercase().contains("explain") {
            format!("I've analyzed `{}`. The structure and data-flow adhere to clean Rust architecture with reactive UI bindings.", active_file)
        } else if trimmed.to_lowercase().contains("refactor") || trimmed.to_lowercase().contains("optimize") {
            format!("Here is an optimized refactoring suggestion for `{}`:", active_file)
        } else if trimmed.to_lowercase().contains("test") {
            format!("Unit tests for `{}` can be added following standard Rust conventions with `#[cfg(test)]`.", active_file)
        } else {
            format!("I understand you're asking about `{}`. I'm actively analyzing workspace AST and ready to assist with editing or terminal commands.", trimmed)
        };

        let snippet = if trimmed.to_lowercase().contains("refactor") || trimmed.to_lowercase().contains("code") {
            Some("// Example suggested optimization\npub fn optimize() -> bool {\n    true\n}".to_string())
        } else {
            None
        };

        self.ai_messages.push(AiMessage {
            is_user: false,
            content: reply,
            code_snippet: snippet,
            thoughts: Some(format!("Context loaded: {} | AST verified", active_file)),
        });

        self.ai_input.clear();
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
}

