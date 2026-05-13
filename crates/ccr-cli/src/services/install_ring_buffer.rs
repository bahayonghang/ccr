//! Fixed-capacity ring buffer for install events.
//!
//! Stores the 50 most recent `Log` events plus 1 terminal event slot.
//! Credential patterns are redacted before storage.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::services::install_types::{AttemptId, InstallEvent, RingBufferSnapshot};

const MAX_LOG_ENTRIES: usize = 50;

/// Shared handle to the ring buffer.
#[derive(Debug, Clone)]
pub struct RingBufferHandle(Arc<Mutex<RingBuffer>>);

impl RingBufferHandle {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(RingBuffer::new())))
    }

    /// Record an event into the ring buffer.
    pub fn record(&self, event: &InstallEvent) {
        if let Ok(mut buf) = self.0.lock() {
            buf.record(event);
        }
    }

    /// Take a snapshot of the current buffer state.
    pub fn snapshot(&self) -> RingBufferSnapshot {
        self.0
            .lock()
            .map(|buf| buf.snapshot())
            .unwrap_or_else(|_| RingBufferSnapshot {
                attempt_id: None,
                logs: Vec::new(),
                terminal: None,
            })
    }

    /// Clear the buffer (called on new attempt).
    pub fn clear(&self) {
        if let Ok(mut buf) = self.0.lock() {
            buf.clear();
        }
    }
}

impl Default for RingBufferHandle {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct RingBuffer {
    attempt_id: Option<AttemptId>,
    logs: VecDeque<InstallEvent>,
    terminal: Option<InstallEvent>,
}

impl RingBuffer {
    fn new() -> Self {
        Self {
            attempt_id: None,
            logs: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            terminal: None,
        }
    }

    fn clear(&mut self) {
        self.attempt_id = None;
        self.logs.clear();
        self.terminal = None;
    }

    fn record(&mut self, event: &InstallEvent) {
        let event_attempt = event.attempt_id();

        // If we see a Started event with a new attempt_id, clear the buffer.
        if matches!(event, InstallEvent::Started { .. }) {
            self.clear();
            self.attempt_id = Some(event_attempt);
        }

        if event.is_terminal() {
            self.terminal = Some(event.clone());
        } else if matches!(event, InstallEvent::Log { .. }) {
            // Evict oldest if at capacity.
            if self.logs.len() >= MAX_LOG_ENTRIES {
                self.logs.pop_front();
            }
            self.logs.push_back(event.clone());
        }
        // Started and Progress events are not stored in the ring buffer.
    }

    fn snapshot(&self) -> RingBufferSnapshot {
        RingBufferSnapshot {
            attempt_id: self.attempt_id,
            logs: self.logs.iter().cloned().collect(),
            terminal: self.terminal.clone(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Credential redaction
// ──────────────────────────────────────────────────────────────────────────────

/// Known credential-like patterns to redact from log lines.
const CREDENTIAL_PATTERNS: &[&str] = &[
    "token=",
    "key=",
    "password=",
    "secret=",
    "authorization:",
    "bearer ",
    "api_key=",
    "apikey=",
];

/// Redact known credential patterns from a log line.
///
/// Replaces the value after the pattern with `***` up to the next whitespace or end of line.
pub fn redact(line: &str) -> String {
    let mut result = line.to_string();
    let lower = line.to_lowercase();

    for pattern in CREDENTIAL_PATTERNS {
        if let Some(start) = lower.find(pattern) {
            let value_start = start + pattern.len();
            // Find the end of the value (next whitespace, comma, or end of string)
            let value_end = result[value_start..]
                .find(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '"' || c == '\'')
                .map(|pos| value_start + pos)
                .unwrap_or(result.len());

            if value_end > value_start {
                result.replace_range(value_start..value_end, "***");
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::install_types::{
        AttemptId, DurationClass, InstallPlan, LogStream, PackageManager, Platform,
    };
    use std::collections::BTreeMap;

    fn make_log(attempt_id: AttemptId, seq: u64) -> InstallEvent {
        InstallEvent::Log {
            attempt_id,
            stream: LogStream::Stdout,
            line: format!("line {seq}"),
            seq,
        }
    }

    fn make_started(attempt_id: AttemptId) -> InstallEvent {
        InstallEvent::Started {
            attempt_id,
            plan: InstallPlan {
                platform: Platform::Macos,
                package_manager: PackageManager::Cargo,
                command: "cargo".to_string(),
                args: vec!["install".to_string()],
                envs: BTreeMap::new(),
                elevation_required: false,
                duration_class: DurationClass::Slow,
                plan_id: uuid::Uuid::new_v4(),
            },
        }
    }

    #[test]
    fn ring_buffer_evicts_oldest_log() {
        let handle = RingBufferHandle::new();
        let id = AttemptId::new();

        handle.record(&make_started(id));
        for seq in 0..60 {
            handle.record(&make_log(id, seq));
        }

        let snap = handle.snapshot();
        assert_eq!(snap.logs.len(), MAX_LOG_ENTRIES);
        // Oldest should be seq=10 (0..9 evicted)
        if let InstallEvent::Log { seq, .. } = &snap.logs[0] {
            assert_eq!(*seq, 10);
        } else {
            panic!("expected Log event");
        }
    }

    #[test]
    fn ring_buffer_stores_terminal() {
        let handle = RingBufferHandle::new();
        let id = AttemptId::new();

        handle.record(&make_started(id));
        handle.record(&InstallEvent::Succeeded {
            attempt_id: id,
            duration_ms: 5000,
            installed_version: Some("0.5.3".to_string()),
        });

        let snap = handle.snapshot();
        assert!(snap.terminal.is_some());
        assert!(
            snap.terminal
                .as_ref()
                .expect("terminal event should be recorded")
                .is_terminal()
        );
    }

    #[test]
    fn ring_buffer_clears_on_new_attempt() {
        let handle = RingBufferHandle::new();
        let id1 = AttemptId::new();
        let id2 = AttemptId::new();

        handle.record(&make_started(id1));
        handle.record(&make_log(id1, 0));
        handle.record(&make_log(id1, 1));

        // New attempt clears old data
        handle.record(&make_started(id2));
        let snap = handle.snapshot();
        assert_eq!(snap.attempt_id, Some(id2));
        assert!(snap.logs.is_empty());
        assert!(snap.terminal.is_none());
    }

    #[test]
    fn redact_masks_token() {
        let line = "Authorization: Bearer sk-abc123xyz";
        let result = redact(line);
        assert!(result.contains("***"));
        assert!(!result.contains("sk-abc123xyz"));
    }

    #[test]
    fn redact_masks_key_value() {
        let line = "token=my_secret_token other_stuff";
        let result = redact(line);
        assert!(result.contains("token=***"));
        assert!(!result.contains("my_secret_token"));
    }

    #[test]
    fn redact_preserves_safe_lines() {
        let line = "Compiling llmusage v0.5.3";
        let result = redact(line);
        assert_eq!(result, line);
    }
}
