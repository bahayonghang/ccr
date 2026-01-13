// UI 相关子命令

/// 🎨 UI 操作子命令
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum UiAction {
    /// 显示 `ccr ui` 帮助
    Help,

    /// 更新/安装用户目录下的 CCR UI 到最新版本（默认 main）
    Update,
}

/// 🎯 临时Token操作子命令
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum TempTokenAction {
    /// 显示 TempToken 命令帮助
    ///
    /// 示例: ccr temp-token help
    Help,

    /// 设置临时Token
    ///
    /// 临时覆盖当前配置的token,不修改toml配置文件
    /// 示例: ccr temp-token set sk-test-xxx
    ///       ccr temp-token set sk-xxx --base-url https://api.test.com
    ///       ccr temp-token set sk-xxx --model claude-opus-4
    Set {
        /// 临时使用的token
        token: String,

        /// 临时base_url(可选)
        #[arg(long)]
        base_url: Option<String>,

        /// 临时model(可选)
        #[arg(long)]
        model: Option<String>,
    },

    /// 显示当前临时配置
    ///
    /// 查看当前设置的临时配置状态
    /// 示例: ccr temp-token show
    Show,

    /// 清除临时配置
    ///
    /// 删除所有临时配置覆盖
    /// 示例: ccr temp-token clear
    Clear,
}
