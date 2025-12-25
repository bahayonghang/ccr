// 🚀 CCR (Claude Code Configuration Switcher) 主程序
// 📦 配置管理工具,支持完整审计追踪
//
// 核心功能：
// - ⚙️  配置切换与管理
// - 📝 操作历史追踪
// - 🔒 文件锁保证并发安全
// - 🌐 Web 管理界面

mod commands;
mod core;
mod help;
mod managers;
mod models;
mod platforms;
mod services;
mod sessions;
mod storage;
mod sync;
mod utils;

#[cfg(feature = "tui")]
mod tui;

#[cfg(feature = "web")]
mod web;

// CLI 模块 - 命令行结构定义和命令分发
mod cli;

use clap::Parser;
use cli::{Cli, CommandDispatcher};
use core::init_logger;

/// 🎯 主函数入口
///
/// 执行流程:
/// 1. 🔧 初始化日志系统
/// 2. 📝 解析命令行参数
/// 3. 🚀 路由并执行对应命令
/// 4. ❌ 处理错误并返回退出码
fn main() {
    // 🔧 初始化日志系统
    init_logger();

    // 📝 解析命令行参数
    let cli = Cli::parse();

    // 🚀 执行命令并处理错误
    if let Err(e) = CommandDispatcher::dispatch(&cli) {
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
