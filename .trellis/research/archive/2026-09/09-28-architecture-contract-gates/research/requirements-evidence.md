# T10 逐项验收证据矩阵

生成日期（UTC）：2026-09-29。基线：34d8a85e0e48b793733835e0304c8ed33940fcee。没有创建修复提交，fix_commit 为 null。

本表覆盖 39 个子任务 AC。精确测试选择器、源码行、原始日志、SHA-256 和 mock/平台边界保存在 requirements-evidence.json；表内引用的是证据组，不是新增通过结论。旧失败复现仍缺失的 P1 必须保持开放。

| AC | 机制 | 测试组 | operation × platform × entry |
| --- | --- | --- | --- |
| T01.AC1 | 资源身份统一；锁内read/validate/mutate/leaf CAS；并发不同字段均保留 | repository, config_handlers | config mutation;Claude/Codex/Grok;service/platform/Tauri |
| T01.AC2 | 显式平台和同源current投影；读取不bootstrap/autofix | repository, diagnostics, permission_noop_repository | list/current/validate;three platforms;CLI/shared repository |
| T01.AC3 | 严格patch与version conflict拒绝；unknown TOML/secret/marker保留 | repository, config_handlers, permission_noop_repository, permission_noop_guard, permission_base_regression | patch/rename;three platforms;repository/Tauri |
| T02.AC1 | 共享application预检与已发布写journal；逐写故障逆序补偿，外部新版保留 | profile_cli, profile_tui, profile_desktop, journal | apply;Claude/Codex/Grok;CLI/TUI/Tauri |
| T02.AC2 | 同一contract harness比较持久状态；operation ID重放不重复计数 | profile_cli, profile_tui, profile_desktop | apply/replay;Claude/Codex/Grok;three adapters |
| T02.AC3 | committed outcome与ancillary warning分离；页面不自动再激活 | profile_warning_binary, profile_tui, config_page | history failure;Claude/Codex/Grok;binary/presenter/React |
| T02.AC4 | 三个adapter委托同一同步application owner；边界guard检查无终端IO/exit | profile_cli, profile_tui, profile_desktop | application boundary;three platforms;three adapters |
| T02.AC5 | rename纳入同一journal，current/default与unknown值在逐写故障下保持可解释 | profile_cli, config_handlers | rename;Claude/Codex;CLI/Tauri |
| T03.AC1 | explicit Claude配置页switch/enable进入共享application，enabled/runtime/current一致 | config_handlers, config_page | switch/enable;Claude generic page;React/generated/Tauri/application |
| T03.AC2 | 对象形状/absent/null验证；active保护在operation lock内重读；多进程合并 | config_handlers, repository | CRUD;Claude generic page;Tauri/repository |
| T03.AC3 | 缺失平台明确失败无写；typed client/registry同步；完整bindings另列 | config_handlers, registry | wire compatibility;Claude generic page;registry/generated/Tauri |
| T04.AC1 | binary统一返回诊断ExitCode；invalid/corrupt/unreadable与warning分类 | diagnostics | validate;Claude/Codex/Grok;actual binary |
| T04.AC2 | validate/apply复用领域auth validator；原生denied read不降为missing | diagnostics, profile_cli | validate/apply;API-key/subscription;binary/application |
| T04.AC3 | 只读前后路径/字节/mtime相同；Grok支持、Gemini/Droid保留adapter状态明确 | diagnostics, repository | doctor/current;five platform capability labels;binary/shared |
| T05.AC1 | 固定时钟下UUID no-clobber备份，每次前镜像独立，保留keep-10 | backups | write/backup/rotate;Windows/Linux;guarded writer |
| T05.AC2 | 旧新命名稳定reader/rotation；DACL和mode在payload前建立，readonly发布后恢复 | backups, permission_noop_guard, permission_base_regression, permission_atomic_policy | restore/metadata;Windows/Linux;writer/consumer |
| T05.AC3 | pending secret:true/BackupPolicy::None；disk-first发布、失败保留旧目标 | pending, oauth | create/replace/cancel/expire;Windows/Linux scoped;store/Tauri |
| T05.AC4 | Debug/Serialize/errors/events不含合成verifier/token sentinel | pending, oauth | failure/cancel/expiry;Windows/Linux scoped;store/controller |
| T06.AC1 | 一次锁内发布snapshot/token；cancel_requested保留active直到runner cleanup完成 | usage_registry, control_policy | start/cancel/admit;all usage providers;Tauri owner |
| T06.AC2 | 五种terminal不可回退；取消请求与清理完成分开；诊断保留 | usage_registry, usage_ui | cancel/progress/terminal;usage job;backend/frontend |
| T06.AC3 | execution deadline覆盖stdout/callback/wait/stderr；cleanup共享预算并回报失败 | usage_executor, process_tree | stream/cancel/cleanup;Windows/Linux;adapter/core |
| T06.AC4 | stdout 1MiB+1、stderr 64×64KiB有界；no-crate guards验证SQL owner | usage_executor, usage_registry | stream bounds/DTO;usage adapter;backend |
| T07.AC1 | shell/store持有job；路由重挂载对账snapshot并恢复取消 | command_page, command_owner | route leave/return;command jobs;React and backend owner |
| T07.AC2 | 同步提交锁、job-ID隔离、terminal优先；history pending/saved/failed且单次写尝试 | command_page, command_owner | start/events/history;command jobs;React/store/backend |
| T07.AC3 | disposed协议释放迟到listen；shell暂停隔离迟到start/cancel响应 | command_listener, command_page | mount/unmount/listen;command events;React shell |
| T08.AC1 | snapshot与dirty leaves无损映射；只发送编辑字段，保留union/unknown/default语义；Codex helper真实磁盘往返 | settings_mapping, settings_page, codex_settings_backend, codex_settings_persistence | typed save;Codex/OpenCode;React/mapper/domain/Rust persistence helper |
| T08.AC2 | managed metadata进入可执行capability；disabled+原因+未知枚举当前值 | settings_page, grok_settings, settings_visible_i18n | managed settings;Grok/Codex;React/backend |
| T08.AC3 | 真实raw editor入口、确认/CAS/invalid草稿；Local能力约束与Grok无备份；原生Linux公共编辑器CSP单列 | settings_page, grok_settings, settings_raw_session, runtime_style_nonce | raw edit;Claude/Codex/Grok;React/CodeMirror/Tauri |
| T09.AC1 | probe/load状态明确；stale成功数据仍可见，off pending阻止重复并保留失败前状态 | auth_query, auth_cache | auth probe/load/off;platform adapters;React/Query |
| T09.AC2 | snapshot/baseline/draft分离；环境身份/世代冻结旧会话，后端捕获同一目标 | settings_session, settings_raw_session, settings_environment | refetch/save/environment transition;typed/raw settings;React/Query/Tauri |
| T09.AC3 | 实际翻译订阅参与memo依赖，同一Query引用和挂载卡片语言切换 | config_page | locale switch;Configs;React |
| T10.AC1 | 跨workspace aggregate复用tauri-ci；已运行故障fixture非零；root正式完整gate单独验收 | aggregate, bindings, calendar, lint_main_tests, lint_state_tests, lint_claude_test, lint_codex_profile_test, doctor_deadline, tauri_process_gateway, oauth_windows_pending_failure, store_path_platform_contract | aggregate;Windows/Linux/macOS declared;local/hosted |
| T10.AC2 | 39AC及9P1逐项映射，旧源码/实际红例/修后行为分开；缺失红例不提升验收 | artifact_evidence：6 项 | evidence;all tasks;review |
| T10.AC3 | registry/manifest权威计数；现行路径与usage owner导航；生命周期契约无损抽取 | artifact_evidence：7 项 | spec navigation;all packages;documentation |
| T10.AC4 | 正式失败原样保留；保护脚本哈希和Insights状态单独核对；不创建commit | artifact_evidence：15 项 | baseline/final gates;original workspace;root integration |
| T11.AC1 | 控制投递与执行admission分离；真实owner持permit到cleanup/join结束 | control_policy, command_owner, usage_registry, oauth | C01-C06;23 command/install/usage/OAuth IDs;registry/runtime/owner |
| T11.AC2 | bind和disk先成功后发布URL；单controller负责唯一terminal及active reuse | oauth, pending | start/complete;Codex OAuth;loopback/controller/store |
| T11.AC3 | listener/socket/body/exchange有界取消；commit_in_progress拒绝伪取消 | oauth | cancel/deadline;Codex OAuth;loopback/HTTP/controller |
| T11.AC4 | ACL/confirmation元数据保持；sentinel不外泄；完整binding guard保留调用前230文件 | registry, oauth, pending, bindings | wire/security;Tauri;registry/generated/controller |

## 验收限制

- 使用真实前端组件与底层 transport mock 的测试，只证明所列前端调用链；Rust 临时文件、真实子进程和 loopback fixture 单独提供后端证据。两者组合不等于原生 UI 端到端。
- A01/A02/A03/A11 原始 CLI 库红例、A09 缩短 descriptor deadline 的原生子进程红例、A14 原 mapper 红例和 A15 原组件红例均已记录。A02 为确定性旧快照交错；A03 仅覆盖已提交后的 history failure；A11 为原函数权限失败组合。各范围不外推。
- A09 旧 cleanup-error precedence、新增 descendant 用例、A03 旧 TUI off-before-invalid 与每种 rename 故障、A15 旧 raw-editor 可达性未分别运行旧基线，保留为源码证据。A08 旧 pre-run spawn 窗口同样保留源码边界。
- T10 AC2/AC3/AC4 的 artifact_evidence 分别保存 P1/旧红例、规范导航/抽取/上下文、正式与隔离门禁/受保护基线的路径及 SHA-256。空 test_groups 不表示空验收证据。
- T10 专职 trellis-check 已恢复，当前报告为 remaining-dedicated-check.md/json。历史线程限制失败仍保留；T01 既有作者关系和本轮文档单行修复的独立性边界单独记录。
- 历史原工作区 frontend-check 保留两个脚本的五个 no-console 错误。后续五处输出替换已获授权，remaining 正式 frontend-check 通过；旧失败原样保留。root CI、Linux Tauri 和原生 WebView 使用各自的原始日志及源码归属。

## 本轮补充证据

| 运行 | 退出码 | 范围 |
| --- | ---: | --- |
| tauri_all_targets_clippy | 0 | 见原始日志和命令元数据 |
| tauri_codex | 0 | {"matched_passed": 56} |
| tauri_state | 0 | {"matched_passed": 3} |
| tauri_close-action | 0 | {"matched_passed": 2} |
| tauri_claude-read | 0 | {"matched_passed": 1} |
| settings_smoke | 0 | {"matched_passed": 45, "test_files_passed": 4} |
| i18n | 0 | {"matched_passed": 24, "locale_leaf_keys": 4523} |
| typescript | 0 | 见原始日志和命令元数据 |
| scoped_eslint | 0 | 见原始日志和命令元数据 |
| linux_198_process | 0 | 见原始日志和命令元数据 |
| linux_198_codex_clippy | 0 | 见原始日志和命令元数据 |
| security_root_audit | 0 | 见原始日志和命令元数据 |
| security_tauri_audit | 0 | 见原始日志和命令元数据 |
| security_dependency_governance | 0 | 见原始日志和命令元数据 |
| doctor_original_narrow | 0 | 见原始日志和命令元数据 |
| doctor_controlled_red | 101 | 见原始日志和命令元数据 |
| doctor_startup_failure | 5 | 见原始日志和命令元数据 |
| doctor_focused_retry | 0 | 见原始日志和命令元数据 |
| doctor_package | 0 | {"unit_passed": 345, "integration_passed": 12, "doctest_passed": 1, "doctest_ignored": 1} |
| doctor_clippy | 0 | 见原始日志和命令元数据 |
| linux_198_doctor | 0 | {"passed_count": 3} |
| root_full_ci_before_doctor | 1 | 见原始日志和命令元数据 |
| root_full_ci_after_doctor | 1 | 见原始日志和命令元数据 |
| formal_frontend_gate | 1 | 见原始日志和命令元数据 |
| linux_195_workspace_msrv | 0 | 见原始日志和命令元数据 |
| windows_195_tauri_msrv | 0 | 见原始日志和命令元数据 |
| ui_production_build | 0 | 见原始日志和命令元数据 |
| bindings_followup | 0 | {"cli_binding_exports": 24, "usage_binding_exports": 9, "tauri_binding_exports": 197, "zero_case_targets_not_counted": true} |
| standalone_tauri_ci | 0 | {"tauri_behavior_passed": 407, "tauri_behavior_ignored": 1, "tauri_guard_passed": 2, "cli_binding_exports": 24, "usage_binding_exports": 9, "tauri_binding_exports": 197, "inventory_passed": 1, "inventory_overlaps_behavior_suite": true, "zero_case_targets_not_counted": true} |
| linux_198_tauri_process_before_import_fix | 0 | 见原始日志和命令元数据 |
| linux_198_core_process_before_import_fix | 0 | 见原始日志和命令元数据 |
| linux_198_tauri_strict_final | 0 | 见原始日志和命令元数据 |
| linux_oauth_zero_match | 0 | {"aggregate_passed": 0, "intact_named_selector_count": 0, "named_capture_complete": true, "overlaps_other_runs": true} |
| linux_198_tauri_process_final | 0 | {"aggregate_passed": 8, "intact_named_selector_count": 5, "named_capture_complete": false, "overlaps_other_runs": true} |
| windows_195_tauri_msrv_final | 0 | 见原始日志和命令元数据 |
| windows_oauth_zero_match | 0 | {"aggregate_passed": 0, "intact_named_selector_count": 0, "named_capture_complete": true, "overlaps_other_runs": true} |
| tauri_fmt_final | 0 | 见原始日志和命令元数据 |
| linux_oauth_corrected | 0 | {"aggregate_passed": 20, "intact_named_selector_count": 18, "named_capture_complete": false, "overlaps_other_runs": true} |
| windows_oauth_corrected | 0 | {"aggregate_passed": 1, "intact_named_selector_count": 1, "named_capture_complete": true, "overlaps_other_runs": true} |
| frontend_full_tests | 0 | {"files": 168, "tests": 901, "overlaps_other_runs": true} |
| frontend_coverage | 0 | {"files": 168, "tests": 901, "overlaps_other_runs": true} |
| remaining_windows_frontend | 0 | {"files": 168, "tests": 901, "i18n_checks": 24, "overlaps_other_runs": true} |
| remaining_windows_ci_before_fixture | 1 | 见原始日志和命令元数据 |
| remaining_linux_tauri_ci | 0 | {"tauri_behavior_passed": 397, "tauri_behavior_ignored": 1, "tauri_guard_passed": 2, "cli_binding_exports": 24, "usage_binding_exports": 9, "tauri_binding_exports": 197, "inventory_passed": 1, "inventory_overlaps_behavior_suite": true, "zero_case_targets_not_counted": true} |
| remaining_linux_coverage | 0 | {"tauri_behavior_passed": 397, "tauri_behavior_ignored": 1, "tauri_guard_passed": 2, "overlaps_linux_tauri_ci": true} |
| remaining_linux_native_build | 0 | 见原始日志和命令元数据 |
| remaining_windows_ci_after_fixture | 1 | 见原始日志和命令元数据 |
| remaining_csp_focused | 0 | 见原始日志和命令元数据 |
| remaining_csp_types | 0 | 见原始日志和命令元数据 |
| remaining_csp_lint | 0 | 见原始日志和命令元数据 |
| remaining_csp_ui_build | 0 | 见原始日志和命令元数据 |
| remaining_linux_native_after_csp | 0 | 见原始日志和命令元数据 |
| remaining_linux_tauri_ci_final | 0 | 见原始日志和命令元数据 |
| remaining_linux_coverage_final | 0 | 见原始日志和命令元数据 |
| remaining_windows_ci_final | 0 | 见原始日志和命令元数据 |
| remaining_linux_workspace_quality_after_permissions | 127 | 见原始日志和命令元数据 |
| remaining_linux_workspace_quality_after_octal | 101 | 见原始日志和命令元数据 |
| remaining_windows_ci_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_tauri_ci_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_coverage_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_native_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_workspace_quality_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_workspace_coverage_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_workspace_audit_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_workspace_msrv_after_path | 0 | 见原始日志和命令元数据 |
| remaining_linux_workspace_coverage_final | 101 | 见原始日志和命令元数据 |
| remaining_linux_permissions_red | 101 | 见原始日志和命令元数据 |
| remaining_linux_workspace_quality_after_python | 101 | 见原始日志和命令元数据 |
| remaining_linux_store_path_red | 101 | 见原始日志和命令元数据 |
| remaining_doctor_empty_path_red | 101 | 见原始日志和命令元数据 |
| remaining_doctor_focused | 0 | 见原始日志和命令元数据 |
| remaining_doctor_library | 0 | 见原始日志和命令元数据 |
| remaining_doctor_clippy | 0 | 见原始日志和命令元数据 |
| remaining_permissions_config | 0 | 见原始日志和命令元数据 |
| remaining_permissions_guarded | 0 | 见原始日志和命令元数据 |
| remaining_permissions_atomic | 0 | 见原始日志和命令元数据 |
| remaining_permissions_clippy | 0 | 见原始日志和命令元数据 |
| remaining_permissions_check | 0 | 见原始日志和命令元数据 |
| remaining_store_path_linux_exact | 0 | 见原始日志和命令元数据 |
| remaining_store_path_windows_exact | 0 | 见原始日志和命令元数据 |
| remaining_store_path_linux_lib | 0 | 见原始日志和命令元数据 |
| remaining_store_path_linux_clippy | 0 | 见原始日志和命令元数据 |
| remaining_store_path_format | 0 | 见原始日志和命令元数据 |
| remaining_store_path_diff_check | 0 | 见原始日志和命令元数据 |
| remaining_cli_versions_diagnostic | 0 | 见原始日志和命令元数据 |
| remaining_cli_versions_focused | 0 | 见原始日志和命令元数据 |

- 四组 Tauri 日志去重后为 62 个实际通过 selector；空 target 和 export_bindings 不计入。不同运行之间存在覆盖重叠，不累计为唯一测试总数。
- Web 的 7 个成功 receipt 使用合成 IPC；model-only patch、通知数组、未知值、raw CodeMirror 与纯 Web 不可用状态已记录。实际磁盘写入属于独立后端 helper fixture。
- 当前源码 SHA 只定位选择器。运行绑定单列 capture_mode、recorded_source_sha256 与一致性结果；旧日志不会因 SHA 刷新而获得新源码执行证明。
- 覆盖率按原始日志记录 Statements、Branches、Functions、Lines 的分子与分母，不将百分比四舍五入为满足其他阈值。

- 两次 OAuth 错误选择器运行均为 exit 0 / 0 匹配，明确排除行为验收。修正后 Linux Rust 汇总为 20 项通过、完整具名记录为 18 项；Windows 汇总与完整具名记录均为 1 项。process 复验汇总 8 项通过、完整具名记录为 5 项；缺损名称不猜补，重复运行不累计。
- a531、fa79、46f5 和 d8ee 冻结分开记录。旧 full CI、独立 tauri-ci、MSRV 和 OAuth 定向检查保留原来的源码归属；新正式 Linux 门禁与各自的命令前后 map 关联。fa79 的继承 lineage 计数偏差保留独立 provenance correction。
- 原 exporter AV 和后续 Doctor fixture spawn 失败分属不同的 root CI 运行。后续重跑不修改历史失败，AV 原因仍未查明。

## 剩余事项续作

- 旧 41 条运行对象逐个按 canonical SHA-256 校验保持一致；旧 227 项 artifact 的验证计数保存在 remaining-mapping-baseline.json。
- Linux 完整 just tauri-ci：397 项行为测试通过、1 项 ignored、2 项 guard；导出分区为 24/9/197。inventory 的 1 项与行为测试重叠，空 target 不累计。
- d8ee 阶段 gateway 行覆盖率为 654/703（93.0298719772404%），现行门槛为 85%。该阶段 Tauri 总体行覆盖率为 17964/34456（52.1360575806826%）；46f5 及后续阶段报告分别保留。现行 Tauri recipe 未设置总体门槛。
- 原生 attempt6 在 Linux WebKitGTK/WSLg 中通过：确认滚动锁与 CodeMirror nonce、CSP 内联脚本拒绝、合成 Claude 配置磁盘往返及清理。attempt1–5 的失败均保留；不外推到 Windows/macOS WebView、Codex/Grok 原生保存或发布安装包。
- 原生执行时二进制 SHA 与 build 记录关联。后续 Cargo 会覆盖同路径可执行文件，当前文件哈希不同不重写执行时证据。
- 最新完成的 root aggregate：remaining_windows_ci_after_path，exit 0。
- The planned macOS native process matrix is required and not run
- Native Linux WebKitGTK custom-protocol debug acceptance passed for the synthetic Claude raw editor, confirmation scroll locking, CSP and file save. Windows/macOS WebViews, native Codex/Grok and release packaging remain outside this receipt
- Native post-save style evidence records no captured style CSP violations. Style DOM removal and body scroll-lock restoration were not measured; process-group cleanup has separate evidence
- No real remote SSH/WSL accounts or provider OAuth login
- Historical root exporter STATUS_ACCESS_VIOLATION remains unexplained; later completed runs do not establish its cause
- The earlier five no-console errors remain in immutable failed logs. The two script edits were subsequently authorized, and the separate remaining formal frontend gate passed
- Actual Linux just tauri-ci and coverage-tauri passed on remaining source maps; neither command is the entire Linux root CI
- d8ee-epoch Tauri gateway line coverage is 654/703 (93.0298719772404%), with threshold 85%; its overall is 17964/34456 (52.1360575806826%) without a Tauri overall threshold; the 46f5 report and later epochs remain separate
- Source epochs and command-start maps remain distinct. No source hash refresh reattributes historical runs
- Task context validators exited zero but retained context-injection warnings; see remaining_context_preservation.warnings for exact affected tasks and limits
