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
