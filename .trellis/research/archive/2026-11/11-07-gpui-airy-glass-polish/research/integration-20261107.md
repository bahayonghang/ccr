# 2026-11-07 续作与集成证据

本收据保留首个源码批次与 `5ce9c4…` 构建的完整门禁证据。后续用户恢复了核心原生验收，并发现、修复一行精确值行焦点样式；最新 `e206ad…` 构建、针对性检查和已通过的真实窗口结果见 [核心原生验收](./native-core-acceptance-20261107.md)。下文“未运行原生”的记录是前一批次当时状态，不再代表本轮核心验收仍未运行。

## 入口状态

- 分支 `dev`，续作起点 `f2a2667c`。
- 唯一未归档任务 `11-07-gpui-airy-glass-polish` 已是 `in_progress`；会话指针为空。本轮重新绑定既有任务，没有新建任务。
- 起点未跟踪文件：本任务目录、`ccr-desktop/assets/`、`ccr-desktop/build.rs`。起点尚无已跟踪代码变更，不能把研究文件或半成品资源编译脚本记作实施完成。
- 最近完成的 Claude 首页重构已有独立提交 `f0182cd7`。当前任务只调整其必要共享样式，不重复旧重构。

## 实施与证据边界

- 延续现有 airy-glass / 单一 `theme.rs`，保持实色纸面降级；不修改业务、数据源、数据库或 Tauri 产品代码。
- 原生像素、Windows 任务栏/Alt-Tab 实际显示、IME/系统无障碍不由 headless 或构建结果代替。
- 本轮不扩展窗口尺寸、DPI、跨显示器或跨平台矩阵。原 PRD 的截图矩阵仍为未运行，不能凭源码修复勾选原生验收。
- `slop_check` 机械检查不解析 Rust GPUI，适用性为 n/a；源码审查和原生视觉验收分别记录。
- 品牌需遵守 `.trellis/spec/ccr-ui/frontend/brand-asset-pipeline.md`；现有 SVG 是 PNG carrier，已有正式 ICO。不得另画单色替代品牌。

## 检查收据

| 检查 | 本轮结果 |
| --- | --- |
| 根 `just version-check` | 通过，exit 0；版本、文档/锁策略与依赖治理均通过 |
| 根 `just fmt-check` | 通过，exit 0；根/Tauri fmt、11 个 JSON 配置格式及 5 个格式器测试通过 |
| Trellis `task.py validate 11-07-gpui-airy-glass-polish` | 通过；implement 5 条、check 4 条有效上下文 |
| 品牌源文件检查 | 通过；正式 PNG 为 512×512 RGBA，ICO 副本与源逐字节相同，16/24/32/48/64/256 六个尺寸帧可解码 |
| GPUI 最终 `just desktop-ci` | 通过，exit 0；含 fmt / all-targets check / 严格 lint / 全测试 / release build |
| GPUI 最终测试 | 通过；库 241 passed / 4 ignored，Shell 10 passed / 1 ignored，UI 2 passed，shutdown 2 passed / 1 ignored；库用时 77.57s |
| Windows AccessKit patch gate | 通过；3 passed / 6 filtered，独立补丁 gate 按原规则运行 |
| 最终 Python command-contract gate | 通过；16 tests / 20.713s，exit 0 |
| `just desktop-test-ui` | 单独运行通过，exit 0；Shell 10 passed / 1 ignored，UI 2 passed |
| Release 构建与 exe 资源 | 通过；优化构建 4m32s，ID 1 ICON group 的六帧与正式 ICO 图像字节逐一相同 |
| 当前构建原生视觉验收 | 未运行 |

### 环境与失败恢复

- 沙箱内 `mbx setup --status` 指向沙箱 LocalAppData 下不存在的 shim，首次 `desktop-check` 在编译前停止。仅在检查子进程 PATH 前置 `rustup which cargo` 所确认的工具链目录；不更改持久 PATH、Cargo 配置或现有校验逻辑。随后 all-targets 类型检查通过。
- 首次严格 lint 发现新 build.rs 的 `collapsible_if`，独立 reviewer 修复后严格 `desktop-lint` 通过。
- 随后 `desktop-ci` 的 Python 配方检查在 Git Bash 创建 MSYS 对象时遇到沙箱权限错误 `0xC0000022`，Linux/macOS 两个参数子项受影响。经工具授权在沙箱外重跑原命令 `python -m unittest discover -s tests -p 'test_*.py'`：16 tests / 21.544s、exit 0。未跳过测试或降低覆盖；此结果不是其他操作系统实际运行证据。
- ICO 源/副本 SHA-256：`af47b53ab2a9290974bd6a54a09934e1220b7b124e8a12c249189e6338c0926a`。

### 独立审查与核心回归

- reviewer 修复了新 Tab decorator 覆盖禁用颜色的问题；同一 disabled 条件同时传给普通按钮与 Tab 样式，保留 busy/disabled/toggled 语义。
- 保留正值图段的最小可见填充，避免 1px 间隙边框完全遮住小值；非 Claude 成本显示恢复布局调整前的四位小数精度。
- 精确值表新增内层表格与外层页面的焦点 reveal；每日报表行 focus handle/subscription 仅保留当前报告范围。
- 长表回归采用真实 `focus_main → Tab → Enter → Tab/Shift-Tab` 路径，不假设 Kit 的鼠标点击会转移键盘焦点。按锁定 GPUI core 测试激活 headless window 并推进事件循环，在单一 1280×800 测试视口覆盖 30/90 日，不扩展原生显示矩阵。
- 同一 active-window fixture 临时停用 reveal 的负例确实失败：`30D tab overview-exact-08-01` 已聚焦，但行 y=808..846 在 page y=0..800 之外（table y=780.5..1020.5），exit 1 / 0.47s。原源码已恢复并更新 mtime，避免 Cargo 复用探针产物；临时文件已移除。恢复后的最终正例结果另列，不把该有意失败算成未解决产品失败。
- 恢复后的 Overview 聚焦回归通过：`overview_ranges_hide_stale_totals_and_preserve_exact_metrics`，1 passed / 0 failed / 244 filtered，52.27s；覆盖 30/90 日所有行双向遍历与视口交集边界。
- 首次沙箱外完整库测试：239 passed / 2 failed / 4 ignored。一个失败是新增 Usage 测试错误假设默认 Daily（原产品默认 Sources）；已保留产品逻辑，改为显式 `Sources → Daily → Models` 点击、等待和选中断言。另一个是已有 1 MiB 栈子进程保护捕获 Observer 确认流程栈溢出；定位到本任务新增 Insights selector builder 加大单一 render 栈帧，不能通过提高栈或跳过子项掩盖。修复后结果另列。
- 栈修复红绿：修复前同一 Observer 子进程在 `RUST_MIN_STACK=1048576` 下以 `-1073741571` 栈溢出退出；仅将新增 product tabs / usage sections 构造提为同模块私有 `Div` helper 后，同一子项 exit 0、1 passed / 0.76s。未改栈限制、公开 API、布局或业务条件。
- 原 27 子项 1 MiB 栈与 poisoned-lock 保护完整重跑通过：wrapper 1 passed / 28.74s；修正后的真实 Usage 切换与精确成本回归通过：1 passed / 1.52s。随后冻结源码，重跑完整 `desktop-ci`。
- 最终完整库运行已通过：241 passed / 0 failed / 4 ignored，77.57s；其后的 Shell/UI/shutdown 及 doc tests 均通过，详见上表。所有 Rust 测试保留默认并行与 `--skip export_bindings`；既有 helper ignored 项不伪装成执行通过。
- 根 `just ci` 未运行；本轮产品代码范围是独立 GPUI workspace，未改领域 Rust、Tauri/React 产品代码或数据库。按既有任务计划执行桌面门禁与根版本/格式检查。

### 最终产物

- `ccr-desktop/target/release/ccr-gpui.exe` SHA-256：`5ce9c4c198f2fe8dfa4a7cc12f7ed2d932d4c678706de90500b966fec4c40499`。
- 以 Windows data-file 方式读取资源，未执行应用：ID 1 `RT_GROUP_ICON` 的 16/24/32/48/64/256 六个 `RT_ICON` 图像原始字节集合与正式源 ICO 相同。
- 原始日志：`outputs/desktop/airy-glass-polish-ci-20261107.log`；资源验证 JSON：`outputs/desktop/airy-glass-polish-icon-resource-20261107.json`。这些是本机生成证据，不是已提交或已发布的包。

## 收尾状态

源码实施、独立审查、自动化门禁与 Windows release 资源核验已完成。原生截图/构图、标题栏/任务栏/Alt-Tab 实际显示及系统交互验收未运行，保留任务 `in_progress`，不归档。当前工作区未提交，未推送；下一轮先核对工作树与本收据，不重复实施已完成源码。
