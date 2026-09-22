use ::editor::editor::Editor;
use std::path::PathBuf;

#[derive(Clone)]
pub struct EditorTab {
    pub name: String,
    pub path: Option<PathBuf>,
    pub editor: Editor,
    pub is_dirty: bool,
    pub scroll_y: f32,
    pub shaping_cache:
        std::collections::HashMap<usize, (String, zenthra_text::prelude::ShapedBuffer)>,
    pub version: u64,
    pub lines_added: usize,
    pub lines_removed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffLineState {
    Unchanged,
    Added,    // + line in green
    Removed,  // - line in red
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffHunk {
    pub start_line: usize,
    pub removed_lines: Vec<String>,
    pub added_lines_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileDiffReview {
    pub path: PathBuf,
    pub original_content: String,
    pub proposed_content: String,
    pub diff_lines: Vec<(String, DiffLineState)>,
    pub is_new_file: bool,
    pub active_hunk_idx: usize,
}

impl FileDiffReview {
    pub fn total_removed_lines(&self) -> usize {
        self.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count()
    }

    pub fn total_added_lines(&self) -> usize {
        self.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count()
    }

    #[allow(dead_code)]
    pub fn hunk_start_lines(&self) -> Vec<usize> {
        let mut starts = Vec::new();
        let mut in_hunk = false;
        for (i, (_, s)) in self.diff_lines.iter().enumerate() {
            if *s != DiffLineState::Unchanged {
                if !in_hunk {
                    starts.push(i);
                    in_hunk = true;
                }
            } else {
                in_hunk = false;
            }
        }
        if starts.is_empty() {
            starts.push(0);
        }
        starts
    }
}

pub fn path_matches(p1: &std::path::Path, p2: &std::path::Path) -> bool {
    if p1 == p2 {
        return true;
    }
    if let (Ok(c1), Ok(c2)) = (p1.canonicalize(), p2.canonicalize()) {
        if c1 == c2 {
            return true;
        }
    }
    let s1 = p1.to_string_lossy();
    let s2 = p2.to_string_lossy();
    let clean1 = s1.trim_start_matches("./").trim_start_matches('.');
    let clean2 = s2.trim_start_matches("./").trim_start_matches('.');
    if clean1 == clean2 || clean1.ends_with(clean2) || clean2.ends_with(clean1) {
        return true;
    }
    p1.ends_with(p2) || p2.ends_with(p1)
}


#[derive(Debug, Clone, PartialEq)]
pub enum GitStatus {
    Modified,
    Untracked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AiProvider {
    Groq,
    Gemini,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AiModel {
    GroqGptOss120b,
    GroqGptOss20b,
    GroqQwen3_8_27b,
    GeminiFlashLatest,
    GeminiProLatest,
    Gemini25Flash,
}

impl AiModel {
    pub const ALL: [AiModel; 6] = [
        AiModel::GroqGptOss120b,
        AiModel::GroqGptOss20b,
        AiModel::GroqQwen3_8_27b,
        AiModel::GeminiFlashLatest,
        AiModel::GeminiProLatest,
        AiModel::Gemini25Flash,
    ];

    pub fn provider(&self) -> AiProvider {
        match self {
            AiModel::GroqGptOss120b | AiModel::GroqGptOss20b | AiModel::GroqQwen3_8_27b => {
                AiProvider::Groq
            }
            AiModel::GeminiFlashLatest | AiModel::GeminiProLatest | AiModel::Gemini25Flash => {
                AiProvider::Gemini
            }
        }
    }

    pub fn model_id(&self) -> &'static str {
        match self {
            AiModel::GroqGptOss120b => "openai/gpt-oss-120b",
            AiModel::GroqGptOss20b => "openai/gpt-oss-20b",
            AiModel::GroqQwen3_8_27b => "qwen/qwen3.8-27b",
            AiModel::GeminiFlashLatest => "gemini-flash-latest",
            AiModel::GeminiProLatest => "gemini-pro-latest",
            AiModel::Gemini25Flash => "gemini-2.5-flash",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AiModel::GroqGptOss120b => "GPT-OSS 120B",
            AiModel::GroqGptOss20b => "GPT-OSS 20B",
            AiModel::GroqQwen3_8_27b => "Qwen3.8 27B",
            AiModel::GeminiFlashLatest => "Gemini Flash",
            AiModel::GeminiProLatest => "Gemini Pro",
            AiModel::Gemini25Flash => "Gemini 2.5 Flash",
        }
    }
}

impl Default for AiModel {
    fn default() -> Self {
        AiModel::GroqGptOss120b
    }
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
pub enum ExplorationItem {
    AnalyzedFile {
        path: String,
        start_line: Option<usize>,
        end_line: Option<usize>,
    },
    Searched {
        query: String,
    },
    ExploredDirectory {
        path: String,
    },
    Thought {
        summary: String,
        duration_secs: Option<u64>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExplorationGroup {
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub items: Vec<ExplorationItem>,
    #[serde(default)]
    pub is_active: bool,
}

impl ExplorationGroup {
    pub fn summary_label(&self) -> String {
        let mut files = 0;
        let mut folders = 0;
        let mut searches = 0;
        let mut thoughts = 0;

        for item in &self.items {
            match item {
                ExplorationItem::AnalyzedFile { .. } => files += 1,
                ExplorationItem::ExploredDirectory { .. } => folders += 1,
                ExplorationItem::Searched { .. } => searches += 1,
                ExplorationItem::Thought { .. } => thoughts += 1,
            }
        }

        let mut parts = Vec::new();
        if files > 0 {
            parts.push(format!("{} file{}", files, if files == 1 { "" } else { "s" }));
        }
        if folders > 0 {
            parts.push(format!("{} folder{}", folders, if folders == 1 { "" } else { "s" }));
        }
        if searches > 0 {
            parts.push(format!("{} search{}", searches, if searches == 1 { "" } else { "es" }));
        }

        if !parts.is_empty() {
            format!("Explored {}", parts.join(", "))
        } else if thoughts > 0 {
            "Thought for 1s".to_string()
        } else if self.is_active {
            "Exploring codebase...".to_string()
        } else {
            "Exploration".to_string()
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TrajectoryStep {
    Exploration(ExplorationGroup),
    FileEdit {
        path: String,
        lines_added: usize,
        lines_removed: usize,
    },
    Command {
        command: String,
    },
    Plan {
        title: String,
        steps: Vec<String>,
        #[serde(default)]
        expanded: bool,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiMessage {
    pub is_user: bool,
    pub content: String,
    pub code_snippet: Option<String>,
    pub thoughts: Option<String>,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub trajectory: Vec<TrajectoryStep>,
    #[serde(default)]
    pub is_working: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AiUiUpdate {
    Status(String),
    ToolStart {
        name: String,
        args: serde_json::Value,
    },
    ToolDone {
        name: String,
    },
    Done {
        reply: String,
        session_json: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct BreadcrumbPopup {
    pub dir_path: PathBuf,
    pub selected_name: String,
    pub popup_x: f32,
    pub popup_y: f32,
    pub expanded_dirs: std::collections::HashSet<PathBuf>,
}
