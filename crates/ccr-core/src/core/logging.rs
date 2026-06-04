use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

static LOG_GUARDS: OnceLock<Mutex<Vec<WorkerGuard>>> = OnceLock::new();

pub struct ColorOutput;

#[allow(dead_code)]
impl ColorOutput {
    pub fn success(msg: &str) {
        println!("{} {}", "[OK]".green().bold(), msg.green());
    }

    pub fn info(msg: &str) {
        println!("{} {}", "[INFO]".blue().bold(), msg);
    }

    pub fn warning(msg: &str) {
        println!("{} {}", "[WARN]".yellow().bold(), msg.yellow());
    }

    pub fn error(msg: &str) {
        eprintln!("{} {}", "[ERR]".red().bold(), msg.red());
    }

    pub fn step(msg: &str) {
        println!("{} {}", "[STEP]".cyan().bold(), msg.cyan());
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
        let padding = " ".repeat(indent);
        println!("{}{}: {}", padding, key.bold(), value);
    }

    pub fn key_value_sensitive(key: &str, value: &str, indent: usize) {
        let padding = " ".repeat(indent);
        let masked = Self::mask_sensitive(value);
        println!("{}{}: {}", padding, key.bold(), masked.dimmed());
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
        if path.extension().and_then(|item| item.to_str()) != Some("log") {
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

fn resolve_log_filter() -> String {
    std::env::var("CCR_LOG_LEVEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("RUST_LOG")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "info".to_string())
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

    cleanup_old_logs(&log_dir);

    let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "ccr.log");
    let (writer, guard) = tracing_appender::non_blocking(file_appender);
    store_worker_guard(guard);
    Some(writer)
}

pub fn init_logger() {
    let _ = LogTracer::init();

    let log_level = resolve_log_filter();
    let env_filter = EnvFilter::new(log_level);

    let stdout_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE)
        .with_ansi(true);

    let file_layer = build_file_writer().map(|writer| {
        fmt::layer()
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
        .try_init();
}

pub fn init_file_only_logger() {
    let _ = LogTracer::init();

    let log_level = resolve_log_filter();
    let env_filter = EnvFilter::new(log_level);

    let file_layer = build_file_writer().map(|writer| {
        fmt::layer()
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
        .try_init();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_support::TestLogEnv;
    use std::ffi::OsStr;

    #[test]
    fn test_resolve_log_filter_precedence() {
        let mut env = TestLogEnv::new();

        env.set_env("RUST_LOG", OsStr::new("warn"));
        env.remove_env("CCR_LOG_LEVEL");
        assert_eq!(resolve_log_filter(), "warn");

        env.set_env("CCR_LOG_LEVEL", OsStr::new("debug"));
        assert_eq!(resolve_log_filter(), "debug");
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
    fn test_markers() {
        assert!(!ColorOutput::current_marker().is_empty());
        assert_eq!(ColorOutput::normal_marker(), " ");
    }
}
