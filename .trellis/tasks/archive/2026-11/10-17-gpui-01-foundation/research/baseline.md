# Tauri release 基线预检（2026-10-17）

结论：已完成本机环境、隔离路径及构建预检；原生性能基线 **not_run**，GPUI/Tauri 相对指标 **not_comparable**。没有启动 Tauri、修改用户 HOME/USERPROFILE、访问真实账户、采集 GPUI benchmark 或生成虚构样本。Windows 原生控制由主会话确认可用，不能将它记为 blocker。

本记录只负责基线研究和日志，不代表 01 或父任务验收通过。用户已批准现有方案；以下为实施中发现的验证缺口。

## 产物和当前构建

- 环境：`baseline-environment.json`，包含硬件、工具链、git commit、锁文件及已有构建产物 SHA-256。
- 原始预检：`outputs/desktop/perf/tauri-baseline-2026-10-17/`。
- 实际构建：在 `ccr-ui/src-tauri` 执行 `cargo build --locked --offline --release --features custom-protocol --bin ccr-desktop`，退出码 **0**，Cargo 报告 **8m 21s**。收据 `tauri-release-build.json`，完整日志 `tauri-release-build.log`。
- 应用未运行，截图与全部性能原始样本未采集。`metrics-not-run.json` 明确保存空 samples 和 not_run/not_comparable，不能据此计算百分位数。

## 实际命令与退出码

| 命令 / 预检 | 退出码 | 结果 / 日志 |
| --- | --- | --- |
| `rustc -Vv` | 0 | 1.98.0、LLVM 22.1.8、x86_64-pc-windows-msvc；rustc-version.json |
| `cargo -V` | 0 | 1.98.0；cargo-version.json |
| `rustup target list --installed` | 0 | 仅 Windows MSVC；rustup-targets.json |
| `bun --version` | 0 | 1.4.3，仓库 packageManager 为 1.4.0；bun-version.json，未安装或更换工具 |
| `mbx setup --status` | 0 | 当前 Cargo shim 生效；mbx-setup.json |
| `mbx doctor --json`（sandbox） | 1 | D:/mbx-cache 不可写；mbx-doctor.json |
| `mbx doctor --json`（authorized） | 0 | cache/target/restore 均通过，说明前次是 sandbox 边界，不是磁盘 ACL 损坏；mbx-doctor-authorized.json |
| `mbx cache dir` / `mbx settings get cache_dir` | 0 / 0 | D:/mbx-cache，未修改配置；mbx-cache-dir.json / mbx-cache-setting.json |
| `cargo metadata --manifest-path ccr-ui/src-tauri/Cargo.toml --locked --offline --no-deps --format-version 1` | 0 | 独立 workspace_root 为 ccr-ui/src-tauri；tauri-metadata.json |
| `git rev-parse HEAD` | 0 | 1dfe776b29fc6b52c8a9d0a6c8e49efbb8026d48；git-head.json |
| `powercfg /getactivescheme` | 0 | 平衡；power-scheme.json |
| `Get-CimInstance` 四个硬件类（sandbox） | 每项 1 | 拒绝访问；hardware-preflight.json |
| `Get-CimInstance` 四个硬件类（authorized） | 0 | 只选 OS/CPU/GPU/驱动/内存字段，无机器名、用户、序列号；hardware-authorized.json |
| Win32 GetDpiForSystem / GetProcessDpiAwareness / GetSystemMetrics | 0 | display-preflight.json；探针进程 DPI unaware，96 DPI 不能当作目标应用真实 DPI |
| `Get-FileHash -Algorithm SHA256` 五个输入文件 | 0 | input-hashes-before.json；fixture 不存在，fixture hash 为 null |
| `Get-Command` 工具解析 | 0 | tool-resolution.json；WPR/WPA 已安装，PresentMon 未在 PATH 找到 |
| `wpr -profiles` | 0 | wpr-profiles.json；存在 GPU/DesktopComposition/HTMLResponsiveness 等 profile；未开始或停止 trace |
| `cargo build --locked --offline --release --features custom-protocol --bin ccr-desktop`（cwd: ccr-ui/src-tauri，authorized） | 0 | release optimized 构建成功，8m 21s；tauri-release-build.json / .log；没有启动 exe |
| 构建后 SHA-256 对比 | 0 | input-hashes-after.json；根/Tauri Cargo.lock、Bun lock 与 dist/index.html 均未变；exe 已更新 |
| 24 个 JSON 解析、owned files 尾随空白与状态 invariant 检查 | 0 | json-validation.json；确认 release exit 0、tauri_launched=false、performance_result=not_run |
| `git diff --check --`（两份研究文件） | 0 | 文件仍 untracked，另以显式尾随空白检查覆盖新增文件；没有对共享工作区执行修复 |

首个 release 构建包装命令将日志路径错误地相对到 `ccr-ui/src-tauri`，重定向失败，Cargo **未执行**；PowerShell 返回 0 不能视作构建通过。已将日志改为绝对仓库路径后重新执行真实构建。指定的 `C:/Users/lyh/.codex/RTK.md` 不存在；不存在的路径读取没有被当作规范内容。

## 环境限制

Windows 11 Pro build 26300；Intel Core Ultra 9 275HX（24 核/24 逻辑处理器）；68,143,853,568 bytes 内存；NVIDIA RTX 5090 Laptop GPU 驱动 32.0.16.1714、Intel Graphics 驱动 32.0.101.6733。CIM 同时报出三个虚拟显示驱动，不能在没有目标窗口和呈现 trace 时断言实际渲染适配器。

活动显示信息报出 2560×1600 @240Hz 和 2560×1440 @143Hz；尚未设置或证明固定 60Hz。探针进程 DPI unaware，系统 API 返回的 2048×1152 与 96 DPI 可能经过虚拟化；没有将其冒称为 100% 的 native 验收。100%/150%/200% 应逐一读取目标窗口 effective DPI 并采样。

现有 target 是符号链接，指向 `D:/mbx-cache/targets/v1/44061aa3da5bb563d33700d15e5ae7660d5d3da33a41e506f28329282a48c2b6`。保留原链接及缓存，不更改 target 放置、不 clean。构建前已有 exe 的时间为 2026-10-14，不能拿它证明当前源码构建。现有 dist/index.html 保存 hash，但没有进行前端重建；Rust 构建成功也不能证明完整前端与本次 commit 对齐。正式基线前应使用受支持的冻结前端依赖重建 dist，记录完整资产树 hash 后再构建自定义协议 release。

本次成功构建保留了 MSVC 对 command-macros DLL 的库/对象创建 warning，以及 mbx 因两个仓库绝对资产路径缺 stable mapping 而未缓存的 warning；没有隐藏或以修改缓存配置修复。它们没有使 Cargo 返回非零。构建后 exe 的 SHA-256/字节数在环境 JSON 与 input-hashes-after.json 中；应用未运行。

主会话已通过 `@oai/sky list_windows` 确认 Windows native helper 可用。本子 agent 未复制该主会话 UI 日志，也未据此宣称 Tauri 已运行。macOS/Linux 主机和原生证据未提供；仅安装 Windows target，不运行跨平台 native 测量。

## 为什么现有环境变量不足以隔离整个桌面

| 源码 owner / 位置 | 当前事实 | 恢复所需的最小路径注入边界 |
| --- | --- | --- |
| `ccr-ui/src-tauri/src/main.rs:124` | main 初始化 logger；setup 无条件进行 catalog migration、两类 DB 初始化、llmusage discovery、tray refresh、OAuth 恢复和环境注册 | composition root 在初始化前解析一个明确 fixture 描述，并把显式路径传给对应 owner；不以更改全局 HOME 来绕过 |
| `crates/ccr-db/src/database/mod.rs:68` | get_db_path 直接取 dirs::home_dir，创建 ~/.ccr-ui/ccr-ui.db，**不读 CCR_ROOT/CCR_DATA_DIR** | ccr-db 提供接受 app DB 路径和 fixture legacy-home 的工厂；已有默认入口保持兼容 |
| `crates/ccr-db/src/database/mod.rs:101` | create_app_pool/create_usage_archive_pool 均对真实 home 执行 run_all_migrations；archive migration 还会调用 get_db_path | 同时注入 app DB、archive DB、legacy migration source home；仅重定向 archive 输出目录不够；复用既有 migrations/schema |
| `crates/ccr-config/src/models/platform.rs:484` 与 `managers/config/catalog/migrate.rs:29` | CCR_ROOT 只控制 CCR profile/catalog root，migrate_catalogs 遍历 implemented platforms | 使用 fixture PlatformPaths/显式配置路径；保留 CAS、备份、原子写和锁 owner，不重写 profile 持久化 |
| `ccr-ui/src-tauri/src/state.rs:216` | DesktopShellPreferences 默认路径直接取真实 home；load_or_quarantine_corrupt 可能 rename 损坏文件；后续 save 也用默认路径 | AppState 持有明确 preferences path，接入现有 load_from_path/load_or_quarantine_corrupt_at/save_to_path，读写必须同根 |
| `ccr-ui/src-tauri/src/state.rs:357`、`crates/ccr-db/src/services/log_persistence.rs:76` | LogPersistenceService 的配置仅含 retention/flush；持久化通过 database::with_connection 使用全局池 | monitoring owner 必须使用与 fixture app DB 对齐的池；可复用 DbAccess::Pool，不凭空增加文件日志目录。ccr-core logger 另需在启动前选定独立根，不能仅关闭日志级别 |
| `ccr-ui/src-tauri/src/llmusage_adapter/mod.rs:37`、`crates/ccr-usage/src/paths.rs:10` | llmusage runtime discovery 本身不建库；默认发现真实 .llmusage；已有 from_paths/AppPaths::from_root | 直接复用这些显式 path seams，将 fixture 投影库交给 ccr-usage；不添加 SQL owner、不链接 upstream crate；LLMUSAGE_HOME 只解决这一域 |
| `ccr-ui/src-tauri/src/main.rs:188`、`:198`、`:202` | 首次 tray refresh、pending OAuth listener 恢复及本机/WSL/SSH 注册在启动执行 | Codex settings/auth store、SSH host pool/trust root、环境发现和外部进程 runner 均需绑定合成 fixture；不得连接真实账户、恢复真实 OAuth 或探测远端 |
| `ccr-ui/src-tauri/src/platform/local.rs:142` | 各平台目录多数直接取真实 home；GROK_HOME 仅部分覆盖 | platform adapters 使用同一 fixture home/path provider；各领域 manager 已有显式路径构造器时直接复用，避免通用动态 dispatch |
| `crates/ccr-cli/src/application/profile_contract.rs:34`、`crates/ccr-cli/src/lib.rs:100` | profile contract fixture 使用 new_with_home_env，后者改 HOME/USERPROFILE；它是局部 test-support fixture | 不直接调用它作为 benchmark launch harness；复用其合成数据语义，通过显式 paths/provider 生成所需数据 |
| Tauri/WebView runtime | 应用 WebView 存储默认仍属于生产打包身份，单独 DB env 不隔离 localStorage/preferences/cache | 基线 runtime 使用独立 WebView data directory，保持被测产品代码与 release 优化，记录该差异 |

这些是待实现恢复边界，尚未修改上述 owner。S0 的“无用量记录”也必须经历完整生产启动链，不能用无状态空窗口代替。S1 数据应由领域 API/既有 ccr-usage fixture owner 生成 100 profiles、10,000 sessions、100,000 usage records；S2 生成 10,000 候选记录和固定 60 秒日志源。数据文件、manifest、schema/version 和 SHA-256 都要保存；当前均未生成。

已定位可复用的合成数据入口：`crates/ccr-usage/src/fixtures.rs:23` 的 `create_projection_db`（schema 19）、`:153` 的 `seed_bucket` 和 `:245` 的 `seed_event`，由 `test-fixtures` feature 暴露；它们属于既有 SQL owner，不能复制到 GPUI。会话归档继续由 `ccr-db::database::repositories::usage_repo` 的 `UsageSessionArchiveEntry`/upsert API 构建，catalog 通过 `ccr-config` API 构建。生成器应放在测试/采样工具侧；正式应用不因采样而启用 test-fixtures。

## 启动和 action 可采集性

`ccr-ui/scripts/verify-release-window.ps1:54` 仅启动应用、固定等待 5 秒后检查窗口可见性/尺寸，并最终 force stop。它既不隔离数据，也不提供 first-present/action-ready，不运行这个脚本作为基线。`ccr-ui/src/main.tsx:33` 的 React shell mounting、Rust main/setup 日志均不是 OS process-create 或实际显示首帧。

`ccr-ui/src/utils/perfTelemetry.ts` 已有基于 PerformanceObserver 的 FP/FCP/LCP、长任务、INP、route timing，默认由 query/localStorage 控制。可以作为附加 WebView 诊断，但它采用 navigation/JS 时基，不自动等同 OS 呈现时间或用户 action 完成，不可替代合同主指标。浏览器 `measure-vite-route.mjs` 同理。

恢复时应由专用外部 launcher 记录 process-create 的单调时钟和 PID，使用可归属到应用/WebView 进程树的实际 presentation trace，标明共享 WebView 进程的归属限制；同时在真实可执行的首屏 action 处理及对应 UI commit 点记录同一试次 ID 与时钟映射。native helper 可发送输入和取得截图，但截图轮询不能精确代替首帧事件。确认 WebView/browser 与 OS QPC 时钟映射误差，并报告 instrumentation/profiler 开销。不要把 rAF 回调、main 入口、window visible 或“setup complete”当作真实呈现。

本机已存在 WPR 10.0.26100 和 WPA，`wpr -profiles` 可列出 GPU/DesktopComposition/HTMLResponsiveness 等 profile，未来可用于验证呈现事件和进程树归属。profile 存在并不证明所需时间点已经可正确解析；当前没有 ETL、没有启动 trace，也没有更改系统采样状态或安装 PresentMon。

## 恢复顺序与判定

1. 按上述 owner 增加显式 fixture path/provider，覆盖完整启动、读写、WebView storage 和关闭链；通过越界访问拒绝/审计确认全部路径留在 fixture，保持真实 HOME/USERPROFILE。
2. 使用领域 owner 生成 S0/S1/S2，固定 seed/version 和 fixture hash；关闭自动导入/联网，保留同操作下两端一致的后台负载。
3. 对齐当前前端与 Rust release 产物、锁文件、git dirty diff、环境和 effective DPI；先验证目标窗口原生显示、输入、退出和进程树清理。
4. 接入 process-create、first-present、first-action-handled 及输入到对应 UI commit 的同试次 trace；验证计时和 WebView 进程归属，固定 60Hz 并记录电源/后台负载。
5. **先采 Tauri**：热启动 30 次 nearest-rank p95；真实 OS cache 重置/重启下冷启动至少 5 次 median/max；S1 稳定 120 秒 CPU/private bytes/working set；其余按父性能合同，不改变预算。
6. 保留全部原始 JSON、环境/版本/数据 hash、截图和汇总后再测同机同数据 GPUI。没有基线时相对 80% 指标为 not_comparable；空窗口不能替代完整迁移应用验收。

本轮没有请求重新批准方案，没有提交、push、发布或归档任务。后续是否由 02/04 逐域提取路径 seam 或由专门采样修复承担，应由主会话按单一写入 owner 协调，不能在本基线记录的权限下修改产品代码。
