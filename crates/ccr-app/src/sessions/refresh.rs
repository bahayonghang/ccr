//! Shared incremental archive refresh; provider parsing and persistence retain domain ownership.
use ccr_db::database::{
    DbPool,
    repositories::usage_repo::{
        self, AgentSessionArchiveSource, AgentSessionSourceState, UsageSessionArchiveEntry,
        UsageSourceState,
    },
};
use ccr_store::sessions::{
    AgentSessionAgentId, AgentSessionFidelity, AgentSessionProviderRegistry,
    AgentSessionSourceKind, AgentSessionSourceRef,
};
use chrono::Utc;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::UNIX_EPOCH,
};

#[derive(Debug, Clone, Default)]
pub struct RefreshCounters {
    pub discovered: u64,
    pub unchanged: u64,
    pub fingerprinted: u64,
    pub parsed: u64,
    pub upserted: u64,
    pub partial: u64,
    pub locked: u64,
    pub errors: u64,
}
#[derive(Debug, Clone)]
pub struct ProviderRefresh {
    pub agent: AgentSessionAgentId,
    pub counters: RefreshCounters,
}
#[derive(Debug, Clone)]
pub struct RefreshReport {
    pub total: RefreshCounters,
    pub providers: Vec<ProviderRefresh>,
}

pub fn refresh_archive_with_registry(
    pool: &DbPool,
    registry: &AgentSessionProviderRegistry,
) -> Result<RefreshReport, String> {
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
    // Restored rollouts retain their original mtime and bytes. An unchanged
    // fingerprint alone cannot turn a missing/deleted projection back to live.
    let non_live_containers = archive_map
        .values()
        .filter(|row| row.source_state != UsageSourceState::Live)
        .map(|row| (row.platform.clone(), row.file_path.clone()))
        .collect::<HashSet<_>>();
    drop(conn);
    let mut report = RefreshReport {
        total: RefreshCounters::default(),
        providers: Vec::new(),
    };
    let mut pending_entries = Vec::new();
    let mut pending_states = Vec::new();
    let mut seen_by_agent = HashMap::<String, Vec<(String, String)>>::new();
    let mut authoritative_agents = HashSet::<String>::new();

    for agent in AgentSessionAgentId::ALL {
        let mut counters = RefreshCounters::default();
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
            if non_live_containers.contains(&(agent.as_str().to_string(), path.clone()))
                || !state_map.get(&state_key).is_some_and(|state| {
                    state.source_stat_hash == container_hash && state.last_error_code.is_none()
                })
            {
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
                report.providers.push(ProviderRefresh { agent, counters });
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
            if !non_live_containers.contains(&(
                agent.as_str().to_string(),
                path.to_string_lossy().to_string(),
            )) && members.iter().all(|source| {
                archive_map
                    .get(&(
                        agent.as_str().to_string(),
                        source.physical_path.to_string_lossy().to_string(),
                        source.member_id.clone().unwrap_or_default(),
                    ))
                    .is_some_and(|row| row.source_state == UsageSourceState::Live)
            }) && state_map.get(&state_key).is_some_and(|state| {
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
                    .filter(|row| row.source_state == UsageSourceState::Live)
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
        report.providers.push(ProviderRefresh { agent, counters });
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

fn accumulate(total: &mut RefreshCounters, item: &RefreshCounters) {
    total.discovered += item.discovered;
    total.unchanged += item.unchanged;
    total.fingerprinted += item.fingerprinted;
    total.parsed += item.parsed;
    total.upserted += item.upserted;
    total.partial += item.partial;
    total.locked += item.locked;
    total.errors += item.errors;
}
