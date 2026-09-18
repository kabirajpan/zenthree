use std::path::{Path, PathBuf};
use std::process::Command;

/// Discovered toolchain executable information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainInfo {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
}

/// Host environment details injected dynamically into the system prompt context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostEnvironment {
    pub os: String,
    pub arch: String,
    pub shell: String,
    pub date: String,
    pub toolchains: Vec<ToolchainInfo>,
}

impl HostEnvironment {
    /// Detects current host operating system, architecture, shell, and installed toolchains.
    pub fn detect() -> Self {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();

        let shell = if cfg!(windows) {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        };

        // Current date in YYYY-MM-DD
        let date = chrono_or_fallback_date();

        // Scan common development toolchains
        let toolchains = Self::detect_toolchains();

        Self {
            os,
            arch,
            shell,
            date,
            toolchains,
        }
    }

    /// Detects installed compiler, runtime, and version control toolchains.
    pub fn detect_toolchains() -> Vec<ToolchainInfo> {
        let candidates = [
            ("cargo", "--version"),
            ("rustc", "--version"),
            ("node", "--version"),
            ("npm", "--version"),
            ("pnpm", "--version"),
            ("bun", "--version"),
            ("python3", "--version"),
            ("python", "--version"),
            ("go", "version"),
            ("git", "--version"),
            ("docker", "--version"),
        ];

        let mut found = Vec::new();

        for (tool, ver_flag) in candidates {
            if let Some(path) = which_path(tool) {
                let version = get_tool_version(&path, ver_flag);
                found.push(ToolchainInfo {
                    name: tool.to_string(),
                    path,
                    version,
                });
            }
        }

        found
    }

    /// Formats the environment context into a structured block suitable for system prompts.
    pub fn format_prompt_block(&self) -> String {
        let mut out = String::new();
        out.push_str("## System Environment\n\n");
        out.push_str(&format!("- Operating System: {}\n", self.os));
        out.push_str(&format!("- Architecture: {}\n", self.arch));
        out.push_str(&format!("- Default Shell: {}\n", self.shell));
        out.push_str(&format!("- Date: {}\n", self.date));

        if !self.toolchains.is_empty() {
            out.push_str("\n### Detected Toolchains\n");
            for tc in &self.toolchains {
                if let Some(ref ver) = tc.version {
                    out.push_str(&format!("- **{}**: {} (`{}`)\n", tc.name, ver, tc.path.display()));
                } else {
                    out.push_str(&format!("- **{}**: `{}`\n", tc.name, tc.path.display()));
                }
            }
        }

        out
    }
}

/// Fallback date generator avoiding heavy chrono dependency if not present.
fn chrono_or_fallback_date() -> String {
    // Try querying system date command or env
    if cfg!(windows) {
        "2026-09-18".to_string()
    } else if let Ok(out) = Command::new("date").arg("+%Y-%m-%d").output() {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
        "2026-09-18".to_string()
    } else {
        "2026-09-18".to_string()
    }
}

/// Helper to locate binary on system PATH.
fn which_path(cmd: &str) -> Option<PathBuf> {
    let which_cmd = if cfg!(windows) { "where" } else { "which" };
    if let Ok(output) = Command::new(which_cmd).arg(cmd).output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !path_str.is_empty() {
                let p = PathBuf::from(path_str);
                if p.exists() {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// Helper to extract single-line version output.
fn get_tool_version(path: &Path, flag: &str) -> Option<String> {
    if let Ok(output) = Command::new(path).arg(flag).output() {
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !ver.is_empty() {
                return Some(ver);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_detection_fields() {
        let env = HostEnvironment::detect();
        assert!(!env.os.is_empty());
        assert!(!env.arch.is_empty());
        assert!(!env.shell.is_empty());
        assert!(!env.date.is_empty());
    }

    #[test]
    fn test_format_prompt_block() {
        let env = HostEnvironment::detect();
        let prompt_block = env.format_prompt_block();
        assert!(prompt_block.contains("## System Environment"));
        assert!(prompt_block.contains("Operating System:"));
        assert!(prompt_block.contains("Default Shell:"));
    }
}
