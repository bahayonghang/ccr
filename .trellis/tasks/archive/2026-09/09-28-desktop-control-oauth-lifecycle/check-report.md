# T11 独立检查报告

日期：2026-09-28。检查角色：/root/check_t11。基线 commit：34d8a85e0e48b793733835e0304c8ed33940fcee。

## 结论

AC1、AC2、AC3 在本机 Windows 隔离 fixture 范围通过。检查发现并修复两项 OAuth owner 竞态，每项均保存失败反例及修复后结果。AC4 的权限、协议和 secret 证据通过；正式 bindings guard 未通过，交 T10 处理。不得将 T11 或父任务标为完整验收通过。

T07 所需 command 生命周期契约已独立通过，可作为 scoped 前置批准。T06 原先等待 T11 的共享 Usage 门禁已独立复验为 70 个单元测试及 2 个 no-crate guard 通过。上述结论不代表完整 just ci、真实 OAuth、安装或 native Webview 验收。

## Findings (fixed)

### F1：旧 OAuth 清理重试删除新登录 pending

- 文件：ccr-ui/src-tauri/src/commands/codex_auth/oauth.rs，Controller::cancel_login。
- 问题：失败登录的 done 保留 CleanupFailed。第二个取消请求可先捕获旧 Arc<Login>，再于首次重试清空 slot、后续 start 保存新登录之后恢复执行。旧代码在验证当前 slot 身份之前调用 backend.clear，删除新登录的 pending。
- 反例：oauth_tests.rs:695，stale_cleanup_retry_cannot_clear_a_replacement_login。修复前 0/1，保存的 second 登录变为 None。证据：check-oauth-stale-red.log。
- 修复：清理重试先在短临界区确认 slot 仍属于该 Arc<Login>，再通过原子 claim 确保只有一个重试 owner。清盘期间不持 slot 或 login-state mutex；旧 owner 返回 oauth_login_id_changed，重复重试返回 oauth_cleanup_in_progress。失败释放 retry claim，成功才清空原 slot。
- 验证：旧 owner 交错回归通过；oauth_tests.rs:652 的 cleanup barrier 证明另一个 cancel 被确定拒绝、控制锁保持可达、第二个 start 不发布新登录。两个用例包含在 check-oauth-green.log 的 21/21 中。

### F2：等待 admission 的恢复流程重建已取消登录

- 文件：ccr-ui/src-tauri/src/commands/codex_auth.rs:682；codex_auth/oauth.rs，Controller::restore_admitted。
- 问题：恢复流程先读取 pending，再等待 module admission。等待期间 cancel 可以清盘；旧 create closure 随后把捕获的旧 pending 重新保存，并再次发布该登录。
- 反例：oauth_tests.rs:617，restore_waiting_for_admission_does_not_resurrect_cancelled_pending。将既有行为提取到实际生产 restore helper 后，admission barrier 红测 0/1：取消后仍返回 Ok(Some(saved))。证据：check-oauth-restore-red.log。
- 修复：恢复入口取得 admission 后沿用 start 的权威存储重读；没有 pending 时返回 oauth_saved_login_missing，禁止以早前快照创建登录。正常 start 的创建和活动 ID 幂等复用保持。
- 验证：同一 helper 的 barrier 回归通过，取消后没有 pending、slot 或新 URL。check-oauth-green.log：21/21。更新 .trellis/spec/ccr-codex/backend/backend-guidelines.md，记录恢复与重试契约。

## Findings (not fixed)

### N1：正式 bindings guard 未通过

本次独占运行一次 just tauri-bindings-check。第一阶段 CLI export 测试进程 ccr_cli-957d4a190e1b8043.exe 在执行测试前以 0xc0000005 STATUS_ACCESS_VIOLATION 退出。原因未查明。没有证据将该异常归因于源码、mbx 或另一个 runner。日志：check-bindings.log；结果：check-bindings-result.json。

当前 recipe 先删除 generated 目录，再执行导出。失败后目录从 227 个文件变为 0。检查前已保存全部文件的原始字节和 SHA256；已仅恢复这些缺失文件，227/227 逐字节一致，未覆盖并发文件。证据：check-bindings-before.json、check-bindings-after.json、check-bindings.diff、check-bindings-restoration.json。恢复操作没有手工改写任何 DTO。

实施阶段另外记录的 5 个生成文件尾部漂移，本轮因导出启动失败未能重新验证。已检查 guard、normalizer、bindings recipe 及本地 ts-rs 导出实现；尚无可重现证据确定尾部残留原因。AC4 保留未通过。生成器失败恢复及门禁稳定性属于 T10 的跨域门禁范围，未在本任务修改脚本或依赖，也未循环重跑生成。

### N2：正式 frontend lint 的既有失败

bun run lint 失败，5 个 no-console 错误均来自受保护用户文件：.tmp-desktop-probe.mjs 的 20、24、92 行；.tmp-insights-visual.mjs 的 236、381 行。另有 2 个 warning，来自 measure-distribution.mjs 与 checkin-accounts-tab.smoke.test.tsx。未修改、排除或删除这些文件；未将缩减范围 lint 当作正式通过。check-frontend-lint.log 保留完整结果。

### N3：平台与交付范围

未运行完整 just tauri-ci 或 just ci；未进行 Linux/macOS OAuth socket、权限或 native desktop 验收；未访问真实 OAuth 服务、真实账户、真实软件安装或用户配置。当前 Windows loopback、受控进程、secret 文件失败 fixture 证明相应本机行为，不能替代上述范围。

## 23 个命令与 C01–C06

源码确认 command/install 使用同一个原 Singleton semaphore。后台 start 从 task-local 转交原 OwnedSemaphorePermit，不能重取另一个 permit；真实 runner 或 executor join owner 在清理完成后释放。OAuth 使用原 codex_auth module gate，幂等活动 start 先复用 ID，新登录才取得 permit。Usage 同步与后台入口使用同一 T06 registry/token。

| 命令 ID | 独立验证 |
| --- | --- |
| execute_ccr_command | C01/C05；foreground barrier；同 global gate |
| start_ccr_command_job | C01/C05；真实 registry/token owner；start 返回后持 permit |
| get_ccr_command_job_status | C01/C06；barrier 前读取真实 snapshot |
| cancel_ccr_command_job | C01/C06；barrier 前 token ack；错误 ID 不影响 owner |
| llmusage_install_detect | C02/C06；逐 ID runtime 投递；CLI 检测回归 |
| llmusage_install_probe_capabilities | C02/C06；逐 ID runtime 投递；能力回归 |
| llmusage_install_plan | C02/C05；共享 foreground gate，控制仍可达 |
| llmusage_install_execute | C02/C05；实际 InstallService/executor join owner；terminal-looking event 不释放 slot |
| llmusage_install_cancel | C02/C06；实际 attempt token ack；错误 ID 无影响 |
| llmusage_install_recent | C02/C06；真实 ring snapshot 在 cleanup barrier 下可读 |
| llmusage_install_manual_catalog | C02/C06；逐 ID runtime 投递；CLI catalog 回归 |
| llmusage_install_check | C02/C06；逐 ID runtime 投递；既有组合未更改 |
| import_usage_v2 | C03/C05；单源 foreground gate 与 T06 active owner |
| import_all_usage_v2 | C03/C05；全源 foreground gate 与同一 owner |
| start_usage_import_job_v2 | C03/C05；原子 admission/token，清理前复用活动 ID |
| get_usage_import_job_status_v2 | C03/C06；foreground barrier 期间 snapshot 可读 |
| cancel_usage_import_job_v2 | C03/C06；真实 token ack、cancel_requested、唯一终态 |
| codex_oauth_login_start | C04/C05；已绑定 listener、磁盘先保存、活动 ID 复用、同 module 互斥 |
| codex_oauth_login_completed | C04/C06；控制投递；exchange/commit 串行；commit 取消明确拒绝 |
| codex_oauth_login_cancel | C04/C06；silent/slow body 取消、真实 cleanup 后返回；新增 stale/retry 回归 |
| codex_oauth_submit_callback_url | C04/C06；exchange barrier 下可达；state/origin/旧 ID/重复校验 |
| codex_is_oauth_port_in_use | C04/C06；Parallel 投递与已绑定 loopback 探测 |
| codex_release_oauth_port | C04/C06；controller 取消与 ProcessGateway 所有权；外部 PID 保持 report-only |

install 独立 get/status 为 N/A，已有 recent；OAuth 独立 login get/status 为 N/A。runtime 的注入 body 测试证明逐 ID wrapper 投递；另有真实 registry/service/controller 的 owner/barrier 测试。没有将两者声明为 Webview IPC 点击验收。

C05 同时验证 command/install 跨入口互斥、wrapper abort 后已转交 permit 保留、early error 释放未转交 permit。C06 使用 registry 的真实 ACL/confirmation guard 和错误 ID 检查。独立 check-manifest-baseline.json 比较全部 348 个 descriptor：无新增/删除，只有 7 个 control 的 concurrency 变为 parallel，以及 OAuth start 的 timeout_enforcement 变为 business_owned；risk、authorization、confirmation、audit、输入输出类型和平台字段不变。

## Verification

所有 Rust 行为过滤器均使用 --offline --locked、--test-threads=1、--skip export_bindings。环境设置仅作用于命令：CCR_SKIP_ICON_GENERATION=1。不同过滤器有交集，不累计为唯一测试总数。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| command_exec | 24/24 | check-command-exec.log |
| runtime_policy | 11/11，修复后复验 | check-runtime_policy.log |
| oauth | 21/21，包含 3 个检查新增回归 | check-oauth-green.log |
| handler_registry | 21/21，包含 inventory、ACL 与确认 | check-handler_registry.log |
| usage | 70/70 + 2/2 no-crate guards | check-usage.log |
| services::install_ | 60/60 | check-install.log |
| port_release_report | 2/2 | check-port_release_report.log |
| process::gateway | 10/10 | check-process-gateway.log |
| Tauri cargo fmt --check | pass | check-fmt.log |
| Tauri cargo clippy --bin ccr-desktop -- -D warnings | pass；已有 MSVC linker_messages 提示保留 | check-clippy.log |
| bun run type-check | pass；生成恢复后的再次检查已在 root 停止重复检查消息到达前启动，并正常结束 | check-final-typecheck.log |
| API smoke | 3 文件、13/13；与上项同一已启动进程 | check-api-smoke.log |
| bun run lint | fail：既有 5 个用户文件错误 | check-frontend-lint.log |
| just tauri-bindings-check | fail：CLI export 进程启动异常；未运行导出 | check-bindings.log |
| git diff --check | pass | check-diff.log |

源文件与受保护用户文件指纹保存在 check-evidence.json。无任务状态、start 指针、commit、push、archive 或 Insights 生命周期改动。检查新增生产改动限于 OAuth adapter/controller、测试与对应规范。

## 逐 AC 与下游交接

- AC1：本机 scoped 通过。C01–C06、后台 owner permit、global cross-family 互斥、控制/权限均有独立证据。
- AC2：本机 scoped 通过。bind/save/state 失败不发布活动登录；恢复取消竞态已修复；重复完成不二次提交。
- AC3：本机 scoped 通过。silent socket、慢 body、取消/deadline、cleanup failure 和 commit completion-aware 由隔离测试覆盖。
- AC4：部分通过，整体未通过。manifest/security/sentinel/API 通过；bindings guard 失败保留给 T10。
- T07：command DTO/event/snapshot 不变，cancel 只请求取消，最终 terminal 等待 runner 实际完成；独立 scoped 前置批准已发给 root。
- T06：共享 Usage 门禁 70+2 已独立通过，可更新原等待 T11 的证据；不改变其 native 平台限制。
- T10：处理正式生成 guard 与完整聚合门禁；独立生成窗口已释放，当前无检查器后台进程。
