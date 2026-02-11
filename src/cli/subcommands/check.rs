// 检查操作子命令
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum CheckAction {
    /// 显示 Check 命令帮助
    ///
    /// 示例: ccr check help
    Help,

    /// 检测环境变量冲突
    Conflicts,
}
