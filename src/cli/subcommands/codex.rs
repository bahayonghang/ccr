// 🔐 Codex 子命令定义
//
// 定义 Codex 多账号管理的 CLI 子命令结构

use clap::Subcommand;

/// Codex 子命令
///
/// 管理 Codex CLI 的多账号登录状态
#[derive(Subcommand)]
pub enum CodexAction {
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
}

/// Codex Auth 子命令
///
/// 管理 Codex 账号的保存、切换、删除等操作
#[derive(Subcommand)]
pub enum CodexAuthAction {
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

    /// 列出所有已保存的账号
    ///
    /// 显示所有已保存的 Codex 账号，包括当前登录状态
    /// 示例: ccr codex auth list
    List,

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

    /// 显示当前账号信息
    ///
    /// 显示当前 ~/.codex/auth.json 的账号信息
    /// 示例: ccr codex auth current
    Current,

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
