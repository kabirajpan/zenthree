pub mod types;

pub use types::*;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiSettingsState {
    #[serde(default)]
    pub open: bool,
    #[serde(skip)]
    pub just_opened: bool,
    #[serde(default)]
    pub active_tab: AiSettingsTab,
    #[serde(default)]
    pub preset_mode: AiPresetMode,

    // Appearance settings
    #[serde(default)]
    pub input_outline: bool,
    #[serde(default = "default_true")]
    pub input_shadow: bool,
    #[serde(default)]
    pub input_tone: AiInputTone,
    #[serde(default)]
    pub message_bg: AiMessageBg,
    #[serde(default)]
    pub text_size: AiTextSize,

    // General preferences
    #[serde(default = "default_true")]
    pub auto_scroll: bool,
    #[serde(default = "default_true")]
    pub show_thoughts: bool,
    #[serde(default)]
    pub sound_notifications: bool,
    #[serde(default = "default_true")]
    pub auto_approve_read: bool,

    // Model settings
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default)]
    pub thinking_budget: AiThinkingBudget,

    // Customization
    #[serde(default)]
    pub custom_instructions: String,

    // API Keys — transient UI state
    #[serde(skip)]
    pub api_key_inputs: std::collections::HashMap<String, String>, // provider_name -> input text
    #[serde(skip)]
    pub revealed_api_keys: std::collections::HashSet<String>,
    #[serde(skip)]
    pub api_key_save_msg: Option<String>,
    // Custom provider form
    #[serde(skip)]
    pub custom_provider_form_open: bool,
    #[serde(skip)]
    pub custom_provider_name: String,
    #[serde(skip)]
    pub custom_provider_key: String,
    #[serde(skip)]
    pub custom_provider_url: String,

    // External Agents — model selection dropdown
    #[serde(skip)]
    pub antigravity_model_dropdown_open: bool,
    #[serde(default)]
    pub selected_antigravity_model_id: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_temperature() -> f32 {
    0.7
}

impl Default for AiSettingsState {
    fn default() -> Self {
        Self {
            open: false,
            just_opened: false,
            active_tab: AiSettingsTab::Appearance,
            preset_mode: AiPresetMode::SystemDefault,
            input_outline: true,
            input_shadow: true,
            input_tone: AiInputTone::Charcoal,
            message_bg: AiMessageBg::DeepDark,
            text_size: AiTextSize::Regular,
            auto_scroll: true,
            show_thoughts: true,
            sound_notifications: false,
            auto_approve_read: true,
            temperature: 0.7,
            thinking_budget: AiThinkingBudget::Budget4k,
            custom_instructions: String::new(),
            api_key_inputs: std::collections::HashMap::new(),
            revealed_api_keys: std::collections::HashSet::new(),
            api_key_save_msg: None,
            custom_provider_form_open: false,
            custom_provider_name: String::new(),
            custom_provider_key: String::new(),
            custom_provider_url: String::new(),
            antigravity_model_dropdown_open: false,
            selected_antigravity_model_id: None,
        }
    }
}

impl AiSettingsState {
    pub fn is_system_default(&self) -> bool {
        self.preset_mode == AiPresetMode::SystemDefault
    }

    pub fn effective_input_outline(&self) -> bool {
        if self.is_system_default() {
            true
        } else {
            self.input_outline
        }
    }

    pub fn effective_input_shadow(&self) -> bool {
        if self.is_system_default() {
            true
        } else {
            self.input_shadow
        }
    }

    pub fn effective_input_tone(&self) -> AiInputTone {
        if self.is_system_default() {
            AiInputTone::Charcoal
        } else {
            self.input_tone
        }
    }

    pub fn effective_message_bg(&self) -> AiMessageBg {
        if self.is_system_default() {
            AiMessageBg::DeepDark
        } else {
            self.message_bg
        }
    }

    pub fn effective_text_size(&self) -> AiTextSize {
        if self.is_system_default() {
            AiTextSize::Regular
        } else {
            self.text_size
        }
    }
}
