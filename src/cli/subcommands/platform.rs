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

    /// 切换到指定平台
    ///
    /// 切换当前激活的平台
    /// 示例: ccr platform switch codex
    Switch {
        /// 平台名称 (claude, codex, gemini, qwen, iflow)
        platform_name: String,
    },

    /// 显示当前平台信息
    ///
    /// 查看当前激活平台的详细信息
    /// 示例: ccr platform current
    /// 示例: ccr platform current --json
    Current {
        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },

    /// 显示平台详细信息
    ///
    /// 查看指定平台的配置和状态
    /// 示例: ccr platform info claude
    /// 示例: ccr platform info claude --json
    Info {
        /// 平台名称
        platform_name: String,

        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },

    /// 初始化平台配置
    ///
    /// 为指定平台创建配置目录结构
    /// 示例: ccr platform init codex
    Init {
        /// 平台名称
        platform_name: String,
    },
}
