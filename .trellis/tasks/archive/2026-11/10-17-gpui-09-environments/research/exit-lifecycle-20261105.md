# GPUI 09 退出生命周期夹具收据

日期：2026-11-05（America/New_York）。实施基线为 `dev@38487daf4f7a0decde6c14c8452c55e350e30bbd`。本切片新增 Windows 集成测试，不修改产品实现、共享 owner、依赖、锁文件或 Tauri 载荷；9.1–9.6 不勾选，任务不归档。

## 实施与证明范围

`ccr-desktop/tests/runtime_shutdown.rs` 调用产品已有 `RuntimeOwner`、`RuntimeHandle::job`、`CommandJobs::run_fixture_blocking` 和 `--native-command-fixture`。测试自身作为闭合父进程，启动现有原生二进制的合成后代模式；没有打开 GPUI 窗口。fixture root 是 `outputs/desktop` 下唯一临时目录，具有现有 ownership marker；不更改 HOME/USERPROFILE，不运行真实 CCR CLI 命令。Windows 子进程隐藏启动。

`shutdown_reclaims_a_view_detached_command_and_its_descendant`：

- 等待合成后代输出 `Synthetic process progress 0/20`，读取父进程创建的 PID 握手文件，并打开两个 retained Windows 进程句柄；两个句柄在关停前都确认存活。
- 命令为 Running、运行时任务大于零、无 terminal 时丢弃视图订阅，再调用最终 `owner.shutdown()`。这是视图 detach，不是 `ManagedProcess::spawn_detached`。
- 关停后保留命令状态为 Cancelled、任务为零、拒绝新工作；以原进程句柄确认父和后代退出，不把 PID 不存在或拒绝访问解释为成功。
- 每个句柄的最终确认最多等待 1 秒。独立 check 发现仅此限制不能排除 `owner.shutdown()` 等待后代自然运行 10 秒才返回；现已从 job 入队前计时，要求进程启动、ready、完整 shutdown 和两个句柄确认总计小于 5 秒，并同时要求取消终态。这是排除自然完成的夹具因果断言，不是性能验收。失败路径 guard 仍关闭运行时；retained process handle 的 Drop 对仍存活的本 fixture 进程执行终止和有界等待。

`final_owner_join_survives_an_expired_quit_observer`：

- 通过进入握手确认一个读任务正在等待显式释放。
- 模拟 200ms quit observer 超时并丢弃该 future，确认 cancellation 已请求但没有 terminal、任务仍在运行。
- 最终 owner join 在另一个已握手线程执行；释放读任务前，完成通道的有界等待超时，不能把线程尚未调度当作等待证明。
- 显式释放读任务后，最终 join 完成，保留 Cancelled 且任务数为零。此测试不调用 OS 原生关闭事件，也不是性能采样。

## Windows 终止信号观察

首版集成测试将关停后的 `WaitForSingleObject(handle, 0)` 当成即时退出证明，窄测试退出 101。保留日志 `runtime-shutdown-initial-failure.log`，未将它称为产品修复前失败。

源码链中 `ManagedProcess::terminate_tree` 等待直接子进程 reap，并检查 Windows Job 的 `BasicAccountingInformation.active_processes == 0`；`CommandJobs` 还等待 stdout/stderr reader joins 后才发布终态。增加标识诊断后实际观察为：父句柄 initial/final 均 0；后代 initial 为 258，最终为 0，有界等待耗时 **1525 微秒**（`runtime-shutdown-handles.log`）。这说明该次观测存在 Job 清空/读管道结束与后代句柄 signaled 间的短暂间隙；不声称 owner 返回瞬间句柄已 signaled，也不将这份日志推导成其他进程类型或 OS 的结果。

未发现需要在本切片修改共享 owner 的缺陷；现有规范已经要求最终 join 和真实进程树确认，不新增第二套清理实现。独立 check 发现自然结束可能造成假通过，主会话将 admission 前计时覆盖整个关停链路的验证要求补入 `shared-desktop-services.md`，不更改产品合同。

## 实际命令与退出码

全部从仓库根运行，日志目录 `outputs/desktop/gpui09-exit-20261105/`。

| 命令/阶段 | 退出码与结果 |
| --- | --- |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --test runtime_shutdown -- --skip export_bindings` 首次 sandbox 调用 | 1；无法验证 mbx shim identity，尚未执行测试；随后使用既有缓存访问权限 |
| 同命令，首版零等待断言 | 101；1 passed、1 failed、1 helper ignored；失败日志保留 |
| 同命令，有界确认后 | 0；2 passed、1 helper ignored |
| 同命令加 `--nocapture` 的句柄诊断 | 0；2 passed、1 helper ignored，记录上述父/后代信号 |
| `just desktop-fmt-check` | 首次 1，仅本新测试 assert 格式；手工修正后 0，初始日志保留 |
| `just desktop-lint` | 0；all-target 严格 Clippy |
| `just desktop-test` | 首次 101：既有用量 SQLite 字节比较失败，101 passed/1 failed；按授权完整重跑一次后 0 |
| 最终完整 desktop suite | Python 12、AccessKit 3、库 102、native_shell 8、native_ui 2、新 runtime_shutdown 2 均通过；闭合 helper 默认 ignored，并由进程树测试显式启动 |
| 根 `just ci` | 本切片未运行；没有共享产品/跨模块改动，不把前序 CI 当成本轮执行 |

测试保持默认并行与 `--skip export_bindings`。既有 hard-link 缓存复制警告未修改；未削弱字节比较、覆盖率或门禁。最终完整 suite 包含新增 final-join 握手与 `view_detached` 命名。

## 并行工作与剩余验收

在完整 desktop-test 期间，另一会话修改 `crates/ccr-codex/src/models/codex_auth.rs`、`crates/ccr/tests/commands/output_presentation.rs`，并新建 `.trellis/tasks/11-05-codex-auth-save-default-store/`。2026-11-06T03:01:00Z（纽约 11 月 5 日 22:01）核对时 HEAD 仍为上述基线；ccr-codex 的并行改动触发 desktop 重编译。这些文件未由本切片编辑、回滚或纳入提交，门禁是该时刻共享工作区的结果，不能冒称隔离 clean-tree 构建。

本轮没有 WSL/doctor、SSH 连接/keyscan、密码或身份文件读取，没有 UI 操作、截图或新窗口栈。Windows 合成 process/task 清理取得集成证据；真实 native window close、watcher 清理、托盘/隐藏恢复、macOS/Linux 原生壳、WSL 实机读写、真实环境切换及 glass/solid 原生矩阵继续保持既有 not_run/unsupported。9.4 整项和任务仍开放。独立 check 的因果断言修正、首次认证消费断言失败、用户新标准下的适配与最终通过见 [检查收据](check-exit-lifecycle-20261105.md)，上表仅为实施阶段结果。
