# Implement - `ccr codex fix` 速度优化

## 1. Planning Gate

- [x] 用户批准修订后的 `prd.md` / `design.md` / 本节。
- [x] 批准后才运行 `task.py start`；批准前不改产品代码。

实施前读：

- `.trellis/spec/ccr-codex/backend/codex-app-server-cleanup.md`
- `.trellis/spec/ccr-codex/backend/backend-guidelines.md`
- `.trellis/spec/ccr-codex/backend/test-fixtures.md`
- `.trellis/spec/ccr-cli/backend/backend-guidelines.md`
- `.trellis/spec/ccr-cli/backend/test-fixtures.md`
- `.trellis/spec/ccr-core/backend/managed-process.md`
- `.trellis/tasks/08-17-ccr-codex-fix-speed/research/command-chain-analysis.md`
- `.trellis/tasks/08-17-ccr-codex-fix-speed/research/review-verification.md`

## 2. Process Grace Early-Stop

- [x] 改 `crates/ccr-codex/src/services/codex_process_service.rs`：`targets` 为空则跳出宽限；发过信号则保留 settle。
- [x] 改 `pid_reuse_is_tracked_by_start_time`：空快照后早停；settle 中的 replacement 断言为 `respawned`，无 deadline KILL。
- [x] 新增：早停后 settle 发现 replacement → `respawned`、退出投影为进程不干净。
- [x] 新增或保留：目标一直存在 → 走满 `poll_rounds` 再 KILL。
- [x] 新增：第二轮已空则后续不再 `wait`。
- [x] 保留 Unix 真实 fixture 的 dry-run 发现用例。
- [x] 验证：`cargo test -p ccr-codex codex_process_service -- --test-threads=1`
- 回滚点：只回退该文件循环条件与对应测试。

## 3. CLI Flag And Orchestration

- [x] `CodexAction::Fix` 增加 `--doctor`；`dispatch` 传入三参数。
- [x] 更新 `fix.rs` 模块注释和 Clap 帮助：默认不跑 doctor。
- [x] 默认跳过 PATH 查找、doctor、doctor 后 inspection。
- [x] `--doctor` 缺失二进制 → 127；默认路径按 2/1/3/0。
- [x] 解析测试：裸命令、`--doctor`、与 `--dry-run` / `--repair-runtime` 组合。
- [x] 验证：`cargo test -p ccr-cli --lib codex_fix -- --test-threads=1`

## 4. Doctor Runner (`ManagedProcess`)

- [x] 将捕获逻辑收成可注入 `timeout: Duration` 的函数，生产传 30 s。
- [x] `ManagedProcess::spawn`；并发排空受限 stdout/stderr（`read_bounded_line` 或等价上限）。
- [x] 正常结束走 `wait`；超时走 `terminate_tree` 并 await reap。
- [x] 非 JSON stdout 不再二次 spawn。
- [x] `extract_highlights` 改为顶层字段 + 非 ok 检查 id。
- [x] 默认输出：分段、耗时、`doctor = skipped`；环境只展开已设置变量。
- [x] 集成测试（`ccr-cli` lib 或 `ccr` binary，PATH 用 `TestHostEnv` / 隔离 fixture）：
  - [x] 默认路径：fake `codex` 写计数文件，调用次数为 0。
  - [x] `--doctor` + JSON fixture：渲染 version / overallStatus。
  - [x] `--doctor` + 非 JSON stdout：spawn 次数为 1，走文本路径。
  - [x] `--doctor` + 挂起父进程再拉孙进程：短超时后父/孙均不在进程表。
  - [x] `--doctor` 非 dry-run：报告文件存在且无 sentinel secret。
  - [x] `--dry-run --doctor`：不落盘报告。
  - [x] `--doctor` 期间改 snapshot：退出码 3。
- [x] Unix fake 脚本不得依赖被掏空 PATH 中的外部助手。
- [x] 验证：`cargo test -p ccr-cli --lib fix -- --test-threads=1`；`cargo test -p ccr-core process_gateway -- --test-threads=1`（确认未破坏既有树回收）。

## 5. Binary And Help Tests

- [x] `crates/ccr/tests/commands/codex_fix.rs`：PATH 清空的默认命令不再断言 127；漂移为 3。
- [x] runtime inspection 失败仍跑 fake doctor 的用例改为 `--dry-run --doctor`。
- [x] `help.rs`：帮助含 `--doctor`，并体现默认不跑上游 doctor。
- [x] 验证：`cargo test -p ccr --test commands codex_fix -- --test-threads=1`；`cargo test -p ccr --test commands help -- --test-threads=1`

## 6. Docs And Spec

- [x] 更新中英文 `docs/reference/commands/codex.md`、`docs/reference/platforms/codex.md`。
- [x] Phase 3.3 更新 `codex-app-server-cleanup.md`（默认不跑 doctor、早停 + respawned 语义、127 范围、ManagedProcess、highlights）。
- [x] 验证：`just docs-check`

## 7. Quality Gate

- [x] `just fmt-check`
- [x] `just lint-strict`
- [x] 上述测试全部通过
- [x] 手工：默认 `ccr codex fix --dry-run` 热启动 5 次，记录中位数；确认不启动 doctor，输出含 skipped 与耗时

手工记录（2026-08-17，Windows，`cargo run -q -p ccr -- codex fix --dry-run`）：墙钟中位数 737 ms（含 debug 启动）；进程清理 26 ms，本地诊断 21 ms，`doctor = skipped`。本机 runtime 为 mismatch，退出码 3。

## Risky Files

| 文件 | 风险 |
|------|------|
| `crates/ccr-codex/src/services/codex_process_service.rs` | 早停漏目标，或 replacement 分类错误 |
| `crates/ccr-cli/src/commands/codex/fix.rs` | 进程树回收、退出码、脱敏 |
| `crates/ccr/tests/commands/codex_fix.rs` | 旧 127 / 必跑 doctor 断言 |

## Rollback

各步可独立回退。若必须撤回 `--doctor`，恢复默认调用 doctor，但可保留早停与 `ManagedProcess`。不回退为 `kill_on_drop` 单进程方案。
