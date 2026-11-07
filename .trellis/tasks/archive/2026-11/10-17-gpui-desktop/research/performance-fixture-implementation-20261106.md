# Tauri 显式 fixture 隔离实施收据

2026-11-06；复用现有父任务、当前工作区。对应研究合同为
`performance-isolation-20261106.md`。本批实现可运行的隔离启动基础和本地核心图，
**不宣称整应用所有业务已接通，也不提供可比较的性能结果**。

## 已实现

- 启动参数 `--fixture-root <绝对现存目录>`；必须有
  `ccr-tauri-fixture.json`，内容形如
  `{"version":1,"fixture_id":"synthetic-s0"}`。标记最多4096 bytes，ID仅允许
  ASCII字母数字、下划线、连字符。重复参数、缺标记、未知版本、路径穿越和
  symlink/reparse拒绝；安装的进程身份不能被替换。
- 参数、标记与现有树验证在 logger、migration、Tauri Context/WebView 之前。
  固定角色路径检查实际canonical containment，后续admission重新检查固定owner
  路径。错误没有host fallback，没有设置HOME/USERPROFILE或环境路径替换。
- `home/`是provider与legacy HOME视图，`ccr/`是当前catalog/history/archive根；
  `ui/ccr-ui.db`为main DB，`ccr/analytics/usage.db`为独立archive，`llmusage/`
  为上游只读投影；logs/preferences/locks/backups/crypto/project/webview均在根内。
- `init_logger_at`复用现有SecureDailyWriter、脱敏、bridge、权限与14天清理规则。
  `migrate_catalogs_at`复用原catalog迁移、journal、源退役和锁。
- `initialize_app_pool_at`将返回的同一main Pool注册GLOBAL_POOL，重复绑定在打开
  新目标前拒绝；archive显式入口接收migration_home和legacy_ui_path，保留原schema
  与迁移owner。普通默认wrapper兼容。
- State保留fixture与偏好路径；加载、损坏隔离、异步保存同路径。HTTP client构建
  不调用host代理resolver。日志bridge/flush、缓存与后台维护生命周期保留；维护
  的storage检查使用捕获目录。
- fixture Context关闭配置窗口的自动create；State安装后按原WindowConfig显式创建
  main。main、隐藏tray panel和main重建共用root内WebView data_directory，并在标题
  明确显示`CCR · Fixture {id}`；不把合成模式混作普通用户状态。
- fixture startup不恢复OAuth listener、不探测WSL/SSH、不探测installed CLI。
  Local使用捕获根；刷新仍返回该Local。tray读取捕获Codex账户/runtime事实，
  fixture不请求quota，切换账户保持捕获owner与同步锁scope。
- handler registry先保留ACL之后的metadata/confirmation审计，再做fixture admission；
  Rust runtime_policy同样在future body之前拒绝。只有明确审计的ID可运行，unknown、
  未接入和外部能力拒绝并显示静态原因。ReadOnly风险标签不自动获得放行。
- 捕获的generic/Claude Profiles含真实读写/CAS与apply/off；Claude Settings、MCP、
  Agents/Plugins/SlashCommands消费原owner；usage只读投影、session archive/provider
  detail/refresh、UI-state/偏好、全局monitoring保持原消费者。未增加另一套业务SQL。
- ClaudeAuthService及ProfileLifecycleContext默认保留process-env观察；显式fixture
  `with_process_environment(false)`从构造贯穿只读认证及apply/off中重建的auth owner。
  文件证据保持。Tauri和原native Fixture/Claude均显式false；认证mutation/OAuth在
  Tauri fixture中继续关闭。
- Auth summary自身还需`with_profile_paths`，由PlatformRoots统一绑定；否则内部
  `ClaudePlatform::new`仍可读ambient数据。关闭process-env但缺profilepaths的service
  在任何ambient平台构造前静态拒绝。该修复同样覆盖native内部account-list调用。

## 实际验证

使用固定Rust1.98.0、真实cargo/rustc与Git Bash PATH，仅调整检查进程。
目标缓存为既有受管链接，默认沙箱无法开build-lock；通过授权后的同一窄命令运行，
没有改ACL或清缓存。所有Rust测试默认并行，保留`--skip export_bindings`。

| 命令/范围 | 实际结果与覆盖 |
| --- | --- |
| Tauri `check --locked --offline --bin ccr-desktop` | exit0；首次仅4个本批类型错，已修；后续默认wrapper调用保留，未加lint suppress |
| Tauri `test ... --bin ccr-desktop fixture::tests` | 3 passed，1 ignored helper；父测试实际启动helper且要求`1 passed`，不是零匹配 |
| fixture child真实核心图 | legacy catalog迁移、同一global/State池、第二安装拒绝且不创建目标、A/B偏好隔离、真实profile更新/CAS冲突、guarded Settings备份、usage 3requests/42tokens、MCP真实写/读、session真实refresh/detail、monitoring落main DB，以及OAuth future body未执行 |
| 根 `-p ccr-cli --lib captured_` | 最终5 passed；新auth-source、apply/off继承策略和summary捕获3条，另2为既有captured logout/Grok回归，不按全新数量累计；summary用合成ambient-root普通文件trap证明不回落 |
| 同一新测试exe `services::claude_auth_service::tests` | 26 passed，含默认service行为与显式summary捕获 |
| 根 `-p ccr-core --test log_path_isolation` | 3 passed，另helper由父真实执行；normal console/file-only及explicit目录，explicit设置不同ambient CCR_ROOT，保留跨根字节/mtime与managed清理边界 |
| 当前Tauri测试exe `commands::handler_registry` | 21 passed；原命令ID/typed client/ACL/confirmation/inventory回归 |
| 同exe `commands::runtime_policy` | 11 passed；原控制交付、生命周期、并发permit和deadline回归 |
| Tauri `clippy --all-targets --all-features -- -D warnings` | exit0；工具链proc-macro linker informational message不受`-D`控制，无代码allow来换绿 |

独立check发现并修复两条隐性lock fallback：实际Claude add/delete，以及
SettingsManager async helper内guarded writer没有继承captured lock scope。
新增独立child将CCR_LOCK_DIR指向普通文件；两条测试真实先OS183失败、修后2/2通过。
不能拿manager字段存在或generic update通过替代这两条证明。

reviewer最终Tauri全目标/全特性严格clippy exit0，包含锁、summary与审计顺序修复。
随后补fixture标题与更强child ambient路径trap后，fresh core fixture3/3再次通过：
CCR_ROOT/CCR_DATA_DIR/CCR_CONFIG_PATH/CLAUDE_CONFIG_DIR均指向合成普通文件，实际
auth/MCP/CRUD仍使用捕获根。没有修改HOME/USERPROFILE；没有新窗口运行收据。

## 保持开放

- 这批不是全部Tauri业务对等：未审计平台/网络/进程/SSH/WSL/安装/更新/外部打开等
  ID仍closed。网络真实动作不能因fixture存在而悄悄放行；需要逐域captured owner。
- 当前native测试Fixture的profiles与platforms派生不同根，不能复制几份数据冒充
  同逻辑基线。新benchmark启动身份需显式冻结和统一captured布局；旧Fixture布局不改。
- S0/S1/S2同逻辑seed、数据hash、动态S2日志注入尚未实现；当前测试小数据不是性能场景。
- process-create/first-present/action-ready真实时钟尚未实现；没有拿main入口、JS paint、
  layout/on_next_frame或平均FPS冒充真实presented时间。
- WebView localStorage/cookies/IndexedDB/cache实际隔离与native窗口加载尚未验证，
  用户已明确“稍后再做原生验证”；本批没有启动任何窗口。
- macOS root-contained WebView profile当前锁定API不满足支持系统范围，fixture在任何
  副作用前明确拒绝；不是macOS通过。Windows/Linux builder配置不代表Linux native收据。
- 性能结果均为`not_run`，两端比较均为`not_comparable`。未改变预算/CI阈值、任务状态
  或归档条件；父任务和01/09保持开放。
