use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GitBlameLine {
    pub author: String,
    pub time_ago: String,
    pub summary: String,
    pub commit_hash: String,
    pub is_uncommitted: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FileBlame {
    pub lines: HashMap<usize, GitBlameLine>, // 0-indexed line number -> blame info
    pub fetched_at: Instant,
}

pub struct GitBlameManager {
    cache: HashMap<PathBuf, FileBlame>,
    pending: HashMap<PathBuf, Instant>,
    request_tx: Sender<(PathBuf, PathBuf)>,
    result_rx: Receiver<(PathBuf, HashMap<usize, GitBlameLine>)>,
}

impl Default for GitBlameManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GitBlameManager {
    pub fn new() -> Self {
        let (request_tx, request_rx) = channel::<(PathBuf, PathBuf)>();
        let (result_tx, result_rx) = channel::<(PathBuf, HashMap<usize, GitBlameLine>)>();

        std::thread::Builder::new()
            .name("zenthree-git-blame".into())
            .spawn(move || {
                while let Ok((file_path, repo_root)) = request_rx.recv() {
                    let parsed = run_git_blame(&file_path, &repo_root);
                    let _ = result_tx.send((file_path, parsed));
                }
            })
            .expect("Failed to spawn git blame thread");

        Self {
            cache: HashMap::new(),
            pending: HashMap::new(),
            request_tx,
            result_rx,
        }
    }

    /// Poll for any completed blame requests in non-blocking fashion.
    /// Returns true if new blame info was stored (requiring UI redraw).
    pub fn poll(&mut self) -> bool {
        let mut updated = false;
        while let Ok((path, lines)) = self.result_rx.try_recv() {
            self.pending.remove(&path);
            self.cache.insert(
                path,
                FileBlame {
                    lines,
                    fetched_at: Instant::now(),
                },
            );
            updated = true;
        }
        updated
    }

    /// Retrieve the blame annotation for a given line (0-indexed).
    /// If not cached, triggers an asynchronous background fetch.
    pub fn get_line_blame(
        &mut self,
        file_path: &Path,
        repo_root: &Path,
        line_idx: usize,
    ) -> Option<GitBlameLine> {
        if let Some(file_blame) = self.cache.get(file_path) {
            return file_blame.lines.get(&line_idx).cloned();
        }

        self.request_blame(file_path, repo_root);
        None
    }

    /// Request blame computation for a file if not already pending.
    pub fn request_blame(&mut self, file_path: &Path, repo_root: &Path) {
        if let Some(pending_time) = self.pending.get(file_path) {
            if pending_time.elapsed().as_secs() < 3 {
                return;
            }
        }
        self.pending.insert(file_path.to_path_buf(), Instant::now());
        let _ = self.request_tx.send((file_path.to_path_buf(), repo_root.to_path_buf()));
    }

    /// Invalidate cache for a specific file (e.g. after save)
    pub fn invalidate(&mut self, file_path: &Path) {
        self.cache.remove(file_path);
        self.pending.remove(file_path);
    }

    /// Invalidate all cached blames (e.g. after commit or branch switch)
    pub fn invalidate_all(&mut self) {
        self.cache.clear();
        self.pending.clear();
    }
}

fn run_git_blame(file_path: &Path, repo_root: &Path) -> HashMap<usize, GitBlameLine> {
    let mut map = HashMap::new();

    let output = std::process::Command::new("git")
        .args(["blame", "--line-porcelain", "--"])
        .arg(file_path)
        .current_dir(repo_root)
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o.stdout,
        _ => return map,
    };

    let text = String::from_utf8_lossy(&output);
    let now_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut current_final_line: Option<usize> = None;
    let mut current_hash = String::new();
    let mut current_author = String::new();
    let mut current_time: i64 = 0;
    let mut current_summary = String::new();

    for line in text.lines() {
        if line.starts_with('\t') {
            if let Some(final_line) = current_final_line.take() {
                if final_line > 0 {
                    let line_idx = final_line - 1;
                    let is_uncommitted = current_hash.starts_with("0000000")
                        || current_author == "Not Committed Yet";

                    let (author, time_ago) = if is_uncommitted {
                        ("You".to_string(), "uncommitted changes".to_string())
                    } else {
                        let time_str = format_time_ago(current_time, now_epoch);
                        (current_author.clone(), time_str)
                    };

                    map.insert(
                        line_idx,
                        GitBlameLine {
                            author,
                            time_ago,
                            summary: current_summary.clone(),
                            commit_hash: current_hash.clone(),
                            is_uncommitted,
                        },
                    );
                }
            }
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3
            && parts[0].len() == 40
            && parts[1].chars().all(|c| c.is_ascii_digit())
            && parts[2].chars().all(|c| c.is_ascii_digit())
        {
            if let Ok(line_no) = parts[2].parse::<usize>() {
                current_final_line = Some(line_no);
                current_hash = parts[0].to_string();
                current_author.clear();
                current_time = 0;
                current_summary.clear();
            }
        } else if let Some(author) = line.strip_prefix("author ") {
            current_author = author.trim().to_string();
        } else if let Some(time_str) = line.strip_prefix("author-time ") {
            current_time = time_str.trim().parse::<i64>().unwrap_or(0);
        } else if let Some(summary) = line.strip_prefix("summary ") {
            current_summary = summary.trim().to_string();
        }
    }

    map
}

pub fn format_time_ago(timestamp: i64, now: i64) -> String {
    if timestamp <= 0 {
        return "recently".to_string();
    }
    let diff = (now - timestamp).max(0);
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let m = diff / 60;
        if m == 1 {
            "1 minute ago".to_string()
        } else {
            format!("{} minutes ago", m)
        }
    } else if diff < 86400 {
        let h = diff / 3600;
        if h == 1 {
            "1 hour ago".to_string()
        } else {
            format!("{} hours ago", h)
        }
    } else if diff < 86400 * 30 {
        let d = diff / 86400;
        if d == 1 {
            "1 day ago".to_string()
        } else {
            format!("{} days ago", d)
        }
    } else if diff < 86400 * 365 {
        let mo = diff / (86400 * 30);
        if mo == 1 {
            "1 month ago".to_string()
        } else {
            format!("{} months ago", mo)
        }
    } else {
        let y = diff / (86400 * 365);
        if y == 1 {
            "1 year ago".to_string()
        } else {
            format!("{} years ago", y)
        }
    }
}
