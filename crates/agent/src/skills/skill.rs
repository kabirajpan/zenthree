use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub tags: Vec<String>,
    pub path: PathBuf,
    pub instructions: String,
}

impl Skill {
    pub fn metadata(&self) -> SkillMetadata {
        SkillMetadata {
            name: self.name.clone(),
            description: self.description.clone(),
            version: self.version.clone(),
            tags: self.tags.clone(),
        }
    }
}

#[derive(Debug)]
pub enum SkillError {
    Io(std::io::Error),
    InvalidFrontmatter(String),
    InvalidMetadata(String),
    DuplicateName(String),
    MissingSkill(String),
}

impl fmt::Display for SkillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "skill I/O error: {}", error),
            Self::InvalidFrontmatter(error) => write!(f, "invalid skill frontmatter: {}", error),
            Self::InvalidMetadata(error) => write!(f, "invalid skill metadata: {}", error),
            Self::DuplicateName(name) => write!(f, "duplicate skill name: {}", name),
            Self::MissingSkill(name) => write!(f, "skill not found: {}", name),
        }
    }
}

impl std::error::Error for SkillError {}

impl From<std::io::Error> for SkillError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
