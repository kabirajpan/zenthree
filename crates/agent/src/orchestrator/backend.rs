use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AgentBackend {
    #[default]
    Native,
    GooseAcp,
}

impl AgentBackend {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Native => "Native (Built-in)",
            Self::GooseAcp => "Goose (Autonomous ACP)",
        }
    }

    pub fn is_goose(&self) -> bool {
        matches!(self, Self::GooseAcp)
    }

    pub fn is_native(&self) -> bool {
        matches!(self, Self::Native)
    }
}
