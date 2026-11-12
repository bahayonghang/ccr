# GPUI Claude Code #161 实施清单

主规格：https://github.com/bahayonghang/ccr/issues/161 。设计已批准，原生验收与实现分别记证据。

## 基线（2026-11-10）

- 分支 `dev`；原有修改：`.trellis/workspace/lyh/{index.md,journal-3.md}`、`ccr-desktop/{CONTEXT.md,Cargo.toml,Cargo.lock}`；未跟踪 `2026-11-10-103942-pastedcontent-id668f.txt`。保留，不提交、不清理。
- 原始 diff 保存在忽略目录 `ccr-desktop/outputs/claude-code-redesign/implementation/baseline.patch`。
- `just version-check`、`just fmt-check`：通过。
- #150/#151/#156 均 open，源码仍有相关缺陷；#155 是研究，不计实施缺陷。

## Owner / 行为 / 验证

| 阶段 | 文件 / owner | 行为 | 验证命令与状态 |
| --- | --- | --- | --- |
| P0 | ccr-usage/db；ccr-app/resources；native resources | 成功 rebuild 纳入全局新鲜度；资源与记忆文件使用 Claude 配置根；工作台数字用系统字体 tnum | implemented；ccr-usage 66 测试通过；配置根覆盖回归通过 |
| P1 | shell/platform_navigation、page_header；MCP、insights | Claude 九 tab、唯一 h1、分组无障碍名称、局部横滚/宽度变化 reveal、持久焦点；真实 MCP source/target 与会话废纸篓入口拒绝 | implemented；scope bypass/global 合法操作单元通过；desktop-test-ui 13+2 通过 |
| P2 | ccr-app/config、usage/observer；native adapters | 缺键/保存默认值/读取错误、存在性 CAS；未知次数 Option；导入默认保留、显式覆盖、替换预备份、revision/当前项重验 | implemented；shared subscription/config/profile owner 测试通过；显式导出新文件、不覆盖/不越界回归已加入 |
| P3 | features/workbench；runtime/insights/claude_workbench | 同一报告时钟/平台/时区；三 KPI、30 日历柱图、全量精确值、模型、内联订阅、Token/缓存；缺失不补零、输出含推理不重复 | implemented；固定时钟缺失/零/缓存/来源隔离与格式测试通过；新原生像素 not_run |
| P4 | features/configs、profiles、platforms、resources | 单一集合；60px 虚拟行、搜索/筛选/排序、3:2（列表 ≥360、详情 ≥320）、窄窗 drill-in；编辑与集合工作面；保存/导入/恢复不应用且保留实际 current、未知字段/空凭据保留、草稿离页确认与 generation；指令与扩展按平台固定（用户级作用域与未接入状态已写明） | implemented；当前 A/选中 B、解除 A/应用 B、脏草稿双向 Tab/Esc/返回触发者界面测试通过；300 单元通过；当前项保存不改变运行文件回归通过 |
| P5 | ccr-app/resources；ccr-skills；native resources | 真实用户级子代理/技能/旧命令/插件登记只读来源；复用 Store 与 Skill 元数据解析；分开 missing/empty/unreadable/invalid/unsafe/limit/unconnected；旧数组保留 | implemented；只读、部分失败、超限、技能版本与旧数组字节稳定测试通过；其他平台 CRUD 回归通过 |
| P6 | DESIGN、三个真实 spec、证据 | 生产 DESIGN 已落实批准合同；配置/共享服务/用量新鲜度合同已同步 | in_progress；最终 desktop-ci 通过；just ci 在既有审计失败；原生工具阻塞，不能宣称验收完成 |

## 验证记录

- 基线：`just version-check`、`just fmt-check` 通过。
- `cargo test -p ccr-usage --features test-fixtures --locked -- --skip export_bindings`：66 通过（另有契约测试）；包含成功 rebuild 后新鲜度更新、失败不推进。
- `cargo test -p ccr-app --lib --locked -- --skip export_bindings`：202 通过、4 ignored（当时版本）；新增资源、导出、配置根测试有单独日志，最终 gate 继续覆盖。
- `cargo test -p ccr-app --test config_read --test profile_services --test resources --locked -- --skip export_bindings`：相关 owner 集成通过。首次 shared read 字节断言出现一次失败，单测与完整 lib 重跑通过；未归因、未删除测试。
- `cargo clippy -p ccr-app --locked --all-targets --all-features -- -D warnings -D clippy::unwrap_used` 通过。
- GPUI 完整 lib：301 通过、4 ignored。新增当前项保存不应用测试已在最终 gate 通过。
- `just desktop-test-ui`：13+2 通过、1 ignored（夹具准备）；配置方案选择/应用/解除、离页确认真实控件路径已包含。
- `just desktop-build`：新 release 构建通过；最后生产调整后 `just desktop-ci` 完整通过并重新构建。
- 失败已修正：编辑字段被普通预览挤压；状态读阻止列表方向键；确认触发按钮卸载导致焦点丢失；旧 UI ID 与新合同不符。未降低阈值、未改变默认并行、未删失败测试换绿。
- 隔离夹具准备：ignored `prepare_native_workbench_fixture` 通过，仅创建 `outputs/desktop/claude-161-native-20261110` 的合成方案、用量与扩展文件；未读取或修改真实账户、凭据、方案。
- 详细输出在忽略目录 `ccr-desktop/outputs/claude-code-redesign/implementation/`；最终结果会在此补齐。

## 原生验收

四个指定场景均 **not_run**：1280×800 浅色中文、2000×1080 浅色英文、960×640 深色英文（Codex 导航）、1280×800 深色中文。

Computer Use `@oai/sky.list_windows()` 返回：`Computer Use native pipe is unavailable: failed to connect native pipe: The system cannot find the file specified. (os error 2)`。初次失败后一次有界恢复（重试、重置 JS kernel、重新初始化并重试）仍同一错误。没有可绑定窗口，未进行 UI 输入或截图。外窗、客户区、DPI 和截图对应关系均未观测，不能从文件名或期望尺寸推断。未修改真实偏好；夹具尚未用于原生交互。未上传任何图片。HTML/headless/编译通过均不替代此验收。#161 保持 open，#138 与设计票保持原关闭状态。

## 稳定 ID 实施记录

- 保留原含义：`tool-home/profiles/auth/platform-mcp/platform-usage/platform-sessions/providers/models/files`、`configuration:*` 和单项 `profile-*`。
- 迁移：`tool-accounts` → `tool-settings`；Claude 合并为 `tool-extensions`；删除旧 agents/plugins/slash/subscription tab 与 `workbench-*` 重复跳转。
- 本地订阅：`workbench-subscription-edit/save/cancel/field-*`；图表 `claude-cost-bar-*` 与工作台精确值切换；不复用旧导航 ID。
- 新增配置筛选：`configuration-filter-clear/filter-*/provider-*/tag-*/sort/manage`；行内 `configuration-apply:<name>`；`profile-import-preview/overwrite-*/commit`；显式导出 `profile-export-save-file`；离页确认 `profile-navigation-confirmation/cancel/discard`。
- 平台嵌入隐藏跨平台资源/设置/用量导航，全局原 ID 维持用途；旧 policy/clean-backups/clear-history/source-force 不出现在新方案页。

## 最终 gate 的既有失败与独立补验

- `just ci` 已运行：版本、格式、OMP/harness/Copilot、严格 Clippy、workspace check、根测试、release 全部通过；安全审计失败（8 项 openssl 0.10.76 漏洞；另有 warning）。根 `Cargo.lock` 和 `ccr-ui/src-tauri/Cargo.lock` 与 HEAD 完全一致，未修改。单独 Tauri lockfile audit 同样失败。按本轮禁止无关依赖升级的约束，未改 lockfile、未忽略漏洞、未降低 gate。
- 审计后的独立检查：`just frontend-check`（含 docs）通过；`just frontend-audit` 通过（既有 2/2 active exceptions，未改）；`just frontend-coverage` 通过（Statements 73.15%、Branches 59.98%、Functions 72.03%、Lines 75.4%，阈值未改）；`just vscode-ci` 通过。
- `just ci-governance-check` 首次使用 Windows System32 的 WSL bash，因缺失 Docker ext4.vhdx 而失败；仅为该命令进程前置现有 `C:\Program Files\Git\bin` 后通过，未改仓库脚本或系统配置。
- Tauri config contract 曾因 shared `usage_count: None` 与既有 wire 的 0 默认值不同而失败。生产 adapter 保留 Tauri 原输出，测试显式验证其默认映射；native 仍保留未知，未改变 wire 或绑定。完整 `just tauri-ci` 已通过（334 normal tests、2 SQL guard、200 binding exports、inventory；阈值和绑定未变）。
- P4 收尾：字段区独立滚动，保存/确认/取消留在字段前；精确值保留实际浮点精度，空窗口在精确表同样不补零；切平台清掉旧平台集合事实，解除动作在当前状态读完成前不可用。最终 `just desktop-ci` 完整通过。
- P3 新状态回归通过：缺键显示未设置；明确保存默认值显示自动判断；精确值表和内联字段可构造；无效月费保留草稿；离页递增 generation 并清报告/订阅/在途订阅。新增 Claude 与编辑流程被加入既有 Windows 1 MiB 子进程回归。

## 集合写入的当前项保护收尾

- 按 #161 目标，native 已删除 `profile-source-force` 和强制源码移除命令意图。`source::save_without_activation` 在平台应用锁内保留实际 current 标记；导入提交同用该策略。`backups::restore_without_activation` 先预备份、再按 revision 保存，拒绝移除当前项，不从备份伪造一次应用。旧 shared/Tauri 的明确 force 与 restore 接口保持兼容。
- 新 shared 回归 12 passed / 1 ignored：文档/备份伪造 current=B 时仍保持实际 A；移除 A 返回 ActivationConflict 且 revision 不变；stale 仍 Conflict；未知字段保留。
- 新 native owner 回归 8 passed：单项编辑、集合 TOML 保存和恢复均不改变运行文件字节；实际当前项保持 alpha，未知字段/凭据保留，集合移除当前项被拒绝。
- TOML 文档用 `toml::from_str::<toml::Value>`；本版本 Value 的 `str::parse` 解析的是值，不能替代文档解析。该错误在新回归中定位后已修正。

## 最终验证状态

- 最后生产代码后的 `just ci`（仅该进程 PATH 选 Git Bash）再次运行：版本/格式/OMP/harness/Copilot、Strict Clippy、workspace、完整 Rust 测试、release 均通过；约 6 分 12 秒后仍在既有安全审计失败。未修改根/Tauri lockfiles。
- 最后生产代码后的 `desktop-ci-verified.log`：格式、所有 target check、严格 Clippy、Python/AccessKit 补丁、完整 GPUI lib（301 passed / 4 ignored）、native_shell（13 passed / 1 ignored）、native_ui（2 passed）、runtime_shutdown（2 passed / 1 ignored）均通过。既有 1 MiB 子进程包含新 Claude 完整报告/订阅分支与方案编辑流程，已通过。最终 release 构建通过（5m 08s），`just desktop-ci` exit 0。
- `just tauri-ci` 已通过：334 normal tests / 5 ignored / 200 filtered（由独立绑定 gate 处理），guard、binding exports 与 inventory 全通过。Tauri wire 行为与生成文件均不变。

## 交付状态与构建身份

P0–P5 生产实现与相关回归已落地。P6 的生产 DESIGN、真实合同及可独立执行 gate 已完成；**原生四场 not_run 与既有依赖审计失败仍使 P6 / #161 未验收完成**，不以代码完成冒充发布或原生通过。

- 最终 release：`ccr-desktop/target/release/ccr-gpui.exe`；SHA-256 `93be9b5554c44cda83efbef41fe1077d5b5d3cd9602cf1ee28cf3282b2eb6b0d`；54150144 bytes。
- 本地证据：`build-identity.json` 与 `native-acceptance.json`。无新原生截图，外窗/客户区/DPI 均 null。
- 原有 `ccr-desktop/Cargo.toml`、`Cargo.lock`、`CONTEXT.md` 与 baseline.patch 的 diff 逐段比对相同；session 记录前原 journal/index 的 diff 也相同。未跟踪 pastedcontent 文件保留。journal/index 仅按 mini-trellis 新增本次实际交接记录。
- 不 commit、push、发布或上传截图；本会话未关闭 #161，未改 #138 或设计票。

## 续做核对与补齐（2026-11-10，Session 167）

本节是对 Session 166 的增量，前文的失败与 not_run 收据保留。开始时仍为 `dev`，HEAD `4696c36c5315f92f62afa6129c38889fd419e575`，65 个变更路径；这些是继承工作区，不是本轮新增。完整开始状态、文件 SHA-256 与 diff 在忽略目录 `implementation/continuation-start.{json,patch}`。

### 本轮实际修改

- 配置列表的空态此前仅检查搜索文字，供应商/标签/启用状态筛掉全部行时误报空集合。改为依据已读取的完整集合判断。新控件回归先失败（实际 `This catalog has no profiles`），修复后通过；同时覆盖窄窗详情返回保留选择/供应商/标签/焦点、筛掉所选项清除选择与行内应用、清除筛选恢复集合且不自动选择。补齐当前方案文字旁的既有 `badge::dot`，不新建组件或 token。
- 新增 960×640 Codex 控件回归，依次通过 11 个 tab：键盘焦点、h1、选中项在局部滚动区内可见及循环返回。它是 Kit headless 控件证据，不能当作指定深色英文原生验收。
- 首次 `desktop-ci` 有一项既有关闭测试失败：退出保存读回 1280 而非 1320，同时输出排队保存取消诊断；其余 301 通过、4 ignored。单项原样复现通过，保留两份日志。等待助手按忙等次数推进虚拟时钟，而文件写入在真实 Tokio worker，存在提前耗尽两秒虚拟预算的时钟竞争；仅修复该助手按实际经过时间推进，不改生产两秒预算、测试十秒上限或默认并行。关闭 owner 6 项通过；完整重跑另记最终结果。
- `DESIGN.md` 补明空集合/筛选无匹配的区分、过期选择清理和清除筛选不选择/应用。其他已批准设计和共享消费者策略不变。

### #161 必查项的实际执行边界

| 必查项 | 核对的生产入口与回归 | 证据边界 |
| --- | --- | --- |
| 九 tab / 其他平台能力集合 / 唯一 h1 / stable ID | `shell/platform_navigation::platform_items`、`page_header::LensBar`；exact-platform-capabilities、native_shell 的 platform workbench 与新 Codex 键盘测试 | 源码与控件；指定四场原生仍 not_run |
| 宽窄导航与选中 reveal、持久焦点 | 同一 shell-owned LensBar、ScrollHandle、宽度变化 reveal 与目的地/lens 焦点检查；新 960 Codex 测试逐项验证 | 没有用 headless 冒充原生尺寸/DPI |
| MCP 实际 source/target 权限 | editor/copy prepare+save 重验 captured `platform_scope`；batch prepare/save 拒绝平台上下文；`platform_authority_rejects_forged_batch_and_global_plan_without_writes`、原全局批量/copy/CAS 测试 | 不靠 RequestKey.platform 授权，全局原动作保留 |
| 非 Codex 废纸篓拒绝 | session_operations 与 delivery 的读、prepare、执行前 `codex_actions_allowed`；`captured_non_codex_scope_rejects_trash_and_prepared_commands_without_side_effects` | 保留全局、Codex、Local、版本、Busy 与二阶段 owner |
| 平台用量/会话固定来源 | `InsightsRequests::read` 重验 source/family 和 detail 实际来源；import admission 验证 options.source；平台 import 控件回归 | refresh、切页不等于导入；全局入口保留 |
| 订阅缺键、默认值、读取失败、CAS | shared `read_saved` / `save_saved_if_current`，native observer 与工作台 subscription_state；缺键/明确默认值/错误/CAS/草稿回归 | Tauri 历史默认 wire 维持原 adapter |
| 今日无记录与真零、金额精度 | claude_workbench 的 captured ReportClock 与 Claude-only owner 投影；固定时钟缺日/零、subcent、精确值控件测试 | 不将无库/不可读补零，不推断未使用 |
| 缓存与推理 | 同一 30 天 recent，`cache_ratio` 零分母 None，output 直接使用兼容 trend 字段；固定数据 30+15=45、不再次加算；total 使用 owner 字段 | 不使用全历史缓存，首页不加入 #152–#154 模块 |
| 当前 A / 选中 B | ConfigurationApp 与 ProfileTools target 分离，Off 由生命周期 owner 读取 current；native_shell 选择/解除 A/应用 B 控件回归 | 选择与方向键不应用；禁用项明确启用并应用 |
| 保存/导入/恢复不应用 | native ApplyPolicy::Never、`save_without_activation` / `restore_without_activation` / import::commit；shared 12 项及 native 8 项当前项/运行文件字节回归 | 文档/备份不能伪造 current 或移除当前项；Tauri 旧 force 接口兼容 |
| 凭据/未知字段/错误草稿/revision | dirty-key patch、空凭据不发送、原始 revision、typed SourceSaveOutcome；profile editor 与 profile_services owner 回归 | 冲突不丢草稿、不强行覆盖 |
| 导入同名与 stale preview | `config/import::{preview,commit}` 的显式 overwrite、提交原 revision 与当前项重验；reviewed_import 回归 | 同名默认保留；替换预备份；失败无覆盖 |
| 确认/离页/晚回包 | ProfileTools 的 post-mount generation、FocusTrap 与返回焦点；shell navigation_guard；外部焦点、初始取消、双向 Tab、Esc、离页清理测试 | 已开始写仍归 app owner；原生焦点没有通过收据 |
| 列表/空 pane/窄窗返回 | 60px uniform_list、3:2、内容高度与 max_h、detail_open、持久 scroll/focus；新筛选返回测试及原虚拟长列表测试 | 没有扩大显示器/DPI 矩阵 |
| 扩展只读 | `claude_extensions::discover` 仅捕获路径上的有界读，Store/Skill parser；metadata 状态、unsafe/limit、部分失败及旧数组字节稳定测试 | 不写、不联网、不下载、不执行，不把发现称客户端加载 |
| 账户/设置/指令及其他平台 | PlatformsView pinned/surface、资源 open_platform/open_extensions 与已有文件 owner；native_shell 平台文件测试和原 settings/resources/Codex/OpenCode/Grok 回归 | 保留真实文件 CRUD、Grok 多阶段/CAS 与 Tauri 共享接口 |

### 最小依赖安全修复

- 本轮重新 fetch RustSec 数据库成功：revision `28d73fc95e5972af3403225a217def57dcaba3c4`，1446 advisories。根 OpenSSL **0.10.76**、Tauri **0.10.75** 各命中 8 项：RUSTSEC-2026-0463、0473、0474、0475、0476、0477、0478、0479；不是假设两份 lockfile 原版本相同。
- 定向修复为 `openssl 0.10.80`，它要求 `openssl-sys >=0.9.116`；再定向选择最低必要 `0.9.116`。Cargo 最初自动选出的 0.9.118 已收窄，未交付无关升级。manifest 均不需要修改；桌面原有 Cargo.toml/Cargo.lock 不动。
- 修复下限的上游依据：[RUSTSEC-2026-0477](https://rustsec.org/advisories/RUSTSEC-2026-0477.html)、[OpenSSL 0.10.80 manifest](https://github.com/rust-openssl/rust-openssl/blob/openssl-v0.10.80/openssl/Cargo.toml)。其余受影响范围以本轮原始 audit JSON 为准。
- 两份锁各只改这两个 package，包数保持根 495、Tauri 678；记录 before/after checksum/依赖列表，直接下载 crates.io 静态归档计算 SHA-256，与 lock checksum 相符。收据：`continuation-dependency-integrity.json`、`continuation-dependency-compatibility.json`。两包声明 rust-version 1.80，未改项目 MSRV；本机没有 1.95 toolchain，未宣称实跑 MSRV。
- `--target all` 依赖链为 openssl ← native-tls ← hyper-tls ← reqwest 及 shared consumers；实际 Windows 默认图 `cargo tree -i openssl` 无活动节点。锁审计覆盖不等于 Windows 运行可达性，也不等于 Linux TLS 运行验收。
- `just audit` 两份锁均 exit 0；原有 unmaintained/unsound warnings 保留（根 3、Tauri 12），未加 ignore、未降低门槛。本轮最小授权只处理阻塞漏洞。
- 原始审计 JSON、依赖链、更新尝试、修后 audit 日志均在既有忽略 evidence 目录，未覆盖上一轮收据。

### 原生工具和夹具复核

已读当前 computer-use skill 和 guidance/API/confirmations。本轮 `@oai/sky.list_windows()` 初次、重试、重置 JS kernel 后重新 import 并重试，均为同一 `native pipe ... os error 2`。一次有界恢复已结束；未绕过工具自建协议或用 HTML/headless 替代。

既有 `outputs/desktop/claude-161-native-20261110` 的合成 sentinel 与 fixture.json 已核对；owner 是 `native_shell::prepare_native_workbench_fixture`，拒绝覆盖已有目录，要求 outputs/desktop 下绝对路径。fixture registry parent 绑定隔离 preferences、profile/platform/resource/MCP/observer roots；用量等三个显式参数仍须成组传入、无 home 回退。本轮未启动或覆盖夹具、未操作真实账户/凭据/方案、未修改真实偏好。

四场仍全部 **not_run**，外窗/客户区/DPI/截图均 null，输入与新增截图 0。准确收据 `continuation-native-acceptance.json`；新 release 身份在最终构建后另记，不能沿用旧哈希冒充新候选验收。#161 保持 open；#138 与设计决策票未修改。

### 本轮最终门禁

- `just desktop-test-ui`：14 shell + 2 input 通过，1 个显式 fixture preparation ignored。最终 `desktop-ci` 再次覆盖相同 UI 集合。
- `just desktop-ci`：最终 exit 0。格式、所有 target check、严格 Clippy、16 个 Python 测试、AccessKit 断言、GPUI lib 302 passed / 4 ignored、native_shell 14 passed / 1 ignored、native_ui 2 passed、runtime_shutdown 2 passed / 1 ignored 均通过，含 Windows 1 MiB 子进程；release 重新构建成功（3m46s）。首轮失败日志保留，未删除测试或降低门槛。
- 新 release：`ccr-desktop/target/release/ccr-gpui.exe`，54151680 bytes，SHA-256 `7aae250732d959131ccd8643752eb07bfc2b55b2531e854e772838c0f7c3edb3`；`continuation-build-identity.json`。源码含继承改动，不是纯 HEAD 构建；该哈希对应候选文件，**无原生截图对应**。
- `just ci`：exit 0，23m19s，16 个阶段全部通过：版本、格式、OMP/harness/Copilot、Strict Clippy、workspace、完整根测试、release、两份锁 audit、CI governance、Tauri CI（334 normal / 2 guard / 200 exports / inventory）、frontend-check（含 docs）、frontend-audit、frontend-coverage、vscode-ci。只在该命令进程的 PATH 前置现有 Git Bash，未修系统 bash/WSL。
- `git diff --check` 通过。代码与自动门禁已完成；P6 / #161 **仍未验收完成**，四场真实原生必须在工具恢复后用此新候选与隔离夹具补做，不能以本次全绿关闭。

### 保留与交接

- 桌面原有 `Cargo.toml`、`Cargo.lock`、`CONTEXT.md` 的字节与本轮开始时一致，且逐文件 diff 与最初 baseline.patch 一致；pastedcontent 字节一致。仅在已授权文件上增量修复，未 reset/checkout/stash/清理，未 `git add -A`，未 commit/push/发布/上传截图。
- 最终状态与保留核验另存 `continuation-preservation.json`；journal 仅追加本轮 Session 167，原字节前缀保留，index 旧会话记录保留。
- 本轮新增修改面为根/Tauri 两份安全 lockfile、配置列表与回归、Codex 导航回归、既有关闭测试时钟等待助手、生产 DESIGN 与本实施清单/交接。Session 166 的其余源码仍保留，不全部记作本轮新增。

## 继续收尾：MSRV 与延迟导航焦点（Session 168）

### 重新核对

- 开始仍是 `dev`、68 个变更路径，HEAD `4696c36c5315f92f62afa6129c38889fd419e575`；release 仍为 `7aae2507…c3edb3`。本轮开始状态与逐文件哈希保存在 `implementation/msrv-start.json`，旧 journal/index 另有字节快照。
- 当前 `mcp__cua_repl` 的 `cua.getState()` 原生清单报同一 `native pipe ... os error 2`，重置 kernel 后再次检查仍失败；浏览器清单同时报 host unavailable。一次有界恢复结束，没有输入、启动夹具或截图。四场原生仍 **not_run**；具体回包与边界记在 `msrv-native-recheck.json`，不把 UI 控件测试当原生。

### 补齐的生产缺陷

`LensBar::restore_focus` 旧回调只比较目的地/tab；在回调排队后用户已进入内容区时，它仍可把焦点移回导航；离页后返回同一 tab 也仅凭目标相等可误通过。

- `Shell::go` 让每次实际导航请求都递增已有 `navigation_generation`，并继续用于脏草稿决定。
- 导航代次在激活请求时捕获，不在后续恢复挂载时重读。延迟恢复同时核验该代次、目的地/tab、本地环境及当前焦点；焦点已改变时不恢复。正常重挂载造成焦点暂时丢失仍允许同代次恢复，不重建焦点 owner。
- 新 `delayed_tab_restore_preserves_a_new_content_focus` 覆盖用户焦点进入内容区、回调排队后离页返回、恢复挂载前离页返回三种顺序。先前两次试跑错误读取尚未刷新的控件快照，不能作为产品缺陷证据；纠正为实时 FocusHandle 后，未加焦点变化守卫时实际句柄从内容区 8 变为导航 137 而失败，加入守卫后通过。保留 `late-focus-red-actual-handle.log`、`late-focus-green-actual-handle.log` 及最初无效试跑日志；最终三场单测收据为 `late-focus-owner-final.log`，不隐去测试修正过程。
- `DESIGN.md` 补明该延迟恢复边界；无新增依赖、Tauri 重设计、平台能力或显示矩阵。

### Rust 1.95 实跑

仅安装官方 `1.95.0` minimal 工具链，显式选择 cargo/rustc，独立 `target/msrv-1.95` 子目录；默认工具链及根 `rust-toolchain.toml` 的 1.98.0 pin 不变。三处 `--locked` 检查均通过：

- 根：`cargo check --workspace --all-targets --all-features`（1m06s）；
- Tauri：带 `.cargo/tauri-ci.toml`，`cargo check --manifest-path ccr-ui/src-tauri/Cargo.toml --all-targets --all-features`（1m36s）；
- GPUI：`cargo check --all-targets`，最终焦点源码后再次通过。

日志 `msrv-195-{root,tauri,desktop}.log` 首行记录 `rustc 1.95.0 (59807616e 2026-04-14)`，最终 GPUI 检查另在 `msrv-195-desktop-final.log`。这是 Windows 编译与测试 target 的类型检查；未声称 Rust 1.95 测试执行、Linux TLS 运行或原生 UI 通过。

### 最终验证

- 最终实时焦点回归通过（含三种晚回调顺序）；`just desktop-test-ui` 14 shell + 2 input 通过。
- 最终 `just desktop-ci` exit 0：格式、全 target check、严格 Clippy、16 Python、AccessKit、lib **303 passed / 4 ignored**、shell 14 passed / 1 ignored、input 2 passed、shutdown 2 passed / 1 ignored；包含 Windows 1 MiB 子进程。最终收据 `late-focus-desktop-ci-final.log`，中间版本的 gate 不能冒充最终版本证据。
- 新 release：`ccr-desktop/target/release/ccr-gpui.exe`，54153216 bytes，SHA-256 `cb1ae935365820c5024b00fce7cc8aa49f6a122f3654df186617a126c2352a66`。`late-focus-build-identity.json` 同时记录最终 shell 源文件/manifest/lockfile 哈希；旧候选收据保留。当前候选无原生截图、尺寸或 DPI 证据。
- 最终 `just ci` exit 0，14m55s，16 个阶段全部通过，含根测试/release、两份锁 audit、治理、完整 Tauri（含绑定/inventory）、frontend/docs/audit/coverage、VS Code。进程 PATH 仅前置 Git Bash，未修改系统 WSL/bash。
- `git diff --check` 通过。根/Tauri/GPUI 三份锁、桌面 Cargo.toml、CONTEXT 与 pastedcontent 相对本轮开始逐字节不变；本轮只增量修改导航 owner/回归、DESIGN 及交接。原 journal 字节前缀与旧索引会话记录的最终核验在 `late-focus-preservation.json`。
- Session 168 仅追加真实结果，明确 **未验收完成**。#161 继续 open；四场 native not_run。未 commit、push、发布或上传截图，未修改真实偏好或业务账户/凭据/方案。


## 全仓未关闭票的生产续做（2026-11-11，Session 169）

用户已明确把范围扩展到整个 CCR 仓库的全部未关闭票。再次查询只有七张：[DESIGN 合同缺陷](https://github.com/bahayonghang/ccr/issues/150)、[用量新鲜度](https://github.com/bahayonghang/ccr/issues/151)、[项目未归属](https://github.com/bahayonghang/ccr/issues/152)、[工具与会话假零费用](https://github.com/bahayonghang/ccr/issues/153)、[工具事件时间口径](https://github.com/bahayonghang/ccr/issues/154)、[Claude 配置目录](https://github.com/bahayonghang/ccr/issues/156)、[重设计实施 PRD](https://github.com/bahayonghang/ccr/issues/161)。已关闭的 Wayfinder 地图与设计决策票不重开，不重新选原型；原先独立的用量缺陷由此次明确授权纳入生产修复。

### 生产修改与边界

- 项目归属：`ccr-usage` 的 Observer 查询及 Home 原有项目分组提供 `project_attribution_complete`，保留现有已归属项目计数。空数据是真实 0；有请求但无项目归属显示原因；混合数据保留已归属数量并标部分覆盖。共享 `ccr-app`、Tauri DTO/事务生成绑定、前端与 GPUI 概览/独立观测页都消费该状态。没有更改上游解析器、推断 cwd、重建真实数据或把项目模块放回 Claude 首页。
- 工具费用：`ccr-db` 不再把 NULL 费用合并成 0，部分已计价的分组也保持未知。共享及 Tauri `SessionRow`/`TopToolRow` 保留 nullable 费用，未知会话 token 改为 null，model/start 继续缺失。默认按调用数在 SQL limit 之前排序；显式按费用排序将未知项置后。GPUI 和前端显示“未记录”，真实零仍显示零，分母零不制造每工具费用。未新增 llmusage 会话费用查询。
- 时间：live/fixed 工具排行、会话与热力图都使用捕获的 as_of 和 julianday 闭区间，排除未来；热力图按每条事件发生时的 Local/UTC 转换后分桶，两个 UI 明示时区。缺失/无效时间戳的工具行跳过并计数，保留后续调用 seq 空位；已变文件重扫对已知错误位置的删除、有效行 upsert、游标提交同事务，游标失败全部回滚。未变文件仍走既有游标快路，不声称已重建过去缓存或真实用户数据。
- 配置目录：补齐 Tauri 输出样式的遗漏，统一使用 `ClaudeRuntimePaths.config_dir` 与捕获的 backup/lock。保留 batch upsert 的“省略项不删除”和守卫写入；夹具存储绑定隔离根，但既有 `fixture_capability_unavailable` 命令门控不扩大。继承的 GPUI 规则/样式、用户指令合同继续保留。
- 新鲜度：复核成功 sync / sync --rebuild / hook-run 的全局查询，增加 Claude/Codex 过滤仍返回同一全局时间的回归；概览与用量正文明确“任意来源”，不承诺当前平台完整。
- DESIGN 漏项：配置页返回/刷新、其他平台首页刷新改用 ghost 语义，刷新加图标；其他平台费用摘要统一两位/小额阈值。订阅已知套餐本地化、未知模式和套餐保留原文；auto 摘要不带套餐/月费。名称/端点/模型搜索保留 PRD 后续批准的范围，不退回旧票的仅名称检索。

### 真实回归与中间失败

证据目录为 `ccr-desktop/outputs/claude-code-redesign/implementation/related-closeout/`（本地忽略）。

- `projects-red.log` 是旧投影缺少归属状态的真实失败；四种归属状态的 shared/Home 及界面回归随后通过。
- `tool-cost-window-red.log` 复现 NULL 被变成 0、1 条窗口内记录被错误算为 3 条；`timestamp-red.log` 复现缺时间戳使用扫描时刻。owner 最终通过。首轮修复时错误把 2000 年固定时钟夹具的 live 历史行期望从 5 改为 3，已改回 5；保留该失败日志，未修改生产逻辑去迎合错误断言。
- 首个 GPUI 费用显示断言没有捕获行单元测试快照，后补明确的现有单元 ID test support，并在项目归属断言后还原无 insight 状态，避免不相关的滚动裁剪。后续 lib 全量通过；不将此 headless 控件回归称作原生验收。
- 首个 Tauri 捕获根测试调用了夹具明确不支持的输出样式命令而失败。已保持能力拒绝，改为断言门控后测试捕获 store；生产自定义目录仍通过真实命令的独立子进程验证。两个最终测试通过，默认 home 字节不变。
- ESLint 指出 Hero 新条件使复杂度达到 18（门槛 16），将归属文案分支放入小型纯 helper 后通过；没有降低门槛或添加忽略。
- `shared-owner-tests.log`：ccr-app 207 passed/4 ignored、配置读取 5 passed/1 ignored、方案服务 12 passed/1 ignored；ccr-db 140、ccr-usage 66 通过，另有既有策略/契约/doc tests。
- 前端四文件 27 项通过；类型检查和窄 eslint 通过。`just tauri-bindings` 正常事务生成五个受影响 DTO；未手工修改生成 TS。`desktop-test-ui.log`：14 shell/1 ignored + 2 input 通过。最终组合门禁结果在下文追加，以上中间结果不冒充最终门禁。

### 原生结果与保留

当前 `cua.getState()` 仍报 native pipe unavailable / os error 2；一次 `js_reset` 后再次调用返回相同错误，浏览器 host 也不可用。已结束有界恢复，具体回包见 `native-recheck.json`。四场指定原生均 not_run；没有启动真实/隔离 GUI、修改偏好、执行登录/切换/同步/扩展安装，也没有新增截图、外窗、客户区或 DPI 观测。

本轮起点 `start.json` 记录 68 个既有变更路径、原 HEAD 与哈希，后续 compact 时已有 88 路径，并非全部新增。根/Tauri 两份安全锁、桌面 Cargo.toml/Cargo.lock、CONTEXT、pastedcontent 与此起点逐字节一致；原 journal/index 在追加前也一致并已另存字节前缀。继续不 commit、push、发布或上传截图。DESIGN 和共享生产合同仅补本轮真实边界。


### 最终组合验证（Session 169）

- `just desktop-test-ui` exit 0：14 shell/1 ignored + 2 input。
- `just desktop-ci` exit 0：格式、所有 target check、严格 Clippy、16 Python + AccessKit、303 lib/4 ignored、14 shell/1 ignored、2 input、2 shutdown/1 ignored，包含 Windows 1 MiB 子进程；新 release 成功（3m48s）。新增归属/费用呈现、订阅 auto/未知模式/套餐、概览未归属状态均由最终 lib 回归覆盖。
- 三处当前源码 Rust 1.95 `--locked` check 均通过：root workspace/all-targets/all-features、Tauri/all-targets/all-features、GPUI/all-targets。分别使用既有隔离 `target/msrv-1.95`，默认工具链与 1.98 pin 不变；仅证明 Windows 编译，不声称 1.95 执行测试或 Linux TLS/原生 UI 通过。
- 新候选 `ccr-desktop/target/release/ccr-gpui.exe`：SHA-256 `22ae86c7a775794f74939a5227911fb461d2a5cb5074943f42e302ed5211187a`，54184960 bytes。`build-identity.json` 与 `validation-inputs.json` 记录 dirty HEAD、工具链及 807 个共享/桌面/第三方源文件的快照哈希，构建后核验不变。无截图、客户区、外窗或 DPI 对应观测。
- 最终仓库 `just ci` exit 0，13m43s，16 阶段全部通过：版本/格式、harness 与治理、严格 Clippy、workspace/根测试/release、两份 Rust audit、完整 Tauri、frontend-check（含文档）、frontend-audit、coverage、vscode-ci。收据 `ci-final.log`；只对该命令进程前置 Git Bash，系统 WSL/bash 不变。Tauri 336 normal / 5 ignored、2 SQL guard、200 exports 与 inventory 通过；前端 174 文件 / 951 测试及覆盖率均通过。原有 Rust 警告和前端两个既有审计例外未添加或扩大。


首轮 `just ci` 运行 15m08s，在 Frontend Check 的 i18n 固定叶子计数校验失败；前 12 阶段（包括根/Tauri audit、治理、完整 Tauri）均通过。原因是本轮新增 6 个双语键后，两个严格基线仍为 4564，而实际中英文各为 4570。已将 `tests/i18n.test.cjs` 与 `scripts/check-i18n.mjs` 的固定基线同步到 4570；集合相等、固定数量、占位符绑定和 key leak 检查均保留，未降低门槛或加白名单。`bun run test:i18n`、`bun run check:i18n` 均通过；前者包含 key leak 自测。`ci.log` 保留失败，最终重跑在 `ci-final.log`。此修正只有校验夹具，无 Rust/GPUI 源码变化，桌面门禁与候选身份不失效。


最终重跑 exit 0；`ci-final.log` 明确 16 阶段全绿，首次失败日志继续保留。`git diff --check` 通过。最终 GitHub 查询仍为上述七张 open，未凭本地测试或 journal 关闭实现票；原生证据仍缺失，尤其 DESIGN 合同票和重设计 PRD 不具备关闭条件。此次没有 commit/push/发布/截图上传。最终保留核验在 `preservation-final.json`，journal 只追加 Session 169，旧字节前缀与索引会话行须保留。


## 续做：原生通道定位与全仓复核修复（2026-11-11，Session 170）

起点 HEAD `4696c36c`、`dev`、107 个变更路径，含本轮新建的交接 Prompt；未 reset/checkout/clean/stash，未 commit/push。会话期间另一个并行会话在 `crates/ccr-cli` 提交了无关的 `b1977fd9 fix(CLI): 补上 project init 遗漏的平台忽略目录`，HEAD 因此前进到 `b1977fd9`；本轮改动不包含该文件，全部仍未提交、未暂存。807 个构建输入哈希与 `validation-inputs.json` 逐一致，`ccr-gpui.exe` 仍为 `22ae86c7…`，因此上轮 `just desktop-ci` / `just ci` 的通过证据在源码未变时仍然有效；本轮改动后按下文重新门禁。

### Computer Use 原生通道：已定位，不是仓库缺陷

用户在本轮明确指示「不需要使用 codex computer use 验证」，原生排障到此为止，不再尝试恢复。定位结论（只读证据）：

- `cua_repl` 的 MCP 配置把原生管道写成固定名 `\\.\\pipe\\codex-computer-use-e5efa1db-0e7a-469f-9bde-cc60dcbd6186`（`~/.codex/plugins/cache/openai-bundled/unified-computer-use/26.1103.60933/.mcp.json`）。
- 该文件写于 11-10 03:51（重启前）；机器于 11-10 20:48 重启，此后 Codex 桌面应用（管道宿主）从未运行：`Get-Process` 无 `Codex.exe`/`ChatGPT.exe`，管道清单里只有 `OpenAI.CodexSandbox…`。
- 因此 `os error 2` 是「该实例的命名管道不存在」，即宿主应用未运行、管道名已过期，不是 CCR 代码缺陷；`cua.getState()` 的 apps/browsers 空清单同源。浏览器 host 不可用是同一宿主缺失的连带结果。
- 未逆向私有管道协议、未手工启动 `codex-computer-use.exe`、未改用 UI Automation/headless 伪造原生证据。四场原生仍为 not_run，无截图、外窗、客户区与 DPI 观测。

### 复核发现并修复的生产缺陷

三个只读审计代理按 PRD 逐条核对导航/作用域、工作台、配置方案与扩展，发现以下真实缺陷，本轮全部修复并补回归（均为未提交的本地改动）：

- 配置目录（#156 漏项）：Tauri `system_prompts` 的用户级 CLAUDE.md 与规则列表仍固定读主目录 `.claude`。改为 `ClaudeRuntimePaths.config_dir`，共享层新增 `list_claude_rules_in(root)`；备份、锁与原子写入不变。红→绿证据：`system-prompts-red.log`（解析到真实主目录）/`system-prompts-green.log`。
- 工作台订阅：保存冲突后提示「取消并重开」而取消并不重读，用户再存必再冲突。改为冲突时置 stale、拒绝用旧 revision 提交、提供「重读存储设置」并在重读后清除；成功保存只走 `Applied` 未激活路径。`save-feedback-red.log` 为旧行为红灯。
- 工作台图表：键盘可达但只有 hover tooltip 能读到日期/金额/口径。改为激活日期条后在同面板显示可见读数行 `workbench-claude-trend-readout`，并高亮精确值表对应行。
- 工作台其余：报告读取失败不再停留在「正在读取」；费用口径行标注报告时区与截止日；端点解析失败按界面语言显示未知；右栏宽度改用与运行概览一致的几何（侧栏/场景边距/纸面亮边/内边距）；KPI 带末格不再画右边线、窄窗第三格加上边线；精确值统一六位小数；模型面板新增本面板内的精确值入口。
- 配置方案：保存成功被误判为「已保存，运行配置未更新」（该状态实为事务回滚）已修正为按 `Applied`/`Unchanged` 分流，回滚时保留草稿与 revision；Claude/Codex 可直接删除当前运行方案，现于命令层拒绝并禁用按钮（`active_config_cannot_be_deleted`）；Esc 与「取消/返回」在有脏草稿时先确认（新 ID `profile-confirm` 复用，动作 `DiscardDraft`）；校验输出补「仅本地字段，不证明端点或凭据可用」；解除当前方案的文案按平台说明凭据影响；替换预览不再每次点击都写备份（备份移到提交），并列出将被移除的方案；输出面板文案区分导出与临时明文视图，非导出不再显示「导出已就绪」。
- 导航与平台：嵌入的用量页不再渲染重复标题；分组 tab 间距按合同改为 16px（含分隔线算术）；嵌入 OpenCode 时补回 `tui.json` 的 TUI 入口（此前被 surfaces 行隐藏后完全不可达）；同步 `page_header.rs`/`shell/mod.rs` 注释与 `DESIGN.md` 的导航段落。
- 共享导入：`ccr-app::config::import::preview` 去掉 `import_preview` 备份写入，新增 `removed` 列表；回归 `replacement_preview_lists_removals_and_leaves_no_backup_behind`。
- 其余合同项：列表/详情恢复 `min_w(360/320)`（此前被紧随的 `min_w_0` 覆盖，等于没有下限）；平台 tab 溢出提示补稳定 ID `platform-tab-overflow` 与可读名称；扩展来源把「不可读」与「不安全路径」分开标注；旧 settings 数组缺失写「—」，不再冒充 0 条；模型面板补本面板内的精确值入口。

### 本轮最终门禁与身份

- 根 `just ci` exit 0，14m43s，16 阶段全绿（版本、格式、OMP/harness/Copilot、严格 Clippy、workspace check、根测试、release、安全审计、治理、完整 Tauri、frontend-check 含文档、frontend-audit、coverage、vscode-ci）；收据 `ci-final4.log`。该次运行覆盖除最后一项桌面专属修复（删除不可达 match 分支，`ccr-desktop` 不属于根 workspace）之外的全部改动；根格式/Clipy 均不覆盖该 crate，故结论不受影响。
- `just desktop-ci` exit 0（收据 `desktop-ci-final2.log`）：格式、check、严格 Clippy、305 lib/4 ignored、14 shell/1 ignored、2 input、2 shutdown/1 ignored，含 Windows 1 MiB 子进程回归与新 release 构建。首次桌面 CI 因新增状态分支后 `_` 通配不可达而失败（`error: unreachable pattern`），已删除该分支后重跑通过；保留失败事实，未降低 lint 门槛。
- 共享 owner：`cargo test -p ccr-app -p ccr-config -p ccr-cli` 与 Tauri（337 + 2）全部通过（`shared-owner.log`、`tauri-tests.log`）。
- 新候选 `ccr-desktop/target/release/ccr-gpui.exe`：SHA-256 `7d74de4963c609b1a4e8e28faa3c106b9dd4649faf0741c5f2faa0c726c48b21`，54228992 bytes，构建于本轮最终源码。该候选未用于任何原生交互；任何原生验收必须在此候选上重新执行。
- 保留核验：`preservation-session170.json` 记录 HEAD `4696c36c`、`dev`、109 个变更路径、暂存区为空，以及根/Tauri lockfile、桌面 Cargo.toml/lock、CONTEXT、pastedcontent 的哈希。未 commit、未 push、未发布、未上传截图、未关闭任何 issue。

### 本轮已知偏差（未修，需后续决策）

- 未删除的遗留 ID/分支：`profile-clean-backups`、`profile-clear-history` 仍在页面动作表里（按平台过滤，当前不可达），`shell` 里仍有 `tool-accounts` 不可达分支与 `Lens::Agents/Plugins/SlashCommands/Observer`。它们只在源码层面存在，不产生新 UI；删除需同时收束命令面，未在本轮做。
- 8.3 夹具清单仍有缺口（无库/不可读库的 Claude 首页、起点未知、缓存分母零的夹具级用例、30 天与全历史分离断言、导入预览 UI、脏字段省略与空凭据 UI、扩展同名多来源与不安全路径）；本轮只补齐了被本轮改动直接影响的回归。
- 账户与设置的局部重排未实施：PRD 第 6 节的「账户按选中对象操作、保存当前登录单列」与「设置区分运行设置 / CCR 元数据 / 供应商目录」在 `features/platforms` 中仍只有隐藏标题与 tab 的改动，未见分组实现。该条属于本轮未完成的 PRD 范围，不能按已实施记录。
- 指令页已写明用户级与「未接入」；扩展页仍缺「同名多来源」的表达，也没有完整导出与其确认动作（不可读/不安全标签与旧数组缺失的「—」已在本轮补齐）。
- 原生验收未运行（用户已豁免 Codex Computer Use 通道）；DESIGN 合同票与重设计 PRD 因此仍不具备关闭条件。
