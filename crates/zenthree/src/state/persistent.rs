use crate::state::types::*;
use crate::state::ZenthreeState;
use crate::theme::ThemeMode;
use ::editor::buffer::Buffer;
use ::editor::editor::Editor;
use std::path::PathBuf;

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
    pub selected_ai_model: Option<AiModel>,
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

impl ZenthreeState {
    pub fn save_persistent_state(&self) {
        let path = get_config_path();
        let current_dir = if self.has_workspace {
            Some(self.current_dir.clone())
        } else {
            None
        };
        let open_tabs: Vec<PathBuf> = if self.has_workspace {
            self.tabs.iter().filter_map(|t| t.path.clone()).collect()
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
            expanded_dirs: if self.has_workspace {
                self.expanded_dirs.iter().cloned().collect()
            } else {
                Vec::new()
            },
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
            selected_ai_model: Some(self.selected_ai_model),
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
                if let Some(theme) = saved.theme {
                    self.theme = theme;
                }
                if let Some(accent) = saved.accent_color {
                    self.accent_color = accent;
                }
                if let Some(icon) = saved.icon_theme {
                    self.icon_theme = icon;
                }
                if let Some(preset) = saved.explorer_preset {
                    self.explorer_preset = preset;
                }
                if let Some(guides) = saved.explorer_tree_guides {
                    self.explorer_tree_guides = guides;
                }
                if let Some(git) = saved.explorer_git_status {
                    self.explorer_git_status = git;
                }
                if let Some(hidden) = saved.explorer_show_hidden {
                    self.explorer_show_hidden = hidden;
                }
                if let Some(sb_vis) = saved.sidebar_visible {
                    self.sidebar_visible = sb_vis;
                }
                if let Some(sb_w) = saved.sidebar_width {
                    self.sidebar_width = sb_w;
                }
                if let Some(st_vis) = saved.status_bar_visible {
                    self.status_bar_visible = st_vis;
                }
                if let Some(ln_vis) = saved.line_numbers_visible {
                    self.line_numbers_visible = ln_vis;
                }
                if let Some(zoom) = saved.zoom_level {
                    self.zoom_level = zoom;
                }
                if !saved.recent_files.is_empty() {
                    self.recent_files = saved.recent_files;
                }
                if let Some(tv) = saved.terminal_visible {
                    self.terminal_visible = tv;
                }
                if let Some(th) = saved.terminal_height {
                    self.terminal_height = th;
                }
                if let Some(aiv) = saved.ai_panel_visible {
                    self.ai_panel_visible = aiv;
                }
                if let Some(aiw) = saved.ai_panel_width {
                    self.ai_panel_width = aiw;
                }
                if let Some(sv) = saved.sidebar_view {
                    self.sidebar_view = sv;
                }
                if !saved.installed_extensions.is_empty() {
                    self.installed_extensions = saved.installed_extensions.into_iter().collect();
                }
                if !saved.ignored_language_prompts.is_empty() {
                    self.ignored_language_prompts =
                        saved.ignored_language_prompts.into_iter().collect();
                }
                if let Some(m) = saved.selected_ai_model {
                    self.selected_ai_model = m;
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
                                let name = tab_path
                                    .file_name()
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
                        let startup_path = self
                            .active_tab
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
}
