//! Explicit-path archive reads. Provider validation/parsing and SQL stay with their domain owners.
pub mod refresh;

use ccr_db::database::{
    DbPool,
    repositories::usage_repo::{self, AgentSessionArchiveQuery, AgentSessionArchiveRow},
};
use ccr_store::sessions::{
    AgentSessionAgentId, AgentSessionFidelity, AgentSessionMessagePage,
    AgentSessionProviderRegistry, AgentSessionSourceKind, AgentSessionSourceRef,
};
use chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    QueryTooLong,
    InvalidDate,
    InvalidDateRange,
    InvalidCursor,
    InvalidFilter,
    InvalidArchiveId,
    Database,
    NotFound,
    UnknownProvider,
    UnknownSourceKind,
    UnknownFidelity,
    SourceUnavailable,
    SourceValidation,
}
impl SessionError {
    pub fn code(self) -> &'static str {
        match self {
            Self::QueryTooLong => "agent_session_query_too_long",
            Self::InvalidDate => "agent_session_invalid_date",
            Self::InvalidDateRange => "agent_session_invalid_date_range",
            Self::InvalidCursor => "agent_session_invalid_cursor",
            Self::InvalidFilter => "agent_session_invalid_filter",
            Self::InvalidArchiveId => "agent_session_invalid_archive_id",
            Self::Database => "agent_session_db_unavailable",
            Self::NotFound => "agent_session_not_found",
            Self::UnknownProvider => "agent_session_unknown_provider",
            Self::UnknownSourceKind => "agent_session_unknown_source_kind",
            Self::UnknownFidelity => "agent_session_unknown_fidelity",
            Self::SourceUnavailable => "agent_session_source_unavailable",
            Self::SourceValidation => "agent_session_source_validation_failed",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SessionListRequest {
    pub agents: Option<Vec<AgentSessionAgentId>>,
    pub query: Option<String>,
    pub cwd_prefix: Option<String>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub source_state: Option<String>,
    pub fidelity: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}
#[derive(Debug)]
pub struct SessionPage {
    /// Summary metadata only, never the archive's private physical source path.
    pub items: Vec<AgentSessionArchiveRow>,
    pub next_cursor: Option<String>,
}
#[derive(Debug)]
pub struct SessionDetail {
    pub archive_id: String,
    pub agent: AgentSessionAgentId,
    pub variant: String,
    pub page: AgentSessionMessagePage,
}

pub fn list_sessions(
    pool: &DbPool,
    request: SessionListRequest,
) -> Result<SessionPage, SessionError> {
    let query_text = normalize_text(request.query, 200)?;
    let cwd_prefix = normalize_text(request.cwd_prefix, 1024)?;
    let started_at = parse_date(request.started_at)?;
    let ended_at = parse_date(request.ended_at)?;
    if started_at
        .zip(ended_at)
        .is_some_and(|(start, end)| start > end)
    {
        return Err(SessionError::InvalidDateRange);
    }
    let (cursor_updated_at, cursor_archive_id) = decode_cursor(request.cursor.as_deref())?;
    let query = AgentSessionArchiveQuery {
        platforms: request
            .agents
            .filter(|agents| !agents.is_empty())
            .unwrap_or_else(|| AgentSessionAgentId::ALL.to_vec())
            .into_iter()
            .map(|agent| agent.as_str().to_string())
            .collect(),
        query: query_text,
        cwd_prefix,
        started_at: started_at.map(|value| value.to_rfc3339()),
        ended_at: ended_at.map(|value| value.to_rfc3339()),
        source_state: validate_choice(
            request.source_state,
            &["live", "missing", "deleted_by_user", "all"],
        )?
        .filter(|value| value != "all"),
        source_fidelity: validate_choice(request.fidelity, &["full", "partial", "locked", "all"])?
            .filter(|value| value != "all"),
        cursor_updated_at,
        cursor_archive_id,
        limit: request.limit.unwrap_or(80).clamp(1, 200),
    };
    let conn = pool.get().map_err(|_| SessionError::Database)?;
    let rows = usage_repo::get_agent_session_archive_page(&conn, &query)
        .map_err(|_| SessionError::Database)?;
    for row in &rows {
        AgentSessionAgentId::parse(&row.platform).ok_or(SessionError::UnknownProvider)?;
        parse_fidelity(&row.source_fidelity)?;
    }
    let next_cursor = (rows.len() == query.limit)
        .then(|| {
            rows.last()
                .map(|row| encode_cursor(row.updated_at, &row.archive_id))
        })
        .flatten();
    Ok(SessionPage {
        items: rows,
        next_cursor,
    })
}

/// Restore a provider-owned source from opaque identity, validating before any transcript read.
pub fn get_detail_with_registry(
    pool: &DbPool,
    registry: &AgentSessionProviderRegistry,
    archive_id: &str,
    before_cursor: Option<u32>,
    limit: Option<usize>,
) -> Result<SessionDetail, SessionError> {
    let (source, fidelity) = resolve_source_with_registry(pool, registry, archive_id)?;
    let mut page = registry
        .read_message_page(&source, before_cursor, limit.unwrap_or(100).clamp(1, 200))
        .map_err(|_| SessionError::SourceUnavailable)?;
    page.fidelity = combine_fidelity(fidelity, page.fidelity);
    Ok(SessionDetail {
        archive_id: archive_id.to_owned(),
        agent: source.agent,
        variant: source.variant,
        page,
    })
}

/// Resolve only an opaque archive identity. Physical paths remain backend-owned.
pub fn resolve_source_with_registry(
    pool: &DbPool,
    registry: &AgentSessionProviderRegistry,
    archive_id: &str,
) -> Result<(AgentSessionSourceRef, AgentSessionFidelity), SessionError> {
    if archive_id.is_empty() || archive_id.len() > 128 {
        return Err(SessionError::InvalidArchiveId);
    }
    let conn = pool.get().map_err(|_| SessionError::Database)?;
    let stored = usage_repo::get_agent_session_archive_source(&conn, archive_id)
        .map_err(|_| SessionError::Database)?
        .ok_or(SessionError::NotFound)?;
    drop(conn);
    let agent =
        AgentSessionAgentId::parse(&stored.platform).ok_or(SessionError::UnknownProvider)?;
    let kind = AgentSessionSourceKind::parse(&stored.source_kind)
        .ok_or(SessionError::UnknownSourceKind)?;
    let source = registry
        .restore_source(
            agent,
            &stored.source_variant,
            PathBuf::from(&stored.file_path),
            kind,
            (!stored.source_member_id.is_empty()).then(|| stored.source_member_id.clone()),
        )
        .map_err(|error| {
            // 保留既有 owner 的错误分类，不暴露路径或解析器原文。
            let message = error.to_string();
            if message.contains("session source is missing")
                || message.contains("session source root is unavailable")
                || message.contains("session member is missing")
            {
                SessionError::SourceUnavailable
            } else {
                SessionError::SourceValidation
            }
        })?;
    Ok((source, parse_fidelity(&stored.source_fidelity)?))
}

pub fn parse_fidelity(value: &str) -> Result<AgentSessionFidelity, SessionError> {
    match value {
        "full" => Ok(AgentSessionFidelity::Full),
        "partial" => Ok(AgentSessionFidelity::Partial),
        "locked" => Ok(AgentSessionFidelity::Locked),
        _ => Err(SessionError::UnknownFidelity),
    }
}
pub fn combine_fidelity(
    archived: AgentSessionFidelity,
    current: AgentSessionFidelity,
) -> AgentSessionFidelity {
    use AgentSessionFidelity::*;
    match (archived, current) {
        (Locked, _) | (_, Locked) => Locked,
        (Partial, _) | (_, Partial) => Partial,
        _ => Full,
    }
}
fn normalize_text(value: Option<String>, max_chars: usize) -> Result<Option<String>, SessionError> {
    let value = value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if value
        .as_ref()
        .is_some_and(|value| value.chars().count() > max_chars)
    {
        return Err(SessionError::QueryTooLong);
    }
    Ok(value)
}
fn parse_date(value: Option<String>) -> Result<Option<DateTime<Utc>>, SessionError> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|_| SessionError::InvalidDate)
        })
        .transpose()
}
fn validate_choice(
    value: Option<String>,
    allowed: &[&str],
) -> Result<Option<String>, SessionError> {
    let value = value
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());
    if value
        .as_ref()
        .is_some_and(|value| !allowed.contains(&value.as_str()))
    {
        return Err(SessionError::InvalidFilter);
    }
    Ok(value)
}
pub fn encode_cursor(updated_at: DateTime<Utc>, archive_id: &str) -> String {
    format!("{}|{archive_id}", updated_at.to_rfc3339())
}
pub fn decode_cursor(
    cursor: Option<&str>,
) -> Result<(Option<String>, Option<String>), SessionError> {
    let Some(cursor) = cursor else {
        return Ok((None, None));
    };
    if cursor.len() > 256 {
        return Err(SessionError::InvalidCursor);
    }
    let (updated_at, archive_id) = cursor.split_once('|').ok_or(SessionError::InvalidCursor)?;
    let updated_at = DateTime::parse_from_rfc3339(updated_at)
        .map_err(|_| SessionError::InvalidCursor)?
        .with_timezone(&Utc)
        .to_rfc3339();
    if archive_id.is_empty() || archive_id.len() > 128 {
        return Err(SessionError::InvalidCursor);
    }
    Ok((Some(updated_at), Some(archive_id.to_string())))
}

#[cfg(test)]
mod tests;
