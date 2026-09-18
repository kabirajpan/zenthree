use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubAgentRole {
    Planner,
    Explorer,
    Detector,
    Builder,
    Tester,
}

impl SubAgentRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            SubAgentRole::Planner => "Planner",
            SubAgentRole::Explorer => "Explorer",
            SubAgentRole::Detector => "Detector",
            SubAgentRole::Builder => "Builder",
            SubAgentRole::Tester => "Tester",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SubAgentRole::Planner => "📋",
            SubAgentRole::Explorer => "🔍",
            SubAgentRole::Detector => "🧠",
            SubAgentRole::Builder => "⚡",
            SubAgentRole::Tester => "🛡️",
        }
    }

    pub fn badge(&self) -> String {
        format!("{} {}", self.icon(), self.display_name())
    }
}

impl Default for SubAgentRole {
    fn default() -> Self {
        SubAgentRole::Builder
    }
}
