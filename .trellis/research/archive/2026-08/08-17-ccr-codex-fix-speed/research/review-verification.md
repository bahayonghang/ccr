# Codex 审阅校验（2026-08-17）

对照仓库源码核验 Canvas 审阅中的 4 个启动前阻塞项与 2 个重要项。任务文件在本轮修订前未改产品代码。

## 1. `kill_on_drop(true)` 不能回收 Windows `.cmd → node` 树 — 成立

现场 PATH 上的 `codex` 是 `C:\home\lyh\.npm-global\codex.CMD`。`which_on_path` 在 Windows 上按 `codex.exe` / `codex.cmd` / `codex.bat` / `codex` 查找（`install_detect.rs` 207–214 行）。

`tokio::process::Command` 的 `kill_on_drop(true)` 只保证终止直接子进程。对 `.cmd` 来说直接子进程是 `cmd.exe`，node 孙进程不在该契约内。设计原稿 §10 已承认这一点，却仍把 `kill_on_drop` 当作满足 AC「超时后不得留下本次 doctor 子进程」的方案，自相矛盾。

仓库已有可复用实现：

- 规范：`.trellis/spec/ccr-core/backend/managed-process.md`
- 实现：`ccr_core::core::process_gateway::ManagedProcess`
- Windows：Job Object + `KILL_ON_JOB_CLOSE` + `TerminateJobObject`
- Unix：新进程组 + 负 PID 信号
- 调用约定：超时后 `terminate_tree(grace)` 并 await reap；禁止把 Drop 当正常取消路径
- `ccr-cli` 的 `install_exec.rs` 已按此模式 spawn
- `process_gateway` 已有 Windows 孙进程 / Unix 进程组回收测试（约 472–521 行）

修订：`--doctor` 必须走 `ManagedProcess`，并带可注入的短超时 seam 与真实挂起父子进程测试。

## 2. Pointer-only CAS 丢失 `repairable` 复检 — 成立

`repair_runtime_with_env`（`platforms/codex.rs` 1426–1451 行）在 `apply_profile` 前全量 `inspect_runtime_with_env`，要求：

- `current.repairable == true`
- `current.resolved_profile == snapshot.resolved_profile`

`repairable` 还依赖 `config.toml`、`auth.json`、环境覆盖（`OPENAI_BASE_URL`、非目标 `OPENAI_API_KEY` / `CODEX_API_KEY`、`CODEX_HOME` 不一致）以及同名 profile 内容。只复读 pointer 无法看见这些翻转。

收益：仅显式 `--repair-runtime` 路径上的一次全量读盘，约数十毫秒。主耗时是 doctor（约 7.6 s）和有进程时的宽限睡眠。

修订：从本任务 MVP 删除 pointer-only CAS。R4 改为「保持全量 CAS」。

## 3. 早停改变 PID reuse / replacement 语义 — 成立

现测 `pid_reuse_is_tracked_by_start_time` 的快照序列为：

1. 初始 `(301, start=1)`
2. 空
3. reuse `(301, start=2)`
4. settle `(301, start=3)`

生产循环在 TERM 后固定再 `discover` 两次（测试 timing）或十次（生产）。空快照之后仍继续，因此 reuse 会在截止点被 KILL，settle 里的 start=3 才是 `respawned`。

若「当前 targets 为空立即结束宽限」：

- 第 2 个快照为空 → 跳出
- 不再对 reuse 发 KILL
- settle 看到的第一个新身份成为 `respawned`
- 退出码 2，需要再跑一次命令

这是可接受的速度/覆盖取舍，但必须写成明确产品语义，并改测试与 spec。原稿把 3 s 窗口说成「空转不增加检测能力」不准确：空转会把 grace 内 replacement 升级为 deadline KILL。

修订：锁定新语义；更新 PID reuse 用例；新增「早停后 settle 发现 replacement → respawned」和「拒不退出 → 走满轮次再 KILL」。

## 4. 验收计划不足以证明编排 — 成立

原稿 implement 只写「更新单测」。下列场景没有列为必测：

- 默认路径 fake `codex` 调用次数为 0
- `--json` 非 JSON 只 spawn 一次
- 超时后整棵进程树消失
- doctor 期间快照变化 → 退出码 3
- `--doctor` 落盘 / `--dry-run --doctor` 不落盘

修订：写入 implement 清单，并为 doctor 超时提供可测 Duration seam。

## 5. JSONL 缺关键规范 — 成立

当时 `implement.jsonl` 含 seed 行和 `design.md`；缺 `managed-process.md`、两侧 `test-fixtures.md`、`ccr-codex` backend-guidelines。`check.jsonl` 同样缺进程树与夹具规范。

## 6. 文档一致性 — 成立，不阻塞架构

控制流摘要曾写成 `2 > 1 > 3 > 0`，后文才补 127。模块注释、Clap 帮助、现有 docs 仍写默认跑 doctor。性能数字是单次采样，不能当 CI 绝对门禁。

## 结论

四条阻塞项均被源码证实。主方案（默认跳过 doctor、收窄 127、输出收敛、宽限早停）保留，并按上文修订写入规划。任务保持 `planning`。
