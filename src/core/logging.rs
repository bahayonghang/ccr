// 🎨 CCR 日志与彩色输出模块
// 💬 提供统一的日志初始化和彩色终端输出工具
//
// 核心功能:
// - 🌈 彩色终端输出(使用 colored crate)
// - 📝 统一的消息格式(成功/错误/警告/信息)
// - 🔐 敏感信息自动掩码
// - 📊 键值对格式化输出
// - 🎯 交互式确认提示
// - 📚 日志级别控制(通过环境变量)
// - 📁 日志文件持久化(按天轮转，保留 14 天)

use colored::*;
use std::io::{self, Write};
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_log::LogTracer;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// 🎨 彩色输出工具
///
/// 提供各种格式化的彩色输出方法,用于改善用户体验
///
/// 消息类型:
/// - ✅ success: 绿色(操作成功)
/// - ℹ️ info: 蓝色(一般信息)
/// - ⚠️ warning: 黄色(警告信息)
/// - ❌ error: 红色(错误信息)
/// - ▶️ step: 青色(步骤提示)
pub struct ColorOutput;

impl ColorOutput {
    /// 输出成功消息 (绿色)
    pub fn success(msg: &str) {
        println!("{} {}", "✓".green().bold(), msg.green());
    }

    /// 输出信息消息 (蓝色)
    pub fn info(msg: &str) {
        println!("{} {}", "ℹ".blue().bold(), msg);
    }

    /// 输出警告消息 (黄色)
    pub fn warning(msg: &str) {
        println!("{} {}", "⚠".yellow().bold(), msg.yellow());
    }

    /// 输出错误消息 (红色)
    pub fn error(msg: &str) {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    }

    /// 输出步骤消息 (青色)
    pub fn step(msg: &str) {
        println!("{} {}", "▶".cyan().bold(), msg.cyan());
    }

    /// 输出标题 (粗体蓝色)
    pub fn title(msg: &str) {
        println!("\n{}", msg.blue().bold());
        println!("{}", "═".repeat(msg.chars().count()).blue());
    }

    /// 输出 Banner
    pub fn banner(version: &str) {
        let banner = format!(
            r#"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║   ██████╗  ██████╗██████╗                                   ║
║  ██╔════╝ ██╔════╝██╔══██╗                                  ║
║  ██║      ██║     ██████╔╝                                  ║
║  ██║      ██║     ██╔══██╗                                  ║
║  ╚██████╗ ╚██████╗██║  ██║                                  ║
║   ╚═════╝  ╚═════╝╚═╝  ╚═╝                                  ║
║                                                              ║
║  Claude Code Configuration Switcher (Rust Version)          ║
║  Version: {:<50} ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#,
            version
        );
        println!("{}", banner.cyan());
    }

    /// 输出分隔符
    pub fn separator() {
        println!("{}", "─".repeat(60).dimmed());
    }

    /// 🔐 掩码敏感信息
    ///
    /// 将敏感信息(如 API Token)进行部分隐藏显示
    ///
    /// 掩码规则:
    /// - 长度 <= 10: 全部替换为 *
    /// - 长度 > 10: 显示前 4 位和后 4 位,中间用 ... 代替
    ///
    /// 示例:
    /// - "sk-ant-1234567890abcdef" → "sk-a...cdef"
    /// - "short" → "*****"
    pub fn mask_sensitive(value: &str) -> String {
        crate::utils::mask_sensitive(value)
    }

    /// 📊 输出键值对
    pub fn key_value(key: &str, value: &str, indent: usize) {
        let padding = " ".repeat(indent);
        println!("{}{}: {}", padding, key.bold(), value);
    }

    /// 🔐 输出键值对(敏感信息自动掩码)
    #[allow(dead_code)]
    pub fn key_value_sensitive(key: &str, value: &str, indent: usize) {
        let padding = " ".repeat(indent);
        let masked = Self::mask_sensitive(value);
        println!("{}{}: {}", padding, key.bold(), masked.dimmed());
    }

    /// ▶️ 输出当前配置标记(带颜色)
    #[allow(dead_code)]
    pub fn current_marker() -> String {
        "▶".green().bold().to_string()
    }

    /// ○ 输出普通项目标记
    #[allow(dead_code)]
    pub fn normal_marker() -> String {
        " ".to_string()
    }

    /// 🤔 询问用户确认(是/否)
    ///
    /// 支持多种输入格式: y/yes/是
    pub fn ask_confirmation(question: &str, default: bool) -> bool {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} {} [{}]: ", "?".yellow().bold(), question, default_str);
        io::stdout().flush().expect("无法刷新标准输出");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("无法读取用户输入");
        let input = input.trim().to_lowercase();

        if input.is_empty() {
            default
        } else {
            matches!(input.as_str(), "y" | "yes" | "是")
        }
    }

    /// 输出配置节状态
    #[allow(dead_code)]
    pub fn config_status(name: &str, is_current: bool, description: Option<&str>) {
        let marker = if is_current {
            Self::current_marker()
        } else {
            Self::normal_marker()
        };

        let desc_str = description
            .map(|d| format!(" - {}", d.dimmed()))
            .unwrap_or_default();

        println!("{} {}{}", marker, name, desc_str);
    }

    /// 输出环境变量状态
    #[allow(dead_code)]
    pub fn env_status(var_name: &str, value: Option<&str>, is_sensitive: bool) {
        match value {
            Some(v) => {
                if is_sensitive {
                    Self::key_value_sensitive(var_name, v, 2);
                } else {
                    Self::key_value(var_name, v, 2);
                }
            }
            None => {
                let padding = "  ";
                println!("{}{}: {}", padding, var_name.bold(), "(未设置)".yellow());
            }
        }
    }
}

/// 📁 获取日志目录路径
///
/// 返回 `~/.ccr/logs/` 目录路径
fn get_log_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".ccr").join("logs"))
}

/// 🧹 清理过期日志文件
///
/// 删除修改时间超过 14 天的日志文件
///
/// # 参数
/// - `log_dir`: 日志目录路径
fn cleanup_old_logs(log_dir: &std::path::Path) {
    const MAX_AGE_DAYS: u64 = 14;
    let max_age = std::time::Duration::from_secs(MAX_AGE_DAYS * 24 * 60 * 60);

    let entries = match std::fs::read_dir(log_dir) {
        Ok(entries) => entries,
        Err(_) => return, // 无法读取目录，静默返回
    };

    let now = std::time::SystemTime::now();

    for entry in entries.flatten() {
        let path = entry.path();

        // 只处理 .log 文件
        if path.extension().and_then(|s| s.to_str()) != Some("log") {
            continue;
        }

        // 获取文件修改时间
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let modified = match metadata.modified() {
            Ok(t) => t,
            Err(_) => continue,
        };

        // 检查是否超过保留期限
        if let Ok(age) = now.duration_since(modified)
            && age > max_age
        {
            // 尝试删除，失败时静默忽略
            let _ = std::fs::remove_file(&path);
        }
    }
}

/// 🔧 初始化日志系统
///
/// 使用环境变量控制日志行为
///
/// 环境变量:
/// - CCR_LOG_LEVEL: 日志级别 (trace, debug, info, warn, error)
///
/// 默认配置:
/// - 级别: info
/// - 终端: 彩色输出
/// - 文件: ~/.ccr/logs/ccr.YYYY-MM-DD.log
///
/// 日志输出:
/// - 终端: 带 ANSI 彩色
/// - 文件: 纯文本，按天轮转，保留 14 天
///
/// 日志格式:
/// - 时间戳 [ccr] 级别 消息内容
pub fn init_logger() {
    // 初始化 log -> tracing 桥接，让依赖库的 log 日志也能被捕获
    // 忽略错误（可能已初始化）
    let _ = LogTracer::init();

    // 从环境变量获取日志级别，默认 info
    let log_level = std::env::var("CCR_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    // 应用到所有 crate，不仅限于 ccr
    let env_filter = EnvFilter::new(log_level);

    // 终端输出层（带彩色）
    let stdout_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .with_span_events(FmtSpan::NONE)
        .with_ansi(true);

    // 尝试创建文件日志层
    let file_layer = get_log_dir().and_then(|log_dir| {
        // 确保日志目录存在
        if std::fs::create_dir_all(&log_dir).is_err() {
            return None;
        }

        // 清理过期日志
        cleanup_old_logs(&log_dir);

        // 创建按天轮转的文件 appender
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "ccr.log");

        // 文件输出层（无色彩）
        Some(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .with_span_events(FmtSpan::NONE)
                .with_ansi(false)
                .with_writer(file_appender),
        )
    });

    // 组合层并初始化（使用 try_init 避免重复初始化时 panic）
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .try_init();
}

/// 🔇 初始化仅文件输出的日志系统（TUI 模式专用）
///
/// TUI 模式下需要禁用终端日志输出，避免日志覆盖 TUI 界面。
/// 此函数仅将日志写入文件，不输出到终端。
///
/// 注意：由于 tracing 只能初始化一次，此函数会静默失败（如果已初始化）
pub fn init_file_only_logger() {
    // 初始化 log -> tracing 桥接
    let _ = LogTracer::init();

    // 从环境变量获取日志级别，默认 info
    let log_level = std::env::var("CCR_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    let env_filter = EnvFilter::new(log_level);

    // 尝试创建文件日志层（无终端输出）
    let file_layer = get_log_dir().and_then(|log_dir| {
        // 确保日志目录存在
        if std::fs::create_dir_all(&log_dir).is_err() {
            return None;
        }

        // 清理过期日志
        cleanup_old_logs(&log_dir);

        // 创建按天轮转的文件 appender
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "ccr.log");

        // 文件输出层（无色彩）
        Some(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .with_span_events(FmtSpan::NONE)
                .with_ansi(false)
                .with_writer(file_appender),
        )
    });

    // 仅初始化文件日志层（无终端输出）
    let _ = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .try_init();
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
        // 这些测试主要确保方法不会 panic
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
