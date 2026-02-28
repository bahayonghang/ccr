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

/// Deserialize hooks: accept both array and object (treat object as empty array)
fn deserialize_hooks<'de, D>(deserializer: D) -> Result<Vec<Hook>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Array(_) => serde_json::from_value(value).map_err(serde::de::Error::custom),
        _ => Ok(Vec::new()),
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

    /// Output style
    #[serde(skip_serializing_if = "Option::is_none")]
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
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_hooks"
    )]
    pub hooks: Vec<Hook>,

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

/// Hook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    /// Event to hook
    pub event: String,
    /// Command to execute
    pub command: String,
    /// Whether the hook is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Hook description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Other unknown fields (for forward compatibility)
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
            hooks: Vec::new(),
            other: HashMap::new(),
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        let parsed: ClaudeSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(
            parsed.env.get("ANTHROPIC_BASE_URL").unwrap(),
            "https://api.anthropic.com"
        );
        assert_eq!(parsed.output_style, Some("nekomata-engineer".to_string()));
    }

    #[test]
    fn test_claude_settings_unknown_fields_preserved() {
        // JSON with unknown fields at root level
        let json = r#"{
            "env": {},
            "output_style": "test",
            "future_field": "should be preserved",
            "another_unknown": 42
        }"#;

        let settings: ClaudeSettings = serde_json::from_str(json).unwrap();

        // Verify unknown fields are captured
        assert!(settings.other.contains_key("future_field"));
        assert_eq!(
            settings.other.get("future_field").unwrap(),
            "should be preserved"
        );
        assert_eq!(settings.other.get("another_unknown").unwrap(), 42);

        // Roundtrip should preserve unknown fields
        let serialized = serde_json::to_string(&settings).unwrap();
        assert!(serialized.contains("future_field"));
        assert!(serialized.contains("another_unknown"));
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
    fn test_agent_unknown_fields_preserved() {
        let json = r#"{
            "name": "test-agent",
            "model": "claude-3",
            "tools": [],
            "experimental_feature": "value"
        }"#;

        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.name, "test-agent");
        assert!(agent.other.contains_key("experimental_feature"));
    }

    #[test]
    fn test_plugin_unknown_fields_preserved() {
        let json = r#"{
            "id": "plugin-1",
            "name": "Test Plugin",
            "version": "1.0.0",
            "plugin_metadata": {"key": "value"}
        }"#;

        let plugin: Plugin = serde_json::from_str(json).unwrap();
        assert_eq!(plugin.id, "plugin-1");
        assert!(plugin.other.contains_key("plugin_metadata"));
    }

    #[test]
    fn test_hook_unknown_fields_preserved() {
        let json = r#"{
            "event": "pre-commit",
            "command": "lint",
            "hook_priority": 10
        }"#;

        let hook: Hook = serde_json::from_str(json).unwrap();
        assert_eq!(hook.event, "pre-commit");
        assert!(hook.other.contains_key("hook_priority"));
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
        // Should use camelCase "mcpServers" not snake_case "mcp_servers"
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
        // Should use camelCase "slashCommands" not snake_case "slash_commands"
        assert!(json.contains("slashCommands"));
        assert!(!json.contains("slash_commands"));
    }

    #[test]
    fn test_default_values() {
        // Test that disabled defaults to false
        let json = r#"{"command": "test", "args": []}"#;
        let server: McpServer = serde_json::from_str(json).unwrap();
        assert!(!server.disabled);

        // Test that enabled defaults to true for Plugin
        let json = r#"{"id": "1", "name": "Test", "version": "1.0"}"#;
        let plugin: Plugin = serde_json::from_str(json).unwrap();
        assert!(plugin.enabled);

        // Test that enabled defaults to true for Hook
        let json = r#"{"event": "test", "command": "echo"}"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        assert!(hook.enabled);
    }

    #[test]
    fn test_skip_serializing_if_empty() {
        let settings = ClaudeSettings::default();
        let json = serde_json::to_string(&settings).unwrap();

        // Empty collections should not be serialized
        assert!(!json.contains("mcpServers"));
        assert!(!json.contains("slashCommands"));
        assert!(!json.contains("agents"));
        assert!(!json.contains("plugins"));
        assert!(!json.contains("hooks"));
    }
}
