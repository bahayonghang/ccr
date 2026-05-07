// 🔐 Codex 子命令定义
//
// 定义 Codex 多账号管理的 CLI 子命令结构

use clap::Subcommand;

use super::profile_args::{
    ProfileCreateActionArgs, ProfileDisableActionArgs, ProfileNameJsonActionArgs,
    ProfileOffActionArgs, ProfileSetFieldActionArgs,
};

/// Codex 子命令
///
/// 管理 Codex CLI 的多账号登录状态
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum CodexAction {
    /// 显示 Codex 命令帮助
    ///
    /// 示例: ccr codex help
    Help,

    /// 账号管理
    ///
    /// 管理 Codex 的多账号登录状态
    /// 示例: ccr codex auth list
    ///       ccr codex auth save my-account
    ///       ccr codex auth switch work
    Auth {
        #[command(subcommand)]
        action: CodexAuthAction,
    },

    /// Profile 路由与模式管理
    Profile {
        #[command(subcommand)]
        action: CodexProfileAction,
    },

    /// 输出当前或指定 profile 的环境变量导出脚本
    ///
    /// 示例: ccr codex env
    ///       ccr codex env work
    Env {
        /// 可选的 profile 名称；省略时使用当前 profile
        name: Option<String>,
    },

    /// 查询账号配额余额
    ///
    /// 查询 Codex 账号的 API 配额使用情况（5h窗口/周限额）
    /// 示例: ccr codex quota
    ///       ccr codex quota --account my-account
    ///       ccr codex quota --json
    Quota {
        /// 指定查询的账号名称（省略时查询所有账号）
        #[arg(short, long)]
        account: Option<String>,

        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,

        /// 强制刷新 token 后查询
        #[arg(long)]
        refresh: bool,
    },

    /// 同步 Codex 历史会话的 provider 元数据
    ///
    /// 修复官方/第三方 provider 切换后历史会话不可见的问题。
    /// 默认仅同步最近 7 天内的对话，避免导入过久远的历史记录。
    /// 示例: ccr codex sync-history --provider custom --dry-run
    ///       ccr codex sync-history --provider custom
    ///       ccr codex sync-history --provider openai
    ///       ccr codex sync-history --max-age-days 30
    ///       ccr codex sync-history status
    SyncHistory {
        /// 显式指定历史同步目标 provider；省略时使用当前 ~/.codex/config.toml 的根级 model_provider
        #[arg(long)]
        provider: Option<String>,

        /// 自动保留最近 N 份 sync-history 备份
        #[arg(long)]
        keep: Option<usize>,

        /// 最大导入会话年龄（天）；默认仅同步最近 7 天内的对话
        #[arg(long, default_value_t = 7)]
        max_age_days: u64,

        /// Preview rollout / SQLite changes without writing backups or state.
        #[arg(long)]
        dry_run: bool,

        /// 指定 Codex home 目录（默认使用 ~/.codex 或 CCR_CODEX_DIR）
        #[arg(long)]
        codex_home: Option<String>,

        #[command(subcommand)]
        action: Option<CodexSyncHistoryAction>,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum CodexProfileAction {
    /// 显示 Codex Profile 命令帮助
    Help,

    /// 显示当前 Codex profile/runtime 状态
    Current {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 列出 Codex profiles
    List {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 切换到指定 Codex profile
    Switch {
        /// 要切换到的 profile 名称
        name: String,
    },

    /// 退出当前 profile 路由，回到 official auth runtime
    /// Create a new Codex profile
    Create(ProfileCreateActionArgs),

    /// Update one Codex profile field
    SetField(ProfileSetFieldActionArgs),

    /// Enable a Codex profile
    Enable(ProfileNameJsonActionArgs),

    /// Disable a Codex profile
    Disable(ProfileDisableActionArgs),

    /// Delete a Codex profile
    Delete(ProfileDisableActionArgs),

    Off(ProfileOffActionArgs),
}

/// Codex 历史同步子命令
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum CodexSyncHistoryAction {
    /// 查看当前 provider 与历史元数据分布
    Status {
        /// 指定 Codex home 目录（默认使用 ~/.codex 或 CCR_CODEX_DIR）
        #[arg(long)]
        codex_home: Option<String>,
    },

    /// 从 sync-history 备份恢复
    Restore {
        /// 备份目录路径
        backup_dir: String,

        /// 指定 Codex home 目录（默认使用 ~/.codex 或 CCR_CODEX_DIR）
        #[arg(long)]
        codex_home: Option<String>,

        /// Restore backed up global state and state_5.sqlite as well as rollout metadata.
        #[arg(long)]
        restore_state: bool,
    },

    /// 清理旧的 sync-history 备份
    PruneBackups {
        /// 保留最近 N 份备份
        #[arg(long, default_value_t = 5)]
        keep: usize,

        /// 指定 Codex home 目录（默认使用 ~/.codex 或 CCR_CODEX_DIR）
        #[arg(long)]
        codex_home: Option<String>,
    },
}

/// Codex Auth 子命令
///
/// 管理 Codex 账号的保存、切换、删除等操作
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum CodexAuthAction {
    /// 显示 Codex Auth 命令帮助
    ///
    /// 示例: ccr codex auth help
    Help,

    /// 保存当前登录到指定名称
    ///
    /// 将当前 ~/.codex/auth.json 保存为命名账号
    /// 示例: ccr codex auth save work
    ///       ccr codex auth save personal -d "个人账号"
    ///       ccr codex auth save work --force  # 覆盖已存在的账号
    Save {
        /// 账号名称 (只能包含字母、数字、下划线和连字符)
        name: String,

        /// 账号描述 (可选)
        #[arg(short, long)]
        description: Option<String>,

        /// 强制覆盖已存在的账号
        #[arg(short, long)]
        force: bool,
    },

    /// 更新已保存账号的元数据
    ///
    /// 仅修改 auth_registry.toml 中的账号描述，不覆盖账号快照。
    Update {
        /// 账号名称
        name: String,

        /// 新的账号描述
        #[arg(short, long, conflicts_with = "clear_description")]
        description: Option<String>,

        /// 清空账号描述
        #[arg(long, conflicts_with = "description")]
        clear_description: bool,

        /// 以 JSON 格式输出（供扩展消费）
        #[arg(long)]
        json: bool,
    },

    /// 列出所有已保存的账号
    ///
    /// 显示所有已保存的 Codex 账号，包括当前登录状态
    /// 示例: ccr codex auth list
    List,

    /// 将当前 runtime OAuth tokens 回写到匹配的已保存账号
    ///
    /// 用于修复 refresh_token 轮换导致的快照过期问题。
    /// 示例: ccr codex auth sync
    Sync,

    /// 修复指定账号的 OAuth tokens（从 ~/.codex/auth.json 与 ~/.codex/backups 扫描最新副本）
    ///
    /// 示例: ccr codex auth repair team
    Repair {
        /// 要修复的账号名称
        name: String,
    },

    /// 切换到指定账号
    ///
    /// 将 ~/.codex/auth.json 切换为指定账号的登录状态
    /// 示例: ccr codex auth switch work
    Switch {
        /// 要切换到的账号名称
        name: String,
    },

    /// 删除指定账号
    ///
    /// 删除已保存的账号（不会影响当前登录状态）
    /// 示例: ccr codex auth delete old-account
    ///       ccr codex auth delete old-account --force  # 跳过确认
    Delete {
        /// 要删除的账号名称
        name: String,

        /// 跳过确认提示
        #[arg(short, long)]
        force: bool,
    },

    /// 重命名已保存账号
    ///
    /// 原子地迁移 auth 文件、registry 与 usage_ledger 归因，不需要重新登录。
    /// 示例: ccr codex auth rename old new
    ///       ccr codex auth rename old new --force  # 覆盖同名账号
    Rename {
        /// 当前账号名称
        old_name: String,

        /// 新账号名称 (只能包含字母、数字、下划线和连字符)
        new_name: String,

        /// 当新名称已存在时强制覆盖
        #[arg(short, long)]
        force: bool,

        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 显示当前账号信息
    ///
    /// 显示当前 ~/.codex/auth.json 的账号信息
    /// 示例: ccr codex auth current
    ///       ccr codex auth current --json
    Current {
        /// 以 JSON 格式输出（供扩展消费）
        #[arg(long)]
        json: bool,
    },

    /// 导出所有账号到 JSON 文件
    ///
    /// 将所有已保存的账号导出为 JSON 格式，默认保存到 Downloads 目录
    /// 示例: ccr codex auth export              # 导出到 Downloads/codex-auth-export-YYYY-MM-DD.json
    ///       ccr codex auth export --no-secrets # 不包含 Token
    Export {
        /// 不包含敏感信息 (Token 等)
        #[arg(long)]
        no_secrets: bool,
    },

    /// 从 JSON 文件导入账号
    ///
    /// 从 JSON 文件导入账号数据，默认自动扫描 Downloads 目录
    /// 示例: ccr codex auth import                  # 从 Downloads 自动查找
    ///       ccr codex auth import --replace        # 替换模式
    ///       ccr codex auth import --force          # 强制覆盖
    Import {
        /// 使用替换模式 (覆盖同名账号)
        #[arg(long)]
        replace: bool,

        /// 强制覆盖 (在合并模式下覆盖已存在的账号)
        #[arg(short, long)]
        force: bool,
    },
}
