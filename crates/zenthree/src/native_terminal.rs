use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::path::Path;

pub struct NativeTerminalSession {
    master: Box<dyn MasterPty + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub lines: Arc<Mutex<Vec<String>>>,
}

fn clean_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_escape = false;
    let mut in_bracket = false;

    for c in input.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == '[' {
                in_bracket = true;
                continue;
            }
            if in_bracket {
                if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '~' {
                    in_escape = false;
                    in_bracket = false;
                }
                continue;
            }
            in_escape = false;
            continue;
        }
        if c != '\x07' && c != '\x08' {
            out.push(c);
        }
    }
    out
}

impl NativeTerminalSession {
    pub fn new(cwd: &Path) -> Option<Self> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }).ok()?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");

        let _child = pair.slave.spawn_command(cmd).ok()?;

        let writer = Arc::new(Mutex::new(pair.master.take_writer().ok()?));
        let lines = Arc::new(Mutex::new(vec![
            format!("Zenthree Native Terminal [{}]", shell),
        ]));

        if let Ok(mut reader) = pair.master.try_clone_reader() {
            let lines_clone = Arc::clone(&lines);
            std::thread::spawn(move || {
                let mut buf = [0u8; 4096];
                let mut current_line = String::new();
                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    for c in chunk.chars() {
                        if c == '\n' || c == '\r' {
                            if !current_line.is_empty() {
                                let stripped = clean_ansi(&current_line);
                                if !stripped.is_empty() {
                                    if let Ok(mut l) = lines_clone.lock() {
                                        l.push(stripped);
                                        let len = l.len();
                                        if len > 1000 {
                                            l.drain(0..len - 1000);
                                        }
                                    }
                                }
                                current_line.clear();
                            }
                        } else {
                            current_line.push(c);
                        }
                    }
                }
            });
        }

        Some(Self {
            master: pair.master,
            writer,
            lines,
        })
    }

    pub fn write_line(&mut self, text: &str) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(format!("{}\n", text).as_bytes());
            let _ = w.flush();
        }
    }

    #[allow(dead_code)]
    pub fn write_raw(&mut self, bytes: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(bytes);
            let _ = w.flush();
        }
    }

    pub fn resize(&mut self, rows: u16, cols: u16) {
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }
}
