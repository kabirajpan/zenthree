use super::{SubAgent, SubAgentRole};

pub struct ExplorerAgent;

impl SubAgent for ExplorerAgent {
    fn role(&self) -> SubAgentRole {
        SubAgentRole::Explorer
    }

    fn description(&self) -> &'static str {
        "Codebase reconnaissance, symbol lookup, and surgical context reading"
    }
}

impl ExplorerAgent {
    pub fn new() -> Self {
        Self
    }
}
