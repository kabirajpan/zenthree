use super::{SubAgent, SubAgentRole};

pub struct PlannerAgent;

impl SubAgent for PlannerAgent {
    fn role(&self) -> SubAgentRole {
        SubAgentRole::Planner
    }

    fn description(&self) -> &'static str {
        "Architectural breakdown, dependency mapping, and structured mission formulation"
    }
}

impl PlannerAgent {
    pub fn new() -> Self {
        Self
    }
}
