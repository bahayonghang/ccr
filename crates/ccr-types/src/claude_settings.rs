//! Claude Settings Types
//!
//! Shared types for Claude Code settings management.
//! All nested types preserve unknown fields via `#[serde(flatten)] other`.

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Helper function to check if a bool is false (for skip_serializing_if)
pub fn is_false(b: &bool) -> bool {
    !*b
}

/// Helper function to return true as default
pub fn default_true() -> bool {
    true
}

fn value_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Canonical Claude Code hooks shape: `event -> matcher groups[]`.
pub type HooksConfig = HashMap<String, Vec<HookMatcherGroup>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyHook {
    pub event: String,
    pub command: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

fn legacy_hooks_to_config(legacy_hooks: Vec<LegacyHook>) -> HooksConfig {
    let mut config = HooksConfig::new();

    for legacy in legacy_hooks {
        let mut handler_other = legacy.other;
        if !legacy.enabled {
            handler_other.insert("enabled".to_string(), Value::Bool(false));
        }
        if let Some(description) = legacy.description {
            handler_other.insert("description".to_string(), Value::String(description));
        }

        config
            .entry(legacy.event)
            .or_default()
            .push(HookMatcherGroup {
                matcher: None,
                hooks: vec![Hook {
                    handler_type: "command".to_string(),
                    command: Some(legacy.command),
                    url: None,
                    prompt: None,
                    model: None,
                    timeout: None,
                    status_message: None,
                    allowed_env_vars: None,
                    headers: None,
                    async_execution: None,
                    other: handler_other,
                }],
                other: HashMap::new(),
            });
    }

    config
}

/// Deserialize hooks.
///
/// Canonical input is the official object-based format:
/// `{ "PreToolUse": [{ "matcher": "...", "hooks": [...] }] }`
///
/// Legacy array-based hooks are still accepted for backward compatibility and
/// are normalized into the canonical grouped format on write.
fn deserialize_hooks<'de, D>(deserializer: D) -> Result<HooksConfig, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(HooksConfig::new()),
        Value::Object(_) => serde_json::from_value(value).map_err(serde::de::Error::custom),
        Value::Array(_) => {
            let legacy_hooks: Vec<LegacyHook> =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(legacy_hooks_to_config(legacy_hooks))
        }
        other => Err(serde::de::Error::custom(format!(
            "invalid type for hooks: expected object or array, got {}",
            value_type_name(&other)
        ))),
    }
}

/// Claude Code settings structure
///
/// Complete settings for Claude Code, including all known fields
/// and preserving unknown fields for forward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeSettings {
    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Output style.
    ///
    /// Serializes as `outputStyle`, accepts legacy `output_style` on input.
    #[serde(
        rename = "outputStyle",
        alias = "output_style",
        skip_serializing_if = "Option::is_none"
    )]
    pub output_style: Option<String>,

    /// Permissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Value>,

    /// MCP Servers
    #[serde(
        default,
        rename = "mcpServers",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub mcp_servers: HashMap<String, McpServer>,

    /// Slash Commands
    #[serde(
        default,
        rename = "slashCommands",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub slash_commands: Vec<SlashCommand>,

    /// Agents
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<Agent>,

    /// Plugins
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<Plugin>,

    /// Hooks
    #[serde(
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_hooks"
    )]
    pub hooks: HooksConfig,

    /// Other unknown fields (for forward compatibility)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Environment variable keys managed by CCR inside `settings.json`'s `env` map.
///
/// These are the keys written by profile switching and cleared by
/// [`ClaudeSettings::clear_managed_vars`]. Mapping from profile fields lives in
/// `ccr-config` (`ConfigSection::to_managed_env_pairs`), which references these
/// constants so key names never drift between crates.
pub mod env_keys {
    pub const ANTHROPIC_BASE_URL: &str = "ANTHROPIC_BASE_URL";
    pub const ANTHROPIC_AUTH_TOKEN: &str = "ANTHROPIC_AUTH_TOKEN";
    pub const ANTHROPIC_MODEL: &str = "ANTHROPIC_MODEL";
    pub const ANTHROPIC_SMALL_FAST_MODEL: &str = "ANTHROPIC_SMALL_FAST_MODEL";
    pub const ANTHROPIC_DEFAULT_OPUS_MODEL: &str = "ANTHROPIC_DEFAULT_OPUS_MODEL";
    pub const ANTHROPIC_DEFAULT_SONNET_MODEL: &str = "ANTHROPIC_DEFAULT_SONNET_MODEL";
    pub const ANTHROPIC_DEFAULT_HAIKU_MODEL: &str = "ANTHROPIC_DEFAULT_HAIKU_MODEL";
    pub const ANTHROPIC_DEFAULT_FABLE_MODEL: &str = "ANTHROPIC_DEFAULT_FABLE_MODEL";
    pub const ANTHROPIC_DEFAULT_OPUS_MODEL_NAME: &str = "ANTHROPIC_DEFAULT_OPUS_MODEL_NAME";
    pub const ANTHROPIC_DEFAULT_SONNET_MODEL_NAME: &str = "ANTHROPIC_DEFAULT_SONNET_MODEL_NAME";
    pub const ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME: &str = "ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME";
    pub const ANTHROPIC_DEFAULT_FABLE_MODEL_NAME: &str = "ANTHROPIC_DEFAULT_FABLE_MODEL_NAME";
    pub const ANTHROPIC_CUSTOM_MODEL_OPTION: &str = "ANTHROPIC_CUSTOM_MODEL_OPTION";
    pub const ANTHROPIC_CUSTOM_MODEL_OPTION_NAME: &str = "ANTHROPIC_CUSTOM_MODEL_OPTION_NAME";
    pub const CLAUDE_CODE_SUBAGENT_MODEL: &str = "CLAUDE_CODE_SUBAGENT_MODEL";
    pub const CLAUDE_CODE_EFFORT_LEVEL: &str = "CLAUDE_CODE_EFFORT_LEVEL";
    pub const CLAUDE_CODE_AUTO_COMPACT_WINDOW: &str = "CLAUDE_CODE_AUTO_COMPACT_WINDOW";
    pub const API_TIMEOUT_MS: &str = "API_TIMEOUT_MS";
    pub const CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC: &str =
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC";

    /// Managed env keys that do not carry the `ANTHROPIC_` prefix.
    ///
    /// These are explicitly configured by profiles, so profile switching must
    /// clear them together with the `ANTHROPIC_*` overrides.
    pub const NON_ANTHROPIC_MANAGED_KEYS: &[&str] = &[
        CLAUDE_CODE_SUBAGENT_MODEL,
        CLAUDE_CODE_EFFORT_LEVEL,
        CLAUDE_CODE_AUTO_COMPACT_WINDOW,
        API_TIMEOUT_MS,
        CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC,
    ];
}

impl ClaudeSettings {
    /// Create an empty settings value (same as `Default::default()`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove every `ANTHROPIC_*` entry from `env`, keeping all other keys.
    pub fn clear_anthropic_vars(&mut self) {
        self.env.retain(|key, _| !key.starts_with("ANTHROPIC_"));
    }

    /// Remove every CCR-managed entry from `env`.
    ///
    /// Covers all `ANTHROPIC_*` keys plus [`env_keys::NON_ANTHROPIC_MANAGED_KEYS`].
    pub fn clear_managed_vars(&mut self) {
        self.clear_anthropic_vars();
        for key in env_keys::NON_ANTHROPIC_MANAGED_KEYS {
            self.env.remove(*key);
        }
    }

    /// Replace the managed portion of `env` with `pairs`.
    ///
    /// Clears all managed keys first (see [`Self::clear_managed_vars`]), then
    /// inserts each pair. Non-managed env entries are untouched. This is the
    /// data-side core of profile switching; callers obtain `pairs` from
    /// `ConfigSection::to_managed_env_pairs` in `ccr-config`.
    pub fn apply_managed_env(&mut self, pairs: impl IntoIterator<Item = (String, String)>) {
        self.clear_managed_vars();
        for (key, value) in pairs {
            self.env.insert(key, value);
        }
    }

    /// Snapshot of the managed `ANTHROPIC_*`/runtime env values for display.
    ///
    /// Returns each known managed key mapped to its current value (or `None`).
    pub fn anthropic_env_status(&self) -> HashMap<String, Option<String>> {
        let vars = [
            env_keys::ANTHROPIC_BASE_URL,
            env_keys::ANTHROPIC_AUTH_TOKEN,
            env_keys::ANTHROPIC_MODEL,
            env_keys::ANTHROPIC_SMALL_FAST_MODEL,
            env_keys::ANTHROPIC_DEFAULT_OPUS_MODEL,
            env_keys::ANTHROPIC_DEFAULT_SONNET_MODEL,
            env_keys::ANTHROPIC_DEFAULT_HAIKU_MODEL,
            env_keys::ANTHROPIC_DEFAULT_FABLE_MODEL,
            env_keys::ANTHROPIC_DEFAULT_OPUS_MODEL_NAME,
            env_keys::ANTHROPIC_DEFAULT_SONNET_MODEL_NAME,
            env_keys::ANTHROPIC_DEFAULT_HAIKU_MODEL_NAME,
            env_keys::ANTHROPIC_DEFAULT_FABLE_MODEL_NAME,
            env_keys::CLAUDE_CODE_AUTO_COMPACT_WINDOW,
            env_keys::API_TIMEOUT_MS,
            env_keys::CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC,
        ];

        let mut status = HashMap::new();
        for var in vars {
            status.insert(var.to_string(), self.env.get(var).cloned());
        }
        status
    }

    /// Whether any `ANTHROPIC_*` override is present in `env`.
    pub fn has_anthropic_overrides(&self) -> bool {
        self.env.keys().any(|key| key.starts_with("ANTHROPIC_"))
    }

    /// Strictly validate the env vars required by API-key mode.
    ///
    /// The error is a plain message; callers wrap it into their own error type
    /// (e.g. `CcrError::ValidationError`).
    pub fn validate_api_key_mode(&self) -> Result<(), String> {
        let base_url = self
            .env
            .get(env_keys::ANTHROPIC_BASE_URL)
            .ok_or_else(|| "缺少必需的环境变量: ANTHROPIC_BASE_URL".to_string())?;

        if base_url.trim().is_empty() {
            return Err("环境变量不能为空: ANTHROPIC_BASE_URL".to_string());
        }

        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err("ANTHROPIC_BASE_URL 必须以 http:// 或 https:// 开头".to_string());
        }

        let auth_token = self
            .env
            .get(env_keys::ANTHROPIC_AUTH_TOKEN)
            .ok_or_else(|| "缺少必需的环境变量: ANTHROPIC_AUTH_TOKEN".to_string())?;

        if auth_token.trim().is_empty() {
            return Err("环境变量不能为空: ANTHROPIC_AUTH_TOKEN".to_string());
        }

        Ok(())
    }

    /// Validate the `ANTHROPIC_*` overrides in these settings.
    ///
    /// Subscription mode (no `ANTHROPIC_*` overrides at all) is valid; once any
    /// override exists, the strict API-key-mode rules apply.
    pub fn validate(&self) -> Result<(), String> {
        if !self.has_anthropic_overrides() {
            return Ok(());
        }
        self.validate_api_key_mode()
    }
}

/// MCP Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    /// Command to execute (for stdio-based servers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// URL endpoint (for HTTP/SSE-based servers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    /// Whether the server is disabled
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
    /// Other unknown fields (for forward compatibility)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Slash command configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashCommand {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// Command to execute
    pub command: String,
    /// Command arguments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Whether the command is disabled
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
    /// Other unknown fields (for forward compatibility)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Agent name
    pub name: String,
    /// Model to use
    pub model: String,
    /// Available tools
    #[serde(default)]
    pub tools: Vec<String>,
    /// System prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Whether the agent is disabled
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
    /// Other unknown fields (for forward compatibility)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    /// Plugin ID
    pub id: String,
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Whether the plugin is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Plugin configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    /// Other unknown fields (for forward compatibility)
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Hook matcher group.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookMatcherGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matcher: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<Hook>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

/// Hook handler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    #[serde(rename = "type")]
    pub handler_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(rename = "statusMessage", skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    #[serde(rename = "allowedEnvVars", skip_serializing_if = "Option::is_none")]
    pub allowed_env_vars: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "async", skip_serializing_if = "Option::is_none")]
    pub async_execution: Option<bool>,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_settings_roundtrip() {
        let settings = ClaudeSettings {
            env: HashMap::from([(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.anthropic.com".to_string(),
            )]),
            output_style: Some("nekomata-engineer".to_string()),
            permissions: None,
            mcp_servers: HashMap::new(),
            slash_commands: Vec::new(),
            agents: Vec::new(),
            plugins: Vec::new(),
            hooks: HashMap::from([(
                "PostToolUse".to_string(),
                vec![HookMatcherGroup {
                    matcher: Some("Write|Edit".to_string()),
                    hooks: vec![Hook {
                        handler_type: "command".to_string(),
                        command: Some("./check-style.sh".to_string()),
                        url: None,
                        prompt: None,
                        model: None,
                        timeout: Some(30),
                        status_message: None,
                        allowed_env_vars: None,
                        headers: None,
                        async_execution: None,
                        other: HashMap::new(),
                    }],
                    other: HashMap::new(),
                }],
            )]),
            other: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        let parsed: ClaudeSettings = serde_json::from_str(&json).unwrap();
        assert!(json.contains("outputStyle"));
        assert!(!json.contains("output_style"));

        assert_eq!(
            parsed.env.get("ANTHROPIC_BASE_URL").unwrap(),
            "https://api.anthropic.com"
        );
        assert_eq!(parsed.output_style, Some("nekomata-engineer".to_string()));
        assert_eq!(
            parsed
                .hooks
                .get("PostToolUse")
                .and_then(|groups| groups.first())
                .and_then(|group| group.matcher.as_deref()),
            Some("Write|Edit")
        );
    }

    #[test]
    fn test_claude_settings_unknown_fields_preserved() {
        let json = r#"{
            "env": {},
            "outputStyle": "test",
            "future_field": "should be preserved",
            "another_unknown": 42
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();

        assert!(settings.other.contains_key("future_field"));
        assert_eq!(
            settings.other.get("future_field").unwrap(),
            "should be preserved"
        );
        assert_eq!(settings.other.get("another_unknown").unwrap(), 42);

        let serialized = serde_json::to_string(&settings).unwrap();
        assert!(serialized.contains("future_field"));
        assert!(serialized.contains("another_unknown"));
    }

    #[test]
    fn test_output_style_alias_deserialization() {
        let json = r#"{
            "env": {},
            "output_style": "legacy-style"
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.output_style, Some("legacy-style".to_string()));
    }

    #[test]
    fn test_output_style_duplicate_keys_rejected() {
        let json = r#"{
            "env": {},
            "outputStyle": "new-style",
            "output_style": "legacy-style"
        }"#;

        let err = serde_json::from_str::<ClaudeSettings>(json).unwrap_err();
        assert!(err.to_string().contains("duplicate field"));
    }

    #[test]
    fn test_mcp_server_unknown_fields_preserved() {
        let json = r#"{
            "command": "node",
            "args": ["server.js"],
            "future_mcp_field": "preserved"
        }"#;

        let server: McpServer = serde_json::from_str(json).unwrap();
        assert_eq!(server.command, Some("node".to_string()));
        assert!(server.other.contains_key("future_mcp_field"));

        let serialized = serde_json::to_string(&server).unwrap();
        assert!(serialized.contains("future_mcp_field"));
    }

    #[test]
    fn test_slash_command_unknown_fields_preserved() {
        let json = r#"{
            "name": "test",
            "description": "Test command",
            "command": "echo",
            "new_slash_field": true
        }"#;

        let cmd: SlashCommand = serde_json::from_str(json).unwrap();
        assert_eq!(cmd.name, "test");
        assert!(cmd.other.contains_key("new_slash_field"));
    }

    #[test]
    fn test_hook_handler_unknown_fields_preserved() {
        let json = r#"{
            "type": "command",
            "command": "echo ok",
            "future_hook_field": true
        }"#;

        let hook: Hook = serde_json::from_str(json).unwrap();
        assert_eq!(hook.handler_type, "command");
        assert!(hook.other.contains_key("future_hook_field"));

        let serialized = serde_json::to_string(&hook).unwrap();
        assert!(serialized.contains("future_hook_field"));
    }

    #[test]
    fn test_hook_matcher_group_unknown_fields_preserved() {
        let json = r#"{
            "matcher": "Write",
            "hooks": [{ "type": "command", "command": "echo ok" }],
            "group_metadata": { "x": 1 }
        }"#;

        let group: HookMatcherGroup = serde_json::from_str(json).unwrap();
        assert_eq!(group.matcher.as_deref(), Some("Write"));
        assert!(group.other.contains_key("group_metadata"));
    }

    #[test]
    fn test_hooks_canonical_object_deserializes() {
        let json = r#"{
            "env": {},
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "Bash",
                        "hooks": [
                            {
                                "type": "command",
                                "command": "./security-check.sh"
                            }
                        ]
                    }
                ],
                "UserPromptSubmit": [
                    {
                        "hooks": [
                            {
                                "type": "prompt",
                                "prompt": "validate"
                            }
                        ]
                    }
                ]
            }
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.hooks.len(), 2);
        assert_eq!(
            settings.hooks["PreToolUse"][0].hooks[0].command.as_deref(),
            Some("./security-check.sh")
        );
        assert_eq!(
            settings.hooks["UserPromptSubmit"][0].hooks[0]
                .prompt
                .as_deref(),
            Some("validate")
        );
    }

    #[test]
    fn test_hooks_legacy_array_is_normalized() {
        let json = r#"{
            "env": {},
            "hooks": [
                {
                    "event": "Stop",
                    "command": "echo stop",
                    "enabled": false,
                    "description": "legacy stop hook"
                }
            ]
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.hooks.len(), 1);
        let stop_group = &settings.hooks["Stop"][0];
        assert_eq!(stop_group.matcher, None);
        assert_eq!(stop_group.hooks[0].handler_type, "command");
        assert_eq!(stop_group.hooks[0].command.as_deref(), Some("echo stop"));
        assert_eq!(stop_group.hooks[0].other["enabled"], Value::Bool(false));
        assert_eq!(
            stop_group.hooks[0].other["description"],
            Value::String("legacy stop hook".to_string())
        );
    }

    #[test]
    fn test_mcp_servers_camel_case_serialization() {
        let mut settings = ClaudeSettings::default();
        settings.mcp_servers.insert(
            "test-server".to_string(),
            McpServer {
                command: Some("node".to_string()),
                url: None,
                args: vec!["server.js".to_string()],
                env: None,
                disabled: false,
                other: HashMap::new(),
            },
        );

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("mcpServers"));
        assert!(!json.contains("mcp_servers"));
    }

    #[test]
    fn test_slash_commands_camel_case_serialization() {
        let mut settings = ClaudeSettings::default();
        settings.slash_commands.push(SlashCommand {
            name: "test".to_string(),
            description: "Test".to_string(),
            command: "echo".to_string(),
            args: None,
            disabled: false,
            other: HashMap::new(),
        });

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("slashCommands"));
        assert!(!json.contains("slash_commands"));
    }

    #[test]
    fn test_default_values() {
        let json = r#"{"command": "test", "args": []}"#;
        let server: McpServer = serde_json::from_str(json).unwrap();
        assert!(!server.disabled);

        let json = r#"{"id": "1", "name": "Test", "version": "1.0"}"#;
        let plugin: Plugin = serde_json::from_str(json).unwrap();
        assert!(plugin.enabled);

        let json = r#"{"type": "command", "command": "echo"}"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        assert_eq!(hook.handler_type, "command");
        assert_eq!(hook.command.as_deref(), Some("echo"));
    }

    #[test]
    fn test_skip_serializing_if_empty() {
        let settings = ClaudeSettings::default();
        let json = serde_json::to_string(&settings).unwrap();

        assert!(!json.contains("mcpServers"));
        assert!(!json.contains("slashCommands"));
        assert!(!json.contains("agents"));
        assert!(!json.contains("plugins"));
        assert!(!json.contains("hooks"));
    }

    #[test]
    fn test_hooks_missing_defaults_to_empty() {
        let json = r#"{"env": {}}"#;
        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();
        assert!(settings.hooks.is_empty());
    }

    // ═══ 变更/查询/验证逻辑（迁自 ccr-cli managers/settings.rs）═══

    #[test]
    fn test_clear_anthropic_vars() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "test".into());
        settings.env.insert("OTHER_VAR".into(), "keep".into());

        settings.clear_anthropic_vars();

        assert!(!settings.env.contains_key("ANTHROPIC_BASE_URL"));
        assert!(settings.env.contains_key("OTHER_VAR"));
    }

    #[test]
    fn test_clear_managed_vars_drops_claude_code_keys() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "test".into());
        settings
            .env
            .insert("CLAUDE_CODE_SUBAGENT_MODEL".into(), "x".into());
        settings
            .env
            .insert("CLAUDE_CODE_EFFORT_LEVEL".into(), "max".into());
        settings
            .env
            .insert("CLAUDE_CODE_AUTO_COMPACT_WINDOW".into(), "1000000".into());
        settings
            .env
            .insert("API_TIMEOUT_MS".into(), "3000000".into());
        settings.env.insert(
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".into(),
            "1".into(),
        );
        settings.env.insert("OTHER_VAR".into(), "keep".into());

        settings.clear_managed_vars();

        for key in env_keys::NON_ANTHROPIC_MANAGED_KEYS {
            assert!(!settings.env.contains_key(*key), "{key} 应被清除");
        }
        assert!(!settings.env.contains_key("ANTHROPIC_BASE_URL"));
        assert!(settings.env.contains_key("OTHER_VAR"));
    }

    #[test]
    fn test_apply_managed_env_replaces_managed_and_keeps_others() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "https://old.example.com".into());
        settings.env.insert("KEEP_ME".into(), "value".into());

        settings.apply_managed_env([
            (
                "ANTHROPIC_BASE_URL".to_string(),
                "https://new.example.com".to_string(),
            ),
            ("ANTHROPIC_AUTH_TOKEN".to_string(), "sk-new".to_string()),
        ]);

        assert_eq!(
            settings.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://new.example.com".to_string())
        );
        assert_eq!(
            settings.env.get("ANTHROPIC_AUTH_TOKEN"),
            Some(&"sk-new".to_string())
        );
        assert_eq!(settings.env.get("KEEP_ME"), Some(&"value".to_string()));
    }

    // 🔁 二次 apply 时上一档的托管键不残留（防串档）
    #[test]
    fn test_apply_managed_env_clears_stale_keys_on_switch() {
        let mut settings = ClaudeSettings::new();
        settings.apply_managed_env([
            (
                "ANTHROPIC_DEFAULT_FABLE_MODEL".to_string(),
                "glm-5.2[1m]".to_string(),
            ),
            ("CLAUDE_CODE_EFFORT_LEVEL".to_string(), "max".to_string()),
        ]);

        settings.apply_managed_env([(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://api.test.com".to_string(),
        )]);

        assert!(!settings.env.contains_key("ANTHROPIC_DEFAULT_FABLE_MODEL"));
        assert!(!settings.env.contains_key("CLAUDE_CODE_EFFORT_LEVEL"));
        assert!(settings.env.contains_key("ANTHROPIC_BASE_URL"));
    }

    #[test]
    fn test_anthropic_env_status_reports_known_keys() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "https://api.test.com".into());

        let status = settings.anthropic_env_status();

        assert_eq!(
            status.get("ANTHROPIC_BASE_URL"),
            Some(&Some("https://api.test.com".to_string()))
        );
        assert_eq!(status.get("ANTHROPIC_AUTH_TOKEN"), Some(&None));
        assert!(status.contains_key("CLAUDE_CODE_AUTO_COMPACT_WINDOW"));
    }

    #[test]
    fn test_validate_subscription_mode_allows_no_overrides() {
        let settings = ClaudeSettings::new();
        assert!(settings.validate().is_ok());
        assert!(!settings.has_anthropic_overrides());
    }

    #[test]
    fn test_validate_full_api_key_mode_passes() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "https://test.com".into());
        settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".into(), "token".into());

        assert!(settings.has_anthropic_overrides());
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_partial_overrides() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "https://test.com".into());

        assert!(settings.validate().is_err());
        let err = settings.validate_api_key_mode().unwrap_err();
        assert!(err.contains("ANTHROPIC_AUTH_TOKEN"));
    }

    #[test]
    fn test_validate_api_key_mode_rejects_bad_base_url() {
        let mut settings = ClaudeSettings::new();
        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "ftp://test.com".into());
        settings
            .env
            .insert("ANTHROPIC_AUTH_TOKEN".into(), "token".into());

        let err = settings.validate_api_key_mode().unwrap_err();
        assert!(err.contains("http://"));

        settings
            .env
            .insert("ANTHROPIC_BASE_URL".into(), "  ".into());
        let err = settings.validate_api_key_mode().unwrap_err();
        assert!(err.contains("不能为空"));
    }

    // ═══ 读→改→写→读 往返保留（本类型作为唯一 shape 的核心回归防线）═══

    #[test]
    fn test_roundtrip_preserves_rich_and_unknown_fields_across_apply() {
        let disk_json = r#"{
            "env": {
                "ANTHROPIC_BASE_URL": "https://old.example.com",
                "MY_CUSTOM_VAR": "keep-me"
            },
            "outputStyle": "engineer",
            "mcpServers": {
                "fs": { "command": "node", "args": ["fs.js"], "vendor_flag": true }
            },
            "hooks": {
                "PreToolUse": [
                    { "matcher": "Bash", "hooks": [{ "type": "command", "command": "./sec.sh" }] }
                ]
            },
            "plugins": [
                { "id": "p1", "name": "Plugin", "version": "1.0", "future_field": "x" }
            ],
            "statusline": { "theme": "warm" },
            "totally_unknown_top_level": [1, 2, 3]
        }"#;

        let mut settings: ClaudeSettings = serde_json::from_str(disk_json).unwrap();
        settings.apply_managed_env([(
            "ANTHROPIC_BASE_URL".to_string(),
            "https://new.example.com".to_string(),
        )]);

        let written = serde_json::to_string_pretty(&settings).unwrap();
        let reloaded: ClaudeSettings = serde_json::from_str(&written).unwrap();

        // 托管 env 已替换，非托管 env 保留
        assert_eq!(
            reloaded.env.get("ANTHROPIC_BASE_URL"),
            Some(&"https://new.example.com".to_string())
        );
        assert_eq!(
            reloaded.env.get("MY_CUSTOM_VAR"),
            Some(&"keep-me".to_string())
        );

        // 富字段保留（含嵌套未知字段）
        assert_eq!(reloaded.output_style.as_deref(), Some("engineer"));
        assert_eq!(
            reloaded.mcp_servers["fs"].other.get("vendor_flag"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            reloaded.hooks["PreToolUse"][0].hooks[0].command.as_deref(),
            Some("./sec.sh")
        );
        assert_eq!(
            reloaded.plugins[0].other.get("future_field"),
            Some(&Value::String("x".to_string()))
        );

        // 顶层未知字段保留
        assert!(reloaded.other.contains_key("statusline"));
        assert!(reloaded.other.contains_key("totally_unknown_top_level"));
    }

    // legacy 数组 hooks 在往返后归一化为 canonical object 格式（有意的规范化行为）
    #[test]
    fn test_roundtrip_normalizes_legacy_hooks_to_canonical() {
        let disk_json = r#"{
            "env": {},
            "hooks": [
                { "event": "Stop", "command": "echo stop" }
            ]
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(disk_json).unwrap();
        let written = serde_json::to_string(&settings).unwrap();

        let value: Value = serde_json::from_str(&written).unwrap();
        assert!(
            value["hooks"].is_object(),
            "legacy 数组必须归一化为 object 格式"
        );
        assert_eq!(value["hooks"]["Stop"][0]["hooks"][0]["command"], "echo stop");
    }

    #[test]
    fn test_hooks_invalid_type_rejected() {
        let string_json = r#"{"env": {}, "hooks": "run"}"#;
        let string_err = serde_json::from_str::<ClaudeSettings>(string_json).unwrap_err();
        assert!(
            string_err
                .to_string()
                .contains("invalid type for hooks: expected object or array, got string")
        );

        let number_json = r#"{"env": {}, "hooks": 1}"#;
        let number_err = serde_json::from_str::<ClaudeSettings>(number_json).unwrap_err();
        assert!(
            number_err
                .to_string()
                .contains("invalid type for hooks: expected object or array, got number")
        );

        let bool_json = r#"{"env": {}, "hooks": true}"#;
        let bool_err = serde_json::from_str::<ClaudeSettings>(bool_json).unwrap_err();
        assert!(
            bool_err
                .to_string()
                .contains("invalid type for hooks: expected object or array, got boolean")
        );
    }
}
