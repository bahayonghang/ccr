/// Project-level workflow commands.
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ProjectAction {
    /// Initialize Git, Trellis platforms, an ignored ref/ directory, and Agent rules
    ///
    /// Interactive mode checks Claude Code, Codex, Cursor, Oh My Pi, and Grok by
    /// default, then asks for a developer name. `ccr -y project init` reads
    /// `git config user.name` and runs `trellis init` with those five flags and
    /// `--yes`. `.agents/`, `.claude/`, `.codex/`, and `ref/` are merged into
    /// `.gitignore`.
    Init,
}
