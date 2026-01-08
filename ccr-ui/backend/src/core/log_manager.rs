//! Log Manager Module
//!
//! Provides log configuration, directory management, and automatic cleanup functionality.
//! Supports configuration via environment variables with sensible defaults.

use chrono::{DateTime, Duration, Local};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

/// Log configuration structure
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub log_dir: PathBuf,
    pub retention_days: u32,
    pub log_level: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_dir: get_default_log_dir(),
            retention_days: 14,
            log_level: "info".to_string(),
        }
    }
}

impl LogConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(dir) = std::env::var("CCR_LOG_DIR")
            && !dir.is_empty()
        {
            config.log_dir = PathBuf::from(dir);
        }

        if let Ok(days_str) = std::env::var("CCR_LOG_RETENTION_DAYS")
            && let Ok(days) = days_str.parse::<u32>()
            && (1..=365).contains(&days)
        {
            config.retention_days = days;
        }

        if let Ok(level) = std::env::var("CCR_LOG_LEVEL")
            && !level.is_empty()
        {
            config.log_level = level;
        } else if let Ok(level) = std::env::var("RUST_LOG")
            && !level.is_empty()
        {
            config.log_level = level;
        }

        config
    }

    /// Check if the configuration is valid
    #[allow(dead_code)]
    pub fn is_valid(&self) -> bool {
        (1..=365).contains(&self.retention_days)
    }
}

fn get_default_log_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("logs")))
        .unwrap_or_else(|| PathBuf::from("logs"))
}

/// Create log directories synchronously (for startup before async runtime)
pub fn create_log_directories_sync(base_dir: &Path) -> std::io::Result<()> {
    let backend_dir = base_dir.join("backend");
    let frontend_dir = base_dir.join("frontend");
    std::fs::create_dir_all(&backend_dir)?;
    std::fs::create_dir_all(&frontend_dir)?;
    Ok(())
}

/// Spawn an async task to clean up old log files
pub fn spawn_cleanup_task(log_dir: PathBuf, retention_days: u32) {
    tokio::spawn(async move {
        if let Err(e) = cleanup_old_logs(&log_dir, retention_days).await {
            warn!("Failed to cleanup old logs: {}", e);
        }
    });
}

/// Delete log files older than the retention period
pub async fn cleanup_old_logs(log_dir: &Path, retention_days: u32) -> std::io::Result<()> {
    let cutoff = Local::now() - Duration::days(retention_days as i64);
    let mut deleted_count = 0;
    let mut failed_count = 0;

    for subdir in &["backend", "frontend"] {
        let dir = log_dir.join(subdir);
        if !dir.exists() {
            continue;
        }

        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to read log directory {:?}: {}", dir, e);
                continue;
            }
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            if let Ok(metadata) = entry.metadata().await
                && let Ok(modified) = metadata.modified()
            {
                let modified_time: DateTime<Local> = modified.into();
                if modified_time < cutoff {
                    match fs::remove_file(&path).await {
                        Ok(_) => deleted_count += 1,
                        Err(e) => {
                            warn!("Failed to delete old log {:?}: {}", path, e);
                            failed_count += 1;
                        }
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        info!("Cleaned up {} old log file(s)", deleted_count);
    }

    if failed_count > 0 {
        warn!(
            "Failed to delete {} log file(s) during cleanup",
            failed_count
        );
    }

    Ok(())
}

/// Strip ANSI escape sequences from a string
///
/// This function removes all ANSI escape codes (colors, formatting, cursor movements)
/// from the input string while preserving the actual content.
#[allow(dead_code)]
pub fn strip_ansi(input: &str) -> String {
    // Matches ANSI escape sequences: ESC[...m, ESC[...H, ESC]...BEL, etc.
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // ESC character, start of escape sequence
            match chars.peek() {
                Some('[') => {
                    // CSI sequence: ESC[...letter
                    chars.next(); // consume '['
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next.is_ascii_alphabetic() || next == '~' {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC sequence: ESC]...BEL or ESC]...ESC\
                    chars.next(); // consume ']'
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\x07' {
                            // BEL
                            break;
                        }
                        if next == '\x1b' {
                            // Check for ST (ESC\)
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    // Unknown escape sequence, skip one more character
                    chars.next();
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_simple_color() {
        let input = "\x1b[32mgreen text\x1b[0m";
        assert_eq!(strip_ansi(input), "green text");
    }

    #[test]
    fn test_strip_ansi_multiple_codes() {
        let input = "\x1b[1;31mbold red\x1b[0m and \x1b[34mblue\x1b[0m";
        assert_eq!(strip_ansi(input), "bold red and blue");
    }

    #[test]
    fn test_strip_ansi_256_color() {
        let input = "\x1b[38;5;196mred 256\x1b[0m";
        assert_eq!(strip_ansi(input), "red 256");
    }

    #[test]
    fn test_strip_ansi_no_escape() {
        let input = "plain text without escape";
        assert_eq!(strip_ansi(input), input);
    }

    #[test]
    fn test_strip_ansi_unicode() {
        let input = "\x1b[32m中文文本\x1b[0m emoji 🎉";
        assert_eq!(strip_ansi(input), "中文文本 emoji 🎉");
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.retention_days, 14);
        assert_eq!(config.log_level, "info");
        assert!(config.is_valid());
    }

    #[test]
    fn test_log_config_validation() {
        // Test invalid retention_days = 0
        let config_zero = LogConfig {
            retention_days: 0,
            ..LogConfig::default()
        };
        assert!(!config_zero.is_valid());

        // Test invalid retention_days = 366
        let config_over = LogConfig {
            retention_days: 366,
            ..LogConfig::default()
        };
        assert!(!config_over.is_valid());

        // Test valid retention_days = 30
        let config_valid = LogConfig {
            retention_days: 30,
            ..LogConfig::default()
        };
        assert!(config_valid.is_valid());
    }
}
