use regex::Regex;
use std::sync::LazyLock;

/// Hardcoded security denial message explaining why the command is unconditionally blocked.
pub const HARDCODED_SECURITY_DENIAL: &str =
    "Blocked by built-in security rule. This operation is considered too harmful to be executed and cannot be overridden by any settings.";

static DENIAL_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    const FLAGS: &str = r"(--[a-zA-Z0-9][-a-zA-Z0-9_]*(=[^\s]*)?\s+|-[a-zA-Z]+\s+)*";
    const TRAILING_FLAGS: &str = r"(\s+--[a-zA-Z0-9][-a-zA-Z0-9_]*(=[^\s]*)?|\s+-[a-zA-Z]+)*\s*$";

    vec![
        // 1. Recursive deletion of root: rm -rf / or rm -rf /* or rm / -rf
        (
            Regex::new(&format!(r"\brm\s+{FLAGS}(--\s+)?/\*?{TRAILING_FLAGS}")).unwrap(),
            "Recursive deletion of root directory",
        ),
        // 2. Recursive deletion of home: rm -rf ~ or rm -rf ~/ or rm -rf ~/*
        (
            Regex::new(&format!(r"\brm\s+{FLAGS}(--\s+)?~/?\*?{TRAILING_FLAGS}")).unwrap(),
            "Recursive deletion of home directory",
        ),
        // 3. Recursive deletion of home via $HOME or ${HOME}
        (
            Regex::new(&format!(r"\brm\s+{FLAGS}(--\s+)?(\$HOME|\$\{{HOME\}})/?(\*)?{TRAILING_FLAGS}")).unwrap(),
            "Recursive deletion of $HOME directory",
        ),
        // 4. Recursive deletion of current working directory: rm -rf . or rm -rf ./*
        (
            Regex::new(&format!(r"\brm\s+{FLAGS}(--\s+)?\./?\*?{TRAILING_FLAGS}")).unwrap(),
            "Recursive deletion of current directory",
        ),
        // 5. Recursive deletion of parent directory: rm -rf .. or rm -rf ../*
        (
            Regex::new(&format!(r"\brm\s+{FLAGS}(--\s+)?\.\./?\*?{TRAILING_FLAGS}")).unwrap(),
            "Recursive deletion of parent directory",
        ),
        // 6. Raw block device formatting / writes: dd if=... of=/dev/sd* or /dev/nvme*
        (
            Regex::new(r"\bdd\s+.*of=/dev/(sd[a-z]|nvme[0-9]n[0-9]|hd[a-z]|vd[a-z]|loop[0-9])").unwrap(),
            "Direct raw block device write via dd",
        ),
        (
            Regex::new(r"\bmkfs(\.[a-zA-Z0-9]+)?\s+/dev/").unwrap(),
            "Filesystem format operation via mkfs",
        ),
        // 7. Partition tools targeting raw devices
        (
            Regex::new(r"\b(fdisk|gdisk|parted)\s+/dev/").unwrap(),
            "Raw disk partitioning tool invocation",
        ),
        // 8. Classic shell fork bomb
        (
            Regex::new(r":\(\)\s*\{\s*:\|:&\s*\};:").unwrap(),
            "Shell fork bomb denial of service",
        ),
        // 9. Host shutdown / reboot commands
        (
            Regex::new(r"\b(shutdown\s+-h|\bpoweroff\b|\breboot\b|\binit\s+[06]\b)").unwrap(),
            "System shutdown or reboot command",
        ),
        // 10. Irreversible git clean
        (
            Regex::new(r"\bgit\s+clean\s+-[a-zA-Z]*f[a-zA-Z]*d[a-zA-Z]*x").unwrap(),
            "Irreversible deletion of untracked files via git clean -fdx",
        ),
    ]
});

/// Evaluator for hardcoded, unbypassable security rules.
#[derive(Debug, Clone, Default)]
pub struct HardcodedSecurityRules;

impl HardcodedSecurityRules {
    /// Checks if a command string violates any hardcoded security rules.
    /// Returns the specific violation reason if blocked, or None if safe.
    pub fn check_command(command: &str) -> Option<&'static str> {
        let trimmed = command.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 1. Direct regex match on full command string
        for (pattern, reason) in DENIAL_PATTERNS.iter() {
            if pattern.is_match(trimmed) {
                return Some(reason);
            }
        }

        // 2. Path normalization bypass check for rm commands
        // Catches tricks like `rm -rf /tmp/../../` which resolves to `/`
        if trimmed.starts_with("rm ") || trimmed.starts_with("rm\t") {
            if let Some(reason) = Self::check_rm_traversal(trimmed) {
                return Some(reason);
            }
        }

        None
    }

    /// Checks if an rm command uses relative traversal (`..`) to target root, home, or parent.
    fn check_rm_traversal(command: &str) -> Option<&'static str> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        for part in parts.iter().skip(1) {
            if part.starts_with('-') {
                continue;
            }

            let path = std::path::Path::new(part);
            let mut depth: isize = 0;
            let mut hits_root = false;

            for comp in path.components() {
                match comp {
                    std::path::Component::RootDir => hits_root = true,
                    std::path::Component::Normal(_) => depth += 1,
                    std::path::Component::ParentDir => {
                        depth -= 1;
                        if depth <= 0 && hits_root {
                            return Some("Path traversal resolving to root directory in rm command");
                        }
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// Returns true if the command matches any hardcoded denial rule.
    pub fn is_denied(command: &str) -> bool {
        Self::check_command(command).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocks_rm_root() {
        assert!(HardcodedSecurityRules::is_denied("rm -rf /"));
        assert!(HardcodedSecurityRules::is_denied("rm -rfv /"));
        assert!(HardcodedSecurityRules::is_denied("rm / -rf"));
        assert!(HardcodedSecurityRules::is_denied("rm -rf /*"));
        assert!(HardcodedSecurityRules::is_denied("rm --recursive --force /"));
    }

    #[test]
    fn test_blocks_rm_home() {
        assert!(HardcodedSecurityRules::is_denied("rm -rf ~"));
        assert!(HardcodedSecurityRules::is_denied("rm -rf ~/"));
        assert!(HardcodedSecurityRules::is_denied("rm -rf $HOME"));
        assert!(HardcodedSecurityRules::is_denied("rm -rf ${HOME}"));
    }

    #[test]
    fn test_blocks_rm_relative_parent_and_current() {
        assert!(HardcodedSecurityRules::is_denied("rm -rf ."));
        assert!(HardcodedSecurityRules::is_denied("rm -rf ./"));
        assert!(HardcodedSecurityRules::is_denied("rm -rf .."));
        assert!(HardcodedSecurityRules::is_denied("rm -rf ../"));
    }

    #[test]
    fn test_blocks_traversal_to_root() {
        assert!(HardcodedSecurityRules::is_denied("rm -rf /tmp/../../"));
        assert!(HardcodedSecurityRules::is_denied("rm -rf /var/log/../../../"));
    }

    #[test]
    fn test_blocks_raw_disk_format_and_dd() {
        assert!(HardcodedSecurityRules::is_denied("dd if=/dev/zero of=/dev/sda bs=1M"));
        assert!(HardcodedSecurityRules::is_denied("mkfs.ext4 /dev/nvme0n1p1"));
        assert!(HardcodedSecurityRules::is_denied("fdisk /dev/sdb"));
    }

    #[test]
    fn test_blocks_fork_bomb() {
        assert!(HardcodedSecurityRules::is_denied(":(){ :|:& };:"));
    }

    #[test]
    fn test_blocks_git_clean_fdx() {
        assert!(HardcodedSecurityRules::is_denied("git clean -fdx"));
    }

    #[test]
    fn test_allows_benign_commands() {
        assert!(!HardcodedSecurityRules::is_denied("cargo test --workspace"));
        assert!(!HardcodedSecurityRules::is_denied("rm target/debug/build.log"));
        assert!(!HardcodedSecurityRules::is_denied("git status"));
        assert!(!HardcodedSecurityRules::is_denied("echo 'hello world'"));
    }
}
