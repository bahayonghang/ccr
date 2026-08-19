# Design - `ccr codex fix` 速度优化

## 1. Design Goal

默认路径只做本地清理和 CCR 诊断，把约 7 s 的 `codex doctor` 移到 `--doctor`。进程宽限只为仍匹配的目标服务。`--doctor` 一旦启动，必须按 `ManagedProcess` 回收整棵进程树。修复写入门槛保持全量 `repairable` CAS。

## 2. Boundaries

### `ccr-core`

不改 `ManagedProcess` 契约。`ccr-cli` 按 `.trellis/spec/ccr-core/backend/managed-process.md` 调用：`spawn`、排空管道、`wait` / `terminate_tree` + reap。

### `ccr-codex`

改 `CodexProcessService` 宽限循环。不改 argv 分类、owner 校验、refresh kind、公开结果结构。不改 `repair_runtime` 的全量 CAS。

### `ccr-cli`

改 `CodexAction::Fix` 增加 `doctor: bool`；编排默认跳过 doctor；doctor 路径改走 `ManagedProcess`；收敛渲染。

TUI / Tauri / VS Code 不调用 `fix_command`。

## 3. Control Flow

```text
A  cleanup_report(dry_run)                 // 始终
B  inspect_runtime                         // 可失败，不中断后续独立段
C  optional repair + full inspect          // --repair-runtime；CAS 保持全量
D  env hints                               // 始终，紧凑
E  if doctor {
     which_on_path("codex")?               // 缺失 → 127
     run_codex_doctor via ManagedProcess
     inspect_runtime                       // 变化 → 退出码 3
     render_doctor
   } else {
     render doctor=skipped
   }
G  exit 127 > 2 > 1 > 3 > 0
```

`--repair-runtime` 不隐含 `--doctor`。

## 4. Process Grace Early-Stop

`cleanup_with_backend` 在已发送至少一次信号后：

1. 每 300 ms `discover`，最多 10 轮。
2. 当前 `targets` 为空则跳出循环。
3. 跳出后若仍有目标，对当时身份发 KILL。
4. 只要本轮发送过信号，就 `wait(RESPAWN_SETTLE)` 再做最终快照。

dry-run / 初始为空：一次 discover，零等待。

### Replacement 语义（相对旧实现的明确变化）

旧：空快照之后仍走完剩余宽限；期间出现的新 `pid+start_time` 记入 `discovered_during_cleanup`，截止点对其 KILL。现测 `pid_reuse_is_tracked_by_start_time` 依赖此行为。

新：第一次空快照结束宽限。之后在 settle 才看到的新身份只进入 `respawned`，退出码 2。用户再跑一次命令。

`escalates_every_target_present_at_deadline` 不受影响：宽限内从未变空，仍会 KILL 截止时目标。

## 5. `--doctor` 与 `ManagedProcess`

Clap：`Fix { dry_run, repair_runtime, doctor }`。帮助写明默认只做本地工作。

`run_codex_doctor(bin, persist, timeout)`：

1. 组装 `tokio::process::Command`：`stdin` null，`stdout`/`stderr` piped。
2. `ManagedProcess::spawn`。Job Object / 进程组在 spawn 时挂上，覆盖 `.cmd → node` 孙进程。
3. `take_stdout` / `take_stderr`，并发用 `read_bounded_line` 或有上限的读把管道排空，避免阻塞。
4. `tokio::time::timeout(timeout, child.wait())`：
   - 完成：收集 stdout。
   - 超时：`terminate_tree(1s)` 并 await；返回 `DoctorError::Timeout`。
5. 禁止把 Drop 当正常超时路径。`kill_on_drop` 只是 `ManagedProcess::spawn` 的底层保险。

JSON 处理：

- stdout 是 JSON → 现脱敏 / 高亮 / 可选落盘。
- 已有 stdout 但不是 JSON → `sanitize_doctor_text`，**不**再 spawn 第二次。
- spawn 失败 / 超时 → warning。

`extract_highlights`：

1. 顶层 `codexVersion`、`overallStatus`。
2. `checks` 里 `status != "ok"` 的 `id` + `status`。
3. 删除 `HIGHLIGHT_KEYS` 子串扫描。

`timeout` 必须是参数，生产传 30 s，测试传数百毫秒。

## 6. Repair CAS

保持 `repair_runtime_with_env` 现语义：全量 inspect，要求 `repairable` 且 `resolved_profile` 未变。本任务不改该函数的读盘策略。

## 7. Output

`ColorOutput::step` 分段，带毫秒耗时。

默认：

- `process_state` + 清理摘要
- 一份 runtime 字段
- 已设置的环境变量
- `doctor = skipped` + `--doctor` 提示

`--doctor` 追加 version、overallStatus、非 ok 检查 id、报告路径。

修复成功后打成功句和修复后的 `runtime_consistency`，不完整复述路径表。

同步更新 `fix.rs` 模块注释、Clap 帮助、中英文档：删除「清理后总是运行 doctor」。

## 8. Compatibility

| 旧行为 | 新行为 |
|--------|--------|
| 裸命令总是跑 doctor | 仅 `--doctor` |
| PATH 无 `codex` → 127 | 仅 `--doctor` 时 127 |
| `--json` 非 JSON 再跑一遍 | 用已有文本 |
| TERM 后固定 3 s + 1 s | 目标空则停，仍 1 s settle |
| grace 内空后再出现的 PID | 旧：deadline KILL；新：`respawned` + 退出码 2 |
| 修复前全量 CAS | 不变 |

## 9. Spec Updates (Phase 3.3)

`.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`：

- CLI 增加 `--doctor`；默认不跑 doctor。
- 127 仅绑定 `--doctor`。
- 动态终止：允许空目标早停；settle 期新身份为 `respawned`。
- doctor 必须 `ManagedProcess` + `terminate_tree`；非 JSON 不二次启动。
- highlights 改为非 ok 检查 id。

同步双语 docs。

## 10. Risks

- Desktop 在 settle 之后拉起 app-server：退出码 2 或下次再跑。默认输出必须写清。
- 用户以为裸命令仍做上游健康检查：必须出现 skipped 提示。
- `AssignProcessToJobObject` 失败时 `ManagedProcess::spawn` 返回错误，按 doctor spawn 失败 warning，不得留下未托管子进程。
