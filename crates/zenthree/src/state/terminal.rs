use crate::state::types::*;
use crate::state::ZenthreeState;
use std::path::Path;

impl ZenthreeState {
    pub fn run_active_file(&mut self) -> bool {
        let Some(idx) = self.active_tab.filter(|&idx| idx < self.tabs.len()) else {
            self.set_status("Open a file before running");
            return false;
        };

        let Some(path) = self.tabs[idx].path.clone() else {
            self.save_active_tab();
            let Some(path) = self.tabs.get(idx).and_then(|tab| tab.path.clone()) else {
                self.set_status("Save the active file before running");
                return false;
            };
            return self.run_file_at_path(idx, path);
        };

        self.run_file_at_path(idx, path)
    }

    fn run_file_at_path(&mut self, idx: usize, path: std::path::PathBuf) -> bool {
        let Some(command) = run_command_for_file(&path) else {
            self.set_status(format!(
                "No run command is configured for .{} files",
                path.extension().and_then(|ext| ext.to_str()).unwrap_or("")
            ));
            return false;
        };

        if self.tabs.get(idx).map(|tab| tab.is_dirty).unwrap_or(false) {
            self.save_active_tab();
            if self.tabs.get(idx).map(|tab| tab.is_dirty).unwrap_or(true) {
                return false;
            }
        }

        self.terminal_visible = true;
        self.ensure_terminal_session();
        let Some(session) = self.terminal_session.as_mut() else {
            self.set_status("Could not start the integrated terminal");
            return false;
        };

        session.write_line(&command);
        self.set_status(format!("Running {}", path.display()));
        true
    }

    pub fn ensure_terminal_session(&mut self) {
        if self.terminal_session.is_none() && !self.terminal_spawn_failed {
            if let Some(sess) =
                crate::native_terminal::NativeTerminalSession::new(&self.current_dir, 24, 80)
            {
                self.terminal_session = Some(sess);
            } else {
                self.terminal_spawn_failed = true;
                self.terminal_lines.push(TerminalLine {
                    text: "Failed to spawn native PTY session. Please check your shell configuration.".to_string(),
                    is_command: false,
                    is_error: true,
                });
            }
        }

    }

    #[allow(dead_code)]
    pub fn send_terminal_input(&mut self, text: &str) {
        self.ensure_terminal_session();

        let trimmed = text.trim();
        if !trimmed.is_empty() {
            if self.terminal_history.last().map(|s| s.as_str()) != Some(trimmed) {
                self.terminal_history.push(trimmed.to_string());
            }
        }

        self.terminal_history_idx = None;

        if trimmed == "clear" || trimmed == "cls" {
            self.clear_terminal();
        }

        if let Some(ref mut sess) = self.terminal_session {
            sess.write_line(text);
        } else {
            self.terminal_lines.push(TerminalLine {
                text: format!("$ {}", text),
                is_command: true,
                is_error: false,
            });
            if !trimmed.is_empty() {
                self.terminal_lines.push(TerminalLine {
                    text: format!("Executed (simulated): {}", trimmed),
                    is_command: false,
                    is_error: false,
                });
            }
        }
        self.terminal_input.clear();
        self.terminal_cursor_idx = 0;
    }

    #[allow(dead_code)]
    pub fn send_terminal_sigint(&mut self) {
        if let Some(ref mut sess) = self.terminal_session {
            sess.write_raw(b"\x03");
        } else {
            self.terminal_lines.push(TerminalLine {
                text: "^C".to_string(),
                is_command: false,
                is_error: false,
            });
        }
        self.terminal_input.clear();
        self.terminal_cursor_idx = 0;
        self.terminal_history_idx = None;
    }

    pub fn clear_terminal(&mut self) {
        if let Some(ref mut sess) = self.terminal_session {
            sess.clear();
        } else {
            self.terminal_spawn_failed = false;
        }
        self.terminal_lines.clear();
        self.terminal_input.clear();
        self.terminal_cursor_idx = 0;
        self.terminal_history_idx = None;
    }

    #[allow(dead_code)]
    pub fn terminal_history_prev(&mut self) {
        if self.terminal_history.is_empty() {
            return;
        }
        let next_idx = match self.terminal_history_idx {
            None => self.terminal_history.len().saturating_sub(1),
            Some(idx) => idx.saturating_sub(1),
        };
        self.terminal_history_idx = Some(next_idx);
        if let Some(cmd) = self.terminal_history.get(next_idx) {
            self.terminal_input = cmd.clone();
            self.terminal_cursor_idx = self.terminal_input.len();
        }
    }

    #[allow(dead_code)]
    pub fn terminal_history_next(&mut self) {
        if let Some(idx) = self.terminal_history_idx {
            if idx + 1 < self.terminal_history.len() {
                self.terminal_history_idx = Some(idx + 1);
                self.terminal_input = self.terminal_history[idx + 1].clone();
                self.terminal_cursor_idx = self.terminal_input.len();
            } else {
                self.terminal_history_idx = None;
                self.terminal_input.clear();
                self.terminal_cursor_idx = 0;
            }
        }
    }
}

fn run_command_for_file(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let runtime = match extension.as_str() {
        "py" | "pyw" => {
            if cfg!(windows) {
                "python"
            } else {
                "python3"
            }
        }
        "js" | "cjs" | "mjs" => "node",
        "ts" | "cts" | "mts" => "deno run",
        "go" => "go run",
        "rb" => "ruby",
        "php" => "php",
        "lua" => "lua",
        "sh" | "bash" => {
            if cfg!(windows) {
                "bash"
            } else {
                "sh"
            }
        }
        "ps1" => {
            if cfg!(windows) {
                "powershell -File"
            } else {
                "pwsh -File"
            }
        }
        _ => return None,
    };

    let command = format!(
        "{runtime} {}",
        quote_shell_argument(&path.display().to_string())
    );
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Some(command);
    };
    let directory = quote_shell_argument(&parent.display().to_string());
    Some(format!(
        "(cd {}{} && {command})",
        if cfg!(windows) { "/d " } else { "" },
        directory
    ))
}

#[cfg(windows)]
fn quote_shell_argument(argument: &str) -> String {
    format!("\"{}\"", argument.replace('"', "\"\""))
}

#[cfg(unix)]
fn quote_shell_argument(argument: &str) -> String {
    format!("'{}'", argument.replace('\'', "'\\''"))
}

#[cfg(test)]
mod run_tests {
    use super::{quote_shell_argument, run_command_for_file};
    use std::path::Path;

    #[test]
    fn run_command_uses_the_file_extension_to_choose_a_runtime() {
        let command = run_command_for_file(Path::new("sample.py")).unwrap();
        assert!(command.contains(if cfg!(windows) {
            "python "
        } else {
            "python3 "
        }));
        assert!(!command.starts_with("(cd "));
        assert!(run_command_for_file(Path::new("sample.unknown")).is_none());
    }

    #[test]
    fn shell_argument_quoting_preserves_spaces_and_quotes() {
        let quoted = quote_shell_argument("folder/it's a file.py");
        #[cfg(windows)]
        assert_eq!(quoted, "\"folder/it's a file.py\"");
        #[cfg(unix)]
        assert_eq!(quoted, "'folder/it'\\''s a file.py'");
    }

    #[test]
    fn run_command_uses_the_active_file_directory() {
        let command = run_command_for_file(Path::new("workspace folder/sample.js")).unwrap();
        #[cfg(windows)]
        assert_eq!(
            command,
            "(cd /d \"workspace folder\" && node \"workspace folder/sample.js\")"
        );
        #[cfg(unix)]
        assert_eq!(
            command,
            "(cd 'workspace folder' && node 'workspace folder/sample.js')"
        );
    }
}
