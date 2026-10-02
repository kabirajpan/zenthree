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
    pub just_opened: bool,
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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitFileStatusCode {
    Modified,
    Added,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

pub type GitStatus = GitFileStatusCode;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitChangeItem {
    pub path: std::path::PathBuf,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AiProvider {
    Antigravity,
    Groq,
    Gemini,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AiModel {
    // ── Google Antigravity (External Agent / OAuth) ───────────────────────────
    AntigravityGemini38FlashHigh,
    AntigravityGemini38FlashMedium,
    AntigravityGemini38FlashLow,
    AntigravityGemini37FlashHigh,
    AntigravityGeminiProAgent,
    AntigravityGemini31ProLow,
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
    pub const ALL: [AiModel; 19] = [
        // Google Antigravity
        AiModel::AntigravityGemini38FlashHigh,
        AiModel::AntigravityGemini38FlashMedium,
        AiModel::AntigravityGemini38FlashLow,
        AiModel::AntigravityGemini37FlashHigh,
        AiModel::AntigravityGeminiProAgent,
        AiModel::AntigravityGemini31ProLow,
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

    pub const ANTIGRAVITY_MODELS: [AiModel; 6] = [
        AiModel::AntigravityGemini38FlashHigh,
        AiModel::AntigravityGemini38FlashMedium,
        AiModel::AntigravityGemini38FlashLow,
        AiModel::AntigravityGemini37FlashHigh,
        AiModel::AntigravityGeminiProAgent,
        AiModel::AntigravityGemini31ProLow,
    ];

    pub const ZENE_MODELS: [AiModel; 13] = [
        AiModel::GroqGptOss120b,
        AiModel::GroqGptOss20b,
        AiModel::GroqQwen3_8_27b,
        AiModel::GeminiFlashLatest,
        AiModel::GeminiProLatest,
        AiModel::Gemini25Flash,
        AiModel::Gemini25Pro,
        AiModel::Gemini25FlashLite,
        AiModel::Gemini35Flash,
        AiModel::Gemini35FlashLite,
        AiModel::Gemini36Flash,
        AiModel::Gemini37Flash,
        AiModel::Gemini38Flash,
    ];

    pub fn is_antigravity(&self) -> bool {
        self.provider() == AiProvider::Antigravity
    }

    pub fn provider(&self) -> AiProvider {
        match self {
            AiModel::AntigravityGemini38FlashHigh
            | AiModel::AntigravityGemini38FlashMedium
            | AiModel::AntigravityGemini38FlashLow
            | AiModel::AntigravityGemini37FlashHigh
            | AiModel::AntigravityGeminiProAgent
            | AiModel::AntigravityGemini31ProLow => AiProvider::Antigravity,
            AiModel::GroqGptOss120b | AiModel::GroqGptOss20b | AiModel::GroqQwen3_8_27b => {
                AiProvider::Groq
            }
            _ => AiProvider::Gemini,
        }
    }

    pub fn model_id(&self) -> &'static str {
        match self {
            // Google Antigravity
            AiModel::AntigravityGemini38FlashHigh   => "gemini-3.8-flash-high",
            AiModel::AntigravityGemini38FlashMedium => "gemini-3.8-flash-medium",
            AiModel::AntigravityGemini38FlashLow    => "gemini-3.8-flash-low",
            AiModel::AntigravityGemini37FlashHigh   => "gemini-3.7-flash-high",
            AiModel::AntigravityGeminiProAgent      => "gemini-pro-agent",
            AiModel::AntigravityGemini31ProLow      => "gemini-3.1-pro-low",
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
            // Google Antigravity
            AiModel::AntigravityGemini38FlashHigh   => "Gemini 3.8 Flash (High)",
            AiModel::AntigravityGemini38FlashMedium => "Gemini 3.8 Flash (Medium)",
            AiModel::AntigravityGemini38FlashLow    => "Gemini 3.8 Flash (Low)",
            AiModel::AntigravityGemini37FlashHigh   => "Gemini 3.7 Flash (High)",
            AiModel::AntigravityGeminiProAgent      => "Gemini 3.1 Pro (High)",
            AiModel::AntigravityGemini31ProLow      => "Gemini 3.1 Pro (Low)",
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

    #[allow(dead_code)]
    pub fn short_name(&self) -> &'static str {
        match self {
            AiModel::AntigravityGemini38FlashHigh   => "Gemini 3.8 Flash",
            AiModel::AntigravityGemini38FlashMedium => "Gemini 3.8 Flash",
            AiModel::AntigravityGemini38FlashLow    => "Gemini 3.8 Flash",
            AiModel::AntigravityGemini37FlashHigh   => "Gemini 3.7 Flash",
            AiModel::AntigravityGeminiProAgent      => "Gemini 3.1 Pro",
            AiModel::AntigravityGemini31ProLow      => "Gemini 3.1 Pro",
            other => other.display_name(),
        }
    }
}

impl Default for AiModel {
    fn default() -> Self {
        AiModel::Gemini35Flash
    }
}

impl std::fmt::Display for AiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
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
            AgentMode::Agent => "\u{f0833}",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum SelectedAgent {
    #[default]
    Zene,
    Antigravity,
}

#[allow(dead_code)]
impl SelectedAgent {
    pub const ALL: [Self; 2] = [
        Self::Zene,
        Self::Antigravity,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Zene => "ZENE",
            Self::Antigravity => "Google Antigravity",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::Zene => "ZENE",
            Self::Antigravity => "Antigravity",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Zene => "\u{f0833}",
            Self::Antigravity => "\u{f0833}",
        }
    }

    pub fn is_external(&self) -> bool {
        matches!(self, Self::Antigravity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum AntigravityMode {
    #[default]
    Default,
    AutoEdit,
    Yolo,
}

#[allow(dead_code)]
impl AntigravityMode {
    pub const ALL: [Self; 3] = [
        Self::Default,
        Self::AutoEdit,
        Self::Yolo,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::AutoEdit => "auto_edit",
            Self::Yolo => "yolo",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Default => "\u{f0833}",
            Self::AutoEdit => "🛠",
            Self::Yolo => "🚀",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Default => "Collaborative with safety checkpoints",
            Self::AutoEdit => "Direct file mutations applied automatically",
            Self::Yolo => "Autonomous execution with auto-approved tools",
        }
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
    Git,
    Extensions,
    Notes,
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
    pub just_opened: bool,
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
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    BatchRead {
        paths: Vec<String>,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    Searched {
        query: String,
        #[serde(default)]
        scope: Option<String>,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    ExploredDirectory {
        path: String,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    ActivatedSkill {
        name: String,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    ExecutedCommand {
        command: String,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    CheckedDiagnostics {
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    WrittenFile {
        path: String,
        #[serde(default)]
        lines_added: usize,
        #[serde(default)]
        lines_removed: usize,
        #[serde(default)]
        is_new: bool,
        #[serde(default)]
        output: Option<String>,
        #[serde(default)]
        expanded: bool,
    },
    Thought {
        summary: String,
        duration_secs: Option<u64>,
    },
}

#[allow(dead_code)]
impl ExplorationItem {
    pub fn is_expanded(&self) -> bool {
        match self {
            ExplorationItem::AnalyzedFile { expanded, .. } => *expanded,
            ExplorationItem::BatchRead { expanded, .. } => *expanded,
            ExplorationItem::Searched { expanded, .. } => *expanded,
            ExplorationItem::ExploredDirectory { expanded, .. } => *expanded,
            ExplorationItem::ActivatedSkill { expanded, .. } => *expanded,
            ExplorationItem::ExecutedCommand { expanded, .. } => *expanded,
            ExplorationItem::CheckedDiagnostics { expanded, .. } => *expanded,
            ExplorationItem::WrittenFile { expanded, .. } => *expanded,
            ExplorationItem::Thought { .. } => false,
        }
    }

    pub fn toggle_expanded(&mut self) {
        match self {
            ExplorationItem::AnalyzedFile { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::BatchRead { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::Searched { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::ExploredDirectory { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::ActivatedSkill { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::ExecutedCommand { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::CheckedDiagnostics { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::WrittenFile { expanded, .. } => *expanded = !*expanded,
            ExplorationItem::Thought { .. } => {},
        }
    }

    pub fn output(&self) -> Option<&str> {
        match self {
            ExplorationItem::AnalyzedFile { output, .. } => output.as_deref(),
            ExplorationItem::BatchRead { output, .. } => output.as_deref(),
            ExplorationItem::Searched { output, .. } => output.as_deref(),
            ExplorationItem::ExploredDirectory { output, .. } => output.as_deref(),
            ExplorationItem::ActivatedSkill { output, .. } => output.as_deref(),
            ExplorationItem::ExecutedCommand { output, .. } => output.as_deref(),
            ExplorationItem::CheckedDiagnostics { output, .. } => output.as_deref(),
            ExplorationItem::WrittenFile { output, .. } => output.as_deref(),
            ExplorationItem::Thought { .. } => None,
        }
    }

    pub fn set_output(&mut self, text: String) {
        match self {
            ExplorationItem::AnalyzedFile { output, .. } => *output = Some(text),
            ExplorationItem::BatchRead { output, .. } => *output = Some(text),
            ExplorationItem::Searched { output, .. } => *output = Some(text),
            ExplorationItem::ExploredDirectory { output, .. } => *output = Some(text),
            ExplorationItem::ActivatedSkill { output, .. } => *output = Some(text),
            ExplorationItem::ExecutedCommand { output, .. } => *output = Some(text),
            ExplorationItem::CheckedDiagnostics { output, .. } => *output = Some(text),
            ExplorationItem::WrittenFile { output, .. } => *output = Some(text),
            ExplorationItem::Thought { .. } => {},
        }
    }
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
                ExplorationItem::BatchRead { paths, .. } => files += paths.len(),
                ExplorationItem::ExploredDirectory { .. } => folders += 1,
                ExplorationItem::Searched { .. } => searches += 1,
                ExplorationItem::ActivatedSkill { .. } => skills += 1,
                ExplorationItem::ExecutedCommand { .. } => commands += 1,
                ExplorationItem::CheckedDiagnostics { .. } => {},
                ExplorationItem::WrittenFile { .. } => files += 1,
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
        #[serde(default)]
        is_new: bool,
        #[serde(default)]
        expanded: bool,
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
    pub thoughts_expanded: bool,
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
    Thought(String),
    TextDelta(String),
    ToolStart {
        name: String,
        args: serde_json::Value,
    },
    ToolDone {
        name: String,
        result: Option<serde_json::Value>,
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


