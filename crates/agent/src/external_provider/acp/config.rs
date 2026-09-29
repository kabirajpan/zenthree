use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Configuration for launching an external ACP agent.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AcpAgentConfig {
    /// Name or identifier for this agent (e.g. "antigravity-acp", "claude-code")
    pub id: String,
    /// Display name (e.g. "Google Antigravity")
    pub name: String,
    /// Path to executable command (e.g. `/path/to/agy_acp_server.par` or `npx`)
    pub command: PathBuf,
    /// Command-line arguments passed to the agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables to set for the agent process
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    /// Optional authentication method id to auto-authenticate with (e.g. "oauth-personal")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_method_id: Option<String>,
    /// Default model requested for new sessions (if supported)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    /// Default mode requested (e.g. "default", "yolo", "auto_edit")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
}

impl AcpAgentConfig {
    /// Create a new ACP agent configuration.
    pub fn new(id: impl Into<String>, name: impl Into<String>, command: impl Into<PathBuf>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            auth_method_id: None,
            default_model: None,
            default_mode: None,
        }
    }

    /// Add a command-line argument.
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple command-line arguments.
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Set an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the authentication method ID.
    pub fn with_auth_method(mut self, method_id: impl Into<String>) -> Self {
        self.auth_method_id = Some(method_id.into());
        self
    }

    /// Convert into the underlying `agent_client_protocol::AcpAgentConfig` for process spawning.
    pub fn to_sdk_config(&self) -> agent_client_protocol::AcpAgentConfig {
        let mut sdk_config = agent_client_protocol::AcpAgentConfig::new(&self.command);
        sdk_config = sdk_config.args(&self.args);
        sdk_config = sdk_config.envs(&self.env);
        sdk_config
    }

    /// Attempts to auto-detect a configured or installed Google Antigravity ACP agent binary.
    ///
    /// Checks in order:
    /// 1. `ANTIGRAVITY_ACP_PATH` environment variable.
    /// 2. Zenthree's own external agent directory: `~/.local/share/zenthree/external_agents/`
    /// 3. Zed's external agent directory (suggested fallback): `~/.local/share/zed/external_agents/`
    /// 4. System PATH for `agy_acp_server.par` or `agy_acp_server`
    pub fn suggest_antigravity() -> Option<Self> {
        let path = detect_antigravity_binary()?;

        let mut config = Self::new("antigravity-acp", "Google Antigravity", path);

        #[cfg(target_os = "linux")]
        {
            // On Linux, Google's PAR launcher requires --uid= to stay as the current user
            config.args.push("--uid=".to_string());
        }

        // Do not force interactive authenticate on every session; let agent load its configured auth from settings.json
        config.auth_method_id = None;
        Some(config)
    }
}

/// Look for the `agy_acp_server` executable across candidate paths without hardcoding a single fixed path.
fn detect_antigravity_binary() -> Option<PathBuf> {
    // 1. Env override
    if let Ok(path_str) = std::env::var("ANTIGRAVITY_ACP_PATH") {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
    }

    let home = dirs::home_dir()?;

    // 2. Zenthree external agents directory
    let zenthree_agent_dir = home.join(".local/share/zenthree/external_agents");
    if let Some(p) = find_binary_in_tree(&zenthree_agent_dir) {
        return Some(p);
    }

    // 3. Zed registry external agents directory (optional detection)
    let zed_agent_dir = home.join(".local/share/zed/external_agents/registry/antigravity-acp");
    if let Some(p) = find_binary_in_tree(&zed_agent_dir) {
        return Some(p);
    }

    // 4. Check PATH
    if let Ok(path_env) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_env) {
            let candidate1 = dir.join("agy_acp_server.par");
            if candidate1.exists() {
                return Some(candidate1);
            }
            let candidate2 = dir.join("agy_acp_server");
            if candidate2.exists() {
                return Some(candidate2);
            }
        }
    }

    None
}

fn find_binary_in_tree(dir: &Path) -> Option<PathBuf> {
    if !dir.exists() {
        return None;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = find_binary_in_tree(&path) {
                    return Some(found);
                }
            } else if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name == "agy_acp_server.par" || file_name == "agy_acp_server" || file_name == "agy_acp_server.exe" {
                    return Some(path);
                }
            }
        }
    }

    None
}
