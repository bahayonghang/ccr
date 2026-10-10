# 添加 ccr codex auth relogin：本地移除登录后重新登录，不吊销远端 token

## Goal

新增 `ccr codex auth relogin`。该命令移除当前本地 Codex 登录，但不调用 `codex logout`，因此 OpenAI 服务端不吊销该账号的 refresh token，ccr 已保存的账号快照继续有效。之后用户在本地重新登录新账号。

## Background（详见 `research/root-cause.md`）

- `codex login`、`codex logout`、codex TUI `/logout`、app-server `account/logout` 都会向 `https://auth.openai.com/oauth/revoke` 吊销当前 refresh token（openai/codex PR #17825）。
- 2026-10-10 两次 codex-tui 发起的 `account/logout` 分别吊销了 khanh 和 163 的 token。ccr 快照保存的是同一个 token，所以切换回去后报 `refresh_token_invalidated`。
- app-server 守护进程在内存中缓存 tokens。`account/logout` 吊销的是守护进程缓存的 token（上游测试 `auth_manager_logout_with_revoke_uses_cached_auth`）。只删除 `auth.json` 不能阻止经守护进程发起的吊销。

## Requirements

- R1 新增子命令 `ccr codex auth relogin`。
- R2 移除本地登录时不执行 `codex logout`，也不向任何 revoke 端点发请求。
- R3 删除前把 runtime 中轮换过的 tokens 回写到匹配的已保存账号（沿用 `auth off` 现有的 `sync_runtime_with_saved_account_best_effort`）。
- R4 删除前备份 `auth.json`（沿用 `auth off` 现有的备份、删除流程）。
- R5 凭据存储为 keyring 或 auto 时，命令拒绝执行并说明原因。这两种模式下 ccr 只能调用 `codex logout` 清除凭据，会吊销远端 token。
- R6 检测到托管 app-server 守护进程时，停止该守护进程，使其丢弃缓存的 tokens。下次启动 codex 时守护进程自动拉起并读取新的 `auth.json`。
- R7 本地移除完成后，在当前终端启动 `codex login`（继承 stdin/stdout/stderr），等待其退出。此时 `auth.json` 已不存在，`codex login` 的登录前清理没有可吊销的 token。
- R8 `codex login` 成功后，提示用户用 `ccr codex auth save <name>` 保存新账号；`codex login` 失败或不可用时，报告失败，并说明本地已登出、可手动运行 `codex login`。
- R9 帮助文本（clap 子命令说明、`help_config.rs` 常用任务）和命令参考文档（`docs/reference/commands/codex.md`、`docs/en/reference/commands/codex.md`）加入 `relogin`。

## Acceptance Criteria

- [x] `ccr codex auth relogin --help` 显示命令说明。
- [x] file 存储下：`auth.json` 被备份后删除；匹配的已保存快照在删除前完成回写；整个流程没有启动 `codex logout`。
- [x] keyring/auto 存储下：命令返回错误，`auth.json`、keyring 和守护进程都没有变化，也没有启动 `codex logout`。
- [x] 有托管守护进程时，删除 `auth.json` 后、启动 `codex login` 前执行守护进程停止；停止失败时输出警告和手动命令 `codex app-server daemon stop`，并中止，不启动 `codex login`。
- [x] 单元测试覆盖：store 判定拒绝路径、守护进程停止失败时不启动登录、登录命令参数为 `login`。
- [x] `just fmt-check`、`just lint-strict`、`just test` 通过；docs 改动通过 `cd docs && bun run build`。

## Out of Scope

- Codex Auth TUI 新增按键（TUI 已有 `o` 本地登出）。
- 登录完成后自动保存快照。
- 已被吊销的账号（khanh、163）的恢复。用户需重新登录这两个账号并用 `save --force` 覆盖快照。

## Decisions

- Q1 已定：relogin 自动启动 `codex login`。
- Q2 已定：守护进程用 `codex app-server daemon stop`（codex-cli 0.162.1 已确认有 stop 子命令）。
