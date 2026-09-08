# 补齐 Grok Auth CLI 多账号命令

## 目标
使 `ccr grok auth save gmail` 可用，参考 Codex 补齐已有 Grok 多账号服务的 CLI 闭环。

## 证据
- 已运行 `ccr grok auth save --help`，得到 unrecognized subcommand。PATH 为 C:\Users\lyh\.cargo\bin\ccr.exe。
- 源码 `crates/ccr-cli/src/cli/subcommands/grok.rs:31` 与 `cli/dispatch.rs:618` 仅接入 help/current/off；`cli/help_config.rs:183` 仍称不保存快照（后两路径均位于 crates/ccr-cli/src）。
- `crates/ccr-cli/src/services/grok_auth_service/accounts.rs:311` 起已有 read_snapshot/save_current/delete_account/switch_account/off_checked，TUI 已使用；旧归档任务 09-08-tui-grok-auth-replacement 明确排除了新增 CLI save/switch。因此问题不只是安装版本陈旧。
- Codex 命令参考：`crates/ccr-cli/src/cli/subcommands/codex.rs:293`。

## 需求与验收
- R1/AC1：新增 `save <name> [--scope <scope>] [-f|--force] [--json]`。单一合法 OAuth 来源自动选择；多个来源必须显式指定 scope；无来源/无效 scope 零写入；重名默认拒绝，force 明确覆盖。用户原命令在单来源 fixture 上通过。
- R2/AC2：新增 `list [--json]`、`switch <name> [--json]`、`delete <name> [-f|--force] [--json]`。list 展示保存账号与可选来源；删除默认确认，支持全局 -y/force，取消零写入。错误必须非零退出。
- R3/AC3：保留裸命令 TUI、current 存在性查询及 off 共享协调器。list 输出来源和匹配等安全元数据，区分本地匹配与服务端有效登录。
- R4/AC4：复用保存只写 CCR 库、切换先回存且仅替换目标 scope、删除不登出、off 保留库的契约；损坏存储/未知身份/并发冲突不盲写；config/profile/MCP 保持。stdout/stderr/JSON 无凭据 sentinel。
- R5/AC5：帮助、中英文 Grok 命令文档和所属 spec 一致，不再称不保存快照；说明切换用于先结束当前 Grok 后的新会话。
- R6/AC6：隔离 fixture 覆盖真实 CLI 解析分发及 A/B 保存、回存切换、删除、登出；执行相关服务/帮助回归及格式、lint，最终 just ci。旧任务跳过测试的约束仅属于旧任务。

## 已批准范围
用户于 2026-09-08 回复“批准”，本任务已启动。本轮补齐上述核心能力。Codex update/描述、rename、sync、repair、import/export 没有现成 Grok 对应接口，涉及新增存储或平台特有语义，另行规划，不机械移植。无后台刷新、新依赖、UI 改造、全局安装、真实凭据操作、Git 提交推送或发布。
