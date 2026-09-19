use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde_json::Value;

use crate::traits::tool::Tool;
use crate::types::error::ToolError;
use super::diagnostics::GetDiagnosticsTool;
use super::fs::{DeleteFileTool, EditFileTool, GetProjectStructureTool, ListDirectoryTool, ReadFileTool, RenameFileTool, WriteFileTool};
use super::git::{GitDiffTool, GitStatusTool};
use super::plan::{CreatePlanTool, PlanState, UpdatePlanStepTool};
use super::search::{FindFilesTool, SearchCodebaseTool};
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
        registry.register(ReadFileTool);
        registry.register(GetDiagnosticsTool);
        registry
    }

    /// Pre-configured registry with the essential tools:
    /// read_file, write_file, edit_file, list_directory, get_project_structure,
    /// find_files, search_codebase, run_terminal, get_diagnostics.
    pub fn essentials() -> Self {
        let mut registry = Self::minimal();
        registry.register(WriteFileTool);
        registry.register(EditFileTool);
        registry.register(ListDirectoryTool);
        registry.register(GetProjectStructureTool);
        registry.register(FindFilesTool);
        registry.register(SearchCodebaseTool);
        registry.register(RunTerminalTool);
        registry
    }

    /// Pre-configured registry containing all tools (essentials + git + file ops + planner).
    pub fn all() -> Self {
        let mut registry = Self::essentials();
        registry.register(DeleteFileTool);
        registry.register(RenameFileTool);
        registry.register(GitStatusTool);
        registry.register(GitDiffTool);

        let plan_state = Arc::new(Mutex::new(PlanState::default()));
        registry.register(CreatePlanTool::new(plan_state.clone()));
        registry.register(UpdatePlanStepTool::new(plan_state));
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

    /// Find a tool reference by name.
    pub fn find(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    /// Execute a tool by name with provided JSON arguments.
    pub fn execute(&self, name: &str, args: Value) -> Result<Value, ToolError> {
        let tool = self.tools.get(name)
            .ok_or_else(|| ToolError::ExecutionFailed(format!("Tool '{}' not found in registry", name)))?;
        tool.execute(args)
    }

    /// Sorted list of all registered tool names.
    pub fn list_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.tools.keys().map(|k| k.as_str()).collect();
        names.sort();
        names
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
}
