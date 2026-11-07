# GPUI 轻盈玻璃工作台视觉细节优化

## 目标

用户打开 GPUI 工作台时，能一眼认出产品（品牌 mark）、认出当前页与当前 tab（选中态可见）、读懂首页签名组件（平台带、有界图、状态栏），并在每个页面感受到与 Tauri 版同一套 airy-glass 语言：克制的 hairline、12px 面板、pill 按钮、对齐的等宽数据。修复共享 chrome/token 层一次惠及所有页面，再逐页巡检。

证据基线：`research/visual-audit.md`（file:line + Tauri 参照 + judged checklist）。用户截图基线：2026-11-07 浅色 Overview 首屏（品牌图标缺失、tab 无选中态、平台卡墙、图表粗糙、下半屏死区）。

## 需求

### R1 品牌身份

- 侧栏品牌区接入真实 CCR mark（复用 `ccr-ui/public/icons/icon.svg` 资产，不新画替代品），与产品名/工作台状态组成品牌行。
- Windows 标题栏/任务栏/Alt-Tab 使用 CCR 图标（复用 `ccr-ui/src-tauri/icons/` 的 ico 或等价嵌入路径）；不声称 macOS/Linux 图标已验证。

### R2 Tab 与共享 chrome

- 所有 lens 工具行（Overview/Monitoring、平台 context tabs、Commands/Usage/MCP 工具行）选中态一眼可辨：容器 + 选中面 + 主色文字，对齐 Tauri segmented/chip 语言；hover、键盘焦点、aria 语义保留。
- 工具行与内容之间有明确分隔（hairline 或容器边界），不再悬浮纯文本。
- 侧栏当前路由：tint + 细 border 表达，不用粗侧边条；分组标题、设置 dock、版本行保留。
- 底部状态栏升级为签名组成：环境/用量状态点 + 编号路由 hints + tabular clock，hairline 分隔；不引入每帧轮询（clock 低频 tick）。

### R3 Overview 签名组件

- 平台带：永不换行，窄宽横向滚动；每格 2px 平台色 top tick + micro label；sparkline 中性 bar、峰值平台色；空序列 dashed baseline；无记录显示原因不铺零。
- 逐日图：25/50/75% gridlines；堆叠段 surface 色间隙；高度落在 160–240px；逐段 tooltip + exact-value table 可达；零值日 dashed baseline；图例与身份色一致。
- 总量读数对齐列网格（label 左 / 数值 mono 右对齐 tabular），不挤成空格字符串。
- Next actions：去嵌套边框（面板内安静行，hover 一个色阶），删除 glued "›" 字符，用真实 icon 或纯动词标签。
- 首屏以下死区：给出构图决策（接 Insights 摘要 rail 或事件流，参照 Tauri 首屏下 composition），不留下半屏空白纸面。

### R4 共享 token 层

- 面板圆角 12px（密集控件保持 8px/4px）；hairline 改 alpha 细线（对齐 Tauri 24% 浓度），控件边界仍用 border-control。
- 主/次按钮 pill（GPUI DESIGN Shapes 合同）；focus 2px 环与间隔保留。
- 所有新增/修改角色进 `src/theme.rs` 唯一 token 层，先改 DESIGN.md 再改代码；对比度单测同步扩展。

### R5 逐页巡检

- 平台 workbench（Claude/Codex/Grok/OpenCode/Antigravity）、Insights/Usage、Settings、Commands、MCP、Check-ins、Sync：应用 R2/R4 共享修复后逐页核对层级、对齐、空态、选中态；workbench 逐日 bar 改中性/平台身份色，总量改对齐列。
- 每页采集原生截图（light/dark × zh/en，1280×800 与 960×640），记录观察与未运行项。

## 约束

- 不引入假玻璃：纸面维持实色降级（DESIGN.md:106），不加 Glass 开关、不嵌套玻璃、不装饰动画。
- 不改 Tauri 产品代码、数据库、采集器；资产复用需记录来源与许可（仓库自有资产）。
- 保留键盘/F6/Tab/IME、焦点恢复、虚拟化与 stable ID；现有测试 id 与 aria 不破坏，更新而非弱化断言。
- DPI/跨显示器矩阵维持 user-deferred（2026-10-18 决定），不记通过。

## 原始验收条件（历史快照）

2026-11-07 后续用户明确将本轮原生验收限定为图标、Tab 选中态、Overview 精确值表，不做窗口尺寸/DPI 矩阵。该核心验收及发现的焦点修复已通过，见 `research/native-core-acceptance-20261107.md`；以下混合了历史全量矩阵的复合 AC 不凭本轮窄验收全部打勾，也不强制补做用户已排除的矩阵。

- [ ] 品牌：侧栏 mark 与 Windows 窗口图标在浅色/深色截图可见；资产来源记录在 research。
- [ ] Tab：所有工具行选中态在两主题下一眼可辨；焦点环与 aria_toggled 保留；原生截图佐证。
- [ ] 平台带：960–1280 宽不换行、窄宽横滚；tick/micro label/中性 sparkline/峰值色/dashed 空基线齐备。
- [ ] 逐日图：gridlines 25/50/75、段间隙、高度 160–240、逐段 tooltip、exact table、零值日基线；现有 overview 交互测试绿。
- [ ] Next actions 无嵌套边框卡、无 glued chevron；状态栏含状态点/路由 hints/clock。
- [ ] token：面板 12px、hairline alpha、pill 按钮、侧栏 tint+border；对比度单测覆盖新配对。
- [ ] 逐页截图清单完成（overview、claude workbench、usage、settings 至少四页 × 两主题 × 两语言 × 两尺寸），未运行项明确列出。
- [x] `just desktop-fmt-check`、`desktop-lint`、`desktop-test`、`desktop-test-ui` 全绿（最终 desktop-ci 与独立 desktop-test-ui 均 exit 0，收据见 research/integration-20261107.md）。

## 最终归档验收口径

2026-11-07 用户在核心原生复验通过后明确要求“用中文分批提交本轮 GPUI 改动，更新验收记录并归档当前 Trellis 任务，不要推送，也不用补做尺寸和 DPI 矩阵”。按这一最终授权收口；上面的历史复合 AC 不被补记为全矩阵通过。

- [x] R1/R2/R4：品牌资源、共享 token/Tab/禁用态、侧栏和状态栏已实施，相关控件与对比度检查通过。
- [x] R3：Overview 平台带、图表、对齐读数、精确值表和覆盖范围已实施；30/90 日双向键盘与可见范围回归通过。
- [x] 核心原生：侧栏/标题栏/任务栏图标、两主题选中态与精确值行焦点、自动滚动、返回和收起已通过真实窗口及独立图片复核。
- [x] R5：逐页源码巡检与本轮核心原生验收证据已记录；历史四页/语言/尺寸矩阵按最终范围排除，保持未运行，不作为追加归档门禁。
- [x] 完整源码批次 `desktop-ci` 通过；原生发现的一行焦点修复另有 focused 回归、fmt/lint/UI、release 构建与新构建原生通过证据。

Alt-Tab 单独截图、系统读屏/IME、其他 OS 与尺寸/DPI/跨显示器矩阵没有新增验收，不声称通过。源码提交及归档收据见 `research/native-core-acceptance-20261107.md`。

## 范围外

真实背景模糊、macOS/Linux 原生验收、DPI 矩阵、托盘、Tauri 代码、业务逻辑与数据层、自定义字体。
