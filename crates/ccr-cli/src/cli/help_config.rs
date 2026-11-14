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
    交互向导（默认推荐 mini-trellis）: ccr project init
    指定引擎: ccr project init --workflow mini-trellis
    非交互兼容（默认 trellis 五项 + Git 用户名）: ccr -y project init

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
在当前工作目录中依次准备 Git 仓库、Trellis 或 mini-trellis 工作流、Agent 忽略规则和本地 ref/ 目录。

如果当前目录位于父级 Git 仓库中，会保留该仓库边界，不创建嵌套仓库。";

const PROJECT_AFTER_LONG_HELP: &str = "\
常用任务:
  交互式向导（默认推荐 mini-trellis，手动输入开发者用户名）
    ccr project init

  交互式指定工作流引擎（跳过引擎单选）
    ccr project init --workflow mini-trellis
    ccr project init --workflow trellis

  非交互初始化（未传 --workflow 时默认兼容 trellis 五项 + git config user.name）
    ccr -y project init
    ccr -y project init --workflow mini-trellis

阶段顺序:
  1. [1/4] 检查 Git 仓库（检测现有仓库或父工作树，必要时运行 git init）
  2. [2/4] 配置工作流与宿主平台（前置检测特征指纹冲突，采集引擎、平台与开发者用户名）
  3. [3/4] 执行 trellis init 或 mini-trellis init 并校验最低工作流产物
  4. [4/4] 准备本地 ref/ 目录，并以原子写入向 .gitignore 合并 .agents/、.claude/、.codex/、.cursor/、.omp/、.grok/、ref/

边界:
  - 必须预先安装 git 以及所选引擎对应的 trellis 或 mini-trellis
  - trellis 支持 5 项平台（Claude Code、Codex、Cursor、Oh My Pi、Grok）；mini-trellis 支持 4 项平台（不含 Grok）
  - 跨引擎或混杂目录会在调用外部 CLI 前以退出码 90 阻断，并提示 mini-trellis migrate 或 mini-trellis uninstall
  - 同引擎重跑与失败后重试均保持安全幂等（不附加 -s/--skip-existing）";

const PROJECT_INIT_LONG_ABOUT: &str = "\
以四段式向导初始化当前工作目录的项目工作流（支持 trellis 与 mini-trellis 双工作流引擎）。

Git 检查通过后，交互模式未传 --workflow 时默认推荐 mini-trellis，再按所选引擎展示默认全选的宿主平台复选菜单（trellis 支持 Claude Code、Codex、Cursor、Oh My Pi、Grok 共 5 项；mini-trellis 支持 Claude Code、Codex、Cursor、Oh My Pi 共 4 项，不含 Grok），并要求手动输入开发者用户名（不读取 git config user.name 作为默认值）。`ccr -y project init` 未传 --workflow 时默认兼容 trellis，读取 git config user.name 并向 `trellis init` 或 `mini-trellis init` 显式传入该引擎全部平台开关与 `--yes`（不传 `-s`）。随后校验引擎最低工作流产物，创建本地 ref/，并以原子写入把固定 7 条规则（.agents/、.claude/、.codex/、.cursor/、.omp/、.grok/ 和 ref/）合并进当前目录的 .gitignore，保留已有内容与换行风格。";

const PROJECT_INIT_AFTER_LONG_HELP: &str = "\
用法:
  交互向导（默认推荐 mini-trellis）: ccr project init
  显式指定引擎进入交互平台选择: ccr project init --workflow mini-trellis
  非交互默认兼容（trellis 5 项 + git config user.name + --yes）: ccr -y project init
  非交互初始化 mini-trellis（4 项 + git config user.name + --yes）: ccr -y project init --workflow mini-trellis

父仓库:
  当前目录位于父级 Git 工作树时会显示仓库根并跳过嵌套 git init；工作流引擎、ref/ 和
  .gitignore 仍只作用于调用命令时的当前目录。

冲突与恢复:
  - 已有 trellis 目录运行 --workflow mini-trellis，或已有 mini-trellis 目录运行 trellis（含裸 -y 默认），以及双引擎混杂目录，均在启动外部 CLI 前以退出码 90 阻断并提示 mini-trellis migrate 或 mini-trellis uninstall。
  - 同引擎重复运行安全走重初始化并保留自定义资产；任一阶段失败后不会回滚已完成阶段，修复原因后在同一目录重跑即可。";

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

  7. 换号或添加账号（不吊销已保存账号的 token）
     ccr codex auth relogin

边界:
  - 只有 cli_auth_credentials_store = file 时，CCR 才支持保存和切换多账号
  - API Key / Provider Key 模式无需 save / switch";

const GROK_AUTH_LONG_ABOUT: &str = "\
管理 Grok 官方 OAuth 账号快照、切换和运行时登出。";

const GROK_AUTH_AFTER_LONG_HELP: &str = "\
常用任务:
  保存当前账号: ccr grok auth save gmail
  多来源保存: ccr grok auth save gmail --scope <scope>
  查看账号与来源: ccr grok auth list --json
  切换保存账号: ccr grok auth switch gmail
  删除保存账号: ccr grok auth delete gmail
  查看当前官方会话: ccr grok auth current
  登出官方会话: ccr grok auth off
  打开 Grok Auth 界面: ccr grok auth

边界:
  - save 只写 CCR 账号库；重名覆盖必须显式使用 --force
  - 切换前结束当前 Grok；switch 先回存原账号，再恢复目标 scope，用于新会话
  - delete 不登出；off 回存可识别的原账号后删除整个 $GROK_HOME/auth.json，保留账号库
  - current 仅检查文件存在；list 的本地匹配不代表服务端登录有效
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

可显式清理三类目标：

- planning-with-files 生成的规划文件
- 旧备份文件
- CCR 根目录里的构建缓存和停用数据库副本";

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

  清理根目录存储
    先预览: ccr clean storage --dry-run

  兼容旧脚本
    ccr clean --dry-run
    ccr clean --days 30

边界:
  - 裸 `ccr clean` 显示编号菜单，回车默认执行 1.planfiles（仅当前目录根层），输入 q 取消
  - `ccr clean --all` 递归处理当前目录及子目录中的 task_plan.md / findings.md / progress.md
  - `ccr clean planfiles` 默认只处理当前目录根层的 task_plan.md / findings.md / progress.md
  - `ccr clean planfiles --all` 默认不跟随符号链接目录
  - `ccr clean backups` 只处理 ~/.claude/backups 下的 .bak 文件
  - `ccr clean storage` 不删除 analytics/usage.db、data.db、配置、凭据和签到数据
  - `ccr clean storage` 不跟随符号链接
  - `LLMUSAGE_HOME` 指向旧目录时，`ccr clean storage` 保留该目录";

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
