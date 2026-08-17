use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MonitoringLevel {
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl MonitoringLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl From<&str> for MonitoringLevel {
    fn from(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "debug" => Self::Debug,
            "warn" | "warning" => Self::Warn,
            "error" => Self::Error,
            _ => Self::Info,
        }
    }
}

impl From<String> for MonitoringLevel {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringEntry {
    pub id: String,
    pub timestamp: String,
    pub level: MonitoringLevel,
    pub channel: String,
    pub event_type: String,
    pub source: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Value>,
}

impl MonitoringEntry {
    pub fn new(
        level: MonitoringLevel,
        channel: impl Into<String>,
        event_type: impl Into<String>,
        source: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            level,
            channel: channel.into(),
            event_type: event_type.into(),
            source: source.into(),
            message: message.into(),
            correlation_id: None,
            fields: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    pub fn with_fields(mut self, fields: Value) -> Self {
        self.fields = Some(fields);
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendLogInput {
    pub level: MonitoringLevel,
    pub message: String,
    #[serde(default)]
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoringFeedQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<MonitoringLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monitoring_level_parses_aliases() {
        assert_eq!(MonitoringLevel::from("warning"), MonitoringLevel::Warn);
        assert_eq!(MonitoringLevel::from("ERROR"), MonitoringLevel::Error);
        assert_eq!(MonitoringLevel::from("other"), MonitoringLevel::Info);
    }

    #[test]
    fn monitoring_entry_builder_sets_defaults() {
        let entry = MonitoringEntry::new(
            MonitoringLevel::Info,
            "system",
            "runtime.started",
            "desktop",
            "started",
        );

        assert_eq!(entry.level, MonitoringLevel::Info);
        assert_eq!(entry.channel, "system");
        assert_eq!(entry.event_type, "runtime.started");
        assert_eq!(entry.source, "desktop");
        assert_eq!(entry.message, "started");
        assert!(entry.correlation_id.is_none());
    }
}
