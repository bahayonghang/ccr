# Tauri 整应用 fixture 隔离：源码研究与实施合同

2026-11-06；本文件是独立源码研究/设计交接，**没有实现 fixture boot**。
没有启动 UI、WSL、SSH、OAuth、真实账户、网络或进程探测，没有修改
HOME/USERPROFILE、用户数据或任何生产代码。没有采集性能样本。

结论：仅新增 `main.rs` fixture 分支和几个环境变量，不能兑现整应用隔离。
当前已存在可复用的显式 roots、领域构造和连接池接口，但启动顺序、全局池
注册、logger、WebView profile 与未捕获 roots 的 IPC 必须一起闭合。
下列设计可交给真正 implement 角色实施；不能把中间隔离探针作为可比较的
整应用 fixture 模式交付。

## 1. 已证实的泄漏入口

| owning source | 当前行为 | fixture 必须处理的位置 |
| --- | --- | --- |
| `src-tauri/src/main.rs:120` | 第一条操作是 `ccr_core::init_logger()` | 解析/验证 boot mode 必须在 logger 之前；错误只发静态 stderr，不初始化 host logger |
| `crates/ccr-core/src/core/logging.rs:250,395` | logger 取 CCR_ROOT/logs 或 HOME/.ccr/logs；创建目录、清旧日志、收紧文件权限 | 显式 logger log-dir 接口；不能先初始化再改 env，也不能 fixture 失败回 host |
| `main.rs:135`；`ccr-config/.../catalog/migrate.rs:29` | `migrate_catalogs()` 为每个平台 `PlatformPaths::new`，可能迁移/移动旧 profiles.toml | 对捕获的 PlatformPaths 调用同一迁移 owner；不调用 ambient 遍历 |
| `ccr-db/src/database/mod.rs:69,107` | 主 DB 固定 HOME/.ccr-ui/ccr-ui.db；初始化后放 GLOBAL_POOL 和 AppState | 显式 DB 路径 + migration home，注册同一个池；不能开一个 fixture State 池却让 managers 用 host global |
| 同文件 `133`；`migrations.rs:1847,2047` | archive 路径虽然认 CCR_ROOT，但迁移另用 HOME 和 legacy main DB，读取 HOME/.ccr/data.db | archive、legacy main/CLI DB、legacy JSON/log home 都要捕获；不能只替换 archive 文件 |
| `state.rs:232,331,398` | 偏好 load/quarantine 从 HOME；异步 save 又重新 `default_path()` | State 保留捕获的偏好路径；load/quarantine/save 同路径，线程不得重新解析 HOME |
| `main.rs:192` → `desktop_shell.rs:116` → `commands/codex_tray.rs:39` | 启动 tray refresh 使用 `CodexAuthService::new()`；即使 force=false 也可能 `fetch_current_quota()` | 使用捕获 auth/cache 投影；fixture 禁止远端 quota/refresh，不把 force=false 当无网络证明 |
| `main.rs:205` → `commands/codex_auth.rs:589` | 自动恢复 OAuth pending store/listener；通过 runtime_policy 进入恢复 owner | fixture 不能自动恢复 listener；不能读取 host pending 文件或绑定真实端口 |
| `main.rs:211`、`commands/environment.rs:86` | 启动及显式刷新均探测 WSL；LocalEnvironment 读 dirs/home/from_env | fixture 注册捕获 Local；环境刷新复用同一 fixture policy，拒绝 WSL/真实 SSH |
| `main.rs:476` | 定时维护解析 `CostTracker::default_storage_dir()` | 使用 captured cost root；不是取消维护来降低 fixture 空闲 CPU |
| `services/agent_sessions.rs:288,323,345` | detail/provider/refresh 构造 `AgentSessionProviderRegistry::from_default_home()` | registry 用 captured provider home 并 `without_environment_overrides()`；archive 池隔离不等于正文隔离 |
| `commands/config.rs:106`、`claude_auth.rs:386`、`codex.rs:125,350,480` 等 | commands 自己 `with_default`/`new`/`dirs::home_dir`，并非都通过 AppState | 逐条调用链捕获或在 dispatch 前拒绝；ReadOnly descriptor 不能证明无写入 |
| `commands/checkin.rs:345` | 名为 list 的操作会 backfill builtin IDs；manager 默认 global，账户 crypto 另有目录 | DB 与 crypto root 都捕获；读/写分类不能代替审计 |
| `bridge.rs:40,57` | 日志 bridge + 每 2 秒 flush；LogPersistenceService 默认 global DB | 保持真实 bridge/flush 生命周期，确保 global/State 是同一 fixture 池 |

上述行号是研究时工作区来源，后续并行变更可能移动行号。

## 2. WebView 隔离必须早于 setup

实际锁文件为 **Tauri 2.11.5**。本机已安装源码
`.../tauri-2.11.5/src/app.rs:2521` 的 `setup` 先遍历 config 中 `create=true`
的窗口并 build WebView，然后才调用项目 `.setup` 闭包。因此在现有项目
`.setup` 内设置主窗口 data directory 已经太晚。

推荐 fixture 分支：在构建 app 的 Context 上保留原 main WindowConfig，但令
自动 `create=false`；setup 先完成隔离 State/IPC policy 注册，再用
`WebviewWindowBuilder::from_config(...).data_directory(absolute_fixture_path)`
创建同一个 main 窗口。默认分支保持原自动创建行为。不要换页面/删 UI。

同一 builder policy 必须覆盖：

- `desktop_shell.rs:358` 新建隐藏 tray panel；
- `desktop_shell.rs:592` 主窗口销毁后的重建；
- `commands/waf.rs:286` 的外部登录 WebView：fixture 明确禁止该外网操作，
  不能遗漏后悄悄使用默认 profile。

Tauri 2.11.5 的绝对 `data_directory(PathBuf)` API 已在本机源码确认。
`tauri-utils-2.9.3/src/config.rs:2218` 则说明配置字段 dataDirectory 相对
appDataDir/label，不能直接假定写入 config 的绝对路径等价于 builder API。
Windows/Linux 可据此设计目录隔离；**未运行原生验证**。

macOS 的 dataDirectory 不支持；另有 dataStoreIdentifier，系统要求 macOS
14+，其存储也不是任意 fixture root 目录。项目现有最低 macOS 是 10.13。
因此首个 fixture 模式必须明确 target support；不能把 Windows 方案静默
推广到 macOS 或伪称全路径 root-contained。可在未有可靠 WebKit 方案时于
logger/window 创建前拒绝该 target，保持该平台 not_run/unsupported。

## 3. 推荐显式边界

建议新增 `src-tauri/src/fixture.rs`，boot 输入为单一 opt-in
`--fixture-root <absolute-existing-root>`（名称是提议，尚未实现）。
正常启动没有该参数，行为保持不变。

`BootMode::{Normal, Fixture(Arc<FixtureLayout>)}` 是进程启动时唯一冻结身份，
不能运行中改 root。FixtureLayout 只含已验证的路径和 policy，不含凭据。
manifest 的所有相对目标要解析为绝对路径并验证实际 canonical containment；
拒绝 traversal、根/祖先/目标的 symlink、Windows reparse/junction、冲突的
重复路径、缺 marker、版本未知、非目录及不可读文件。先验证再创建受控子目录。
可参照已有 native fixture 的 canonical 现存父目录检查，不能用字符串
starts_with 或锁目录的大小写归一化身份冒充实际 containment。

路径至少覆盖以下稳定角色：

```text
fixture root/
  manifest.json             # version、fixture identity、各 role 路径/数据哈希
  home/                     # provider source home + 受控 legacy HOME 视图
  ccr/                      # platform catalogs、registry、history/cost/sync
  ui/ccr-ui.db              # 当前 main DB，global/State 同一池
  ccr/analytics/usage.db    # archive，保持独立 DB
  llmusage/                 # 上游已生成 SQLite，只读投影
  logs/                    # logger 创建/清理范围
  preferences/             # desktop-shell.json + corrupt quarantine
  locks/ backups/ crypto/   # 所有 guarded writes/credential encrypt owner
  webview/main/             # WebView localStorage/IndexedDB/cookies/cache
  webview/tray/             # 每窗口 storage 角色，按 locked engine 规则核查
```

这是角色布局建议，seed 阶段决定实际 manifest schema；不要为了复用 native
旧测试各自的 feature 临时目录而复制真实用户数据。两端 S0/S1/S2 应用同一
逻辑数据/来源哈希，不能误把当前 GPUI 单测的多个独立 fixture 子根当作已经
可比的整应用数据集。

启动顺序应为：纯参数解析/布局验证 → 显式 logger → 显式 catalog migrations
与 main/archive migrations → 同池 global 注册 → captured AppState（prefs/
registry/auth/provider roots）→ IPC policy → captured main/tray builders →
正常 bridge/maintenance。任何必需 fixture role 缺失/不安全均终止启动，不能
调用 Normal resolver。

## 4. 可直接复用的 owning API

- `ccr_cli::application::platform_roots::PlatformRoots::fixture`：纯构造，明确
  home/ccr/Claude/Codex/Grok/locks；`claude_auth()` 与 `codex_auth()` 已提供
  captured auth owner。不能先 `capture()` 再覆盖其中一两个字段。
- `ProfileLifecycleContext::{for_claude,for_codex,for_grok}`：已有显式
  PlatformPaths/runtime/locks/history 构造；`run` 在实际同步操作线程绑定
  locks。参考 `ccr-desktop/src/runtime/profiles.rs:180`，不复制其 UI adapter。
- `ClaudeRuntimePaths::resolve_with(home, |_| None)`：禁止继承 ambient
  CLAUDE_CONFIG_DIR 等变量；同样检查其他 provider 的 env override。
- `LlmusageRuntime::from_paths(AppPaths::from_root(...))`：不创建/迁移上游 DB。
  它仍包含真实 CLI 边界，**不能因此放行 fixture import**；导入网络/进程
  另由 closed policy 拒绝或后续明确受控 synthetic executor。
- `ccr_db::database::create_pool` + `migrations::run_all_migrations(conn, home)`
  + `migrate_usage_archive_from_legacy_dbs(conn, home, legacy_ui_path)`：均已有
  explicit arguments；不要复制 schema/migration SQL。
- `DbAccess::Pool`、`UiStateManager::with_db`、checkin
  `AccountManager::with_db(crypto_dir, db)`/`ProviderManager::with_db`、
  `CheckinService::with_db_and_client`：现成的 captured consumers。
- `AgentSessionProviderRegistry::new(provider_home).without_environment_overrides()`
  与共享 `ccr_app::sessions::{get_detail_with_registry,refresh::refresh_archive_with_registry}`：
  原 source validator/pagination/正文规则保持。
- `DesktopShellPreferences::{load_or_quarantine_corrupt_at,save_to_path}`：
  已有显式路径；State 要保留它供 detached save 使用。
- `LockManager::with_scoped_directory`：只在实际 blocking closure 内生效，
  不跨 await 或 spawn 自动传播。catalog migration 同样需要它。

## 5. 最小基础批次文件与需先协调的 shared seam

| 文件 | 必需改动 |
| --- | --- |
| 新 `src-tauri/src/fixture.rs` | boot/layout 验证、closed policy、纯构造与路径/启动回归；不是新的领域服务 |
| `src-tauri/src/main.rs` | before-logger 顺序，显式初始化，冻结身份和已捕获 Local/后台，主 WebView 创建顺序 |
| `src-tauri/src/state.rs` | captured roots/preferences path 构造和异步保存；Normal wrapper 保持兼容 |
| `src-tauri/src/platform/local.rs` | 从显式 roots resolve 读/写/backup，fixture 不 lookup host CLI |
| `src-tauri/src/desktop_shell.rs` | main/tray 创建及重建共用 storage policy，tray auth/cache 从 captured owner 读取 |
| `src-tauri/src/commands/handler_registry.rs` | dispatch 前按真实 command ID 检查 fixture policy，保持现有 ACL/confirmation/audit |
| `src-tauri/src/commands/runtime_policy.rs` | Rust 直接调用的后台入口也执行 fixture admission，不仅前端 IPC |
| `src-tauri/src/services/agent_sessions.rs` 和 command adapter | 已有 registry-aware helper 提升可用性，传 captured provider registry |
| 各平台 command modules | 按下节逐域消费捕获 owner；不能把所有 ::new 留原样 |

这不是仅 main/state 可完成的范围。以下三个 shared owner seam 需要协调授权后
才改；建议只增加显式入口，由现有 Normal wrapper 调用同一实现：

1. **ccr-core logging**：显式 log-dir 初始化接口；仍使用原脱敏、SecureDailyWriter、
   bridge、权限与 guards，不创建 Tauri 私有 logger 或修改全局 env。
2. **ccr-db database**：显式 main/archive initializer，并把同一 main Pool 注册
   到 GLOBAL_POOL。现有 `initialize_app_pool` 忽略 OnceLock.set 失败，显式
   initializer 应拒绝已绑定别的池/路径，不能返回与 global 不一致的 State。
   默认调用者 API 保持；测试用子进程或独立 inject pool，避免重置全局单例。
3. **ccr-config catalog migration**：公开窄的 captured-PlatformPaths 迁移入口，
   复用现有 private migrate_paths；不绕过 legacy retirement/backup/锁。

`commands/{grok,settings_raw,shell,system}.rs` 有其他批次在途修改，本研究没有
编辑它们。后续 fixture owner 接合这些文件应先与当前 owner 协调；锁住这些
文件不意味着其中 ambient resolvers 已完成隔离。

## 6. 未审计 fallback 的封闭策略与比较资格

现有 `generate_handler` 宏两平台分支在 `audit_invoke` 后统一 dispatch，适合
加入 fixture gate；**不能用 CommandRisk::ReadOnly 自动放行**，如 list_providers
本身写 backfill，session provider status 读取真实 HOME，tray get 可能联网。

按 registry command ID 维护 fixture disposition（captured-safe、pure、外部
能力显式拒绝、unreviewed）。缺条目/unknown 必须在 handler body 之前拒绝，
不给 unreviewed payload 任何 host side effects。直接 Rust 后台入口还需相同
admission，guard 不能只存在于 React API/IPC 层。

最小交付顺序：

1. 先完成基础初始化与全部 registry ID 的 fail-closed 覆盖测试；此时仅称
   **隔离基础**，不提供可比较 benchmark boot 收据。
2. 逐域接合本地 core graph：profiles/status/edit、platform settings/auth 文件
   观测、MCP/resources、usage dashboard、session archive/provider/detail、UI
   favorites/preferences、monitoring。使用同一真实领域 owner 和 fixture 数据，
   保持 UI composition、虚拟列表、缓存、任务、bridge 与 maintenance。
3. fixture 对网络/quota/OAuth/WSL/SSH/真实 CLI/install/update/WAF/external open
   明确拒绝，不能发真实请求或用假成功替代；这不阻止本地业务能力工作。
   逐个确认每个直接后台 seam 也没有外部调用。
4. 用 benchmark scenario 的**必需 IPC/动作集合**验证实际支持，若其中任何
   被 unreviewed/拒绝，baseline 标 not_comparable。不能关掉全部 IPC、拿空
   dashboard/错误页/无维护的壳声称“整应用”。

因此当前没有可安全直接启用的 fixture boot，且不存在只改环境变量就能跳过
这些步骤的实现捷径。真正 implement 应先拿 shared seam 所有权，再按上述
本地 core graph 扩展，维持未审计入口拒绝。

## 7. 可测成功条件

- 无 opt-in 时解析为 Normal，默认构造/命令行为不变；非法/缺少 root 在
  logger、migration、window、background owner 之前失败。
- 两个 fixture root 各放不同 main/archive/catalog/provider/prefs/legacy 哨兵；
  通过真实入口读写 A 不会看到 B，legacy migration 仅消费 A 的 legacy home。
  日志创建/清理、prefs quarantine/异步 save、crypto/backup/locks 均只在 A。
- 子进程安装生产 boot identity，构造 schema/pools，验证 global 与 State
  查询同一 DB；重复/冲突 install 拒绝。不要并行测试里修改 HOME，保留默认
  测试并行和 `--skip export_bindings`。
- 以 injected denial/recording executors 验证 startup 不恢复 OAuth、不探测
  WSL/SSH、不查询 quota、不启动 installed CLI；真实阻塞由 owner 控制，不能
  单靠一个 mock main 计划断言所有领域已隔离。
- registry 完整覆盖；unreviewed/外部命令在 body 前拒绝；至少一个真实写入、
  CAS 冲突/备份和一条真实 usage/session 投影从 fixture core graph 走通。
- 主窗口/tray/rebuild 使用同一存储策略，headless 检验 builder 参数只能证明
  配置传递；实际 WebView localStorage/cookies/IndexedDB 隔离仍需之后 native
  收据，本轮用户暂缓，记 not_run。
- 不发明 first-present clock。process-create/first-present/action-ready 观测与
  S0/S1/S2 seeds 是后续独立工程；main 时间、JS paint、on_next_frame 都不冒领
  presented。先完成隔离后再评估可比较性与样本。

## 8. 本研究验证边界

已读取 task PRD/design/implement、最新 handoff、performance 合同、before-dev、
root/ccr-ui code_map/AGENTS、shared guides，以及相关 DB/IPC/usage specs。
本机锁定 Tauri/API 源码检查如上；没有使用网页版本猜 API。

本轮只读源码、写此研究文档：lint/type-check/tests/native/performance **not_run**。
这不是实现验收或性能 PASS。没有改 main/state/shared owner、global env、命令
API、schema、CI 阈值或用户配置。
