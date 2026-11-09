# GPUI 09 退出生命周期独立检查

日期：2026-11-05（America/New_York）。检查时 HEAD 为 `38487daf4f7a0decde6c14c8452c55e350e30bbd`；工作区同时存在另一会话的 Codex 认证存储变更，不是 clean-tree 收据。范围为新增 `ccr-desktop/tests/runtime_shutdown.rs`、09 的 implement/evidence/实施收据，以及主会话同步的 `shared-desktop-services.md` 测试规范；另新增本检查收据。用户最新授权追加 `ccr-desktop/src/runtime/platforms.rs` 的新默认 File 消费测试适配，生产代码仍未修改。

## 已修正

- 原测试只对最终句柄等待设 1 秒上限。若 `owner.shutdown()` 等到合成后代自然运行 10 秒后才返回，Cancelled 和 signaled 仍可能满足，存在假通过。现在从 `RuntimeHandle::job` 入队前计时，覆盖进程启动、ready、整个 shutdown 及两个句柄确认，总计必须小于 5 秒；这排除自然结束，不是性能预算。
- 同步实施计划与收据，明确不能将实施阶段门禁代替修正后最终检查。主会话将这一可复用验证约束补入共享测试规范；产品 API、运行时和进程清理实现未改。

## 调用链与断言核对

- 合成父进程由真实 `CommandJobs::run_fixture_blocking` → `ProcessGateway`/`ManagedProcess` 启动，后代只运行既有闭合 `--native-command-fixture`。父/后代输出和 PID 握手完成后，两个 retained Windows handle 均验证存活，之后才 detach 视图并关停。
- `ManagedProcess::terminate_tree` 等待直接子进程 reap、确认 Windows Job active processes 为零；`CommandJobs` 还 join 输出 readers 后才保留 Cancelled。最终 OS 句柄等待独立于 Job accounting：本次窄测试父句柄即时 signaled，后代初始 258，807 微秒后 signaled。没有宣称返回瞬间 signal 或所有 OS 通过。
- 读任务 entry 和 final-join 线程 entry 均有握手；释放工作之前完成通道有界超时，释放后 join 完成且终态 Cancelled/active tasks 零。取消 observer future 不会被当成任务已退出。
- 测试断言失败时，retained process handle 会终止仍存活的 fixture 进程并有界等待；runtime guard 请求取消并 join。阻塞读的释放 sender 在 runtime guard 之前析构，失败会断开 channel 而非永久占住读；final-join 失败路径也会断开释放 channel。合成后代本身有 10 秒有限生命周期，产品 command cleanup/reader joins 有既有 deadline，不增加永不退出的 helper。
- 显式 fixture 路径和 marker、子进程 env 注入、隐藏窗口启动保持；没有改全局 HOME/USERPROFILE，没有真实 CLI、WSL/doctor、SSH/keyscan、凭据读取或 UI 操作。Windows-only 测试不证明其他 OS、真实窗口关闭、watcher 或托盘。

## 首次完整门禁失败与用户新标准

`just desktop-test` 退出 **101**：`runtime::platforms::tests::codex_auto_store_and_grok_demo_do_not_invent_login` 在 `ccr-desktop/src/runtime/platforms.rs:1148` 得到 `NotDetected`，期望 `StoreUnsupported`（库测试 101 passed / 1 failed，后续 integration 未由该命令执行）。

源码证据：并行改动把 `CredentialStoreKind::from_config_value(None)` 默认值从 `auto` 改为 `file`；该 GPUI fixture 未显式配置 store，也没有 auth.json，因此返回 Missing → NotDetected。它不是 SQLite 偶发字节比较失败，没有通过重试碰运气或回退共享产品语义。

用户随后明确要求“不用建立新的 worktree，按照新的标准来”，并授权当前 GPUI 消费边界适配。检查前 `platforms.rs` 没有并行修改；本轮只在其 `cfg(test)` 中：

- 给原 auto 场景显式写 `cli_auth_credentials_store = "auto"`，继续断言 StoreUnsupported，不用新默认偷换显式 auto 的语义。
- 新增 `codex_default_file_store_without_auth_is_not_detected`，分别覆盖缺 config 文件和存在 config 但缺 store 设置；断言 NotDetected、未登录、无账户、配置内 API key 不进入投影、auth.json 不被创建且 config 字节/缺失状态保持。

现有生产映射已正确消费 Missing → NotDetected，无需修改。这个新消费测试依赖另一任务的共享默认 File 改动；最终通过是当前共享工作区结果，不能称原基线加本切片独立可过。同步后重新跑必要窄测与完整门禁，没有放宽断言。

并行范围包括 `crates/ccr-cli` 的 auth-off/auth-save、`crates/ccr-codex` 的 model/platform/service、CLI output integration test、对应 specs 和 `.trellis/research/archive/2026-11/11-05-codex-auth-save-default-store`。本检查只读定位有关差异，未修改、回滚或提交这些文件。

## 已撤销的隔离尝试

用户新指令到达前已创建 `outputs/desktop/gpui09-exit-20261105/isolated` detached worktree（基线同上），复制当时六个切片文件；`isolated-inputs.json` 保留原 SHA-256 清单。只运行 `desktop-fmt-check`，退出 0；未运行隔离 build/lint/test，也没有运行中的隔离进程。用户新指令后停止该方案，主会话核对路径和白名单后移除了该 worktree（退出 0）；输入清单与格式日志保留。这不是任何隔离产品验收通过的证据。

## 实际验证

日志目录：`outputs/desktop/gpui09-exit-20261105/`。全部根目录执行，保持默认并行和 `--skip export_bindings`；构建使用既有缓存访问授权，未改缓存配置。

| 命令 | 退出码 / 结果 |
| --- | --- |
| `just desktop-fmt-check` | 0；`check-desktop-fmt.log` |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --test runtime_shutdown -- --skip export_bindings --nocapture` | 0；2 passed / 1 helper ignored；`check-runtime-shutdown.log`，helper 在父测试子进程中显式执行 |
| `just desktop-lint` | 0；all-target 严格 Clippy / type-check，`check-desktop-lint.log` |
| `just desktop-test` | 101；上述并行认证语义导致的既有断言失败，`check-desktop-test.log` |
| `task.py validate .trellis/research/archive/2026-11/10-17-gpui-09-environments` | 0；两个 manifest 各 8 个有效引用 |
| `git diff --check` | 0 |
| `just ci` | 本切片 not_run；未改共享产品模块，不冒用历史 CI |

### 用户新标准下的最终门禁

以下在原主工作区根目录执行，均为最新测试版本，无本次重试：

| 命令 | 退出码 / 结果 |
| --- | --- |
| `just desktop-fmt-check` | 0；`check-new-standard-fmt.log` |
| `python ccr-desktop/scripts/desktop.py cargo test --locked --lib runtime::platforms::tests::codex_ -- --skip export_bindings` | 0；2 passed，`check-new-standard-codex.log` |
| `just desktop-lint` | 0；all-target 严格 Clippy / type-check，`check-new-standard-lint.log` |
| `just desktop-test` | 0；Python 12、AccessKit 3、库 103、native_shell 8、native_ui 2、runtime_shutdown 2 全部通过，helper 默认 ignored 但由进程树测试显式启动；`check-new-standard-test.log` |

完整 desktop 门禁已按用户选择的新标准恢复通过，历史失败仍保留。没有本范围未修复问题；9.1–9.6、01 和父任务仍保持未完成，不能把合成 OS 进程测试转记为原生窗口/watcher/其他 OS 通过。无提交或归档。
