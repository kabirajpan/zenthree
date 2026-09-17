use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;

pub struct NativeTerminalSession {
    master: Box<dyn MasterPty + Send>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub lines: Arc<Mutex<Vec<String>>>,
    pub pending_line: Arc<Mutex<String>>,
    pub has_new_output: Arc<AtomicBool>,
}

pub fn clean_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next(); // consume '['
                    // CSI sequence: read until final byte (0x40..=0x7E)
                    while let Some(&next_c) = chars.peek() {
                        chars.next();
                        if (0x40..=0x7E).contains(&(next_c as u32)) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next(); // consume ']'
                    // OSC sequence: read until BEL (\x07) or ST (\x1b\)
                    while let Some(next_c) = chars.next() {
                        if next_c == '\x07' {
                            break;
                        }
                        if next_c == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some('(') | Some(')') => {
                    chars.next();
                    chars.next(); // consume charset designator
                }
                Some(_) => {
                    chars.next(); // consume single escaped char
                }
                None => {}
            }
            continue;
        }

        if c == '\x08' {
            out.pop();
        } else if c != '\x07' {
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

        #[cfg(windows)]
        let shell = if let Ok(s) = std::env::var("SHELL") {
            s
        } else {
            "powershell.exe".to_string()
        };
        #[cfg(not(windows))]
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());

        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");

        let _child = pair.slave.spawn_command(cmd).ok()?;

        let writer = Arc::new(Mutex::new(pair.master.take_writer().ok()?));
        let lines = Arc::new(Mutex::new(vec![
            format!("Zenthree Native Terminal [{}]", shell),
        ]));
        let pending_line = Arc::new(Mutex::new(String::new()));
        let has_new_output = Arc::new(AtomicBool::new(true));

        if let Ok(mut reader) = pair.master.try_clone_reader() {
            let lines_clone = Arc::clone(&lines);
            let pending_clone = Arc::clone(&pending_line);
            let has_new_clone = Arc::clone(&has_new_output);

            std::thread::spawn(move || {
                let mut buf = [0u8; 8192];
                let mut current_line = String::new();

                while let Ok(n) = reader.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    let mut chars = chunk.chars().peekable();

                    while let Some(c) = chars.next() {
                        if c == '\r' {
                            if chars.peek() == Some(&'\n') {
                                chars.next(); // consume '\n'
                                let stripped = clean_ansi(&current_line);
                                if let Ok(mut l) = lines_clone.lock() {
                                    l.push(stripped);
                                    let len = l.len();
                                    if len > 2000 {
                                        l.drain(0..len - 2000);
                                    }
                                }
                                current_line.clear();
                            } else {
                                // Standalone \r: in-place overwrite for progress spinners
                                current_line.clear();
                            }
                        } else if c == '\n' {
                            let stripped = clean_ansi(&current_line);
                            if let Ok(mut l) = lines_clone.lock() {
                                l.push(stripped);
                                let len = l.len();
                                if len > 2000 {
                                    l.drain(0..len - 2000);
                                }
                            }
                            current_line.clear();
                        } else {
                            current_line.push(c);
                        }
                    }

                    let pending_text = clean_ansi(&current_line);
                    if let Ok(mut p) = pending_clone.lock() {
                        *p = pending_text;
                    }

                    has_new_clone.store(true, Ordering::Release);
                }
            });
        }

        Some(Self {
            master: pair.master,
            writer,
            lines,
            pending_line,
            has_new_output,
        })
    }

    pub fn poll_output(&self) -> bool {
        self.has_new_output.swap(false, Ordering::AcqRel)
    }

    pub fn get_visible_lines(&self, max_visible: usize, scroll_offset: usize) -> (Vec<String>, String) {
        let pending = self.pending_line.lock().map(|p| p.clone()).unwrap_or_default();
        let lines = if let Ok(l) = self.lines.lock() {
            let total = l.len();
            let effective_end = total.saturating_sub(scroll_offset);
            let start = effective_end.saturating_sub(max_visible);
            if start < effective_end && effective_end <= total {
                l[start..effective_end].to_vec()
            } else if effective_end <= total {
                l[..effective_end].to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        (lines, pending)
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
