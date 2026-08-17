use ccr_core::{redact_log_text, redact_log_value};
use ccr_types::FrontendLogInput;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;

pub const MAX_FRONTEND_LOG_BATCH: usize = 32;
pub const MAX_MESSAGE_CHARS: usize = 2000;
pub const MAX_SOURCE_CHARS: usize = 64;
pub const MAX_CORR_CHARS: usize = 64;
pub const MAX_FIELDS_JSON_BYTES: usize = 8192;

pub fn take_frontend_log_batch(entries: Vec<FrontendLogInput>) -> Vec<FrontendLogInput> {
    entries
        .into_iter()
        .take(MAX_FRONTEND_LOG_BATCH)
        .map(sanitize_frontend_log)
        .collect()
}

pub fn sanitize_frontend_log(input: FrontendLogInput) -> FrontendLogInput {
    let message = truncate_chars(&redact_log_text(&input.message), MAX_MESSAGE_CHARS);
    let source = sanitize_source(&input.source);
    let correlation_id = sanitize_correlation(input.correlation_id);
    let timestamp = Some(sanitize_timestamp(input.timestamp.as_deref()));
    let fields = input.fields.map(|value| sanitize_fields(&value));

    FrontendLogInput {
        level: input.level,
        message,
        source,
        timestamp,
        correlation_id,
        fields,
    }
}

fn sanitize_source(source: &str) -> String {
    let trimmed = truncate_chars(source.trim(), MAX_SOURCE_CHARS);
    if trimmed.is_empty() {
        "frontend".to_string()
    } else {
        trimmed
    }
}

fn sanitize_correlation(correlation_id: Option<String>) -> Option<String> {
    let value = correlation_id?;
    let trimmed = truncate_chars(value.trim(), MAX_CORR_CHARS);
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn sanitize_timestamp(raw: Option<&str>) -> String {
    let now = Utc::now();
    let Some(value) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return now.to_rfc3339();
    };

    let Ok(parsed) = DateTime::parse_from_rfc3339(value) else {
        return now.to_rfc3339();
    };
    let parsed = parsed.with_timezone(&Utc);
    let delta = now.signed_duration_since(parsed);
    if delta > Duration::hours(24) || delta < Duration::hours(-1) {
        return now.to_rfc3339();
    }
    parsed.to_rfc3339()
}

fn sanitize_fields(value: &Value) -> Value {
    let redacted = redact_log_value(value);
    match serde_json::to_vec(&redacted) {
        Ok(bytes) if bytes.len() > MAX_FIELDS_JSON_BYTES => {
            serde_json::json!({ "truncated": true })
        }
        Ok(_) => redacted,
        Err(_) => serde_json::json!({ "truncated": true }),
    }
}

fn truncate_chars(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_types::MonitoringLevel;
    use serde_json::json;

    fn sample(message: &str) -> FrontendLogInput {
        FrontendLogInput {
            level: MonitoringLevel::Error,
            message: message.to_string(),
            source: String::new(),
            timestamp: Some("2026-03-07T00:00:00Z".to_string()),
            correlation_id: Some("session-1".to_string()),
            fields: None,
        }
    }

    #[test]
    fn sanitize_frontend_log_redacts_secret_fields_and_text() {
        let input = FrontendLogInput {
            fields: Some(json!({
                "apiKey": "sk-ant-1234567890abcdef",
                "note": "Bearer abcdefghijklmnopqrstuv"
            })),
            ..sample("using key sk-ant-1234567890abcdef")
        };

        let sanitized = sanitize_frontend_log(input);
        let rendered = format!(
            "{} {}",
            sanitized.message,
            sanitized.fields.as_ref().unwrap()
        );
        assert!(!rendered.contains("sk-ant-1234567890abcdef"));
        assert!(!rendered.contains("abcdefghijklmnopqrstuv"));
        assert_eq!(sanitized.source, "frontend");
        assert_eq!(sanitized.correlation_id.as_deref(), Some("session-1"));
    }

    #[test]
    fn sanitize_frontend_log_truncates_message_and_oversize_fields() {
        let long_message: String = "a".repeat(MAX_MESSAGE_CHARS + 20);
        let huge = "x".repeat(MAX_FIELDS_JSON_BYTES + 8);
        let input = FrontendLogInput {
            fields: Some(json!({ "note": huge })),
            ..sample(&long_message)
        };

        let sanitized = sanitize_frontend_log(input);
        assert_eq!(sanitized.message.chars().count(), MAX_MESSAGE_CHARS);
        assert_eq!(sanitized.fields, Some(json!({ "truncated": true })));
    }

    #[test]
    fn take_frontend_log_batch_keeps_first_32() {
        let entries = (0..33)
            .map(|index| sample(&format!("msg-{index}")))
            .collect();
        let batch = take_frontend_log_batch(entries);
        assert_eq!(batch.len(), 32);
        assert_eq!(batch[0].message, "msg-0");
        assert_eq!(batch[31].message, "msg-31");
    }

    #[test]
    fn sanitize_replaces_stale_timestamp() {
        let input = FrontendLogInput {
            timestamp: Some("2010-01-01T00:00:00Z".to_string()),
            ..sample("ok")
        };
        let sanitized = sanitize_frontend_log(input);
        let stamp = sanitized.timestamp.unwrap();
        assert_ne!(stamp, "2010-01-01T00:00:00Z");
        assert!(DateTime::parse_from_rfc3339(&stamp).is_ok());
    }
}
