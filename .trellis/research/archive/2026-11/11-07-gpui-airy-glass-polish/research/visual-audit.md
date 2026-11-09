# GPUI 视觉审计证据（2026-11-07）

审计对象：`ccr-desktop/`（GPUI 原生工作台），对照 `ccr-desktop/DESIGN.md`、`ccr-ui/DESIGN.md` 与用户提供的原生截图（1280+ 逻辑宽、浅色）。Tauri 参照来自 `ccr-ui/`。行号均经 Read/Grep 核实。

## 0. 工具边界

- `slop_check.mjs ccr-desktop/src --scope app --design ccr-desktop/DESIGN.md --json`：exit 0、无输出。该脚本只解析 CSS/JSX/Tailwind 等 Web 产物，Rust GPUI 不在机械门范围内 → 机械门记 **n/a**，以下用 judged checklist 人工回答。
- 材质事实：GPUI 锁定渲染面没有应用内背景采样 API，纸面为实色降级（`ccr-desktop/DESIGN.md:106`、`src/theme.rs:178-195`）。本审计不要求假玻璃；"轻盈感'差距主要来自层级、 hairline、圆角、选中态与签名组件细节，而非 blur 本身。

## 1. 品牌图标缺失

- 窗口选项无图标：`ccr-desktop/src/lib.rs:12-29`（`window_options` 仅 title/app_id）；`lib.rs:91` `set_app_identity` 只设名字。截图左上为 OS 默认占位图标。
- 侧栏品牌区纯文字：`ccr-desktop/src/shell/mod.rs:969-992`（"CCR" + "Workbench"，无 mark）。
- Tauri 参照：`ccr-ui/src/shell/MainLayoutChrome.tsx:63-71`（`/icons/icon.svg` h-10 w-10 + brand 字体 + tagline）；资产 `ccr-ui/public/icons/{icon,logo}.{svg,png}`、`ccr-ui/src-tauri/icons/`（ico/icns 全尺寸）；`Titlebar.tsx:107` 复用 icon.svg。

## 2. Tab 选中态不可见（所有带工具行的页面）

- GPUI：`shell/mod.rs:703-725` `lens_button` = `Button::ghost().selected(...)`；工具行 `shell/mod.rs:824-833` 无容器、无分隔线；平台 context tabs 同路径 `shell/platform_navigation.rs:128`。
- 根因：Kit ghost 的 selected 背景 = `tokens.secondary_active`（cargo registry `gpui-component-0.7.3/src/button/button.rs:1291-1303`）；而 `ccr-desktop/src/theme.rs:272` 把 `secondary_active` 映射为 `elevated` #f2f7f5，与纸面背景（`theme.rs:127,185`）完全同色 → 选中 tab 在纸面上不可见。截图 "Overview / Monitoring" 两枚纯文本无差别。
- Tauri 参照：`ccr-ui/src/ui/tabs.tsx:17,33-37`（segmented：list h-9 rounded-lg bg-elevated p-1；选中 bg-surface + text-primary + shadow-sm）；`ccr-ui/src/features/platform/TabButton.tsx:15-17`（chip：accent/40 border + accent/10 bg）；`shell.css:328-353` 子导航 chip。

## 3. Overview 平台带（签名组件退化）

- `overview.rs:309-318` 带 `flex_wrap` → 窄窗换行成卡片墙；合同要求永不换行、窄宽横向滚动（`ccr-ui/DESIGN.md` Platform Quote Band；`dashboard-platform-matrix.css:9-16`）。
- 单元格 `overview.rs:321-369`：7px 圆点 + semibold 名；缺 2px 平台色 top tick（`dashboard-platform-matrix.css:27`）、缺 0.14em tracked micro label（`:238`）。
- sparkline `overview.rs:816-853`：全部 bar 平台色；合同为中性 bar + 峰值平台色（`dashboard-platform-matrix.css:184-194`）。
- 空序列基线 `overview.rs:828-829` 实线 border_b；合同为 1px dashed（`dashboard-platform-matrix.css:202-207`）。

## 4. Overview 逐日图

- gridlines 仅 [0.0, 0.5]：`overview.rs:567`；合同 25/50/75%（`dashboard-usage-movement.css:172-182`）。
- 高度 144px：`overview.rs:855`；低于 GPUI DESIGN 的 160–240 与 Tauri `clamp(10rem,26vh,16rem)`（`dashboard-usage-movement.css:145`）。
- 堆叠段无 surface 色间隙、tooltip 仅日总量：`overview.rs:862-889`；合同要求段间隙 + 逐段 tooltip + exact-value table（`ccr-ui/DESIGN.md` Bounded Usage Chart / Insights 二级编码；`InsightsAgentTrend.tsx:99-115`）。
- 零值日不画任何基线（截图 11-06/11-07 空白）；合同空序列 dashed baseline。

## 5. Next actions 嵌套卡与模板标点

- `overview.rs:637-705`：bordered 面板内套 4 个 bordered 按钮（gate 4 card-in-card）；行尾文本 "›"（`overview.rs:699`，gate 63 arrow glued）。
- Tauri 参照：`dashboard-next-actions.css:1-12` 同 home-card token，行安静 hover，无嵌套边框。

## 6. 共享 token 层偏差（影响所有页面）

- 面板圆角 8px：`overview.rs:315,446,647` 用 `INPUT_RADIUS`（`theme.rs:21`）；Tauri home card 12px（`ccr-ui/src/styles/components/home.css:28-30` + `tokens.css:458`）。
- hairline 全浓度：`theme.rs:259` border = solid #cedbd5；Tauri 为 border-default/24% alpha（`home.css:44-46`）→ 截图卡边框偏重、发闷。
- 按钮非 pill：`theme.rs:247` radius=8，Kit Button 默认 rounding=radius；GPUI DESIGN Shapes 明确"主/次按钮均 pill"；Tauri `primitives.css:601` radius-full。
- 侧栏选中：`shell/mod.rs:622-627` 14% blend 无 border tick；Tauri `shell.css:209-226` 10% tint + 24% border + primary text。
- 状态栏：`shell/mod.rs:1105-1124` 单行 muted 文本；Tauri 签名含状态点、01–04 路由 hints、tabular clock、hairline 竖分隔（`DashboardStatusBar.tsx:14,16-21`、`dashboard-status-bar.css:3-27`）。
- 纸面/外场：`theme.rs:150-157` 160deg 渐变存在；`theme.rs:181-195` inset ≤12px、实色纸面。截图外场几乎被纸面占满、渐变只在右缘可见 → 轻盈感弱；属构图与层级问题，不是 blur 问题。

## 7. 各页面共性与样例

- 共性根因：§2 的 lens_button 与 §6 的 token 偏差作用于每个带工具行/面板的页面（Commands、Usage、MCP、平台 workbench、Settings）。
- 平台 workbench 样例：`workbench.rs:386-398` 总量为单行 mono 字符串、空格分隔（无对齐列/ tabular 网格）；`workbench.rs:424-437` 逐日 bar 轨道 muted + 填充 primary 绿（绿色扩进图表；Tauri 用中性/平台身份色）；页头 actions `flex_wrap`（`workbench.rs:532-564`）。
- Overview 首屏以下大片死区：`overview.rs:204-233` 顶对齐滚动，无 Insights/事件 rail（Tauri `DashboardView.tsx:299-315` 首屏下接 Insights）。

## 8. Judged checklist（anti-slop.md §6，人工）

- 3 fail：四张等权平台 tile 读作指标卡墙（应为 ruled strip）。
- 4 fail：next-actions 嵌套边框卡。
- 9 fail：所有分区同一种分隔（16px gap + 同形 8px 白面板）。
- 63 fail："›"  glued chevron；状态行 middle dot 属 live state，保留。
- 76 fail：品牌 mark 槽位空（真实资产存在于 ccr-ui，未接入）。
- 11 pass：hover 仅在交互元素；29 pass：无 orb/glow；46a pass：真实捕获数据、诚实空态；54 pass：分组标题承载导航事实；65 pass；66 pass（版本行是事实）；F4 pass；73 pass。
- 40 pass-with-note：`theme.rs:353-387` 对比度单测覆盖 token 对；原生渲染复核仍待实施期采集（headless 不等于原生）。
- 44b/58/84 n/a：原生密集工作台，非营销页；截图为真实运行捕获。

## 9. 结论

问题集中在共享 chrome/token 层与签名组件细节，而非材质 API。优先修 token 层与 chrome（一次修复惠及所有页面），再修 overview 签名组件，最后逐页巡检。真实玻璃维持 DESIGN.md 的诚实降级，不引入假 Glass 开关。