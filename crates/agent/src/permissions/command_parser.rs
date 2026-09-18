/// POSIX shell command chain parser and substitution detector.
/// Splits compound commands (e.g. `cmd1 && cmd2 || cmd3 ; cmd4 | cmd5`) into discrete
/// sub-commands for individual security and permission evaluation.
#[derive(Debug, Clone, Default)]
pub struct ShellCommandParser;

impl ShellCommandParser {
    /// Extracts individual sub-commands from a compound shell command line,
    /// splitting by `&&`, `||`, `;`, `|`, and newlines while respecting quotes.
    pub fn extract_subcommands(command_line: &str) -> Vec<String> {
        let mut commands = Vec::new();
        let mut current = String::new();

        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;

        let chars: Vec<char> = command_line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];

            if escaped {
                current.push(c);
                escaped = false;
                i += 1;
                continue;
            }

            if c == '\\' && !in_single_quote {
                escaped = true;
                current.push(c);
                i += 1;
                continue;
            }

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                current.push(c);
                i += 1;
                continue;
            }

            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                current.push(c);
                i += 1;
                continue;
            }

            // Outside quotes: check for command chain separators
            if !in_single_quote && !in_double_quote {
                // Check for double character operators: `&&` or `||`
                if i + 1 < len {
                    let next = chars[i + 1];
                    if (c == '&' && next == '&') || (c == '|' && next == '|') {
                        let trimmed = current.trim();
                        if !trimmed.is_empty() {
                            commands.push(trimmed.to_string());
                        }
                        current.clear();
                        i += 2;
                        continue;
                    }
                }

                // Check for single character operators: `;`, `|`, or newline
                if c == ';' || c == '|' || c == '\n' {
                    let trimmed = current.trim();
                    if !trimmed.is_empty() {
                        commands.push(trimmed.to_string());
                    }
                    current.clear();
                    i += 1;
                    continue;
                }
            }

            current.push(c);
            i += 1;
        }

        let trimmed = current.trim();
        if !trimmed.is_empty() {
            commands.push(trimmed.to_string());
        }

        commands
    }

    /// Detects whether a command line contains shell substitutions or interpolations:
    /// e.g. `$VAR`, `${VAR}`, `$(...)`, backticks, `$((...))`, `<(...)`, `>(...)`.
    pub fn has_substitutions(command_line: &str) -> bool {
        !Self::find_substitutions(command_line).is_empty()
    }

    /// Finds all shell substitutions or variable interpolations present in the command line.
    pub fn find_substitutions(command_line: &str) -> Vec<String> {
        let mut substitutions = Vec::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escaped = false;

        let chars: Vec<char> = command_line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let c = chars[i];

            if escaped {
                escaped = false;
                i += 1;
                continue;
            }

            if c == '\\' && !in_single_quote {
                escaped = true;
                i += 1;
                continue;
            }

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                i += 1;
                continue;
            }

            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                i += 1;
                continue;
            }

            // Single quotes in POSIX shell prevent all interpolations.
            // Inside double quotes or unquoted, interpolations execute.
            if !in_single_quote {
                // Backtick command substitution `cmd`
                if c == '`' {
                    substitutions.push("`...` backtick substitution".to_string());
                }

                // $ substitutions: $(...), $((...)), ${...}, $VAR
                if c == '$' && i + 1 < len {
                    let next = chars[i + 1];
                    if next == '(' {
                        substitutions.push("$(...) command substitution".to_string());
                    } else if next == '{' {
                        substitutions.push("${...} parameter expansion".to_string());
                    } else if next.is_ascii_alphabetic() || next == '_' {
                        substitutions.push("$VAR variable interpolation".to_string());
                    }
                }

                // Process substitution <(...) or >(...)
                if (c == '<' || c == '>') && i + 1 < len && chars[i + 1] == '(' {
                    substitutions.push("<(...) process substitution".to_string());
                }
            }

            i += 1;
        }

        substitutions.sort();
        substitutions.dedup();
        substitutions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_subcommands_simple() {
        let sub = ShellCommandParser::extract_subcommands("echo 1 && echo 2 || echo 3; echo 4 | grep 4");
        assert_eq!(sub, vec!["echo 1", "echo 2", "echo 3", "echo 4", "grep 4"]);
    }

    #[test]
    fn test_extract_subcommands_preserves_quotes() {
        // Operators inside quotes must NOT be split
        let sub = ShellCommandParser::extract_subcommands("echo 'hello && world' && echo \"foo ; bar\"");
        assert_eq!(sub, vec!["echo 'hello && world'", "echo \"foo ; bar\""]);
    }

    #[test]
    fn test_detects_substitutions() {
        assert!(ShellCommandParser::has_substitutions("echo $(whoami)"));
        assert!(ShellCommandParser::has_substitutions("echo `whoami`"));
        assert!(ShellCommandParser::has_substitutions("echo $HOME"));
        assert!(ShellCommandParser::has_substitutions("echo ${MY_VAR}"));
        assert!(ShellCommandParser::has_substitutions("diff <(ls) <(ls -a)"));

        // Single quotes prevent substitution
        assert!(!ShellCommandParser::has_substitutions("echo '$(whoami)'"));
        assert!(!ShellCommandParser::has_substitutions("echo '$HOME'"));
        assert!(!ShellCommandParser::has_substitutions("echo 'plain string'"));
    }
}
