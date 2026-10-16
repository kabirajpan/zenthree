use crate::state::ZenthreeState;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceTask {
    pub label: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub is_build_task: bool,
    pub is_default_build_task: bool,
}

fn tasks_file(workspace: &Path) -> PathBuf {
    workspace.join(".vscode").join("tasks.json")
}

fn read_tasks(workspace: &Path) -> Result<(PathBuf, Value), String> {
    let path = tasks_file(workspace);
    let source = std::fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {}", path.display(), error))?;
    let json: Value = serde_json::from_str(&source)
        .map_err(|error| format!("Invalid {}: {}", path.display(), error))?;
    Ok((path, json))
}

pub fn workspace_tasks(workspace: &Path) -> Result<Vec<WorkspaceTask>, String> {
    let (_, json) = read_tasks(workspace)?;
    let Some(tasks) = json.get("tasks").and_then(Value::as_array) else {
        return Err("The workspace tasks.json must contain a tasks array".to_string());
    };
    tasks
        .iter()
        .enumerate()
        .map(|(index, task)| {
            parse_task(task).ok_or_else(|| {
                format!(
                    "Task {} in {} must have a string label and command",
                    index + 1,
                    tasks_file(workspace).display()
                )
            })
        })
        .collect()
}

fn parse_task(task: &Value) -> Option<WorkspaceTask> {
    let label = task.get("label")?.as_str()?.to_string();
    let command = task.get("command")?.as_str()?.to_string();
    let args = task
        .get("args")
        .and_then(Value::as_array)
        .map(|args| {
            args.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();
    let cwd = task
        .get("options")
        .and_then(|options| options.get("cwd"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let group = task.get("group");
    let group_kind = group
        .and_then(|group| group.get("kind").and_then(Value::as_str))
        .or_else(|| group.and_then(Value::as_str));
    let is_build_task = group_kind == Some("build");
    let is_default_build_task = is_build_task
        && group
            .and_then(|group| group.get("isDefault"))
            .and_then(Value::as_bool)
            .unwrap_or(false);
    Some(WorkspaceTask {
        label,
        command,
        args,
        cwd,
        is_build_task,
        is_default_build_task,
    })
}

impl ZenthreeState {
    pub fn get_workspace_tasks(&mut self) -> Vec<WorkspaceTask> {
        match workspace_tasks(&self.current_dir) {
            Ok(tasks) => tasks,
            Err(_) if !tasks_file(&self.current_dir).exists() => Vec::new(),
            Err(error) => {
                self.set_status(error);
                Vec::new()
            }
        }
    }

    pub fn run_workspace_task(&mut self, label: &str) -> bool {
        let tasks = match workspace_tasks(&self.current_dir) {
            Ok(tasks) => tasks,
            Err(error) => {
                self.set_status(error);
                return false;
            }
        };
        let Some(task) = tasks.iter().find(|task| task.label == label).cloned() else {
            self.set_status(format!("Task '{}' was not found", label));
            return false;
        };
        self.run_task(&task);
        true
    }

    pub fn run_default_build_task(&mut self) -> bool {
        let tasks = match workspace_tasks(&self.current_dir) {
            Ok(tasks) => tasks,
            Err(error) => {
                self.set_status(error);
                return false;
            }
        };
        let Some(task) = tasks
            .iter()
            .find(|task| task.is_default_build_task)
            .cloned()
        else {
            self.set_status("No default build task is configured");
            return false;
        };
        self.run_task(&task);
        true
    }

    fn run_task(&mut self, task: &WorkspaceTask) {
        let file_path = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .and_then(|tab| tab.path.as_ref())
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let substitutions = [
            ("${workspaceFolder}", self.current_dir.display().to_string()),
            ("${file}", file_path.clone()),
            (
                "${fileBasename}",
                Path::new(&file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            (
                "${fileDirname}",
                Path::new(&file_path)
                    .parent()
                    .map(|path| path.display().to_string())
                    .unwrap_or_default(),
            ),
        ];
        let expand = |value: &str| {
            substitutions
                .iter()
                .fold(value.to_string(), |value, (key, replacement)| {
                    value.replace(key, replacement)
                })
        };
        let mut command = expand(&task.command);
        for arg in &task.args {
            command.push(' ');
            command.push_str(&crate::state::terminal::quote_shell_argument(&expand(arg)));
        }
        let cwd = task
            .cwd
            .as_deref()
            .map(expand)
            .map(PathBuf::from)
            .map(|cwd| {
                if cwd.is_absolute() {
                    cwd
                } else {
                    self.current_dir.join(cwd)
                }
            })
            .unwrap_or_else(|| self.current_dir.clone());
        let command = crate::state::terminal::command_in_directory(&cwd, &command);

        self.show_terminal();
        if let Some(session) = self.active_terminal_session_mut() {
            session.write_line(&command);
            self.focused_panel = crate::state::types::FocusedPanel::Terminal;
            self.last_cursor_activity = std::time::Instant::now();
            self.set_status(format!("Running task: {}", task.label));
        } else {
            self.set_status("Could not start the integrated terminal");
        }
    }

    pub fn configure_workspace_tasks(&mut self) {
        let path = tasks_file(&self.current_dir);
        if !path.exists() {
            let template = json!({
                "version": "2.0.0",
                "tasks": []
            });
            if let Some(parent) = path.parent() {
                if let Err(error) = std::fs::create_dir_all(parent) {
                    self.set_status(format!("Could not create tasks directory: {}", error));
                    return;
                }
            }
            let contents = match serde_json::to_string_pretty(&template) {
                Ok(contents) => contents,
                Err(error) => {
                    self.set_status(format!("Could not create task template: {}", error));
                    return;
                }
            };
            if let Err(error) = std::fs::write(&path, contents) {
                self.set_status(format!("Could not create {}: {}", path.display(), error));
                return;
            }
        }
        self.open_file(&path);
    }

    pub fn set_default_build_task(&mut self, label: &str) {
        let (path, mut json) = match read_tasks(&self.current_dir) {
            Ok(tasks) => tasks,
            Err(error) => {
                self.set_status(error);
                return;
            }
        };
        let Some(tasks) = json.get_mut("tasks").and_then(Value::as_array_mut) else {
            self.set_status("The workspace tasks.json must contain a tasks array");
            return;
        };
        let mut found = false;
        for task in tasks.iter_mut() {
            let is_selected = task.get("label").and_then(Value::as_str) == Some(label);
            let is_build = task.get("group").and_then(|group| {
                group
                    .get("kind")
                    .and_then(Value::as_str)
                    .or_else(|| group.as_str())
            }) == Some("build");
            if is_selected && !is_build {
                self.set_status("Only a task in the build group can be the default build task");
                return;
            }
            if is_build {
                task["group"] = json!({
                    "kind": "build",
                    "isDefault": is_selected
                });
                found |= is_selected;
            }
        }
        if !found {
            self.set_status(format!("Build task '{}' was not found", label));
            return;
        }
        match serde_json::to_string_pretty(&json)
            .map_err(|error| error.to_string())
            .and_then(|contents| std::fs::write(&path, contents).map_err(|error| error.to_string()))
        {
            Ok(()) => {
                self.set_status(format!("Default build task: {}", label));
                self.fs_watcher.watch(&self.current_dir);
            }
            Err(error) => {
                self.set_status(format!("Could not update {}: {}", path.display(), error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::workspace_tasks;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_build_group_and_args() {
        let dir = std::env::temp_dir().join(format!(
            "zenthree_tasks_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(dir.join(".vscode")).unwrap();
        std::fs::write(
            dir.join(".vscode").join("tasks.json"),
            r#"{"version":"2.0.0","tasks":[{"label":"Build app","type":"process","command":"cargo","args":["build"],"group":{"kind":"build","isDefault":true}}]}"#,
        )
        .unwrap();

        let tasks = workspace_tasks(&dir).unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].command, "cargo");
        assert_eq!(tasks[0].args, ["build"]);
        assert!(tasks[0].is_build_task);
        assert!(tasks[0].is_default_build_task);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
