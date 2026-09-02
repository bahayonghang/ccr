use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use ccr_db::database::DbPool;
use ccr_db::database::repositories::usage_repo::{
    self, AgentSessionArchiveQuery, AgentSessionArchiveRow, AgentSessionArchiveSource,
    AgentSessionSourceState, UsageSessionArchiveEntry, UsageSourceState,
};
use ccr_store::sessions::{
    AgentSessionAgentId, AgentSessionFidelity, AgentSessionMessage, AgentSessionProviderRegistry,
    AgentSessionSourceKind, AgentSessionSourceRef, ProviderAvailability,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

const DEFAULT_LIST_LIMIT: usize = 80;
const DEFAULT_DETAIL_LIMIT: usize = 100;
const MAX_QUERY_CHARS: usize = 200;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub enum AgentSessionAgentDto {
    Grok,
    Claude,
    Codex,
    OpenCode,
    Pi,
    Omp,
    Antigravity,
    Kimi,
}

impl AgentSessionAgentDto {
    fn into_store(self) -> AgentSessionAgentId {
        match self {
            Self::Grok => AgentSessionAgentId::Grok,
            Self::Claude => AgentSessionAgentId::Claude,
            Self::Codex => AgentSessionAgentId::Codex,
            Self::OpenCode => AgentSessionAgentId::OpenCode,
            Self::Pi => AgentSessionAgentId::Pi,
            Self::Omp => AgentSessionAgentId::Omp,
            Self::Antigravity => AgentSessionAgentId::Antigravity,
            Self::Kimi => AgentSessionAgentId::Kimi,
        }
    }
}

impl From<AgentSessionAgentId> for AgentSessionAgentDto {
    fn from(value: AgentSessionAgentId) -> Self {
        match value {
            AgentSessionAgentId::Grok => Self::Grok,
            AgentSessionAgentId::Claude => Self::Claude,
            AgentSessionAgentId::Codex => Self::Codex,
            AgentSessionAgentId::OpenCode => Self::OpenCode,
            AgentSessionAgentId::Pi => Self::Pi,
            AgentSessionAgentId::Omp => Self::Omp,
            AgentSessionAgentId::Antigravity => Self::Antigravity,
            AgentSessionAgentId::Kimi => Self::Kimi,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub enum AgentSessionAvailabilityDto {
    NotInstalled,
    NoData,
    Available,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub enum AgentSessionFidelityDto {
    Full,
    Partial,
    Locked,
}

impl From<AgentSessionFidelity> for AgentSessionFidelityDto {
    fn from(value: AgentSessionFidelity) -> Self {
        match value {
            AgentSessionFidelity::Full => Self::Full,
            AgentSessionFidelity::Partial => Self::Partial,
            AgentSessionFidelity::Locked => Self::Locked,
        }
    }
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionListRequestDto {
    #[serde(default)]
    #[ts(optional)]
    pub agents: Option<Vec<AgentSessionAgentDto>>,
    #[serde(default)]
    #[ts(optional)]
    pub query: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub cwd_prefix: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub started_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub ended_at: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub source_state: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub fidelity: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub cursor: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionListItemDto {
    pub archive_id: String,
    pub session_id: String,
    pub agent: AgentSessionAgentDto,
    pub variant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub title: Option<String>,
    pub cwd: String,
    #[ts(as = "f64")]
    pub message_count: i64,
    #[ts(as = "f64")]
    pub user_message_count: i64,
    #[ts(as = "f64")]
    pub assistant_message_count: i64,
    #[ts(as = "f64")]
    pub tool_use_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub source_state: String,
    pub fidelity: AgentSessionFidelityDto,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionPageDto {
    pub items: Vec<AgentSessionListItemDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionDetailRequestDto {
    pub archive_id: String,
    #[serde(default)]
    #[ts(optional)]
    pub before_cursor: Option<u32>,
    #[serde(default)]
    #[ts(optional)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionMessageDto {
    pub key: String,
    pub ordinal: u32,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timestamp: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub tool_name: Option<String>,
    pub clipped: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionDetailDto {
    pub archive_id: String,
    pub agent: AgentSessionAgentDto,
    pub variant: String,
    pub fidelity: AgentSessionFidelityDto,
    pub messages: Vec<AgentSessionMessageDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub next_before: Option<u32>,
    pub has_older: bool,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionProviderStatusDto {
    pub agent: AgentSessionAgentDto,
    pub availability: AgentSessionAvailabilityDto,
    pub variants: Vec<String>,
    pub source_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub fidelity: Option<AgentSessionFidelityDto>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub error_category: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionRefreshCountersDto {
    #[ts(as = "f64")]
    pub discovered: u64,
    #[ts(as = "f64")]
    pub unchanged: u64,
    #[ts(as = "f64")]
    pub fingerprinted: u64,
    #[ts(as = "f64")]
    pub parsed: u64,
    #[ts(as = "f64")]
    pub upserted: u64,
    #[ts(as = "f64")]
    pub partial: u64,
    #[ts(as = "f64")]
    pub locked: u64,
    #[ts(as = "f64")]
    pub errors: u64,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionProviderRefreshDto {
    pub agent: AgentSessionAgentDto,
    pub counters: AgentSessionRefreshCountersDto,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/types/generated/agent_sessions/")]
pub struct AgentSessionRefreshReportDto {
    pub total: AgentSessionRefreshCountersDto,
    pub providers: Vec<AgentSessionProviderRefreshDto>,
}

pub fn list_sessions(
    pool: &DbPool,
    request: AgentSessionListRequestDto,
) -> Result<AgentSessionPageDto, String> {
    let query_text = normalize_text(request.query, MAX_QUERY_CHARS)?;
    let cwd_prefix = normalize_text(request.cwd_prefix, 1024)?;
    let started_at = parse_date(request.started_at)?;
    let ended_at = parse_date(request.ended_at)?;
    if started_at
        .zip(ended_at)
        .is_some_and(|(start, end)| start > end)
    {
        return Err("agent_session_invalid_date_range".into());
    }
    let (cursor_updated_at, cursor_archive_id) = decode_cursor(request.cursor.as_deref())?;
    let source_state = validate_choice(
        request.source_state,
        &["live", "missing", "deleted_by_user", "all"],
    )?;
    let source_fidelity = validate_choice(request.fidelity, &["full", "partial", "locked", "all"])?;
    let query = AgentSessionArchiveQuery {
        platforms: request
            .agents
            .filter(|agents| !agents.is_empty())
            .unwrap_or_else(|| {
                AgentSessionAgentId::ALL
                    .into_iter()
                    .map(AgentSessionAgentDto::from)
                    .collect()
            })
            .into_iter()
            .map(|agent| agent.into_store().as_str().to_string())
            .collect(),
        query: query_text,
        cwd_prefix,
        started_at: started_at.map(|value| value.to_rfc3339()),
        ended_at: ended_at.map(|value| value.to_rfc3339()),
        source_state: source_state.filter(|value| value != "all"),
        source_fidelity: source_fidelity.filter(|value| value != "all"),
        cursor_updated_at,
        cursor_archive_id,
        limit: request.limit.unwrap_or(DEFAULT_LIST_LIMIT).clamp(1, 200),
    };
    let conn = pool
        .get()
        .map_err(|error| format!("agent_session_db_unavailable:{error}"))?;
    let rows = usage_repo::get_agent_session_archive_page(&conn, &query)
        .map_err(|error| format!("agent_session_list_failed:{error}"))?;
    let next_cursor = (rows.len() == query.limit)
        .then(|| {
            rows.last()
                .map(|row| encode_cursor(row.updated_at, &row.archive_id))
        })
        .flatten();
    Ok(AgentSessionPageDto {
        items: rows.into_iter().map(row_to_dto).collect::<Result<_, _>>()?,
        next_cursor,
    })
}

pub fn get_detail(
    pool: &DbPool,
    request: AgentSessionDetailRequestDto,
) -> Result<AgentSessionDetailDto, String> {
    if request.archive_id.is_empty() || request.archive_id.len() > 128 {
        return Err("agent_session_invalid_archive_id".into());
    }
    let registry = AgentSessionProviderRegistry::from_default_home()
        .map_err(|_| "agent_session_home_unavailable".to_string())?;
    get_detail_with_registry(pool, request, &registry)
}

fn get_detail_with_registry(
    pool: &DbPool,
    request: AgentSessionDetailRequestDto,
    registry: &AgentSessionProviderRegistry,
) -> Result<AgentSessionDetailDto, String> {
    if request.archive_id.is_empty() || request.archive_id.len() > 128 {
        return Err("agent_session_invalid_archive_id".into());
    }
    let conn = pool
        .get()
        .map_err(|error| format!("agent_session_db_unavailable:{error}"))?;
    let stored = usage_repo::get_agent_session_archive_source(&conn, &request.archive_id)
        .map_err(|error| format!("agent_session_source_lookup_failed:{error}"))?
        .ok_or_else(|| "agent_session_not_found".to_string())?;
    drop(conn);
    let source = restore_source(registry, &stored)?;
    let page = registry
        .read_message_page(
            &source,
            request.before_cursor,
            request.limit.unwrap_or(DEFAULT_DETAIL_LIMIT).clamp(1, 200),
        )
        .map_err(|_| "agent_session_source_unavailable".to_string())?;
    let fidelity = combine_fidelity(
        parse_fidelity(&stored.source_fidelity)?,
        page.fidelity.into(),
    );
    Ok(AgentSessionDetailDto {
        archive_id: stored.archive_id,
        agent: source.agent.into(),
        variant: source.variant,
        fidelity,
        messages: page.messages.into_iter().map(message_to_dto).collect(),
        next_before: page.next_before,
        has_older: page.has_older,
    })
}

pub fn provider_statuses() -> Result<Vec<AgentSessionProviderStatusDto>, String> {
    let registry = AgentSessionProviderRegistry::from_default_home()
        .map_err(|_| "agent_session_home_unavailable".to_string())?;
    Ok(registry
        .provider_statuses()
        .into_iter()
        .map(|status| AgentSessionProviderStatusDto {
            agent: status.agent.into(),
            availability: match status.availability {
                ProviderAvailability::NotInstalled => AgentSessionAvailabilityDto::NotInstalled,
                ProviderAvailability::NoData => AgentSessionAvailabilityDto::NoData,
                ProviderAvailability::Available => AgentSessionAvailabilityDto::Available,
                ProviderAvailability::Error => AgentSessionAvailabilityDto::Error,
            },
            variants: status.variants,
            source_count: status.source_count,
            fidelity: status.fidelity.map(Into::into),
            error_category: status.error_category,
        })
        .collect())
}

pub fn refresh_archive(pool: &DbPool) -> Result<AgentSessionRefreshReportDto, String> {
    let registry = AgentSessionProviderRegistry::from_default_home()
        .map_err(|_| "agent_session_home_unavailable".to_string())?;
    refresh_archive_with_registry(pool, &registry)
}

fn refresh_archive_with_registry(
    pool: &DbPool,
    registry: &AgentSessionProviderRegistry,
) -> Result<AgentSessionRefreshReportDto, String> {
    let conn = pool
        .get()
        .map_err(|error| format!("agent_session_db_unavailable:{error}"))?;
    let source_states = usage_repo::get_agent_session_source_states(&conn)
        .map_err(|error| format!("agent_session_state_lookup_failed:{error}"))?;
    let archives = usage_repo::get_all_agent_session_archive_sources(&conn)
        .map_err(|error| format!("agent_session_archive_lookup_failed:{error}"))?;
    let state_map: HashMap<(String, String, String), AgentSessionSourceState> = source_states
        .into_iter()
        .map(|state| {
            (
                (
                    state.platform.clone(),
                    state.source_path.clone(),
                    state.source_kind.clone(),
                ),
                state,
            )
        })
        .collect();
    let archive_map: HashMap<(String, String, String), AgentSessionArchiveSource> = archives
        .into_iter()
        .map(|row| {
            (
                (
                    row.platform.clone(),
                    row.file_path.clone(),
                    row.source_member_id.clone(),
                ),
                row,
            )
        })
        .collect();
    drop(conn);
    let mut report = AgentSessionRefreshReportDto {
        total: AgentSessionRefreshCountersDto::default(),
        providers: Vec::new(),
    };
    let mut pending_entries = Vec::new();
    let mut pending_states = Vec::new();
    let mut seen_by_agent = HashMap::<String, Vec<(String, String)>>::new();
    let mut authoritative_agents = HashSet::<String>::new();

    for agent in AgentSessionAgentId::ALL {
        let mut counters = AgentSessionRefreshCountersDto::default();
        let mut skipped_containers = HashSet::<PathBuf>::new();
        let mut skipped_members = Vec::<(String, String)>::new();
        for container in registry.shared_sqlite_containers(agent) {
            let path = container.physical_path.to_string_lossy().to_string();
            let state_key = (
                agent.as_str().to_string(),
                path.clone(),
                container.kind.as_str().to_string(),
            );
            let Ok(container_hash) = registry.container_fingerprint(&container) else {
                continue;
            };
            if !state_map.get(&state_key).is_some_and(|state| {
                state.source_stat_hash == container_hash && state.last_error_code.is_none()
            }) {
                continue;
            }
            let archived_members = archive_map
                .keys()
                .filter(|(platform, source_path, _)| {
                    platform == agent.as_str() && source_path == &path
                })
                .map(|(_, source_path, member)| (source_path.clone(), member.clone()))
                .collect::<Vec<_>>();
            if archived_members.is_empty() {
                continue;
            }
            skipped_members.extend(archived_members);
            skipped_containers.insert(container.physical_path);
        }
        let sources = match registry.discover_for_refresh(agent, &skipped_containers) {
            Ok(sources) => {
                authoritative_agents.insert(agent.as_str().to_string());
                sources
            }
            Err(_) => {
                counters.errors = 1;
                accumulate(&mut report.total, &counters);
                report.providers.push(AgentSessionProviderRefreshDto {
                    agent: agent.into(),
                    counters,
                });
                continue;
            }
        };
        counters.discovered = (sources.len() + skipped_members.len()) as u64;
        counters.unchanged = skipped_members.len() as u64;
        seen_by_agent
            .entry(agent.as_str().to_string())
            .or_default()
            .extend(skipped_members);
        let mut groups =
            HashMap::<(PathBuf, AgentSessionSourceKind), Vec<AgentSessionSourceRef>>::new();
        for source in sources {
            seen_by_agent
                .entry(agent.as_str().to_string())
                .or_default()
                .push((
                    source.physical_path.to_string_lossy().to_string(),
                    source.member_id.clone().unwrap_or_default(),
                ));
            groups
                .entry((source.physical_path.clone(), source.kind))
                .or_default()
                .push(source);
        }
        for ((path, kind), members) in groups {
            let first = &members[0];
            let container_hash = match registry.container_fingerprint(first) {
                Ok(value) => value,
                Err(_) => {
                    counters.errors += 1;
                    continue;
                }
            };
            let state_key = (
                agent.as_str().to_string(),
                path.to_string_lossy().to_string(),
                kind.as_str().to_string(),
            );
            if state_map.get(&state_key).is_some_and(|state| {
                state.source_stat_hash == container_hash && state.last_error_code.is_none()
            }) {
                counters.unchanged += members.len() as u64;
                continue;
            }
            let errors_before = counters.errors;
            for source in &members {
                counters.fingerprinted += 1;
                let fingerprint = match registry.quick_fingerprint(source) {
                    Ok(value) => value,
                    Err(_) => {
                        counters.errors += 1;
                        continue;
                    }
                };
                let identity = (
                    agent.as_str().to_string(),
                    source.physical_path.to_string_lossy().to_string(),
                    source.member_id.clone().unwrap_or_default(),
                );
                if archive_map
                    .get(&identity)
                    .and_then(|row| row.source_stat_hash.as_deref())
                    == Some(fingerprint.as_str())
                {
                    counters.unchanged += 1;
                    continue;
                }
                counters.parsed += 1;
                match registry.parse_summary(source) {
                    Ok(summary) => {
                        match summary.fidelity {
                            AgentSessionFidelity::Partial => counters.partial += 1,
                            AgentSessionFidelity::Locked => counters.locked += 1,
                            AgentSessionFidelity::Full => {}
                        }
                        pending_entries.push(summary_to_archive(
                            summary,
                            fingerprint,
                            archive_map.get(&identity),
                        )?);
                        counters.upserted += 1;
                    }
                    Err(_) => counters.errors += 1,
                }
            }
            let group_error = counters.errors > errors_before;
            let metadata = path.metadata().ok();
            pending_states.push(AgentSessionSourceState {
                platform: agent.as_str().to_string(),
                source_path: path.to_string_lossy().to_string(),
                source_kind: kind.as_str().to_string(),
                source_size: metadata
                    .as_ref()
                    .and_then(|value| i64::try_from(value.len()).ok()),
                source_mtime_ns: metadata
                    .and_then(|value| value.modified().ok())
                    .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                    .and_then(|value| i64::try_from(value.as_nanos()).ok()),
                source_stat_hash: container_hash,
                last_success_at: (!group_error).then(Utc::now),
                last_error_code: group_error.then(|| "provider_parse_failed".to_string()),
            });
        }
        accumulate(&mut report.total, &counters);
        report.providers.push(AgentSessionProviderRefreshDto {
            agent: agent.into(),
            counters,
        });
    }

    let conn = pool
        .get()
        .map_err(|error| format!("agent_session_db_unavailable:{error}"))?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| format!("agent_session_transaction_failed:{error}"))?;
    for entry in &pending_entries {
        usage_repo::upsert_session_archive_entry(&tx, entry)
            .map_err(|error| format!("agent_session_upsert_failed:{error}"))?;
    }
    for state in &pending_states {
        usage_repo::upsert_agent_session_source_state(&tx, state)
            .map_err(|error| format!("agent_session_state_upsert_failed:{error}"))?;
    }
    for agent in AgentSessionAgentId::ALL {
        if !authoritative_agents.contains(agent.as_str()) {
            continue;
        }
        usage_repo::mark_agent_session_archive_missing_by_identity(
            &tx,
            agent.as_str(),
            seen_by_agent
                .get(agent.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]),
        )
        .map_err(|error| format!("agent_session_reconcile_failed:{error}"))?;
    }
    tx.commit()
        .map_err(|error| format!("agent_session_transaction_failed:{error}"))?;
    Ok(report)
}

fn summary_to_archive(
    summary: ccr_store::sessions::AgentSessionSummary,
    fingerprint: String,
    existing: Option<&AgentSessionArchiveSource>,
) -> Result<UsageSessionArchiveEntry, String> {
    let metadata = summary.source.physical_path.metadata().ok();
    Ok(UsageSessionArchiveEntry {
        archive_id: existing
            .map(|row| row.archive_id.clone())
            .unwrap_or_else(|| {
                usage_repo::agent_session_archive_id(
                    summary.agent.as_str(),
                    &summary.source.physical_path.to_string_lossy(),
                    summary.source.member_id.as_deref().unwrap_or_default(),
                )
            }),
        session_id: summary.native_session_id,
        platform: summary.agent.as_str().to_string(),
        title: summary.title,
        cwd: summary.cwd,
        file_path: summary.source.physical_path.to_string_lossy().to_string(),
        file_hash: Some(fingerprint.clone()),
        source_variant: summary.variant,
        source_kind: summary.source.kind.as_str().to_string(),
        source_member_id: summary.source.member_id.unwrap_or_default(),
        source_size: metadata
            .as_ref()
            .and_then(|value| i64::try_from(value.len()).ok()),
        source_mtime_ns: metadata
            .and_then(|value| value.modified().ok())
            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
            .and_then(|value| i64::try_from(value.as_nanos()).ok()),
        source_stat_hash: Some(fingerprint),
        message_count: i64::from(summary.message_count),
        user_message_count: i64::from(summary.user_message_count),
        assistant_message_count: i64::from(summary.assistant_message_count),
        tool_use_count: i64::from(summary.tool_use_count),
        source_fidelity: summary.fidelity.as_str().to_string(),
        created_at: summary.created_at,
        updated_at: summary.updated_at,
        source_state: UsageSourceState::Live,
        last_seen_at: Some(Utc::now()),
        raw_deleted_at: None,
        archived_at: Utc::now(),
    })
}

fn restore_source(
    registry: &AgentSessionProviderRegistry,
    stored: &AgentSessionArchiveSource,
) -> Result<AgentSessionSourceRef, String> {
    let agent = AgentSessionAgentId::parse(&stored.platform)
        .ok_or_else(|| "agent_session_unknown_provider".to_string())?;
    let kind = AgentSessionSourceKind::parse(&stored.source_kind)
        .ok_or_else(|| "agent_session_unknown_source_kind".to_string())?;
    registry
        .restore_source(
            agent,
            &stored.source_variant,
            PathBuf::from(&stored.file_path),
            kind,
            (!stored.source_member_id.is_empty()).then(|| stored.source_member_id.clone()),
        )
        .map_err(map_restore_source_error)
}

fn map_restore_source_error(error: impl std::fmt::Display) -> String {
    // 只回传稳定错误码，不把路径或 CcrError 英文原文交给渲染器。
    let message = error.to_string();
    if message.contains("session source is missing")
        || message.contains("session source root is unavailable")
        || message.contains("session member is missing")
    {
        "agent_session_source_unavailable".to_string()
    } else {
        "agent_session_source_validation_failed".to_string()
    }
}

fn row_to_dto(row: AgentSessionArchiveRow) -> Result<AgentSessionListItemDto, String> {
    let agent = AgentSessionAgentId::parse(&row.platform)
        .ok_or_else(|| "agent_session_unknown_provider".to_string())?;
    let fidelity = parse_fidelity(&row.source_fidelity)?;
    Ok(AgentSessionListItemDto {
        archive_id: row.archive_id,
        session_id: row.session_id,
        agent: agent.into(),
        variant: row.source_variant,
        title: row.title,
        cwd: row.cwd,
        message_count: row.message_count,
        user_message_count: row.user_message_count,
        assistant_message_count: row.assistant_message_count,
        tool_use_count: row.tool_use_count,
        created_at: row.created_at.to_rfc3339(),
        updated_at: row.updated_at.to_rfc3339(),
        source_state: row.source_state.as_str().to_string(),
        fidelity,
    })
}

fn message_to_dto(message: AgentSessionMessage) -> AgentSessionMessageDto {
    AgentSessionMessageDto {
        key: message.key,
        ordinal: message.ordinal,
        role: message.role,
        timestamp: message.timestamp.map(|value| value.to_rfc3339()),
        content: message.content,
        tool_name: message.tool_name,
        clipped: message.clipped,
    }
}

fn parse_fidelity(value: &str) -> Result<AgentSessionFidelityDto, String> {
    match value {
        "full" => Ok(AgentSessionFidelityDto::Full),
        "partial" => Ok(AgentSessionFidelityDto::Partial),
        "locked" => Ok(AgentSessionFidelityDto::Locked),
        _ => Err("agent_session_unknown_fidelity".into()),
    }
}

fn combine_fidelity(
    archived: AgentSessionFidelityDto,
    current: AgentSessionFidelityDto,
) -> AgentSessionFidelityDto {
    match (archived, current) {
        (AgentSessionFidelityDto::Locked, _) | (_, AgentSessionFidelityDto::Locked) => {
            AgentSessionFidelityDto::Locked
        }
        (AgentSessionFidelityDto::Partial, _) | (_, AgentSessionFidelityDto::Partial) => {
            AgentSessionFidelityDto::Partial
        }
        _ => AgentSessionFidelityDto::Full,
    }
}

fn normalize_text(value: Option<String>, max_chars: usize) -> Result<Option<String>, String> {
    let value = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if value
        .as_ref()
        .is_some_and(|value| value.chars().count() > max_chars)
    {
        return Err("agent_session_query_too_long".into());
    }
    Ok(value)
}

fn parse_date(value: Option<String>) -> Result<Option<DateTime<Utc>>, String> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|_| "agent_session_invalid_date".to_string())
        })
        .transpose()
}

fn validate_choice(value: Option<String>, allowed: &[&str]) -> Result<Option<String>, String> {
    let value = value
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());
    if value
        .as_ref()
        .is_some_and(|value| !allowed.contains(&value.as_str()))
    {
        return Err("agent_session_invalid_filter".into());
    }
    Ok(value)
}

fn encode_cursor(updated_at: DateTime<Utc>, archive_id: &str) -> String {
    format!("{}|{archive_id}", updated_at.to_rfc3339())
}

fn decode_cursor(cursor: Option<&str>) -> Result<(Option<String>, Option<String>), String> {
    let Some(cursor) = cursor else {
        return Ok((None, None));
    };
    if cursor.len() > 256 {
        return Err("agent_session_invalid_cursor".into());
    }
    let (updated_at, archive_id) = cursor
        .split_once('|')
        .ok_or_else(|| "agent_session_invalid_cursor".to_string())?;
    let updated_at = DateTime::parse_from_rfc3339(updated_at)
        .map_err(|_| "agent_session_invalid_cursor".to_string())?
        .with_timezone(&Utc)
        .to_rfc3339();
    if archive_id.is_empty() || archive_id.len() > 128 {
        return Err("agent_session_invalid_cursor".into());
    }
    Ok((Some(updated_at), Some(archive_id.to_string())))
}

fn accumulate(total: &mut AgentSessionRefreshCountersDto, item: &AgentSessionRefreshCountersDto) {
    total.discovered += item.discovered;
    total.unchanged += item.unchanged;
    total.fingerprinted += item.fingerprinted;
    total.parsed += item.parsed;
    total.upserted += item.upserted;
    total.partial += item.partial;
    total.locked += item.locked;
    total.errors += item.errors;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn temp_usage_pool(temp: &TempDir) -> DbPool {
        let pool = ccr_db::database::create_pool(&temp.path().join("usage.db"), None).unwrap();
        let conn = pool.get().unwrap();
        ccr_db::database::migrations::run_all_migrations(&conn, temp.path()).unwrap();
        drop(conn);
        pool
    }

    fn empty_list_request() -> AgentSessionListRequestDto {
        AgentSessionListRequestDto {
            agents: None,
            query: None,
            cwd_prefix: None,
            started_at: None,
            ended_at: None,
            source_state: None,
            fidelity: None,
            cursor: None,
            limit: None,
        }
    }

    #[test]
    fn cursor_round_trip_and_limit_validation() {
        let now = Utc::now();
        let encoded = encode_cursor(now, "as-1234");
        let decoded = decode_cursor(Some(&encoded)).unwrap();
        assert_eq!(decoded.1.as_deref(), Some("as-1234"));
        assert!(decode_cursor(Some("bad")).is_err());
    }

    #[test]
    fn all_eight_agents_map_to_store_ids() {
        let values = [
            AgentSessionAgentDto::Grok,
            AgentSessionAgentDto::Claude,
            AgentSessionAgentDto::Codex,
            AgentSessionAgentDto::OpenCode,
            AgentSessionAgentDto::Pi,
            AgentSessionAgentDto::Omp,
            AgentSessionAgentDto::Antigravity,
            AgentSessionAgentDto::Kimi,
        ];
        assert_eq!(
            values.map(AgentSessionAgentDto::into_store),
            AgentSessionAgentId::ALL
        );
    }

    #[test]
    fn detail_fidelity_never_overstates_an_archived_degradation() {
        assert_eq!(
            combine_fidelity(
                AgentSessionFidelityDto::Partial,
                AgentSessionFidelityDto::Full,
            ),
            AgentSessionFidelityDto::Partial
        );
        assert_eq!(
            combine_fidelity(
                AgentSessionFidelityDto::Full,
                AgentSessionFidelityDto::Locked,
            ),
            AgentSessionFidelityDto::Locked
        );
    }

    #[test]
    fn unchanged_refresh_skips_parse_and_upsert_work() {
        let temp = TempDir::new().unwrap();
        let transcript = temp.path().join(".pi/agent/sessions/project/s1.jsonl");
        fs::create_dir_all(transcript.parent().unwrap()).unwrap();
        fs::write(
            &transcript,
            "{\"type\":\"message\",\"message\":{\"role\":\"user\",\"content\":\"hello\"}}\n",
        )
        .unwrap();
        let pool = temp_usage_pool(&temp);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
            .without_environment_overrides();

        let first = refresh_archive_with_registry(&pool, &registry).unwrap();
        assert_eq!(first.total.parsed, 1);
        assert_eq!(first.total.upserted, 1);
        let second = refresh_archive_with_registry(&pool, &registry).unwrap();
        assert_eq!(second.total.discovered, 1);
        assert_eq!(second.total.unchanged, 1);
        assert_eq!(second.total.parsed, 0);
        assert_eq!(second.total.upserted, 0);
    }

    #[test]
    fn default_list_excludes_legacy_non_target_platform_rows() {
        let temp = TempDir::new().unwrap();
        let pool = temp_usage_pool(&temp);
        let now = Utc::now();
        let entry = UsageSessionArchiveEntry {
            archive_id: usage_repo::agent_session_archive_id("gemini", "C:/private/source", ""),
            session_id: "legacy-gemini".into(),
            platform: "gemini".into(),
            title: None,
            cwd: String::new(),
            file_path: "C:/private/source".into(),
            file_hash: None,
            source_variant: "gemini-jsonl".into(),
            source_kind: "file".into(),
            source_member_id: String::new(),
            source_size: None,
            source_mtime_ns: None,
            source_stat_hash: None,
            message_count: 0,
            user_message_count: 0,
            assistant_message_count: 0,
            tool_use_count: 0,
            source_fidelity: "full".into(),
            created_at: now,
            updated_at: now,
            source_state: UsageSourceState::Live,
            last_seen_at: Some(now),
            raw_deleted_at: None,
            archived_at: now,
        };
        let conn = pool.get().unwrap();
        usage_repo::upsert_session_archive_entry(&conn, &entry).unwrap();
        drop(conn);
        let page = list_sessions(&pool, empty_list_request()).unwrap();
        assert!(page.items.is_empty());
    }

    fn write_codex_jsonl(home: &std::path::Path, relative: &str) -> std::path::PathBuf {
        let path = home.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "{\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"hello\"}]}}\n",
        )
        .unwrap();
        path
    }

    fn detail_request(archive_id: &str) -> AgentSessionDetailRequestDto {
        AgentSessionDetailRequestDto {
            archive_id: archive_id.to_string(),
            before_cursor: None,
            limit: None,
        }
    }

    fn assert_stable_error(error: &str, expected: &str) {
        assert_eq!(error, expected);
        assert!(!error.contains("session source"));
        assert!(!error.contains('\\') && !error.contains('/'));
    }

    #[test]
    fn get_detail_maps_missing_jsonl_to_source_unavailable() {
        let temp = TempDir::new().unwrap();
        let live = write_codex_jsonl(temp.path(), ".codex/sessions/2026/09/02/rollout-live.jsonl");
        let pool = temp_usage_pool(&temp);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
            .without_environment_overrides();
        refresh_archive_with_registry(&pool, &registry).unwrap();
        let page = list_sessions(&pool, empty_list_request()).unwrap();
        let archive_id = page.items[0].archive_id.clone();
        fs::remove_file(&live).unwrap();

        let error =
            get_detail_with_registry(&pool, detail_request(&archive_id), &registry).unwrap_err();
        assert_stable_error(&error, "agent_session_source_unavailable");
    }

    #[test]
    fn get_detail_maps_wrong_extension_to_validation_failed() {
        let temp = TempDir::new().unwrap();
        let txt = temp.path().join(".codex/sessions/2026/09/02/evil.txt");
        fs::create_dir_all(txt.parent().unwrap()).unwrap();
        fs::write(&txt, "not a jsonl session").unwrap();
        let pool = temp_usage_pool(&temp);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
            .without_environment_overrides();
        let now = Utc::now();
        let file_path = txt.to_string_lossy().to_string();
        let entry = UsageSessionArchiveEntry {
            archive_id: usage_repo::agent_session_archive_id("codex", &file_path, ""),
            session_id: "evil".into(),
            platform: "codex".into(),
            title: None,
            cwd: String::new(),
            file_path: file_path.clone(),
            file_hash: None,
            source_variant: "codex-live".into(),
            source_kind: "file".into(),
            source_member_id: String::new(),
            source_size: None,
            source_mtime_ns: None,
            source_stat_hash: None,
            message_count: 0,
            user_message_count: 0,
            assistant_message_count: 0,
            tool_use_count: 0,
            source_fidelity: "full".into(),
            created_at: now,
            updated_at: now,
            source_state: UsageSourceState::Live,
            last_seen_at: Some(now),
            raw_deleted_at: None,
            archived_at: now,
        };
        let conn = pool.get().unwrap();
        usage_repo::upsert_session_archive_entry(&conn, &entry).unwrap();
        drop(conn);

        let error = get_detail_with_registry(&pool, detail_request(&entry.archive_id), &registry)
            .unwrap_err();
        assert_stable_error(&error, "agent_session_source_validation_failed");
        assert!(!error.contains(&file_path));
    }

    #[test]
    fn get_detail_maps_escaped_path_to_validation_failed() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join(".codex/sessions")).unwrap();
        let outside = temp.path().join("outside.jsonl");
        fs::write(&outside, "{\"role\":\"user\"}\n").unwrap();
        let pool = temp_usage_pool(&temp);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
            .without_environment_overrides();
        let now = Utc::now();
        let file_path = outside.to_string_lossy().to_string();
        let entry = UsageSessionArchiveEntry {
            archive_id: usage_repo::agent_session_archive_id("codex", &file_path, ""),
            session_id: "escaped".into(),
            platform: "codex".into(),
            title: None,
            cwd: String::new(),
            file_path: file_path.clone(),
            file_hash: None,
            source_variant: "codex-live".into(),
            source_kind: "file".into(),
            source_member_id: String::new(),
            source_size: None,
            source_mtime_ns: None,
            source_stat_hash: None,
            message_count: 0,
            user_message_count: 0,
            assistant_message_count: 0,
            tool_use_count: 0,
            source_fidelity: "full".into(),
            created_at: now,
            updated_at: now,
            source_state: UsageSourceState::Live,
            last_seen_at: Some(now),
            raw_deleted_at: None,
            archived_at: now,
        };
        let conn = pool.get().unwrap();
        usage_repo::upsert_session_archive_entry(&conn, &entry).unwrap();
        drop(conn);

        let error = get_detail_with_registry(&pool, detail_request(&entry.archive_id), &registry)
            .unwrap_err();
        assert_stable_error(&error, "agent_session_source_validation_failed");
        assert!(!error.contains(&file_path));
    }

    #[test]
    fn refresh_upserts_live_jsonl_and_marks_deleted_archive_missing() {
        let temp = TempDir::new().unwrap();
        write_codex_jsonl(temp.path(), ".codex/sessions/2026/09/02/rollout-live.jsonl");
        let old = write_codex_jsonl(temp.path(), ".codex/sessions/2026/03/06/rollout-old.jsonl");
        let pool = temp_usage_pool(&temp);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
            .without_environment_overrides();
        refresh_archive_with_registry(&pool, &registry).unwrap();
        fs::remove_file(&old).unwrap();
        refresh_archive_with_registry(&pool, &registry).unwrap();

        let page = list_sessions(&pool, empty_list_request()).unwrap();
        let live_item = page
            .items
            .iter()
            .find(|item| item.session_id == "rollout-live")
            .unwrap();
        let old_item = page
            .items
            .iter()
            .find(|item| item.session_id == "rollout-old")
            .unwrap();
        assert_eq!(live_item.source_state, "live");
        assert_eq!(old_item.source_state, "missing");

        let detail =
            get_detail_with_registry(&pool, detail_request(&live_item.archive_id), &registry)
                .unwrap();
        assert_eq!(detail.archive_id, live_item.archive_id);
        assert!(!detail.messages.is_empty());
    }
}
