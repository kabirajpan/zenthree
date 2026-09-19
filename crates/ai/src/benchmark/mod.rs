pub mod suite;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use serde::{Deserialize, Serialize};

pub use suite::get_standard_benchmark_suite;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BenchmarkCategory {
    BugFix,
    CodeGeneration,
    Refactoring,
    SelfHealing,
    Scaffolding,
    BoundaryAndRouting,
}

impl BenchmarkCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BugFix => "BugFix",
            Self::CodeGeneration => "CodeGeneration",
            Self::Refactoring => "Refactoring",
            Self::SelfHealing => "SelfHealing",
            Self::Scaffolding => "Scaffolding",
            Self::BoundaryAndRouting => "BoundaryAndRouting",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationRule {
    FileExists(String),
    FileNotExists(String),
    ContentContains { path: String, expected: String },
    ContentNotContains { path: String, unexpected: String },
    NoRootDumping { forbidden_root_dirs: Vec<String> },
    DirectoryContainsFile { dir: String, filename: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkTask {
    pub id: String,
    pub name: String,
    pub category: BenchmarkCategory,
    pub difficulty: BenchmarkDifficulty,
    pub prompt: String,
    pub initial_files: Vec<(String, String)>,
    pub validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub task_id: String,
    pub task_name: String,
    pub category: String,
    pub passed: bool,
    pub score: f32,
    pub duration_ms: u64,
    pub details: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub total_tasks: usize,
    pub passed_tasks: usize,
    pub pass_rate: f32,
    pub avg_duration_ms: f64,
    pub category_scores: HashMap<String, CategoryScore>,
    pub results: Vec<BenchmarkResult>,
    pub timestamp_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryScore {
    pub total: usize,
    pub passed: usize,
    pub pass_rate: f32,
}

impl BenchmarkReport {
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();
        out.push_str("# 📊 Zenthree Autonomous Agent Benchmark Report\n\n");
        out.push_str(&format!(
            "**Overall Pass Rate**: `{:.1}%` ({}/{} tasks passed) | **Average Latency**: `{} ms`\n\n",
            self.pass_rate, self.passed_tasks, self.total_tasks, self.avg_duration_ms as u64
        ));

        out.push_str("### Category Breakdown\n\n");
        out.push_str("| Category | Passed / Total | Pass Rate |\n");
        out.push_str("| :--- | :--- | :--- |\n");

        let mut sorted_categories: Vec<_> = self.category_scores.iter().collect();
        sorted_categories.sort_by_key(|(k, _)| (*k).clone());

        for (cat, score) in sorted_categories {
            out.push_str(&format!(
                "| **{}** | {} / {} | {:.1}% |\n",
                cat, score.passed, score.total, score.pass_rate
            ));
        }
        out.push('\n');

        out.push_str("### Detailed Task Scorecard\n\n");
        out.push_str("| Status | Task ID | Name | Category | Latency |\n");
        out.push_str("| :--- | :--- | :--- | :--- | :--- |\n");

        for r in &self.results {
            let status = if r.passed { "✅ PASS" } else { "❌ FAIL" };
            out.push_str(&format!(
                "| {} | `{}` | {} | {} | {} ms |\n",
                status, r.task_id, r.task_name, r.category, r.duration_ms
            ));
        }

        out
    }

    pub fn to_json_export(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub struct BenchmarkRunner {
    pub tasks: Vec<BenchmarkTask>,
}

impl BenchmarkRunner {
    pub fn new(tasks: Vec<BenchmarkTask>) -> Self {
        Self { tasks }
    }

    pub fn setup_workspace_for_task(task: &BenchmarkTask) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zenthree_bench_{}", task.id));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        for (rel_path, content) in &task.initial_files {
            let full_path = dir.join(rel_path);
            if let Some(parent) = full_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            fs::write(full_path, content).unwrap();
        }

        dir
    }

    pub fn cleanup_workspace(workspace: &Path) {
        let _ = fs::remove_dir_all(workspace);
    }

    pub fn evaluate_validations(workspace: &Path, rules: &[ValidationRule]) -> (bool, Vec<String>) {
        let mut all_passed = true;
        let mut details = Vec::new();

        for rule in rules {
            match rule {
                ValidationRule::FileExists(rel) => {
                    let p = workspace.join(rel);
                    if p.is_file() {
                        details.push(format!("File exists: `{}`", rel));
                    } else {
                        all_passed = false;
                        details.push(format!("FAIL: Expected file `{}` does not exist", rel));
                    }
                }
                ValidationRule::FileNotExists(rel) => {
                    let p = workspace.join(rel);
                    if !p.exists() {
                        details.push(format!("File does not exist (as expected): `{}`", rel));
                    } else {
                        all_passed = false;
                        details.push(format!("FAIL: File `{}` should not exist", rel));
                    }
                }
                ValidationRule::ContentContains { path, expected } => {
                    let p = workspace.join(path);
                    if let Ok(c) = fs::read_to_string(&p) {
                        if c.contains(expected) {
                            details.push(format!("Content in `{}` contains expected text snippet", path));
                        } else {
                            all_passed = false;
                            details.push(format!("FAIL: Content in `{}` missing expected snippet: `{}`", path, expected));
                        }
                    } else {
                        all_passed = false;
                        details.push(format!("FAIL: Cannot read `{}` to check content", path));
                    }
                }
                ValidationRule::ContentNotContains { path, unexpected } => {
                    let p = workspace.join(path);
                    if let Ok(c) = fs::read_to_string(&p) {
                        if !c.contains(unexpected) {
                            details.push(format!("Content in `{}` correctly omits unwanted text", path));
                        } else {
                            all_passed = false;
                            details.push(format!("FAIL: Content in `{}` contains forbidden text: `{}`", path, unexpected));
                        }
                    } else {
                        all_passed = false;
                        details.push(format!("FAIL: Cannot read `{}` to check forbidden content", path));
                    }
                }
                ValidationRule::NoRootDumping { forbidden_root_dirs } => {
                    for forbidden in forbidden_root_dirs {
                        let p = workspace.join(forbidden);
                        if p.exists() {
                            all_passed = false;
                            details.push(format!("FAIL: Anti-root-dumping violated! Root directory contains `{}`", forbidden));
                        } else {
                            details.push(format!("Clean root: `{}` not dumped to root", forbidden));
                        }
                    }
                }
                ValidationRule::DirectoryContainsFile { dir, filename } => {
                    let d = workspace.join(dir);
                    let f = d.join(filename);
                    if f.is_file() {
                        details.push(format!("Directory `{}` contains `{}`", dir, filename));
                    } else {
                        all_passed = false;
                        details.push(format!("FAIL: Directory `{}` missing `{}`", dir, filename));
                    }
                }
            }
        }

        (all_passed, details)
    }

    pub fn run_task<F>(&self, task: &BenchmarkTask, execute_fn: F) -> BenchmarkResult
    where
        F: FnOnce(&Path) -> Result<(), String>,
    {
        let ws = Self::setup_workspace_for_task(task);
        let start = Instant::now();

        let exec_result = execute_fn(&ws);
        let duration_ms = start.elapsed().as_millis() as u64;

        let (passed, details, error) = match exec_result {
            Ok(()) => {
                let (val_passed, val_details) = Self::evaluate_validations(&ws, &task.validation_rules);
                (val_passed, val_details, None)
            }
            Err(e) => (false, vec![format!("Execution error: {}", e)], Some(e)),
        };

        Self::cleanup_workspace(&ws);

        BenchmarkResult {
            task_id: task.id.clone(),
            task_name: task.name.clone(),
            category: task.category.as_str().to_string(),
            passed,
            score: if passed { 100.0 } else { 0.0 },
            duration_ms,
            details,
            error,
        }
    }

    pub fn generate_report(&self, results: Vec<BenchmarkResult>) -> BenchmarkReport {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let pass_rate = if total > 0 { (passed as f32 / total as f32) * 100.0 } else { 0.0 };
        let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();
        let avg_duration_ms = if total > 0 { total_duration as f64 / total as f64 } else { 0.0 };

        let mut cat_counts: HashMap<String, (usize, usize)> = HashMap::new();
        for r in &results {
            let entry = cat_counts.entry(r.category.clone()).or_insert((0, 0));
            entry.0 += 1;
            if r.passed {
                entry.1 += 1;
            }
        }

        let mut category_scores = HashMap::new();
        for (cat, (cat_total, cat_passed)) in cat_counts {
            category_scores.insert(
                cat,
                CategoryScore {
                    total: cat_total,
                    passed: cat_passed,
                    pass_rate: if cat_total > 0 { (cat_passed as f32 / cat_total as f32) * 100.0 } else { 0.0 },
                },
            );
        }

        let timestamp_epoch = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        BenchmarkReport {
            total_tasks: total,
            passed_tasks: passed,
            pass_rate,
            avg_duration_ms,
            category_scores,
            results,
            timestamp_epoch,
        }
    }
}
