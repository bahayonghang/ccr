//! Grok Build profile command definitions.

use clap::{Args, Subcommand};

use super::profile_args::{
    ProfileDisableActionArgs, ProfileNameJsonActionArgs, ProfileOffActionArgs,
    ProfileSetFieldActionArgs,
};

#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum GrokAction {
    /// Show Grok command help
    Help,

    /// Manage Grok runtime profiles
    Profile {
        #[command(subcommand)]
        action: Box<GrokProfileAction>,
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
    Switch { name: String },

    /// Create a Grok profile
    Create(Box<GrokProfileCreateActionArgs>),

    /// Update one Grok profile field
    SetField(ProfileSetFieldActionArgs),

    /// Enable a Grok profile
    Enable(ProfileNameJsonActionArgs),

    /// Disable a Grok profile
    Disable(ProfileDisableActionArgs),

    /// Delete a Grok profile; --force restores the entry runtime first
    Delete(ProfileDisableActionArgs),

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
        visible_alias = "auth-token",
        conflicts_with = "env_key"
    )]
    pub api_key: Option<String>,
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
    #[arg(long = "env-key", conflicts_with = "api_key")]
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
    #[arg(long)]
    pub disabled: bool,
    #[arg(long)]
    pub json: bool,
}
