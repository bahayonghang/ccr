# Install 计划改为后端 opaque handle

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P1-01、P2-13 ｜ 报告 Epic A1

## Goal

消除"渲染层可提交完整可执行 `InstallPlan`，后端原样执行其中 `command/args/envs`"这一 renderer→RCE 升级链。改为渲染层只持有一次性 opaque `plan_id`，后端保存 canonical plan 并按平台内部生成可执行文件。

## 背景 / 证据（已核实）

- `ccr-ui/src-tauri/src/commands/install.rs:44-49` — `llmusage_install_execute(plan: InstallPlan)` 接收前端完整 plan
- `crates/ccr-cli/src/services/install_types.rs:177-187` — `InstallPlan` 含 `command: String` / `args: Vec<String>` / `envs: BTreeMap` / `plan_id: Uuid`
- `crates/ccr-cli/src/services/install_service.rs:72-97` — `execute` 只做 slot/attempt 管理，不按 `plan_id` 重取 canonical plan
- `crates/ccr-cli/src/services/install_exec.rs:51-57` — `Command::new(&plan.command).args(&plan.args).envs(&plan.envs)`
- `ccr-ui/src/api/domains/install.ts:23-26` — TS `AttemptId` 误声明为带 index signature 的 object，与 Rust transparent UUID string 漂移（P2-13）

## Requirements

- [ ] `llmusage_install_execute` 的 IPC 入参改为仅 `plan_id`（Uuid），不再接收 `command/args/envs`
- [ ] 后端在生成 plan 时（`llmusage_install_plan`）保存 canonical plan 到进程内注册表，携带 TTL（建议 120s）、single-use（consume 后失效）标记
- [ ] execute 时用 `plan_id` 从注册表 consume canonical plan；伪造/过期/重复的 `plan_id` 返回 typed error
- [ ] 首选把 executable/args 收敛为 `InstallAction` enum（如 `CargoInstallLlmusage` / `HomebrewInstallLlmusage` / `ScoopInstallLlmusage` / `WingetInstallLlmusage`），executor 内部按平台生成实际命令，不接受任意字符串 executable
- [ ] 删除手写的 TS `AttemptId` 类型，改用 ts-rs 生成绑定（P2-13）；hosted drift check 与 ci-governance 子任务协调
- [ ] canonical plan 可选绑定当前 host（capabilities/OS），execute 时校验
- [ ] audit event 记录 action / plan_id / attempt_id，不记录 secret env value

## Acceptance Criteria

- [ ] IPC schema 中不再出现 `command`/`args`/`envs` 字段
- [ ] 单元测试证明：任意前端字符串无法进入 `Command::new`
- [ ] 安全回归测试覆盖（报告 §9.1 Install）：forged plan command、modified args/env、expired plan、reused plan、plan for different OS、unknown plan id
- [ ] `just frontend-check-quick` + `just lint-strict` + `just test` 通过

## Out of Scope

- 不把此流程扩展为任意软件包安装器
- 不保留接受 renderer 完整 `command/args/envs` 的兼容入口
- 不在本子任务重构 install 之外的通用进程生命周期；该部分由 process-gateway 负责

## Notes

- 短期阻断可先"隐藏 install 按钮 / 仅展示手动安装说明"（报告阶段 0 回滚策略），但正式修复应落地 opaque handle
- 参考报告 §2.1 推荐实现（`CanonicalInstallPlan` + `consume_once`）
- 涉及 Tauri IPC 与前端类型，触发 tauri-ipc-reviewer / frontend-quality-reviewer 复查
