use super::{SubAgent, SubAgentRole};

pub struct BuilderAgent;

impl SubAgent for BuilderAgent {
    fn role(&self) -> SubAgentRole {
        SubAgentRole::Builder
    }

    fn description(&self) -> &'static str {
        "Code synthesis, surgical SEARCH/REPLACE diff formulation, and file creation"
    }
}

impl BuilderAgent {
    pub fn new() -> Self {
        Self
    }
}
