use super::log_bridge::BridgeEnqueueLayer;
use super::log_redact::{is_sensitive_log_key, redact_log_text};
use super::log_writer::{
    SecureDailyWriter, is_managed_log_file, set_owner_only_dir, tighten_existing_log_files,
};
use crate::utils::mask_sensitive;
use colored::*;
use std::fmt as std_fmt;
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tracing::field::{Field, Visit};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_log::LogTracer;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::format::{FmtSpan, FormatFields, Writer};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub use super::log_bridge::{
    BridgedLogEvent, EnqueueResult, close_bridged_log_sender, current_log_correlation_id,
    dropped_bridged_log_count, enter_bridge_consumer, take_bridged_log_receiver,
    try_enqueue_bridged_log,
};

static LOG_GUARDS: OnceLock<Mutex<Vec<WorkerGuard>>> = OnceLock::new();

pub struct ColorOutput;

/// Semantic state for a human-readable CLI message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputStatus {
    Success,
    Warning,
    Error,
    Step,
    Skipped,
}

struct OutputCapabilities {
    symbols: bool,
    style: bool,
}

impl OutputCapabilities {
    fn for_target(is_terminal: bool) -> Self {
        let dumb = std::env::var("TERM").is_ok_and(|term| term == "dumb");
        // Force permits styling a redirected stream. colored owns color precedence.
        let force = std::env::var("CLICOLOR_FORCE").is_ok_and(|value| value != "0");
        Self {
            symbols: is_terminal && !dumb,
            style: !dumb
                && (is_terminal || force)
                && colored::control::SHOULD_COLORIZE.should_colorize(),
        }
    }
}

fn format_status(status: OutputStatus, msg: &str, capabilities: OutputCapabilities) -> String {
    let (marker, label) = match (status, capabilities.symbols) {
        (OutputStatus::Success, true) => ("✓", ""),
        (OutputStatus::Warning, true) => ("!", " 警告:"),
        (OutputStatus::Error, true) => ("×", " 错误:"),
        (OutputStatus::Step, true) => ("→", ""),
        (OutputStatus::Skipped, true) => ("-", " 跳过:"),
        (OutputStatus::Success, false) => ("成功:", ""),
        (OutputStatus::Warning, false) => ("警告:", ""),
        (OutputStatus::Error, false) => ("错误:", ""),
        (OutputStatus::Step, false) => ("进度:", ""),
        (OutputStatus::Skipped, false) => ("跳过:", ""),
    };
    let marker = if capabilities.style {
        match status {
            OutputStatus::Success => marker.green().bold().to_string(),
            OutputStatus::Warning => marker.yellow().bold().to_string(),
            OutputStatus::Error => marker.red().bold().to_string(),
            OutputStatus::Step => marker.cyan().bold().to_string(),
            OutputStatus::Skipped => marker.to_string(),
        }
    } else {
        marker.to_string()
    };
    format!("{marker}{label} {msg}")
}

fn format_key_value(key: &str, value: &str, indent: usize, style: bool) -> String {
    let padding = " ".repeat(indent);
    let key = if style {
        key.bold().to_string()
    } else {
        key.to_string()
    };
    let mut lines = value.split('\n');
    let mut output = format!("{padding}{key}: {}", lines.next().unwrap_or_default());
    for line in lines {
        output.push_str(&format!("\n{padding}  {line}"));
    }
    output
}

#[allow(dead_code)]
impl ColorOutput {
    /// Configure colored once at CLI startup for terminals without ANSI support.
    pub fn configure_cli_output() {
        if std::env::var("TERM").is_ok_and(|term| term == "dumb") {
            colored::control::set_override(false);
        }
    }

    /// Format a status without selecting an output stream.
    ///
    /// `is_terminal` describes the stream that will receive the message.
    pub fn format_status(status: OutputStatus, msg: &str, is_terminal: bool) -> String {
        format_status(status, msg, OutputCapabilities::for_target(is_terminal))
    }

    pub fn success(msg: &str) {
        println!(
            "{}",
            Self::format_status(OutputStatus::Success, msg, io::stdout().is_terminal())
        );
    }

    pub fn info(msg: &str) {
        println!("{msg}");
    }

    pub fn warning(msg: &str) {
        println!(
            "{}",
            Self::format_status(OutputStatus::Warning, msg, io::stdout().is_terminal())
        );
    }

    pub fn error(msg: &str) {
        eprintln!(
            "{}",
            Self::format_status(OutputStatus::Error, msg, io::stderr().is_terminal())
        );
    }

    pub fn step(msg: &str) {
        println!(
            "{}",
            Self::format_status(OutputStatus::Step, msg, io::stdout().is_terminal())
        );
    }

    pub fn title(msg: &str) {
        println!("\n{}", msg.blue().bold());
        println!("{}", "=".repeat(msg.chars().count()).blue());
    }

    pub fn banner(version: &str) {
        println!(
            "{}",
            format!(
                "\n==============================\nCCR (Rust)\nVersion: {}\n==============================",
                version
            )
            .cyan()
        );
    }

    pub fn separator() {
        println!("{}", "-".repeat(60).dimmed());
    }

    pub fn mask_sensitive(value: &str) -> String {
        crate::utils::mask_sensitive(value)
    }

    pub fn key_value(key: &str, value: &str, indent: usize) {
        println!(
            "{}",
            format_key_value(
                key,
                value,
                indent,
                OutputCapabilities::for_target(io::stdout().is_terminal()).style,
            )
        );
    }

    pub fn key_value_sensitive(key: &str, value: &str, indent: usize) {
        let masked = Self::mask_sensitive(value);
        Self::key_value(key, &masked, indent);
    }

    pub fn current_marker() -> String {
        "*".green().bold().to_string()
    }

    pub fn normal_marker() -> String {
        " ".to_string()
    }

    pub fn ask_confirmation(question: &str, default: bool) -> bool {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} {} [{}]: ", "?".yellow().bold(), question, default_str);
        if let Err(err) = io::stdout().flush() {
            tracing::warn!(error = %err, "刷新确认提示输出失败，返回默认值");
            return default;
        }

        let mut input = String::new();
        if let Err(err) = io::stdin().read_line(&mut input) {
            tracing::warn!(error = %err, "读取确认输入失败，返回默认值");
            return default;
        }
        let input = input.trim().to_lowercase();

        if input.is_empty() {
            default
        } else {
            matches!(input.as_str(), "y" | "yes" | "true" | "1")
        }
    }

    pub fn config_status(name: &str, is_current: bool, description: Option<&str>) {
        let marker = if is_current {
            Self::current_marker()
        } else {
            Self::normal_marker()
        };

        let desc_str = description
            .map(|desc| format!(" - {}", desc.dimmed()))
            .unwrap_or_default();

        println!("{} {}{}", marker, name, desc_str);
    }

    pub fn env_status(var_name: &str, value: Option<&str>, is_sensitive: bool) {
        match value {
            Some(current) => {
                if is_sensitive {
                    Self::key_value_sensitive(var_name, current, 2);
                } else {
                    Self::key_value(var_name, current, 2);
                }
            }
            None => {
                let padding = "  ";
                println!("{}{}: {}", padding, var_name.bold(), "(not set)".yellow());
            }
        }
    }
}

fn get_log_dir() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("CCR_ROOT").filter(|value| !value.is_empty()) {
        return Some(PathBuf::from(root).join("logs"));
    }
    dirs::home_dir().map(|home| home.join(".ccr").join("logs"))
}

fn cleanup_old_logs(log_dir: &std::path::Path) {
    const MAX_AGE_DAYS: u64 = 14;
    let max_age = std::time::Duration::from_secs(MAX_AGE_DAYS * 24 * 60 * 60);

    let entries = match std::fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    let now = std::time::SystemTime::now();

    for entry in entries.flatten() {
        let path = entry.path();
        if !is_managed_log_file(&path) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };

        let modified = match metadata.modified() {
            Ok(modified) => modified,
            Err(_) => continue,
        };

        if let Ok(age) = now.duration_since(modified)
            && age > max_age
        {
            let _ = std::fs::remove_file(&path);
        }
    }
}

const THIRD_PARTY_WARN: &str = "hyper=warn,reqwest=warn,h2=warn,rustls=warn,tokio=warn";
const DEFAULT_LOG_FILTER: &str = "info,hyper=warn,reqwest=warn,h2=warn,rustls=warn,tokio=warn";

fn resolve_log_filter() -> String {
    let raw = std::env::var("CCR_LOG_LEVEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("RUST_LOG")
                .ok()
                .filter(|value| !value.trim().is_empty())
        });

    match raw {
        None => DEFAULT_LOG_FILTER.to_string(),
        Some(value) => normalize_filter_directive(&value),
    }
}

fn normalize_filter_directive(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DEFAULT_LOG_FILTER.to_string();
    }

    if is_bare_level(trimmed) {
        return format!("{},{THIRD_PARTY_WARN}", trimmed.to_ascii_lowercase());
    }

    match EnvFilter::try_new(trimmed) {
        Ok(_) => trimmed.to_string(),
        Err(_) => DEFAULT_LOG_FILTER.to_string(),
    }
}

fn is_bare_level(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "trace" | "debug" | "info" | "warn" | "error" | "off"
    )
}

struct RedactingFormat;

impl<'writer> FormatFields<'writer> for RedactingFormat {
    fn format_fields<R: RecordFields>(
        &self,
        writer: Writer<'writer>,
        fields: R,
    ) -> std_fmt::Result {
        let mut visitor = RedactingVisitor {
            writer,
            result: Ok(()),
            first: true,
        };
        fields.record(&mut visitor);
        visitor.result
    }
}

struct RedactingVisitor<'a> {
    writer: Writer<'a>,
    result: std_fmt::Result,
    first: bool,
}

impl RedactingVisitor<'_> {
    fn write_field(&mut self, name: &str, value: &str) {
        if self.result.is_err() {
            return;
        }
        let redacted = if is_sensitive_log_key(name) {
            mask_sensitive(value)
        } else {
            redact_log_text(value)
        };
        let prefix = if self.first { "" } else { " " };
        self.first = false;
        if name == "message" {
            self.result = write!(self.writer, "{prefix}{redacted}");
        } else {
            self.result = write!(self.writer, "{prefix}{name}={redacted}");
        }
    }
}

impl Visit for RedactingVisitor<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std_fmt::Debug) {
        self.write_field(field.name(), &format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.write_field(field.name(), value);
    }
}

fn store_worker_guard(guard: WorkerGuard) {
    let guards = LOG_GUARDS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut guards) = guards.lock() {
        guards.push(guard);
    }
}

fn build_file_writer() -> Option<NonBlocking> {
    let log_dir = get_log_dir()?;
    if std::fs::create_dir_all(&log_dir).is_err() {
        return None;
    }
    if set_owner_only_dir(&log_dir).is_err() {
        return None;
    }

    cleanup_old_logs(&log_dir);
    tighten_existing_log_files(&log_dir);

    let writer = SecureDailyWriter::new(log_dir);
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);
    store_worker_guard(guard);
    Some(non_blocking)
}

fn build_env_filter() -> EnvFilter {
    EnvFilter::try_new(resolve_log_filter()).unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER))
}

pub fn init_logger() {
    let _ = LogTracer::init();
    super::log_bridge::ensure_log_bridge_queue();
    let _ = current_log_correlation_id();

    let env_filter = build_env_filter();
    let stdout_layer = fmt::layer()
        .fmt_fields(RedactingFormat)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE)
        .with_ansi(true);
    let file_layer = build_file_writer().map(|writer| {
        fmt::layer()
            .fmt_fields(RedactingFormat)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_span_events(FmtSpan::NONE)
            .with_ansi(false)
            .with_writer(writer)
    });

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .with(BridgeEnqueueLayer)
        .try_init();
}

pub fn init_file_only_logger() {
    let _ = LogTracer::init();
    super::log_bridge::ensure_log_bridge_queue();
    let _ = current_log_correlation_id();

    let env_filter = build_env_filter();
    let file_layer = build_file_writer().map(|writer| {
        fmt::layer()
            .fmt_fields(RedactingFormat)
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_span_events(FmtSpan::NONE)
            .with_ansi(false)
            .with_writer(writer)
    });

    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(BridgeEnqueueLayer)
        .try_init();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestLogEnv;
    use std::ffi::OsStr;
    use std::fs::{File, FileTimes};
    use std::time::{Duration, SystemTime};

    #[test]
    fn log_dir_uses_nonempty_ccr_root() {
        let mut env = TestLogEnv::new();
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("custom-ccr");
        env.set_env("CCR_ROOT", root.as_os_str());

        assert_eq!(get_log_dir(), Some(root.join("logs")));
        assert!(!root.exists());
    }

    #[test]
    fn empty_or_unset_log_root_retains_system_home_fallback() {
        let mut env = TestLogEnv::new();
        let expected = dirs::home_dir().map(|home| home.join(".ccr/logs"));
        env.remove_env("CCR_ROOT");
        assert_eq!(get_log_dir(), expected);
        env.set_env("CCR_ROOT", OsStr::new(""));
        assert_eq!(get_log_dir(), expected);
    }

    #[test]
    fn test_resolve_log_filter_precedence() {
        let mut env = TestLogEnv::new();

        env.set_env("RUST_LOG", OsStr::new("warn"));
        env.remove_env("CCR_LOG_LEVEL");
        assert_eq!(resolve_log_filter(), format!("warn,{THIRD_PARTY_WARN}"));

        env.set_env("CCR_LOG_LEVEL", OsStr::new("debug"));
        assert_eq!(resolve_log_filter(), format!("debug,{THIRD_PARTY_WARN}"));
    }

    #[test]
    fn test_resolve_log_filter_invalid_falls_back() {
        let mut env = TestLogEnv::new();
        env.remove_env("RUST_LOG");
        env.set_env("CCR_LOG_LEVEL", OsStr::new("!!!not-a-filter"));
        assert_eq!(resolve_log_filter(), DEFAULT_LOG_FILTER);
    }

    #[test]
    fn test_resolve_log_filter_keeps_full_directive() {
        let mut env = TestLogEnv::new();
        env.remove_env("RUST_LOG");
        env.set_env("CCR_LOG_LEVEL", OsStr::new("ccr_core=debug,hyper=error"));
        assert_eq!(resolve_log_filter(), "ccr_core=debug,hyper=error");
    }

    #[test]
    fn test_mask_sensitive() {
        assert_eq!(
            ColorOutput::mask_sensitive("sk-ant-1234567890abcdef"),
            "sk-a...cdef"
        );
        assert_eq!(ColorOutput::mask_sensitive("short"), "*****");
        assert_eq!(ColorOutput::mask_sensitive(""), "");
    }

    #[test]
    fn test_output_methods() {
        ColorOutput::success("Success message");
        ColorOutput::info("Info message");
        ColorOutput::warning("Warning message");
        ColorOutput::error("Error message");
        ColorOutput::step("Step message");
        ColorOutput::separator();
    }

    #[test]
    fn status_words_and_symbols_keep_distinct_meanings() {
        for (status, terminal, plain) in [
            (OutputStatus::Success, "✓ message", "成功: message"),
            (OutputStatus::Warning, "! 警告: message", "警告: message"),
            (OutputStatus::Error, "× 错误: message", "错误: message"),
            (OutputStatus::Step, "→ message", "进度: message"),
            (OutputStatus::Skipped, "- 跳过: message", "跳过: message"),
        ] {
            for (symbols, expected) in [(true, terminal), (false, plain)] {
                assert_eq!(
                    format_status(
                        status,
                        "message",
                        OutputCapabilities {
                            symbols,
                            style: false,
                        },
                    ),
                    expected
                );
            }
        }
    }

    #[test]
    fn status_keeps_empty_long_and_multiline_message_content() {
        for msg in [
            "".to_string(),
            "内容".repeat(256),
            "first\nsecond".to_string(),
        ] {
            assert_eq!(
                format_status(
                    OutputStatus::Success,
                    &msg,
                    OutputCapabilities {
                        symbols: false,
                        style: false,
                    },
                ),
                format!("成功: {msg}")
            );
        }
    }

    #[test]
    fn fields_indent_each_continuation_and_keep_empty_lines() {
        assert_eq!(
            format_key_value("描述", "first\nsecond\n\nfourth\n", 2, false),
            "  描述: first\n    second\n    \n    fourth\n    "
        );
        assert_eq!(format_key_value("邮箱", "", 0, false), "邮箱: ");
        let long_value = "a".repeat(256);
        assert_eq!(
            format_key_value("路径", &long_value, 2, false),
            format!("  路径: {long_value}")
        );
    }

    #[test]
    fn test_markers() {
        assert!(!ColorOutput::current_marker().is_empty());
        assert_eq!(ColorOutput::normal_marker(), " ");
    }

    #[test]
    fn managed_log_name_matches_daily_files() {
        assert!(!is_managed_log_file(std::path::Path::new("ccr.log")));
        assert!(is_managed_log_file(std::path::Path::new(
            "ccr.log.2026-07-12"
        )));
        assert!(!is_managed_log_file(std::path::Path::new("other.log")));
        assert!(!is_managed_log_file(std::path::Path::new("ccr.txt")));
    }

    #[test]
    fn cleanup_removes_only_expired_ccr_rolling_logs() {
        let temp = tempfile::tempdir().expect("temp log dir should be created");
        let expired = temp.path().join("ccr.log.2026-01-01");
        let recent = temp.path().join("ccr.log.2026-07-12");
        let unrelated = temp.path().join("other.log.2026-01-01");

        File::create(&expired).expect("expired log should be created");
        File::create(&recent).expect("recent log should be created");
        File::create(&unrelated).expect("unrelated log should be created");

        let old_time = SystemTime::now() - Duration::from_secs(15 * 24 * 60 * 60);
        for path in [&expired, &unrelated] {
            let file = File::options()
                .write(true)
                .open(path)
                .expect("test log should open");
            file.set_times(FileTimes::new().set_modified(old_time))
                .expect("test log timestamp should update");
        }

        cleanup_old_logs(temp.path());

        assert!(!expired.exists());
        assert!(recent.exists());
        assert!(unrelated.exists());
    }
}
