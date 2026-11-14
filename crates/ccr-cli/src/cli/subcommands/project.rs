pub use crate::commands::project::WorkflowEngine;

/// Project-level workflow commands.
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ProjectAction {
    /// Initialize Git, workflow platforms, an ignored ref/ directory, and Agent rules
    ///
    /// Interactive mode checks Claude Code, Codex, Cursor, Oh My Pi, and Grok by
    /// default, then asks for a developer name. `ccr -y project init` reads
    /// `git config user.name` and runs `trellis init` with those five flags and
    /// `--yes`. `.agents/`, `.claude/`, `.codex/`, `.cursor/`, `.omp/`, `.grok/`,
    /// and `ref/` are merged into `.gitignore`.
    Init {
        /// 选择工作流引擎（交互模式默认推荐 mini-trellis，-y 默认 trellis）
        #[arg(long, value_enum, ignore_case = true)]
        workflow: Option<WorkflowEngine>,
    },
}
