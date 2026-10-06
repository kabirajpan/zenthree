use crate::state::types::*;
use crate::state::ZenthreeState;

impl ZenthreeState {
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
