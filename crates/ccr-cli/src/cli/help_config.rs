// CLI 帮助命令树增强
//
// 基于 derive 生成的命令树统一补齐任务导向帮助，保证：
// - `ccr --help`
// - `ccr <命令> --help`
// - `ccr help <命令路径...>`
// 看到的是同一套帮助内容。

use crate::cli::Cli;
use clap::{Command, CommandFactory};

const ROOT_HELP_TEMPLATE: &str = "\
{before-help}{name} {version}
{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

const SUBCOMMAND_HELP_TEMPLATE: &str = "\
{before-help}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}";

const ROOT_LONG_ABOUT: &str = "\
统一管理 Claude / Codex / OpenCode 等 AI CLI 的配置、平台和账号。

先看当前任务，再进入对应子命令。";

const ROOT_AFTER_LONG_HELP: &str = "\
常用任务:

  切换平台
    先看: ccr platform --help
    查看平台: ccr platform list
    执行切换: ccr platform switch codex
    确认结果: ccr platform current

  切换 Codex Auth
    先看: ccr codex auth --help
    查看当前登录: ccr codex auth current
    首次保存: ccr codex auth save work
    切换账号: ccr codex auth switch work
    确认结果: ccr codex auth current

  把 Codex 订阅导入 OpenCode
    先看: ccr opencode auth --help
    先预览: ccr opencode auth import-codex --dry-run
    执行导入: ccr opencode auth import-codex
    查看结果: ccr opencode

更多入口:
  ccr help platform
  ccr help codex auth
  ccr help opencode auth";

const HELP_LONG_ABOUT: &str = "\
查看任务导向帮助。

支持多段命令路径。";

const HELP_AFTER_LONG_HELP: &str = "\
示例:
  ccr help
  ccr help platform
  ccr help codex auth
  ccr help opencode auth";

const PLATFORM_LONG_ABOUT: &str = "\
查看当前平台、切换平台、管理每个平台的 profile。";

const PLATFORM_AFTER_LONG_HELP: &str = "\
常用任务:
  先看当前有哪些平台
    ccr platform list

  切换到目标平台
    ccr platform switch codex

  确认当前平台
    ccr platform current

更多:
  查看单个平台详情: ccr platform info codex
  管理 profile: ccr platform profile --help";

const CODEX_LONG_ABOUT: &str = "\
管理 Codex CLI 的账号、配额和历史同步。";

const CODEX_AFTER_LONG_HELP: &str = "\
常用入口:
  多账号切换: ccr codex auth --help
  查看配额: ccr codex quota --help
  导出当前 profile 环境: ccr codex env
  打开 Codex 账号界面: ccr codex";

const CODEX_AUTH_LONG_ABOUT: &str = "\
管理 Codex 的多账号登录状态。";

const CODEX_AUTH_AFTER_LONG_HELP: &str = "\
常用任务:
  1. 先确认当前登录状态
     codex login
     ccr codex auth current

  2. 首次保存当前登录
     ccr codex auth save work

  3. 查看已保存账号
     ccr codex auth list

  4. 切换到目标账号
     ccr codex auth switch work

  5. 再次确认切换结果
     ccr codex auth current

边界:
  - 只有 cli_auth_credentials_store = file 时，CCR 才支持保存和切换多账号
  - API Key / Provider Key 模式无需 save / switch";

const OPENCODE_LONG_ABOUT: &str = "\
管理 OpenCode 的账号迁移入口。";

const OPENCODE_AFTER_LONG_HELP: &str = "\
常用入口:
  从 Codex 导入可兼容账号: ccr opencode auth --help
  打开 OpenCode 账号界面: ccr opencode";

const OPENCODE_AUTH_LONG_ABOUT: &str = "\
从已保存的 Codex Auth 账号导入可兼容的 OpenCode 账号。";

const OPENCODE_AUTH_AFTER_LONG_HELP: &str = "\
常用任务:
  1. 先预览可迁移账号
     ccr opencode auth import-codex --dry-run

  2. 确认无误后执行导入
     ccr opencode auth import-codex

  3. 打开 OpenCode 界面确认结果
     ccr opencode

边界:
  - 只迁移 ChatGPT OAuth 账号
  - API key / provider 账号会跳过
  - 不会覆盖现有 OpenCode 账号";

const VERSION_LONG_ABOUT: &str = "\
显示当前安装的 CCR 详细版本信息。";

const VERSION_AFTER_LONG_HELP: &str = "\
使用方式:
  人读详细信息
    ccr version

  脚本或 CI 只取版本号
    ccr --version
    ccr -V";

const CLEAN_LONG_ABOUT: &str = "\
清理两类文件：

- 旧备份文件
- planning-with-files 生成的规划文件";

const CLEAN_AFTER_LONG_HELP: &str = "\
常用任务:
  清理旧备份
    先预览: ccr clean --dry-run
    清理 30 天前的备份: ccr clean --days 30

  清理当前目录下的规划文件
    先预览: ccr clean planfiles --dry-run
    执行清理: ccr clean planfiles

边界:
  - `ccr clean` 只处理 ~/.claude/backups 下的 .bak 文件
  - `ccr clean planfiles` 只处理 task_plan.md / findings.md / progress.md
  - `ccr clean planfiles` 默认不跟随符号链接目录";

pub fn build_cli_command() -> Command {
    Cli::command()
        .help_template(ROOT_HELP_TEMPLATE)
        .override_usage("ccr [选项] [配置名称] [命令]")
        .disable_help_subcommand(true)
        .long_about(ROOT_LONG_ABOUT)
        .after_long_help(ROOT_AFTER_LONG_HELP)
        .subcommand_help_heading("Commands")
        .mut_subcommand("help", configure_help_command)
        .mut_subcommand("platform", configure_platform_command)
        .mut_subcommand("version", configure_version_command)
        .mut_subcommand("codex", configure_codex_command)
        .mut_subcommand("opencode", configure_opencode_command)
        .mut_subcommand("clean", configure_clean_command)
}

fn configure_help_command(cmd: Command) -> Command {
    cmd.about("查看任务导向帮助")
        .help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(HELP_LONG_ABOUT)
        .after_long_help(HELP_AFTER_LONG_HELP)
}

fn configure_platform_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(PLATFORM_LONG_ABOUT)
        .after_long_help(PLATFORM_AFTER_LONG_HELP)
}

fn configure_version_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(VERSION_LONG_ABOUT)
        .after_long_help(VERSION_AFTER_LONG_HELP)
}

fn configure_clean_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(CLEAN_LONG_ABOUT)
        .after_long_help(CLEAN_AFTER_LONG_HELP)
}

fn configure_codex_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(CODEX_LONG_ABOUT)
        .after_long_help(CODEX_AFTER_LONG_HELP)
        .mut_subcommand("auth", configure_codex_auth_command)
}

fn configure_codex_auth_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(CODEX_AUTH_LONG_ABOUT)
        .after_long_help(CODEX_AUTH_AFTER_LONG_HELP)
}

fn configure_opencode_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(OPENCODE_LONG_ABOUT)
        .after_long_help(OPENCODE_AFTER_LONG_HELP)
        .mut_subcommand("auth", configure_opencode_auth_command)
}

fn configure_opencode_auth_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(OPENCODE_AUTH_LONG_ABOUT)
        .after_long_help(OPENCODE_AUTH_AFTER_LONG_HELP)
}
