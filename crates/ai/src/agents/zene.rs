use super::{BuilderAgent, ExplorerAgent, PlannerAgent, SubAgentRole, TesterAgent};

pub struct ZeneAgent {
    pub planner: PlannerAgent,
    pub explorer: ExplorerAgent,
    pub builder: BuilderAgent,
    pub tester: TesterAgent,
}

impl Default for ZeneAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeneAgent {
    pub fn new() -> Self {
        Self {
            planner: PlannerAgent::new(),
            explorer: ExplorerAgent::new(),
            builder: BuilderAgent::new(),
            tester: TesterAgent::new(),
        }
    }

    pub fn name(&self) -> &'static str {
        "ZENE"
    }

    pub fn full_title(&self) -> &'static str {
        "Zenthree Engineering & Neural Executor"
    }

    pub fn sub_agent_role_for_step(&self, step_idx: usize) -> SubAgentRole {
        match step_idx {
            0 => SubAgentRole::Planner,
            1 => SubAgentRole::Explorer,
            2 => SubAgentRole::Builder,
            _ => SubAgentRole::Tester,
        }
    }
}
