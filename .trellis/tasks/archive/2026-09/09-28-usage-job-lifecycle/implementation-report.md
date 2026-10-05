# T06 实施报告：Usage 后台任务取消与进程清理

日期：2026-09-28。工作区：D:/Documents/Code/Github/ccr。基线 commit：34d8a85e0e48b793733835e0304c8ed33940fcee。

## 实施结论

已实现四项 AC 对应的本机行为：原子发布 snapshot/token；cancel_requested 非终态；runner 在进程退出与有界清理后提交唯一终态；执行 deadline 覆盖 stdout、回调、wait 与 stderr；cleanup_failed 保留真实清理错误。Windows 原生进程测试通过。独立 Trellis 检查和父任务完整门禁仍由 root/T10 接续，本报告不宣告整个父任务验收完成。

旧实现的两项直接反例先执行后修复：Cancelled 被晚到失败覆盖，Failed 被 cancel/进度覆盖为 Running。baseline-tests.log 为 0 通过、2 失败；新实现同名测试通过。其余 barrier、executor 故障注入与 React 用例为本次新增回归证据，未宣称全部都曾在旧二进制上执行。

## 责任和边界

- 根因一：旧 AppState 分别维护 snapshot、token 和 active，token 在延迟 runner 内登记；取消命令直接写 Cancelled。
- 根因二：旧 stream 未执行 descriptor deadline；Finished/Cancelled NDJSON 回调在进程退出前改写 job；取消/解析失败先于 cleanup error 返回。
- 修复 owner：usage_jobs.rs 的 UsageImportJobs，以及 llmusage_adapter/cli.rs 的 execute_sync_process。Tauri handler 只负责入参、启动、事件与监控适配。
- 保留外部 llmusage CLI、ccr-usage SQL owner、原有进度/统计投影、单行限制和 stderr 保留限制。未改 parser 协议、配置、权限、registry manifest、其他任务状态或用户临时文件。

## 修改文件

| 文件 | 变更 |
| --- | --- |
| ccr-ui/src-tauri/src/usage_jobs.rs | registry record 合并 snapshot/token；原子 admission；取消请求与单次终态；5 项行为测试 |
| ccr-ui/src-tauri/src/state.rs | usage 专属三个锁替换为一个 registry 锁；提供 admission/cancel/progress/complete API |
| ccr-ui/src-tauri/src/commands/usage.rs | start 传入已登记 token；cancel 只请求取消；runner 按 typed result 提交终态；保留 summary/source 计数 |
| ccr-ui/src-tauri/src/llmusage_adapter/cli.rs | 全过程 deadline、有界 cleanup、deferred terminal summary、即时 stdout 单行上限、可注入进程 seam |
| ccr-ui/src-tauri/src/llmusage_adapter/cli_lifecycle_tests.rs | 15 项 executor/上限/原生回归测试 |
| ccr-ui/src-tauri/src/llmusage_adapter/error.rs | Cancelled、TimedOut、CleanupFailed 具名错误 |
| ccr-ui/src-tauri/src/process/gateway.rs | terminate/reap 整体受 grace 限制；成功 wait/terminate 后注销进程 |
| ccr-ui/src-tauri/src/process/mod.rs | 导出已有 ManagedChild，供 executor 使用同一进程 owner |
| ccr-ui/src/features/usage/useUsageImport.ts | 完整状态消费、终态防回退、订阅后查询恢复即时完成 |
| ccr-ui/src/utils/usageImportNormalization.ts | 状态判定与快照接收规则、summary 投影 |
| ccr-ui/src/types/generated/usage/UsageImportJobStatus.ts | ts-rs 新增 cancel_requested、timed_out、cleanup_failed |
| ccr-ui/src/types/generated/usage/UsageImportJobStage.ts | ts-rs 同步阶段枚举 |
| ccr-ui/tests/usage/usage-import-lifecycle.smoke.test.tsx | 13 项状态与 React 行为测试 |
| .trellis/spec/ccr/backend/llmusage-provider-adapter.md | 同步 usage lifecycle/stream 执行契约 |

UsageImportResultV2.ts 曾因 ts-rs 依赖导出出现行尾空格。使用仓库 normalize-generated-bindings.mjs 修复；该文件最终无 diff。没有手工修改生成定义。

## 逐 AC 证据

| AC | 机制与证据 | 本机结果 |
| --- | --- | --- |
| AC1 | UsageImportJobs::admit/request_cancel（usage_jobs.rs:236、265）；cancel_before_runner_start_prevents_spawn_and_keeps_admission（:378）用 Tokio barrier 阻挡 runner，取消后 active 仍存在，重复 admission 返回同 ID，释放 barrier 后实际生产 spawn guard 的 fake spawner 计数为 0，complete 后才接纳新 job | 通过 |
| AC2 | UsageImportJobs::complete（:291）；terminal matrix 覆盖 finished/failed/cancelled/timed_out/cleanup_failed 的重复 cancel、晚到 progress、重复 complete；running cancel 注入 cleanup error 并检查 diagnostic；React 防止终态后旧 progress 恢复 loading | 通过 |
| AC3 | execute_sync_process（cli.rs:186）同一 deadline 覆盖消费、回调、wait、stderr。fake deadline 30 ms + cleanup 60 ms + 250 ms 调度容差，外层 1 s 仅作为测试挂起保护；覆盖 silent、EOF 活进程、Finished 后仍活、stderr holder、blocked callback、cleanup Err、hung reap、missing stderr。Windows silent timeout 和 running cancel 使用隔离临时目录中的受控 PowerShell；core managed tree 测试确认 Windows 后代退出 | 通过；Linux/macOS 未运行 |
| AC4 | read_stdout_line（cli.rs:359）复用 bounded reader，并加 cap+1 的 Take；无限无换行输入在 1 MiB 处失败。stderr fixture 写 70 条超长行，断言仅保留 64 条、每条不超过 64 KiB。DTO 只增加枚举值，token 留在非 Serialize record。no-crate guard 2 项通过；Tauri usage_bucket_30m 搜索仅命中既有注释 | 通过 |

## 实际检查

Rust 命令均使用命令局部 CCR_SKIP_ICON_GENERATION=1。没有设置全局串行测试策略。export_bindings 单独串行运行；行为测试跳过 mutating export 测试。

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml usage_jobs::lifecycle_tests -- --nocapture（修复前） | 2 失败，反例成立 | baseline-tests.log |
| cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml usage_jobs -- --nocapture | 8 通过（5 行为、3 export） | lifecycle-tests.log |
| cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml llmusage_adapter -- --nocapture（中间验证） | 33 通过 | adapter-tests.log |
| cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml usage -- --skip export_bindings --nocapture | 68 单元测试 + 2 no-crate guards 通过 | usage-tests.log |
| cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml usage_jobs::export_bindings -- --test-threads=1 | 3 项定向生成测试通过 | bindings-tests.log |
| just tauri-process-smoke | Tauri gateway 10 + ccr-core process 5，共 15 项通过 | process-smoke.log |
| cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings | 通过；MSVC linker_messages warning 不受 -D warnings 控制 | clippy.log |
| cd ccr-ui; bun run test:smoke -- tests/usage/usage-import-lifecycle.smoke.test.tsx tests/usage/usage-import-normalization.smoke.test.ts | 2 文件、18 项通过 | frontend-lifecycle-tests.log |
| cd ccr-ui; bun run test:smoke -- tests/usage | 18 文件；82 通过、1 失败，见下文 | frontend-usage-tests.log |
| cd ccr-ui; bun run type-check | 通过 | type-check.log |
| cd ccr-ui; bun run lint | 正式路径失败：5 个既有 no-console、2 warnings；新增复杂度/深度问题已修复 | lint.log |
| cd ccr-ui; bun scripts/normalize-generated-bindings.mjs | 通过，两个 enum 为最终生成定义差异 | 工具回执 |
| cargo fmt --manifest-path ccr-ui/src-tauri/Cargo.toml -- --check | 通过 | fmt-check.log |
| git diff --check | 无空白错误；仅 Git 行尾转换提示 | diff-check.log |

## 失败和未验证边界

- 完整 usage 前端 suite 的唯一失败为 usage-date-window.smoke.test.ts:78：期望 all_time 100 天，实际 99 天。该 helper 和测试未修改。root 已用 TZ 对照确认本地午夜毫秒差跨 DST 的缺陷，并登记 T10 research/usage-calendar-day-preflight.json。本任务不改该 owner，也不降低期望。
- 正式 lint 的 5 个 no-console 位于用户 ccr-ui/.tmp-desktop-probe.mjs 和 .tmp-insights-visual.mjs。两个文件没有删除、修改或添加 lint 忽略。lint:style 因正式 ESLint 失败未执行。
- Linux/macOS process tree、完整 native UI、真实用户 llmusage 数据库和真实 CLI 同步未运行。没有读取/改写用户配置。
- 本任务不声称通过全仓 just ci、全量绑定 drift guard或独立 trellis-check。相关集成检查交父任务/T10；没有用定向测试替代失败门禁。
- 此次 T06 测试未观察到 0xc0000005。其他 owner 报告的进程异常和 ccr-core lock.rs Windows metadata 错误不在本任务内修复。

## T11 接口交接

- start owner API：AppState::admit_usage_import_job(snapshot)，仅返回 Some(token) 的调用负责启动 runner；None 表示已有 active job，必须返回该 snapshot，不再 spawn。
- cancel owner API：AppState::request_usage_import_cancel(job_id)。cancel_requested 是非终态；不得从 handler/runtime policy 强行删除 active 或预写 Cancelled。
- get owner API：get_usage_import_job/get_active_usage_import_job。对 control 操作的 policy override 必须让这些方法及时到达 registry；foreground import 占用 usage module permit 时也必须可取消后台 job。
- completion owner API：complete_usage_import_job(job_id, UsageImportCompletion)，仅 runner 在 executor 返回后调用。Finished 携带结果、summary 和 source_count；Error 保持 typed Cancelled/TimedOut/CleanupFailed。
- cleanup_failed 表示终止/回收/reader completion 不能被确认。错误可见，不能降级为取消成功。registry 在 runner 的清理尝试和有界 join 返回前保持 active；cleanup_failed 是 runner 的最终失败结果，保留诊断后释放 admission。
- 不改 handler_registry、command manifest 或 risk 默认映射；T11 负责 control/admission 的独立策略验收。

没有修改 task.json、执行 task.py start、提交或归档。检查器可从本报告和日志继续。
