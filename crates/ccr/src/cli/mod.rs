// CLI 模块 - 命令行接口定义和命令分发
//
// 提供统一的 CLI 结构定义和命令路由机制

pub mod definitions;
pub mod dispatch;
pub mod subcommands;

pub use definitions::{Cli, Commands};
pub use dispatch::CommandDispatcher;
