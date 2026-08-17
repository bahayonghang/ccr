# 优化 `ccr codex fix` 执行速度

## Goal

缩短 `ccr codex fix` 的墙钟时间，同时保持进程清理、本地 profile/runtime 诊断、`--repair-runtime` 写入门槛和退出码优先级。默认路径只做 CCR 本地工作；上游 `codex doctor` 改为显式 `--doctor`。有残留进程时，匹配目标已空就结束宽限；grace 内之后才出现的 replacement 记为 `respawned`。输出按阶段给出结论和耗时，不再重复打印同一组路径和 provider 字段。

## Background

命令今天串行执行：进程清理 → 只读 inspection → 可选 `--repair-runtime` → 环境提示 → 无条件 `codex doctor` → doctor 后再 inspection。契约见 `.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`。

2026-08-17 本机 Windows / `ccr 7.1.4` / `codex 0.147.0`，状态为 `process_state=clean` 且 `runtime_consistency=match`。下列数字是单次热启动采样，用来定位主因，不是跨机器 SLA：

| 场景 | 墙钟时间 |
|------|----------|
| `ccr codex fix --dry-run` 热启动 | 7708 ms |
| 单独 `codex doctor --json` | 7663 ms |
| PATH 去掉 `codex` 的同一命令 | 91 ms（现退出 127） |

现场 doctor 跑 18 项检查，含网络探活、更新检查和 rollout 对账。有真实 app-server 时，清理阶段再固定加上 `10 × 300ms + 1s`。计时见 `research/command-chain-analysis.md`。审阅校验见 `research/review-verification.md`。

已确认产品决定：

- 默认不调用 doctor，需要时加 `--doctor`（2026-08-17）。
- 修复路径保持全量 `repairable` CAS；pointer-only 移出本任务（审阅后）。
- 宽限早停后，仅在 settle 出现的 replacement 记为 `respawned`，退出码 2（审阅后）。

## Requirements

### R1. 进程宽限在目标已空时结束

- 发出 TERM（Windows 上 TERM 不受支持则立即 KILL）之后，每 300 ms 全表 `discover`，最多约 3 s。
- 某一轮匹配目标为空：立即结束宽限循环，不再继续 `wait`。
- 只要本轮发送过信号，仍做约 1 s settle，再拍最终快照。
- **replacement 语义（新契约）**：空快照之后、settle 里才出现的新 `pid+start_time` 一律进入 `cleanup.respawned`，不在已结束的宽限里补发 deadline KILL。命令以退出码 2 结束，提示再跑一次。
- 截止时仍有匹配目标：对当时身份发 KILL，再 settle；与现在相同。
- dry-run 与初始快照为空：只做一次 discover，立即返回。
- argv 窄匹配、owner fail-closed、`pid+start_time` 身份校验不变。

### R2. 默认不跑 doctor；`--doctor` 才调用上游

- `ccr codex fix` / `--dry-run` / `--repair-runtime` 默认不查找 PATH 中的 `codex`，不启动该二进制，不做 doctor 后 inspection。
- `--doctor` 才运行：先 `codex doctor --json`，以 stdout 是否为有效 JSON 判成功；30 s 超时。
- `--dry-run --doctor` 仍运行 doctor，不写 runtime，不落盘报告。
- `--repair-runtime` 不隐含 `--doctor`。
- 退出码 `127` 仅在传入 `--doctor` 且 PATH 中没有 `codex` 时使用。
- 默认输出用一行说明 doctor 已跳过，并提示 `--doctor`。
- 退出码优先级保持 `127 > 2 > 1 > 3 > 0`。127 的触发条件变窄。

### R3. `--doctor` 用 `ManagedProcess` 管理整棵进程树

- 禁止只靠 `kill_on_drop(true)` 宣称「无残留」。Windows 上 `which_on_path` 可命中 `codex.cmd`，直接子进程是 `cmd.exe`，node 孙进程不会随 drop 可靠退出。
- 必须使用 `ccr_core::core::process_gateway::ManagedProcess`：`spawn` → 并发排空受限 stdout/stderr → 正常路径 `wait`；超时路径 `terminate_tree(grace)` 并 await reap。
- `--json` 已返回非 JSON 文本时，按现有规则脱敏后当文本渲染，不得再启动第二次 `codex doctor`。
- spawn 失败或超时：warning，不 panic；若无进程残留且无本地漂移，退出码 0。
- 生产超时仍为 30 s；实现必须提供可注入的 Duration seam，供短超时集成测试使用。

### R4. 修复路径保持全量 CAS

- `--repair-runtime` 在 `apply_profile` 前继续全量 `inspect_runtime`。
- 仍要求 `current.repairable == true` 且 `resolved_profile` 与快照相同；否则拒绝写入。
- 本任务不改为只读 pointer。doctor 之后的 inspection 仅在实际传了 `--doctor` 时执行；快照变化仍退出码 3。

### R5. 输出按阶段收敛，并暴露耗时

- 保留 `process_state` / `runtime_consistency` / `provider_auth_validity`。后者仍为 `not_checked`。
- 默认输出不得把同一 `CODEX_HOME`、`config.toml`、`model provider` 再打印第三次。
- 环境段只展开已设置的变量；未设置项不逐行占位。
- `--doctor` 高亮：顶层 `codexVersion` / `overallStatus`；所有非 ok 检查的 id 与 status。不再用键名子串 `config|auth|provider|model` 扫 `details`。
- `overallStatus` 非 ok 时必须出现对应检查 id。
- 打印进程清理、本地 inspection 的耗时（毫秒）。doctor 段：已运行则打耗时，跳过则打 `skipped`。
- 脱敏不变。耗时数字用于人读，不作为 CI 绝对门禁。

## Acceptance Criteria

- [ ] 无 app-server 的默认 `ccr codex fix` / `--dry-run`：不启动 `codex` 二进制（带计数文件的 fake 调用次数为 0），进程阶段一次 discover，无 3 s 睡眠。
- [ ] PATH 无 `codex` 的默认命令：不再退出 127；按进程 / runtime / 漂移给出 0/1/2/3。
- [ ] `ccr codex fix --doctor` 在 PATH 无 `codex` 时退出 127。
- [ ] `ccr codex fix --doctor` 仍解析并渲染 doctor JSON；`--dry-run --doctor` 不落盘报告；非 dry-run `--doctor` 落盘脱敏报告（写盘失败则无路径，不影响退出码）。
- [ ] 真实清理中目标在宽限内退出：循环提前结束，仍做 settle。
- [ ] 空快照之后、settle 才出现的 replacement：进入 `respawned`，不对其补发 deadline KILL，退出码 2。
- [ ] 目标在宽限内一直存在：走满约 3 s 后再 KILL，再 settle。
- [ ] `--doctor` 超时后，本次拉起的父进程与其孙进程均不在进程表中（真实挂起父子 fixture，短超时 seam）。
- [ ] `--doctor` 且 `--json` 返回非 JSON：doctor 二进制只被调用一次。
- [ ] `--doctor` 运行期间人为改变 profile/runtime 快照：退出码 3，输出不得把 doctor 结果归到旧 profile。
- [ ] `--repair-runtime --dry-run` 对受管文件字节不变；真实修复仍只走 `apply_profile`，且诊断后 `repairable` 翻转时拒绝写入。
- [ ] 默认干净输出含 `doctor = skipped`（或等价文案）和 `--doctor` 提示；不含 doctor 检查字段。
- [ ] `--doctor` 且 `overallStatus=warning` 时，输出包含对应检查 id，且不再出现 `search provider` / `configured servers` / 重复的 `model provider`。
- [ ] 输出含进程、inspection 耗时；doctor 为耗时或 `skipped`。
- [ ] 现有 secret-free 断言、Unix 伪 app-server 发现测试继续通过。
- [ ] 帮助与中英文档列出 `--doctor`，并写明默认不再调用上游 doctor。
- [ ] 手工复测：默认干净路径热启动 5 次，记录中位数；用于交付说明，不写入 CI 阈值。

## Out of Scope

- 不新增第三方 Provider 凭据探测。
- 不把 `repair_runtime` 改成 pointer-only CAS。
- 不放宽 app-server 匹配，不改 owner fail-closed。
- 不把 `stable_current_profile()` 引入诊断路径。
- 不改 `ccr` 全局冷启动和日志系统初始化。
- 不修改上游 `codex doctor`。
- 不新增 `--verbose` / `--skip-doctor`。
- 不在 TUI / Tauri / VS Code 增加并列入口。

## Technical Notes

- 实现前读：`codex-app-server-cleanup.md`、`ccr-codex` / `ccr-cli` backend-guidelines、`ccr-core/backend/managed-process.md`、`ccr-cli` 与 `ccr-codex` 的 `test-fixtures.md`。
- 兼容变化：裸命令不再因缺少 `codex` 退出 127；裸命令不再保证跑 doctor。
- 兼容变化：grace 内先空后出现的 replacement 从「deadline KILL」改为 `respawned` + 退出码 2。
- PATH 相关测试用 `TestHostEnv`（ccr-cli）或 binary fixture 隔离；Unix fake 脚本不得依赖被掏空 PATH 里的外部助手。
- 验证命令见 `implement.md`。
