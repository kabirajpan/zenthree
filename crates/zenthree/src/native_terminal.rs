use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem, MasterPty};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::path::Path;

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
        let pair = pty_system.openpty(PtySize {
            rows: r,
            cols: c,
            pixel_width: 0,
            pixel_height: 0,
        }).ok()?;

        let shell = if cfg!(target_os = "windows") {
            std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| {
                if cfg!(target_os = "macos") {
                    "/bin/zsh".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            })
        };
        let mut cmd = CommandBuilder::new(&shell);
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let _child = pair.slave.spawn_command(cmd).ok()?;

        let writer = Arc::new(Mutex::new(pair.master.take_writer().ok()?));
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
            let _ = w.write_all(format!("{}\n", text).as_bytes());
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
