# T11 实施报告：桌面控制投递与 OAuth 生命周期

日期：2026-09-28。工作区：D:/Documents/Code/Github/ccr。基线 commit：34d8a85e0e48b793733835e0304c8ed33940fcee。

## 实施结果

已实现 registry 的 23 个生命周期命令分类、7 个控制入口的独立投递、command/install 后台 permit 转交、Usage 同步与后台导入的共同 admission，以及统一 OAuth controller。现有命令 ID、DTO、事件名称与权限边界保留。定向行为验证通过；正式 bindings guard 未通过，AC4 保留未验收。独立 Trellis 检查、生成问题、全量门禁和父任务验收由 root/T10 接续。

修复前先执行 3 个共享 permit 反例，结果为 0 通过、3 失败：command cancel、usage cancel、OAuth completion 投递在执行 barrier 释放前无法到达。修复后同名反例通过。其余真实 owner、故障注入和网络 fixture 是新增回归证据；没有声称这些用例全部在旧实现上执行。

## 责任和实现

- Registry 在 `handler_registry.rs:60` 按真实 ID 定义 resource/operation。risk、authorization、confirmation、audit 保持原规则。仅 7 个 control 改为 Parallel；其他 foreground/start 的原互斥资源不变。
- `runtime_policy.rs:44` 的 task-local admission 只允许所属 start 取走一次原 permit。command 与 install 继续使用同一个 Singleton semaphore。handler 错误自动释放未转交 permit；handler 返回或 abort 不释放已经交给后台 owner 的 permit。
- `command_exec.rs:1856` 的 `launch_command_job` 先登记 snapshot/token，再启动 runner。runner 完成后释放 permit。status/cancel 通过真实 registry 读快照及请求取消；错误 ID 不影响活动 job。
- `install_service.rs:282` 的 `execute_with_admission` 将 slot 和 permit 交给 executor join owner。事件 adapter 不再收到 terminal event 就清空 slot。执行器返回 join handle，清理完成后才允许新 attempt；原 `execute` 接口仍可供 CLI 使用。
- `usage.rs:712` 的 `run_admitted_usage_sync` 让单源/全源同步入口使用 T06 的 `AppState::admit_usage_import_job`。已存在 active 时返回确定错误，后台 start 继续幂等返回现有 ID。注册 token 传入 `run_sync_collect`，runner 在 T06 executor 完成清理后提交唯一终态。
- `codex_auth/oauth.rs:103` 的 `start_admitted` 先使用 start 专属短序列锁检查幂等复用，再为新登录取得原 codex_auth module permit。控制入口不使用此 start 锁。登录 owner 持有 permit 至 commit 与 cleanup 完成。
- OAuth start 持有真实 listener，完成 nonblocking/runtime 转换与 T05 secret pending save 后才发布 ID/URL。恢复 pending 使用相同 controller，并验证期限、callback origin、state/code。磁盘路径仍使用 `PlatformPaths/CCR_ROOT` 的既有位置。
- OAuth owner 统一持有 listener、accepted socket、cancel token、completion channel、monotonic deadline 和 pending cleanup。accept、socket read/write、token request 与完整 body 使用同一 deadline；取消后 join listener，再清 pending，再发布内部终态/完成 reply。
- HTTP token 处理拒绝重定向，限制 body 为 1 MiB，并使用固定错误信息；服务响应内容、verifier 和 token 不进入返回错误。callback event 只包含 loginId，timeout event 保留无 secret 的 callbackUrl。
- Account commit 一旦开始，controller 返回 `oauth_commit_in_progress` 拒绝取消。`spawn_blocking` secret writer 保持 completion-aware，permit 持至 commit 和 cleanup 完成。该行为有 commit barrier 测试；没有报告虚假的取消成功。
- Pending cleanup 失败保留 `CleanupFailed`，阻止新 login；取消入口可重试 pending cleanup。端口 release 先取消 controller 自己的 listener，再沿用 ProcessGateway 的所有权识别和报告，不扩大外部进程权限。

## 修改文件

| 文件 | T11 变更 |
| --- | --- |
| ccr-ui/src-tauri/src/commands/handler_registry.rs | 显式生命周期矩阵、控制并发规则、23 ID 安全边界测试 |
| ccr-ui/src-tauri/src/commands/runtime_policy.rs | 单次 permit 转交、OAuth 幂等前置 admission、11 项运行策略测试 |
| ccr-ui/src-tauri/src/commands/command_exec.rs | 后台 job owner 持 permit、真实 registry 控制和清理 barrier 测试 |
| crates/ccr-cli/src/services/install_exec.rs | 返回 executor join handle |
| crates/ccr-cli/src/services/install_service.rs | attempt owner 持 slot/permit、测试用 runner seam |
| ccr-ui/src-tauri/src/commands/install.rs | 将 permit 转给 InstallService，移除 adapter 提前清 slot |
| ccr-ui/src-tauri/src/commands/usage.rs | 在 T06 代码上增加同步入口的共同 admission/token/terminal 路径 |
| ccr-ui/src-tauri/src/llmusage_adapter/cli.rs | collect 接收已注册 CancellationToken；保留 T06 executor |
| ccr-ui/src-tauri/src/commands/codex_auth.rs | controller 适配、T05 store 路径、HTTP body 约束与脱敏 |
| ccr-ui/src-tauri/src/commands/codex_auth/oauth.rs | 新增统一 lifecycle controller |
| ccr-ui/src-tauri/src/commands/codex_auth/oauth_tests.rs | 新增 listener/storage/HTTP/admission/commit 故障注入测试 |
| ccr-ui/src/api/generated/command-manifest.json | registry 生成的 7 control concurrency 和 OAuth start timeout owner 变化 |
| ccr-ui/src/api/generated/commandCapabilities.ts | 同步生成 capability policy |
| .trellis/spec/ccr/backend/tauri-handler-registry.md | 控制投递与 admission 生命周期契约 |
| .trellis/spec/ccr-codex/backend/backend-guidelines.md | OAuth owner、deadline、commit、secret cleanup 契约 |

T05/T06 与 T02 的共享文件既有改动保留。未修改 ProcessGateway、Cargo.lock、任务状态、用户临时脚本或 Insights 任务。完整生成窗口已保留 T02 profiles 三个类型、更新 Grok Applied 的 ProfileOutcome，并保留 T06 usage 两个枚举；这些 DTO 结构变化分别归对应任务所有。

## AC 追溯

| AC | 证据 | 状态 |
| --- | --- | --- |
| AC1 / R1 | C01–C06 下表；runtime policy、command owner、InstallService、UsageImportJobs、OAuth controller 的 barrier/ack 测试；23 ID manifest 基线逐字段比较 | 本机定向行为通过；独立检查待 root |
| AC2 / R2 | bind race/save failure 不发布 slot；start 返回前端口已独占；恢复 callback state/origin 验证；重复 completion 不二次 commit | 通过本机合成 fixture |
| AC3 / R3 | silent accepted socket 超时；慢 HTTP body 取消与 server EOF；slow body + silent socket 共用 400 ms deadline；cleanup barrier 持 admission；commit cancellation 明确拒绝 | 通过本机 loopback/owner fixture |
| AC4 / R4 | 23 ID 安全/协议字段比较、ACL/confirmation、API smoke、sentinel；inventory/bindings guard | inventory/API/security 通过；正式 bindings guard 失败，AC4 未完成验收 |

## C01–C06 与 23 ID

下表的运行策略测试均在 barrier 释放前验证结果。P1 = `control_matrix_command_and_install_delivery`；P2 = `control_matrix_usage_delivery`；P3 = `control_matrix_oauth_delivery`。这三项用注入 handler body 验证逐 ID 投递。另列的 owner 测试调用真实 registry/service/controller，证明取消到达、错误 ID 无影响和 admission 清理顺序。没有将注入 body 测试当成 Webview IPC 或真实安装验收。

| 真实命令 ID | C 编号 | 运行路径与 owner 证据 |
| --- | --- | --- |
| execute_ccr_command | C01/C05 | P1 foreground 持共享 gate；existing command owner 在其 barrier 下 status/cancel 可达 |
| start_ccr_command_job | C01/C05 | single-use transfer；command_owner_receives_cancel_before_cleanup_and_keeps_start_admission |
| get_ccr_command_job_status | C01/C06 | P1；foreground_barrier_does_not_block_existing_command_owner_controls 读真实 snapshot |
| cancel_ccr_command_job | C01/C06 | P1；真实 token ack、错误 ID、cleanup barrier |
| llmusage_install_detect | C02/C06 | P1 逐 ID 投递；CLI detect 回归 |
| llmusage_install_probe_capabilities | C02/C06 | P1 逐 ID 投递；CLI capabilities 回归 |
| llmusage_install_plan | C02/C06 | P1 foreground 持共享 gate；foreground_install_plan_does_not_block_existing_attempt_owner |
| llmusage_install_execute | C02/C05 | install_owner_cancel_and_recent_reach_attempt_before_cleanup；terminal-looking event 不释放 slot；第二 start 不并行 |
| llmusage_install_cancel | C02/C06 | P1；真实 InstallService token ack；错误 attempt ID 无影响 |
| llmusage_install_recent | C02/C06 | P1；真实 ring snapshot 在清理 barrier 期间可读 |
| llmusage_install_manual_catalog | C02/C06 | P1 逐 ID 投递；CLI catalog 回归 |
| llmusage_install_check | C02/C06 | P1 逐 ID 投递；已有检测/能力组合保持 |
| import_usage_v2 | C03/C05 | P2；usage_owner_controls_reach_existing_job_during_each_foreground_import 单源分支 |
| import_all_usage_v2 | C03/C05 | P2；同一真实 UsageImportJobs owner 测试全源分支 |
| start_usage_import_job_v2 | C03/C05 | T06 原子 admission/token；T11 owner 测试现有 ID 复用、清理期间 active 保留 |
| get_usage_import_job_status_v2 | C03/C06 | P2；真实 owner snapshot 在同步导入 barrier 下可读 |
| cancel_usage_import_job_v2 | C03/C06 | P2；真实 token ack、错误 ID、cleanup failure typed terminal |
| codex_oauth_login_start | C04/C05 | oauth_start_reuses_live_id_before_waiting_for_same_module_admission；真 listener；幂等同 ID；新登录与同 module mutation 互斥 |
| codex_oauth_login_completed | C04/C06 | P3 在 codex_save_auth 长操作下可投递；真实 controller 串行 exchange/commit、重复完成拒绝 |
| codex_oauth_login_cancel | C04/C06 | P3；cancel_exchange_keeps_admission_until_cleanup_finishes；慢 body 取消；commit 阶段明确拒绝 |
| codex_oauth_submit_callback_url | C04/C06 | P3；exchange barrier 下通过实际 command runtime 到 controller，重复/state/origin/旧 ID 拒绝 |
| codex_is_oauth_port_in_use | C04/C06 | P3；exchange barrier 下真实已绑定 loopback listener 的探测保持可达 |
| codex_release_oauth_port | C04/C06 | P3；按此 ID 执行 controller cancel 到达并等待 cleanup；既有 port report/ProcessGateway 所有权测试另行回归 |

install 独立 get/status：N/A，协议没有该入口。OAuth 独立 login get/status：N/A，协议没有该入口。

C05 同时覆盖原 global Singleton 的跨 command/install 互斥：一个后台 owner 持 permit 时，command 与 install 的所有执行入口仍排队，控制入口可达。Usage 保持 T06 active slot 至执行器返回；OAuth active start 在等待长期 module permit 前幂等复用。取消请求和 terminal-looking event 均不会提前允许第二作业。

C06 的 `lifecycle_matrix_preserves_each_commands_security_and_execution_resource` 遍历全部 23 ID，检查 ACL、confirmation、authorization、audit 和 resource。`manifest-baseline-comparison.json` 独立比较 HEAD 与工作区 generated manifest，确认 security/wire 字段全部不变。auxiliary-window/opaque capability 的既有测试保留。

## 实际检查

Rust 命令使用命令局部 `CCR_SKIP_ICON_GENERATION=1`。行为测试跳过 `export_bindings`；绑定生成在单一窗口执行。测试路径实际为 `commands::codex::auth::oauth`；早期 `codex_auth` 过滤器只命中 generated-client 测试，该结果不计 OAuth 行为证据。

| 命令/检查 | 有效结果 | 日志 |
| --- | --- | --- |
| 修复前 runtime policy 三项 control matrix | 0 通过、3 失败 | baseline-controls.log |
| cargo ... test runtime_policy -- --skip export_bindings --nocapture | 11 通过 | final-runtime_policy.log |
| cargo ... test oauth -- --skip export_bindings --nocapture | 18 通过 | final-oauth.log |
| cargo ... test command_exec -- --skip export_bindings --nocapture | 24 通过 | final-command_exec.log |
| cargo ... test usage -- --skip export_bindings --nocapture | 70 单元 + 2 no-crate guard 通过 | final-usage.log |
| cargo ... test handler_registry -- --skip export_bindings --nocapture | 21 通过 | final-handler_registry.log |
| cargo test --offline --locked -p ccr-cli --lib services::install_ | 60 通过 | cli-install-tests.log |
| cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --bin ccr-desktop -- -D warnings | 通过；MSVC linker_messages warning 不受 -D warnings 控制 | final-clippy.log |
| just tauri-command-inventory | 1 通过，生成 manifest/capabilities | inventory-generation.log |
| just tauri-command-inventory-check | 1 通过 | inventory-check.log |
| just tauri-bindings（独占窗口串行修复） | CLI 24、ccr-usage 9、Tauri 194 个 export 测试均通过，生成命令退出 0 | bindings-repair.log |
| just tauri-bindings-check（修复后） | 失败：5 文件与检查前基线不同；export 测试均通过；最终 5 文件与 HEAD 无 diff，不能将 guard 记为通过 | bindings-check.log |
| cargo fmt --manifest-path ccr-ui/src-tauri/Cargo.toml -- --check | T02 格式化其 Grok handler 后复验通过 | fmt-check.log |
| cargo ... test port_release_report -- --skip export_bindings --nocapture | 2 通过；未知 PID 不认领、无 owned process 时保持 no-op | final-port_release_report.log |
| cargo ... test process::gateway -- --skip export_bindings --nocapture | 10 通过，含 URL allowlist、owned PID、timeout/reap、stdin/output 上限 | final-process-gateway.log |
| cd ccr-ui; bun run type-check | 通过 | frontend-type-check.log |
| cd ccr-ui; bun run test:smoke -- tests/api/api-facade-boundary.smoke.test.ts tests/api/typed-json-boundary.smoke.test.ts tests/api/typed-command-boundary.smoke.test.ts | 3 文件、13 测试通过 | frontend-api-smoke.log |
| git diff --check | 通过，仅 Git 行尾转换提示 | diff-check.log |

测试计数存在过滤器间重叠，不将各行相加声称唯一用例总数。

## 中间失败与未验证范围

- 严格 clippy 首轮遇到 T04 正在移动接口的 `validate_report_command` 未解析和 Platform 短名借用错误。所属代理完成修复后复验通过；原始日志保留为 `transient-cli-clippy.log`。T11 未改该代码。
- CLI install 首轮编译被 T02 `profile_contract.rs:163` 未定义 instance 阻断。所属代理修复后，offline locked 重跑 60 项通过；原始日志为 `transient-profile-test-compile.log`。
- 首轮 bindings guard 检出 T02 已知的 Grok Applied outcome 漂移，已按 Rust 定义重新生成。第二轮出现 11 个生成文件的尾部残留，原因未查明；失败日志与 diff 分别保存为 `bindings-corrupt-output.log` 和 `bindings-corrupt-output.diff`。root 确认独占生成窗口后，本代理执行串行修复与 guard 复验。修复命令成功，正式 guard 仍报告 HeatmapCell.ts、InsightDto.ts、HomeInsightsResponse.ts、InsightsTally.ts、InsightsTotals.ts 共 5 文件与检查前基线不同。最终这 5 文件均与 HEAD 无 diff，尾部残留已不存在；生成目录最终 tracked diff 仅 Grok outcome 与 T06 两个 usage 枚举，另保留 profiles 三个新增类型。正式 guard 失败状态保留，不以生成命令成功替代。
- 生成期间观察到另一条 `application_ --test-threads=1 --skip export_bindings` runner；未停止该进程，没有证据将生成问题归因于该 runner。原因为未查明，交 root 的独立 checker/T10 分析。
- 原生 Windows 上已执行本机 loopback listener/socket、慢 HTTP body、端口重绑定、T05 pending held-file cleanup denial，以及既有 ProcessGateway 的受控进程 timeout/reap/stdin/output 测试。没有进行真实 OAuth 服务/真实账户、真实软件安装、用户配置写入或 native Webview 点击验收。
- Linux/macOS listener、文件权限和 native desktop 行为未由 T11 执行。T06 的原生进程 cleanup 证据由其 implementation-report.md 提供，T11 未更改 process gateway。
- 完整 `just tauri-ci`、`just ci`、全量 frontend/native UI 与独立检查由 root/T10 接续。本报告不把定向结果当成完整门禁。
- Account commit 和本地秘密持久化维持 completion-aware。网络 deadline 覆盖 accept/read/request/body；已经开始的不可安全取消 secret commit 由 owner 等待完成，不承诺强制中断阻塞文件系统操作。

## T07/T10 交接

- command DTO、status/event 字段不变。cancel 表示请求已送达，最终完成依赖 runner 的实际清理。后台 start 返回不释放执行资源。
- install 的 event adapter 只转发事件；attempt slot 由 executor join owner 释放。terminal event 与可接纳新 attempt 的时间不可直接等同。
- Usage 保留 T06 的 cancel_requested 非终态、唯一 terminal 和 cleanup_failed。同步导入也登记 active/job token，后台 start 可按现有协议复用正在执行的 ID。
- OAuth `codex-oauth-login-completed` 仍表示 callback 已接收。账户提交结果以 login_completed handler 返回为准；取消和 timeout 不构造成功 DTO。commit 期间取消的 `oauth_commit_in_progress` 应作为确定错误显示。
- 本代理的生成/检查进程已全部结束，生成窗口于 2026-09-28 08:22 -05:00 释放。没有再次运行导出。T02 可对当前 Grok outcome 类型运行最终 frontend type-check。
- 未执行 `task.py start`、修改 task.json、提交或归档。生产文件冻结，报告与失败证据交 root 独立 checker。command 相关契约可做 scoped 验收；完整 AC4/生成问题和全量集成交 T10，不宣告父任务或 T11 全部验收完成。
