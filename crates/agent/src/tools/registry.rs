use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;

use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;
use super::diagnostics::GetDiagnosticsTool;
use super::fs::{DeleteFileTool, EditFileTool, ListDirectoryTool, MultiReplaceFileContentTool, ReadFileTool, RenameFileTool, WriteFileTool};
use super::git::{GitDiffTool, GitStatusTool};
use super::plan::{CreatePlanTool, PlanState, UpdatePlanStepTool};
use super::search::SearchTool;
use super::subagent::StartSubagentTool;
use super::terminal::RunTerminalTool;

/// Central registry managing agent tools, schema exports, and dynamic execution.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::minimal()
    }
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Minimal 2-tool registry for initial loop proof:
    /// read_file and get_diagnostics (both read-only, no approval needed).
    pub fn minimal() -> Self {
        let mut registry = Self::new();
        registry.register(ReadFileTool::new());
        registry.register(GetDiagnosticsTool);
        registry
    }

    /// Pre-configured registry with the essential tools:
    /// read_file, write_file, edit_file, multi_replace_file_content, list_directory, search,
    /// run_terminal, get_diagnostics, create_plan, update_plan_step.
    pub fn essentials() -> Self {
        let mut registry = Self::minimal();
        registry.register(WriteFileTool::default());
        registry.register(EditFileTool::default());
        registry.register(MultiReplaceFileContentTool::default());
        registry.register(ListDirectoryTool::new());
        registry.register(SearchTool::new());
        registry.register(RunTerminalTool::new());

        let plan_state = Arc::new(Mutex::new(PlanState::default()));
        registry.register(CreatePlanTool::new(plan_state.clone()));
        registry.register(UpdatePlanStepTool::new(plan_state));
        registry
    }

    /// Pre-configured registry containing all tools (essentials + git + file ops + tasks + schedule).
    pub fn all() -> Self {
        let mut registry = Self::essentials();
        registry.register(DeleteFileTool::default());
        registry.register(RenameFileTool::default());
        registry.register(GitStatusTool);
        registry.register(GitDiffTool);
        registry.register(StartSubagentTool::default());

        let task_mgr = crate::tasks::TaskManager::default();
        let notifs = Arc::new(Mutex::new(Vec::new()));
        let scheduler = crate::tasks::Scheduler::new(notifs);
        registry.register(crate::tools::ManageTaskTool::new(task_mgr));
        registry.register(crate::tools::ScheduleTool::new(scheduler));

        registry
    }

    /// Register a new tool.
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    /// Register an already boxed tool.
    pub fn register_boxed(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    /// Find a tool reference by name, stripping hallucinated namespaces (e.g., "repo_browser.get_project_structure" -> "get_project_structure")
    /// and resolving legacy aliases ("find_files", "search_codebase" -> "search", "get_project_structure" -> "list_directory").
    pub fn find(&self, name: &str) -> Option<&dyn Tool> {
        let lookup_name = if let Some(pos) = name.rfind('.') {
            &name[pos + 1..]
        } else {
            name
        };

        if let Some(tool) = self.tools.get(lookup_name) {
            return Some(tool.as_ref());
        }

        // Canonical specification aliases:
        match lookup_name {
            "replace_file_content" => self.tools.get("edit_file").map(|t| t.as_ref()),
            "write_to_file" => self.tools.get("write_file").map(|t| t.as_ref()),
            "view_file" => self.tools.get("read_file").map(|t| t.as_ref()),
            "run_command" => self.tools.get("run_terminal").map(|t| t.as_ref()),
            "list_dir" | "get_project_structure" => self.tools.get("list_directory").map(|t| t.as_ref()),
            "grep_search" | "find_files" | "search_codebase" => self.tools.get("search").map(|t| t.as_ref()),
            _ => None,
        }
    }

    /// Execute a tool by name with provided JSON arguments directly.
    pub fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        let tool = self.find(name)
            .ok_or_else(|| ToolError::ExecutionFailed(format!("Tool '{}' not found in registry", name)))?;
        tool.execute(args)
    }

    /// Sorted list of all registered tool names.
    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tools.keys().map(|k| k.as_str()).collect();
        names.sort();
        names
    }

    /// Sorted list of all registered tool names matching a specific risk level.
    pub fn list_names_by_risk(&self, risk: ToolRisk) -> Vec<&str> {
        let mut names: Vec<&str> = self.tools.values()
            .filter(|t| t.risk() == risk)
            .map(|t| t.name())
            .collect();
        names.sort();
        names
    }

    /// Sorted list of all registered safe (ReadOnly) tool names.
    pub fn safe_names(&self) -> Vec<&str> {
        self.list_names_by_risk(ToolRisk::ReadOnly)
    }

    /// Total count of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns true if no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// List of tool parameter schemas.
    pub fn schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| t.schema()).collect()
    }

    /// List of tool schemas formatted for OpenAI / Groq tool calling.
    pub fn openai_schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| t.to_openai_schema()).collect()
    }

    /// Generate tools specification array for OpenAI / Groq tool calling.
    pub fn to_openai_tools(&self) -> Value {
        let items: Vec<Value> = self.tools.values().map(|t| t.to_openai_schema()).collect();
        Value::Array(items)
    }

    /// Generate functionDeclarations specification array for Gemini tool calling.
    pub fn to_gemini_tools(&self) -> Value {
        let items: Vec<Value> = self.tools.values().map(|t| t.to_gemini_schema()).collect();
        Value::Array(items)
    }

    /// Generate a compact 1-line tool manifest (name + one-line description, ~50 tokens total).
    pub fn manifest(&self) -> String {
        let names = self.list_names();
        let mut lines = Vec::new();
        for name in names {
            if let Some(tool) = self.tools.get(name) {
                // First line of description
                let desc = tool.description().lines().next().unwrap_or("").trim();
                lines.push(format!("{:20} : {}", name, desc));
            }
        }
        lines.join("\n")
    }

    /// Returns pairs of (tool_name, one_line_description) for all registered tools.
    pub fn manifest_pairs(&self) -> Vec<(String, String)> {
        let names = self.list_names();
        let mut pairs = Vec::new();
        for name in names {
            if let Some(tool) = self.tools.get(name) {
                let desc = tool.description().lines().next().unwrap_or("").trim().to_string();
                pairs.push((name.to_string(), desc));
            }
        }
        pairs
    }

    /// List of tool schemas formatted for OpenAI / Groq tool calling, filtered to specified tool names.
    pub fn openai_schemas_for(&self, names: &[&str]) -> Vec<Value> {
        let mut schemas = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for name in names {
            if let Some(tool) = self.find(*name) {
                if seen.insert(tool.name()) {
                    schemas.push(tool.to_openai_schema());
                }
            }
        }
        schemas
    }

    /// List of tool schemas formatted for Gemini tool calling, filtered to specified tool names.
    pub fn gemini_schemas_for(&self, names: &[&str]) -> Vec<Value> {
        let mut schemas = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for name in names {
            if let Some(tool) = self.find(*name) {
                if seen.insert(tool.name()) {
                    schemas.push(tool.to_gemini_schema());
                }
            }
        }
        schemas
    }
}
