use serde::{Deserialize, Serialize};

use super::SourceKind;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum JobEvent {
    Started {
        job_id: String,
        files_total: u64,
    },
    BootstrapStarted,
    MigrationStarted {
        version: u32,
        name: String,
        latest_version: u32,
    },
    MigrationFinished {
        version: u32,
        name: String,
        elapsed_ms: u64,
    },
    PricingUpgradeStarted {
        from_version: String,
        to_version: String,
        total_events: usize,
    },
    PricingUpgradeProgress {
        from_version: String,
        to_version: String,
        processed_events: usize,
        total_events: usize,
        elapsed_ms: u64,
    },
    PricingBucketReconcileStarted {
        to_version: String,
        bucket_count: usize,
    },
    PricingUpgradeFinished {
        from_version: String,
        to_version: String,
        updated_events: usize,
        bucket_count: usize,
        deleted_orphan_buckets: usize,
        elapsed_ms: u64,
    },
    LockWaiting {
        timeout_ms: u64,
    },
    LockAcquired {
        wait_ms: u64,
    },
    TokenAccountingRepairStarted {
        sources: Vec<SourceKind>,
    },
    TokenAccountingRepairFinished {
        sources: Vec<SourceKind>,
    },
    SourceStarted {
        source: SourceKind,
        files_total: u64,
    },
    Progress {
        source: SourceKind,
        files_scanned: u64,
        records_imported: u64,
        current_file: Option<String>,
    },
    RecentReady {
        source: SourceKind,
    },
    SourceFinished {
        source: SourceKind,
        stats: SourceSyncStats,
    },
    Finished {
        summary: SyncSummaryEvent,
    },
    Failed {
        error: String,
    },
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(default, rename_all = "snake_case")]
pub struct SourceSyncStats {
    #[serde(default = "default_source")]
    pub source: SourceKind,
    pub files_processed: usize,
    pub changed_files: usize,
    pub bytes_scanned: u64,
    pub events_seen: usize,
    pub events_replayed: usize,
    pub events_inserted: usize,
    pub parse_ms: u64,
    pub write_ms: u64,
    pub lock_wait_ms: u64,
    pub absent: bool,
    pub last_error: Option<String>,
}

fn default_source() -> SourceKind {
    SourceKind::Codex
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub struct SyncSummaryEvent {
    #[serde(default)]
    pub sources: usize,
    #[serde(default)]
    pub total_seen: usize,
    #[serde(default)]
    pub total_inserted: usize,
}

pub fn is_optional_source_absent(stats: &SourceSyncStats) -> bool {
    stats.absent
}

pub fn parse_ndjson_event(line: &str) -> Result<Option<JobEvent>, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    // 上游 llmusage 0.5.3 的 tracing-subscriber 默认 writer 是 stdout（违反它自家 ADR 0001
    // "进度只写 stderr" 契约），导致 INFO 日志会跟 NDJSON 事件混在同一条流。这里把
    // "不像 JSON 的行" 当成噪音软跳过，避免一句日志干掉整条 sync。以 `{` 开头但解析失败
    // 仍当真正的契约破坏抛出来，便于发现上游字段不兼容等问题。
    if !trimmed.starts_with('{') {
        let preview: String = trimmed.chars().take(160).collect();
        tracing::debug!(noise = %preview, "skipped non-json line from llmusage stdout");
        return Ok(None);
    }
    serde_json::from_str::<JobEvent>(trimmed)
        .map(Some)
        .map_err(|error| format!("invalid llmusage json event `{trimmed}`: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_source_finished_with_optional_absent_flag() {
        let event = parse_ndjson_event(r#"{"event":"source_finished","source":"opencode","stats":{"source":"opencode","absent":true,"last_error":"missing"}}"#)
            .unwrap()
            .unwrap();
        match event {
            JobEvent::SourceFinished { source, stats } => {
                assert_eq!(source, SourceKind::Opencode);
                assert!(stats.absent);
                assert_eq!(stats.last_error.as_deref(), Some("missing"));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn skips_ansi_colored_tracing_line_mixed_into_stdout() {
        // 上游 logging.rs 默认 writer 是 stdout，开 sync 时 tracing INFO 日志会带 ANSI 控制码
        // 直接进 stdout。该行不是 JSON，应该被软跳过而不是把整条 sync 干掉。
        let noisy = "\u{1b}[2m2026-05-13T06:46:44.764240Z\u{1b}[0m  \u{1b}[32m INFO\u{1b}[0m 开始执行全量本地真源同步";
        assert_eq!(parse_ndjson_event(noisy).unwrap(), None);
    }

    #[test]
    fn skips_plain_text_log_line_without_braces() {
        assert_eq!(parse_ndjson_event("[INFO] starting sync").unwrap(), None);
        assert_eq!(parse_ndjson_event("warning: locked").unwrap(), None);
    }

    #[test]
    fn rejects_brace_started_but_malformed_json() {
        // `{` 开头但格式错才是真正的契约破坏，应当暴露，不能跟噪音混为一谈。
        let result = parse_ndjson_event("{not-json");
        assert!(result.is_err(), "brace-prefixed malformed JSON must error");
        let message = result.unwrap_err();
        assert!(
            message.contains("invalid llmusage json event"),
            "error message should preserve diagnostic prefix, got: {message}"
        );
    }

    #[test]
    fn noise_between_legal_events_does_not_break_stream() {
        let started = parse_ndjson_event(r#"{"event":"started","job_id":"cli","files_total":0}"#)
            .unwrap()
            .expect("started event should parse");
        let noise = parse_ndjson_event("\u{1b}[32m INFO\u{1b}[0m 准备解析 codex 真源").unwrap();
        let finished = parse_ndjson_event(
            r#"{"event":"finished","summary":{"sources":1,"total_seen":3,"total_inserted":3}}"#,
        )
        .unwrap()
        .expect("finished event should parse");

        assert!(noise.is_none());
        assert!(matches!(started, JobEvent::Started { .. }));
        assert!(matches!(finished, JobEvent::Finished { .. }));
    }

    #[test]
    fn parses_current_pricing_and_accounting_lifecycle_events() {
        let fixtures = [
            r#"{"event":"pricing_upgrade_started","from_version":"v1","to_version":"v2","total_events":10}"#,
            r#"{"event":"pricing_upgrade_progress","from_version":"v1","to_version":"v2","processed_events":5,"total_events":10,"elapsed_ms":12}"#,
            r#"{"event":"pricing_bucket_reconcile_started","to_version":"v2","bucket_count":4}"#,
            r#"{"event":"pricing_upgrade_finished","from_version":"v1","to_version":"v2","updated_events":10,"bucket_count":4,"deleted_orphan_buckets":1,"elapsed_ms":20}"#,
            r#"{"event":"token_accounting_repair_started","sources":["kimi_code","pi","grok"]}"#,
            r#"{"event":"token_accounting_repair_finished","sources":["antigravity"]}"#,
        ];

        for fixture in fixtures {
            assert!(
                parse_ndjson_event(fixture).unwrap().is_some(),
                "current event must parse: {fixture}"
            );
        }
    }
}
