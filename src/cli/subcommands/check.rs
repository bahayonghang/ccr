// 检查操作子命令
#[derive(clap::Subcommand)]
pub enum CheckAction {
    /// 检测环境变量冲突
    Conflicts,
}
