# 本轮 Rust 增量独立审查

审查完成：在指定六个文件的本轮增量中未发现新增代码缺陷。四组已有 Tauri 日志包含 **62 个唯一通过 selector**。Tauri all-targets Clippy 的精确命令与 exit 0 已由作者补充归档；六个当前 Rust 源码 SHA 均独立核对为匹配 root 冻结。源码归属采用执行后确认，相关证据边界已保留。

## 范围与独立性

- Reviewer：/root/implement_t10_gates；被审作者：/root/implement_t09。
- 专用 trellis-check 派发再次因 agent thread limit reached 失败。按 root 指派复用现有非作者代理，使用已读取的 trellis-check 流程。没有将该代理称为成功派发的专用 checker。
- 只审查 main/state item 位置、Claude 布尔断言、Codex 同步 runtime/envguard、Codex settings helper 与磁盘往返、ccr-codex Windows 测试导入。
- 排除 state/Claude 在本轮之前的架构改动、前端与译文、本 reviewer 自己完成的两份 Cargo.lock 安全升级。未修改产品、原日志、矩阵、父报告或任务状态。

## 源码检查

| 位置 | 核对结果 |
| --- | --- |
| ccr-ui/src-tauri/src/main.rs:500–546 | tests 模块整体移至文件末尾。两个 Windows 可执行 close-action 测试及 macOS cfg 测试的主体和断言未变；后台维护函数未改。 |
| ccr-ui/src-tauri/src/state.rs:814–896 | CacheFillRegistration、push_sample、percentile_95 完整移到 tests 前。三个 preferences 测试未删减。前序 usage lifecycle 改动不归入本轮。 |
| ccr-ui/src-tauri/src/commands/claude.rs:699–732 | assert_eq!(is_some(), true) 改为 assert!(is_some())，仍要求 statusLine 为对象，断言强度相同。 |
| ccr-ui/src-tauri/src/commands/codex.rs:2280–2340 | 普通 #[test] 创建 current-thread runtime，再获取共享 ENV_LOCK。guard 仍覆盖两个环境变量修改、runtime.block_on、codex_list_profiles().await 和恢复。profile 数量、real 名称、无 legacy 三项断言不变。 |
| ccr-ui/src-tauri/src/commands/codex_settings.rs:18–39 | handler 仍在 spawn_blocking 内解析原路径并执行同一 read → apply → write → message 顺序。private helper 不新增命令或 DTO。join 错误映射、双 ?、成功后的 cache invalidation 和 open_json 顺序未变。 |
| crates/ccr-codex/src/services/codex_process_service.rs:566–578 | 仅 tests 模块中的 SysinfoProcessBackend / process_refresh_kind 导入增加 cfg(windows)。使用点均在 :712 的 Windows fixture 内；生产 backend、跨平台 process 实现与测试断言不变。 |

六个文件均在内存中反向还原本轮改动，得到与 continuation-*-before.json **完全一致的原始字节 SHA-256**。没有写回还原结果。settings 文件原有混合换行；还原时保留前 22 行现有换行，并恢复旧第 23–34 行 CRLF、35–36 行 LF 后匹配原始 SHA。相应 SHA、当前换行数量及增量范围在 JSON 中逐项记录。

Codex profile handler 的单个 spawn_blocking 在 codex_profiles.rs:12–46 被 await，测试返回前该 worker 已完成。本轮未引入未等待的任务。原测试手动恢复环境变量在断言 panic 下的恢复边界仍存在；本轮没有改变该行为，也未将成功路径测试描述为 panic 恢复验收。

## Settings 实际持久化证据

新增 selector 为 commands::codex::settings::tests::model_only_update_preserves_notifications_and_untouched_fields_on_disk，定义在 codex_settings.rs:46。

测试将完整 fixture 写到临时 config.toml，使用生产 handler 调用的同一 update_codex_settings_at_path helper，仅提交 model 字段，然后从磁盘重新读取并解析 TOML。测试断言返回消息、model 新值、notifications 精确数组，并把原 TOML 仅改 model 后与保存结果完整比较。比较覆盖 fixture 中未知根数组、tui notification_condition/notification_method、status_line、history、profiles 与 mcp_servers 字段。

底层仍调用 codex.rs:827–845 的 read_codex_config 与 AtomicWriter.secret(true) 写入。apply_codex_settings_update 位于 :1294；非目标字段省略时维持原值。handler 在 persistence 返回成功后才调用 :809 的 cache_remove，错误传播顺序保持。registry :653 仍引用 super::codex::codex_update_settings；codex.rs:317–330 仍加载并导出同一 settings 模块。

该测试提供共享持久化 helper 的真实文件语义往返证据。该测试未经过 Tauri invoke、State/cache 路径或原生 WebView；未验证评论、键顺序、格式字节、写失败故障注入或所有 TOML 类型的通用保真。

## 已有执行日志核对

以下结果由作者或 root 的已有运行提供。reviewer 只解析日志和元数据，未重新执行 Cargo。

| 元数据 selector | 实际唯一通过 | 日志 | 退出码 |
| --- | ---: | --- | ---: |
| commands::codex:: | 56 | continuation-codex-tests.log | 0 |
| state::tests:: | 3 | continuation-state-tests.log | 0 |
| tests::close_action | 2 | continuation-close-action-tests.log | 0 |
| commands::claude::tests::read_claude_settings_from_env_reads_top_level_env | 1 | continuation-claude-read-tests.log | 0 |

四条命令均为 cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml <selector> -- --skip export_bindings。命令未添加串行参数。作者补充元数据记为 Cargo default；未保存执行时 RUST_TEST_THREADS 环境快照，报告不据此推断具体线程数。

已逐行提取 selector 并去重。新增文件往返、同步 runtime profiles 测试、原通知数组 read/merge 测试均有实际 ok 行。每组另有 0 tests 的 integration guard target，全部排除。export_bindings 未计入；macOS cfg 的 chrome 测试未运行。JSON 保存完整 selector、日志行号、原始日志 SHA 与摘要。

continuation-tauri-all-targets-clippy.log 结尾记录 Finished dev profile，耗时 40.69s；未出现 error 行。linker_messages 和 mbx cache 警告仍在，不能称为零警告。作者已在 continuation-tauri-lint-implementation.json 归档精确命令 cargo --config .cargo/tauri-ci.toml clippy --manifest-path ccr-ui/src-tauri/Cargo.toml --all-targets -- -D warnings，cwd 为 D:/Documents/Code/Github/ccr，exit 0 来自 executor session 50695 的完成观察。原始 compiler log 已保存，未另存原始 executor 完成响应 JSON；本报告保留该来源区别。

continuation-linux-codex-clippy-online.json 明确记录 Rust 1.98.0、cargo clippy --locked -p ccr-codex --all-targets --all-features -- -D warnings，exit 0。metadata 的 codex_process_service.rs 与 Cargo.lock SHA 均匹配当前读取版本。该证据验证本次 Windows-only 导入在 Linux 上的编译检查；该证据不属于 Rust 1.95 MSRV 验收。

## 源码归属补充

作者元数据使用 capture_mode=post_run_confirmed_unchanged，说明六个 Rust 文件在 Clippy 与后续四组测试前冻结，之后没有该 owner 的源码写入。reviewer 已独立核对作者记录、当前六个文件与 continuation-source-freeze.json 完全一致，并核对五个 Tauri 日志 SHA、argv 与退出码记录一致。

这些 SHA 在执行后采集，没有逐命令开始时快照，也不构成全部依赖或编译输入的快照。报告没有将执行后确认改称开始时采集。JSON 保存作者元数据和 root freeze 的 SHA，以区分直接日志事实、作者执行记录与独立源码核对。

## 规范与剩余边界

所核规范包括 typed-ipc-bindings 的 State-free helper、命令类型、默认行为测试与 exports 分离；tauri-handler-registry 的阻塞 worker 生命周期；ccr-codex test-fixtures 的显式临时路径；app-server-cleanup 的受控 Windows fixture。helper 提取没有修改 wire schema、注册项或 timeout/admission 元数据，无需从本轮变更推导生成任务。

本审查没有运行完整 CI、bindings/exports、UI 或真实 provider OAuth。macOS 原生进程清理矩阵仍须由父任务单独保持未完成；Windows/Linux 日志不替代该验收。报告不发布父任务或整个项目的全绿结论。
