pub use crate::commands::project::WorkflowEngine;

/// Project-level workflow commands.
#[derive(clap::Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum ProjectAction {
    /// Initialize Git, a Trellis or mini-trellis workflow, an ignored ref/ directory, and Agent ignore rules.
    #[command(
        about = "在当前目录初始化 Git、工作流引擎（Trellis / mini-trellis）、本地 ref/ 与 .gitignore 规则"
    )]
    Init {
        /// 选择工作流引擎（交互模式默认推荐 mini-trellis；-y 非交互模式默认兼容 trellis）
        #[arg(long, value_enum, ignore_case = true)]
        workflow: Option<WorkflowEngine>,
    },
}
