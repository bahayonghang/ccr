# 2026-11-07 核心原生验收

## 用户范围与环境

用户本轮明确要求：启动最新 GPUI，重点验证图标、Tab 选中态、Overview 精确值表，不做窗口尺寸与 DPI 矩阵。只使用启动后现有窗口；没有改变窗口大小、缩放或显示器。切换 light/dark 用于核对核心选中/焦点对比，结束时恢复初始 Light / English 偏好。

原生操作：`computer-use` 的 `@oai/sky`，真实 Windows 窗口截图与 UIA；没有浏览器 mock、直接 PowerShell UI Automation、用户认证或业务写入。截图保存在被忽略的本机 `outputs/desktop/native-core-20261107/`，不自动提交含本机数据的画面。

## 首次构建与观察

- 直接启动 `ccr-desktop/target/release/ccr-gpui.exe`；启动前 SHA-256 为 `5ce9c4c198f2fe8dfa4a7cc12f7ed2d932d4c678706de90500b966fec4c40499`，与上一轮 release 收据相同。
- 侧栏彩色 CCR mark、标题栏图标实际可见；light/dark 下保持原色。未用 EXE 资源读取代替真实窗口观察。新构建复验时还捕捉到任务栏 CCR 图标；Alt-Tab 未单独采集。
- 全局 Overview / Monitoring 选中面、边界与文字在 light/dark 下随切换变化；Claude 平台 Workbench → Profiles 选中也能辨认。没有进入认证、编辑、删除或应用配置流程。
- 90D 在本机读取真实报告；精确值表通过 `90D → Tab → Exact values → Enter` 展开，日期与四来源及图中合计可读。滚动至表尾可见真实非零行（如 11-01 的 447 + 2 = 449），图中合计与其他来源总数的范围提示保留。
- 从表头 Tab 前进，Shift+Tab 返回，UIA 可见对应逐日行焦点；手动滚至表尾后再 Tab，会把原离屏焦点的下一行重新滚回表格可视区。收起后表格节点消失。

## 发现与最小修复

首次构建 **没有通过可见行焦点验收**：UIA 焦点正确、自动滚动正确，但重复纯 Tab（中间无指针操作）后仍看不到稳定的 2px 行边框。证据：`03-exact-90d-first-focus-light.png`、`04-exact-pure-tab-light.png`、`06-exact-keyboard-reveal-light.png` 与各自 UIA 文本。

独立审查确认锁定 GPUI 的 `.focus_visible` 同时要求逻辑焦点和最后输入为 Keyboard；Windows 转发任何 `WM_MOUSEMOVE` 可清除键盘模态。未确定具体事件发送者，不归因于 Sky 或用户移动指针。仅将精确值行 `.focus_visible(...)` 改为 `.focus(...)`，让逻辑行焦点持续可见；保留 ring token、透明 2px 预留边框、尺寸、Tab 顺序、滚动、aria 与所有数据逻辑。

修复后的自动验证：

- Overview 30/90 日真实控件几何回归：1 passed / 0 failed / 244 filtered，82.04s。
- `desktop-fmt-check`、`desktop-lint`：exit 0。
- `desktop-test-ui`：exit 0，Shell 10 passed / 1 ignored，UI 2 passed。
- 独立源码审查：接受这一行修改；像素结果须以重建后的原生窗口为准。
- 当前仅做受影响回归；没有为一行焦点样式重复不相关的完整仓库 CI 或尺寸/DPI 矩阵。上一轮完整 `desktop-ci` 的收据与旧产物哈希保留在 `integration-20261107.md`。

## 修复构建与复验

- `desktop-build` exit 0，优化构建用时 4m22s。构建日志：`outputs/desktop/native-core-focus-build-20261107.log`。
- 新 exe SHA-256：`e206adfad79b564366a831a2cd262636d9a7632534f6d44763a0f13a5b64aa79`，与修复前不同；重新启动该绝对路径返回新窗口，避免旧进程冒充最新构建。
- 新构建再次通过 `90D → Tab → Exact values → Enter → Tab`：08-10 行有清晰绿色边框；下一次 Tab 边框移动到 08-11，与 UIA 焦点一致。
- `Shift+Tab → 08-10 → Shift+Tab → Exact values → Enter` 后表格消失，按钮焦点环可见。表格内部和外层页面的自动 reveal 仍正常。
- 深色用真实 `F6 → 主区 Refresh → Tab 经时段控件到 Exact values → Enter → Tab` 进入首行，薄荷绿色边框清晰可见。没有直接注入行焦点。
- 独立图片复核确认 12/13/14 的焦点边框与关闭路径，11 的任务栏图标可见。主会话另实看深色 15。
- 结束时恢复 Light / English，保留新构建窗口在 Overview；没有保留偏好更改、业务草稿或启动执行命令。

## 证据索引

- `01-overview-light.png`：初始品牌/标题栏与 Overview selected。
- `02-monitoring-selected-light.png`：浅色 Monitoring selected。
- `03-exact-90d-first-focus-light.png`、`04-exact-pure-tab-light.png`：修复前焦点缺少明显边框。
- `05-exact-90d-tail-light.png`：表尾精确数据。
- `06-exact-keyboard-reveal-light.png`：离屏焦点通过 Tab 自动回显。
- `07-overview-dark.png`、`08-monitoring-selected-dark.png`：深色品牌和全局 Tab。
- `09-platform-profiles-selected-dark.png`：平台上下文 Tab。
- `10-fixed-overview-light.png`：修复后的新构建启动，品牌/选中态仍可见。
- `11-fixed-taskbar-icon.png`：新构建实际任务栏图标。
- `12-fixed-exact-first-focus-light.png`、`13-fixed-exact-next-focus-light.png`：浅色行焦点边框及移动。
- `14-fixed-exact-keyboard-closed-light.png`：键盘返回入口并收起。
- `15-fixed-exact-focus-dark.png`：深色可见行焦点。
- `16-final-overview-light.png`：恢复原主题、保留窗口供用户查看。

## 状态

本轮指定的核心原生验收 **通过**：侧栏/标题栏/任务栏图标、light/dark 全局选中态与平台上下文选中态、精确值展开/实际数据/滚动/可见焦点/反向返回/收起。发现的行焦点缺失已最小修复并在新构建实看通过。

窗口尺寸/DPI/跨显示器矩阵按用户要求不做，不作为追加验证。未跑四页×两语言×两尺寸的历史截图清单，不将本轮窄范围结果泛化为该矩阵通过；Alt-Tab、系统读屏/IME和其他 OS 无新验收。验收结束时尚未提交或归档，后续收尾如下。

## 中文分批提交与归档授权

2026-11-07 用户在上述核心验收通过后明确要求分批提交、更新记录并归档，且不推送、不补做尺寸/DPI 矩阵。归档采用已完成的源码/自动化与本轮核心原生验收口径；历史未运行项保留原状态。

| 本地代码提交 | 内容 |
| --- | --- |
| `43d1b9e79f750f57d6f5193ebbc76276c214581e` | 品牌资源、共享 token/导航/状态栏、工具行及对应回归 |
| `dacaca40676df0b96797bb665bcc47284b3eb57b` | Overview 图表/精确值表/可见焦点、工作台呈现与契约 |

提交仅记录已验证的源码，没有再次编译或替换正在运行的 `e206ad…` 产物。截图、EXE 和运行日志继续保留在被忽略的本机 outputs，不进入本次提交。归档完成状态以 task.json 的 completed 与 archive 目录为准。

归档结果：已通过 `task.py archive --no-commit` 移至 `.trellis/research/archive/2026-11/11-07-gpui-airy-glass-polish`，task.json 为 completed，日期 2026-11-07。活动任务指针已清除；JSONL 自引用同步到归档路径并重新验证。使用 `--no-commit` 仅为按用户要求自行创建中文归档提交，不跳过状态校验。未推送。
