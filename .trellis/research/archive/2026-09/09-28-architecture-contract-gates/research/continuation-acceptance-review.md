# 后续验收范围独立核查

日期：2026-09-28。审查者：`/root/implement_t10_specs`。

## 结论

仍需明确保留的验收项为：最终变更后的正式根 aggregate、T06 明列的 macOS 进程树清理矩阵、T08 Codex 仅改 model 后真实保存与文件重新读取。主会话已把 Codex 精确 fixture 交给 implement_t09，本报告不提前授予新测试通过。

T08 设计要求后续 Web/native smoke 分别记录。已有实际 Web 错误/不可用状态视觉和 React/CodeMirror 自动化；正常表单的成功浏览器交互与原生 smoke 仍需各自证据。当前没有已验证的完整 desktop 临时用户根隔离，不能仅设置 CCR_ROOT 后运行真实桌面应用。

真实 provider OAuth、真实 SSH 账户、交互 TUI 和全面原生视觉属于明示未测扩展。T11 设计明确不调用真实 OAuth。不把真实账户需求加入原本定义为合成 fixture 的 AC，也不把已经运行的真实进程/loopback 测试降格为纯 mock。

## 责任与执行边界

按 check.jsonl → prd.md → design.md → implement.md 读取 T10，然后读取实际存在的 `.codex/agents/trellis-check.toml`、trellis-check skill、父 integration-report 和 continuation-scope。复用此前读取的领域规范，核对 11 份独立报告、39 项 AC 和相关源码。UI 最小入口参照 ccr-ui-visual-workflow。

专用 trellis-check 派发再次因 `agent thread limit reached` 失败。本次为主会话授权的非作者范围复核。本人此前编写的 T10 规范、矩阵和证据仅作状态对照，其独立结论采用 gate owner 的 cross-check-specs。

仅写本报告与同名 JSON。没有运行 Cargo、bindings 或完整 CI，没有操作浏览器/原生 UI，没有修改源码、原日志、矩阵、任务状态或父汇总，没有访问真实账户。并行 owner 的新修正由后续指定审查验收；文件已经出现不表示检查通过。

## 最少充分验证清单

### V01 正式本机门禁

- 分类：正式最终验收必需。
- 要求：T10 `prd.md:28,35`，`design.md:11,14`，`implement.md:34-36`。
- 现有证据：父 `integration-report.md` 和 `research/root-delivery-after-utc-ci.json/.log` 记录上一冻结版 workspace tests、Release 通过，根 CI 在 audit 失败；独立完整 Tauri 通过。原工作区 frontend-check 仍记录两脚本 5 条 no-console，隔离授权源码副本成功不覆盖原失败。
- 最小入口：新锁文件 owner 完成后审计两个 Cargo.lock 并运行依赖治理；获得明确授权的脚本修正须通过原 `bun run lint:ci`；最终冻结后由 root 执行一次 `just ci`。不为覆盖计数重复已经通过的同源过滤集，不降低门禁。
- 本机可完成。依赖/脚本新日志本阶段没有独立授予通过。

### V02 macOS 原生进程清理

- 分类：规划必需阻断。
- 要求：T06 `design.md:13` 明列 Windows/Linux/macOS tree cleanup matrix；T10 `design.md:11` 要求保留该矩阵。
- 源码：`.github/workflows/tauri-rust-ci.yml:51-69`；`justfile:1554-1556`；`crates/ccr-core/src/core/process_gateway.rs:698,731`。
- 已有证据：Windows 10 项 Tauri + 7 项 core、Linux Rust 1.95 core 9 项通过；macOS 未运行。来源为 T06 `check-process-smoke-final.log` 与 `check-core-process-linux-final.log`。
- 最小入口：在同源码、同锁文件、Rust 1.98.0 的 macOS 设备或 runner 上运行 `just tauri-process-smoke`。现有 recipe 已同时执行 Tauri `process::gateway` 与 core `core::process_gateway`，无需再建入口。记录非零测试数、父进程先退出/Drop、后代终止和超时结果。
- 当前 Windows/WSL 不能代替 macOS；需要设备或获授权 runner，不需要真实账户。

### V03 Codex model-only 文件往返

- 分类：本机可补的精确持久化验收。
- 要求：T08 `prd.md:34`、`design.md:11`；父 `design.md:64` 明列 typed read → patch → reread。
- 源码：`ccr-ui/tests/platforms/settings-lossless.smoke.test.ts:39,84-89` 由 mock transport 模拟写回；`commands/codex_settings.rs:18-35` 是实际 read/apply/write 路径；`commands/codex.rs:2137,2177,2233` 三例分别为文件读取、animations 内存 merge/serialization、null 内存 merge。
- 现有三例全部通过，但没有同次仅改 model 的真实文件保存→reread。该项是证据缺口，没有据此确认产品错误。
- 最小入口：复用 TestProcessEnv 和临时 Codex 根，合成 TOML 含两项 notifications 及非目标扩展；调用真实更新入口或同一真实 read/apply/write 链，仅提交 model；重新读取文件并逐项比较数组、model 与未改字段。新增 selector 实际存在后只跑该最窄用例。主会话已指派 implement_t09。
- 不需要 WebView、真实账户或网络。

### V04 正常 Web 表单与原生 Settings smoke

- 分类：T08 指定后续 smoke；全面原生视觉属于更大范围。
- 要求：T08 `design.md:13`；T10 `implement.md` 的“UI 改动做相关 Web 行为/视觉验证并另列 native 限制”。
- 已有：父 `research/web-preview-preflight.md:23-45` 保存实际 Web 错误、重试、Commands 不可用布局和宽度观察；React/CodeMirror 路由用例通过。旧 browser mock 安装曾失败，不能当正常表单保存成功证据。
- Web 最小入口：在 ccr-ui 运行 `bun run dev:web -- --host 127.0.0.1 --strictPort`，独立标签页明确安装 synthetic IPC 后检查 managed 锁与原因、未知枚举、raw 编辑器与冲突可达性。记录底层 mock 边界。
- Native 最小入口：先证明完整 desktop 启动隔离，或使用可丢弃 OS 用户环境，再以合成配置检查同一路由及 raw 入口。生产 CSP 必须在嵌入前端的原生模式验；`tauri.conf.json:74` 的 `devCsp=null` 使开发模式成功不足以证明生产 CSP。
- 未找到现有完整启动隔离 harness。`TestProcessEnv` 仅是测试级 guard；详细定位见后文。当前不直接启动桌面应用读取真实用户配置。

### V05 Linux pinned 1.98 平台结论

- 分类：授予 Linux 正式平台/hosted required 结论所需；不把旧 MSRV 定向结果扩大为完整平台结论。
- 要求：T10 `design.md:11`；`rust-toolchain.toml:3`；`.github/workflows/ci.yml:31-59`；`.github/workflows/tauri-rust-ci.yml:27-49`。
- 现有环境：WSL Ubuntu-24.04；cwd `/mnt/d/Documents/Code/Github/ccr`；target `/tmp/ccr-architecture-linux-01a0e781`。cargo/rustc/rustdoc 来自 `/home/lyh/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin`，实际 Rust 1.95。T06 `check-report.md:46,57` 记录子进程 RUSTUP_TOOLCHAIN=stable、RUSTC/RUSTDOC 绝对路径、PATH 前置对应 bin、清空 wrapper；未改全局配置。
- 旧进程命令：`cargo test --offline --locked --target-dir /tmp/ccr-architecture-linux-01a0e781 -p ccr-core process_gateway -- --nocapture`，9 passed。T05 `check-linux-oauth-pending.json` 保存实际 cwd、绝对 cargo 路径、`test --locked -p ccr-codex oauth_pending -- --test-threads=1` 与 5 passed。T02 `root-linux-repository-final-alias.json` 和 `root-linux-write-journal-faults-final-alias.json` 分别记录 repository 17 passed/1 ignored、journal 9 passed 及前后指纹。
- 最小可行入口：先只读核对实际 rustc/cargo 版本；现有记录未证明安装了 1.98。具备 pinned 工具链与 workflow 声明的 native 依赖后，先跑 `just tauri-process-smoke`。
- 若授予正式 Linux 结论，再运行 ci.yml 的 root version/governance/fmt/lint-strict/test/check-workspace 和 tauri-rust-ci.yml 的 `just tauri-ci`；hosted required 的完整宣称还需对应 coverage/MSRV lanes，保留阈值。无需重复 Windows 全套来证明 Linux。
- 现有 WSL 可承载；本轮未安装工具链、运行 Cargo 或读取新运行状态。

### V06 明示未测扩展

- 依据：T11 `design.md:12-13` 使用 fake listener/storage、本地 loopback、受控 socket/server，明确不调用真实 OAuth；T09 `design.md:11-12` 明确 real BaseAuth + mock domain 和受控 Query promises；父 `design.md:74-76` 保留原生视觉和外部账户边界。
- 已有 Windows 实际 loopback/socket/body 与 pending store 证据；TUI adapter/presenter 与 48 帧通过；RemoteSettingsEnvironment 和临时 Local 文件检查环境身份与目标捕获。真实 OS 资源释放可以由这些本机 fixture 验证，不要求登录真实服务。
- 最小充分范围不扩大到真实 provider OAuth、SSH 账户保存或交互 TUI。未来如要求真实远程保存，需要明确的可丢弃 SSH/WSL 主机和配置根。全面原生视觉继续列未测，与 V04 指定功能 smoke 分开。

### V07 fix_commit 追溯边界

- T10 `prd.md:36` 要求修复 commit 字段，`:42` 又把提交列为不在本任务范围；当前 9 组 P1 保留 `fix_commit=null` 和源码/差异 SHA。
- 不虚构 commit，不为填字段自行提交。主会话区分行为证据完成与等待获授权后填入的提交追溯；无需为该字段重复产品测试。

## 39 项 AC 状态

同名 JSON 保存全部 39 项 AC 的 prd/design 精确行、已有测试源码与选择器、日志、mock/平台边界及最小验证入口。除 V02/T06.AC3、V03/T08.AC1、V04/T08.AC3 和 T10 最终门禁/追溯外，没有发现需要为数量重复运行的同源行为用例。

T10 AC2/AC3/AC4 是本人原规范/矩阵/保留记录，只作状态对照，其独立结论引用 gate owner 的 `cross-check-specs.md`。没有用本报告改变任一 AC 或 task.json 状态。

## 旧独立报告的未修项

| 来源 | 原记录 | 当前状态与最小动作 |
| --- | --- | --- |
| T01/T02/T03 | generated 尾空格、fmt、完整 bindings 待验 | 后续 T10 事务生成、完整 Tauri、全仓 diff/fmt 证据关闭；旧日志保留。 |
| T04 | 诊断局部问题 | 独立报告无剩余本任务已确认问题；最终 workspace 覆盖实际 binary 契约。 |
| T05 | Windows 持锁元数据 OS33 | T10 改为持锁句柄读取，原断言保留；native 1 passed 与最终 workspace 通过。 |
| T06 | usage 控制投递失败、dateWindow DST | T11 真正 owner 策略 70+2、T10 真实时区回归关闭。 |
| T07/T08 | T09 在途类型/lint、dirty reset/环境身份 | 后续 T09 独立检查与最终 TS/前端检查关闭；5 条原脚本 lint 另留。 |
| T11 | export 失败清空目录、guard 失败 | T10 精确目录恢复与最终 230 文件 guard 通过关闭；启动异常原因未查明。 |

仍需保留或另轮确认的项目：

1. **正式 audit/lint**：上一冻结版 rustls 审计和两脚本 lint 失败确实存在。当前另有 owner 处理新锁文件及授权修正，本阶段未对新结果授予独立通过。
2. **Linux 测试 imports**：T05 `check-report.md:61-63` 和旧 Linux 日志确认 `codex_process_service.rs` 的两个 Windows-only 测试符号产生 unused imports。初始检查仍存在，报告落盘时已观察到 root 改为 `#[cfg(windows)]` 单列导入。新修改等待本轮独立验证；最窄入口为 Linux 1.98 `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings`。不再把已改源码写成未修。
3. **Tauri 额外 all-targets lint**：T09 `check-report.md:52`、`research/backend-clippy.log:43-118` 记录 main/state 的 items_after_test_module、Claude bool_assert_comparison、Codex await_holding_lock。正式 bin Clippy 不覆盖这四项测试 lint。并行修正若发生须另验；当前不扩大或降低正式 gate 定义。
4. **Windows 运行不稳定**：doctor grandchild.pid 缺失、测试程序启动异常原因未查明；后续通过不是根因修复。`cross-check-specs.md:103-111` 的自然退出可能误满足死亡断言是源码风险，未被复现为产品错误。若新正式运行再失败，先采 exe/source SHA 和进程启动/终止证据后最窄调查，不无界复跑全套。
5. **旧 UI 文案缺失**：父 `web-preview-preflight.md:9,25` 记录 `claudeSettings.subtitle`。初始审查确认 settings-claude.ts:44 引用、BaseSettings.tsx:60/93-95 消费，zh-CN:1492 和 en-US:1537 对应对象无 subtitle。已反馈 root；后续最小补键与新 i18n 日志另作非作者核查。这是低优先级界面问题，不扩大为架构阻断。

没有删除原日志或把 0 匹配用例计为通过。V03 是精确持久化证据缺口，V04 是隔离和原生验证缺口，均未被表述为已证产品错误。

## 完整 desktop 隔离定位

- `ccr-ui/scripts/dev-web-warm-start.mjs:37` 只启动 Vite；plain Web 无 native invoke。成功合成 UI 必须先明确安装 mockIPC。
- `ccr-ui/src-tauri/src/test_support.rs:12` 是测试级 TestProcessEnv，不封装整个桌面启动。
- `main.rs:125` 先 init_logger；`crates/ccr-core/src/core/logging.rs:147-148,290-299` 直接使用 home 下日志目录，并执行创建、旧日志清理与权限操作。
- `crates/ccr-db/src/database/mod.rs:68-74` 的应用 ccr-ui.db 固定使用 home。usage archive 路径支持 CCR_ROOT/CCR_DATA_DIR，但 `:128-131` 迁移仍读取 home 和应用 DB。
- `main.rs:194,201-255` 恢复 pending listener 并发现 WSL。仅设置 CCR_ROOT/CCR_DATA_DIR 未证明启动只访问临时用户根。
- 本机可安全继续临时目录 Rust/handler fixture 与纯 Web 合成 UI。完整 native smoke 需先有被验证的全启动隔离或可丢弃 OS 用户环境。

## 交接

优先收敛本机新 Codex 保存回归、锁文件审计和获授权 lint，然后在最终冻结点执行一次 aggregate。macOS 必需矩阵和安全隔离的 native smoke 保持明确开放。新增依赖/脚本的非作者审查按主会话后续指派执行。
