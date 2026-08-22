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
统一管理 Claude / Codex / Grok 等 AI CLI 的配置、平台和账号。

先看当前任务，再进入对应子命令。";

const ROOT_AFTER_LONG_HELP: &str = "\
常用任务:

  初始化当前项目
    先看: ccr project init --help
    交互初始化: ccr project init
    使用 Trellis 默认值: ccr -y project init

  查看平台与 Profile
    查看运行状态: ccr current
    查看支持平台: ccr platform list
    Claude Profile: ccr claude profile --help
    Codex Profile: ccr codex profile --help
    Grok Profile: ccr grok profile --help

  切换 Codex Auth
    先看: ccr codex auth --help
    查看当前登录: ccr codex auth current
    首次保存: ccr codex auth save work
    切换账号: ccr codex auth switch work
    确认结果: ccr codex auth current

  登出官方运行时登录
    先看: ccr claude auth off --help
    Claude: ccr claude auth off
    Codex: ccr codex auth off
    Grok: ccr grok auth off

更多入口:
  ccr help platform
  ccr help codex auth
  ccr help grok auth";

const HELP_LONG_ABOUT: &str = "\
查看任务导向帮助。

支持多段命令路径。";

const HELP_AFTER_LONG_HELP: &str = "\
示例:
  ccr help
  ccr help platform
  ccr help codex auth
  ccr help grok auth";

const PROJECT_LONG_ABOUT: &str = "\
在当前工作目录中依次准备 Git 仓库、Trellis 工作流和 Agent 目录忽略规则。

如果当前目录位于父级 Git 仓库中，会保留该仓库边界，不创建嵌套仓库。";

const PROJECT_AFTER_LONG_HELP: &str = "\
常用任务:
  交互式初始化
    ccr project init

  使用 Trellis 默认值进行非交互初始化
    ccr -y project init

阶段顺序:
  1. 检测 Git 工作树；必要时运行 git init
  2. 在当前目录运行 trellis init
  3. 向 .gitignore 合并 .agents/、.claude/、.codex/

边界:
  - 必须预先安装 git 和 trellis
  - Trellis 的用户名和 Agent 平台选择由 trellis init 自己处理
  - 任一阶段失败后可在同一目录安全重试";

const PROJECT_INIT_LONG_ABOUT: &str = "\
初始化当前工作目录的项目工作流。

CCR 先检测或初始化 Git，再继承当前终端运行 trellis init，最后以原子写入方式把
.agents/、.claude/ 和 .codex/ 合并到当前目录的 .gitignore。已有内容和换行风格会保留。";

const PROJECT_INIT_AFTER_LONG_HELP: &str = "\
用法:
  交互选择 Trellis 用户名和 Agent 平台: ccr project init
  将全局 --yes 转发给 Trellis: ccr -y project init

父仓库:
  当前目录位于父级 Git 工作树时会显示仓库根并跳过 git init；Trellis 和
  .gitignore 仍作用于调用命令时的当前目录。

恢复:
  Git、Trellis 或 .gitignore 阶段失败后不会回滚已完成阶段；修复原因后重复运行即可。";

const PLATFORM_LONG_ABOUT: &str = "\
列出 CCR 可识别的 AI CLI 平台。

运行状态和 Profile 管理已迁移到各自的显式命令入口。";

const PLATFORM_AFTER_LONG_HELP: &str = "\
常用任务:
  查看 CCR 支持的平台
    ccr platform list

  查看当前运行状态
    ccr current

  初始化平台 Profile 目录
    ccr <claude|codex|grok> profile init

  管理 Claude Profile
    ccr claude profile --help

  管理 Codex Profile
    ccr codex profile --help

  管理 Grok Profile
    ccr grok profile --help

边界:
  旧的平台路由入口已退休；现有脚本仍会收到明确的迁移错误。";

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

  6. 登出官方运行时登录
     ccr codex auth off

边界:
  - 只有 cli_auth_credentials_store = file 时，CCR 才支持保存和切换多账号
  - API Key / Provider Key 模式无需 save / switch";

const GROK_AUTH_LONG_ABOUT: &str = "\
查看并登出 Grok 官方运行时登录。CCR 不保存 Grok 账号快照。";

const GROK_AUTH_AFTER_LONG_HELP: &str = "\
常用任务:
  查看当前官方会话: ccr grok auth current
  登出官方会话: ccr grok auth off
  打开 Grok Auth 界面: ccr grok auth

边界:
  - 只删除 $GROK_HOME/auth.json
  - 不读取或写入 mcp_credentials.json
  - 不修改 profile 指针或 config.toml";

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
裸 `ccr clean` 会进入交互式清理菜单。

可显式清理两类目标：

- planning-with-files 生成的规划文件
- 旧备份文件";

const CLEAN_AFTER_LONG_HELP: &str = "\
常用任务:
  交互式清理菜单
    打开菜单: ccr clean
    自动执行默认编号: ccr -y clean

  清理当前目录根层的规划文件
    先预览根目录规划文件: ccr clean planfiles --dry-run
    执行默认根目录清理: ccr clean

  递归清理所有规划文件
    递归预览所有规划文件: ccr clean planfiles --all --dry-run
    递归清理所有规划文件: ccr clean --all

  清理旧备份
    先预览: ccr clean backups --dry-run
    清理 30 天前的备份: ccr clean backups --days 30

  兼容旧脚本
    ccr clean --dry-run
    ccr clean --days 30

边界:
  - 裸 `ccr clean` 显示编号菜单，回车默认执行 1.planfiles（仅当前目录根层），输入 q 取消
  - `ccr clean --all` 递归处理当前目录及子目录中的 task_plan.md / findings.md / progress.md
  - `ccr clean planfiles` 默认只处理当前目录根层的 task_plan.md / findings.md / progress.md
  - `ccr clean planfiles --all` 默认不跟随符号链接目录
  - `ccr clean backups` 只处理 ~/.claude/backups 下的 .bak 文件";

pub fn build_cli_command() -> Command {
    Cli::command()
        .help_template(ROOT_HELP_TEMPLATE)
        .override_usage("ccr [选项] [配置名称] [命令]")
        .disable_help_subcommand(true)
        .long_about(ROOT_LONG_ABOUT)
        .after_long_help(ROOT_AFTER_LONG_HELP)
        .subcommand_help_heading("Commands")
        .mut_subcommand("help", configure_help_command)
        .mut_subcommand("project", configure_project_command)
        .mut_subcommand("platform", configure_platform_command)
        .mut_subcommand("version", configure_version_command)
        .mut_subcommand("codex", configure_codex_command)
        .mut_subcommand("grok", configure_grok_command)
        .mut_subcommand("clean", configure_clean_command)
}

fn configure_project_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(PROJECT_LONG_ABOUT)
        .after_long_help(PROJECT_AFTER_LONG_HELP)
        .mut_subcommand("init", configure_project_init_command)
}

fn configure_project_init_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(PROJECT_INIT_LONG_ABOUT)
        .after_long_help(PROJECT_INIT_AFTER_LONG_HELP)
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

fn configure_grok_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .mut_subcommand("auth", configure_grok_auth_command)
}

fn configure_grok_auth_command(cmd: Command) -> Command {
    cmd.help_template(SUBCOMMAND_HELP_TEMPLATE)
        .long_about(GROK_AUTH_LONG_ABOUT)
        .after_long_help(GROK_AUTH_AFTER_LONG_HELP)
}
