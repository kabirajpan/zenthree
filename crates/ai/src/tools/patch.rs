use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use super::matcher;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchReplaceBlock {
    pub search: String,
    pub replace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileEditAction {
    CreateOrOverwrite(String),
    SurgicalPatch(Vec<SearchReplaceBlock>),
}

#[derive(Debug, Clone)]
pub struct TargetFileModification {
    pub file_path: PathBuf,
    pub action: FileEditAction,
}

impl TargetFileModification {
    pub fn new_overwrite(file_path: PathBuf, code: String) -> Self {
        Self {
            file_path,
            action: FileEditAction::CreateOrOverwrite(code),
        }
    }

    pub fn new_patch(file_path: PathBuf, patches: Vec<SearchReplaceBlock>) -> Self {
        Self {
            file_path,
            action: FileEditAction::SurgicalPatch(patches),
        }
    }

    pub fn apply_to(&self, original: &str) -> Result<String, String> {
        match &self.action {
            FileEditAction::CreateOrOverwrite(code) => Ok(code.clone()),
            FileEditAction::SurgicalPatch(blocks) => apply_search_replace(original, blocks),
        }
    }
}

pub fn apply_search_replace(
    original: &str,
    blocks: &[SearchReplaceBlock],
) -> Result<String, String> {
    matcher::match_and_replace(original, blocks)
}
