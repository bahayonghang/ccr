// CCR 日志与彩色输出模块
// 提供统一的日志初始化和彩色终端输出工具

use colored::*;
use std::io::{self, Write};

/// 彩色输出工具
///
/// 提供各种格式化的彩色输出方法，用于改善用户体验
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
║  Claude Code Configuration Switcher - Configuration Management Tool         ║
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

    /// 掩码敏感信息
    ///
    /// 将敏感信息(如 API Token)进行部分隐藏显示
    pub fn mask_sensitive(value: &str) -> String {
        if value.len() <= 10 {
            "*".repeat(value.len())
        } else {
            let visible_prefix = &value[..4];
            let visible_suffix = &value[value.len() - 4..];
            format!("{}...{}", visible_prefix, visible_suffix)
        }
    }

    /// 输出键值对
    pub fn key_value(key: &str, value: &str, indent: usize) {
        let padding = " ".repeat(indent);
        println!("{}{}: {}", padding, key.bold(), value);
    }

    /// 输出键值对（敏感信息）
    pub fn key_value_sensitive(key: &str, value: &str, indent: usize) {
        let padding = " ".repeat(indent);
        let masked = Self::mask_sensitive(value);
        println!("{}{}: {}", padding, key.bold(), masked.dimmed());
    }

    /// 输出当前配置标记（带颜色）
    pub fn current_marker() -> String {
        "▶".green().bold().to_string()
    }

    /// 输出普通项目标记
    pub fn normal_marker() -> String {
        " ".to_string()
    }

    /// 询问用户确认（是/否）
    pub fn ask_confirmation(question: &str, default: bool) -> bool {
        let default_str = if default { "Y/n" } else { "y/N" };
        print!("{} {} [{}]: ", "?".yellow().bold(), question, default_str);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();

        if input.is_empty() {
            default
        } else {
            matches!(input.as_str(), "y" | "yes" | "是")
        }
    }

    /// 输出配置节状态
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

/// 初始化日志系统
///
/// 使用环境变量 CCR_LOG_LEVEL 控制日志级别
/// 可选值: trace, debug, info, warn, error
pub fn init_logger() {
    let env = env_logger::Env::default()
        .filter_or("CCR_LOG_LEVEL", "info")
        .write_style_or("CCR_LOG_STYLE", "auto");

    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            let level_style = match record.level() {
                log::Level::Error => "ERROR".red().bold(),
                log::Level::Warn => "WARN".yellow().bold(),
                log::Level::Info => "INFO".blue().bold(),
                log::Level::Debug => "DEBUG".green().bold(),
                log::Level::Trace => "TRACE".purple().bold(),
            };

            writeln!(
                buf,
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                level_style,
                record.args()
            )
        })
        .init();
}

#[cfg(test)]
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
