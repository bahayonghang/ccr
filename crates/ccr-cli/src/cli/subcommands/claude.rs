// 🔐 Claude 子命令定义
//
// 定义 Claude 官方订阅账号管理的 CLI 子命令结构

use clap::Subcommand;

use super::profile_args::{
    ProfileCreateActionArgs, ProfileDisableActionArgs, ProfileNameJsonActionArgs,
    ProfileOffActionArgs, ProfileSetFieldActionArgs,
};

/// Claude 子命令
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ClaudeAction {
    /// 显示 Claude 命令帮助
    Help,

    /// 官方订阅账号管理
    Auth {
        #[command(subcommand)]
        action: ClaudeAuthAction,
    },

    /// Profile 路由与模式管理
    Profile {
        #[command(subcommand)]
        action: Box<ClaudeProfileAction>,
    },
}

/// Claude Auth 子命令
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ClaudeAuthAction {
    /// 显示 Claude Auth 命令帮助
    Help,

    /// 保存当前官方登录到指定名称
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

    /// 列出所有已保存的官方账号
    List,

    /// 切换到指定官方账号
    Switch {
        /// 要切换到的账号名称
        name: String,
    },

    /// 删除指定账号
    Delete {
        /// 要删除的账号名称
        name: String,

        /// 跳过确认提示
        #[arg(short, long)]
        force: bool,
    },

    /// 显示当前官方登录信息
    Current {
        /// 以 JSON 格式输出（供扩展消费）
        #[arg(long)]
        json: bool,
    },
}

/// Claude Profile 子命令
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ClaudeProfileAction {
    /// 显示 Claude Profile 命令帮助
    Help,

    /// Open the Claude profiles.toml in your editor.
    /// Creates the file from the example template if it does not exist.
    Open {
        #[arg(long)]
        json: bool,
    },

    /// Initialize the Claude profile directory and example template
    Init {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 显示当前 Claude profile/runtime 状态
    Current {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 列出 Claude profiles
    List {
        /// 以 JSON 格式输出
        #[arg(long)]
        json: bool,
    },

    /// 切换到指定 Claude profile
    Switch {
        /// 要切换到的 profile 名称
        name: String,
    },

    /// Create a new Claude profile
    Create(ProfileCreateActionArgs),

    /// Update one Claude profile field
    SetField(ProfileSetFieldActionArgs),

    /// Enable a Claude profile
    Enable(ProfileNameJsonActionArgs),

    /// Disable a Claude profile
    Disable(ProfileDisableActionArgs),

    /// Delete a Claude profile
    Delete(ProfileDisableActionArgs),

    /// 退出当前 profile，并清理会压制官方登录的 CCR 运行时残留
    Off(ProfileOffActionArgs),
}
