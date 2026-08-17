use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Mutex, OnceLock};

use serde_json::Value;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use uuid::Uuid;

use super::log_redact::{redact_log_text, redact_log_value};

pub const BRIDGE_QUEUE_CAP: usize = 256;

const EXCLUDED_TARGET_PREFIXES: &[&str] = &[
    "ccr_core::log_redact",
    "ccr_core::core::log_redact",
    "ccr_core::core::logging",
    "ccr_core::core::log_bridge",
    "ccr_core::core::log_writer",
    "ccr_db::services::log_persistence",
    "ccr_desktop::monitoring",
    "ccr_desktop::bridge",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueResult {
    Accepted,
    Full,
    Disabled,
    Reentrant,
}

#[derive(Debug, Clone)]
pub struct BridgedLogEvent {
    pub level: String,
    pub target: String,
    pub message: String,
    pub fields: Value,
    pub correlation_id: String,
    pub timestamp: String,
}

static PROCESS_ID: OnceLock<String> = OnceLock::new();
static SENDER: Mutex<Option<SyncSender<BridgedLogEvent>>> = Mutex::new(None);
static RECEIVER: Mutex<Option<Receiver<BridgedLogEvent>>> = Mutex::new(None);
static DROPPED: AtomicU64 = AtomicU64::new(0);
static CLOSED: AtomicBool = AtomicBool::new(false);

thread_local! {
    static BRIDGE_REENTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn current_log_correlation_id() -> &'static str {
    PROCESS_ID.get_or_init(|| Uuid::new_v4().to_string())
}

pub fn dropped_bridged_log_count() -> u64 {
    DROPPED.load(Ordering::Relaxed)
}

pub fn ensure_log_bridge_queue() {
    if CLOSED.load(Ordering::SeqCst) {
        return;
    }
    let mut sender = SENDER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if sender.is_some() {
        return;
    }
    let (next_sender, receiver) = sync_channel(BRIDGE_QUEUE_CAP);
    let mut slot = RECEIVER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *slot = Some(receiver);
    *sender = Some(next_sender);
}

pub fn close_bridged_log_sender() {
    CLOSED.store(true, Ordering::SeqCst);
    let mut sender = SENDER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *sender = None;
}

pub fn take_bridged_log_receiver() -> Option<Receiver<BridgedLogEvent>> {
    ensure_log_bridge_queue();
    RECEIVER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
}

pub fn try_enqueue_bridged_log(event: BridgedLogEvent) -> EnqueueResult {
    if BRIDGE_REENTERED.get() {
        return EnqueueResult::Reentrant;
    }

    ensure_log_bridge_queue();
    let sender = SENDER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(sender) = sender.as_ref() else {
        return EnqueueResult::Disabled;
    };

    match sender.try_send(event) {
        Ok(()) => EnqueueResult::Accepted,
        Err(TrySendError::Full(_)) => {
            DROPPED.fetch_add(1, Ordering::Relaxed);
            EnqueueResult::Full
        }
        Err(TrySendError::Disconnected(_)) => EnqueueResult::Disabled,
    }
}

pub fn should_bridge_event(level: &Level, target: &str) -> bool {
    matches!(*level, Level::WARN | Level::ERROR)
        && target.starts_with("ccr_")
        && !is_excluded_target(target)
}

pub fn is_excluded_target(target: &str) -> bool {
    EXCLUDED_TARGET_PREFIXES
        .iter()
        .any(|prefix| target == *prefix || target.starts_with(&format!("{prefix}::")))
}

pub struct BridgeEnqueueLayer;

impl<S> Layer<S> for BridgeEnqueueLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        if !should_bridge_event(metadata.level(), metadata.target()) {
            return;
        }

        let mut visitor = FieldCollector::default();
        event.record(&mut visitor);
        let message = redact_log_text(&visitor.message);
        let fields = redact_log_value(&Value::Object(visitor.fields));

        let _ = try_enqueue_bridged_log(BridgedLogEvent {
            level: metadata.level().as_str().to_ascii_lowercase(),
            target: metadata.target().to_string(),
            message,
            fields,
            correlation_id: current_log_correlation_id().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        });
    }
}

#[derive(Default)]
struct FieldCollector {
    message: String,
    fields: serde_json::Map<String, Value>,
}

impl tracing::field::Visit for FieldCollector {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        let rendered = format!("{value:?}");
        if field.name() == "message" {
            self.message = trim_debug_quotes(&rendered);
        } else {
            self.fields
                .insert(field.name().to_string(), Value::String(rendered));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields
                .insert(field.name().to_string(), Value::String(value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), Value::from(value));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), Value::from(value));
    }
}

fn trim_debug_quotes(input: &str) -> String {
    input
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(input)
        .to_string()
}

/// 供测试与消费端在回调内使用，避免桥接重入。
pub fn enter_bridge_consumer<T>(func: impl FnOnce() -> T) -> T {
    BRIDGE_REENTERED.set(true);
    let result = func();
    BRIDGE_REENTERED.set(false);
    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn process_id_is_stable() {
        let first = current_log_correlation_id();
        let second = current_log_correlation_id();
        assert_eq!(first, second);
        assert!(!first.is_empty());
    }

    #[test]
    fn excluded_targets_are_not_bridged() {
        assert!(!should_bridge_event(
            &Level::ERROR,
            "ccr_desktop::monitoring"
        ));
        assert!(!should_bridge_event(
            &Level::WARN,
            "ccr_db::services::log_persistence"
        ));
        assert!(should_bridge_event(&Level::ERROR, "ccr_core::core::http"));
        assert!(!should_bridge_event(&Level::INFO, "ccr_core::core::http"));
        assert!(!should_bridge_event(&Level::ERROR, "hyper"));
    }

    #[test]
    fn reentrant_enqueue_is_dropped() {
        enter_bridge_consumer(|| {
            let result = try_enqueue_bridged_log(sample_event("reenter"));
            assert_eq!(result, EnqueueResult::Reentrant);
        });
    }

    #[test]
    fn queue_reports_full_after_capacity() {
        let _receiver = take_bridged_log_receiver();
        let mut accepted = 0;
        let mut full = 0;
        for index in 0..(BRIDGE_QUEUE_CAP + 4) {
            match try_enqueue_bridged_log(sample_event(&index.to_string())) {
                EnqueueResult::Accepted => accepted += 1,
                EnqueueResult::Full => full += 1,
                other => panic!("unexpected {other:?}"),
            }
        }
        assert_eq!(accepted, BRIDGE_QUEUE_CAP);
        assert!(full >= 1);
    }

    fn sample_event(message: &str) -> BridgedLogEvent {
        BridgedLogEvent {
            level: "error".to_string(),
            target: "ccr_core::core::http".to_string(),
            message: message.to_string(),
            fields: Value::Object(serde_json::Map::new()),
            correlation_id: current_log_correlation_id().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
