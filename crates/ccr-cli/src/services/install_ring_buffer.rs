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
    "bearer ",
    "api_key=",
    "apikey=",
];

/// Redact known credential patterns from a log line.
///
/// Replaces the value after the pattern with `***` up to the next whitespace or end of line.
pub fn redact(line: &str) -> String {
    let mut result = line.to_string();
    for pattern in CREDENTIAL_PATTERNS {
        let mut search_from = 0;
        while let Some(relative) = result[search_from..].to_ascii_lowercase().find(pattern) {
            let start = search_from + relative;
            let mut value_start = start + pattern.len();
            while matches!(result.as_bytes().get(value_start), Some(b' ' | b'\t')) {
                value_start += 1;
            }
            let quote = result[value_start..]
                .chars()
                .next()
                .filter(|ch| matches!(ch, '\'' | '"'));
            value_start += usize::from(quote.is_some());
            let mut escaped = false;
            let value_end = result[value_start..]
                .char_indices()
                .find_map(|(offset, ch)| {
                    let end = if let Some(quote) = quote {
                        let end = matches!(ch, '\n' | '\r') || (ch == quote && !escaped);
                        escaped = ch == '\\' && !escaped;
                        end
                    } else {
                        ch.is_whitespace() || matches!(ch, ',' | ';' | '"' | '\'')
                    };
                    end.then_some(value_start + offset)
                })
                .unwrap_or(result.len());

            if value_end > value_start {
                result.replace_range(value_start..value_end, "***");
                search_from = value_start + 3;
            } else {
                search_from = value_start;
            }
        }
    }

    // Keep the install contract's full masking (including legacy `key=`),
    // then cover quoted assignments/JSON/headers through the shared owner.
    ccr_core::core::log_redact::redact_log_text(&result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::install_types::{
        AttemptId, DurationClass, InstallActionKind, InstallPlanView, LogStream, PackageManager,
        PlanId, Platform,
    };

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
            plan: InstallPlanView {
                plan_id: PlanId::new(),
                platform: Platform::Macos,
                package_manager: PackageManager::Cargo,
                action: InstallActionKind::Cargo,
                expected_effects: vec!["Install llmusage".to_string()],
                elevation_required: false,
                duration_class: DurationClass::Slow,
                expires_at_ms: 1_700_000_120_000,
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

    #[test]
    fn regression_redact_multiple_credentials_and_unicode() {
        let result = redact("token=123456789012345678901234567890 password=secret");
        assert_eq!(result, "token=*** password=***");
        let result = redact(
            "İ准备😀 token=中文秘密值 password=another-secret key=legacy-secret token=again-secret",
        );
        assert_eq!(result, "İ准备😀 token=*** password=*** key=*** token=***");
        assert_eq!(redact("Preparing package 12345"), "Preparing package 12345");
    }

    #[test]
    fn regression_authorization_scheme_survives_until_shared_redaction() {
        for line in [
            "Authorization:Basic basic-secret-123",
            "Authorization:Bearer bearer-secret-456",
            "İ准备😀 Authorization:Basic basic-secret-123\nAuthorization:Bearer bearer-secret-456",
        ] {
            let result = redact(line);
            assert!(!result.contains("basic-secret"));
            assert!(!result.contains("bearer-secret"));
            assert!(result.contains("***") || result.contains("..."));
        }
    }

    #[test]
    fn legacy_credential_values_mask_quoted_escaped_repeated_and_unicode_values() {
        for pattern in CREDENTIAL_PATTERNS {
            for (value, expected) in [
                ("plain-private123", "***"),
                ("\"visible private123\"", "\"***\""),
                ("'visible private123'", "'***'"),
                (r#""visible \"escaped\" private123""#, "\"***\""),
                (" \t\"密 😀 private123\"", " \t\"***\""),
                ("\"unfinished private123", "\"***"),
                ("\"\"", "\"\""),
            ] {
                let line = format!("safe {pattern}{value}; other=preserved");
                let expected = if value.starts_with("\"unfinished") {
                    format!("safe {pattern}{expected}")
                } else {
                    format!("safe {pattern}{expected}; other=preserved")
                };
                assert_eq!(redact(&line), expected, "pattern={pattern} value={value}");
            }
            let line =
                format!("İ准备😀 {pattern}\"first private123\" {pattern}'second private456'");
            assert_eq!(
                redact(&line),
                format!("İ准备😀 {pattern}\"***\" {pattern}'***'")
            );
        }
        assert_eq!(
            redact("Compile package v1.2.3; value remains visible"),
            "Compile package v1.2.3; value remains visible"
        );
    }
}
