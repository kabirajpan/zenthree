use super::{SubAgent, SubAgentRole};

pub struct TesterAgent;

impl SubAgent for TesterAgent {
    fn role(&self) -> SubAgentRole {
        SubAgentRole::Tester
    }

    fn description(&self) -> &'static str {
        "Verification gatekeeper, non-destructive compiler checks, and self-healing loop"
    }
}

impl TesterAgent {
    pub fn new() -> Self {
        Self
    }
}
