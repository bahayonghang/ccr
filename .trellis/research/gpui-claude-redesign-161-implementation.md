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

### 账户与设置局部重排（补做上条「未实施」）

本节补做「本轮已知偏差」里账户与设置那一条，只动 `ccr-desktop/src/features/platforms/`（`mod.rs`、新增 `scope.rs`、`layers.rs`、`tests.rs`）、`tests/native_shell.rs` 与 `DESIGN.md` 的局部段落。没有改 `ccr-app` 写入语义，也没有改 Tauri。依据是 #149 的「账户 | 局部重排」「设置 | 局部重排」两行，以及 #139 的「只显示账户那部分 / 配置来源仍是页内开关」。

已实施：
- 账户 tab 分两组。「当前登录」（`platform-accounts-current`）放新账户名、「保存当前登录」和清除运行认证，不读列表选中项。「已保存账户」（`platform-accounts-saved`）是一个可选列表（`platform-accounts-list`，ListBox）加选中状态行（`platform-accounts-selection`）。切换、Codex 重命名、删除和「查询所选账户配额」只作用于选中行；没有选中时禁用。打开时不自动选中；重读后只保留仍存在的选中项；账户操作提交后清除选中。删除和重命名的确认句写出账户名。
- 设置类页面加三类分组标题：`platform-group-runtime`、`platform-group-provider-catalog`、`platform-group-ccr-metadata`。标题旁写来源文件名，`*-scope` 写实际作用域。Antigravity 的 Gemini 路径写明客户端是否读取未验证。OpenCode 的供应商状态归入供应商目录组。配置来源开关只在设置 tab 出现，标题改用 16/600 分区标题。
- Codex 的模型和供应商、OpenCode 的供应商是同一个共享组件渲染的，所以也带上了分组标题。这超出了账户和设置两个 tab，只是继承了共享组件的改动。
- 第二轮（协调方据 #139 规则 1、规则 3 与 #149「配置来源为解释性展开区」确认为已批准决定）：
  - 账户 tab 只显示账户那部分：只渲染凭据观测 `platform-auth-status` 与 Grok 的账户来源 `platform-oauth-source-*`，不再渲染设置文件状态 `platform-file-status` 和运行设置字段 `platform-field-*`。设置文档仍在后台读取并作为账户 CAS 令牌，CAS、确认、后台登录语义不变。
  - 设置类 tab 只显示设置那部分：不再渲染凭据观测和静态已保存账户列表；OpenCode、Antigravity 的「Accounts are not supported」占位一并去掉。核对过设置 tab 上没有任何动作以凭据状态为前提（`features/platforms`、`shell` 中凭据状态只用于渲染 `platform-auth-status` 与账户列表），所以没有需要保留的例外。
  - 配置来源改为页内展开区：开关 `platform-show-layers` 仍在设置 tab 页内，打开后在设置表单之后展开只读来源清单 `platform-layers`，表单与 `platform-save` 仍在原位可用，不再整页替换。复用同一个 `LayersView` 实体，不重建焦点所有者；键盘激活开关时记下开关焦点，收起时把焦点还给开关；此前用键盘激活过开关时，焦点进入展开区后再用鼠标收起，焦点同样回到开关。

未实施，待维护者决定：
- 高级补丁仍没有独立确认（确认放在打开编辑器时还是提交时，未定）。
- Codex 的「添加 API 账户」「导入」放哪一组，未定。
- 普通字段没有再细分分组。
- 切换仍是 ghost 按钮，执行前不确认（#149 只要求保留凭据确认）。
- 只用鼠标、从未用键盘激活过开关时，拿不到 Kit Button 的焦点句柄（它按元素 id 存在 keyed state 里，鼠标按下不取焦点），这时收起不会把焦点移到开关上。

门禁结果（均为真实退出码）：
- `just desktop-fmt-check` 0。格式修复单独用 `just desktop-fmt` 执行，执行前后对另一会话的 7 个文件做了哈希比对，结果不变。
- `just desktop-lint` 0。
- `just desktop-test` 0：accesskit 3 项通过；lib 309 通过、4 忽略；native_shell 14 通过、1 忽略；native_ui 2 通过；runtime_shutdown 2 通过、1 忽略。中途出现过 E0308（Observed 与 Stateful 类型不一致）和 E0616（测试读了私有字段），都改代码后修好，没有删测试。
- `just desktop-test-ui` 0：native_shell 14 通过、1 忽略；native_ui 2 通过。

第二轮门禁（a、b、c 完成后的最终源码，均为真实退出码）：
- 中途：`just desktop-fmt-check` exit 1，只报本轮改的 `mod.rs`、`tests.rs`、`native_shell.rs`；单独跑 `just desktop-fmt` 修复（exit 0），另一会话的 6 个桌面 Rust 文件哈希前后一致。窄跑 `cargo test --lib features::platforms` 时 `configuration_sources_are_discoverable_and_managed_layers_stay_read_only` 失败过一次：裸视图没有外壳的 Tab 绑定，`press("tab")` 走不到开关；改用 `focus_next` 后通过，没有删断言。
- `just desktop-fmt-check` 0；`just desktop-lint` 0。
- `just desktop-test` 0：accesskit 3 项通过；lib 309 通过、4 忽略；native_shell 14 通过、1 忽略；native_ui 2 通过；runtime_shutdown 2 通过、1 忽略。
- `just desktop-test-ui` 0：native_shell 14 通过、1 忽略；native_ui 2 通过。

新测试：`features::platforms::tests::account_actions_follow_the_selected_row_and_saving_ignores_selection`、`features::platforms::tests::settings_scope_groups_follow_each_platform_surface`。另在 `platform_routes_show_supported_local_status_without_secrets` 里补了断言。

原生验收：not_run。

## 扩展页：同名多来源与旧版 settings 数组导出（#161 子任务）

依据：#160「技能与旧 commands 来源可同屏但各保留类型/真实路径，不按名称粗暴合并」「默认脱敏查看/导出，含凭据完整副本必须显式确认且仅临时显示或显式保存」；#149「默认只给脱敏查看/导出；完整导出需确认」；PRD 8.3 临时查看与保存文件是不同动作。

已做：
- 共享层 `crates/ccr-app/src/resources/claude_extensions.rs`：`Source.names` 改为 `entries: Vec<Entry>`，`Entry` 带可选 `SameName { name, peers, rule }`。同名只在同一命名空间内比对：子代理按 frontmatter `name`（读不到则不参与比对，条目照常列出）；技能目录名与旧命令文件名互比；插件按 `@` 前名称。`rule` 只取研究票写明的两条（技能优先于同名旧命令；同名子代理只加载其一、哪一个未写明），其余 `Unverified`。新增「插件自带的子代理/技能/命令」来源，状态未接入（不跟随 installPath）。新增 `legacy_export(settings, full)`：只含存在的 agents/slashCommands/plugins；默认掩码敏感键子树、对其他字符串做日志脱敏、`plugins[].config` 叶子全部掩码；full 原样输出；都不写 settings。`SkillsService::extract_frontmatter_name` 改为 pub。
- 桌面运行时：`ResourceCommand::ExportClaudeLegacy { full }`（只读执行器）与 `SaveClaudeLegacyExport { content, destination }`（作业），非 Claude 返回 Unsupported，非本地环境沿用既有拒绝；保存复用 `config::source::save_export`（只写新文件，已有文件报冲突），夹具环境要求目标在夹具根内。
- GPUI：新文件 `ccr-desktop/src/features/resources/claude_extensions.rs`。同名行在条目下方用辅助色写「同名“x”也见于：来源 · 位置。规则」。旧版条目卡片提供「脱敏导出」和「导出完整副本（含凭据）」；后者走既有 Decision 确认（从外部打开时焦点落在取消、Tab 在确认子树内循环、Esc 回到触发处、确认期间底层动作禁用）。临时输出只读显示，标注「临时显示，尚未保存」；落盘只经「将当前输出保存到新文件…」；关闭或离页清空明文并递增 generation，作废在途读取与保存对话框。
- 原生 1 MiB 栈：首版扩展渲染让 `shell_claude_resources_write_confirm_conflict_and_keep_private_fields` 在 1 MiB 下溢栈（1.1 MB 可过）；改为每个按钮/卡片/面板一个 `#[inline(never)]` 小帧函数并返回 `AnyElement`，1 MiB 下通过。

新 test id：`claude-extension-same-name-{source}-{row}`、`claude-extension-legacy-export`、`claude-extension-legacy-export-full`、`claude-extension-legacy-confirmation`、`claude-extension-legacy-cancel`、`claude-extension-legacy-confirm`、`claude-extension-legacy-trap`、`claude-extension-legacy-output`、`claude-extension-legacy-save-file`、`claude-extension-legacy-close`、`claude-extension-legacy-content`、`claude-extension-status`；`claude-extension-source-{i}` 含义不变，新增 aria 标签，并多出索引 6（插件自带内容）。

新测试：`same_name_entries_stay_listed_per_source_with_documented_rules_only`、`linked_sources_are_refused_without_hiding_other_sources`、`legacy_export_masks_credentials_by_default_and_full_copy_is_verbatim`（ccr-app）；`cached_catalog_without_cache_refuses_without_scanning_or_writing`（ccr-app codex_agents）；`legacy_export_is_read_only_and_saves_only_new_files_inside_the_fixture`、`claude_extension_reads_reject_other_platforms_and_remote_without_fallback`（桌面运行时）；`claude_extensions_show_same_name_sources_and_gate_full_legacy_export`（GPUI）。

未做 / 待维护者决定：
- 用户级与项目级同名：项目范围仍未接入，无法展示。
- 插件自带条目与用户级同名：需跟随 installPath，#160 禁止，未做。
- 托管（managed）范围没有列为未接入来源。
- 旧数组条目是否与真实文件交叉标注同名，未定，未做。
- 脱敏粒度（`mask_sensitive` 保留首尾各 4 字符；plugins config 全部掩码）为本次实现选择，待确认。
- 同名用条目内注记还是分组视图，未定；技能嵌套目录、命令子目录的命名规则为推断。

门禁（真实退出码）：
- `just fmt-check`：先 1（只报本轮文件），按文件 stdin rustfmt 修复并恢复 CRLF 后 0；`just lint-strict` 0；`just test` 0（50 个套件，2605 通过、31 忽略，已知不稳定的 journal 测试未出现）；`just tauri-test` 0（339 通过、5 忽略）；`just tauri-bindings-check` 0。
- `just desktop-fmt-check`：先 1，修复后 0；栈修复后又报 1（import 换行），手工改后 0。`just desktop-lint` 0。
- `just desktop-test`：第一次 101（上述 1 MiB 溢栈：lib 311 通过、1 失败、4 忽略）；栈修复后第二次 101，失败的是未改动的 `features::mcp::tests::mcp_native_actions_write_confirm_and_keep_conflicting_drafts`（tests.rs:97 找不到 “MCP JSON patch” 节点），单独连跑 3 次均通过，判断为负载下偶发；第三次 0：accesskit 3 通过；lib 312 通过、4 忽略；native_shell 14 通过、1 忽略；native_ui 2 通过；runtime_shutdown 2 通过、1 忽略。
- `just desktop-test-ui` 0：native_shell 14 通过、1 忽略；native_ui 2 通过。

原生验收：not_run。

## 稳定 ID 遗留清理（PRD 第 9 节收尾）

依据：PRD #161 第 9 节、第 5.1 节「全局清理备份和清空记录不在本页」；#139 结论评论的 test id 变更清单。本节是纯清理，可达界面、动作、焦点顺序与文案不变。

已删除（均先证明不可达）：
- `shell/mod.rs`：`Lens::Observer`、`Lens::Agents`、`Lens::Plugins`、`Lens::SlashCommands` 及其在 `activate`、`focus_main`、`body` 中的分支。唯一产出它们的是 `platform_navigation::link_lens`，而它的输入 `workbench::Link` 只来自 `WorkbenchView.pending`；9ca72718 删掉工作台导航按钮后 `pending` 只会被置 `None`，没有任何 `Some` 写入。
- 上述死链整条：`features/workbench.rs` 的 `Link` 枚举、`pending` 字段、`take_pending`/`has_pending`；外壳 render 里消费它的 next-frame 块（含 Claude「打开日报」特例）；`InsightsView::select_daily_report`（只被该块调用）。
- `shell/mod.rs::lens_items` 里平台目的地的旧胶囊分支（`tool-accounts`、`Lens::Primary` 指向方案列表的 `tool-profiles`、`tool-platform`、旧 `tool-files`）。原代码在 match 前已对平台目的地提前返回 `platform_items`，这些分支永远走不到；改为在 match 内直接返回 `platform_items`，其余目的地远程时仍返回 `None`，求值顺序等价。
- `shell/mod.rs::activate_primary` 里平台目的地打开方案列表 / 平台设置的分支：同样被前面的提前返回遮蔽，改为在 match 内打开工作台。
- `features/profiles/`：`profile-clean-backups`、`profile-clear-history` 两个动作表项、`Action::Clean`/`Action::ClearHistory` 变体、确认文案与 `confirm` 分支。按平台过滤：Claude 显式排除，Codex/Grok 白名单不含，其他平台 `_ => false`，任何平台都渲染不出来；`act(Action::Clean|ClearHistory)` 没有别的调用方。

没有可迁移的持久化旧值：桌面偏好（`desktop-gpui/preferences.json`）只存主题、语言、退出确认与窗口位置；`ui_state` 只存 CLI 命令收藏/历史；启动参数没有页面/tab 深链。不存在保存过 `tool-accounts`、`Agents`、`Observer` 的地方，因此未加迁移代码。

保留及原因：
- 共享与运行时层不动：`runtime::profiles::ProfileCommand::CleanBackups`/`ClearHistory`（桌面运行时适配，仍有运行时测试）、`ccr_app::config::backups::clean`（Tauri `commands/config.rs` 使用）、`ccr_store` 历史清空（Tauri `clear_history` 与 CLI 使用）、CLI `ccr clean`。删除页面项后 GPUI 内已无清理备份 / 清空配置操作记录的入口（命令目录里 `clean` 为 preview_only、不可执行），本轮按 PRD 不另开入口。
- `InsightsPage::Observer` 与 `insights-observer-tab`：全局用量仍可达。
- 「平台嵌入时不渲染」类 ID 全部保留：平台目的地内确认不渲染，全局用量保留 `insights-*` 三项。例外：OpenCode 供应商 tab 仍渲染 `platform-settings`（与 `platform-tui` 一起作为 tui.json 子入口，`features/platforms/mod.rs` 嵌入分支有注释说明），属实际可达，未删，测试如实固定。
- 资源与平台视图的未钉住分支（`resource-platform-*`、`resource-instructions`、`resource-claude`、`platform-settings`/`platform-auth` 页内切换）：GPUI 外壳当前只以钉住方式打开这两个视图，这些分支只在直接构造视图的单测里出现；PRD 写的是全局视图保留，未删。
- Claude 旧 settings 数组编辑器（`features/resources/claude_files.rs` 的 Agents/斜杠命令/插件类别）：外壳只用 Rules/OutputStyles 打开，非文件类别按钮在文件类别下不渲染，GPUI 内已不可达；它不在第 9 节 ID 表内，旧数组处置归 #160，本轮未动。

测试：`platform_navigation` 改为逐平台断言完整有序 tab id（Claude 9、Codex 11、Grok 5、Antigravity 6、OpenCode 6）；新增 `removed_platform_tab_ids_do_not_appear_on_any_platform` 与外壳级 `platform_tabs_render_no_removed_or_global_only_ids`（逐平台逐 tab 点击，删除类与嵌入隐藏类 ID 均不出现，全局用量仍有三个 `insights-*` 页签）。`profiles/mod.rs` 原有的不存在断言保留。

门禁（真实退出码）：`just desktop-fmt-check` 中途 1（只报本轮新测试的两处换行，手工改后）→ 0；`just desktop-lint` 0；`just desktop-test` 0（Python 16 OK；accesskit 3 通过；lib 314 通过、4 忽略；native_shell 14 通过、1 忽略；native_ui 2 通过；runtime_shutdown 2 通过、1 忽略）；`just desktop-test-ui` 0（native_shell 14 通过、1 忽略；native_ui 2 通过）。新渲染测试首跑失败一次，即上述 OpenCode `platform-settings` 例外，改为如实断言，没有删断言。

原生验收：not_run。

## PRD 8.3 必须验证行为：测试对照与补测（2026-11-12）

依据：#161 第 8.3 节七条，逐项拆成断言后读测试体核对（不按测试名判断）。本节只补测试；生产代码仅为可测性给三处既有元素加了测试观测 ID（`claude-cache-{0..5}`、`claude-usage-freshness`，以及 `configuration-empty-detail` 的 `.test_support()`），不改行为、文案或布局。未修任何生产缺陷。原生矩阵（8.4）不在本节，仍 not_run；以下 headless 控件测试不算原生验收。

结论记号：✅ 已覆盖；◐ 部分覆盖；✗ 未覆盖。「新」= 本节新增测试。

| 条 | 断言 | 证据（测试） | 结论 |
| --- | --- | --- | --- |
| 导航 | Claude 九 tab、Codex 十一、Grok/Antigravity/OpenCode 准确集合与顺序 | `approved_navigation_has_exact_platform_capabilities_and_ids`、`platform_tabs_render_no_removed_or_global_only_ids` | ✅ |
| 导航 | 唯一 h1 = 当前 tab 名（每平台每 tab） | 新 `every_platform_heading_names_the_selected_tab`；原 Codex 窄窗测试只覆盖 Codex | ✅（“唯一”只核对 h1 节点与名称，未枚举全树 heading 数） |
| 导航 | 宽窄布局切换不重建焦点 | 新 `platform_tab_focus_survives_crossing_the_wide_layout_breakpoint`（2000→1280→2000→960，焦点留在同一 tab、标题行/下一行切换）；`delayed_tab_restore_preserves_a_new_content_focus` | ✅（headless） |
| 导航 | 窄窗横滚、选中项 reveal | `narrow_codex_navigation_reveals_each_keyboard_selection` | ✅（headless） |
| 导航 | 旧 ID 改名/删除清单 | `removed_platform_tab_ids_do_not_appear_on_any_platform`、`platform_tabs_render_no_removed_or_global_only_ids` | ✅ |
| 作用域 | 异平台 MCP batch 拒绝且无写入、全局 batch 可用 | `platform_authority_rejects_forged_batch_and_global_plan_without_writes` | ✅ |
| 作用域 | 异平台 MCP 编辑/复制（prepare 与 save、伪造 RequestKey.platform）拒绝且无写入，全局复制/编辑可用 | 新 `platform_scope_rejects_foreign_editor_and_copy_targets_without_writes` | ✅（补前 ✗：只有 batch 有测试） |
| 作用域 | 非 Codex 作用域废纸篓拒绝无副作用 | `captured_non_codex_scope_rejects_trash_and_prepared_commands_without_side_effects` | ✅ |
| 作用域 | 全局/Codex 废纸篓仍可用 | `opaque_codex_actions_reject_conflicts_and_keep_trash_recoverable`（未作用域的全局请求可移入废纸篓） | ◐（Codex 作用域下可用没有单独断言） |
| 作用域 | 导入 source 不能伪造 | 新 `scoped_import_rejects_forged_or_missing_source_before_admission`（运行时准入）；`platform_import_locks_source_to_its_platform_context`（界面） | ✅（补前 ◐：只有界面路径） |
| 作用域 | 远程不支持不回落 Local | `remote_request_does_not_fall_back_to_local_data`、`remote_and_unconfigured_reads_fail_closed_without_creating_paths`、`claude_extension_reads_reject_other_platforms_and_remote_without_fallback`、新导入测试的 SSH 分支 | ✅ |
| 用量 | 无库 | 新 `missing_and_unreadable_databases_fail_instead_of_reading_as_zero`（数据）、新 `claude_home_states_unavailable_unknown_and_window_scoped_cache`（KPI 写“不可用”、无缓存数字） | ✅（补前 ✗） |
| 用量 | 不可读库 | 同上两项 | ✅（补前 ✗） |
| 用量 | 无今日行 | `captured_calendar_distinguishes_missing_today_zero_and_same_window_tokens`；新界面测试断言“今日尚无已导入记录” | ✅ |
| 用量 | 真实零 | 同上数据测试；`platform_workbench_claude_partial_costs_recover_and_link_to_daily_report`（界面 $0.00，用本机时钟） | ✅ |
| 用量 | 低于显示精度的非零 | `recorded_amounts_keep_zero_and_subcent_cost_distinct` | ✅（格式函数层） |
| 用量 | 起点未知 | 新 `empty_database_keeps_start_latest_and_sync_unknown_and_sync_time_is_global`；新界面测试“起点：未知” | ✅（补前 ✗） |
| 用量 | 同步未知 / 全局口径 | 新数据测试（只有 Codex 同步时 Claude 报告拿到同一全局时间）；新界面测试 | ✅ |
| 用量 | 同步过期 | 新界面测试只证明旧时间按“全局”原样陈述；Claude 首页没有过期标记 | ◐ 交回维护者（见下） |
| 用量 | 未保存订阅 / 显式保存默认值 / 读取失败 / CAS | `saved_presence_distinguishes_defaults_and_preserves_cas`（共享）；`claude_snapshot_subscription_drafts_and_leave_remain_scoped`（界面） | ✅ |
| 用量 | 缓存分母零（夹具级） | 新数据测试 `tokens_and_cache_come_only_from_the_30_day_window`；新界面测试“不可计算（分母为零）” | ✅（补前只有纯函数） |
| 用量 | 输出含推理、不重复加算；总量用 owner total | 原数据测试 30+15=45；新数据与界面测试（输出 45，总量 45） | ✅ |
| 用量 | 不得用全历史缓存冒充 30 天 | 新数据测试（窗口外 500 缓存读取不进入 30 天）；新界面测试（变异：缓存面板改读 history 即红） | ✅（补前 ✗） |
| 配置 | 当前 A、选中 B 动作目标正确 | `selected_profile_is_independent_from_running_profile_and_dirty_navigation` | ✅ |
| 配置 | 禁用项明确“启用并应用” | 新 `wide_catalog_splits_three_to_two_and_sizes_lists_by_content`（行内按钮文案） | ✅（补前 ✗；动作栏 `profile-apply` 同名文案未单测） |
| 配置 | 筛选后不操作过期选中项、无匹配可恢复 | `catalog_filters_recover_and_narrow_back_preserves_selection` | ✅ |
| 配置 | 3:2 比例、短列表/空 pane 收紧、长列表限高、不自动选中 | 新同上测试 | ✅（补前 ✗；见下“观察”） |
| 编辑 | 保存不应用 | `saving_current_profile_preserves_runtime_credentials_and_unknown_catalog_fields`、`save_feedback_matches_what_the_lifecycle_actually_did`；新编辑测试断言 current 不变 | ✅ |
| 编辑 | dirty keys | 共享 `dedicated_patch_retains_omitted_fields_and_preserves_platform_semantics`；新 `edit_sends_only_changed_fields_and_an_empty_credential_keeps_the_secret` 只能证明存储结果 | ◐（界面省略未改键不可经存储观察：owner 把空说明归一为 None，变异存活） |
| 编辑 | 未知字段保留 | 原生 owner 测试；新编辑测试 | ✅ |
| 编辑 | 空凭据不清除 | 新编辑测试（界面路径）；`grok_credentials_preserve_replace_and_clear_without_display_round_trip`（共享） | ✅（补前界面 ✗） |
| 编辑 | 导入冲突默认保留、提交前 revision 改变则阻止 | 共享 `reviewed_import_keeps_matching_names_rejects_stale_revision_and_protects_current`；新 `import_preview_keeps_matching_names_and_a_stale_revision_blocks_commit`（界面：预览、覆盖按钮默认未选、输入改变禁用提交、外部改目录后提交被拒不写入、导入不应用） | ✅（补前界面 ✗） |
| 编辑 | 当前方案保护、备份、CAS | `catalog_edit_and_restore_preserve_activation_and_current_item`、`replacement_preview_lists_removals_and_leaves_no_backup_behind`、`two_source_clients_from_one_revision_have_exactly_one_winner`、`rendered_edit_save_conflict_cancel_and_delete_confirmation` | ✅ |
| 编辑 | Grok 部分结果不回归 | `platform_change_clears_only_previous_platform_recovery`（界面恢复动作随平台清理） | ◐（未逐个复核 shared/CLI 层 Grok 多阶段测试体） |
| 焦点 | 外部焦点真实打开、初始取消、双向 Tab、Esc 回触发者 | `catalog_decisions_enter_from_outside_without_mutating`、`confirmation_recovers_outside_focus_before_first_frame`、`confirmation_enters_after_native_callback_before_draw_order`、`nested_confirmation_preserves_editor_return_focus` | ✅（headless） |
| 焦点 | 快速重复打开 | 同 `confirmation_focus_boundary`（取消后立即再开、再开后离页） | ◐（未覆盖“同一帧内连点两次”） |
| 焦点 | 离页 / 换平台 / 晚回包不抢焦点不覆盖新上下文 | `confirmation_focus_boundary` 离页分支、`delayed_tab_restore_preserves_a_new_content_focus`、`leave_discards_scoped_pending_text_and_old_response_identity`；新 `late_source_response_after_leave_or_platform_change_is_dropped` | ✅ |
| 焦点 | 换环境 | `switch_keeps_each_environment_cache_draft_and_job`（共享，仅名称核对） | ◐（未见方案页换环境的界面测试） |
| 扩展 | 坏 manifest / 缺失 / 超限 / 不安全路径 / 部分失败 / 无项目 / 未接入 | `discovery_is_read_only_partial_and_keeps_legacy_arrays`、`skills_use_existing_metadata_and_registry_limits_do_not_hide_other_sources`、`linked_sources_are_refused_without_hiding_other_sources` | ✅（数据层） |
| 扩展 | 各状态在界面分别表达（含“客户端加载未验证”只用于已发现） | 新 `every_extension_source_state_reads_differently_in_both_languages`；`claude_extensions_show_same_name_sources_and_gate_full_legacy_export` | ✅（补前界面只断言了已发现/未接入/旧条目） |
| 扩展 | 缺缓存（Codex） | `cached_catalog_without_cache_refuses_without_scanning_or_writing` | ✅ |
| 扩展 | 同名多来源 | `same_name_entries_stay_listed_per_source_with_documented_rules_only`、`claude_extensions_show_same_name_sources_and_gate_full_legacy_export` | ✅ |
| 扩展 | unsupported | `claude_extension_reads_reject_other_platforms_and_remote_without_fallback`、`fixed_resource_missing_read_does_not_create_and_unsupported_never_falls_back` | ✅ |
| 扩展 | 只读：无写入 / 无执行 / 旧数组不变 | 上述发现测试（无 backups/locks、settings 字节不变、`.ps1` 不执行）；`legacy_export_is_read_only_and_saves_only_new_files_inside_the_fixture` | ✅（“无网络”为源码推断：发现路径无网络调用，没有运行时断言） |

变异验证（改坏生产代码 → 测试变红 → 手工逐字节改回，`git diff` 哈希核对一致）：缓存面板改读全历史；KPI 去掉“有记录才显示金额”判断；扩展“不安全路径”改成“不可读”文案；编辑页空凭据也提交；MCP 复制保存只校验来源平台。五处均变红。另试“未改说明也提交”，测试仍绿，已如上记为 ◐。

交回维护者（未修，需要设计决定）：
- Claude 首页没有同步过期标记，只陈述全局时间。#161 第 4.1 节未要求标记，第 8.3 节把“同步过期”列为夹具项；是否在 Claude 首页加过期判断待定。
- 观察：配置列表外层焦点容器 `list-focus` 在短列表时仍拉伸到可用高度（2 行时约 731px），实色行本身按内容收紧；聚焦描边会画在整个拉伸区域。是否算违反“实色列表随内容收紧”待定。

仍未覆盖：四个原生场景（需要真实窗口）；界面层 dirty-key 省略的可观察证明（需要能截获提交补丁的测试缝，属设计变更）；同帧双击重复打开确认；方案页换环境的界面测试；扩展只读“无网络”的运行时断言；Codex 作用域下废纸篓可用的正向断言；动作栏 `profile-apply` 的“启用并应用”文案。以上除原生外都可以补，本节因范围与上下文预算未做。

门禁（最终代码，真实退出码）：`just desktop-fmt-check` 中途 1（只报本节新增测试所在的 4 个文件，逐文件 rustfmt 修复后）→ 0；`just desktop-lint` 0；`just desktop-test` 0（Python 16 OK；accesskit 3 通过；lib 326 通过、4 忽略；native_shell 15 通过、1 忽略；native_ui 2 通过；runtime_shutdown 2 通过、1 忽略）；`just desktop-test-ui` 0（native_shell 15 通过、1 忽略；native_ui 2 通过）；`ccr-desktop` 下 `python scripts/desktop.py cargo check --locked --all-targets` 0；根目录 `cargo check --workspace --all-targets --locked` 0。未改共享 crate，未跑根 `just test`。已知不稳定测试本轮未出现。

原生验收：not_run。

## 原生四场验收豁免（2026-11-12）

用户在 2026-11-12 的 Claude 会话 `a9201481-5b17-4a1c-90a8-851a4fa074b7` 中，对「原生四场验收是补做还是豁免」回复「豁免」。#161 第 8.4 节的四个原生场景不再补做。

豁免不等于通过：四场仍是 not_run，没有截图、外窗、客户区或 DPI 观测，headless 控件测试不算原生验收。

豁免时的自动门禁：`dev`、HEAD `ccd7db98` 加当时工作区，`just desktop-ci` 退出 0，`just ci` 退出 0（16 个阶段）。
