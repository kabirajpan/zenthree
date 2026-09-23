use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, Instant};
use zenthra_state::ArcSignal;

pub struct WorkspaceWatcher {
    _watcher: Option<RecommendedWatcher>,
    stop_tx: Option<Sender<()>>,
    pub pending_signal: ArcSignal<bool>,
}

impl Default for WorkspaceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceWatcher {
    pub fn new() -> Self {
        Self {
            _watcher: None,
            stop_tx: None,
            pending_signal: ArcSignal::new(false),
        }
    }

    pub fn watch(&mut self, root: &Path) {
        self.stop();

        if !root.exists() || !root.is_dir() {
            return;
        }

        let pending = self.pending_signal.clone();
        let (raw_tx, raw_rx): (Sender<PathBuf>, Receiver<PathBuf>) = channel();
        let (stop_tx, stop_rx): (Sender<()>, Receiver<()>) = channel();

        // Background debouncer thread
        std::thread::Builder::new()
            .name("zenthree-fs-debouncer".into())
            .spawn(move || {
                let debounce_window = Duration::from_millis(200);
                'outer: loop {
                    // Check if stopped
                    if stop_rx.try_recv().is_ok() {
                        break;
                    }

                    // Block until at least one event arrives
                    let path = match raw_rx.recv_timeout(Duration::from_millis(100)) {
                        Ok(p) => p,
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    };

                    let _ = path;
                    let mut last_event = Instant::now();

                    // Debounce burst of events
                    loop {
                        if stop_rx.try_recv().is_ok() {
                            break 'outer;
                        }

                        let elapsed = last_event.elapsed();
                        if elapsed >= debounce_window {
                            // Debounce window expired with no new events, mark dirty and wake UI
                            pending.set(true);
                            break;
                        }

                        let wait_time = debounce_window.saturating_sub(elapsed);
                        match raw_rx.recv_timeout(wait_time) {
                            Ok(_) => {
                                last_event = Instant::now();
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                                pending.set(true);
                                break;
                            }
                            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                                pending.set(true);
                                break 'outer;
                            }
                        }
                    }
                }
            })
            .ok();

        let tx = raw_tx.clone();
        let root_canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

        let watcher_res = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Filter: only care about create, remove, rename, or file modifications
                    let relevant = matches!(
                        event.kind,
                        EventKind::Create(_)
                            | EventKind::Remove(_)
                            | EventKind::Modify(_)
                    );
                    if !relevant {
                        return;
                    }

                    // Check if any path is NOT in ignored directories (e.g., .git/objects, target/)
                    for path in &event.paths {
                        let path_str = path.to_string_lossy();
                        // Ignore git internal pack/objects churn, but allow .git index/HEAD for status updates
                        let is_git_churn = path_str.contains("/.git/objects")
                            || path_str.contains("/.git/logs")
                            || path_str.contains("/.git/refs");
                        let is_target = path_str.contains("/target/debug")
                            || path_str.contains("/target/release")
                            || path_str.contains("/target/tmp");
                        let is_editor_temp = path_str.ends_with('~') || path_str.ends_with(".swp");

                        if !is_git_churn && !is_target && !is_editor_temp {
                            let _ = tx.send(path.clone());
                            break;
                        }
                    }
                }
            },
            Config::default(),
        );

        match watcher_res {
            Ok(mut watcher) => {
                if let Ok(()) = watcher.watch(&root_canonical, RecursiveMode::Recursive) {
                    self._watcher = Some(watcher);
                    self.stop_tx = Some(stop_tx);
                }
            }
            Err(e) => {
                log::warn!("Failed to create workspace file watcher: {:?}", e);
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        self._watcher = None;
    }

    pub fn take_pending(&self) -> bool {
        self.pending_signal.with_mut(|flag| {
            let was = *flag;
            *flag = false;
            was
        })
    }
}

impl Drop for WorkspaceWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}
