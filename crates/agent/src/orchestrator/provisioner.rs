use std::collections::HashSet;
use crate::orchestrator::intent::AgentIntent;
use crate::orchestrator::reflex::{BioReflexClient, ReflexDecision};
use crate::tools::registry::ToolRegistry;

/// Result of evaluating intent and provisioning tools for a turn.
#[derive(Debug)]
pub enum ProvisioningResult {
    /// Normal provisioning with the set of active tool names.
    ActiveTools(HashSet<String>),
    /// Physical safety reflex lockout engaged — execution must be blocked immediately.
    ReflexLockout(ReflexDecision),
}

/// Dynamic tool provisioner that aligns active tools to detected intent.
#[derive(Default)]
pub struct ToolProvisioner;

impl ToolProvisioner {
    /// Determines the active tools for the upcoming turn based on explicit overrides or biological reflex classification.
    pub fn provision(
        user_input: &str,
        tools: &ToolRegistry,
        explicit_tools: Option<HashSet<String>>,
        explicit_intent: Option<AgentIntent>,
        reflex: &BioReflexClient,
    ) -> ProvisioningResult {
        // Priority 1: Explicit tools override
        if let Some(explicit) = explicit_tools {
            return ProvisioningResult::ActiveTools(explicit);
        }

        // Priority 2: Explicit intent override
        if let Some(intent) = explicit_intent {
            let mut active = HashSet::new();
            match intent {
                AgentIntent::Discussion | AgentIntent::Exploration => {
                    for name in tools.safe_names() {
                        active.insert(name.to_string());
                    }
                }
                AgentIntent::Action(_) => {
                    for name in tools.list_names() {
                        active.insert(name.to_string());
                    }
                }
            }
            return ProvisioningResult::ActiveTools(active);
        }

        // Priority 3: Biological Connectome Reflex Engine classification
        if let Some(decision) = reflex.classify(user_input) {
            if decision.is_danger {
                return ProvisioningResult::ReflexLockout(decision);
            }

            let mut active = HashSet::new();
            match decision.intent {
                AgentIntent::Discussion | AgentIntent::Exploration => {
                    for name in tools.safe_names() {
                        active.insert(name.to_string());
                    }
                }
                AgentIntent::Action(_) => {
                    for name in tools.list_names() {
                        active.insert(name.to_string());
                    }
                }
            }
            return ProvisioningResult::ActiveTools(active);
        }

        // Priority 4: Fallback mode (when reflex engine offline) -> provision all registered tools
        let mut active = HashSet::new();
        for name in tools.list_names() {
            active.insert(name.to_string());
        }
        ProvisioningResult::ActiveTools(active)
    }
}
