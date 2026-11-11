use serde_json::Value;
use std::sync::Arc;

use crate::traits::tool::ToolRisk;

/// Decision outcome returned by Decide hooks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookDecision {
    /// Proceed without modification or user interruption.
    Allow,
    /// Intercept and demand interactive user confirmation with a reason.
    RequireApproval(String),
    /// Abort execution with a hard policy violation error message.
    Deny(String),
}

/// Dynamic hook interface mirroring Antigravity's 3-phase lifecycle.
pub trait AgentHook: Send + Sync {
    /// Name or identifier of this hook for debugging and metrics.
    fn name(&self) -> &str;

    /// Phase 1: Inspect hook for passive telemetry and auditing.
    fn on_turn_start(&self, _prompt: &str) {}
    fn on_tool_start(&self, _tool_name: &str, _args: &Value) {}
    fn on_tool_finish(&self, _tool_name: &str, _result: &Result<Value, String>) {}
    fn on_turn_finish(&self, _response: &str) {}

    /// Phase 2: Decide hook for security, permissions, and policy evaluation.
    fn decide(&self, _tool_name: &str, _args: &Value, _risk: ToolRisk) -> HookDecision {
        HookDecision::Allow
    }

    /// Phase 3: Transform hook for argument sanitization or secret redaction.
    fn transform_args(&self, _tool_name: &str, args: Value) -> Value {
        args
    }
}

/// Registry managing active lifecycle hooks.
#[derive(Default, Clone)]
pub struct HookRegistry {
    hooks: Vec<Arc<dyn AgentHook>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn register<H: AgentHook + 'static>(&mut self, hook: H) {
        self.hooks.push(Arc::new(hook));
    }

    pub fn register_arc(&mut self, hook: Arc<dyn AgentHook>) {
        self.hooks.push(hook);
    }

    pub fn is_empty(&self) -> bool {
        self.hooks.is_empty()
    }

    pub fn trigger_turn_start(&self, prompt: &str) {
        for h in &self.hooks {
            h.on_turn_start(prompt);
        }
    }

    pub fn trigger_tool_start(&self, tool_name: &str, args: &Value) {
        for h in &self.hooks {
            h.on_tool_start(tool_name, args);
        }
    }

    pub fn trigger_tool_finish(&self, tool_name: &str, result: &Result<Value, String>) {
        for h in &self.hooks {
            h.on_tool_finish(tool_name, result);
        }
    }

    pub fn trigger_turn_finish(&self, response: &str) {
        for h in &self.hooks {
            h.on_turn_finish(response);
        }
    }

    /// Evaluates all decide hooks in sequence.
    /// Priority: Deny (immediate hard refusal) > RequireApproval (interactive confirmation) > Allow.
    pub fn evaluate_decide(&self, tool_name: &str, args: &Value, risk: ToolRisk) -> HookDecision {
        let mut approval_needed = None;
        for h in &self.hooks {
            match h.decide(tool_name, args, risk) {
                HookDecision::Deny(reason) => return HookDecision::Deny(reason),
                HookDecision::RequireApproval(reason) => {
                    if approval_needed.is_none() {
                        approval_needed = Some(HookDecision::RequireApproval(reason));
                    }
                }
                HookDecision::Allow => {}
            }
        }
        approval_needed.unwrap_or(HookDecision::Allow)
    }

    /// Pipes arguments through all registered transform hooks.
    pub fn apply_transforms(&self, tool_name: &str, mut args: Value) -> Value {
        for h in &self.hooks {
            args = h.transform_args(tool_name, args);
        }
        args
    }
}
