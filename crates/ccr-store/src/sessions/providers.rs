//! Read-only local Agent session providers used by the desktop session browser.
//!
//! This registry deliberately does not reuse `ccr_config::Platform`: session
//! sources have different identities and lifecycles than configurable CLIs.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_DISCOVERY_DEPTH: usize = 8;
#[cfg(test)]
const DEFAULT_MESSAGE_LIMIT: usize = 100;
const MAX_MESSAGE_LIMIT: usize = 200;
const MAX_MESSAGE_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentSessionAgentId {
    Grok,
    Claude,
    Codex,
    OpenCode,
    Pi,
    Omp,
    Antigravity,
    Kimi,
}

impl AgentSessionAgentId {
    pub const ALL: [Self; 8] = [
        Self::Grok,
        Self::Claude,
        Self::Codex,
        Self::OpenCode,
        Self::Pi,
        Self::Omp,
        Self::Antigravity,
        Self::Kimi,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Grok => "grok",
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
            Self::Pi => "pi",
            Self::Omp => "omp",
            Self::Antigravity => "antigravity",
            Self::Kimi => "kimi",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|agent| agent.as_str() == value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSessionSourceKind {
    File,
    Bundle,
    SqliteMember,
}

impl AgentSessionSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Bundle => "bundle",
            Self::SqliteMember => "sqlite_member",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "file" => Some(Self::File),
            "bundle" => Some(Self::Bundle),
            "sqlite_member" => Some(Self::SqliteMember),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSessionFidelity {
    Full,
    Partial,
    Locked,
}

impl AgentSessionFidelity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Partial => "partial",
            Self::Locked => "locked",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailability {
    NotInstalled,
    NoData,
    Available,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentSessionProviderDefinition {
    pub agent: AgentSessionAgentId,
    pub label: &'static str,
    pub variants: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSessionSourceRef {
    pub agent: AgentSessionAgentId,
    pub variant: String,
    pub root: PathBuf,
    pub physical_path: PathBuf,
    pub kind: AgentSessionSourceKind,
    pub member_id: Option<String>,
    pub project_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionSummary {
    pub native_session_id: String,
    pub agent: AgentSessionAgentId,
    pub variant: String,
    pub title: Option<String>,
    pub cwd: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: u32,
    pub user_message_count: u32,
    pub assistant_message_count: u32,
    pub tool_use_count: u32,
    pub fidelity: AgentSessionFidelity,
    pub source: AgentSessionSourceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionMessage {
    pub key: String,
    pub ordinal: u32,
    pub role: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub content: String,
    pub tool_name: Option<String>,
    pub clipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionMessagePage {
    pub messages: Vec<AgentSessionMessage>,
    pub next_before: Option<u32>,
    pub has_older: bool,
    pub fidelity: AgentSessionFidelity,
}

struct BoundedProviderPage {
    messages: Vec<AgentSessionMessage>,
    next_before: Option<u32>,
    had_source_messages: bool,
    degraded: bool,
    fidelity: AgentSessionFidelity,
}

impl BoundedProviderPage {
    fn into_dto(self) -> AgentSessionMessagePage {
        AgentSessionMessagePage {
            has_older: self.next_before.is_some(),
            messages: self.messages,
            next_before: self.next_before,
            fidelity: self.fidelity,
        }
    }
}

struct BoundedMessageWindow {
    source_key: String,
    before: Option<u32>,
    limit: usize,
    next_ordinal: u32,
    messages: VecDeque<AgentSessionMessage>,
    had_source_messages: bool,
    degraded: bool,
}

impl BoundedMessageWindow {
    fn new(source: &AgentSessionSourceRef, before: Option<u32>, limit: usize) -> Self {
        Self {
            source_key: native_source_key(source),
            before,
            limit,
            next_ordinal: 0,
            messages: VecDeque::with_capacity(limit.saturating_add(1)),
            had_source_messages: false,
            degraded: false,
        }
    }

    fn wants_more(&self) -> bool {
        self.before.is_none_or(|end| self.next_ordinal < end)
    }

    fn push(&mut self, mut message: AgentSessionMessage) -> bool {
        self.had_source_messages = true;
        if !self.wants_more() {
            return false;
        }
        message.ordinal = self.next_ordinal;
        message.key = format!("{}:{}", self.source_key, self.next_ordinal);
        self.next_ordinal = self.next_ordinal.saturating_add(1);
        self.messages.push_back(message);
        if self.messages.len() > self.limit {
            self.messages.pop_front();
        }
        self.wants_more()
    }

    fn finish(self) -> BoundedProviderPage {
        let messages = self.messages.into_iter().collect::<Vec<_>>();
        let next_before = messages
            .first()
            .and_then(|message| (message.ordinal > 0).then_some(message.ordinal));
        BoundedProviderPage {
            messages,
            next_before,
            had_source_messages: self.had_source_messages,
            degraded: self.degraded,
            fidelity: AgentSessionFidelity::Full,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSessionProviderStatus {
    pub agent: AgentSessionAgentId,
    pub availability: ProviderAvailability,
    pub variants: Vec<String>,
    pub source_count: usize,
    pub fidelity: Option<AgentSessionFidelity>,
    pub error_category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentSessionProviderRegistry {
    home: PathBuf,
}

impl AgentSessionProviderRegistry {
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    pub fn from_default_home() -> Result<Self> {
        dirs::home_dir().map(Self::new).ok_or(CcrError::ConfigError(
            "home directory is unavailable".into(),
        ))
    }

    pub const fn definitions() -> [AgentSessionProviderDefinition; 8] {
        [
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Grok,
                label: "Grok",
                variants: &["grok-bundle"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Claude,
                label: "Claude",
                variants: &["claude-jsonl"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Codex,
                label: "Codex",
                variants: &["codex-live", "codex-archived"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::OpenCode,
                label: "OpenCode",
                variants: &["opencode-storage", "opencode-sqlite"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Pi,
                label: "Pi",
                variants: &["pi-jsonl"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Omp,
                label: "OMP",
                variants: &["omp-jsonl"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Antigravity,
                label: "Antigravity",
                variants: &["antigravity-ide", "antigravity-cli"],
            },
            AgentSessionProviderDefinition {
                agent: AgentSessionAgentId::Kimi,
                label: "Kimi",
                variants: &["kimi-legacy", "kimi-code"],
            },
        ]
    }

    pub fn provider_statuses(&self) -> Vec<AgentSessionProviderStatus> {
        Self::definitions()
            .into_iter()
            .map(|definition| {
                let roots = self.roots(definition.agent);
                let installed = roots.iter().any(|(_, root)| root.exists());
                match self.discover(definition.agent) {
                    Ok(sources) => AgentSessionProviderStatus {
                        agent: definition.agent,
                        availability: if !installed {
                            ProviderAvailability::NotInstalled
                        } else if sources.is_empty() {
                            ProviderAvailability::NoData
                        } else {
                            ProviderAvailability::Available
                        },
                        variants: definition
                            .variants
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                        source_count: sources.len(),
                        fidelity: sources
                            .iter()
                            .filter_map(|source| self.source_fidelity(source).ok())
                            .min_by_key(|value| match value {
                                AgentSessionFidelity::Locked => 0,
                                AgentSessionFidelity::Partial => 1,
                                AgentSessionFidelity::Full => 2,
                            }),
                        error_category: None,
                    },
                    Err(_) => AgentSessionProviderStatus {
                        agent: definition.agent,
                        availability: ProviderAvailability::Error,
                        variants: definition
                            .variants
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                        source_count: 0,
                        fidelity: None,
                        error_category: Some("discovery_failed".into()),
                    },
                }
            })
            .collect()
    }

    pub fn discover_all(&self) -> Vec<(AgentSessionAgentId, Result<Vec<AgentSessionSourceRef>>)> {
        AgentSessionAgentId::ALL
            .into_iter()
            .map(|agent| (agent, self.discover(agent)))
            .collect()
    }

    pub fn discover(&self, agent: AgentSessionAgentId) -> Result<Vec<AgentSessionSourceRef>> {
        self.discover_for_refresh(agent, &HashSet::new())
    }

    /// Discover sources while honoring container gates already proven unchanged.
    ///
    /// Refresh orchestration uses this to avoid opening an unchanged shared
    /// OpenCode database merely to enumerate the same member ids again.
    pub fn discover_for_refresh(
        &self,
        agent: AgentSessionAgentId,
        skipped_containers: &HashSet<PathBuf>,
    ) -> Result<Vec<AgentSessionSourceRef>> {
        let mut sources = Vec::new();
        for (variant, root) in self.roots(agent) {
            if !root.is_dir() {
                continue;
            }
            match agent {
                AgentSessionAgentId::Claude
                | AgentSessionAgentId::Pi
                | AgentSessionAgentId::Omp => {
                    for path in walk_regular_files(&root, MAX_DISCOVERY_DEPTH)? {
                        if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
                            sources.push(source(
                                agent,
                                &variant,
                                &root,
                                path,
                                AgentSessionSourceKind::File,
                            ));
                        }
                    }
                }
                AgentSessionAgentId::Codex => {
                    for path in walk_regular_files(&root, MAX_DISCOVERY_DEPTH)? {
                        if path.extension().and_then(|value| value.to_str()) == Some("jsonl") {
                            sources.push(source(
                                agent,
                                &variant,
                                &root,
                                path,
                                AgentSessionSourceKind::File,
                            ));
                        }
                    }
                }
                AgentSessionAgentId::Kimi => {
                    for path in walk_regular_files(&root, MAX_DISCOVERY_DEPTH)? {
                        if path.file_name().and_then(|value| value.to_str()) == Some("wire.jsonl") {
                            sources.push(source(
                                agent,
                                &variant,
                                &root,
                                path,
                                AgentSessionSourceKind::File,
                            ));
                        }
                    }
                }
                AgentSessionAgentId::Grok => {
                    for path in walk_regular_files(&root, 4)? {
                        if path.file_name().and_then(|value| value.to_str()) == Some("summary.json")
                            && path
                                .parent()
                                .and_then(Path::file_name)
                                .and_then(|value| value.to_str())
                                .is_some_and(valid_member_id)
                        {
                            sources.push(source(
                                agent,
                                &variant,
                                &root,
                                path,
                                AgentSessionSourceKind::Bundle,
                            ));
                        }
                    }
                }
                AgentSessionAgentId::OpenCode => {
                    if variant == "opencode-storage" {
                        let storage_root = root.join("storage").join("session");
                        if storage_root.is_dir() {
                            for path in walk_regular_files(&storage_root, 4)? {
                                if path.extension().and_then(|value| value.to_str()) == Some("json")
                                {
                                    sources.push(source(
                                        agent,
                                        &variant,
                                        &root,
                                        path,
                                        AgentSessionSourceKind::Bundle,
                                    ));
                                }
                            }
                        }
                    } else {
                        let db = root.join("opencode.db");
                        if db.is_file() && !skipped_containers.contains(&db) {
                            for member in sqlite_session_ids(&db)? {
                                let mut item = source(
                                    agent,
                                    &variant,
                                    &root,
                                    db.clone(),
                                    AgentSessionSourceKind::SqliteMember,
                                );
                                item.member_id = Some(member);
                                sources.push(item);
                            }
                        }
                    }
                }
                AgentSessionAgentId::Antigravity => {
                    for source_dir in [root.join("conversations"), root.join("implicit")] {
                        for path in walk_regular_files(&source_dir, 2)? {
                            let extension = path.extension().and_then(|value| value.to_str());
                            if matches!(extension, Some("db" | "pb")) {
                                let kind = if extension == Some("db") {
                                    AgentSessionSourceKind::Bundle
                                } else {
                                    AgentSessionSourceKind::File
                                };
                                sources.push(source(agent, &variant, &root, path, kind));
                            }
                        }
                    }
                }
            }
        }
        sources.sort_by(|left, right| {
            left.physical_path
                .cmp(&right.physical_path)
                .then(left.member_id.cmp(&right.member_id))
        });
        sources.dedup_by(|left, right| {
            left.physical_path == right.physical_path && left.member_id == right.member_id
        });
        Ok(sources)
    }

    /// Return shared SQLite containers without opening them or enumerating members.
    pub fn shared_sqlite_containers(
        &self,
        agent: AgentSessionAgentId,
    ) -> Vec<AgentSessionSourceRef> {
        if agent != AgentSessionAgentId::OpenCode {
            return Vec::new();
        }
        self.roots(agent)
            .into_iter()
            .filter(|(variant, _)| variant == "opencode-sqlite")
            .filter_map(|(variant, root)| {
                let db = root.join("opencode.db");
                db.is_file().then(|| {
                    source(
                        agent,
                        variant,
                        &root,
                        db,
                        AgentSessionSourceKind::SqliteMember,
                    )
                })
            })
            .collect()
    }

    pub fn quick_fingerprint(&self, source: &AgentSessionSourceRef) -> Result<String> {
        self.validate_stored_source(source)?;
        let mut state = String::new();
        append_stat(&mut state, &source.physical_path)?;
        for companion in self.companions(source) {
            if companion.is_file() {
                append_stat(&mut state, &companion)?;
            }
        }
        if let Some(member) = &source.member_id {
            state.push_str(member);
            if source.agent == AgentSessionAgentId::OpenCode {
                state.push_str(&sqlite_session_watermark(&source.physical_path, member)?);
            }
        }
        Ok(blake3::hash(state.as_bytes()).to_hex().to_string())
    }

    /// Fast container-level state used before any shared SQLite member query.
    pub fn container_fingerprint(&self, source: &AgentSessionSourceRef) -> Result<String> {
        let mut state = String::new();
        append_stat(&mut state, &source.physical_path)?;
        for companion in self.companions(source) {
            if companion.is_file() {
                append_stat(&mut state, &companion)?;
            }
        }
        Ok(blake3::hash(state.as_bytes()).to_hex().to_string())
    }

    /// Rebuild a provider-owned source reference from the private archive row.
    /// The renderer never supplies any of these fields.
    pub fn restore_source(
        &self,
        agent: AgentSessionAgentId,
        variant: &str,
        physical_path: PathBuf,
        kind: AgentSessionSourceKind,
        member_id: Option<String>,
    ) -> Result<AgentSessionSourceRef> {
        let root = self
            .roots(agent)
            .into_iter()
            .find_map(|(candidate, root)| (candidate == variant).then_some(root))
            .ok_or_else(|| CcrError::ConfigError("unknown session source variant".into()))?;
        let source = AgentSessionSourceRef {
            agent,
            variant: variant.to_string(),
            root,
            physical_path,
            kind,
            member_id,
            project_hint: None,
        };
        self.validate_stored_source(&source)?;
        Ok(source)
    }

    pub fn parse_summary(&self, source: &AgentSessionSourceRef) -> Result<AgentSessionSummary> {
        let (messages, fidelity) = self.read_all_messages(source)?;
        let metadata = metadata_for(source)?;
        let id = native_session_id(source, &metadata);
        let created_at = messages
            .iter()
            .filter_map(|message| message.timestamp)
            .min()
            .unwrap_or(metadata.modified);
        let updated_at = messages
            .iter()
            .filter_map(|message| message.timestamp)
            .max()
            .unwrap_or(metadata.modified);
        let user_message_count = u32::try_from(
            messages
                .iter()
                .filter(|message| message.role == "user")
                .count(),
        )
        .unwrap_or(u32::MAX);
        let assistant_message_count = u32::try_from(
            messages
                .iter()
                .filter(|message| message.role == "assistant")
                .count(),
        )
        .unwrap_or(u32::MAX);
        let tool_use_count = u32::try_from(
            messages
                .iter()
                .filter(|message| message.role == "tool" || message.tool_name.is_some())
                .count(),
        )
        .unwrap_or(u32::MAX);
        let (title, cwd) = summary_metadata(source, &messages)?;
        Ok(AgentSessionSummary {
            native_session_id: id,
            agent: source.agent,
            variant: source.variant.clone(),
            title,
            cwd: cwd
                .or_else(|| source.project_hint.clone())
                .unwrap_or_default(),
            created_at,
            updated_at,
            message_count: user_message_count
                .saturating_add(assistant_message_count)
                .saturating_add(tool_use_count),
            user_message_count,
            assistant_message_count,
            tool_use_count,
            fidelity,
            source: source.clone(),
        })
    }

    pub fn read_message_page(
        &self,
        source: &AgentSessionSourceRef,
        before: Option<u32>,
        limit: usize,
    ) -> Result<AgentSessionMessagePage> {
        self.validate_stored_source(source)?;
        let requested = limit.clamp(1, MAX_MESSAGE_LIMIT);
        let jsonl_path = match source.agent {
            AgentSessionAgentId::Grok => {
                Some(source.physical_path.with_file_name("chat_history.jsonl"))
            }
            AgentSessionAgentId::Claude
            | AgentSessionAgentId::Codex
            | AgentSessionAgentId::Pi
            | AgentSessionAgentId::Omp
            | AgentSessionAgentId::Kimi => Some(source.physical_path.clone()),
            AgentSessionAgentId::OpenCode | AgentSessionAgentId::Antigravity => None,
        };
        if let Some(path) = jsonl_path {
            if !path.is_file() {
                return Ok(AgentSessionMessagePage {
                    messages: Vec::new(),
                    next_before: None,
                    has_older: false,
                    fidelity: AgentSessionFidelity::Partial,
                });
            }
            let (messages, next_before, malformed) =
                read_jsonl_message_page(source, &path, before, requested)?;
            let fidelity = if malformed {
                AgentSessionFidelity::Partial
            } else {
                self.source_fidelity(source)?
            };
            return Ok(AgentSessionMessagePage {
                has_older: next_before.is_some(),
                messages,
                next_before,
                fidelity,
            });
        }
        let mut page = match source.agent {
            AgentSessionAgentId::OpenCode => read_opencode_message_page(source, before, requested)?,
            AgentSessionAgentId::Antigravity => {
                read_antigravity_message_page(source, before, requested)?
            }
            _ => unreachable!("JSONL-backed providers returned above"),
        };
        let source_fidelity = self.source_fidelity(source)?;
        page.fidelity = if page.degraded
            || (source.agent == AgentSessionAgentId::Antigravity
                && ((!page.had_source_messages && source_fidelity != AgentSessionFidelity::Locked)
                    || (page.had_source_messages
                        && source_fidelity == AgentSessionFidelity::Locked)))
        {
            AgentSessionFidelity::Partial
        } else {
            source_fidelity
        };
        Ok(page.into_dto())
    }

    fn read_all_messages(
        &self,
        source: &AgentSessionSourceRef,
    ) -> Result<(Vec<AgentSessionMessage>, AgentSessionFidelity)> {
        self.validate_stored_source(source)?;
        let mut messages = match source.agent {
            AgentSessionAgentId::Grok => read_grok_messages(source)?,
            AgentSessionAgentId::OpenCode => read_opencode_messages(source)?,
            AgentSessionAgentId::Antigravity => read_antigravity_messages(source)?,
            AgentSessionAgentId::Kimi => read_kimi_messages(&source.physical_path)?,
            _ => read_jsonl_messages(&source.physical_path, source.agent)?,
        };
        let fidelity = self.source_fidelity_with_messages(source, &messages);
        for (ordinal, message) in messages.iter_mut().enumerate() {
            message.ordinal = ordinal as u32;
            message.key = format!("{}:{}", native_source_key(source), ordinal);
        }
        Ok((messages, fidelity))
    }

    pub fn validate_stored_source(&self, source: &AgentSessionSourceRef) -> Result<()> {
        if !self
            .roots(source.agent)
            .iter()
            .any(|(variant, root)| variant == &source.variant && root == &source.root)
        {
            return Err(CcrError::ConfigError(
                "session source root is not provider-owned".into(),
            ));
        }
        let canonical_root = source
            .root
            .canonicalize()
            .map_err(|_| CcrError::ConfigError("session source root is unavailable".into()))?;
        let canonical_path = source
            .physical_path
            .canonicalize()
            .map_err(|_| CcrError::ConfigError("session source is missing".into()))?;
        if !canonical_path.starts_with(&canonical_root) || !canonical_path.is_file() {
            return Err(CcrError::ConfigError(
                "session source escaped its canonical root".into(),
            ));
        }
        if !valid_source_shape(source, &canonical_root, &canonical_path) {
            return Err(CcrError::ConfigError(
                "session source shape does not match its provider".into(),
            ));
        }
        match source.kind {
            AgentSessionSourceKind::SqliteMember => {
                let member = source
                    .member_id
                    .as_deref()
                    .filter(|value| valid_member_id(value))
                    .ok_or_else(|| CcrError::ConfigError("invalid session member".into()))?;
                if source.agent != AgentSessionAgentId::OpenCode
                    || !sqlite_member_exists(&canonical_path, member)?
                {
                    return Err(CcrError::ConfigError("session member is missing".into()));
                }
            }
            _ if source.member_id.is_some() => {
                return Err(CcrError::ConfigError("unexpected session member".into()));
            }
            _ => {}
        }
        Ok(())
    }

    fn source_fidelity(&self, source: &AgentSessionSourceRef) -> Result<AgentSessionFidelity> {
        self.validate_stored_source(source)?;
        if source.agent == AgentSessionAgentId::Antigravity
            && source
                .physical_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("pb")
        {
            return Ok(
                if antigravity_plaintext_fallback_exists(source)
                    || antigravity_process_key_available()
                {
                    AgentSessionFidelity::Partial
                } else {
                    AgentSessionFidelity::Locked
                },
            );
        }
        if source.agent == AgentSessionAgentId::Grok
            && !source
                .physical_path
                .with_file_name("chat_history.jsonl")
                .is_file()
        {
            return Ok(AgentSessionFidelity::Partial);
        }
        Ok(AgentSessionFidelity::Full)
    }

    fn source_fidelity_with_messages(
        &self,
        source: &AgentSessionSourceRef,
        messages: &[AgentSessionMessage],
    ) -> AgentSessionFidelity {
        if source.agent == AgentSessionAgentId::Antigravity
            && source
                .physical_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("pb")
        {
            if !messages.is_empty()
                || antigravity_plaintext_fallback_exists(source)
                || antigravity_process_key_available()
            {
                return AgentSessionFidelity::Partial;
            }
            return AgentSessionFidelity::Locked;
        }
        if source.agent == AgentSessionAgentId::Grok
            && !source
                .physical_path
                .with_file_name("chat_history.jsonl")
                .is_file()
        {
            return AgentSessionFidelity::Partial;
        }
        let jsonl_path = if source.agent == AgentSessionAgentId::Grok {
            source.physical_path.with_file_name("chat_history.jsonl")
        } else {
            source.physical_path.clone()
        };
        if jsonl_path.extension().and_then(|value| value.to_str()) == Some("jsonl")
            && jsonl_has_malformed_lines(&jsonl_path)
        {
            return AgentSessionFidelity::Partial;
        }
        if source.agent == AgentSessionAgentId::Antigravity && messages.is_empty() {
            return AgentSessionFidelity::Partial;
        }
        AgentSessionFidelity::Full
    }

    fn companions(&self, source: &AgentSessionSourceRef) -> Vec<PathBuf> {
        match source.agent {
            AgentSessionAgentId::Grok => [
                "chat_history.jsonl",
                "signals.json",
                "updates.jsonl",
                "prompt_context.json",
            ]
            .into_iter()
            .map(|name| source.physical_path.with_file_name(name))
            .collect(),
            AgentSessionAgentId::OpenCode if source.kind == AgentSessionSourceKind::Bundle => {
                opencode_storage_companions(source)
            }
            AgentSessionAgentId::Antigravity => antigravity_companions(source),
            _ => Vec::new(),
        }
    }

    fn roots(&self, agent: AgentSessionAgentId) -> Vec<(String, PathBuf)> {
        match agent {
            AgentSessionAgentId::Claude => vec![("claude-jsonl".into(), claude_root(&self.home))],
            AgentSessionAgentId::Codex => {
                let root = env_path("CODEX_HOME").unwrap_or_else(|| self.home.join(".codex"));
                vec![
                    ("codex-live".into(), root.join("sessions")),
                    ("codex-archived".into(), root.join("archived_sessions")),
                ]
            }
            AgentSessionAgentId::Grok => vec![(
                "grok-bundle".into(),
                self.home.join(".grok").join("sessions"),
            )],
            AgentSessionAgentId::OpenCode => {
                let root = opencode_root(&self.home);
                vec![
                    ("opencode-storage".into(), root.clone()),
                    ("opencode-sqlite".into(), root),
                ]
            }
            AgentSessionAgentId::Pi => vec![(
                "pi-jsonl".into(),
                self.home.join(".pi").join("agent").join("sessions"),
            )],
            AgentSessionAgentId::Omp => vec![(
                "omp-jsonl".into(),
                self.home.join(".omp").join("agent").join("sessions"),
            )],
            AgentSessionAgentId::Antigravity => vec![
                (
                    "antigravity-ide".into(),
                    self.home.join(".gemini").join("antigravity"),
                ),
                (
                    "antigravity-cli".into(),
                    self.home.join(".gemini").join("antigravity-cli"),
                ),
            ],
            AgentSessionAgentId::Kimi => vec![
                (
                    "kimi-legacy".into(),
                    self.home.join(".kimi").join("sessions"),
                ),
                (
                    "kimi-code".into(),
                    self.home.join(".kimi-code").join("sessions"),
                ),
            ],
        }
    }
}

#[derive(Debug)]
struct SourceMetadata {
    modified: DateTime<Utc>,
}

fn source(
    agent: AgentSessionAgentId,
    variant: impl Into<String>,
    root: &Path,
    physical_path: PathBuf,
    kind: AgentSessionSourceKind,
) -> AgentSessionSourceRef {
    AgentSessionSourceRef {
        agent,
        variant: variant.into(),
        root: root.to_path_buf(),
        physical_path,
        kind,
        member_id: None,
        project_hint: None,
    }
}

fn claude_root(home: &Path) -> PathBuf {
    env_path("CLAUDE_PROJECTS_DIR")
        .or_else(|| env_path("CLAUDE_CONFIG_DIR").map(|root| root.join("projects")))
        .unwrap_or_else(|| home.join(".claude").join("projects"))
}

fn opencode_root(home: &Path) -> PathBuf {
    env_path("OPENCODE_DATA_DIR")
        .or_else(|| env_path("XDG_DATA_HOME").map(|root| root.join("opencode")))
        .unwrap_or_else(|| home.join(".local").join("share").join("opencode"))
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn walk_regular_files(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    fn visit(
        path: &Path,
        depth: usize,
        max_depth: usize,
        out: &mut Vec<PathBuf>,
    ) -> std::io::Result<()> {
        if depth > max_depth {
            return Ok(());
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            if ty.is_symlink() {
                continue;
            }
            if ty.is_dir() {
                visit(&entry.path(), depth + 1, max_depth, out)?;
            } else if ty.is_file() {
                out.push(entry.path());
            }
        }
        Ok(())
    }
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    visit(root, 0, max_depth, &mut out).map_err(CcrError::IoError)?;
    Ok(out)
}

fn visit_regular_files_sorted<F>(root: &Path, max_depth: usize, visitor: &mut F) -> Result<()>
where
    F: FnMut(&Path) -> Result<bool>,
{
    fn visit<F>(path: &Path, depth: usize, max_depth: usize, visitor: &mut F) -> Result<bool>
    where
        F: FnMut(&Path) -> Result<bool>,
    {
        if depth > max_depth {
            return Ok(true);
        }
        let mut entries = fs::read_dir(path)
            .map_err(CcrError::IoError)?
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(CcrError::IoError)?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let ty = entry.file_type().map_err(CcrError::IoError)?;
            if ty.is_symlink() {
                continue;
            }
            if ty.is_dir() {
                if !visit(&entry.path(), depth + 1, max_depth, visitor)? {
                    return Ok(false);
                }
            } else if ty.is_file() && !visitor(&entry.path())? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    if root.is_dir() {
        visit(root, 0, max_depth, visitor)?;
    }
    Ok(())
}

fn valid_member_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn valid_source_shape(
    source: &AgentSessionSourceRef,
    canonical_root: &Path,
    canonical_path: &Path,
) -> bool {
    let Ok(relative) = canonical_path.strip_prefix(canonical_root) else {
        return false;
    };
    let extension = canonical_path.extension().and_then(|value| value.to_str());
    let file_name = canonical_path.file_name().and_then(|value| value.to_str());
    match (source.agent, source.variant.as_str()) {
        (AgentSessionAgentId::Claude, "claude-jsonl")
        | (AgentSessionAgentId::Codex, "codex-live" | "codex-archived")
        | (AgentSessionAgentId::Pi, "pi-jsonl")
        | (AgentSessionAgentId::Omp, "omp-jsonl") => {
            source.kind == AgentSessionSourceKind::File
                && source.member_id.is_none()
                && extension == Some("jsonl")
        }
        (AgentSessionAgentId::Kimi, "kimi-legacy" | "kimi-code") => {
            source.kind == AgentSessionSourceKind::File
                && source.member_id.is_none()
                && file_name == Some("wire.jsonl")
        }
        (AgentSessionAgentId::Grok, "grok-bundle") => {
            source.kind == AgentSessionSourceKind::Bundle
                && source.member_id.is_none()
                && file_name == Some("summary.json")
                && canonical_path
                    .parent()
                    .and_then(Path::file_name)
                    .and_then(|value| value.to_str())
                    .is_some_and(valid_member_id)
        }
        (AgentSessionAgentId::OpenCode, "opencode-storage") => {
            source.kind == AgentSessionSourceKind::Bundle
                && source.member_id.is_none()
                && relative.starts_with(Path::new("storage").join("session"))
                && extension == Some("json")
        }
        (AgentSessionAgentId::OpenCode, "opencode-sqlite") => {
            source.kind == AgentSessionSourceKind::SqliteMember
                && file_name == Some("opencode.db")
                && relative == Path::new("opencode.db")
        }
        (AgentSessionAgentId::Antigravity, "antigravity-ide" | "antigravity-cli") => {
            let expected_kind = match extension {
                Some("db") => Some(AgentSessionSourceKind::Bundle),
                Some("pb") => Some(AgentSessionSourceKind::File),
                _ => None,
            };
            expected_kind == Some(source.kind)
                && source.member_id.is_none()
                && (relative.starts_with("conversations") || relative.starts_with("implicit"))
        }
        _ => false,
    }
}

fn open_sqlite_read_only(path: &Path) -> Result<Connection> {
    Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| {
        CcrError::DatabaseError(format!("read-only session database open failed: {error}"))
    })
}

fn sqlite_session_ids(path: &Path) -> Result<Vec<String>> {
    let conn = open_sqlite_read_only(path)?;
    let mut stmt = conn
        .prepare("SELECT id FROM session ORDER BY id")
        .map_err(|error| {
            CcrError::DatabaseError(format!("session member listing failed: {error}"))
        })?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| {
            CcrError::DatabaseError(format!("session member listing failed: {error}"))
        })?;
    Ok(rows
        .filter_map(std::result::Result::ok)
        .filter(|id| valid_member_id(id))
        .collect())
}

fn sqlite_member_exists(path: &Path, member: &str) -> Result<bool> {
    let conn = open_sqlite_read_only(path)?;
    conn.query_row(
        "SELECT 1 FROM session WHERE id = ?1 LIMIT 1",
        [member],
        |_| Ok(()),
    )
    .optional()
    .map(|value| value.is_some())
    .map_err(|error| CcrError::DatabaseError(format!("session member validation failed: {error}")))
}

fn sqlite_session_watermark(path: &Path, member: &str) -> Result<String> {
    let conn = open_sqlite_read_only(path)?;
    let value = conn
        .query_row(
            "SELECT COALESCE(time_updated, 0) FROM session WHERE id = ?1",
            [member],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(|error| {
            CcrError::DatabaseError(format!("session member watermark failed: {error}"))
        })?
        .unwrap_or_default();
    Ok(value.to_string())
}

fn append_stat(output: &mut String, path: &Path) -> Result<()> {
    let metadata = path.metadata().map_err(CcrError::IoError)?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |value| value.as_nanos());
    output.push_str(&format!(
        "{}:{}:{modified};",
        path.file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("source"),
        metadata.len()
    ));
    Ok(())
}

fn metadata_for(source: &AgentSessionSourceRef) -> Result<SourceMetadata> {
    let metadata = source.physical_path.metadata().map_err(CcrError::IoError)?;
    let modified = metadata
        .modified()
        .map(DateTime::<Utc>::from)
        .unwrap_or_else(|_| Utc::now());
    Ok(SourceMetadata { modified })
}

fn native_session_id(source: &AgentSessionSourceRef, _metadata: &SourceMetadata) -> String {
    source
        .member_id
        .clone()
        .or_else(|| {
            source
                .physical_path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_else(|| native_source_key(source))
}

fn native_source_key(source: &AgentSessionSourceRef) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(source.agent.as_str().as_bytes());
    hasher.update(source.variant.as_bytes());
    hasher.update(source.physical_path.to_string_lossy().as_bytes());
    if let Some(member) = &source.member_id {
        hasher.update(member.as_bytes());
    }
    hasher.finalize().to_hex()[..20].to_string()
}

fn read_jsonl_values(path: &Path) -> Result<Vec<Value>> {
    let file = File::open(path).map_err(CcrError::IoError)?;
    let mut values = Vec::new();
    for line in BufReader::new(file).lines() {
        let Ok(line) = line else { continue };
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            values.push(value);
        }
    }
    Ok(values)
}

fn read_jsonl_messages(
    path: &Path,
    agent: AgentSessionAgentId,
) -> Result<Vec<AgentSessionMessage>> {
    let mut messages = Vec::new();
    for value in read_jsonl_values(path)? {
        if let Some(message) = normalize_message(&value, agent) {
            messages.push(message);
        }
    }
    Ok(messages)
}

fn read_jsonl_message_page(
    source: &AgentSessionSourceRef,
    path: &Path,
    before: Option<u32>,
    limit: usize,
) -> Result<(Vec<AgentSessionMessage>, Option<u32>, bool)> {
    let file = File::open(path).map_err(CcrError::IoError)?;
    let mut messages = VecDeque::with_capacity(limit.saturating_add(1));
    let mut ordinal = 0_u32;
    let mut malformed = false;
    for line in BufReader::new(file).lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => {
                malformed = true;
                continue;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let value = match serde_json::from_str::<Value>(&line) {
            Ok(value) => value,
            Err(_) => {
                malformed = true;
                continue;
            }
        };
        let parsed = if source.agent == AgentSessionAgentId::Kimi {
            parse_kimi_value(&value)
        } else {
            normalize_message(&value, source.agent)
        };
        let Some(mut message) = parsed else {
            continue;
        };
        if before.is_some_and(|end| ordinal >= end) {
            break;
        }
        message.ordinal = ordinal;
        message.key = format!("{}:{ordinal}", native_source_key(source));
        ordinal = ordinal.saturating_add(1);
        messages.push_back(message);
        if messages.len() > limit {
            messages.pop_front();
        }
    }
    let messages = messages.into_iter().collect::<Vec<_>>();
    let next_before = messages
        .first()
        .and_then(|message| (message.ordinal > 0).then_some(message.ordinal));
    Ok((messages, next_before, malformed))
}

fn jsonl_has_malformed_lines(path: &Path) -> bool {
    let Ok(file) = File::open(path) else {
        return true;
    };
    BufReader::new(file).lines().any(|line| match line {
        Ok(line) => !line.trim().is_empty() && serde_json::from_str::<Value>(&line).is_err(),
        Err(_) => true,
    })
}

fn normalize_message(value: &Value, agent: AgentSessionAgentId) -> Option<AgentSessionMessage> {
    let payload = value.get("payload").unwrap_or(value);
    let nested = value
        .get("message")
        .filter(|entry| entry.is_object())
        .unwrap_or(payload);
    let event_type = string_at(value, &["type"]).unwrap_or_default();
    let nested_type = string_at(nested, &["type"]).unwrap_or_default();
    let mut role = string_at(value, &["role"]).or_else(|| string_at(nested, &["role"]));
    if role.is_none() {
        role = match event_type.as_str() {
            "user" | "human" | "turn.prompt" => Some("user".into()),
            "assistant" | "text" | "agent_message" => Some("assistant".into()),
            "tool_use" | "tool_call" | "tool_result" => Some("tool".into()),
            "response_item" if nested_type == "message" => string_at(nested, &["role"]),
            _ => None,
        };
    }
    if agent == AgentSessionAgentId::Kimi && event_type == "TurnBegin" {
        role = Some("user".into());
    }
    let mut role = role?;
    let has_tool_block = nested
        .get("content")
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().any(is_tool_block));
    if matches!(role.as_str(), "toolResult" | "tool_result") || (role == "user" && has_tool_block) {
        role = "tool".into();
    }
    if !matches!(role.as_str(), "user" | "assistant" | "tool") {
        return None;
    }
    let tool_name = string_at(value, &["tool_name"])
        .or_else(|| string_at(nested, &["name"]))
        .or_else(|| nested.get("content").and_then(first_tool_name));
    let content_value = nested
        .get("content")
        .or_else(|| nested.get("text"))
        .or_else(|| payload.get("content"))
        .or_else(|| payload.get("text"))
        .or_else(|| value.get("prompt"));
    let content = content_value.and_then(extract_text).unwrap_or_else(|| {
        tool_name
            .clone()
            .map(|name| format!("Tool: {name}"))
            .unwrap_or_default()
    });
    if content.trim().is_empty() && tool_name.is_none() {
        return None;
    }
    let (content, clipped) = clip_utf8(&content, MAX_MESSAGE_BYTES);
    let timestamp = string_at(value, &["timestamp"])
        .or_else(|| string_at(payload, &["timestamp"]))
        .and_then(|value| parse_timestamp(&value))
        .or_else(|| number_at(value, &["time", "created"]).and_then(parse_unix_timestamp));
    Some(AgentSessionMessage {
        key: String::new(),
        ordinal: 0,
        role,
        timestamp,
        content,
        tool_name,
        clipped,
    })
}

fn is_tool_block(value: &Value) -> bool {
    string_at(value, &["type"]).is_some_and(|kind| {
        matches!(
            kind.as_str(),
            "tool_use" | "tool_result" | "toolCall" | "tool_call"
        )
    })
}

fn first_tool_name(value: &Value) -> Option<String> {
    value.as_array()?.iter().find_map(|item| {
        is_tool_block(item)
            .then(|| string_at(item, &["name"]).or_else(|| string_at(item, &["function", "name"])))
            .flatten()
    })
}

fn extract_text(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.clone()),
        Value::Array(items) => {
            let parts: Vec<String> = items
                .iter()
                .filter_map(|item| {
                    item.as_str().map(ToString::to_string).or_else(|| {
                        item.get("text")
                            .and_then(Value::as_str)
                            .map(ToString::to_string)
                    })
                })
                .collect();
            (!parts.is_empty()).then(|| parts.join("\n"))
        }
        Value::Object(map) => map
            .get("text")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| map.get("content").and_then(extract_text)),
        _ => None,
    }
}

fn read_grok_messages(source: &AgentSessionSourceRef) -> Result<Vec<AgentSessionMessage>> {
    let transcript = source.physical_path.with_file_name("chat_history.jsonl");
    if transcript.is_file() {
        read_jsonl_messages(&transcript, AgentSessionAgentId::Grok)
    } else {
        Ok(Vec::new())
    }
}

fn read_kimi_messages(path: &Path) -> Result<Vec<AgentSessionMessage>> {
    Ok(read_jsonl_values(path)?
        .iter()
        .filter_map(parse_kimi_value)
        .collect())
}

fn parse_kimi_value(value: &Value) -> Option<AgentSessionMessage> {
    let record_type = string_at(value, &["type"]).unwrap_or_default();
    let nested_type = string_at(value, &["message", "type"]).unwrap_or_default();
    let timestamp = string_at(value, &["timestamp"])
        .and_then(|value| parse_timestamp(&value))
        .or_else(|| number_at(value, &["timestamp"]).and_then(parse_unix_timestamp));
    match record_type.as_str() {
        "turn.prompt" | "turn.steer" => value
            .get("input")
            .and_then(extract_text)
            .and_then(|content| kimi_message("user", content, None, timestamp)),
        "context.append_loop_event" => parse_kimi_loop_event(value, timestamp),
        _ => match nested_type.as_str() {
            "TurnBegin" => value
                .get("message")
                .and_then(|message| message.get("payload"))
                .and_then(|payload| payload.get("user_input"))
                .and_then(extract_text)
                .and_then(|content| kimi_message("user", content, None, timestamp)),
            "ContentPart" => value
                .get("message")
                .and_then(|message| message.get("payload"))
                .and_then(kimi_content_part)
                .and_then(|content| kimi_message("assistant", content, None, timestamp)),
            "ToolCall" => {
                let payload = value
                    .get("message")
                    .and_then(|message| message.get("payload"));
                let tool_name =
                    payload.and_then(|payload| string_at(payload, &["function", "name"]));
                kimi_message(
                    "tool",
                    tool_summary("Tool call", tool_name.as_deref()),
                    tool_name,
                    timestamp,
                )
            }
            "ToolResult" => kimi_message("tool", "Tool result".into(), None, timestamp),
            _ => normalize_message(value, AgentSessionAgentId::Kimi),
        },
    }
}

fn parse_kimi_loop_event(
    value: &Value,
    timestamp: Option<DateTime<Utc>>,
) -> Option<AgentSessionMessage> {
    let event = value.get("event")?;
    match string_at(event, &["type"]).as_deref() {
        Some("content.part") => event
            .get("part")
            .and_then(kimi_content_part)
            .and_then(|content| kimi_message("assistant", content, None, timestamp)),
        Some("tool.call") => {
            let tool_name = string_at(event, &["name"]);
            kimi_message(
                "tool",
                tool_summary("Tool call", tool_name.as_deref()),
                tool_name,
                timestamp,
            )
        }
        Some("tool.result") => kimi_message("tool", "Tool result".into(), None, timestamp),
        _ => None,
    }
}

fn kimi_content_part(value: &Value) -> Option<String> {
    match string_at(value, &["type"]).as_deref() {
        Some("text") => string_at(value, &["text"]),
        Some("think") => string_at(value, &["think"]).or_else(|| string_at(value, &["text"])),
        _ => None,
    }
}

fn tool_summary(prefix: &str, name: Option<&str>) -> String {
    name.map_or_else(|| prefix.to_string(), |name| format!("{prefix}: {name}"))
}

fn kimi_message(
    role: &str,
    content: String,
    tool_name: Option<String>,
    timestamp: Option<DateTime<Utc>>,
) -> Option<AgentSessionMessage> {
    if content.trim().is_empty() && tool_name.is_none() {
        return None;
    }
    let (content, clipped) = clip_utf8(&content, MAX_MESSAGE_BYTES);
    Some(AgentSessionMessage {
        key: String::new(),
        ordinal: 0,
        role: role.into(),
        timestamp,
        content,
        tool_name,
        clipped,
    })
}

fn read_opencode_message_page(
    source: &AgentSessionSourceRef,
    before: Option<u32>,
    limit: usize,
) -> Result<BoundedProviderPage> {
    let mut window = BoundedMessageWindow::new(source, before, limit);
    if source.kind == AgentSessionSourceKind::SqliteMember {
        read_opencode_sqlite_message_window(
            &source.physical_path,
            source.member_id.as_deref().unwrap_or_default(),
            &mut window,
        )?;
        return Ok(window.finish());
    }

    let session: Value =
        serde_json::from_reader(File::open(&source.physical_path).map_err(CcrError::IoError)?)
            .map_err(|error| {
                CcrError::ConfigError(format!("opencode session metadata is invalid: {error}"))
            })?;
    let session_id = string_at(&session, &["id"])
        .or_else(|| {
            source
                .physical_path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_default();
    let storage_root = source.root.join("storage");
    let message_root = storage_root.join("message").join(&session_id);
    visit_regular_files_sorted(&message_root, 2, &mut |path| {
        if !window.wants_more() {
            return Ok(false);
        }
        let value =
            match serde_json::from_reader::<_, Value>(File::open(path).map_err(CcrError::IoError)?)
            {
                Ok(value) => value,
                Err(_) => {
                    window.degraded = true;
                    return Ok(true);
                }
            };
        let message_id = string_at(&value, &["id"]).unwrap_or_else(|| {
            path.file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("message")
                .to_string()
        });
        let role = string_at(&value, &["role"]).unwrap_or_else(|| "assistant".into());
        let timestamp = number_at(&value, &["time", "created"]).and_then(parse_unix_timestamp);
        let mut content = String::new();
        let mut clipped = false;
        let mut tool_name = None;
        visit_regular_files_sorted(
            &storage_root.join("part").join(&message_id),
            2,
            &mut |part_path| {
                let part = match serde_json::from_reader::<_, Value>(
                    File::open(part_path).map_err(CcrError::IoError)?,
                ) {
                    Ok(value) => value,
                    Err(_) => {
                        window.degraded = true;
                        return Ok(true);
                    }
                };
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    append_bounded_text(&mut content, text, &mut clipped);
                }
                if tool_name.is_none() {
                    tool_name = string_at(&part, &["tool"]);
                }
                Ok(true)
            },
        )?;
        if content.is_empty() && tool_name.is_none() {
            return Ok(true);
        }
        Ok(window.push(AgentSessionMessage {
            key: String::new(),
            ordinal: 0,
            role,
            timestamp,
            content,
            tool_name,
            clipped,
        }))
    })?;
    Ok(window.finish())
}

fn read_opencode_sqlite_message_window(
    path: &Path,
    member: &str,
    window: &mut BoundedMessageWindow,
) -> Result<()> {
    let conn = open_sqlite_read_only(path)?;
    let mut message_stmt = conn
        .prepare(
            "SELECT id, data, time_created FROM message WHERE session_id = ?1 \
             ORDER BY time_created, id",
        )
        .map_err(|error| {
            CcrError::DatabaseError(format!("opencode message query failed: {error}"))
        })?;
    let rows = message_stmt
        .query_map([member], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| {
            CcrError::DatabaseError(format!("opencode message query failed: {error}"))
        })?;
    let mut part_stmt = conn
        .prepare(
            "SELECT data FROM part WHERE session_id = ?1 AND message_id = ?2 \
             ORDER BY time_created, id",
        )
        .map_err(|error| CcrError::DatabaseError(format!("opencode part query failed: {error}")))?;
    for row in rows {
        if !window.wants_more() {
            break;
        }
        let Ok((message_id, data, created)) = row else {
            window.degraded = true;
            continue;
        };
        let value: Value = serde_json::from_str(&data).unwrap_or_else(|_| {
            window.degraded = true;
            Value::Null
        });
        let role = string_at(&value, &["role"]).unwrap_or_else(|| "assistant".into());
        let mut content = String::new();
        let mut clipped = false;
        let mut tool_name = None;
        let part_rows = part_stmt
            .query_map(params![member, message_id], |row| row.get::<_, String>(0))
            .map_err(|error| {
                CcrError::DatabaseError(format!("opencode part query failed: {error}"))
            })?;
        for part in part_rows {
            let Ok(part) = part else {
                window.degraded = true;
                continue;
            };
            let part: Value = serde_json::from_str(&part).unwrap_or_else(|_| {
                window.degraded = true;
                Value::Null
            });
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                append_bounded_text(&mut content, text, &mut clipped);
            }
            if tool_name.is_none() {
                tool_name = string_at(&part, &["tool"]);
            }
        }
        if content.is_empty() && tool_name.is_none() {
            continue;
        }
        if !window.push(AgentSessionMessage {
            key: String::new(),
            ordinal: 0,
            role,
            timestamp: parse_unix_timestamp(created),
            content,
            tool_name,
            clipped,
        }) {
            break;
        }
    }
    Ok(())
}

fn read_antigravity_message_page(
    source: &AgentSessionSourceRef,
    before: Option<u32>,
    limit: usize,
) -> Result<BoundedProviderPage> {
    let id = source
        .physical_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let trajectory = source.physical_path.with_extension("trajectory.json");
    if trajectory.is_file() {
        let value: Value = serde_json::from_reader(
            File::open(trajectory).map_err(CcrError::IoError)?,
        )
        .map_err(|error| {
            CcrError::ConfigError(format!("antigravity trajectory is invalid: {error}"))
        })?;
        let mut window = BoundedMessageWindow::new(source, before, limit);
        collect_messages_recursive_window(&value, AgentSessionAgentId::Antigravity, &mut window);
        if window.had_source_messages {
            return Ok(window.finish());
        }
    }

    if source
        .physical_path
        .extension()
        .and_then(|value| value.to_str())
        == Some("db")
    {
        let mut window = BoundedMessageWindow::new(source, before, limit);
        if read_generic_message_table_window(&source.physical_path, &mut window).is_ok()
            && window.had_source_messages
        {
            return Ok(window.finish());
        }
    }

    let mut window = BoundedMessageWindow::new(source, before, limit);
    let history = source.root.join("history.jsonl");
    if history.is_file() {
        let file = File::open(&history).map_err(CcrError::IoError)?;
        for line in BufReader::new(file).lines() {
            if !window.wants_more() {
                break;
            }
            let line = match line {
                Ok(line) => line,
                Err(_) => {
                    window.degraded = true;
                    continue;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            let value = match serde_json::from_str::<Value>(&line) {
                Ok(value) => value,
                Err(_) => {
                    window.degraded = true;
                    continue;
                }
            };
            if string_at(&value, &["conversationId", "conversation_id"]).as_deref() != Some(id) {
                continue;
            }
            if let Some(message) = normalize_message(&value, AgentSessionAgentId::Antigravity)
                && !window.push(message)
            {
                break;
            }
        }
    }
    if window.wants_more() {
        let brain = source.root.join("brain").join(id);
        visit_regular_files_sorted(&brain, 3, &mut |path| {
            if !window.wants_more() {
                return Ok(false);
            }
            if !matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("md" | "txt")
            ) {
                return Ok(true);
            }
            let (content, clipped) = match read_bounded_text_file(path) {
                Ok(value) => value,
                Err(_) => {
                    window.degraded = true;
                    return Ok(true);
                }
            };
            if content.trim().is_empty() {
                return Ok(true);
            }
            Ok(window.push(AgentSessionMessage {
                key: String::new(),
                ordinal: 0,
                role: "assistant".into(),
                timestamp: path
                    .metadata()
                    .ok()
                    .and_then(|value| value.modified().ok())
                    .map(DateTime::<Utc>::from),
                content,
                tool_name: None,
                clipped,
            }))
        })?;
    }
    Ok(window.finish())
}

fn read_generic_message_table_window(path: &Path, window: &mut BoundedMessageWindow) -> Result<()> {
    let conn = open_sqlite_read_only(path)?;
    let table = ["message", "messages", "steps"]
        .into_iter()
        .find(|name| {
            conn.query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                [name],
                |_| Ok(()),
            )
            .optional()
            .ok()
            .flatten()
            .is_some()
        })
        .ok_or_else(|| {
            CcrError::DatabaseError("antigravity message table is unavailable".into())
        })?;
    let columns = table_columns(&conn, table)?;
    let role_col = ["role", "sender"]
        .into_iter()
        .find(|candidate| columns.iter().any(|column| column == candidate));
    let content_col = ["content", "text", "message"]
        .into_iter()
        .find(|candidate| columns.iter().any(|column| column == candidate));
    let Some(content_col) = content_col else {
        return Ok(());
    };
    let role_expr = role_col.unwrap_or("'assistant'");
    let sql = format!("SELECT {role_expr}, {content_col} FROM {table} ORDER BY rowid");
    let mut stmt = conn.prepare(&sql).map_err(|error| {
        CcrError::DatabaseError(format!("antigravity message query failed: {error}"))
    })?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)
                    .unwrap_or_else(|_| "assistant".into()),
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        })
        .map_err(|error| {
            CcrError::DatabaseError(format!("antigravity message query failed: {error}"))
        })?;
    for row in rows {
        if !window.wants_more() {
            break;
        }
        let Ok((role, text)) = row else {
            window.degraded = true;
            continue;
        };
        if text.trim().is_empty() {
            continue;
        }
        let (content, clipped) = clip_utf8(&text, MAX_MESSAGE_BYTES);
        if !window.push(AgentSessionMessage {
            key: String::new(),
            ordinal: 0,
            role,
            timestamp: None,
            content,
            tool_name: None,
            clipped,
        }) {
            break;
        }
    }
    Ok(())
}

fn collect_messages_recursive_window(
    value: &Value,
    agent: AgentSessionAgentId,
    window: &mut BoundedMessageWindow,
) -> bool {
    if let Some(message) = normalize_message(value, agent)
        && !window.push(message)
    {
        return false;
    }
    match value {
        Value::Array(items) => {
            for item in items {
                if !collect_messages_recursive_window(item, agent, window) {
                    return false;
                }
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                if item.is_array() && !collect_messages_recursive_window(item, agent, window) {
                    return false;
                }
            }
        }
        _ => {}
    }
    true
}

fn read_opencode_messages(source: &AgentSessionSourceRef) -> Result<Vec<AgentSessionMessage>> {
    if source.kind == AgentSessionSourceKind::SqliteMember {
        return read_opencode_sqlite_messages(
            &source.physical_path,
            source.member_id.as_deref().unwrap_or_default(),
        );
    }
    let session: Value =
        serde_json::from_reader(File::open(&source.physical_path).map_err(CcrError::IoError)?)
            .map_err(|error| {
                CcrError::ConfigError(format!("opencode session metadata is invalid: {error}"))
            })?;
    let session_id = string_at(&session, &["id"])
        .or_else(|| {
            source
                .physical_path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(ToString::to_string)
        })
        .unwrap_or_default();
    let root = source.root.join("storage");
    let message_root = root.join("message").join(&session_id);
    let mut message_rows = BTreeMap::<String, (String, Option<DateTime<Utc>>)>::new();
    for path in walk_regular_files(&message_root, 2)? {
        if let Ok(value) =
            serde_json::from_reader::<_, Value>(File::open(&path).map_err(CcrError::IoError)?)
        {
            let id = string_at(&value, &["id"]).unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("message")
                    .to_string()
            });
            let role = string_at(&value, &["role"]).unwrap_or_else(|| "assistant".into());
            let timestamp = number_at(&value, &["time", "created"]).and_then(parse_unix_timestamp);
            message_rows.insert(id, (role, timestamp));
        }
    }
    let mut messages = Vec::new();
    for (message_id, (role, timestamp)) in message_rows {
        let mut content = Vec::new();
        let mut tool_name = None;
        for path in walk_regular_files(&root.join("part").join(&message_id), 2)? {
            if let Ok(value) =
                serde_json::from_reader::<_, Value>(File::open(&path).map_err(CcrError::IoError)?)
            {
                if let Some(text) = value.get("text").and_then(Value::as_str) {
                    content.push(text.to_string());
                }
                if tool_name.is_none() {
                    tool_name = string_at(&value, &["tool"]);
                }
            }
        }
        let joined = content.join("\n");
        if joined.is_empty() && tool_name.is_none() {
            continue;
        }
        let (content, clipped) = clip_utf8(&joined, MAX_MESSAGE_BYTES);
        messages.push(AgentSessionMessage {
            key: String::new(),
            ordinal: 0,
            role,
            timestamp,
            content,
            tool_name,
            clipped,
        });
    }
    Ok(messages)
}

fn read_opencode_sqlite_messages(path: &Path, member: &str) -> Result<Vec<AgentSessionMessage>> {
    let conn = open_sqlite_read_only(path)?;
    let mut stmt = conn.prepare("SELECT id, data, time_created FROM message WHERE session_id = ?1 ORDER BY time_created, id")
        .map_err(|error| CcrError::DatabaseError(format!("opencode message query failed: {error}")))?;
    let rows = stmt
        .query_map([member], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| {
            CcrError::DatabaseError(format!("opencode message query failed: {error}"))
        })?;
    let mut messages = Vec::new();
    for row in rows {
        let Ok((message_id, data, created)) = row else {
            continue;
        };
        let value: Value = serde_json::from_str(&data).unwrap_or(Value::Null);
        let role = string_at(&value, &["role"]).unwrap_or_else(|| "assistant".into());
        let mut content = Vec::new();
        let mut tool_name = None;
        let mut parts = conn.prepare("SELECT data FROM part WHERE session_id = ?1 AND message_id = ?2 ORDER BY time_created, id")
            .map_err(|error| CcrError::DatabaseError(format!("opencode part query failed: {error}")))?;
        let part_rows = parts
            .query_map(params![member, message_id], |row| row.get::<_, String>(0))
            .map_err(|error| {
                CcrError::DatabaseError(format!("opencode part query failed: {error}"))
            })?;
        for part in part_rows.filter_map(std::result::Result::ok) {
            let part: Value = serde_json::from_str(&part).unwrap_or(Value::Null);
            if let Some(text) = part.get("text").and_then(Value::as_str) {
                content.push(text.to_string());
            }
            if tool_name.is_none() {
                tool_name = string_at(&part, &["tool"]);
            }
        }
        let joined = content.join("\n");
        if joined.is_empty() && tool_name.is_none() {
            continue;
        }
        let (content, clipped) = clip_utf8(&joined, MAX_MESSAGE_BYTES);
        messages.push(AgentSessionMessage {
            key: String::new(),
            ordinal: 0,
            role,
            timestamp: parse_unix_timestamp(created),
            content,
            tool_name,
            clipped,
        });
    }
    Ok(messages)
}

fn read_antigravity_messages(source: &AgentSessionSourceRef) -> Result<Vec<AgentSessionMessage>> {
    let id = source
        .physical_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let trajectory = source.physical_path.with_extension("trajectory.json");
    if trajectory.is_file() {
        let value: Value = serde_json::from_reader(
            File::open(trajectory).map_err(CcrError::IoError)?,
        )
        .map_err(|error| {
            CcrError::ConfigError(format!("antigravity trajectory is invalid: {error}"))
        })?;
        let mut messages = Vec::new();
        collect_messages_recursive(&value, AgentSessionAgentId::Antigravity, &mut messages);
        if !messages.is_empty() {
            return Ok(messages);
        }
    }
    if source
        .physical_path
        .extension()
        .and_then(|value| value.to_str())
        == Some("db")
        && let Ok(messages) = read_generic_message_table(&source.physical_path)
        && !messages.is_empty()
    {
        return Ok(messages);
    }
    let history = source.root.join("history.jsonl");
    let mut messages = if history.is_file() {
        read_jsonl_values(&history)?
            .into_iter()
            .filter(|value| {
                string_at(value, &["conversationId", "conversation_id"]).as_deref() == Some(id)
            })
            .filter_map(|value| normalize_message(&value, AgentSessionAgentId::Antigravity))
            .collect()
    } else {
        Vec::new()
    };
    let brain = source.root.join("brain").join(id);
    for path in walk_regular_files(&brain, 3)? {
        if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("md" | "txt")
        ) {
            let mut text = String::new();
            if File::open(&path)
                .and_then(|mut file| file.read_to_string(&mut text))
                .is_ok()
                && !text.trim().is_empty()
            {
                let (content, clipped) = clip_utf8(&text, MAX_MESSAGE_BYTES);
                messages.push(AgentSessionMessage {
                    key: String::new(),
                    ordinal: 0,
                    role: "assistant".into(),
                    timestamp: path
                        .metadata()
                        .ok()
                        .and_then(|value| value.modified().ok())
                        .map(DateTime::<Utc>::from),
                    content,
                    tool_name: None,
                    clipped,
                });
            }
        }
    }
    Ok(messages)
}

fn read_generic_message_table(path: &Path) -> Result<Vec<AgentSessionMessage>> {
    let conn = open_sqlite_read_only(path)?;
    let table = ["message", "messages", "steps"]
        .into_iter()
        .find(|name| {
            conn.query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                [name],
                |_| Ok(()),
            )
            .optional()
            .ok()
            .flatten()
            .is_some()
        })
        .ok_or_else(|| {
            CcrError::DatabaseError("antigravity message table is unavailable".into())
        })?;
    let columns = table_columns(&conn, table)?;
    let role_col = ["role", "sender"]
        .into_iter()
        .find(|candidate| columns.iter().any(|column| column == candidate));
    let content_col = ["content", "text", "message"]
        .into_iter()
        .find(|candidate| columns.iter().any(|column| column == candidate));
    let Some(content_col) = content_col else {
        return Ok(Vec::new());
    };
    let role_expr = role_col.unwrap_or("'assistant'");
    let sql = format!("SELECT {role_expr}, {content_col} FROM {table} LIMIT 10000");
    let mut stmt = conn.prepare(&sql).map_err(|error| {
        CcrError::DatabaseError(format!("antigravity message query failed: {error}"))
    })?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)
                    .unwrap_or_else(|_| "assistant".into()),
                row.get::<_, String>(1).unwrap_or_default(),
            ))
        })
        .map_err(|error| {
            CcrError::DatabaseError(format!("antigravity message query failed: {error}"))
        })?;
    Ok(rows
        .filter_map(std::result::Result::ok)
        .filter(|(_, text)| !text.trim().is_empty())
        .map(|(role, text)| {
            let (content, clipped) = clip_utf8(&text, MAX_MESSAGE_BYTES);
            AgentSessionMessage {
                key: String::new(),
                ordinal: 0,
                role,
                timestamp: None,
                content,
                tool_name: None,
                clipped,
            }
        })
        .collect())
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| CcrError::DatabaseError(error.to_string()))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| CcrError::DatabaseError(error.to_string()))?;
    Ok(rows.filter_map(std::result::Result::ok).collect())
}

fn collect_messages_recursive(
    value: &Value,
    agent: AgentSessionAgentId,
    output: &mut Vec<AgentSessionMessage>,
) {
    if let Some(message) = normalize_message(value, agent) {
        output.push(message);
    }
    match value {
        Value::Array(items) => {
            for item in items {
                collect_messages_recursive(item, agent, output);
            }
        }
        Value::Object(map) => {
            for item in map.values() {
                if item.is_array() {
                    collect_messages_recursive(item, agent, output);
                }
            }
        }
        _ => {}
    }
}

fn summary_metadata(
    source: &AgentSessionSourceRef,
    messages: &[AgentSessionMessage],
) -> Result<(Option<String>, Option<String>)> {
    let first_user = messages
        .iter()
        .find(|message| message.role == "user")
        .map(|message| message.content.lines().next().unwrap_or_default().trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(120).collect());
    match source.agent {
        AgentSessionAgentId::Grok | AgentSessionAgentId::OpenCode
            if source.kind != AgentSessionSourceKind::SqliteMember =>
        {
            let value: Value = serde_json::from_reader(
                File::open(&source.physical_path).map_err(CcrError::IoError)?,
            )
            .unwrap_or(Value::Null);
            Ok((
                string_at(&value, &["title"]).or(first_user),
                string_at(&value, &["cwd"]).or_else(|| string_at(&value, &["directory"])),
            ))
        }
        AgentSessionAgentId::OpenCode => {
            let conn = open_sqlite_read_only(&source.physical_path)?;
            let member = source.member_id.as_deref().unwrap_or_default();
            let value = conn
                .query_row(
                    "SELECT COALESCE(title,''), COALESCE(directory,'') FROM session WHERE id=?1",
                    [member],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                )
                .or_else(|_| {
                    conn.query_row(
                        "SELECT COALESCE(title,''), '' FROM session WHERE id=?1",
                        [member],
                        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
                    )
                })
                .map_err(|error| {
                    CcrError::DatabaseError(format!(
                        "opencode session metadata query failed: {error}"
                    ))
                })?;
            Ok((
                (!value.0.is_empty()).then_some(value.0).or(first_user),
                (!value.1.is_empty()).then_some(value.1),
            ))
        }
        _ => {
            let values = if source
                .physical_path
                .extension()
                .and_then(|value| value.to_str())
                == Some("jsonl")
            {
                read_jsonl_values(&source.physical_path)?
            } else {
                Vec::new()
            };
            let title = values
                .iter()
                .find_map(|value| {
                    string_at(value, &["title"]).or_else(|| string_at(value, &["session", "title"]))
                })
                .or(first_user);
            let cwd = values.iter().find_map(|value| {
                string_at(value, &["cwd"]).or_else(|| string_at(value, &["session", "cwd"]))
            });
            Ok((title, cwd))
        }
    }
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current.as_str().map(ToString::to_string)
}

fn number_at(value: &Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    current
        .as_i64()
        .or_else(|| current.as_u64().and_then(|value| i64::try_from(value).ok()))
}

fn parse_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|value| value.with_timezone(&Utc))
        .or_else(|| value.parse::<i64>().ok().and_then(parse_unix_timestamp))
}

fn parse_unix_timestamp(value: i64) -> Option<DateTime<Utc>> {
    if value.abs() > 10_000_000_000 {
        Utc.timestamp_millis_opt(value).single()
    } else {
        Utc.timestamp_opt(value, 0).single()
    }
}

fn clip_utf8(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_string(), false);
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    (value[..end].to_string(), true)
}

fn append_bounded_text(output: &mut String, value: &str, clipped: &mut bool) {
    if value.is_empty() {
        return;
    }
    if !output.is_empty() {
        if output.len() == MAX_MESSAGE_BYTES {
            *clipped = true;
            return;
        }
        output.push('\n');
    }
    let remaining = MAX_MESSAGE_BYTES.saturating_sub(output.len());
    if value.len() <= remaining {
        output.push_str(value);
        return;
    }
    let mut end = remaining;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    output.push_str(&value[..end]);
    *clipped = true;
}

fn read_bounded_text_file(path: &Path) -> Result<(String, bool)> {
    let file = File::open(path).map_err(CcrError::IoError)?;
    let mut bytes = Vec::with_capacity(MAX_MESSAGE_BYTES.saturating_add(1));
    file.take((MAX_MESSAGE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(CcrError::IoError)?;
    let exceeded = bytes.len() > MAX_MESSAGE_BYTES;
    bytes.truncate(MAX_MESSAGE_BYTES);
    let text = String::from_utf8_lossy(&bytes);
    let (text, utf8_clipped) = clip_utf8(&text, MAX_MESSAGE_BYTES);
    Ok((text, exceeded || utf8_clipped))
}

fn opencode_storage_companions(source: &AgentSessionSourceRef) -> Vec<PathBuf> {
    let id = source
        .physical_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let root = source.root.join("storage");
    let mut paths = vec![root.join("message").join(id)];
    if let Ok(message_files) = walk_regular_files(&paths[0], 2) {
        for file in message_files {
            if let Some(message_id) = file.file_stem() {
                paths.push(root.join("part").join(message_id));
            }
        }
    }
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            files.push(path);
        } else if path.is_dir()
            && let Ok(found) = walk_regular_files(&path, 2)
        {
            files.extend(found);
        }
    }
    files
}

fn antigravity_companions(source: &AgentSessionSourceRef) -> Vec<PathBuf> {
    let id = source
        .physical_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let mut paths = vec![
        source.physical_path.with_extension("trajectory.json"),
        source.root.join("history.jsonl"),
    ];
    if let Ok(files) = walk_regular_files(&source.root.join("brain").join(id), 3) {
        paths.extend(files);
    }
    paths
}

fn antigravity_plaintext_fallback_exists(source: &AgentSessionSourceRef) -> bool {
    let id = source
        .physical_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if source
        .physical_path
        .with_extension("trajectory.json")
        .is_file()
    {
        return true;
    }
    walk_regular_files(&source.root.join("brain").join(id), 3).is_ok_and(|files| !files.is_empty())
}

fn antigravity_process_key_available() -> bool {
    std::env::var_os("ANTIGRAVITY_KEY").is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn definitions_keep_eight_independent_agent_ids() {
        let definitions = AgentSessionProviderRegistry::definitions();
        assert_eq!(definitions.len(), 8);
        assert_eq!(definitions[5].agent, AgentSessionAgentId::Omp);
        assert_eq!(definitions[5].label, "OMP");
    }

    #[test]
    fn stored_source_validation_rejects_provider_shape_tampering() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(".kimi-code/sessions");
        let valid = root.join("project/session/agents/main/wire.jsonl");
        let unrelated = root.join("project/session/agents/main/private.jsonl");
        write(&valid, "{\"type\":\"turn.prompt\",\"input\":\"hello\"}\n");
        write(&unrelated, "{\"role\":\"user\",\"content\":\"private\"}\n");
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        assert!(
            registry
                .restore_source(
                    AgentSessionAgentId::Kimi,
                    "kimi-code",
                    valid,
                    AgentSessionSourceKind::File,
                    None,
                )
                .is_ok()
        );
        assert!(
            registry
                .restore_source(
                    AgentSessionAgentId::Kimi,
                    "kimi-code",
                    unrelated,
                    AgentSessionSourceKind::File,
                    None,
                )
                .is_err()
        );
    }

    #[test]
    fn discovers_and_pages_file_bundle_and_kimi_sources() {
        let temp = TempDir::new().unwrap();
        write(
            &temp.path().join(".claude/projects/demo/session.jsonl"),
            "{\"type\":\"user\",\"message\":{\"role\":\"user\",\"content\":\"hello\"},\"timestamp\":\"2026-01-01T00:00:00Z\",\"session_id\":\"c1\",\"cwd\":\"/work\"}\n{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":\"world\"}}\n",
        );
        write(
            &temp.path().join(".codex/sessions/2026/01/session.jsonl"),
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"codex question\"}]}}\n{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"codex answer\"}]}}\n",
        );
        write(
            &temp.path().join(".grok/sessions/demo/g1/summary.json"),
            "{\"title\":\"Grok demo\",\"cwd\":\"/work\"}",
        );
        write(
            &temp
                .path()
                .join(".grok/sessions/demo/g1/chat_history.jsonl"),
            "{\"role\":\"user\",\"content\":\"ask\"}\n{\"role\":\"assistant\",\"content\":\"answer\"}\n",
        );
        write(
            &temp
                .path()
                .join(".kimi-code/sessions/wd_demo_deadbeef1234/session_k1/agents/main/wire.jsonl"),
            "{\"type\":\"turn.prompt\",\"input\":[{\"type\":\"text\",\"text\":\"kimi question\"}]}\n{\"type\":\"context.append_loop_event\",\"event\":{\"type\":\"content.part\",\"part\":{\"type\":\"text\",\"text\":\"kimi answer\"}}}\n{\"type\":\"context.append_loop_event\",\"event\":{\"type\":\"tool.call\",\"name\":\"read_file\",\"args\":{\"path\":\"private\"}}}\n",
        );
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        for agent in [
            AgentSessionAgentId::Claude,
            AgentSessionAgentId::Codex,
            AgentSessionAgentId::Grok,
            AgentSessionAgentId::Kimi,
        ] {
            let sources = registry.discover(agent).unwrap();
            assert_eq!(sources.len(), 1, "{agent:?}");
            let summary = registry.parse_summary(&sources[0]).unwrap();
            assert_eq!(summary.agent, agent);
            assert!(summary.message_count >= 2);
            let page = registry.read_message_page(&sources[0], None, 1).unwrap();
            assert_eq!(page.messages.len(), 1);
            assert!(page.has_older);
        }
        let kimi_source = registry
            .discover(AgentSessionAgentId::Kimi)
            .unwrap()
            .remove(0);
        assert_eq!(
            registry.parse_summary(&kimi_source).unwrap().tool_use_count,
            1
        );
    }

    #[test]
    fn pi_and_omp_share_format_but_keep_identity() {
        let temp = TempDir::new().unwrap();
        let transcript = "{\"type\":\"session\",\"id\":\"s1\",\"cwd\":\"/work\"}\n{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"hello\"}}\n";
        write(
            &temp.path().join(".pi/agent/sessions/p/s1.jsonl"),
            transcript,
        );
        write(
            &temp.path().join(".omp/agent/sessions/p/s1.jsonl"),
            transcript,
        );
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let pi = registry
            .parse_summary(&registry.discover(AgentSessionAgentId::Pi).unwrap()[0])
            .unwrap();
        let omp = registry
            .parse_summary(&registry.discover(AgentSessionAgentId::Omp).unwrap()[0])
            .unwrap();
        assert_eq!(pi.agent, AgentSessionAgentId::Pi);
        assert_eq!(omp.agent, AgentSessionAgentId::Omp);
    }

    #[test]
    fn opencode_sqlite_members_are_independent_and_read_only() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(".local/share/opencode");
        fs::create_dir_all(&root).unwrap();
        let db = root.join("opencode.db");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE session(id TEXT PRIMARY KEY, title TEXT, directory TEXT, time_created INTEGER, time_updated INTEGER); CREATE TABLE message(id TEXT PRIMARY KEY, session_id TEXT, data TEXT, time_created INTEGER); CREATE TABLE part(id TEXT PRIMARY KEY, session_id TEXT, message_id TEXT, data TEXT, time_created INTEGER);").unwrap();
        for id in ["s1", "s2"] {
            conn.execute(
                "INSERT INTO session VALUES(?1, ?2, '/work', 1000, 2000)",
                params![id, format!("Session {id}")],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO message VALUES(?1, ?2, '{\"role\":\"user\"}', 1000)",
                params![format!("m-{id}"), id],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO part VALUES(?1, ?2, ?3, '{\"text\":\"hello\"}', 1000)",
                params![format!("p-{id}"), id, format!("m-{id}")],
            )
            .unwrap();
        }
        drop(conn);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let sources = registry.discover(AgentSessionAgentId::OpenCode).unwrap();
        assert_eq!(sources.len(), 2);
        assert_ne!(sources[0].member_id, sources[1].member_id);
        assert_eq!(
            registry.parse_summary(&sources[0]).unwrap().message_count,
            1
        );
    }

    #[test]
    fn opencode_storage_bundle_reads_session_message_and_part_files() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(".local/share/opencode/storage");
        write(
            &root.join("session/project/s1.json"),
            "{\"id\":\"s1\",\"title\":\"Storage session\",\"directory\":\"/work\"}",
        );
        write(
            &root.join("message/s1/m1.json"),
            "{\"id\":\"m1\",\"role\":\"user\",\"time\":{\"created\":1000}}",
        );
        write(
            &root.join("part/m1/p1.json"),
            "{\"id\":\"p1\",\"text\":\"hello from storage\"}",
        );
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::OpenCode)
            .unwrap()
            .remove(0);
        assert_eq!(source.variant, "opencode-storage");
        let summary = registry.parse_summary(&source).unwrap();
        assert_eq!(summary.title.as_deref(), Some("Storage session"));
        assert_eq!(summary.message_count, 1);
        let page = registry.read_message_page(&source, None, 1).unwrap();
        assert_eq!(page.messages[0].content, "hello from storage");
    }

    #[test]
    fn unchanged_opencode_container_gate_skips_member_enumeration() {
        let temp = TempDir::new().unwrap();
        let db = temp.path().join(".local/share/opencode/opencode.db");
        write(&db, "not a sqlite database");
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        assert!(registry.discover(AgentSessionAgentId::OpenCode).is_err());

        let container = registry
            .shared_sqlite_containers(AgentSessionAgentId::OpenCode)
            .remove(0);
        let skipped = HashSet::from([container.physical_path]);
        assert!(
            registry
                .discover_for_refresh(AgentSessionAgentId::OpenCode, &skipped)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn antigravity_encrypted_source_reports_locked_or_partial_without_key() {
        let temp = TempDir::new().unwrap();
        let pb = temp
            .path()
            .join(".gemini/antigravity-cli/conversations/a1.pb");
        write(&pb, "encrypted");
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::Antigravity)
            .unwrap()
            .remove(0);
        let page = registry
            .read_message_page(&source, None, DEFAULT_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(page.fidelity, AgentSessionFidelity::Locked);
        write(
            &temp.path().join(".gemini/antigravity-cli/brain/a1/plan.md"),
            "fallback plan",
        );
        let page = registry
            .read_message_page(&source, None, DEFAULT_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(page.fidelity, AgentSessionFidelity::Partial);
    }

    #[test]
    fn antigravity_sqlite_source_reads_plaintext_messages() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(".gemini/antigravity/conversations");
        fs::create_dir_all(&root).unwrap();
        let db = root.join("a1.db");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE messages(role TEXT, content TEXT); INSERT INTO messages VALUES('user', 'question'); INSERT INTO messages VALUES('assistant', 'answer');").unwrap();
        drop(conn);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::Antigravity)
            .unwrap()
            .remove(0);
        let summary = registry.parse_summary(&source).unwrap();
        assert_eq!(summary.message_count, 2);
        assert_eq!(summary.fidelity, AgentSessionFidelity::Full);
        let status = registry
            .provider_statuses()
            .into_iter()
            .find(|status| status.agent == AgentSessionAgentId::Antigravity)
            .unwrap();
        assert_eq!(status.fidelity, Some(AgentSessionFidelity::Full));
    }

    #[test]
    fn summary_counts_all_messages_beyond_detail_page_limit() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join(".pi/agent/sessions/project/large.jsonl");
        let mut transcript = String::new();
        for index in 0..250 {
            transcript.push_str(&format!("{{\"type\":\"message\",\"message\":{{\"role\":\"user\",\"content\":\"message {index}\"}}}}\n"));
        }
        write(&path, &transcript);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::Pi)
            .unwrap()
            .remove(0);
        let summary = registry.parse_summary(&source).unwrap();
        let page = registry
            .read_message_page(&source, None, MAX_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(summary.message_count, 250);
        assert_eq!(page.messages.len(), MAX_MESSAGE_LIMIT);
        assert!(page.has_older);
        assert_eq!(
            page.messages.first().map(|message| message.ordinal),
            Some(50)
        );
        assert_eq!(page.next_before, Some(50));
        let older = registry
            .read_message_page(&source, page.next_before, MAX_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(older.messages.len(), 50);
        assert_eq!(
            older.messages.first().map(|message| message.ordinal),
            Some(0)
        );
        assert!(!older.has_older);
        assert!(
            page.messages
                .iter()
                .all(|message| older.messages.iter().all(|item| item.key != message.key))
        );
    }

    #[test]
    fn opencode_sqlite_detail_uses_bounded_stable_pages_for_long_sessions() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(".local/share/opencode");
        fs::create_dir_all(&root).unwrap();
        let db = root.join("opencode.db");
        let mut conn = Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE session(id TEXT PRIMARY KEY, title TEXT, directory TEXT, time_created INTEGER, time_updated INTEGER); CREATE TABLE message(id TEXT PRIMARY KEY, session_id TEXT, data TEXT, time_created INTEGER); CREATE TABLE part(id TEXT PRIMARY KEY, session_id TEXT, message_id TEXT, data TEXT, time_created INTEGER);").unwrap();
        let tx = conn.transaction().unwrap();
        tx.execute(
            "INSERT INTO session VALUES('long', 'Long session', '/work', 0, 449)",
            [],
        )
        .unwrap();
        for index in 0..450 {
            let message_id = format!("m-{index:04}");
            tx.execute(
                "INSERT INTO message VALUES(?1, 'long', '{\"role\":\"assistant\"}', ?2)",
                params![message_id, index],
            )
            .unwrap();
            let content = if index == 449 {
                "x".repeat(MAX_MESSAGE_BYTES + 100)
            } else {
                format!("message {index}")
            };
            tx.execute(
                "INSERT INTO part VALUES(?1, 'long', ?2, ?3, ?4)",
                params![
                    format!("p-{index:04}"),
                    message_id,
                    serde_json::json!({ "text": content }).to_string(),
                    index
                ],
            )
            .unwrap();
        }
        tx.commit().unwrap();
        drop(conn);

        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::OpenCode)
            .unwrap()
            .remove(0);
        let latest = registry
            .read_message_page(&source, None, MAX_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(latest.messages.len(), MAX_MESSAGE_LIMIT);
        assert_eq!(latest.messages.first().unwrap().ordinal, 250);
        assert_eq!(latest.messages.last().unwrap().ordinal, 449);
        assert_eq!(latest.next_before, Some(250));
        assert!(latest.messages.last().unwrap().clipped);
        assert_eq!(
            latest.messages.last().unwrap().content.len(),
            MAX_MESSAGE_BYTES
        );

        let older = registry
            .read_message_page(&source, latest.next_before, MAX_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(older.messages.first().unwrap().ordinal, 50);
        assert_eq!(older.messages.last().unwrap().ordinal, 249);
        assert_eq!(older.next_before, Some(50));
        assert!(
            latest
                .messages
                .iter()
                .all(|message| older.messages.iter().all(|item| item.key != message.key))
        );
    }

    #[test]
    fn antigravity_sqlite_detail_uses_bounded_stable_pages_for_long_sessions() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().join(".gemini/antigravity/conversations");
        fs::create_dir_all(&root).unwrap();
        let db = root.join("long.db");
        let mut conn = Connection::open(&db).unwrap();
        conn.execute_batch("CREATE TABLE messages(role TEXT, content TEXT);")
            .unwrap();
        let tx = conn.transaction().unwrap();
        for index in 0..450 {
            let content = if index == 449 {
                "测".repeat(MAX_MESSAGE_BYTES)
            } else {
                format!("message {index}")
            };
            tx.execute("INSERT INTO messages VALUES('assistant', ?1)", [content])
                .unwrap();
        }
        tx.commit().unwrap();
        drop(conn);

        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::Antigravity)
            .unwrap()
            .remove(0);
        let latest = registry
            .read_message_page(&source, None, MAX_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(latest.messages.len(), MAX_MESSAGE_LIMIT);
        assert_eq!(latest.messages.first().unwrap().ordinal, 250);
        assert_eq!(latest.messages.last().unwrap().ordinal, 449);
        assert_eq!(latest.next_before, Some(250));
        assert!(latest.messages.last().unwrap().clipped);
        assert!(latest.messages.last().unwrap().content.len() <= MAX_MESSAGE_BYTES);

        let older = registry
            .read_message_page(&source, latest.next_before, MAX_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(older.messages.first().unwrap().ordinal, 50);
        assert_eq!(older.messages.last().unwrap().ordinal, 249);
        assert_eq!(older.next_before, Some(50));
        assert!(
            latest
                .messages
                .iter()
                .all(|message| older.messages.iter().all(|item| item.key != message.key))
        );
    }

    #[test]
    fn message_clipping_preserves_utf8_boundary() {
        let value = "测".repeat(MAX_MESSAGE_BYTES);
        let (clipped, was_clipped) = clip_utf8(&value, MAX_MESSAGE_BYTES);
        assert!(was_clipped);
        assert!(clipped.is_char_boundary(clipped.len()));
    }

    #[test]
    fn malformed_jsonl_is_partial_and_tool_blocks_are_structured() {
        let temp = TempDir::new().unwrap();
        write(
            &temp.path().join(".claude/projects/demo/session.jsonl"),
            "{\"type\":\"assistant\",\"message\":{\"role\":\"assistant\",\"content\":[{\"type\":\"tool_use\",\"name\":\"Read\",\"input\":{\"path\":\"secret\"}}]}}\n{truncated\n",
        );
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf());
        let source = registry
            .discover(AgentSessionAgentId::Claude)
            .unwrap()
            .remove(0);
        let summary = registry.parse_summary(&source).unwrap();
        let page = registry
            .read_message_page(&source, None, DEFAULT_MESSAGE_LIMIT)
            .unwrap();
        assert_eq!(summary.tool_use_count, 1);
        assert_eq!(summary.fidelity, AgentSessionFidelity::Partial);
        assert_eq!(page.messages[0].tool_name.as_deref(), Some("Read"));
        assert!(!page.messages[0].content.contains("secret"));
    }
}
