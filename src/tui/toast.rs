// TUI Toast notification system — transient messages with auto-expiry

use std::time::{Duration, Instant};

/// Toast severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Success,
    Error,
    Warning,
    Info,
}

/// A transient notification message.
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub born: Instant,
    pub ttl: Duration,
}

impl Toast {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Success,
            born: Instant::now(),
            ttl: Duration::from_secs(3),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Error,
            born: Instant::now(),
            ttl: Duration::from_secs(5),
        }
    }

    #[allow(dead_code)]
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Warning,
            born: Instant::now(),
            ttl: Duration::from_secs(4),
        }
    }

    #[allow(dead_code)]
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: ToastKind::Info,
            born: Instant::now(),
            ttl: Duration::from_secs(3),
        }
    }

    /// Whether this toast has expired.
    pub fn is_expired(&self) -> bool {
        self.born.elapsed() >= self.ttl
    }
}

/// Manages a queue of toast notifications.
#[derive(Debug, Default)]
pub struct ToastManager {
    toasts: Vec<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { toasts: Vec::new() }
    }

    /// Push a new toast.
    pub fn push(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    /// Garbage-collect expired toasts. Returns `true` if any were removed (needs redraw).
    pub fn tick(&mut self) -> bool {
        let before = self.toasts.len();
        self.toasts.retain(|t| !t.is_expired());
        self.toasts.len() != before
    }

    /// The most recent active toast (displayed at bottom of screen).
    pub fn active(&self) -> Option<&Toast> {
        self.toasts.last()
    }

    /// Whether there are any active toasts.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.toasts.is_empty()
    }
}
