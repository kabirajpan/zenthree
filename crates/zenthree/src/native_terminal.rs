use portable_pty::{Child, CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Strips Windows verbatim/UNC prefix (e.g. `\\?\C:\...` or `\\?\UNC\...`) so
/// processes like `cmd.exe` and git that do not support UNC paths can run cleanly.
pub fn strip_unc_prefix(path: &Path) -> PathBuf {
    let s = path.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{}", stripped))
    } else if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

pub struct NativeTerminalSession {
    master: Box<dyn MasterPty + Send>,
    _child: Box<dyn Child + Send + Sync>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub has_new_output: Arc<AtomicBool>,
}

impl Drop for NativeTerminalSession {
    fn drop(&mut self) {
        if matches!(self._child.try_wait(), Ok(Some(_))) {
            return;
        }
        if let Err(e) = self._child.kill() {
            log::error!("Failed to stop native terminal shell: {e}");
            return;
        }
        if let Err(e) = self._child.wait() {
            log::error!("Failed to wait for native terminal shell to stop: {e}");
        }
    }
}

impl NativeTerminalSession {
    pub fn new(cwd: &Path, rows: u16, cols: u16) -> Option<Self> {
        let pty_system = NativePtySystem::default();
        let r = rows.max(4);
        let c = cols.max(20);
        let pair = match pty_system.openpty(PtySize {
            rows: r,
            cols: c,
            pixel_width: 0,
            pixel_height: 0,
        }) {
            Ok(p) => p,
            Err(e) => {
                log::error!("Failed to open native PTY: {e}");
                return None;
            }
        };

        let mut cmd = default_shell_command();

        // Normalize cwd: strip Windows \\?\ verbatim/UNC prefix if present
        let clean_cwd = strip_unc_prefix(cwd);
        if clean_cwd.exists() && clean_cwd.is_dir() {
            cmd.cwd(&clean_cwd);
        } else if let Some(home) = dirs::home_dir() {
            let clean_home = strip_unc_prefix(&home);
            if clean_home.exists() && clean_home.is_dir() {
                cmd.cwd(&clean_home);
            }
        }

        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(e) => {
                log::error!("Failed to spawn native terminal shell: {e}");
                return None;
            }
        };
        drop(pair.slave);

        let mut reader = match pair.master.try_clone_reader() {
            Ok(reader) => reader,
            Err(e) => {
                log::error!("Failed to create native terminal reader: {e}");
                return None;
            }
        };

        let writer_handle = match pair.master.take_writer() {
            Ok(w) => w,
            Err(e) => {
                log::error!("Failed to take PTY writer: {e}");
                return None;
            }
        };
        let writer = Arc::new(Mutex::new(writer_handle));
        let parser = Arc::new(Mutex::new(vt100::Parser::new(r, c, 2000)));
        let has_new_output = Arc::new(AtomicBool::new(true));

        let parser_clone = Arc::clone(&parser);
        let notify_clone = Arc::clone(&has_new_output);
        let writer_clone = Arc::clone(&writer);
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            let mut query_scan = Vec::new();
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut p) = parser_clone.lock() {
                            p.process(&buf[..n]);
                            let query_count =
                                count_cursor_position_queries(&mut query_scan, &buf[..n]);
                            if query_count > 0 {
                                let (row, col) = p.screen().cursor_position();
                                let response = format!(
                                    "\x1b[{};{}R",
                                    row.saturating_add(1),
                                    col.saturating_add(1)
                                );
                                if let Ok(mut writer) = writer_clone.lock() {
                                    for _ in 0..query_count {
                                        if let Err(e) = writer.write_all(response.as_bytes()) {
                                            log::error!(
                                                "Failed to answer native terminal cursor query: {e}"
                                            );
                                            break;
                                        }
                                    }
                                    if let Err(e) = writer.flush() {
                                        log::error!(
                                            "Failed to flush native terminal cursor response: {e}"
                                        );
                                    }
                                }
                            }
                            notify_clone.store(true, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read native terminal output: {e}");
                        break;
                    }
                }
            }
        });

        Some(Self {
            master: pair.master,
            _child: child,
            writer,
            parser,
            has_new_output,
        })
    }

    pub fn write_raw(&mut self, bytes: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(bytes);
            let _ = w.flush();
        }
    }

    pub fn write_line(&mut self, text: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let line_ending = if cfg!(target_os = "windows") { "\r\n" } else { "\n" };
            let _ = w.write_all(format!("{}{}", text, line_ending).as_bytes());
            let _ = w.flush();
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let r = rows.max(4);
        let c = cols.max(20);
        let _ = self.master.resize(PtySize {
            rows: r,
            cols: c,
            pixel_width: 0,
            pixel_height: 0,
        });
        if let Ok(mut p) = self.parser.lock() {
            p.screen_mut().set_size(r, c);
        }
    }

    pub fn clear(&mut self) {
        if let Ok(mut p) = self.parser.lock() {
            let (rows, cols) = p.screen().size();
            *p = vt100::Parser::new(rows, cols, 2000);
        }
    }
}

#[cfg(windows)]
fn default_shell_command() -> CommandBuilder {
    CommandBuilder::new_default_prog()
}

#[cfg(unix)]
fn default_shell_command() -> CommandBuilder {
    let configured_shell = std::env::var_os("SHELL").filter(|shell| Path::new(shell).is_file());
    let fallback_shells: &[&str] = if cfg!(target_os = "macos") {
        &["/bin/zsh", "/bin/bash", "/bin/sh"]
    } else {
        &["/bin/bash", "/bin/zsh", "/bin/sh"]
    };
    let shell = configured_shell
        .or_else(|| {
            fallback_shells
                .iter()
                .map(|shell| std::ffi::OsString::from(*shell))
                .find(|shell| Path::new(shell).is_file())
        })
        .unwrap_or_else(|| std::ffi::OsString::from("/bin/sh"));
    CommandBuilder::new(shell)
}

fn count_cursor_position_queries(pending: &mut Vec<u8>, bytes: &[u8]) -> usize {
    const CURSOR_POSITION_QUERY: &[u8] = b"\x1b[6n";

    pending.extend_from_slice(bytes);
    let mut count = 0;
    let mut consumed = 0;
    while let Some(offset) = pending[consumed..]
        .windows(CURSOR_POSITION_QUERY.len())
        .position(|window| window == CURSOR_POSITION_QUERY)
    {
        consumed += offset + CURSOR_POSITION_QUERY.len();
        count += 1;
    }
    if consumed > 0 {
        pending.drain(..consumed);
    }
    if pending.len() > CURSOR_POSITION_QUERY.len() - 1 {
        let excess = pending.len() - (CURSOR_POSITION_QUERY.len() - 1);
        pending.drain(..excess);
    }
    count
}

#[cfg(test)]
mod tests {
    use super::NativeTerminalSession;
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn native_terminal_accepts_input_and_displays_shell_output() {
        let mut terminal = NativeTerminalSession::new(Path::new("."), 24, 80)
            .expect("native terminal session should start");
        terminal.write_line("echo ZENTHREE_TERMINAL_SMOKE_TEST");

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let output = {
                let parser = terminal.parser.lock().expect("terminal parser lock");
                let screen = parser.screen();
                (0..screen.size().0)
                    .map(|row| {
                        (0..screen.size().1)
                            .filter_map(|col| screen.cell(row, col))
                            .map(|cell| cell.contents())
                            .collect::<String>()
                    })
                    .collect::<String>()
            };

            if output.matches("ZENTHREE_TERMINAL_SMOKE_TEST").count() >= 2 {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "shell did not echo the submitted command; terminal output: {output:?}"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    #[test]
    fn cursor_position_queries_are_detected_across_read_boundaries() {
        let mut pending = Vec::new();
        assert_eq!(
            super::count_cursor_position_queries(&mut pending, b"\x1b[6"),
            0
        );
        assert_eq!(super::count_cursor_position_queries(&mut pending, b"n"), 1);
        assert_eq!(
            super::count_cursor_position_queries(&mut pending, b"\x1b[6n\x1b[6n"),
            2
        );
    }
}
