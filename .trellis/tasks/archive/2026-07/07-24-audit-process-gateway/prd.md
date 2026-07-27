# 统一 ProcessGateway 与进程能力治理

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P1-06、P1-07、P2-01、P2-02、P2-08、P2-09、P3-04 ｜ 报告 Epic A2/A3/A5

## Goal

把散落各 command 的进程 spawn 收敛为统一 `ProcessGateway`，补齐 timeout、输出上限、背压、进程树生命周期、可信可执行文件与端口/URL 能力边界。

## 背景 / 证据（已核实）

### P1-06 前台无 timeout/输出上限
- `ccr-ui/src-tauri/src/commands/command_exec.rs:1698` — 前台 `cmd.output().await`，无 timeout、无 max bytes、无 streaming

### P1-07 后台无界 channel + 全量 snapshot
- `command_exec.rs:1594` — `mpsc::unbounded_channel::<OutputEvent>()`
- `command_exec.rs:1607-1614` — 每行 `update_and_emit` 推送完整 job snapshot

### P2-08 取消不处理进程树
- `command_exec.rs:1630-1638` — cancel 仅 `child.kill()` 后立即标记 Cancelled；孙进程/进程树不处理
- 细节修正：tokio `child.kill()` 会 reap 直接子进程，报告"未等待退出"表述不准，但进程树问题成立

### P2-09 sidecar 身份靠自报
- `command_exec.rs:1338` — 候选缺失回退 PATH `"ccr"`
- `command_exec.rs:1365-1373` — 身份仅靠 `--version` 自报版本

### P2-01 端口杀进程无归属
- `ccr-ui/src-tauri/src/commands/codex_auth.rs:653-678` — `kill_port_processes` 对端口所有 PID `kill -9`/`taskkill /F`，不校验是否 CCR 启动/executable/start time

### P2-02 URL scheme 无 allowlist
- `codex_auth.rs:577-608` — `open_external_url` 只查非空，直接交给 rundll32/open/xdg-open

### P3-04 O(n) 左移
- `command_exec.rs:1244` — `lines.remove(0)` 逐项左移

## Requirements

### Quick wins（可先落地）
- [x] `open_external_url` 加 scheme/host allowlist：仅 `https` + 预期 OAuth hosts；localhost callback 单独 allowlist；拒绝 file/custom scheme（P2-02）
- [x] 删除 generic `kill_port_processes`，改为只终止 backend registry 中 tracked child；UI 只展示 PID/命令提示不强杀（P2-01）
- [x] ring buffer `Vec::remove(0)` 改 `VecDeque::pop_front`（P3-04）

### ProcessGateway 核心
- [x] 前台默认 60s timeout（命令级 override）；stream + max bytes（默认 ≤1 MiB/stream 可配置）；超限终止子进程并返回 structured truncation（P1-06）
- [x] 后台改 bounded mpsc（如 256）；批量 20-50 行或 ≤20Hz；事件只发 delta `{jobId,seq,channel,lines}`，snapshot 仅在查询/terminal 发；dropped count 明确上报（P1-07）
- [x] cancel 用 Unix process group + `killpg` / Windows Job Object；mark terminal 前 wait/reap；cancel timeout/escalation（P2-08）
- [x] `TrustedExecutable` enum（BundledCcr{expected_sha256} / SystemSsh / Cargo / Homebrew...）替代任意 String；production 不回退 PATH，绑定 sidecar 绝对路径 + hash 校验，开发回退需 explicit flag（P2-09）

## Acceptance Criteria

- [x] 进程测试（报告 §9.1 Process）：endless sleep、stdout/stderr flood、child spawns grandchild、kill denied、queue consumer stalled、binary version spoof、PATH precedence attack fixture
- [x] producer>consumer soak 不 OOM；事件频率 ≤20Hz；cancel 后 5s 内 process tree 为 0（Windows 本机验证）
- [x] file/custom scheme URL 被拒；`kill_port` 只作用于 tracked child
- [x] `just lint-strict` + `just test` 通过
- [ ] Linux/Windows/macOS integration tests 全部通过（Windows 已通过；Linux/macOS 等待 hosted CI）

## Out of Scope

- 不提供任意 executable/参数的通用 renderer 进程启动器
- 不终止 backend registry 之外的 PID 或端口占用进程
- 不把 `--version` 自报结果作为二进制身份的唯一证明

## Notes

- 跨平台 process semantics 风险高，按 capability feature flag 逐个迁移并保留旧 adapter（报告 §5 2A 回滚）
- 触发 tauri-ipc-reviewer + rust-security-reviewer 复查
- 与 typed-ipc 子任务协调：process 域应优先纳入 typed 迁移

## Verification Evidence (2026-07-26)

- `just fmt-check`：通过。
- `just lint-strict`：通过；同时覆盖 sensitive persistence policy。
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml -- --test-threads=1`：288 个 Tauri 测试与 2 个 llmusage 依赖边界守卫通过。
- `just frontend-check-quick`：type-check、lint、90 个测试文件 / 400 个测试通过。
- `just test`：workspace 全测试与 doctests 通过。
- Windows descendant fixture 已证明 Job Object 终止孙进程；Unix process-group fixture 已实现但本机未执行。Linux/macOS hosted matrix 证据保留为父任务最终集成门槛，不将其推断为通过。
- 提交前 escape 扫描确认迁移调用方无直接 child spawn；剩余 WSL 同步适配器与 SkillPort detached GUI handoff 是规范记录的非扩展 legacy adapter。
