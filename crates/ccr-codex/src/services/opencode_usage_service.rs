//! OpenCode 本地使用量服务
//!
//! 从 `opencode.db` 的 `message` 表解析本地 usage 记录。
//! 当前优先聚合 `providerID == "openai"` 的 assistant 消息。

use super::codex_usage_service::{
    CodexRollingUsage, CodexUsageRecord, CodexUsageService, CodexUsageStats,
};
use ccr_core::core::error::{CcrError, Result};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;

pub type OpenCodeUsageStats = CodexUsageStats;
pub type OpenCodeRollingUsage = CodexRollingUsage;

/// OpenCode 本地 usage 记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenCodeUsageRecord {
    /// 会话 ID
    pub session_id: String,
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 输入 tokens
    pub input_tokens: u64,
    /// 输出 tokens
    pub output_tokens: u64,
    /// Provider 名称
    pub provider: Option<String>,
    /// 模型名称
    pub model: Option<String>,
}

/// OpenCode 本地 usage 服务
pub struct OpenCodeUsageService {
    opencode_dir: PathBuf,
}

impl OpenCodeUsageService {
    pub fn new(opencode_dir: PathBuf) -> Self {
        Self { opencode_dir }
    }

    fn db_path(&self) -> PathBuf {
        self.opencode_dir.join("opencode.db")
    }

    /// 解析指定 provider 的本地 usage 记录。
    pub fn parse_provider_messages(
        &self,
        provider_filter: &str,
    ) -> Result<Vec<OpenCodeUsageRecord>> {
        let db_path = self.db_path();
        if !db_path.exists() {
            return Ok(Vec::new());
        }

        let connection = Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|err| CcrError::DatabaseError(format!("打开 OpenCode usage 数据库失败: {err}")))?;

        let mut statement = connection
            .prepare(
                r#"
                SELECT session_id, time_updated, data
                FROM message
                WHERE data LIKE ?1
                  AND data LIKE '%providerID%'
                  AND data LIKE '%assistant%'
                ORDER BY time_updated ASC
                "#,
            )
            .map_err(|err| {
                CcrError::DatabaseError(format!("准备 OpenCode usage 查询失败: {err}"))
            })?;

        let provider_pattern = format!("%{}%", provider_filter);
        let rows = statement
            .query_map([provider_pattern], |row| {
                let session_id: String = row.get(0)?;
                let time_updated: i64 = row.get(1)?;
                let data: String = row.get(2)?;
                Ok((session_id, time_updated, data))
            })
            .map_err(|err| {
                CcrError::DatabaseError(format!("读取 OpenCode usage 记录失败: {err}"))
            })?;

        let mut records = Vec::new();
        for row in rows.flatten() {
            if let Some(record) = Self::parse_message_row(row.0, row.1, &row.2, provider_filter) {
                records.push(record);
            }
        }

        Ok(records)
    }

    pub fn compute_rolling_usage_for_records(
        records: &[OpenCodeUsageRecord],
    ) -> OpenCodeRollingUsage {
        let codex_records: Vec<CodexUsageRecord> = records
            .iter()
            .map(|record| CodexUsageRecord {
                session_id: record.session_id.clone(),
                timestamp: record.timestamp,
                input_tokens: record.input_tokens,
                output_tokens: record.output_tokens,
                model: record.model.clone(),
            })
            .collect();
        CodexUsageService::compute_rolling_usage_for_records(&codex_records)
    }

    pub fn format_tokens(tokens: u64) -> String {
        CodexUsageService::format_tokens(tokens)
    }

    fn parse_message_row(
        session_id: String,
        time_updated_ms: i64,
        data: &str,
        provider_filter: &str,
    ) -> Option<OpenCodeUsageRecord> {
        let json: Value = serde_json::from_str(data).ok()?;

        if json.get("role").and_then(Value::as_str) != Some("assistant") {
            return None;
        }

        let provider = json
            .get("providerID")
            .and_then(Value::as_str)
            .or_else(|| {
                json.get("model")
                    .and_then(|model| model.get("providerID"))
                    .and_then(Value::as_str)
            })
            .map(|value| value.to_string());
        if provider.as_deref() != Some(provider_filter) {
            return None;
        }

        let tokens = json.get("tokens")?;
        let input_tokens = tokens.get("input").and_then(Value::as_u64).unwrap_or(0);
        let output_tokens = tokens.get("output").and_then(Value::as_u64).unwrap_or(0);
        if input_tokens == 0 && output_tokens == 0 {
            return None;
        }

        let timestamp = json
            .get("time")
            .and_then(|time| time.get("completed").or_else(|| time.get("created")))
            .and_then(Value::as_i64)
            .and_then(|millis| Utc.timestamp_millis_opt(millis).single())
            .or_else(|| Utc.timestamp_millis_opt(time_updated_ms).single())?;

        let model = json
            .get("modelID")
            .and_then(Value::as_str)
            .or_else(|| {
                json.get("model")
                    .and_then(|model| model.get("modelID"))
                    .and_then(Value::as_str)
            })
            .map(|value| value.to_string());

        Some(OpenCodeUsageRecord {
            session_id,
            timestamp,
            input_tokens,
            output_tokens,
            provider,
            model,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use chrono::Duration;
    use tempfile::TempDir;

    fn create_test_service() -> (OpenCodeUsageService, TempDir) {
        let temp = TempDir::new().unwrap();
        let service = OpenCodeUsageService::new(temp.path().to_path_buf());
        (service, temp)
    }

    fn create_message_table(conn: &Connection) {
        conn.execute_batch(
            r#"
            CREATE TABLE message (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL,
                data TEXT NOT NULL
            );
            "#,
        )
        .unwrap();
    }

    #[test]
    fn parse_provider_messages_filters_openai_assistant_rows() {
        let (service, temp) = create_test_service();
        let db_path = temp.path().join("opencode.db");
        let conn = Connection::open(&db_path).unwrap();
        create_message_table(&conn);

        let now = Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                "msg-1",
                "ses-1",
                now,
                now,
                serde_json::json!({
                    "role": "assistant",
                    "providerID": "openai",
                    "modelID": "gpt-5.4",
                    "time": { "completed": now },
                    "tokens": { "input": 1200, "output": 240 }
                })
                .to_string(),
            ),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO message (id, session_id, time_created, time_updated, data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                "msg-2",
                "ses-2",
                now,
                now,
                serde_json::json!({
                    "role": "assistant",
                    "providerID": "github-copilot",
                    "modelID": "claude-opus-4.6",
                    "time": { "completed": now },
                    "tokens": { "input": 2000, "output": 400 }
                })
                .to_string(),
            ),
        )
        .unwrap();

        let records = service.parse_provider_messages("openai").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].session_id, "ses-1");
        assert_eq!(records[0].model.as_deref(), Some("gpt-5.4"));
        assert_eq!(records[0].input_tokens, 1200);
        assert_eq!(records[0].output_tokens, 240);
    }

    #[test]
    fn compute_rolling_usage_for_records_aggregates_by_window_and_model() {
        let now = Utc::now();
        let records = vec![
            OpenCodeUsageRecord {
                session_id: "ses-1".to_string(),
                timestamp: now - Duration::hours(2),
                input_tokens: 100,
                output_tokens: 20,
                provider: Some("openai".to_string()),
                model: Some("gpt-5.4".to_string()),
            },
            OpenCodeUsageRecord {
                session_id: "ses-2".to_string(),
                timestamp: now - Duration::days(2),
                input_tokens: 200,
                output_tokens: 40,
                provider: Some("openai".to_string()),
                model: Some("gpt-5.4-mini".to_string()),
            },
        ];

        let rolling = OpenCodeUsageService::compute_rolling_usage_for_records(&records);
        assert_eq!(rolling.five_hour.total_requests, 1);
        assert_eq!(rolling.seven_day.total_requests, 2);
        assert_eq!(rolling.all_time.total_requests, 2);
        assert_eq!(
            rolling
                .by_model
                .get("gpt-5.4-mini")
                .map(|stats| stats.total_tokens()),
            Some(240)
        );
    }

    #[test]
    #[ignore]
    fn benchmark_parse_provider_messages() {
        for row_count in [1_000usize, 50_000] {
            let (service, temp) = create_test_service();
            let db_path = temp.path().join("opencode.db");
            let mut conn = Connection::open(&db_path).unwrap();
            create_message_table(&conn);

            let now = Utc::now().timestamp_millis();
            {
                let tx = conn.transaction().unwrap();
                {
                    let mut stmt = tx
                        .prepare(
                            "INSERT INTO message (id, session_id, time_created, time_updated, data)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                        )
                        .unwrap();
                    for index in 0..row_count {
                        let provider = if index % 2 == 0 {
                            "openai"
                        } else {
                            "github-copilot"
                        };
                        stmt.execute((
                            format!("msg-{index}"),
                            format!("ses-{index}"),
                            now,
                            now + index as i64,
                            serde_json::json!({
                                "role": "assistant",
                                "providerID": provider,
                                "modelID": "gpt-5.4",
                                "time": { "completed": now + index as i64 },
                                "tokens": { "input": 1200, "output": 240 }
                            })
                            .to_string(),
                        ))
                        .unwrap();
                    }
                }
                tx.commit().unwrap();
            }

            let start = std::time::Instant::now();
            let records = service.parse_provider_messages("openai").unwrap();
            let elapsed = start.elapsed();

            assert_eq!(records.len(), row_count / 2);
            eprintln!(
                "opencode_parse_provider_messages: rows={row_count}, matched={}, elapsed={elapsed:?}",
                records.len()
            );
        }
    }
}
