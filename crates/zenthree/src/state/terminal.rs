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
        let command = match run_command_for_file(&path) {
            Ok(command) => command,
            Err(message) => {
                self.set_status(message);
                return false;
            }
        };

        if self.tabs.get(idx).map(|tab| tab.is_dirty).unwrap_or(false) {
            self.save_active_tab();
            if self.tabs.get(idx).map(|tab| tab.is_dirty).unwrap_or(true) {
                return false;
            }
        }

        self.show_terminal();
        let Some(session) = self.active_terminal_session_mut() else {
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

    pub fn active_terminal_session_mut(
        &mut self,
    ) -> Option<&mut crate::native_terminal::NativeTerminalSession> {
        if self.active_terminal_idx == 0 {
            self.terminal_session.as_mut()
        } else {
            self.additional_terminal_sessions.get_mut(self.active_terminal_idx - 1)
        }
    }

    pub fn terminal_session_at(
        &self,
        index: usize,
    ) -> Option<&crate::native_terminal::NativeTerminalSession> {
        if index == 0 {
            self.terminal_session.as_ref()
        } else {
            self.additional_terminal_sessions.get(index - 1)
        }
    }

    pub fn terminal_session_at_mut(
        &mut self,
        index: usize,
    ) -> Option<&mut crate::native_terminal::NativeTerminalSession> {
        if index == 0 {
            self.terminal_session.as_mut()
        } else {
            self.additional_terminal_sessions.get_mut(index - 1)
        }
    }

    pub fn terminal_session_count(&self) -> usize {
        usize::from(self.terminal_session.is_some()) + self.additional_terminal_sessions.len()
    }

    pub fn create_terminal_session(&mut self) -> bool {
        if self.terminal_session.is_none() {
            self.ensure_terminal_session();
            if self.terminal_session.is_some() {
                self.active_terminal_idx = 0;
                self.terminal_visible = true;
                self.bottom_panel_tab = BottomPanelTab::Terminal;
                return true;
            }
            return false;
        }

        match crate::native_terminal::NativeTerminalSession::new(&self.current_dir, 24, 80) {
            Some(session) => {
                self.additional_terminal_sessions.push(session);
                self.active_terminal_idx = self.terminal_session_count() - 1;
                self.terminal_visible = true;
                self.bottom_panel_tab = BottomPanelTab::Terminal;
                self.terminal_spawn_failed = false;
                true
            }
            None => {
                self.set_status("Could not start a new terminal session");
                false
            }
        }
    }

    pub fn show_terminal(&mut self) {
        self.terminal_visible = true;
        self.bottom_panel_tab = BottomPanelTab::Terminal;
        self.ensure_terminal_session();
    }

    pub fn select_terminal_session(&mut self, index: usize) {
        if index < self.terminal_session_count() {
            self.active_terminal_idx = index;
            self.focused_panel = FocusedPanel::Terminal;
        }
    }

    pub fn close_active_terminal_session(&mut self) {
        self.close_terminal_session(self.active_terminal_idx);
    }

    pub fn close_terminal_session(&mut self, index: usize) {
        if index >= self.terminal_session_count() {
            return;
        }

        if let Some(selection) = self.terminal_text_selection {
            if selection.terminal_index == index {
                self.terminal_text_selection = None;
            } else if selection.terminal_index > index {
                self.terminal_text_selection = Some(TerminalTextSelection {
                    terminal_index: selection.terminal_index - 1,
                    ..selection
                });
            }
        }
        if let Some(drag_index) = self.terminal_selection_drag {
            if drag_index == index {
                self.terminal_selection_drag = None;
            } else if drag_index > index {
                self.terminal_selection_drag = Some(drag_index - 1);
            }
        }

        if index > 0 {
            self.additional_terminal_sessions.remove(index - 1);
        } else if !self.additional_terminal_sessions.is_empty() {
            self.terminal_session = Some(self.additional_terminal_sessions.remove(0));
        } else {
            self.terminal_session = None;
        }

        let count = self.terminal_session_count();
        if count == 0 {
            self.terminal_visible = false;
            self.focused_panel = FocusedPanel::Editor;
            self.active_terminal_idx = 0;
        } else {
            if self.active_terminal_idx > index {
                self.active_terminal_idx -= 1;
            } else if self.active_terminal_idx == index {
                self.active_terminal_idx = index.min(count - 1);
            }
            self.terminal_visible = true;
            self.focused_panel = FocusedPanel::Terminal;
        }
    }

    pub fn run_selected_text_in_terminal(&mut self) -> bool {
        let Some(idx) = self.active_tab.filter(|&idx| idx < self.tabs.len()) else {
            self.set_status("Open a file and select text to run");
            return false;
        };
        let selected_text = self.tabs[idx].editor.selected_text();
        if selected_text.trim().is_empty() {
            self.set_status("Select text in the editor before running it in the terminal");
            return false;
        }
        self.show_terminal();
        let Some(session) = self.active_terminal_session_mut() else {
            self.set_status("Could not start the integrated terminal");
            return false;
        };
        session.write_line(&selected_text);
        true
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

        if let Some(sess) = self.active_terminal_session_mut() {
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
        if let Some(sess) = self.active_terminal_session_mut() {
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
        if let Some(sess) = self.active_terminal_session_mut() {
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

fn run_command_for_file(path: &Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| "The active file has no supported file extension".to_string())?;
    if extension == "jsx" || extension == "tsx" {
        return run_package_script_for_file(path);
    }

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
        _ => return Err(format!("No run command is configured for .{} files", extension)),
    };

    let command = format!(
        "{runtime} {}",
        quote_shell_argument(&path.display().to_string())
    );
    let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    else {
        return Ok(command);
    };
    Ok(command_in_directory(parent, &command))
}

fn run_package_script_for_file(path: &Path) -> Result<String, String> {
    let package_file = path
        .parent()
        .and_then(|parent| parent.ancestors().map(|dir| dir.join("package.json")).find(|file| file.is_file()))
        .ok_or_else(|| {
            "Cannot run JSX/TSX directly: no package.json was found in this project".to_string()
        })?;
    let package_dir = package_file.parent().ok_or_else(|| {
        format!("Could not determine project directory for {}", package_file.display())
    })?;
    let package_json = std::fs::read_to_string(&package_file).map_err(|error| {
        format!("Could not read {}: {}", package_file.display(), error)
    })?;
    let package: serde_json::Value = serde_json::from_str(&package_json).map_err(|error| {
        format!("Invalid {}: {}", package_file.display(), error)
    })?;
    let scripts = package
        .get("scripts")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            format!(
                "No runnable dev, start, or serve script in {}",
                package_file.display()
            )
        })?;
    let script = ["dev", "start", "serve"]
        .iter()
        .find(|name| scripts.get(**name).and_then(serde_json::Value::as_str).is_some())
        .ok_or_else(|| {
            format!(
                "No runnable dev, start, or serve script in {}",
                package_file.display()
            )
        })?;
    let package_manager = package
        .get("packageManager")
        .and_then(serde_json::Value::as_str)
        .and_then(|value| value.split('@').next())
        .filter(|manager| matches!(*manager, "npm" | "pnpm" | "yarn" | "bun"))
        .map(str::to_string)
        .unwrap_or_else(|| {
            if package_dir.join("bun.lock").exists() || package_dir.join("bun.lockb").exists() {
                "bun".to_string()
            } else if package_dir.join("pnpm-lock.yaml").exists() {
                "pnpm".to_string()
            } else if package_dir.join("yarn.lock").exists() {
                "yarn".to_string()
            } else {
                "npm".to_string()
            }
        });
    let command = format!("{package_manager} run {script}");
    Ok(command_in_directory(package_dir, &command))
}

pub(super) fn command_in_directory(directory: &Path, command: &str) -> String {
    let directory = quote_shell_argument(&directory.display().to_string());
    format!(
        "(cd {}{} && {command})",
        if cfg!(windows) { "/d " } else { "" },
        directory
    )
}

#[cfg(windows)]
pub(super) fn quote_shell_argument(argument: &str) -> String {
    format!("\"{}\"", argument.replace('"', "\"\""))
}

#[cfg(unix)]
pub(super) fn quote_shell_argument(argument: &str) -> String {
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
        assert!(run_command_for_file(Path::new("sample.unknown")).is_err());
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

    #[test]
    fn jsx_runs_the_nearest_package_dev_script_with_detected_package_manager() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let project_dir = std::env::temp_dir().join(format!("zenthree_jsx_run_{unique_id}"));
        let source_dir = project_dir.join("src");
        std::fs::create_dir_all(&source_dir).unwrap();
        std::fs::write(
            project_dir.join("package.json"),
            r#"{"scripts":{"dev":"vite","start":"react-scripts start"}}"#,
        )
        .unwrap();
        std::fs::write(project_dir.join("pnpm-lock.yaml"), "").unwrap();
        let file = source_dir.join("App.jsx");

        let command = run_command_for_file(&file).unwrap();

        assert!(command.contains("pnpm run dev"));
        assert!(command.contains(&quote_shell_argument(&project_dir.display().to_string())));
        assert!(!command.contains("react-scripts start"));
        std::fs::remove_dir_all(project_dir).unwrap();
    }

    #[test]
    fn jsx_without_a_project_script_returns_a_helpful_error() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let project_dir = std::env::temp_dir().join(format!("zenthree_jsx_no_script_{unique_id}"));
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join("package.json"), r#"{"scripts":{"test":"vitest"}}"#)
            .unwrap();

        let error = run_command_for_file(&project_dir.join("App.jsx")).unwrap_err();

        assert!(error.contains("No runnable dev, start, or serve script"));
        std::fs::remove_dir_all(project_dir).unwrap();
    }
}
