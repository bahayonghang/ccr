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
pub struct ProfileOffActionArgs {
    #[arg(long)]
    pub json: bool,
}
