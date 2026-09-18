use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Maximum size for a single SKILL.md file (100KB, matching Zed standard).
pub const MAX_SKILL_FILE_SIZE: usize = 100 * 1024;

/// Standard filename for skill definitions.
pub const SKILL_FILE_NAME: &str = "SKILL.md";

static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").unwrap()
});

/// Origin scope where a skill was loaded from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillScope {
    /// Built into the Zenthree binary (always available, lowest shadow priority).
    Builtin,
    /// Global personal skill (~/.agents/skills/).
    Global,
    /// Project-specific workspace skill (<workspace>/.agents/skills/).
    Workspace,
}

/// Metadata parsed from the YAML frontmatter of a SKILL.md file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub disable_model_invocation: bool,
}

/// A fully loaded or discovered agent skill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Skill {
    pub metadata: SkillMetadata,
    pub scope: SkillScope,
    pub directory_path: PathBuf,
    pub file_path: PathBuf,
    pub body: String,
}

impl Skill {
    /// Parses a skill from raw SKILL.md content and directory location.
    pub fn parse(
        content: &str,
        directory_path: impl AsRef<Path>,
        scope: SkillScope,
    ) -> Result<Self, String> {
        let dir = directory_path.as_ref().to_path_buf();
        let file_path = dir.join(SKILL_FILE_NAME);

        if content.len() > MAX_SKILL_FILE_SIZE {
            return Err(format!(
                "Skill file exceeds maximum permitted size of 100KB (actual: {} bytes)",
                content.len()
            ));
        }

        let (metadata, body) = parse_frontmatter(content)?;

        // Validate skill name against spec: 1-64 chars, lowercase alphanumeric with single hyphens
        if metadata.name.is_empty() || metadata.name.len() > 64 {
            return Err(format!(
                "Invalid skill name '{}': length must be between 1 and 64 characters",
                metadata.name
            ));
        }

        if !NAME_REGEX.is_match(&metadata.name) {
            return Err(format!(
                "Invalid skill name '{}': must be lowercase letters, numbers, and single hyphens (e.g. 'rust-patterns')",
                metadata.name
            ));
        }

        // Validate description
        if metadata.description.is_empty() || metadata.description.len() > 1024 {
            return Err(format!(
                "Invalid skill description for '{}': length must be between 1 and 1024 characters",
                metadata.name
            ));
        }

        Ok(Self {
            metadata,
            scope,
            directory_path: dir,
            file_path,
            body,
        })
    }

    pub fn name(&self) -> &str {
        &self.metadata.name
    }

    pub fn description(&self) -> &str {
        &self.metadata.description
    }

    pub fn is_model_invocable(&self) -> bool {
        !self.metadata.disable_model_invocation
    }
}

/// Helper to parse YAML frontmatter delimited by `---` and extract body markdown.
fn parse_frontmatter(content: &str) -> Result<(SkillMetadata, String), String> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return Err("SKILL.md must begin with YAML frontmatter starting with '---'".to_string());
    }

    let after_first_delimiter = &trimmed[3..];
    let end_idx = after_first_delimiter
        .find("\n---")
        .ok_or_else(|| "SKILL.md frontmatter missing closing '---' delimiter".to_string())?;

    let frontmatter_text = &after_first_delimiter[..end_idx];
    let body_text = &after_first_delimiter[end_idx + 4..].trim_start_matches('\n');

    let mut name = String::new();
    let mut description = String::new();
    let mut disable_model_invocation = false;

    // Parse simple key-value YAML lines
    for line in frontmatter_text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = line.split_once(':') {
            let key = k.trim().to_lowercase();
            let mut val = v.trim().to_string();

            // Strip surrounding quotes
            if (val.starts_with('\'') && val.ends_with('\''))
                || (val.starts_with('"') && val.ends_with('"'))
            {
                if val.len() >= 2 {
                    val = val[1..val.len() - 1].to_string();
                }
            }

            match key.as_str() {
                "name" => name = val,
                "description" => description = val,
                "disable-model-invocation" | "disable_model_invocation" => {
                    disable_model_invocation = val.eq_ignore_ascii_case("true");
                }
                _ => {}
            }
        }
    }

    if name.is_empty() {
        return Err("Required frontmatter field 'name' is missing in SKILL.md".to_string());
    }

    if description.is_empty() {
        return Err("Required frontmatter field 'description' is missing in SKILL.md".to_string());
    }

    Ok((
        SkillMetadata {
            name,
            description,
            disable_model_invocation,
        },
        body_text.to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_skill_parsing() {
        let content = r#"---
name: rust-patterns
description: Guidelines and idioms for high-performance Rust programming.
disable-model-invocation: false
---

# Rust Patterns Body
Always use Result and Option idiomatically.
"#;
        let skill = Skill::parse(content, "/test/path/rust-patterns", SkillScope::Workspace).unwrap();
        assert_eq!(skill.name(), "rust-patterns");
        assert_eq!(skill.description(), "Guidelines and idioms for high-performance Rust programming.");
        assert!(skill.is_model_invocable());
        assert!(skill.body.contains("Always use Result and Option"));
    }

    #[test]
    fn test_rejects_invalid_names() {
        let bad_names = ["Rust_Patterns", "rust--patterns", "-rust", "rust-", "has space"];
        for bad in bad_names {
            let content = format!("---\nname: {}\ndescription: test\n---\nbody", bad);
            assert!(Skill::parse(&content, "/test", SkillScope::Workspace).is_err());
        }
    }

    #[test]
    fn test_supports_quoted_fields() {
        let content = r#"---
name: 'component-generator'
description: "Generates UI components matching project style."
---
# Instructions
"#;
        let skill = Skill::parse(content, "/test", SkillScope::Global).unwrap();
        assert_eq!(skill.name(), "component-generator");
        assert_eq!(skill.description(), "Generates UI components matching project style.");
    }
}
