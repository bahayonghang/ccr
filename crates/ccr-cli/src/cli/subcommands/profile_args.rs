use clap::Args;

#[derive(Args, Debug, Clone)]
pub struct ProfileCreateActionArgs {
    /// Profile name
    pub name: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long = "base-url")]
    pub base_url: Option<String>,
    #[arg(long = "auth-token")]
    pub auth_token: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long = "small-fast-model")]
    pub small_fast_model: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
    #[arg(long = "provider-type")]
    pub provider_type: Option<String>,
    #[arg(long)]
    pub account: Option<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long = "auth-mode")]
    pub auth_mode: Option<String>,
    #[arg(long)]
    pub disabled: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileSetFieldActionArgs {
    /// Profile name
    pub name: String,
    /// Field name in snake_case
    pub field: String,
    /// String value
    #[arg(long, conflicts_with_all = ["value_json", "clear"])]
    pub value: Option<String>,
    /// JSON value, useful for array fields such as tags
    #[arg(long = "value-json", conflicts_with_all = ["value", "clear"])]
    pub value_json: Option<String>,
    /// Clear the field
    #[arg(long, conflicts_with_all = ["value", "value_json"])]
    pub clear: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileNameJsonActionArgs {
    /// Profile name
    pub name: String,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileDisableActionArgs {
    /// Profile name
    pub name: String,
    /// Allow disabling the current profile
    #[arg(long)]
    pub force: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileEditActionArgs {
    /// Profile 名称
    pub name: String,
    /// 改名后的新名称
    #[arg(long)]
    pub rename: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    #[arg(long = "base-url")]
    pub base_url: Option<String>,
    #[arg(long)]
    pub provider: Option<String>,
    /// 整体替换标签，可重复
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long, conflicts_with = "disable")]
    pub enable: bool,
    #[arg(long, conflicts_with = "enable")]
    pub disable: bool,
    /// 补充字段赋值，形式为 field=value，可重复
    #[arg(long = "set", value_name = "FIELD=VALUE")]
    pub set: Vec<String>,
    /// 清空字段，可重复
    #[arg(long = "clear", value_name = "FIELD")]
    pub clear: Vec<String>,
    /// 从标准输入读取密钥，避免进入 shell 历史
    #[arg(long = "auth-token-stdin", conflicts_with = "clear_auth_token")]
    pub auth_token_stdin: bool,
    /// 清除已保存的密钥
    #[arg(long = "clear-auth-token", conflicts_with = "auth_token_stdin")]
    pub clear_auth_token: bool,
    /// 仅当 catalog 仍是这个 revision 时写入
    #[arg(long = "expected-revision")]
    pub expected_revision: Option<String>,
    /// 只修改存储，不重新应用当前生效的运行时
    #[arg(long = "no-apply")]
    pub no_apply: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args, Debug, Clone)]
pub struct CodexProfileEditActionArgs {
    #[command(flatten)]
    pub common: ProfileEditActionArgs,
    /// Codex model_reasoning_effort
    #[arg(long = "reasoning-effort")]
    pub reasoning_effort: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ProfileOffActionArgs {
    #[arg(long)]
    pub json: bool,
}
