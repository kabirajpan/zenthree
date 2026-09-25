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
    pub is_markdown_preview: bool,
}

impl EditorTab {
    pub fn is_markdown(&self) -> bool {
        self.name.ends_with(".md")
            || self.name.ends_with(".markdown")
            || self.path.as_ref()
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
                .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct UnsavedChangesDialog {
    /// Target workspace path to open (or None if closing workspace)
    pub target_workspace: Option<PathBuf>,
    /// List of unsaved file names or paths
    pub unsaved_files: Vec<String>,
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
        let count = self.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
        if count == 0 && (self.is_new_file || self.total_removed_lines() == 0) && !self.proposed_content.is_empty() {
            return self.proposed_content.lines().count().max(1);
        }
        count
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
    // ── Groq ─────────────────────────────────────────────────────────────────
    GroqGptOss120b,
    GroqGptOss20b,
    GroqQwen3_8_27b,
    // ── Gemini (alias / always-latest) ───────────────────────────────────────
    GeminiFlashLatest,
    GeminiProLatest,
    // ── Gemini 2.x ───────────────────────────────────────────────────────────
    Gemini25Flash,
    Gemini25Pro,
    Gemini25FlashLite,
    // ── Gemini 3.x ───────────────────────────────────────────────────────────
    Gemini35Flash,
    Gemini35FlashLite,
    Gemini36Flash,
    Gemini37Flash,
    Gemini38Flash,
}

impl AiModel {
    pub const ALL: [AiModel; 13] = [
        // Groq
        AiModel::GroqGptOss120b,
        AiModel::GroqGptOss20b,
        AiModel::GroqQwen3_8_27b,
        // Gemini aliases
        AiModel::GeminiFlashLatest,
        AiModel::GeminiProLatest,
        // Gemini 2.x
        AiModel::Gemini25Flash,
        AiModel::Gemini25Pro,
        AiModel::Gemini25FlashLite,
        // Gemini 3.x
        AiModel::Gemini35Flash,
        AiModel::Gemini35FlashLite,
        AiModel::Gemini36Flash,
        AiModel::Gemini37Flash,
        AiModel::Gemini38Flash,
    ];

    pub fn provider(&self) -> AiProvider {
        match self {
            AiModel::GroqGptOss120b | AiModel::GroqGptOss20b | AiModel::GroqQwen3_8_27b => {
                AiProvider::Groq
            }
            _ => AiProvider::Gemini,
        }
    }

    pub fn model_id(&self) -> &'static str {
        match self {
            // Groq
            AiModel::GroqGptOss120b    => "openai/gpt-oss-120b",
            AiModel::GroqGptOss20b     => "openai/gpt-oss-20b",
            AiModel::GroqQwen3_8_27b   => "qwen/qwen3.8-27b",
            // Gemini aliases
            AiModel::GeminiFlashLatest => "gemini-flash-latest",
            AiModel::GeminiProLatest   => "gemini-pro-latest",
            // Gemini 2.x
            AiModel::Gemini25Flash     => "gemini-2.5-flash",
            AiModel::Gemini25Pro       => "gemini-2.5-pro",
            AiModel::Gemini25FlashLite => "gemini-2.5-flash-lite",
            // Gemini 3.x
            AiModel::Gemini35Flash     => "gemini-3.5-flash",
            AiModel::Gemini35FlashLite => "gemini-3.5-flash-lite",
            AiModel::Gemini36Flash     => "gemini-3.6-flash",
            AiModel::Gemini37Flash     => "gemini-3.7-flash",
            AiModel::Gemini38Flash     => "gemini-3.8-flash",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            // Groq
            AiModel::GroqGptOss120b    => "GPT-OSS 120B",
            AiModel::GroqGptOss20b     => "GPT-OSS 20B",
            AiModel::GroqQwen3_8_27b   => "Qwen 3.8 27B",
            // Gemini aliases
            AiModel::GeminiFlashLatest => "Gemini Flash (Latest)",
            AiModel::GeminiProLatest   => "Gemini Pro (Latest)",
            // Gemini 2.x
            AiModel::Gemini25Flash     => "Gemini 2.5 Flash",
            AiModel::Gemini25Pro       => "Gemini 2.5 Pro",
            AiModel::Gemini25FlashLite => "Gemini 2.5 Flash Lite",
            // Gemini 3.x
            AiModel::Gemini35Flash     => "Gemini 3.5 Flash",
            AiModel::Gemini35FlashLite => "Gemini 3.5 Flash Lite",
            AiModel::Gemini36Flash     => "Gemini 3.6 Flash",
            AiModel::Gemini37Flash     => "Gemini 3.7 Flash",
            AiModel::Gemini38Flash     => "Gemini 3.8 Flash",
        }
    }
}

impl Default for AiModel {
    fn default() -> Self {
        AiModel::Gemini35Flash
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgentMode {
    Agent,
    Chat,
    Plan,
}

impl AgentMode {
    pub const ALL: [AgentMode; 3] = [
        AgentMode::Agent,
        AgentMode::Chat,
        AgentMode::Plan,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentMode::Agent => "Agent",
            AgentMode::Chat => "Chat",
            AgentMode::Plan => "Plan",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AgentMode::Agent => "✦",
            AgentMode::Chat => "💬",
            AgentMode::Plan => "📋",
        }
    }

    #[allow(dead_code)]
    pub fn description(&self) -> &'static str {
        match self {
            AgentMode::Agent => "Autonomous edits, creates & commands",
            AgentMode::Chat => "Conversational Q&A, no file edits",
            AgentMode::Plan => "Step-by-step implementation planning",
        }
    }
}

impl Default for AgentMode {
    fn default() -> Self {
        AgentMode::Agent
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
        #[serde(default)]
        start_line: Option<usize>,
        #[serde(default)]
        end_line: Option<usize>,
        #[serde(default)]
        symbol: Option<String>,
        #[serde(default)]
        is_peek: bool,
    },
    BatchRead {
        paths: Vec<String>,
    },
    Searched {
        query: String,
        #[serde(default)]
        scope: Option<String>,
    },
    ExploredDirectory {
        path: String,
    },
    ActivatedSkill {
        name: String,
    },
    ExecutedCommand {
        command: String,
    },
    CheckedDiagnostics,
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
        let mut skills = 0;
        let mut commands = 0;
        let mut thoughts = 0;

        for item in &self.items {
            match item {
                ExplorationItem::AnalyzedFile { .. } => files += 1,
                ExplorationItem::BatchRead { paths } => files += paths.len(),
                ExplorationItem::ExploredDirectory { .. } => folders += 1,
                ExplorationItem::Searched { .. } => searches += 1,
                ExplorationItem::ActivatedSkill { .. } => skills += 1,
                ExplorationItem::ExecutedCommand { .. } => commands += 1,
                ExplorationItem::CheckedDiagnostics => {},
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
        if skills > 0 {
            parts.push(format!("{} skill{}", skills, if skills == 1 { "" } else { "s" }));
        }
        if commands > 0 {
            parts.push(format!("{} command{}", commands, if commands == 1 { "" } else { "s" }));
        }

        if !parts.is_empty() {
            let verb = if self.is_active { "Exploring" } else { "Explored" };
            format!("{} {}", verb, parts.join(", "))
        } else if thoughts > 0 {
            if self.is_active {
                "Thinking...".to_string()
            } else {
                "Thought for 1s".to_string()
            }
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

impl AiMessage {
    pub fn is_error(&self) -> bool {
        !self.is_user
            && (self.content.starts_with("Agent error:")
                || self.content.starts_with("Error:")
                || self.content.contains("RESOURCE_EXHAUSTED")
                || self.content.contains("429 Too Many Requests")
                || self.content.contains("Quota exceeded"))
    }

    pub fn parse_error_details(&self) -> (String, String, Option<String>) {
        let text = self.content.trim();
        let is_quota = text.contains("RESOURCE_EXHAUSTED")
            || text.contains("429")
            || text.contains("quota")
            || text.contains("Quota");

        if is_quota {
            let retry_delay = if let Some(pos) = text.find("\"retryDelay\":") {
                let rest = &text[pos + 13..];
                let end = rest.find('"').and_then(|p1| {
                    let next = &rest[p1 + 1..];
                    next.find('"').map(|p2| &rest[p1 + 1..p1 + 1 + p2])
                });
                end.map(|s| s.to_string())
            } else {
                None
            };

            let title = "API Quota Exceeded (429)".to_string();
            let desc = if let Some(delay) = retry_delay {
                format!(
                    "Resource quota has been exhausted. Please wait {} before retrying, or switch to another model.",
                    delay
                )
            } else {
                "Resource quota has been exhausted for this model. Please wait a moment or switch to another model.".to_string()
            };

            (title, desc, Some(text.to_string()))
        } else if text.starts_with("Agent error:") {
            let err_body = text.strip_prefix("Agent error:").unwrap_or(text).trim();
            ("Agent Error".to_string(), err_body.to_string(), Some(text.to_string()))
        } else if text.starts_with("Error:") {
            let err_body = text.strip_prefix("Error:").unwrap_or(text).trim();
            ("Request Error".to_string(), err_body.to_string(), Some(text.to_string()))
        } else {
            ("Error".to_string(), text.to_string(), None)
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub project_path: String,
    pub project_name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub message_count: usize,
}

impl SessionSummary {
    pub fn relative_time_string(&self) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let diff = now.saturating_sub(self.updated_at);
        if diff < 60 {
            "just now".to_string()
        } else if diff < 3600 {
            format!("{}m", diff / 60)
        } else if diff < 86400 {
            format!("{}h", diff / 3600)
        } else if diff < 604800 {
            format!("{}d", diff / 86400)
        } else {
            format!("{}w", diff / 604800)
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedAiSession {
    pub id: String,
    pub title: String,
    pub project_path: String,
    pub project_name: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<AiMessage>,
    pub agent_mode: AgentMode,
    pub model: AiModel,
    pub session_json: Option<String>,
}

pub use super::ai_settings::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum EditorSettingsTab {
    #[default]
    Appearance,
    Editor,
    Terminal,
    Agent,
    Keymaps,
}

impl EditorSettingsTab {
    pub const ALL: [Self; 5] = [
        Self::Appearance,
        Self::Editor,
        Self::Terminal,
        Self::Agent,
        Self::Keymaps,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Editor => "Editor",
            Self::Terminal => "Terminal",
            Self::Agent => "AI & Agent",
            Self::Keymaps => "Keymaps",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Appearance => "\u{f53f}", // palette
            Self::Editor => "\u{f121}",     // code
            Self::Terminal => "\u{f120}",   // terminal
            Self::Agent => "\u{f005}",      // star / sparkle
            Self::Keymaps => "\u{f11c}",    // keyboard
        }
    }
}


