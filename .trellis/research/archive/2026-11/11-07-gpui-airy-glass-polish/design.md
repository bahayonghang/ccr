# 技术设计：GPUI 视觉细节优化

合同来源：`ccr-desktop/DESIGN.md`（唯一设计根）、`ccr-ui/DESIGN.md`（共享语言参照）、`research/visual-audit.md`（证据）。规则：先改 DESIGN.md，再改唯一 Rust token 层 `src/theme.rs`，最后改组件；不建第二主题系统。

## 1. Token 层（theme.rs）

- 新增角色：`PANEL_RADIUS = 12`（面板/带/读数井），`INPUT_RADIUS = 8` 保留给密集控件；`hairline` = subtle 色 alpha ≈ 0.24–0.35（light/dark 分别取值，实测对比），`border` 保留给需要语义的分隔；控件边界继续 `control`。
- `secondary_active` 不再等于纸面同色：tab 选中改由专用 tab token 表达（见 §2），避免 ghost selected 与纸面同色根因复发。
- 按钮 rounding：主/次按钮显式 `PILL_RADIUS`（Kit `ButtonRounded::Size(px(9999.))` 或等价），不改 Kit 全局 radius 以免影响输入/列表。
- 对比度单测（theme.rs tests）扩展 hairline/tab 配对：选中文字 ≥4.5:1 对选中面，tab 容器边界 ≥3:1 对纸面。

## 2. Tab / 工具行（shell）

- 新建原生 segmented lens bar（shell 内私有组件，替换 `lens_button` 的 ghost 用法，覆盖 `shell/mod.rs:824-833` 与 `platform_navigation.rs:128` 两处调用点）：
  - 容器：bg elevated、rounded 8、padding 2px、底部 hairline 与内容分隔；
  - 项：h 28–32、rounded 4–6；选中 bg surface（白）+ primary 文字 + semibold；未选中 secondary 文字；hover 一个中性色阶；focus_visible 2px ring；
  - 语义保留 `aria_toggled`/accessibility_label；方向键行为若现状无则不新增（与现状一致，Enter 切换）。
- 不升级 Kit 版本、不 patch Kit；全部在应用层组合 base Button/div 实现。
- 实施收口：共享 `semantics::tab(button, selected, disabled, cx)` 在普通按钮装饰后应用，并使用相同禁用条件保留 disabled 颜色与既有 busy/无障碍语义。

## 3. 品牌资产

- 侧栏 mark：复用 `ccr-ui/public/icons/icon.svg`。研究项：GPUI 自定义 svg 资产加载路径（`gpui_kit::assets::Assets` 之外组合 AssetSource，或 `svg` element + 自建 source）；若锁定渲染面不支持外部 svg source，退路是把 icon 以 Kit Icon 扩展或内联 path 数据接入，仍用真实资产形状，不新画。
- Windows 窗口图标：研究项 `WindowOptions`/TitlebarOptions 是否暴露 icon；若否，用 build script（winresource 等价）把 `ccr-ui/src-tauri/icons/icon.ico` 嵌入 exe 资源。复制资产到 `ccr-desktop/assets/` 并记录来源；构建开销记录进 dependencies.md。
- 品牌行布局：mark 24–28px + 名称/状态两行，对齐 Tauri MainLayoutChrome 层级但保持原生密度（NAV_BRAND_HEIGHT 可微调）。
- 实施收口：正式 `icon.svg` 是 embedded-PNG carrier，GPUI SVG mask 不适合保留品牌原色；直接在构造时嵌入现有 `ccr-ui/public/icons/icon.png`。Windows 复用已存在的正式 ICO，不另行生成或重画。

## 4. Overview 签名组件

- 平台带：`flex_wrap` → `overflow_x_scroll` + 固定 min 格宽；格顶 2px tick（平台色）；micro label 12px muted（中文不加宽字距，Latin mono 可 0.14em）；sparkline bar 改 neutral（border/subtle 阶），峰值格平台色；空序列 dashed baseline：GPUI 无 dashed border 时用等距 1px 段 div 行模拟（研究项，记录实现）。
- 逐日图：gridlines [0.25,0.5,0.75]；`CHART_HEIGHT` 改 160–200 内固定或 clamp 等价；段间 1–2px surface 间隙；逐段 tooltip（segment 级 detail）；exact-value table 视图切换（复用 insights rows 的表格模式或 overview 内简表：日期 × 4 来源 + 总计，mono tabular）；零值日 dashed baseline 同带。
- 总量：固定列网格（label 左 muted 12、值 mono 右/左对齐 tabular），替换 flex_wrap 空格行；workbench 总量同改。
- Next actions：面板保留，行去 border（透明底 + hover 一个色阶 + focus ring），icon 16px + 动词标签，删除 "›"。
- 首屏下死区：决策接 Insights 摘要 rail（会话/请求/活跃日读数行 + 导入入口），复用 R4 token；若数据未配置显示诚实空态。该决策在实施前以原生截图复核一次。
- 实施收口：摘要使用真实四来源活跃日覆盖与未知归档原因，避免重复总数；原生构图复核仍未运行。精确值表保留当前报告行的 focus/subscription，并在聚焦时 reveal 内层表格及外层页面；30/90 日双向 Tab 回归断言行位于两者视口交集内，不新增全局按键处理。

## 5. 状态栏与侧栏

- 状态栏：左 = 环境 + 状态点（真实状态色）+ 用量来源文案；中 = 01–04 路由 hints（点击跳转，hover 提亮）；右 = tabular clock（30s tick，复用现有 subscription 模式，不每帧轮询）。hairline 竖分隔；高度 28 保留。
- 侧栏选中：10–14% tint + 1px 24% primary border（或 2px left tick 二选一，截图复核），字重 semibold 保留。

## 6. 逐页巡检

共享修复落地后按页核对：workbench（daily bar 中性轨道 + 平台/中性填充、总量列对齐）、insights/usage、settings、commands、mcp、checkin、sync。每页只改层级/对齐/选中态/空态呈现，不改业务逻辑。

## 7.  tradeoffs 与边界

- 实色纸面降级保留；轻盈感靠 hairline alpha、12px 面板、外场可见边距与层级，不靠 blur。
- dashed/横滚/资产加载若遇 GPUI 能力缺口，按 DESIGN.md 流程记录研究结论与降级方案，不假称通过。
- 兼容：测试 id（overview-*、nav-*、tool-*）与 aria 保留；headless 测试更新断言以匹配新结构，不弱化。
- 回滚：按 slice 独立 commit；token 层改动单独 commit，便于 revert。
