use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
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
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub has_new_output: Arc<AtomicBool>,
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

        let mut cmd = if cfg!(target_os = "windows") {
            // portable-pty has built-in ConPTY default program resolution
            // which automatically finds ComSpec / cmd.exe with correct environment
            CommandBuilder::new_default_prog()
        } else {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| {
                if cfg!(target_os = "macos") {
                    "/bin/zsh".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            });
            CommandBuilder::new(&shell)
        };

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

        let _child = match pair.slave.spawn_command(cmd) {
            Ok(child) => child,
            Err(e) => {
                log::error!("Failed to spawn native terminal shell: {e}");
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

        if let Ok(mut reader) = pair.master.try_clone_reader() {
            let parser_clone = Arc::clone(&parser);
            let notify_clone = Arc::clone(&has_new_output);
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    if let Ok(mut p) = parser_clone.lock() {
                        p.process(&buf[..n]);
                        notify_clone.store(true, Ordering::Relaxed);
                    }
                }
            });
        }

        Some(Self {
            master: pair.master,
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
