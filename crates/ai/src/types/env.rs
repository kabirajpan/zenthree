use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectLanguage {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    C,
    Cpp,
    Zig,
    Java,
    Kotlin,
    Swift,
    Ruby,
    PHP,
    Elixir,
    Dart,
    Other(String),
}

impl ProjectLanguage {
    pub fn display_name(&self) -> &str {
        match self {
            ProjectLanguage::Rust => "Rust",
            ProjectLanguage::TypeScript => "TypeScript",
            ProjectLanguage::JavaScript => "JavaScript",
            ProjectLanguage::Python => "Python",
            ProjectLanguage::Go => "Go",
            ProjectLanguage::C => "C",
            ProjectLanguage::Cpp => "C++",
            ProjectLanguage::Zig => "Zig",
            ProjectLanguage::Java => "Java",
            ProjectLanguage::Kotlin => "Kotlin",
            ProjectLanguage::Swift => "Swift",
            ProjectLanguage::Ruby => "Ruby",
            ProjectLanguage::PHP => "PHP",
            ProjectLanguage::Elixir => "Elixir",
            ProjectLanguage::Dart => "Dart",
            ProjectLanguage::Other(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectEnvironment {
    pub language: ProjectLanguage,
    pub build_system: String,
    pub compiler_check_cmd: Option<Vec<String>>,
    pub test_cmd: Option<Vec<String>>,
    pub details: String,
}

impl ProjectEnvironment {
    pub fn is_unidentified(&self) -> bool {
        matches!(self.language, ProjectLanguage::Other(_))
            && self.compiler_check_cmd.is_none()
            && self.test_cmd.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDetectionContext {
    pub file_tree_sample: Vec<String>,
    pub file_extensions: Vec<(String, usize)>,
    pub manifest_candidates: Vec<String>,
    pub readme_sample: Option<String>,
}
