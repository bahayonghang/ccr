//! Local MCP entries projected from configuration files the desktop already reads.
//! This is not a new writer: add, remove, install, and process status stay unread.
//! Environment values, headers, arguments, URLs, and tokens are not copied out.

use super::{ConfigSource, Services};
use ccr_app::{
    platform_settings::{self},
    runtime::{AdmissionError, EnvironmentId, RequestKey, TaskSubscription},
};
use ccr_cli::application::platform_roots::PlatformRoots;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) const FIXTURE_DIR: &str = "desktop-mcp-fixture";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum McpSource {
    ClaudeState,
    ClaudeSettings,
    CodexConfig,
    AntigravityConfig,
    ProjectFile,
    OpenCode,
    Grok,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum McpSourceState {
    Missing,
    Empty,
    Entries(usize),
    Invalid,
    Unreadable,
    NotRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum McpScope {
    User,
    Project(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpEntry {
    pub source: McpSource,
    pub name: String,
    pub scope: McpScope,
    pub command: Option<String>,
    pub has_url: bool,
    pub has_args: bool,
    pub disabled: Option<bool>,
    pub enabled: Option<bool>,
}

impl McpSource {
    fn code(self) -> &'static str {
        match self {
            Self::ClaudeState => "claude-state",
            Self::ClaudeSettings => "claude-settings",
            Self::CodexConfig => "codex-config",
            Self::AntigravityConfig => "antigravity-config",
            Self::ProjectFile => "project-file",
            Self::OpenCode => "opencode",
            Self::Grok => "grok",
        }
    }
}

impl McpEntry {
    pub fn id(&self) -> String {
        let scope = match &self.scope {
            McpScope::User => "user".to_string(),
            McpScope::Project(key) => format!("project:{key}"),
        };
        format!("{}:{scope}:{}", self.source.code(), self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpSourceNote {
    pub source: McpSource,
    pub state: McpSourceState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpCatalog {
    pub entries: Vec<McpEntry>,
    pub notes: Vec<McpSourceNote>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum McpError {
    Unsupported,
    Storage,
}

impl McpError {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unsupported => {
                "MCP files are read only for the local environment. / 只在本地环境读取 MCP 文件"
            }
            Self::Storage => "Cannot resolve local configuration roots. / 无法定位本地配置根目录",
        }
    }
}

struct Bits {
    command: Option<String>,
    has_url: bool,
    has_args: bool,
    disabled: Option<bool>,
    enabled: Option<bool>,
}

enum ParseError {
    Invalid,
}

fn roots(source: &ConfigSource) -> Result<PlatformRoots, McpError> {
    match source {
        ConfigSource::Local => PlatformRoots::capture().map_err(|_| McpError::Storage),
        ConfigSource::Fixture {
            profiles, registry, ..
        } => {
            if !profiles.is_absolute() || !registry.is_absolute() {
                return Err(McpError::Storage);
            }
            let root = profiles
                .parent()
                .ok_or(McpError::Storage)?
                .join(FIXTURE_DIR);
            PlatformRoots::fixture(&root).map_err(|_| McpError::Storage)
        }
    }
}

fn load_entries(
    path: &Path,
    parse: impl FnOnce(&str) -> Result<Vec<McpEntry>, ParseError>,
) -> (Vec<McpEntry>, McpSourceState) {
    match platform_settings::read_text(path) {
        Ok(snapshot) if !snapshot.exists => (Vec::new(), McpSourceState::Missing),
        Ok(snapshot) => match parse(&snapshot.content) {
            Ok(entries) => {
                let state = if entries.is_empty() {
                    McpSourceState::Empty
                } else {
                    McpSourceState::Entries(entries.len())
                };
                (entries, state)
            }
            Err(ParseError::Invalid) => (Vec::new(), McpSourceState::Invalid),
        },
        Err(_) => (Vec::new(), McpSourceState::Unreadable),
    }
}

fn json_object(content: &str) -> Result<serde_json::Map<String, Value>, ParseError> {
    let value: Value = serde_json::from_str(content).map_err(|_| ParseError::Invalid)?;
    value.as_object().cloned().ok_or(ParseError::Invalid)
}

fn nonempty(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

fn json_bits(value: &Value) -> Result<Bits, ParseError> {
    let object = value.as_object().ok_or(ParseError::Invalid)?;
    let command = match object.get("command") {
        None | Some(Value::Null) => None,
        Some(Value::String(text)) => nonempty(text),
        Some(_) => return Err(ParseError::Invalid),
    };
    let mut has_url = false;
    for key in ["url", "serverUrl", "httpUrl"] {
        match object.get(key) {
            None | Some(Value::Null) => {}
            Some(Value::String(text)) => {
                if nonempty(text).is_some() {
                    has_url = true;
                }
            }
            Some(_) => return Err(ParseError::Invalid),
        }
    }
    let has_args = match object.get("args") {
        None | Some(Value::Null) => false,
        Some(Value::Array(items)) => !items.is_empty(),
        Some(_) => return Err(ParseError::Invalid),
    };
    Ok(Bits {
        command,
        has_url,
        has_args,
        disabled: optional_json_bool(object, "disabled")?,
        enabled: optional_json_bool(object, "enabled")?,
    })
}

fn optional_json_bool(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<bool>, ParseError> {
    match object.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(ParseError::Invalid),
    }
}

fn json_servers(
    servers: &serde_json::Map<String, Value>,
    source: McpSource,
    scope: &McpScope,
) -> Result<Vec<McpEntry>, ParseError> {
    let mut entries = Vec::new();
    for (name, value) in servers {
        if name.is_empty() {
            return Err(ParseError::Invalid);
        }
        let bits = json_bits(value)?;
        entries.push(McpEntry {
            source,
            name: name.clone(),
            scope: scope.clone(),
            command: bits.command,
            has_url: bits.has_url,
            has_args: bits.has_args,
            disabled: bits.disabled,
            enabled: bits.enabled,
        });
    }
    Ok(entries)
}

fn top_level_json(content: &str, source: McpSource) -> Result<Vec<McpEntry>, ParseError> {
    let root = json_object(content)?;
    match root.get("mcpServers") {
        None => Ok(Vec::new()),
        Some(Value::Object(servers)) => json_servers(servers, source, &McpScope::User),
        Some(_) => Err(ParseError::Invalid),
    }
}

fn claude_state(content: &str) -> Result<Vec<McpEntry>, ParseError> {
    let root = json_object(content)?;
    let mut entries = match root.get("mcpServers") {
        None => Vec::new(),
        Some(Value::Object(servers)) => {
            json_servers(servers, McpSource::ClaudeState, &McpScope::User)?
        }
        Some(_) => return Err(ParseError::Invalid),
    };
    if let Some(projects) = root.get("projects") {
        let projects = projects.as_object().ok_or(ParseError::Invalid)?;
        for (key, project) in projects {
            if key.is_empty() {
                return Err(ParseError::Invalid);
            }
            let Some(project) = project.as_object() else {
                continue;
            };
            if let Some(servers) = project.get("mcpServers") {
                let servers = servers.as_object().ok_or(ParseError::Invalid)?;
                entries.extend(json_servers(
                    servers,
                    McpSource::ClaudeState,
                    &McpScope::Project(key.clone()),
                )?);
            }
        }
    }
    Ok(entries)
}

fn toml_bits(value: &toml::Value) -> Result<Bits, ParseError> {
    let toml::Value::Table(table) = value else {
        return Err(ParseError::Invalid);
    };
    let command = match table.get("command") {
        None => None,
        Some(toml::Value::String(text)) => nonempty(text),
        Some(_) => return Err(ParseError::Invalid),
    };
    let has_url = match table.get("url") {
        None => false,
        Some(toml::Value::String(text)) => nonempty(text).is_some(),
        Some(_) => return Err(ParseError::Invalid),
    };
    let has_args = match table.get("args") {
        None => false,
        Some(toml::Value::Array(items)) => !items.is_empty(),
        Some(_) => return Err(ParseError::Invalid),
    };
    Ok(Bits {
        command,
        has_url,
        has_args,
        disabled: optional_toml_bool(table, "disabled")?,
        enabled: optional_toml_bool(table, "enabled")?,
    })
}

fn optional_toml_bool(
    table: &toml::map::Map<String, toml::Value>,
    key: &str,
) -> Result<Option<bool>, ParseError> {
    match table.get(key) {
        None => Ok(None),
        Some(toml::Value::Boolean(value)) => Ok(Some(*value)),
        Some(_) => Err(ParseError::Invalid),
    }
}

fn codex_config(content: &str) -> Result<Vec<McpEntry>, ParseError> {
    let toml::Value::Table(root) = toml::from_str(content).map_err(|_| ParseError::Invalid)? else {
        return Err(ParseError::Invalid);
    };
    let Some(servers) = root.get("mcp_servers") else {
        return Ok(Vec::new());
    };
    let toml::Value::Table(servers) = servers else {
        return Err(ParseError::Invalid);
    };
    let mut entries = Vec::new();
    for (name, value) in servers {
        if name.is_empty() {
            return Err(ParseError::Invalid);
        }
        let bits = toml_bits(value)?;
        entries.push(McpEntry {
            source: McpSource::CodexConfig,
            name: name.clone(),
            scope: McpScope::User,
            command: bits.command,
            has_url: bits.has_url,
            has_args: bits.has_args,
            disabled: bits.disabled,
            enabled: bits.enabled,
        });
    }
    Ok(entries)
}

fn note(source: McpSource, state: McpSourceState) -> McpSourceNote {
    McpSourceNote { source, state }
}

fn push_file(
    entries: &mut Vec<McpEntry>,
    notes: &mut Vec<McpSourceNote>,
    source: McpSource,
    path: PathBuf,
    parse: impl FnOnce(&str) -> Result<Vec<McpEntry>, ParseError>,
) {
    let (parsed, state) = load_entries(&path, parse);
    entries.extend(parsed);
    notes.push(note(source, state));
}

pub(crate) fn read_catalog(
    source: &ConfigSource,
    environment: &EnvironmentId,
) -> Result<McpCatalog, McpError> {
    if !matches!(environment, EnvironmentId::Local(_)) {
        return Err(McpError::Unsupported);
    }
    let roots = roots(source)?;
    let mut entries = Vec::new();
    let mut notes = Vec::new();
    push_file(
        &mut entries,
        &mut notes,
        McpSource::ClaudeState,
        roots.claude.state_file.clone(),
        claude_state,
    );
    push_file(
        &mut entries,
        &mut notes,
        McpSource::ClaudeSettings,
        roots.claude.settings_file.clone(),
        |content| top_level_json(content, McpSource::ClaudeSettings),
    );
    push_file(
        &mut entries,
        &mut notes,
        McpSource::CodexConfig,
        roots.codex_runtime.join("config.toml"),
        codex_config,
    );
    push_file(
        &mut entries,
        &mut notes,
        McpSource::AntigravityConfig,
        roots
            .home
            .join(".gemini")
            .join("antigravity-cli")
            .join("mcp_config.json"),
        |content| top_level_json(content, McpSource::AntigravityConfig),
    );
    notes.push(note(McpSource::ProjectFile, McpSourceState::NotRead));
    notes.push(note(McpSource::OpenCode, McpSourceState::NotRead));
    notes.push(note(McpSource::Grok, McpSourceState::NotRead));
    entries.sort_by(|left, right| {
        left.source
            .cmp(&right.source)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| scope_sort(&left.scope).cmp(scope_sort(&right.scope)))
    });
    Ok(McpCatalog { entries, notes })
}

fn scope_sort(scope: &McpScope) -> &str {
    match scope {
        McpScope::User => "",
        McpScope::Project(key) => key,
    }
}

impl Services {
    pub fn mcp_request(
        &self,
        key: RequestKey,
    ) -> Result<TaskSubscription<McpCatalog, McpError>, AdmissionError> {
        let source = self.source.clone();
        let environment = key.environment.clone();
        // The catalog covers every readable file. The key platform is only the request slot.
        self.executor
            .read(key, move |_, _| read_catalog(&source, &environment))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::Duration;

    fn fixture(root: &Path) -> ConfigSource {
        ConfigSource::Fixture {
            profiles: root.join("profiles.toml"),
            registry: root.join("registry.toml"),
            delay: Duration::ZERO,
        }
    }

    fn home(root: &Path) -> PathBuf {
        root.join(FIXTURE_DIR).join("home")
    }

    #[test]
    fn missing_files_do_not_create_directories_or_servers() {
        let root = tempfile::tempdir().unwrap();
        let catalog = read_catalog(
            &fixture(root.path()),
            &EnvironmentId::Local("fixture".into()),
        )
        .unwrap();
        assert!(catalog.entries.is_empty());
        assert!(!root.path().join(FIXTURE_DIR).exists());
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::ClaudeState && note.state == McpSourceState::Missing
        }));
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::CodexConfig && note.state == McpSourceState::Missing
        }));
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::AntigravityConfig && note.state == McpSourceState::Missing
        }));
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::ProjectFile && note.state == McpSourceState::NotRead
        }));
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::OpenCode && note.state == McpSourceState::NotRead
        }));
        assert!(
            catalog
                .notes
                .iter()
                .any(|note| note.source == McpSource::Grok && note.state == McpSourceState::NotRead)
        );
    }

    #[test]
    fn remote_environment_does_not_read_local_files() {
        let root = tempfile::tempdir().unwrap();
        let error = read_catalog(&fixture(root.path()), &EnvironmentId::Ssh("fixture".into()));
        assert!(matches!(error, Err(McpError::Unsupported)));
        assert!(!root.path().join(FIXTURE_DIR).exists());
    }

    #[test]
    fn real_entries_keep_flags_and_drop_secret_fields() {
        let root = tempfile::tempdir().unwrap();
        let home = home(root.path());
        let claude = home.join(".claude");
        let agy = home.join(".gemini").join("antigravity-cli");
        let codex = home.join(".codex");
        fs::create_dir_all(&claude).unwrap();
        fs::create_dir_all(&agy).unwrap();
        fs::create_dir_all(&codex).unwrap();
        fs::write(
            home.join(".claude.json"),
            r#"{"oauth":"mcp-secret-sentinel","mcpServers":{"state-server":{"command":"fixture-mcp-command","args":["mcp-secret-sentinel"],"env":{"TOKEN":"mcp-secret-sentinel"}}},"projects":{"fixture-project":{"mcpServers":{"project-server":{"url":"https://example.test/?token=mcp-secret-sentinel"}}},"ignored":"skip"}}"#,
        )
        .unwrap();
        fs::write(
            claude.join("settings.json"),
            r#"{"apiKey":"mcp-secret-sentinel","mcpServers":{"settings-server":{"command":"fixture-mcp-command","disabled":true,"env":{"X":"mcp-secret-sentinel"}}}}"#,
        )
        .unwrap();
        fs::write(
            codex.join("config.toml"),
            "model = 'kept'\n[mcp_servers.codex-server]\ncommand = 'fixture-mcp-command'\nenabled = false\nbearer_token = 'mcp-secret-sentinel'\n",
        )
        .unwrap();
        fs::write(
            agy.join("mcp_config.json"),
            r#"{"mcpServers":{"agy-server":{"command":"agy","headers":{"Authorization":"mcp-secret-sentinel"}}}}"#,
        )
        .unwrap();
        let catalog = read_catalog(
            &fixture(root.path()),
            &EnvironmentId::Local("fixture".into()),
        )
        .unwrap();
        let rendered = format!("{catalog:?}");
        assert!(!rendered.contains("mcp-secret-sentinel"));
        assert_eq!(
            catalog
                .entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec![
                "project-server",
                "state-server",
                "settings-server",
                "codex-server",
                "agy-server"
            ]
        );
        let state = catalog
            .entries
            .iter()
            .find(|entry| entry.name == "state-server")
            .unwrap();
        assert_eq!(state.command.as_deref(), Some("fixture-mcp-command"));
        assert!(state.has_args);
        assert!(!state.has_url);
        assert_eq!(state.disabled, None);
        let project = catalog
            .entries
            .iter()
            .find(|entry| entry.name == "project-server")
            .unwrap();
        assert!(project.has_url);
        assert_eq!(project.command, None);
        assert_eq!(project.scope, McpScope::Project("fixture-project".into()));
        let settings = catalog
            .entries
            .iter()
            .find(|entry| entry.name == "settings-server")
            .unwrap();
        assert_eq!(settings.disabled, Some(true));
        let codex = catalog
            .entries
            .iter()
            .find(|entry| entry.name == "codex-server")
            .unwrap();
        assert_eq!(codex.enabled, Some(false));
        assert_eq!(codex.command.as_deref(), Some("fixture-mcp-command"));
    }

    #[test]
    fn present_files_without_servers_are_empty_not_a_zero_count() {
        let root = tempfile::tempdir().unwrap();
        let home = home(root.path());
        let claude = home.join(".claude");
        let agy = home.join(".gemini").join("antigravity-cli");
        fs::create_dir_all(&claude).unwrap();
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::create_dir_all(&agy).unwrap();
        fs::write(
            home.join(".claude.json"),
            "{\"oauth\":\"mcp-secret-sentinel\"}",
        )
        .unwrap();
        fs::write(claude.join("settings.json"), "{}").unwrap();
        fs::write(home.join(".codex").join("config.toml"), "model = 'kept'\n").unwrap();
        fs::write(agy.join("mcp_config.json"), "{\"mcpServers\":{}}").unwrap();
        let catalog = read_catalog(
            &fixture(root.path()),
            &EnvironmentId::Local("fixture".into()),
        )
        .unwrap();
        assert!(catalog.entries.is_empty());
        assert!(!format!("{catalog:?}").contains("mcp-secret-sentinel"));
        for source in [
            McpSource::ClaudeState,
            McpSource::ClaudeSettings,
            McpSource::CodexConfig,
            McpSource::AntigravityConfig,
        ] {
            assert!(
                catalog
                    .notes
                    .iter()
                    .any(|note| note.source == source && note.state == McpSourceState::Empty),
                "{source:?} notes={:?}",
                catalog.notes
            );
        }
        assert!(
            catalog
                .notes
                .iter()
                .all(|note| !matches!(note.state, McpSourceState::Entries(0)))
        );
    }

    #[test]
    fn wrong_server_shape_drops_that_file_without_leaking_values() {
        let root = tempfile::tempdir().unwrap();
        let home = home(root.path());
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::write(
            home.join(".claude.json"),
            r#"{"mcpServers":[{"command":"fixture-mcp-command","env":{"TOKEN":"mcp-secret-sentinel"}}]}"#,
        )
        .unwrap();
        fs::write(
            home.join(".codex").join("config.toml"),
            "mcp_servers = [{ command = 'fixture-mcp-command', bearer_token = 'mcp-secret-sentinel' }]\n",
        )
        .unwrap();
        let catalog = read_catalog(
            &fixture(root.path()),
            &EnvironmentId::Local("fixture".into()),
        )
        .unwrap();
        assert!(catalog.entries.is_empty());
        assert!(!format!("{catalog:?}").contains("mcp-secret-sentinel"));
        assert!(!format!("{catalog:?}").contains("fixture-mcp-command"));
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::ClaudeState && note.state == McpSourceState::Invalid
        }));
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::CodexConfig && note.state == McpSourceState::Invalid
        }));
        assert!(
            catalog
                .notes
                .iter()
                .all(|note| !matches!(note.state, McpSourceState::Entries(0)))
        );
    }

    #[test]
    fn invalid_claude_state_does_not_drop_other_files() {
        let root = tempfile::tempdir().unwrap();
        let home = home(root.path());
        fs::create_dir_all(home.join(".codex")).unwrap();
        fs::write(home.join(".claude.json"), "{").unwrap();
        fs::write(
            home.join(".codex").join("config.toml"),
            "[mcp_servers.codex-server]\ncommand = 'fixture-mcp-command'\n",
        )
        .unwrap();
        let catalog = read_catalog(
            &fixture(root.path()),
            &EnvironmentId::Local("fixture".into()),
        )
        .unwrap();
        assert!(catalog.notes.iter().any(|note| {
            note.source == McpSource::ClaudeState && note.state == McpSourceState::Invalid
        }));
        assert_eq!(catalog.entries.len(), 1);
        assert_eq!(catalog.entries[0].name, "codex-server");
    }
}
