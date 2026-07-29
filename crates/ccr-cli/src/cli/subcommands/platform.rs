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
    #[command(hide = true)]
    Switch {
        /// 平台名称 (claude, codex, gemini, qwen, droid)
        platform_name: String,
    },

    /// 显示当前平台信息
    ///
    /// 查看当前激活平台的详细信息
    /// 示例: ccr platform current
    /// 示例: ccr platform current --json
    #[command(hide = true)]
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
    #[command(hide = true)]
    Info {
        /// 平台名称
        platform_name: String,

        /// 以 JSON 格式输出 (用于脚本和工具集成)
        #[arg(long)]
        json: bool,
    },

    /// 退休的平台注册入口，仅保留旧命令解析兼容
    ///
    /// 使用 ccr claude/codex/grok profile init 的对应命令
    #[command(hide = true)]
    Init {
        /// 平台名称
        platform_name: String,
    },

    /// 管理平台 Profile
    ///
    /// 非交互式修改 ~/.ccr/platforms/{platform}/profiles.toml
    /// 示例: ccr platform profile set-field claude work model --value claude-sonnet-4-5
    ///       ccr platform profile create codex prod --model gpt-5-codex --provider OpenAI
    ///       ccr platform profile delete codex old --force
    #[command(hide = true)]
    Profile {
        #[command(subcommand)]
        action: Box<PlatformProfileAction>,
    },
}

#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum PlatformProfileAction {
    /// 创建新的平台 Profile
    Create {
        /// 平台名称（迁移目标支持 claude / codex / grok）
        platform_name: String,
        /// Profile 名称
        name: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long = "base-url")]
        base_url: Option<String>,
        #[arg(long = "auth-token")]
        auth_token: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long = "small-fast-model")]
        small_fast_model: Option<String>,
        #[arg(long)]
        provider: Option<String>,
        #[arg(long = "provider-type")]
        provider_type: Option<String>,
        #[arg(long)]
        account: Option<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long = "auth-mode")]
        auth_mode: Option<String>,
        #[arg(long)]
        disabled: bool,
        /// 以 JSON 输出结果（供扩展消费）
        #[arg(long)]
        json: bool,
    },

    /// 更新单个字段
    SetField {
        /// 平台名称
        platform_name: String,
        /// Profile 名称
        name: String,
        /// 字段名（snake_case）
        field: String,
        /// 字符串值
        #[arg(long, conflicts_with_all = ["value_json", "clear"])]
        value: Option<String>,
        /// JSON 值（适用于 tags 等数组字段）
        #[arg(long = "value-json", conflicts_with_all = ["value", "clear"])]
        value_json: Option<String>,
        /// 清空字段
        #[arg(long, conflicts_with_all = ["value", "value_json"])]
        clear: bool,
        /// 以 JSON 输出结果（供扩展消费）
        #[arg(long)]
        json: bool,
    },

    /// 启用 Profile
    Enable {
        platform_name: String,
        name: String,
        #[arg(long)]
        json: bool,
    },

    /// 禁用 Profile
    Disable {
        platform_name: String,
        name: String,
        /// 即使是当前 Profile 也允许禁用
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },

    /// 删除 Profile
    Delete {
        platform_name: String,
        name: String,
        /// 跳过额外防护提示
        #[arg(long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },
}
