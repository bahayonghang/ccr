//! Grok Build profile command definitions.

use clap::{Args, Subcommand};

use super::profile_args::{
    ProfileDeleteActionArgs, ProfileDisableActionArgs, ProfileEditActionArgs,
    ProfileExportActionArgs, ProfileImportActionArgs, ProfileNameJsonActionArgs,
    ProfileOffActionArgs, ProfileSetFieldActionArgs,
};

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum GrokAction {
    /// Show Grok command help
    Help,

    /// Manage Grok official session login
    Auth {
        #[command(subcommand)]
        action: Option<GrokAuthAction>,
    },

    /// Manage Grok runtime profiles
    Profile {
        #[command(subcommand)]
        action: Box<GrokProfileAction>,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum GrokAuthAction {
    /// Show Grok Auth command help
    Help,

    /// Save one official OAuth source in the CCR account library
    Save {
        name: String,
        /// Required when more than one OAuth source is available
        #[arg(long)]
        scope: Option<String>,
        /// Replace an existing saved alias
        #[arg(short, long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },

    /// List saved accounts and local OAuth sources (not server login validity)
    List {
        #[arg(long)]
        json: bool,
    },

    /// Restore an account for a new session after stopping Grok
    Switch {
        name: String,
        #[arg(long)]
        json: bool,
    },

    /// Delete a saved account without logging out the runtime
    Delete {
        name: String,
        /// Skip deletion confirmation
        #[arg(short, long)]
        force: bool,
        #[arg(long)]
        json: bool,
    },

    /// Show whether a Grok official session file exists
    Current {
        #[arg(long)]
        json: bool,
    },

    /// Log out the current official Grok runtime session
    Off {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum GrokProfileAction {
    /// Show Grok profile command help
    Help,

    /// Open the Grok profiles.toml in your editor.
    /// Creates the file from the example template if it does not exist.
    Open {
        #[arg(long)]
        json: bool,
    },

    /// Initialize the Grok profile directory and example template
    Init {
        #[arg(long)]
        json: bool,
    },

    /// Show the current Grok profile and runtime
    Current {
        #[arg(long)]
        json: bool,
    },

    /// List Grok profiles
    List {
        #[arg(long)]
        json: bool,
    },

    /// Apply one Grok profile
    Switch {
        name: String,
        #[arg(long)]
        json: bool,
    },

    /// Create a Grok profile
    Create(Box<GrokProfileCreateActionArgs>),

    /// Update one Grok profile field
    #[command(hide = true)]
    SetField(ProfileSetFieldActionArgs),

    /// 一次修改多个字段、改名或替换密钥
    Edit(Box<GrokProfileEditActionArgs>),

    /// 导出全部或选定 Grok profiles 为可交换 TOML
    Export(ProfileExportActionArgs),

    /// 从交换文件或历史配置导入 Grok profiles
    Import(ProfileImportActionArgs),

    /// Enable a Grok profile
    Enable(ProfileNameJsonActionArgs),

    /// Disable a Grok profile
    Disable(ProfileDisableActionArgs),

    /// Delete a Grok profile; --force restores the entry runtime first
    Delete(ProfileDeleteActionArgs),

    /// Exit the current profile and clear CCR login leftovers
    Off(ProfileOffActionArgs),
}

#[derive(Args, Debug, Clone)]
pub struct GrokProfileCreateActionArgs {
    /// Profile name
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long = "base-url")]
    pub base_url: Option<String>,
    #[arg(
        long = "api-key",
        alias = "auth-token",
        hide = true,
        conflicts_with_all = ["env_key", "api_key_stdin"]
    )]
    pub api_key: Option<String>,
    /// 从标准输入读取密钥，避免进入 shell 历史
    #[arg(
        long = "api-key-stdin",
        visible_alias = "auth-token-stdin",
        conflicts_with_all = ["api_key", "env_key"]
    )]
    pub api_key_stdin: bool,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
    #[arg(long = "provider-type")]
    pub provider_type: Option<String>,
    #[arg(long)]
    pub account: Option<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long = "api-backend")]
    pub api_backend: Option<String>,
    #[arg(
        long = "env-key",
        conflicts_with_all = ["api_key", "api_key_stdin"]
    )]
    pub env_key: Option<String>,
    #[arg(long = "context-window")]
    pub context_window: Option<u64>,
    #[arg(
        long = "supports-backend-search",
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    pub supports_backend_search: Option<bool>,
    #[arg(long = "reasoning-effort")]
    pub reasoning_effort: Option<String>,
    /// 补充字段赋值，形式为 field=value，可重复
    #[arg(long = "set", value_name = "FIELD=VALUE")]
    pub set: Vec<String>,
    /// 仅当 catalog 仍是这个 revision 时写入
    #[arg(long = "expected-revision")]
    pub expected_revision: Option<String>,
    #[arg(long)]
    pub disabled: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct GrokProfileEditActionArgs {
    #[command(flatten)]
    pub common: ProfileEditActionArgs,
    #[arg(long = "api-backend")]
    pub api_backend: Option<String>,
    #[arg(
        long = "api-key-stdin",
        conflicts_with_all = ["auth_token_stdin", "env_key", "clear_auth_token"]
    )]
    pub api_key_stdin: bool,
    #[arg(
        long = "env-key",
        conflicts_with_all = ["api_key_stdin", "auth_token_stdin", "clear_auth_token"]
    )]
    pub env_key: Option<String>,
    #[arg(long = "context-window")]
    pub context_window: Option<u64>,
    #[arg(
        long = "supports-backend-search",
        num_args = 0..=1,
        default_missing_value = "true"
    )]
    pub supports_backend_search: Option<bool>,
    #[arg(long = "reasoning-effort")]
    pub reasoning_effort: Option<String>,
}
