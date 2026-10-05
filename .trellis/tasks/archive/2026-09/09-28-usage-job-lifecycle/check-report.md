# T06 独立检查报告

日期：2026-09-28。检查角色：trellis-check。工作区：`D:/Documents/Code/Github/ccr`。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。

## 结论

独立检查确认 usage admission、取消请求、typed completion、终态保护和 bounded stream executor 的生产调用链。检查发现并修复了共享 ManagedProcess 的后代清理缺陷。修复后的 Windows 进程门禁 17 项和 Linux 进程回归 9 项通过。最终 usage 过滤器中 T06 原有 68 项通过，新增 T11 控制策略测试 1 项失败。macOS 和父任务完整门禁没有在本检查中执行。正式前端 lint 仍因两个既有用户临时文件的 5 条 no-console 失败，不能将定向通过结果表述为完整门禁通过。

usage registry、AppState API、DTO 和前端消费接口没有在本检查中改写，可以作为 T11 的已审阅本地接口。T11 仍须自行验证控制命令在前台 import 占用时能及时到达 owner，不能从本报告推定 runtime policy 已通过。

## Findings (fixed)

### F1：直接子进程退出后，Unix 后代可以跳过强制终止

- 文件：`crates/ccr-core/src/core/process_gateway.rs:110`、`:120`、`:139`、`:145`、`:164`。
- 原问题：原 terminate_tree 只在直接子进程 wait 超时后发送 SIGKILL。直接子进程提前退出会设置 reaped 并返回；后代即使忽略 SIGTERM，Drop 也会因 reaped 为 true 跳过进程组清理。原 wait 还在返回错误时设置 reaped。该控制流不能证明整个 owned tree 已退出。
- 修复：分别记录直接子进程 reap 和 tree_cleaned。只在成功 wait 后记录 reap；graceful wait 使用一半清理预算，随后无论父进程是否提前退出都强制终止 owned tree。直接子进程回收与进程组退出确认共用剩余 deadline。Unix 以进程组不存在确认清理；Windows 以 Job Object 的 ActiveProcesses 为零确认清理。
- 正常 wait 也会清理直接子进程退出后仍存活的 owned descendants，并将清理确认限制为 5 秒。清理确认失败保留错误和未确认标记。Drop 对未确认 tree 执行强制终止，不受 reaped 标记阻止。
- Tauri `process/gateway.rs:635` 直接使用 core 提供的有界 terminate_tree，移除重复的预算缩减；只有成功的完整 wait/terminate 才注销 owned record。
- 新增 Unix 真实 fixture：父进程响应 TERM 退出、后代忽略 TERM；父进程正常退出后调用 wait；已 reap 父进程的 Drop 清理。新增 Windows fixture：父进程已 reap 后，normal wait 和 Drop 均清理后代。新增通用 fixture：live tree 的确认超时不能标记成功。
- 原缺陷先由源码确认；没有在旧 core 二进制上运行本次新增的 5 个测试。因此本报告不声称完成旧 core 二进制的红绿对照。实现者先前记录的两个 usage 终态反例仍见 baseline-tests.log。

### F2：修复引入的 Windows Send 编译错误

- 文件：`crates/ccr-core/src/core/process_gateway.rs:145`。
- 问题：首版确认循环使用嵌套 async 块，捕获 Windows raw Job Object 指针的共享引用，使调用方 future 要求 Sync，Tauri 跨 crate clippy 报 E0277。
- 修复：使用持有 `&mut self` 的直接 async 循环，逐次检查 deadline。没有添加 unsafe Sync 或放宽类型约束。core 全 target、全 feature clippy 和 Tauri 严格 clippy 复验通过。原日志为 check-tauri-clippy.log，最终日志为 check-tauri-clippy-final.log。

## AC 证据

| 验收 | 实际检查与区分缺陷的证据 | 结果范围 |
| --- | --- | --- |
| AC1 / R1 | `usage_jobs.rs:236,265,378`：一个 registry 锁内保存 snapshot/token；barrier 阻止 runner 运行，先 cancel 后释放 barrier，spawn 计数为 0；取消请求不释放 active；重复 admission 返回同 job。barrier fixture 调用生产 run_sync_stream 复用的 spawn_sync_process；`commands/usage.rs:431,449,1500,1508` 的实际 runner token 传递另经源码核对。 | 本地通过；fixture 没有构造完整 Tauri App，不将该测试表述为原生 UI 点击验收。 |
| AC2 / R2 | `usage_jobs.rs:278,291,420`：五种终态拒绝重复 cancel、晚到 progress、重复 complete；cleanup_failed 保留诊断。`commands/usage.rs:489,666,1537` 只有 runner 完成后提交终态，cancel handler 只发出 request。前端将 cancel_requested 作为 loading，将 timed_out/cleanup_failed 作为失败，订阅后查询弥补即时完成事件丢失。 | registry 行为、源码调用链和 18 项前端定向测试通过。 |
| AC3 / R3 | `cli.rs:186,254,267`：同一 execution deadline 覆盖 stdout、callback、wait 和 stderr；错误返回经过 bounded tree cleanup / stderr join，reader 超时 abort 后返回 cleanup_failed。missing-stderr 提前返回也执行有界 tree cleanup。新增 core 确认机制不把 direct reap 当成 tree completion。 | Windows Tauri gateway 10 + core 7 通过；Linux core 9 通过；macOS 未执行。 |
| AC4 / R4 | `cli.rs:20,359`：stdout 在 1 MiB + 1 byte 处立即截断并报错，不等待 newline/EOF；stderr 64 行、每行 64 KiB，独立并发 drain。record 中的 token 没有 Serialize/TS；两个 generated enum 只扩展状态。2 项 no-crate guards 通过；Tauri 的 usage_bucket_30m 搜索只命中既有热力图注释。 | 本地通过；没有新增 usage SQL、上游 Rust crate 依赖、renderer executable/env/secret 字段。 |

## 检查命令与结果

Tauri 构建命令仅设置命令局部 `CCR_SKIP_ICON_GENERATION=1`。行为测试跳过 export_bindings；没有改全局测试并行策略。本检查没有修改 DTO，因此没有重跑会写共享 generated 目录的导出任务。实现者的 3 项 ts-rs 导出和 normalizer 记录保持有效，完整 drift guard 由 root/T10 负责。

| 命令 | 实际结果 | 日志 |
| --- | --- | --- |
| `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml usage -- --skip export_bindings --nocapture` | 首轮 68 单元测试 + 2 no-crate guards 通过；core 修复后的最终复验 68 通过、1 个新增 T11 测试失败，integration guards 因 bin 失败未再次执行 | check-usage-tests.log；check-usage-tests-final.log |
| `just tauri-process-smoke` | 10 项 Tauri gateway + 7 项 Windows core，通过。附带 integration binary 的过滤结果为 0 项，没有计入通过数量。 | check-process-smoke-final.log |
| Linux `cargo test --offline --locked --target-dir /tmp/ccr-architecture-linux-01a0e781 -p ccr-core process_gateway -- --nocapture` | 最终 9 项通过 | check-core-process-linux-final.log |
| `cargo clippy -p ccr-core --all-targets --all-features -- -D warnings` | 通过 | check-core-clippy-final.log |
| `cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings` | 修复 Send 问题后通过 | check-tauri-clippy-final.log |
| `cd ccr-ui; bun run type-check` | 通过 | check-type-check.log |
| `cd ccr-ui; bun run test:smoke -- tests/usage/usage-import-lifecycle.smoke.test.tsx tests/usage/usage-import-normalization.smoke.test.ts` | 2 文件、18 项通过 | check-frontend-lifecycle.log |
| `cd ccr-ui; bun run lint` | 失败：5 errors、2 warnings；lint:style 未执行 | check-lint.log |
| `rustfmt --edition 2024 --check crates/ccr-core/src/core/process_gateway.rs ccr-ui/src-tauri/src/process/gateway.rs` | 通过 | check-review-fmt.log |
| `git diff --check --` 本检查 4 个代码/规范文件 | 通过；仅 Git CRLF 转换提示 | check-review-diff.log |

### Linux 环境与锁定依赖

使用 WSL Ubuntu-24.04，实际已安装的 `/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo`、rustc 和 rustdoc。命令局部 `RUSTUP_TOOLCHAIN=stable`，RUSTC/RUSTDOC 设置为绝对路径，PATH 仅在子进程前置该 bin；清空子进程的 wrapper 变量。没有调用 rustup shim、安装工具链或修改全局配置。Linux 为 Rust 1.95，Windows 为 `rustc 1.98.0 (88d9e12ae 2026-08-18)`。Linux 结果不能替代仓库 pinned 1.98 的完整平台门禁。

最初离线构建因缓存只有 indexmap 2.14.0，而锁定依赖要求 2.14.1 失败。root 随后授权按现有 Cargo.lock 下载构建依赖；一次 `--locked` 构建的 8 项通过后，新增通用超时 fixture 的最终 `--offline --locked` 9 项复验通过。没有改依赖版本。Cargo.lock 前后 SHA256 均为：

`CA1F03E85B5E0D888109AC8AF7E7943FC1CEECD20A0F6EEB6A60052C5155DB79`。

初始日志路径误指向不存在的 research 子目录，3 条检查命令当时没有启动。随后改用任务目录中的 check-*.log 并实际执行。该工具准备错误没有计作测试结果。

## Findings (not fixed) 与未验证边界

- 最终 usage 过滤器新增命中 `commands::runtime_policy::tests::control_matrix_usage_delivery`。该测试在 `import_usage_v2` 持有 execution permit 时等待 `cancel_usage_import_job_v2` 的 owner acknowledgement，实际 `Err(Elapsed(()))`，期望 `Ok(Ok("cancel_usage_import_job_v2"))`。该控制策略缺陷属于并行实施中的 T11，已将准确日志交 root；本检查没有修改 runtime_policy，也没有用 skip 排除该红测试来宣称完整 usage 门禁通过。
- 正式 lint 的 5 条 no-console 位于 `ccr-ui/.tmp-desktop-probe.mjs:20,24,92` 和 `ccr-ui/.tmp-insights-visual.mjs:236,381`。root 明确要求保留用户文件；本检查没有删改、忽略或放宽规则。另有既有 prefer-const 和 unused-vars warnings。
- 实现者原始全量 tests/usage 结果为 82/83；唯一失败是 dateWindow 的 local-midnight 毫秒差跨 DST 少一天。root 已归属 T10，本检查未改该 owner、未降低 100 天期望、未用定向测试替代该正式失败。该条描述原始日志，不推定 T10 后续修改后的状态。
- macOS 进程组/Drop 平台测试未运行。真实用户 llmusage 同步、原生 UI 交互、浏览器视觉验收、全仓 just ci、全量 generated drift 和完整根/Tauri 测试门禁由 root/T10 接续。
- cleanup_failed 表示有界清理尝试结束但缺少完整清理证明。该状态保留诊断后释放 usage admission；释放 admission 不能证明可能残留的外部进程已退出，也没有实现额外的持久恢复/reaper。Unix 进程组仍存在时不会伪造成功，即使没有可确认的活动成员。
- 除上述已知门禁和平台边界，本检查没有发现需要保留的 T06 代码缺陷。没有运行零命中过滤器来宣称验收，没有把 fake executor 用例当成真实 OS 进程树证据。

## T11 接口交接

- start 使用 `AppState::admit_usage_import_job(snapshot)`。只有返回 Some(token) 的调用启动 runner；None 返回现有 snapshot。
- cancel 使用 `AppState::request_usage_import_cancel(job_id)`。cancel_requested 非终态；runtime policy 不能删除 active、提前写 Cancelled 或把 cleanup_failed 降级为成功。
- get/get_active 只读取当前 registry 快照。T11 负责让控制命令在另一个前台操作占用时仍能到达这些 owner，并验证同资源双 start 仍受 admission 约束。
- complete 只由 runner 在 executor 的进程/reader 清理返回后调用，保持 Finished/Failed/Cancelled/TimedOut/CleanupFailed 分类和终态不可覆盖。
- 本检查未修改 handler_registry、manifest、OAuth 或全局 concurrency 默认值；不会通过全部改 Parallel 绕过控制策略。

规范已同步 `managed-process.md` 和 `llmusage-provider-adapter.md`。没有修改 task.json、启动其他任务、提交或归档，也没有回退其他 owner 的改动。
