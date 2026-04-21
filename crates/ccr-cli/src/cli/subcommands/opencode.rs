// 🔐 OpenCode 子命令定义
//
// 定义 OpenCode Auth 相关的 CLI 子命令结构

use clap::Subcommand;

/// OpenCode 子命令
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum OpenCodeAction {
    /// 显示 OpenCode 命令帮助
    Help,

    /// 账号管理
    ///
    /// 管理 OpenCode 的 openai 多账号快照
    Auth {
        #[command(subcommand)]
        action: OpenCodeAuthAction,
    },
}

/// OpenCode Auth 子命令
#[derive(Subcommand)]
#[command(disable_help_subcommand = true)]
pub enum OpenCodeAuthAction {
    /// 显示 OpenCode Auth 命令帮助
    Help,

    /// 从已保存的 Codex Auth 账号导入可兼容账号
    ///
    /// 示例: ccr opencode auth import-codex
    ///       ccr opencode auth import-codex --dry-run
    ///       ccr opencode auth import-codex --json
    ImportCodex {
        /// 仅预览，不写入 OpenCode 账号快照和注册表
        #[arg(long)]
        dry_run: bool,

        /// 以 JSON 格式输出迁移报告
        #[arg(long)]
        json: bool,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn parse_opencode_import_codex_command() {
        let cli = Cli::try_parse_from([
            "ccr",
            "opencode",
            "auth",
            "import-codex",
            "--dry-run",
            "--json",
        ])
        .unwrap();

        match cli.command {
            Some(crate::cli::Commands::OpenCode {
                action:
                    Some(super::OpenCodeAction::Auth {
                        action: super::OpenCodeAuthAction::ImportCodex { dry_run, json },
                    }),
            }) => {
                assert!(dry_run);
                assert!(json);
            }
            _ => panic!("unexpected command parse result"),
        }
    }
}
