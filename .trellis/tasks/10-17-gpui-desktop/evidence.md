# 规划交付验证

日期：2026-10-17。阶段：planning，未运行 task.py start。

## 已执行

- Trellis create / create --parent --no-start：1 个父任务 + 10 个子任务，task.py list 确认父子关系且全部 planning。
- design_catalog.py show/apply style:codex：生成 ccr-desktop/DESIGN.md，定制后保留源提交与 MIT provenance。
- Bun 执行真实 flattenCatalog()：76 条 route records，18 条 redirect，原始结果保存 research/routes-baseline.json。
- capture_inventory.py：348 个 Windows-inclusive commands / 340 base，39 个命令模块，每项分配 owner；保存源文件 SHA256。
- verify_plan.py：11 个任务、33 个规划文档、106 条 context manifest 引用、44 个浅/深色对比度组合、348 commands 与 76 routes 全部通过结构检查。
- git diff --check：通过。由于本轮文件为 untracked，额外由 verify_plan.py 扫描所有新增任务/设计文件的行尾空白与 Python 语法，避免把 git 的空 diff 当作新文件格式证据。
- 从官方 crate 文档与上游指南核查 gpui-kit 0.7.3、gpui-component 0.7.3 与 gpui-pre 0.3.9 配套关系，来源见 research/architecture.md。

## 尚未执行

没有 Cargo 产品工程、lockfile 解析、build/test/lint、原生窗口、截图、IME/屏幕阅读器、性能基线、三平台运行或安装包验证。justfile 尚未修改；第 01 任务负责基础命令，第 10 任务负责 perf/package 与最终门禁。对比度只是静态 token 计算，不能称为 UI 已验收。

## 环境说明

用户入口引用的 C:/Users/lyh/.codex/RTK.md 不存在，本轮使用了实际可读的仓库指引与技能。命令工具曾长时间无响应，恢复后所有规划结果均重新读取/校验；不将无返回的调用计为成功。

## 复核命令

```powershell
python .trellis/tasks/10-17-gpui-desktop/research/verify_plan.py
python .trellis/scripts/task.py list
git diff --check
```

capture_inventory.py 是初始计划快照生成器；实施后不能直接重跑覆盖已补的 native_action / verification。后续新增源命令需比较 hash、审阅范围差异再更新台账。

## 执行恢复：2026-10-17

- 用户本轮明确批准父任务及全部子任务，授权 01 起依赖顺序实施；不新建任务、不重新规划、不自动提交或归档。
- `git status --short`：退出 0；恢复时仅有十一个任务目录与 ccr-desktop/ 未跟踪，保留全部既有文件。Git 全局 ignore 文件读取有权限警告，不影响状态命令退出码。
- `git rev-parse HEAD`：退出 0，`1dfe776b29fc6b52c8a9d0a6c8e49efbb8026d48`，分支 dev。与规划基线一致，但实施始终以现工作区为准。
- `python .trellis/scripts/get_context.py` / `--mode phase` / `--mode packages`：均退出 0；恢复时无 active task，所有子任务 planning，无先前产品实施证据。
- `python .trellis/scripts/task.py start .trellis/tasks/10-17-gpui-desktop` 与 `.../10-17-gpui-01-foundation`：均退出 0；父任务及 01 为 in_progress，当前 active task 为 01。
- 已加载 trellis-start、trellis-continue、trellis-before-dev、trellis-check、ui、design-md；完整阅读父任务合同、研究、设计与迁移台账。台账的 348 commands / 76 routes 均为 planned，native_action/verification 均为空，没有将任何业务项记为完成。
- `C:/Users/lyh/.codex/RTK.md` 当前不存在；ui 技能首次 sandbox 读取失败，后通过批准的只读 require_escalated 成功加载，无需改写本机技能或全局规则。
- 已派发 trellis-implement 负责 01 工程和命令，另一个 trellis-implement 仅负责隔离基线预检与环境证据。每个文件有单一当前写入负责人，检查角色在实现后接手。

此节仅记录执行恢复，不能作为 01 或父任务 AC 通过证据。命令、编译、原生与性能结果将引用子任务证据。

### 主会话预检

- 台账六个源文件 SHA256 与当前工作区逐一比较：退出 0，全部一致。此为施工前快照核对，不重新生成台账。
- 根 `cargo metadata --locked --offline --no-deps --format-version 1`：退出 0，workspace_root 为仓库根，default-members 只有 ccr，workspace 内没有 GPUI/Tauri package。
- 根 `cargo tree --locked --offline -p ccr --edges normal --prefix none`：退出 0，834 行依赖树，未出现 gpui/gpui-kit/gpui-pre/tauri 节点。
- computer-use 技能已加载；独立 node_repl 中 `import('@oai/sky')` 与 `sky.list_windows()` 成功。Windows 原生控制可用，当时没有 CCR 目标窗口。这不构成 GPUI 窗口运行证据。
- Tauri 安全基线预检发现 setup 的迁移和偏好路径仍触达真实 home；仅重定向 CCR_ROOT/CCR_DATA_DIR 不足以满足 fixture 隔离。因此尚未启动 Tauri 采样，待基线子 agent 保存源码定位和环境记录。未采样项不得判 pass。

### 原生与基线阶段结果

- Windows GPUI debug 产物已由主会话实际启动，完成 Unicode、真实中文 IME 组合/候选提交、选择和正常关闭，关闭后无剩余 ccr-gpui 进程。完整操作、产物 hash、截图与限制见 [native-probe.md](../10-17-gpui-01-foundation/research/native-probe.md)。这是输入探针证据，未算入任何业务 command/route 完成数。
- Tauri `cargo build --locked --offline --release --features custom-protocol --bin ccr-desktop`：退出 0，8m21s。构建前后 root/Tauri Cargo.lock、Bun lock、既有 dist hash 未变；未重建前端或启动旧应用。完整报告见 [baseline.md](../10-17-gpui-01-foundation/research/baseline.md)。仅证明 Rust release 可构建，不证明当前完整前端可交付或性能达标。
- Tauri 性能基线仍 not_run，相对指标 not_comparable；GPUI 性能未采样。未获 macOS/Linux 真实构建/原生 receipt。原生 helper 的 accessibility 查询返回 null，role/name/state 尚待 03 排查。
- migration-inventory.json 仅追加 execution 元数据，保持全部 348 command / 76 route planned；没有重跑 capture_inventory.py 或覆盖项目原有 native_action/verification。

### 本次执行最终范围

- 已交付并独立检查 01 的 Windows 工程基础：ccr-gpui 独立 workspace/lock/target/identity，14 个 desktop-* 公开入口，真实 Input 原生探针，版本脚本双端与隔离/退出码回归。Windows desktop-ci、audit、MSRV 1.95、实际 desktop-dev 和中文 IME 有回执；01 AC1.1–1.4 为有平台边界的通过。
- WSL Ubuntu 使用真实 Linux Rust 1.98 与既有 stable=1.95 完成 locked all-targets check。标准 release 缺 xkbcommon-x11 开发 linker 入口；仅用 outputs 下链接到已核实系统 runtime 的软链接和单次进程 LIBRARY_PATH 后构建成功，记录非标准条件。最终 doctor 仍失败，未安装系统包。Linux WSLg native 尝试出现 EGL/ZINK 警告且无 helper 可验证窗口，清理后无残留进程，不记 native pass。
- 主会话 `just fmt-check` **0**；根 `just ci-governance-check` 在进程内选择现有 Git Bash 且可访问既有 cache 的最终复验 **0**。保留此前 WSL bash/cache 权限失败；没有为验证隐式运行 fmt/version-sync 或修改全局 PATH。最后新增 Linux doctor 缺包回归后 Python **6/6** 通过。
- 01 尚未通过全部验收，继续 in_progress；缺 macOS build/native receipt，完整 Tauri fixture 路径隔离/真实首帧与 action 计时/同机数据性能样本仍未完成。不是将基线安全缺口视为 helper 不可用；所有具体 owner、路径注入和计时恢复步骤已在 baseline.md 记录。
- 依照批准的依赖与验收顺序，02–10 未开始产品实施。先行完成了 02 配置链源图及 03 Windows accessibility 源码风险调研，均只作为研究，不记完成。父任务 AC1–AC8 未全部满足；348 commands / 76 routes 均未计业务完成。
- Windows release 产物：`ccr-desktop/target/release/ccr-gpui.exe`；Linux 条件构建产物：`ccr-desktop/target/x86_64-unknown-linux-gnu/release/ccr-gpui`。原生截图在 `outputs/desktop/native/windows/`；构建/检查/环境 receipt 在 `outputs/desktop/foundation/`，基线预检在 `outputs/desktop/perf/tauri-baseline-2026-10-17/`。
- 未运行最终全根 `just ci`、独立打包、发布或签名。没有自动 commit、push 或 archive；现有 CLI/Tauri 行为入口和用户数据未被迁移。
- 最终只读结构核对退出 **0**：11 个任务、106 个 curated context 引用均有效，348 command/76 route/18 redirect 与全部 planned/空业务证据 invariant 保持；19 个 foundation/native JSON receipt 可解析，99 个文本文件无尾随空白（明确剪枝 target/二进制目录）。`git diff --check` **0**；Cargo.lock 确认列为可跟踪新文件，未被 ignore 丢失。

## 后续执行：2026-10-18

- 用户明确要求继续实施后续任务；按父 implement.md 的工程依赖与开放验收分开跟踪规则，正常激活 02 services。上节是 01 阶段历史结果，不能将“02–10 未开始”当作当前状态。
- 02 实施由两个 trellis-implement 分别负责共享配置编排/领域适配和有界运行时/GPUI 真实读取链；所有 Cargo manifests/locks 只有前者写入。主会话负责证据、台账和原生操作，独立 trellis-check 在代码交接后检查并修复。
- 定位到旧 Tauri 合约 fixture 会通过 profile_contract 修改进程全局 HOME；本任务改用每个测试独立子进程和显式 CCR fixture 路径，保留原有兼容回归覆盖。尚未运行或通过的检查不计成功。
- 01 尚未解决的 macOS、Linux 原生、Tauri 隔离基线和性能比较继续开放；02 开始不意味着这些验收已通过。
- 02 实际实现、检查、原生和未运行项见 [02 evidence](../10-17-gpui-02-services/evidence.md)。业务台账暂时仍全部 planned，待真实实现与检查后逐项更新。

### 02 核心实现与本回合边界

- 已实现 ccr-app 共享配置列表、Tauri 实际 adapter 委托、GPUI 真实读取/筛选/脱敏详情、有界后台运行时及测试 fixture 的独立子进程隔离。根 47 suites 共 2223 passed/0 failed，Tauri、bindings、strict lint/version/fmt 与共享 crate Rust 1.95 检查有实际通过记录，详见 02 独立检查报告。
- 主会话已看到真实 fixture 列表/当前项回退/筛选与慢读取期间的输入响应；随后真实关闭观察到 AccessKit 无效窗口句柄 panic。精确上游 Windows adapter 窄补丁的 stock 回归实际失败、修复后 3 项通过，补丁后的 native 退出仍需复验。
- 用户明确要求只验证核心功能，停止放大缩小等非核心验证；窗口缩放/DPI矩阵记 user-deferred，不记 pass。用户恢复 Computer Use 的请求已执行，但 helper 仍在首次调用立即返回 Esc stop，遵守停止边界，两次本次 fixture 进程均按精确身份清理，native 记录 interrupted。
- 台账现在只有 list_configs 有真实 native_action 和部分核验引用，状态 implemented_pending_native_exit；347 个命令及全部 76 routes 仍 planned，完成计数仍为 0。没有重新生成台账或以原生占位页充数。
- 02 继续 in_progress，03–10 没有开始产品实施。01 原有跨平台/性能缺口保留，最终 AC1–AC8、完整 just ci 与独立打包未完成。没有 commit、push、发布、签名或归档。

### 02 验收完成，继续 03

- 上节为历史中间态。最新 Computer Use 续验实际完成读取、空/命中筛选、当前项回退、脱敏详情、缺失 catalog 错误与 Refresh 恢复、慢读取取消和正常退出；三次原生进程实际退出码均 0、stderr 空，截图和 receipt 在 outputs/desktop/services/native/。独立 check 已复核，02 AC2.1–2.5 通过并标 completed，不归档。
- 补丁后独立 desktop Rust 1.95 locked all-targets 复验也退出 0；原生取消样本不是性能预算采样。父任务性能与其他 OS 缺口没有变化。
- 正常 task.py start 已激活 03；两个 trellis-implement 分别负责原生外壳与注入式 UiStateManager/后台适配，manifest/lock 写入单一 owner。主会话负责原生、任务和台账，待明确冻结后交独立 trellis-check。
- 台账保留原始 348 commands / 76 routes / 18 redirects；仅 list_configs 标 verified_core，业务 command 完成数 1、完整 route 仍 0。未重跑 capture_inventory.py。缩放与 DPI 验证继续 user-deferred，不能视为 pass。

### 03 核心工程交接，04 实施中

- 03已实现真实原生外壳、主题/语言会话设置、导航/焦点、配置虚拟列表/详情返回、同节点语义、命令收藏与已有历史读取；独立check自修删除确认与整行点击两项实质问题，领域127项及desktop/strictlint/check/fmt/MSRV门禁通过。
- 最终6e89原生错误重试、整行点击和慢读取消退出有实际receipt/screenshot；取消生命期26.372718s<30s人工延迟、native0。之前f692实际筛选/详情返回/收藏持久重读及所有产物退出均保留精确hash。独立检查核对6个原生receipt和8张实际PNG，未把时序不充分/未启动尝试计通过。
- 03仍in_progress；UIA/IME、user-deferred矩阵、系统外观实际事件/几何差异等未完成。依父工程依赖规则，源码/核心工程可消费并正常start04，不宣称完整03/父迁移完成。
- 父台账累计verified_core命令3、完整route0；其余03项有test验证或08真实完成事件待接理由。保留原348/76/18，不重新capture或以空页面充数。
- 04由三个trellis-implement分别负责领域生命周期/注入、实际共享策略/Tauri委托和单一Cargo写权、原生编辑/转换器；主会话协调，独立check在冻结后接手。具体边界与命令证据见04/evidence.md。
