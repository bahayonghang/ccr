// 🚀 CCR (Claude Code Configuration Switcher) 主程序
// 📦 配置管理工具,支持完整审计追踪
//
// 核心功能：
// - ⚙️  配置切换与管理
// - 📝 操作历史追踪
// - 🔒 文件锁保证并发安全
// - 🖥️ TUI 与 CCR UI 启动入口

mod application;
mod commands;
mod help;
mod managers;
mod models;
mod platforms;
mod services;
mod sync;

#[cfg(feature = "tui")]
mod tui;

// CLI 模块 - 命令行结构定义和命令分发
mod cli;

#[cfg(feature = "tui")]
use ccr_core::init_file_only_logger;
use ccr_core::init_logger;
use clap::Parser;
use cli::{Cli, CommandDispatcher};

/// 🎯 主函数入口
///
/// 执行流程:
/// 1. 📝 解析命令行参数
/// 2. 🔧 根据模式初始化日志系统（TUI 模式使用文件日志，避免覆盖界面）
/// 3. 🚀 路由并执行对应命令
/// 4. ❌ 处理错误并返回退出码
#[tokio::main]
async fn main() {
    // 📝 解析命令行参数（先解析以确定模式）
    let cli = Cli::parse();

    // 🔧 根据模式初始化日志系统
    // TUI 模式下仅输出到文件，避免日志覆盖 TUI 界面
    #[cfg(feature = "tui")]
    if cli.is_tui_mode() {
        init_file_only_logger();
    } else {
        init_logger();
    }
    #[cfg(not(feature = "tui"))]
    init_logger();

    // 🚀 执行命令并处理错误
    if let Err(e) = CommandDispatcher::dispatch(&cli).await {
        cli::dispatch::handle_error(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parsing() {
        // 测试基本的 CLI 解析
        Cli::command().debug_assert();
    }
}
