use ccr_db::database::DbPool;
use ccr_db::database::repositories::usage_repo::AgentSessionArchiveRow;
#[cfg(test)]
use ccr_db::database::repositories::usage_repo::{
    self, UsageSessionArchiveEntry, UsageSourceState,
};
use ccr_store::sessions::{
    AgentSessionAgentId, AgentSessionFidelity, AgentSessionMessage, AgentSessionProviderRegistry,
    ProviderAvailability,
};
#[cfg(test)]
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

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
    let page = ccr_app::sessions::list_sessions(
        pool,
        ccr_app::sessions::SessionListRequest {
            agents: request.agents.map(|agents| {
                agents
                    .into_iter()
                    .map(AgentSessionAgentDto::into_store)
                    .collect()
            }),
            query: request.query,
            cwd_prefix: request.cwd_prefix,
            started_at: request.started_at,
            ended_at: request.ended_at,
            source_state: request.source_state,
            fidelity: request.fidelity,
            cursor: request.cursor,
            limit: request.limit,
        },
    )
    .map_err(|error| error.code().to_string())?;
    Ok(AgentSessionPageDto {
        items: page
            .items
            .into_iter()
            .map(row_to_dto)
            .collect::<Result<_, _>>()?,
        next_cursor: page.next_cursor,
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
    let detail = ccr_app::sessions::get_detail_with_registry(
        pool,
        registry,
        &request.archive_id,
        request.before_cursor,
        request.limit,
    )
    .map_err(|error| error.code().to_string())?;
    Ok(AgentSessionDetailDto {
        archive_id: detail.archive_id,
        agent: detail.agent.into(),
        variant: detail.variant,
        fidelity: detail.page.fidelity.into(),
        messages: detail
            .page
            .messages
            .into_iter()
            .map(message_to_dto)
            .collect(),
        next_before: detail.page.next_before,
        has_older: detail.page.has_older,
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
    let report = ccr_app::sessions::refresh::refresh_archive_with_registry(pool, registry)?;
    Ok(AgentSessionRefreshReportDto {
        total: refresh_counters(report.total),
        providers: report
            .providers
            .into_iter()
            .map(|provider| AgentSessionProviderRefreshDto {
                agent: provider.agent.into(),
                counters: refresh_counters(provider.counters),
            })
            .collect(),
    })
}
fn refresh_counters(
    value: ccr_app::sessions::refresh::RefreshCounters,
) -> AgentSessionRefreshCountersDto {
    AgentSessionRefreshCountersDto {
        discovered: value.discovered,
        unchanged: value.unchanged,
        fingerprinted: value.fingerprinted,
        parsed: value.parsed,
        upserted: value.upserted,
        partial: value.partial,
        locked: value.locked,
        errors: value.errors,
    }
}

fn row_to_dto(row: AgentSessionArchiveRow) -> Result<AgentSessionListItemDto, String> {
    let agent = AgentSessionAgentId::parse(&row.platform)
        .ok_or_else(|| "agent_session_unknown_provider".to_string())?;
    let fidelity = ccr_app::sessions::parse_fidelity(&row.source_fidelity)
        .map_err(|error| error.code().to_string())?
        .into();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    use ccr_app::sessions::{combine_fidelity, decode_cursor, encode_cursor};
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
            combine_fidelity(AgentSessionFidelity::Partial, AgentSessionFidelity::Full,),
            AgentSessionFidelity::Partial
        );
        assert_eq!(
            combine_fidelity(AgentSessionFidelity::Full, AgentSessionFidelity::Locked,),
            AgentSessionFidelity::Locked
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

    #[test]
    fn tauri_archive_reads_match_shared_explicit_registry_policy() {
        let temp = TempDir::new().unwrap();
        write_codex_jsonl(temp.path(), ".codex/sessions/2026/09/24/rollout-one.jsonl");
        write_codex_jsonl(temp.path(), ".codex/sessions/2026/09/24/rollout-two.jsonl");
        let pool = temp_usage_pool(&temp);
        let registry = AgentSessionProviderRegistry::new(temp.path().to_path_buf())
            .without_environment_overrides();
        refresh_archive_with_registry(&pool, &registry).unwrap();
        let mut request = empty_list_request();
        request.agents = Some(vec![AgentSessionAgentDto::Codex]);
        request.limit = Some(1);
        let wire = list_sessions(&pool, request).unwrap();
        let shared = ccr_app::sessions::list_sessions(
            &pool,
            ccr_app::sessions::SessionListRequest {
                agents: Some(vec![AgentSessionAgentId::Codex]),
                limit: Some(1),
                ..Default::default()
            },
        )
        .unwrap();
        let mapped = AgentSessionPageDto {
            items: shared
                .items
                .into_iter()
                .map(row_to_dto)
                .collect::<Result<_, _>>()
                .unwrap(),
            next_cursor: shared.next_cursor,
        };
        assert_eq!(
            serde_json::to_value(&wire).unwrap(),
            serde_json::to_value(mapped).unwrap()
        );
        let archive_id = &wire.items[0].archive_id;
        let wire = get_detail_with_registry(&pool, detail_request(archive_id), &registry).unwrap();
        let shared =
            ccr_app::sessions::get_detail_with_registry(&pool, &registry, archive_id, None, None)
                .unwrap();
        let mapped = AgentSessionDetailDto {
            archive_id: shared.archive_id,
            agent: shared.agent.into(),
            variant: shared.variant,
            fidelity: shared.page.fidelity.into(),
            messages: shared
                .page
                .messages
                .into_iter()
                .map(message_to_dto)
                .collect(),
            next_before: shared.page.next_before,
            has_older: shared.page.has_older,
        };
        assert_eq!(
            serde_json::to_value(wire).unwrap(),
            serde_json::to_value(mapped).unwrap()
        );
    }
}
