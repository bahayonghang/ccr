use ccr::cli::{Cli, CommandDispatcher, build_cli_command};
#[cfg(feature = "tui")]
use ccr::init_file_only_logger;
use ccr::init_logger;
use clap::FromArgMatches;

/// 🎯 主函数入口
///
/// 执行流程:
/// 1. 📝 解析命令行参数
/// 2. 🔧 根据模式初始化日志系统（TUI 模式使用文件日志，避免覆盖界面）
/// 3. 🚀 路由并执行对应命令
/// 4. ❌ 处理错误并返回退出码
#[tokio::main]
async fn main() {
    let matches = build_cli_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|err| err.exit());

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
        ccr::cli::dispatch::handle_error(e);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_parsing() {
        ccr::cli::build_cli_command().debug_assert();
    }
}
