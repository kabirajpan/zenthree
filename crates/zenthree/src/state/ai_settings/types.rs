#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AiSettingsTab {
    General,
    #[default]
    Appearance,
    Model,
    ExternalAgents,
    Customization,
}

impl AiSettingsTab {
    pub const ALL: [AiSettingsTab; 5] = [
        AiSettingsTab::General,
        AiSettingsTab::Appearance,
        AiSettingsTab::Model,
        AiSettingsTab::ExternalAgents,
        AiSettingsTab::Customization,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            AiSettingsTab::General => "General",
            AiSettingsTab::Appearance => "Appearance",
            AiSettingsTab::Model => "Model",
            AiSettingsTab::ExternalAgents => "External Agents",
            AiSettingsTab::Customization => "Customization",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AiSettingsTab::General => "\u{f013}",       // fa-gear / cog
            AiSettingsTab::Appearance => "\u{f53f}",    // fa-palette
            AiSettingsTab::Model => "\u{f1b2}",         // fa-cube
            AiSettingsTab::ExternalAgents => "\u{f121}",// fa-code / agent
            AiSettingsTab::Customization => "\u{f1de}", // fa-sliders
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AiPresetMode {
    #[default]
    SystemDefault,
    Custom,
}

impl AiPresetMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            AiPresetMode::SystemDefault => "System Default",
            AiPresetMode::Custom => "Custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AiInputTone {
    Dark,       // Deep Dark (#141416)
    #[default]
    Charcoal,   // Charcoal Gray (#202126)
    Bright,     // Brighter Slate Gray (#2c2d34)
}

impl AiInputTone {
    pub fn display_name(&self) -> &'static str {
        match self {
            AiInputTone::Dark => "Dark",
            AiInputTone::Charcoal => "Charcoal",
            AiInputTone::Bright => "Bright",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AiMessageBg {
    #[default]
    DeepDark,   // Deep Dark (#121212)
    Sidebar,    // Sidebar Match (#181818)
}

impl AiMessageBg {
    pub fn display_name(&self) -> &'static str {
        match self {
            AiMessageBg::DeepDark => "Deep Dark",
            AiMessageBg::Sidebar => "Sidebar Match",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AiTextSize {
    Compact,    // 10.5px
    #[default]
    Regular,    // 11.5px
    Large,      // 12.5px
}

impl AiTextSize {
    pub fn pt(&self) -> f32 {
        match self {
            AiTextSize::Compact => 10.5,
            AiTextSize::Regular => 11.5,
            AiTextSize::Large => 12.5,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AiTextSize::Compact => "10.5px",
            AiTextSize::Regular => "11.5px",
            AiTextSize::Large => "12.5px",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum AiThinkingBudget {
    Budget1k,
    #[default]
    Budget4k,
    Budget8k,
    Budget16k,
}

impl AiThinkingBudget {
    pub const ALL: [AiThinkingBudget; 4] = [
        AiThinkingBudget::Budget1k,
        AiThinkingBudget::Budget4k,
        AiThinkingBudget::Budget8k,
        AiThinkingBudget::Budget16k,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            AiThinkingBudget::Budget1k => "1k",
            AiThinkingBudget::Budget4k => "4k",
            AiThinkingBudget::Budget8k => "8k",
            AiThinkingBudget::Budget16k => "16k",
        }
    }

    pub fn tokens(&self) -> u32 {
        match self {
            AiThinkingBudget::Budget1k => 1024,
            AiThinkingBudget::Budget4k => 4096,
            AiThinkingBudget::Budget8k => 8192,
            AiThinkingBudget::Budget16k => 16384,
        }
    }
}
