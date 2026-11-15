// 平台管理子命令
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum PlatformAction {
    /// 显示 Platform 命令帮助
    ///
    /// 示例: ccr platform help
    Help,

    /// 列出所有可用平台
    ///
    /// 显示所有支持的 AI CLI 平台及其状态
    /// 示例: ccr platform list
    /// 示例: ccr platform list --json
    List {
        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },
}
