# ccr codex fix 清理 app-server 并诊断配置

## Goal

新增 `ccr codex fix` 子命令，把用户手写的 `codexfix` bash 脚本能力内建到 CCR：跨平台清理当前用户残留的 Codex `app-server` 进程，然后运行 `codex doctor --json` 展示实际加载的配置与认证来源。用于修复「第三方 URL / Key 切换后不生效」——根因通常是 SSH / Desktop / VS Code Remote 会话留下的 `app-server` 仍在按旧登录态服务。

## Background

- 触发场景：远程 Linux 服务器上，Codex Desktop / VS Code Remote-SSH 断开后 `app-server` 未退出，继续锁定旧的 URL/Key，导致 `~/.codex` 配置切换不生效。
- 现状：CCR 已直接管理 `~/.codex`（auth.json / config.toml）与多账号，但从不清理 Codex 运行进程，也从不调用 `codex` 二进制。`codexfix` 脚本填补的正是「进程清理 + 运行时诊断」这一环。
- 归属：Codex 域，命令挂在既有 `ccr codex <action>` 树下（已确认命令形式为 `ccr codex fix`）。

## Requirements

### R1 进程清理（核心）

- 枚举**当前用户**下、命令行匹配 Codex `app-server` 的进程（等价脚本正则 `codex.*app-server`）。
- **仅** app-server：不得终止 `codex`、`codex exec`、`codex resume` 等普通 CLI 任务，也不得误杀 `ccr` 自身。
- 终止策略跨平台（已确认「跨平台」）：
  - Unix：先 `SIGTERM`，最多等待约 3 秒让进程自清理 socket/DB/状态；仍存活者再 `SIGKILL`。
  - Windows：无优雅信号语义，直接进程终止（terminate）。
- 终止后重新检查；若 app-server 被重新拉起（Desktop / VS Code Remote 仍连接），输出明确告警并以退出码 `2` 结束，提示关闭客户端后重试。

### R2 运行时诊断（核心）

- 校验 PATH 中存在 `codex`；不存在时给出可执行的中文错误，并以退出码 `127` 结束（对齐脚本 command-not-found 语义）。
- 运行 `codex doctor --json`，将完整报告保存到临时文件并打印其路径；尽力（best-effort）提取并高亮关键字段（CODEX_HOME、config.toml 路径、model provider、auth 模式/来源等）。
- 兼容旧版 `codex`：`--json` 不受支持时优雅降级（回退到 `codex doctor` 文本输出并原样呈现，而非报错崩溃）。
- 为 `codex doctor` 的网络探活设置超时，避免命令挂死。

### R3 环境提示（核心，脚本行为的一部分）

- 打印可能影响 Codex 的当前进程环境变量状态：`CODEX_HOME`、`OPENAI_BASE_URL` 直接显示值；`OPENAI_API_KEY` 仅显示「已设置/未设置」，**绝不打印密钥值**。

### R4 `--dry-run`（核心 · MVP，已确认纳入）

- `ccr codex fix --dry-run`：只枚举并列出将被清理的 app-server 进程，**不发送任何信号、不终止**。跳过重生复检；诊断（doctor / 环境提示）仍照常执行。

## Constraints

- `CcrError` 变体集**已冻结**：只能复用现有可执行变体，不得新增变体。
- 安全：绝不记录/打印 access/refresh token、provider API key、OAuth 负载或 `auth.json` 原文；`codex doctor` 自身已做脱敏，CCR 侧亦不得反脱敏。
- 复用既有依赖 `sysinfo`（0.38.4，已是 `ccr-codex` 跨平台依赖）完成进程枚举与信号终止；**不新增第三方进程管理依赖**（不 shell 出 `pgrep`/`pkill`/`taskkill`）。
- 遵守 ccr-codex 域边界：进程清理这类可复用、可测试的域逻辑放在 `ccr-codex` service；CLI 仅做命令装配与呈现。
- 路径解析统一走 `CodexPaths`，保留 `CCR_CODEX_DIR` / `CCR_DATA_DIR` 覆盖。
- 跨平台构建须保持绿色（Windows 与 Unix 均编译通过、Clippy strict 通过）。
- 实现注释用中文，公共 API doc 用英文。

## Out of Scope

- 不改动 `~/.codex/auth.json` 或 `config.toml`（不删、不改，纯诊断 + 进程清理）。
- 不触及账号切换 / profile 路由逻辑（那是既有 `ccr codex auth/profile`）。
- 不做 Tauri UI 暴露（本次仅 CLI；如后续 UI 需要再抽取）。
- 不实现自动关闭 Desktop / VS Code 客户端（脚本也不做，只提示）。

## 已决决策

- `--dry-run`：**纳入 MVP**（见 R4）。对破坏性操作是合理的安全默认，且与 `ccr codex sync-history --dry-run` 惯例一致。
- `--json`（命令整体结果结构化输出）：**本次不做**（简单优先，用户未点名）。如后续扩展有需要再增。
- 用户范围过滤：**不做显式 user_id 过滤**。理由：非特权用户在 OS 层本就只能向自己的进程发信号（他人进程 `kill` 直接 EPERM 失败），app-server 窄匹配已足够；命令按 PID 报告终止成功/失败即可。避免引入 sysinfo `user` feature 的不确定性。

## Acceptance Criteria

- [ ] `ccr codex fix` 与 `ccr codex fix --dry-run` 均可解析并出现在 `ccr codex help` 中。
- [ ] `--dry-run` 只列出、不终止任何进程（无信号发送）。
- [ ] 存在 Codex `app-server` 时：Unix 走 SIGTERM→（超时后）SIGKILL；Windows 走 terminate；命令打印被终止的 PID 列表。
- [ ] `codex exec` / `codex resume` / 普通 `codex` / `ccr` 进程在任何情况下**不被**终止（分类器单测覆盖）。
- [ ] app-server 被重新拉起时：打印告警且退出码为 `2`。
- [ ] PATH 无 `codex` 时：中文可执行错误 + 退出码 `127`。
- [ ] 正常路径运行 `codex doctor --json`，保存报告并打印路径，高亮关键字段；旧版无 `--json` 时不崩溃（降级文本输出）。
- [ ] 环境提示区块打印 CODEX_HOME / OPENAI_BASE_URL 值与 OPENAI_API_KEY 的存在性（密钥值不外泄）。
- [ ] 无任何 token/密钥值出现在输出或日志中。
- [ ] `just fmt-check`、`cargo test -p ccr-codex -- --test-threads=1`、`cargo test -p ccr --test commands -- --test-threads=1`、`just lint-strict` 全绿；Windows 与 Unix 均编译通过。

## Notes

- `codex doctor --json` 为外部 CLI（openai/codex#22336）能力：输出以 `checks`（按 check id 键控）+ `details`（key/value）结构化并**已脱敏**；`--summary` 为人读视图；旧版可能不支持 `--json`。字段高亮须按「包含子串」best-effort 提取，不得硬编码脆弱的 grep 模式。
