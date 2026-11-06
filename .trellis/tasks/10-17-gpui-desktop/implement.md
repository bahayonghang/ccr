# 实施顺序与验证

## 状态与开始条件

2026-10-17 用户已明确批准现有父任务及全部子任务，授权从 01 按依赖顺序执行。父任务与 01 已正常 start；后续子任务仅在依赖满足后激活，不重复请求实施批准。父任务负责范围与最终整合，不直接让一个 agent 同时改全部模块。

| 顺序 | 子任务 | 前置依赖 | 独立交付 |
| --- | --- | --- | --- |
| 01 | gpui-01-foundation | 无 | 三平台构建探针、Windows 原生初验、独立工程和 desktop-* 基础命令 |
| 02 | gpui-02-services | 01 | 共享应用层、线程/生命周期边界、真实配置读取链 |
| 03 | gpui-03-shell | 01、02 | 轻盈玻璃 token/单层 sheet、材质探针与输入/可访问性证据 |
| 04 | gpui-04-profiles | 02、03 | 配置与 profile 操作闭环 |
| 05 | gpui-05-platforms | 04 | 各平台设置与认证闭环 |
| 06 | gpui-06-resources | 02、03、05 | MCP 与扩展资源闭环 |
| 07 | gpui-07-insights | 02、03 | 用量/会话/观测闭环 |
| 08 | gpui-08-jobs | 02、03、04 | 命令/同步/签到任务闭环 |
| 09 | gpui-09-environments | 04、05、06、07、08 | Local/WSL/SSH 与系统生命周期 |
| 10 | gpui-10-acceptance | 01 至 09 | 完整台账、性能、三平台与交付 |

任务树本身不强制依赖，执行者必须读每个子任务 implement.md。可以在前置满足后调度不同功能，但共享 Cargo、ccr-app 契约和 DESIGN.md 只有当前负责者编辑，避免并行重写。dispatch 使用 Trellis implement/check 角色并加载已填写的 jsonl，不因角色改变权限。

## 2026-10-27 视觉增量执行门

后续恢复：用户已明确要求继续实施当前 GPUI 任务，本轮执行最新视觉增量与既有业务计划。恢复记录见 03 implement.md 及父/04 task metadata；历史暂停交接保留原文。已完成的领域/共享代码不重复施工，04 真实原生流程发现的缺陷先修复、复验，再推进依赖它的 05。

历史规划时的暂停约束已由用户后续恢复指令解除，详见 [续作交接](research/resume-20261027.md)。视觉顺序与所有权仍遵循 [airy-glass-plan](research/airy-glass-plan.md)：03 探针与 before 基线 → DESIGN/token/shell → 04 现有页面视觉回归 → 05 至 09 随域接入 → 10 汇总 AC9。01/02 的既有成果不重做，历史证据不覆盖，窗口尺寸/DPI 矩阵仍 user-deferred。当前优先关闭 04 已观察到的原生焦点与反馈缺陷，05/07 可并行推进不依赖该写入流程的域迁移；共享集成由单一 owner 负责，检查与候选构建串行执行。

## 起步步骤

### 2026-10-18 继续执行的依赖判定

用户在 01 工程基础已实施并检查后要求继续后续任务。将可消费的代码/接口依赖与未完成的平台、性能验收分开跟踪：01 的 Windows AC1.1 至 1.4 已验证，足以支撑 02 的共享服务实现；macOS、完整 Tauri fixture 基线和原生矩阵继续作为开放验收项，01 不标完成。后续按代码依赖顺序推进，不能借此豁免任何子任务或父任务的最终验收。仅在实际依赖代码尚未可用或发现兼容/安全问题时阻断相应实施，不让无关的外部验收证据阻断全部后续代码。

1. 对第 01 任务重新核查版本与平台系统要求，先生成可重复的最小 native probe。
2. 记录 Tauri release 基线，禁止把两端 debug/release 混比；基线数据根独立。
3. GPUI 包可编译、能显示文本并正确关闭后，再做主题和业务。先验证中文 IME 和图形兼容，早暴露平台风险。
4. 新增各 desktop 命令时同步写 command graph 检查，执行 just --list 和 dry-run，并实际运行基础命令。
5. 其余每个子任务按“共享逻辑 -> adapter -> native UI -> 回归 -> 台账证据”推进。

## 验证层级

- 规划现在：task.json 父子关系、三个计划文档、manifest 引用存在、需求映射、无待填写占位符、设计对比度计算、git diff --check。
- 01 起：desktop-doctor/fmt-check/check/lint/test/build，独立 target/lock 验证；版本与命令治理小测试。
- 02 起：根 just version-check / fmt-check；相关 crates 单测；共享接口影响 Tauri 时执行对应 Tauri tests/typed bindings gate。
- 数据写入/认证/任务：失败与冲突矩阵必须覆盖，不只成功路径；secret-write-check 和原有 security contracts 继续有效。
- 10：desktop-ci/audit、性能原始数据、三平台 native 截图与手工交互记录；just ci 最终整合，不自动 fmt/version-sync 修复。
- WebView 旧表面视觉若有变化才使用 ccr-ui visual workflow；GPUI 窗口不用浏览器截图冒充验证。

## 易出错文件与回退

- 根 Cargo.toml/Cargo.lock：仅增加中立共享 crate 及必要依赖，不把 GPUI 拉进根默认图。
- ccr-ui/src-tauri/Cargo.toml/Cargo.lock 与服务/命令：保持 wire API 与 existing tests，迁一个域验证一个域。
- scripts/version/version-sync.{ps1,sh}：新增 GPUI manifest 时两端及其测试同时更新。
- scripts/drift/check_dependency_drift.py 与 scripts/ci/ci_surface_policy.py：在 10 阶段覆盖新 surface；不改现有 required 名称和阈值。
- 新锁文件与平台配置：出现不兼容时回到候选验证，不临时浮动 git 或混用核心。
- 数据写入测试只碰 fixture。应用回退仅改变启动入口，不回滚或删除真实数据库。

## 交付证据

每个子任务写 evidence.md，含实际命令、退出码、平台/工具版本、截图或日志路径、未运行项。父任务的 command/route 台账每项关联 owner、native 行为、测试与完成状态。最终不允许以“页面存在”代替业务闭环，以“编译通过”代替 native 运行，以“使用 GPUI”代替性能达标。

## 2026-11-05 续作归档硬门

当前检查点与子任务提交见 [开放验收报告](research/open-acceptance-20261105.md)。01 与 09 仍未独立归档，父任务保持 `in_progress`，不运行 archive 或 finish 脚本。09 切片的最新根 `just ci` 为 0，不替代缺失的子任务验收。

只有以下三个条件同时成立才允许运行父任务归档：01 已独立验收并归档 **AND** 09 已独立验收并归档 **AND** 发布门 `just ci` 退出码为 0。届时使用用户指定命令：

```powershell
python ./.trellis/scripts/task.py archive .trellis/tasks/10-17-gpui-desktop
```

本轮未满足前两个条件，不执行上述命令。子任务工作提交与规划必须先入历史，归档另成纯归档提交；不得把新产品改动混入归档。无新产品或规划改动时不做空提交，不 push、不 amend。

## 2026-11-06 连续核心功能实施

已完成本地工程批次 `13b29f95` 与执行环境/核心操作批次 `0404fa2a`，继续当前用户授权的剩余核心功能。复用既有任务与工作区，未创建新任务/worktree。源码、fixture/headless 与原生收据分别记录，任务状态不因局部提交自动完成。

当前实际接合：Claude rules/output styles 使用现有文件来源；agents/slash commands/plugins 保持 Tauri 的 settings 集合语义，不新增一套 Markdown 代理格式。Codex local agents/source-agent 将原 Tauri state-free 编排提取为共享 owner，保留原载荷，根路径与网络客户端由调用方捕获。资源父页面统一接入现有 ResourcesView，13 个侧栏入口不变。窗口状态复用当前偏好存储和真实 GPUI bounds observer，不新增窗口栈或虚构 watcher；完整位置恢复在同一窗口创建前异步加载。资源/敏感同步/窗口状态批次已提交 e8329cd8，证据见 research/core-resources-20261106.md。继续签到扩展与 OpenCode/Antigravity 文件资源；最后集中门禁、完整 CI、便携打包与隔离 fixture 原生验收。

### 台账对照后的剩余操作闭环

Windows debug/便携 release 已取得 13 入口、三类真实 fixture 保存、偏好恢复和运行任务退出的原生子集收据，见 `research/final-core-20261106.md`。独立收尾检查另确认以下确实缺入口，继续实施而不列为外部阻塞：平台认证关闭及 Codex 账户/模型供应商库管理；完整 WebDAV 解绑与启用文件夹批量推拉；用量 CLI 导入、状态和取消。现有 Refresh 只读数据库，不能冒领导入。仍复用原领域 owner/同一 retained runtime，不加第二窗口或任务栈；WSL、真实 SSH 及用户数据限制不变。

### 2026-11-06 中断后续作

用户要求读取聊天 `01a1af29-f680-7cc3-aad5-e6db1bb7f696`、总结任务状态并继续实施，减少与核心功能无关的 computer use。已读取该聊天并核对 `dev@e8329cd8`、80 项未提交变更；复用当前父任务，不新建任务或 worktree。最后一次 `final-just-ci-complete` 实际 exit=1，失败于用量 CLI 提取后 Tauri 测试引用已移走的 helper，不能认领完整 CI 通过。

1. 完成平台认证关闭、Codex 账户与供应商库操作 → 窄 fixture/真实控件 headless 验证安全确认、原版本和环境拒绝。
2. 完成 WebDAV 解绑与启用文件夹批量推拉 → 复用原 owner，验证修订冲突、备份和部分失败语义。
3. 接通用量导入共享 CLI owner、原生启动/状态/取消与 Tauri 消费 → 验证取消/退出、离页保留、缺 CLI 和自定义来源拒绝。
4. 独立检查完整中断变更，集中桌面三门禁与最终 `just ci`，通过后按实际依赖本地提交；未满足父归档硬门时保持任务开放。

优先复用既有源码、fixture/headless 和 Windows 原生收据；本轮仅在核心操作无法由这些证据判断时追加最小原生验证，不重复侧栏、主题、截图或尺寸/DPI 巡检。保持默认测试并行、`--skip export_bindings` 和既有外部环境限制。

稳定变更独立检查进一步确认三个真实入口缺口（不是旧 planned 状态推断）：Codex 会话导出/克隆/删除、Claude observer 工具/订阅管理、Codex 单账户/全部账户配额。继续消费现有 owner 接入当前 Usage/Sessions/平台入口；Grok 关闭认证必须保留原服务保存轮换令牌的语义，先加捕获根目录 seam 再接原生操作。动态来源、原版本及正确 main/archive 数据库身份需要在写入前绑定。只补必要 fixture/headless 回归，真实配额网络不运行，不因旧子目录已归档而静默丢弃这些能力。
