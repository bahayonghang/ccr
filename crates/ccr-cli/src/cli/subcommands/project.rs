/// Project-level workflow commands.
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ProjectAction {
    /// Initialize Git, Trellis, and local Agent ignore rules in the current directory
    ///
    /// Runs Git detection or initialization first, delegates workflow setup to
    /// `trellis init`, then merges `.agents/`, `.claude/`, and `.codex/` into
    /// the current project's `.gitignore`.
    Init,
}
