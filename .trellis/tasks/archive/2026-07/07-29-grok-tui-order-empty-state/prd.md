# 修复 Grok TUI 顺序与空态命令

## Goal

让 Grok Profile 在默认 TUI 中紧跟 Claude Profile，并确保空配置页面和 CLI 帮助不再把用户引导到已退休的 `ccr platform init/switch/current/info/profile` 路径。

用户应能从 Grok 空态直接找到当前有效的 profile 创建入口，而旧命令仍保留可解析性，以便现有脚本得到明确迁移错误。

## Background

- `crates/ccr-config/src/managers/tui_config.rs` 的内置顺序当前为 Codex Profile -> Grok Profile -> Claude Profile；用户要求改为 Codex Profile -> Claude Profile -> Grok Profile。
- `crates/ccr-tui/src/tui/ui.rs:2036` 在所有 Profile 空态中硬编码 `ccr platform init {platform}`，下一行还推荐通用 `ccr add`。
- `crates/ccr-cli/src/cli/dispatch.rs:348-363` 已有意拒绝 `platform switch/current/info/init/profile`，并通过 `legacy_platform_command_error` 指向显式的 Claude、Codex、Grok profile 命令面；因此报错不是 Grok 未安装或 CCR 版本不匹配，而是调用方提示过期。
- 当前有效的 Grok 创建入口是 `ccr grok profile create <NAME>`；空态在不知道 profile 名称时应链接到 `ccr grok profile create --help`。
- 本机未发现 `~/.ccr/tui.toml`，因此内置默认顺序变更可直接修复当前截图；已有完整自定义顺序不得被迁移重写。
- `ccr platform --help` 和根帮助当前仍展示已退休命令，属于同一发现面漂移。

## Requirements

### R1 默认页签顺序

- 内置 Profile 页签顺序必须为 `codex_profile`、`claude_profile`、`grok_profile`，其后保持现有 Auth 页签顺序不变。
- 缺少 `tui.toml` 时使用新默认顺序。
- 已有完整、合法的自定义 `tab_order` 必须原样保留，不自动重排或写回。
- 旧的不完整顺序继续沿用现有兼容规则：保留用户已列出的相对顺序，仅按新的内置相对顺序补齐缺失页签。

### R2 Profile 空态指引

- Claude、Codex、Grok Profile 共用的空态必须推荐平台显式入口：`ccr {platform} profile create --help`。
- 第二条提示必须告知用户创建后按 `r` 重新加载，而不是推荐与当前平台无关的 `ccr add`。
- 英文和简体中文文案必须等价，并保持当前错误空态的行为不变。
- 空态不得再出现 `ccr platform init` 或 `ccr add`。

### R3 CLI 帮助发现面

- `ccr platform --help` 与 `ccr help platform` 不得在命令列表或任务提示中展示已退休的 `switch`、`current`、`info`、`init`、`profile`。
- 根帮助不得再推荐上述已退休平台命令；应指向 `ccr current`、`ccr platform list` 以及显式的 Claude/Codex/Grok profile 帮助入口。
- 旧命令必须继续被 Clap 解析并进入现有迁移错误分支；不得删除枚举变体、恢复旧行为或把错误退化成 `unknown subcommand`。
- `ccr platform init grok` 必须继续非零退出，并包含 `ccr grok profile ...` 的迁移指引。

### R4 约束

- 不读取、创建或修改真实 Grok 配置、token、`~/.grok`、`~/.ccr/platforms/grok`。
- 不修改无关工作树文件 `ccr-ui/src/types/generated/usage/DailyTrendDto.ts`。
- 不恢复全局 `current_platform/default_platform` 路由。
- 不修改 Profile 创建、切换或应用引擎；本任务只修复顺序与发现/提示面。

## Acceptance Criteria

- [ ] AC1：`TuiTabId::default_order()` 精确返回 Codex Profile -> Claude Profile -> Grok Profile -> Codex Auth -> Claude Auth -> OpenCode Auth。
- [ ] AC2：缺失配置使用 AC1 顺序；完整自定义顺序原样 round-trip；旧不完整顺序只补缺失项，不重排已列项。
- [ ] AC3：Grok 空态的英文与中文都包含 `ccr grok profile create --help` 和按 `r` 重载提示，且不包含 `ccr platform init` 或 `ccr add`。
- [ ] AC4：Claude/Codex 空态同样生成各自的平台显式创建帮助，证明指引不是 Grok 特判。
- [ ] AC5：`ccr platform --help` 与 `ccr help platform` 内容一致，只公开仍受支持的 platform 发现入口；根帮助不再推荐退休命令。
- [ ] AC6：`ccr platform init grok` 仍返回既有 legacy migration 错误，且明确包含 `ccr grok profile ...`。
- [ ] AC7：相关 crate 测试、严格 lint、workspace 测试和最终 `just ci` 通过；无关生成文件内容保持不变。

## Out of Scope

- 恢复或重新设计 `ccr platform init/switch/current/info/profile`。
- 修改 Grok Profile 数据模型、配置样例、认证信息或实际 runtime 切换。
- 全仓公共文档及其他历史运行时提示中的旧命令迁移。现有 Gemini/Droid 等页面缺少统一的显式 profile 替代语义，需另立任务逐平台审定，不能机械替换。

## Deferred Follow-up

- 审计并迁移 `docs/`、`ccr init`/`ccr version` 等其他历史输出中的 `ccr platform ...` 与平台不明确的 `ccr add` 指引；为 Gemini/Droid 等非 Claude/Codex/Grok 平台先确定有效替代流程。
