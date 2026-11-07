# 2026-11-07 全范围独立源码审查

后续原生续查：首次原生发现精确值行可见焦点不稳定，已仅将该行 `.focus_visible` 改为 `.focus`；独立源码复核和新 release 截图 12/13/14 的图片复核均通过，任务栏截图 11 也可见 CCR 图标。主会话另核验深色截图 15。最新范围和 `e206ad…` 产物见 `native-core-acceptance-20261107.md`；下面保留原源码批次的历史验收边界，不将本次核心结果泛化为全矩阵通过。

最终收尾：用户已在核心验收通过后明确授权本地分批提交和归档、不推送、不补尺寸/DPI 矩阵。独立收尾核对未发现源码分组依赖或新增测试需求；按最终范围可归档，后文历史 not_run/不归档表述只描述当时状态。保留未运行项，不将其改写为通过。

审查角色：`trellis-check`，未参与最初实施。范围包括当前任务所有 GPUI 源码、构建资源、测试、设计和 spec 变更；没有审查或更改 Tauri 产品代码与领域数据层。

## 已修复

1. `ccr-desktop/build.rs`：严格 Clippy 发现嵌套条件可折叠。改为 Windows 条件与资源编译错误的 let-chain，保留资源编译失败即阻止构建的语义。
2. `src/shell/semantics.rs` 及六处新 tab 调用：tab 装饰器位于 button 装饰器之后，其显式选中样式会覆盖禁用样式。锁定 Kit 的 `button.rs` 在 disabled 状态中再次应用 instance style，因此忙碌/typed-only selector 曾会看起来可用。新增明确的 disabled 参数，使用与原按钮相同的条件保留 muted 文字/背景和 control 边界；不更改 disabled/busy 的原有可操作性与无障碍元数据。既有 Jobs 控件回归新增 busy 时点击其他 selector 不能切换且 checked 状态仍正确的断言。
3. `src/features/overview.rs`：新增 1px 段间 surface border 会完全覆盖原有 1px 最小高度的正值段。最小高度改为 2px，留至少 1px 身份色，精确值表保持原值。
4. `research/native-capabilities.md`：保留原 SVG API 探针，但将 §1/§6 明确标为非生产品牌方案，删除单色化品牌的建议，指向真实 PNG 方案；ICO 存在的事实与来源已校正。
5. `src/features/workbench.rs`：总量列重排曾顺带把费用格式从固定四位改为大于等于 USD 100 时两位。恢复原有四位费用精度，只保留本任务要求的对齐列变化。

## 全范围检查

- 品牌：侧栏在构造时嵌入仓库已有彩色 PNG，Windows 资源使用正式 ICO 的相同副本；没有手绘/单色替代品、运行时下载或本地绝对资产路径。主会话验证 ICO SHA-256 和各尺寸帧。
- 主题：只有 `theme.rs` 一个 token 所有者。装饰线 alpha 与输入/control 边界分开；panel/input/sheet 半径各自保留；深浅主题新增对比度配对有测试。
- 选中态：顶部 lens、平台上下文、Commands、Usage、MCP 均保留原 stable ID、selected/toggled、原焦点行为；新 tab helper 在普通 button 装饰后应用，禁用状态独立处理。
- 状态栏：Clock 是独立 child entity，仅每 30 秒通知自身，任务随 entity 生命周期取消；不轮询业务数据或每帧刷新。环境/已配置文案保持事实，来源点不声称查询或远端登录成功；编号路由走现有 `Shell::go`，F6 仍按状态区子树识别。
- Overview：平台带不再 wrap，溢出由局部横滚拥有；图表高度 180px，25/50/75 网格、身份色、零日虚线、逐段 tooltip 和精确值表来自捕获报告。总数与四来源图合计明确区分；未知会话/成本不填零。下一步安静行保留原动作与键盘 focus。
- 工作台和逐页：非 Claude 总量改为独立 label/value 对齐，日条使用平台身份色；Claude 保留上一任务费用曲线和报告流程；未改读取范围、凭据/确认、数据库、采集、虚拟化或业务请求。

## 精确值表键盘溢出回归

精确值表的 30/90 日溢出键盘路径：原新增回归只有一行并只断言 focused；原表没有 scroll handle / reveal，不能据此证明视口外行可见。Overview 实施代理已补充与报告行数一起增长/缩减的焦点订阅、内外两层 ScrollHandle reveal，并增加 30/90 日完整正反 Tab 的 focused bounds 与 table/page 视口交集断言。独立复核接受该局部方案，无数据 I/O 或无界任务；最终恢复源码的绿色回归已通过，此项关闭。

最终红灯证据使用同一已激活的 1280×800 headless fixture：临时旁路 reveal 后，30 日键盘回归在首行 `overview-exact-08-01` 失败，exit 1 / 0.47s：已聚焦行 y=808..846 超出页面 y=0..800（表格 y=780.5..1020.5）。这证明原先仅有焦点并不足以使行可见。回归从产品 `focus_main`、Tab、Enter 进入表格，未假设鼠标点击会转移键盘焦点（Kit 明确避免 mouse-down focus）。测试显式激活 headless window，以满足核心 focus 回调的运行条件；不是更改产品条件。旁路仅用于反例，已恢复并 touch 实际修复源码、删除临时探针，重编译后的最终绿色结果为 exit 0 / 52.27s，1 passed、0 failed、244 filtered；完整走过 30/90 日正反键盘路径。

## 验证收据

- 本审查修改后指定 Rust 文件 rustfmt 和 `git diff --check` 通过。
- 主会话反馈严格 `desktop-lint` 已 exit 0；等价涵盖修复后的 all-targets 类型编译。
- 最终恢复源码的精确值表 focused/bounds 回归 exit 0，1 passed / 244 filtered / 52.27s（Overview 代理运行，主会话协调独占 Cargo）；反例与绿色使用相同已激活 fixture。
- 完整 gate 发现的两项问题已窄门收口：相同 Observer 子项的 1 MiB 栈回归 exit 0 / 0.76s；既有 27 子项 1 MiB + 毒化锁目录 wrapper exit 0 / 28.74s；真实 Sources → Daily → Models 与费用精度回归 exit 0 / 1.52s。均为 1 passed / 244 filtered，原断言和栈约束保留。
- 最终全范围 gate 由主会话统一执行并反馈：fmt、all-targets check、严格 lint 均通过；lib 241 passed / 0 failed / 4 ignored（77.57s）；native_shell 10 passed / 1 ignored；native_ui 2 passed；shutdown 2 passed / 1 ignored；Python 16 passed（20.713s）；AccessKit 3 passed / 6 filtered。忽略项保留，未扩大测试线程栈或缩减已有断言。
- 主会话最终核验：完整 `desktop-ci` exit 0，release build 用时 4m32s；另行 `desktop-test-ui` exit 0。Windows exe ID 1 图标组的六帧与正式 ICO 图像原始字节相同；完整命令和产物哈希见 `integration-20261107.md`。
- Python 配方检查首次遇到 Git Bash 的沙箱 MSYS 对象权限错误；主会话在获准的沙箱外窄门重跑，16 tests / 21.544s、exit 0，确认是验证环境问题。没有修改测试或降低覆盖。

## 设计审查与验收边界

Design-atlas 审查为独立源码审查。早期局部 `ship-for-source` 曾被完整测试的两项失败撤回（当时 239 passed、2 failed、4 ignored）；对应问题已修复，且最终全范围源码与测试检查现已通过。源码审查判定为 **source/test accepted**；主会话另已核验 **release integration passed**。没有当前构建原生截图，未执行无历史的原生首屏视觉审查，原生视觉状态仍为 **not_run**；不能作像素级 ship 或完整任务完成判定。

- 旧 judged checklist 中的平台卡墙、嵌套动作边框、glued chevron、品牌空槽已有对应源码修复。
- 配色/半径/选中角色遵守现有 `DESIGN.md`；机械 slop checker 不解析 Rust，结果适用性为 n/a，不能用 exit 0 代替视觉判断。
- 当前 Windows 标题栏/任务栏/Alt-Tab 实际显示、light/dark 与中英文原生像素、系统无障碍/IME 未运行。
- 窗口/DPI/跨显示器/跨平台矩阵保持 user-deferred / not_run，任务原生视觉 AC 保持开放，不归档。

## 完整 gate 新发现（续查）

- `environment_views_fit_native_windows_stack_with_isolated_locks` 并非尺寸矩阵；它把 27 个现有核心流程分别放进 1 MiB 栈、毒化默认 lock 路径的子进程。失败子项是 `features::insights::observer::tests::observer_setup_confirmation_cancels_then_saves_without_a_usage_database`，真实栈溢出。审查代理直接运行已确认的库测试 exe、仅对子进程设置 `RUST_MIN_STACK=1048576` 再现：exit -1073741571，stderr 明确 stack overflow；没有把 Cargo/rustc 线程栈缩小。
- 最小修复已验证：只把本任务新增/修改的产品 tab 与 Usage section 的构造移到同模块私有 `Div` helper，减少巨大 `InsightsView::render` 的 debug builder 临时对象栈帧；控件、条件、顺序、样式、ID、语义和处理器保持原样。相同 1 MiB Observer 回归由 overflow 变为 exit 0 / 0.76s，原 27 子项 wrapper 也 exit 0 / 28.74s。不提高栈限制、不弱化原回归。
- 独立的 Usage 断言问题由原实施代理修正：构造器实际默认 `Section::Sources`，新测试错误地假定 Daily。测试现通过真实 Sources → Daily → Models 点击、等待和 checked/state 断言覆盖切换，保留精确费用与数据库不变检查；产品默认值没有变更。修正后该精确测试 exit 0 / 1.52s。
