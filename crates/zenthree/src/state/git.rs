use crate::state::types::*;
use crate::state::ZenthreeState;
use std::path::Path;

impl ZenthreeState {
    pub fn refresh_git_status(&mut self) {
        if !self.has_workspace {
            self.git_state.is_repo = false;
            self.git_state.branch.clear();
            self.git_state.staged.clear();
            self.git_state.unstaged.clear();
            return;
        }

        let root = &self.current_dir;

        let is_inside = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(&root)
            .output()
            .map(|o| o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "true")
            .unwrap_or(false);

        if !is_inside {
            self.git_state.is_repo = false;
            self.git_state.branch.clear();
            self.git_state.staged.clear();
            self.git_state.unstaged.clear();
            return;
        }

        self.git_state.is_repo = true;

        let branch_out = std::process::Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&root)
            .output();

        let branch = if let Ok(o) = branch_out {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !s.is_empty() {
                s
            } else {
                std::process::Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .current_dir(&root)
                    .output()
                    .map(|ho| {
                        let hash = String::from_utf8_lossy(&ho.stdout).trim().to_string();
                        if hash.is_empty() {
                            "HEAD".to_string()
                        } else {
                            format!("detached ({})", hash)
                        }
                    })
                    .unwrap_or_else(|_| "HEAD".to_string())
            }
        } else {
            "HEAD".to_string()
        };
        self.git_state.branch = branch;

        let status_out = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "-uall"])
            .current_dir(&root)
            .output();

        if let Ok(output) = status_out {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut staged = Vec::new();
                let mut unstaged = Vec::new();

                for line in stdout.lines() {
                    if line.len() < 4 {
                        continue;
                    }
                    let x = line.as_bytes()[0] as char;
                    let y = line.as_bytes()[1] as char;

                    let mut path_str = line[3..].trim();
                    if path_str.starts_with('"') && path_str.ends_with('"') && path_str.len() >= 2 {
                        path_str = &path_str[1..path_str.len() - 1];
                    }

                    let rel_path = if let Some((_, to)) = path_str.split_once(" -> ") {
                        to.trim().to_string()
                    } else {
                        path_str.to_string()
                    };

                    let full_path = root.join(&rel_path);
                    let filename = full_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| rel_path.clone());
                    let dir_path = full_path
                        .parent()
                        .and_then(|p| p.strip_prefix(&root).ok())
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Check Staged (X)
                    if x != ' ' && x != '?' && x != '!' {
                        let status = match x {
                            'M' => GitFileStatusCode::Modified,
                            'A' => GitFileStatusCode::Added,
                            'D' => GitFileStatusCode::Deleted,
                            'R' => GitFileStatusCode::Renamed,
                            'U' => GitFileStatusCode::Conflicted,
                            _ => GitFileStatusCode::Modified,
                        };
                        staged.push(GitChangeItem {
                            path: full_path.clone(),
                            rel_path: rel_path.clone(),
                            filename: filename.clone(),
                            dir_path: dir_path.clone(),
                            status,
                            is_staged: true,
                        });
                    }

                    // Check Unstaged (Y)
                    if y != ' ' && y != '!' {
                        let status = match y {
                            'M' => GitFileStatusCode::Modified,
                            'D' => GitFileStatusCode::Deleted,
                            '?' => GitFileStatusCode::Untracked,
                            'U' => GitFileStatusCode::Conflicted,
                            _ => GitFileStatusCode::Modified,
                        };
                        unstaged.push(GitChangeItem {
                            path: full_path,
                            rel_path,
                            filename,
                            dir_path,
                            status,
                            is_staged: false,
                        });
                    }
                }

                self.git_state.staged = staged;
                self.git_state.unstaged = unstaged;
                self.git_state.status_error = None;
            } else {
                let err = String::from_utf8_lossy(&output.stderr).to_string();
                self.git_state.status_error = Some(err);
            }
        }

        // Fetch git commit history with DAG graph for Graph panel
        let log_out = std::process::Command::new("git")
            .args(["log", "--graph", "-n", "50", "--pretty=format:COMMIT_SPLIT%H|%h|%an|%cr|%s|%D"])
            .current_dir(&root)
            .output();

        if let Ok(lo) = log_out {
            if lo.status.success() {
                let stdout = String::from_utf8_lossy(&lo.stdout);
                let mut history = Vec::new();
                for line in stdout.lines() {
                    if let Some(split_idx) = line.find("COMMIT_SPLIT") {
                        let prefix = &line[..split_idx];
                        let commit_data = &line[split_idx + "COMMIT_SPLIT".len()..];
                        let parts: Vec<&str> = commit_data.splitn(6, '|').collect();
                        if parts.len() >= 5 {
                            let hash = parts[0].to_string();
                            let short_hash = parts[1].to_string();
                            let author = parts[2].to_string();
                            let time_ago = parts[3].to_string();
                            let message = parts[4].to_string();
                            let refs_raw = parts.get(5).unwrap_or(&"");

                            let is_head = history.is_empty() || refs_raw.contains("HEAD");
                            let mut branch_refs = Vec::new();
                            for r in refs_raw.split(',') {
                                let clean = r.trim().trim_start_matches("HEAD -> ").trim();
                                if !clean.is_empty() {
                                    branch_refs.push(clean.to_string());
                                }
                            }

                            // Calculate lane from position of '*' in git's graph prefix
                            let star_pos = prefix.chars().position(|c| c == '*').unwrap_or(0);
                            let lane = (star_pos / 2).min(5);

                            history.push(GitCommitLogItem {
                                hash,
                                short_hash,
                                author,
                                time_ago,
                                message,
                                is_head,
                                branch_refs,
                                lane,
                                graph_prefix: prefix.to_string(),
                            });
                        }
                    }
                }
                self.git_state.commit_history = history;
            }
        }
    }

    pub fn git_stage_file(&mut self, path: &Path) {
        if !self.has_workspace {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["add", "--"])
            .arg(path)
            .current_dir(&self.current_dir)
            .output();
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_unstage_file(&mut self, path: &Path) {
        if !self.has_workspace {
            return;
        }
        let out = std::process::Command::new("git")
            .args(["restore", "--staged", "--"])
            .arg(path)
            .current_dir(&self.current_dir)
            .output();
        if let Ok(o) = out {
            if !o.status.success() {
                let _ = std::process::Command::new("git")
                    .args(["reset", "HEAD", "--"])
                    .arg(path)
                    .current_dir(&self.current_dir)
                    .output();
            }
        }
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_stage_all(&mut self) {
        if !self.has_workspace {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(&self.current_dir)
            .output();
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_unstage_all(&mut self) {
        if !self.has_workspace {
            return;
        }
        let out = std::process::Command::new("git")
            .args(["restore", "--staged", "."])
            .current_dir(&self.current_dir)
            .output();
        if let Ok(o) = out {
            if !o.status.success() {
                let _ = std::process::Command::new("git")
                    .args(["reset", "HEAD"])
                    .current_dir(&self.current_dir)
                    .output();
            }
        }
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_discard_file(&mut self, path: &Path, is_untracked: bool) {
        if !self.has_workspace {
            return;
        }
        if is_untracked {
            if path.is_dir() {
                let _ = std::fs::remove_dir_all(path);
            } else {
                let _ = std::fs::remove_file(path);
            }
        } else {
            let out = std::process::Command::new("git")
                .args(["restore", "--"])
                .arg(path)
                .current_dir(&self.current_dir)
                .output();
            if let Ok(o) = out {
                if !o.status.success() {
                    let _ = std::process::Command::new("git")
                        .args(["checkout", "--"])
                        .arg(path)
                        .current_dir(&self.current_dir)
                        .output();
                }
            }
        }
        self.refresh_git_status();
        self.rebuild_tree();
    }

    pub fn git_commit(&mut self) -> Result<(), String> {
        if !self.has_workspace {
            return Err("No workspace open".to_string());
        }
        let msg = self.git_state.commit_message.trim().to_string();
        if msg.is_empty() {
            return Err("Commit message cannot be empty".to_string());
        }
        if self.git_state.staged.is_empty() && !self.git_state.unstaged.is_empty() {
            self.git_stage_all();
        }
        if self.git_state.staged.is_empty() {
            return Err("No changes to commit".to_string());
        }
        let out = std::process::Command::new("git")
            .args(["commit", "-m", &msg])
            .current_dir(&self.current_dir)
            .output();
        match out {
            Ok(o) if o.status.success() => {
                self.git_state.commit_message.clear();
                self.git_state.status_error = None;
                self.status_message = "Git commit succeeded".to_string();
                self.refresh_git_status();
                self.rebuild_tree();
                Ok(())
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr).to_string();
                let msg = if err.is_empty() {
                    String::from_utf8_lossy(&o.stdout).to_string()
                } else {
                    err
                };
                self.git_state.status_error = Some(msg.clone());
                self.status_message = format!("Commit failed: {}", msg);
                Err(msg)
            }
            Err(e) => {
                let msg = e.to_string();
                self.git_state.status_error = Some(msg.clone());
                self.status_message = format!("Commit error: {}", msg);
                Err(msg)
            }
        }
    }

    pub fn git_init(&mut self) {
        if !self.has_workspace {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&self.current_dir)
            .output();
        self.refresh_git_status();
        self.rebuild_tree();
    }
}
