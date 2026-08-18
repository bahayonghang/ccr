# CCR UI 视觉违和感审计报告

> 审计日期：2026-08-18。方法：Dashboard 实机截图（Catppuccin Mocha 暗色）+ 全量代码实证（`tokens.css` / `utilities.css` / `tailwind.config.ts` / 组件层）。
> 结论先行：**DESIGN.md 的"Editorial Control Room"方向本身没有问题，问题是实现层系统性漂移**。塑料感不是某一个颜色配错，而是六个互相叠加的系统性来源。

---

## 根因 1：拟物高光层 —— "塑料感"的字面来源

每一张卡片都被统一注入 iOS 6 式顶部内高光：

- `src/styles/tokens.css:390,407,491`：`--inner-glow` / `--glass-inner-glow` = `inset 0 1px 0 rgb(255 255 255 / 36-38%)`
- `ccr-ui/tailwind.config.ts:164-178`：Tailwind surface 插件把 `var(--glass-inner-glow)` 叠进 **所有** `.surface-card` / `.surface-modal`
- 同样的高光还出现在：`Card.vue:227,238`、`.glass-card`（`base.css:218-226`）、侧栏/顶栏 chrome（`MainLayout.vue:446-449,461-464`）、`PageHeaderCard.vue:125,157`

叠加效应：每张卡同时携带 **1px 描边 + 大投影 + 内高光** 三层。暗色下 `--shadow-md: 0 12px 28px rgb(0 0 0 / 30%)`（`tokens.css:431-440`），mocha 把 `--shadow-2xl` 推到 56% 黑（`tokens.css:999-1006`）。描边+宽投影的组合正是 DESIGN.md "No Ghost Card Rule" 明令禁止的，但它被写进了 surface 插件的默认配方。

白色内高光模拟的是"光滑材质表面的顶部反光"——人眼对这类反光的直觉联想就是塑料/珐琅/糖果壳。这是"塑料感"一词最直接的物理对应物。

## 根因 2：颜色系统失序 —— 系统外颜色泛滥

token 体系本身是完备的，但大量颜色绕开了它：

- **184 处字面 `rgb()` 散布在 31 个 `.vue` 文件**；73 处 `#hex` 在 5 个文件（其中 62 处是 `AppSettingsView.vue:834-877` 的色板预览，属合理）。
- `ConfigCard.vue:250-262`：当前行高亮硬编码 Tailwind **cyan/violet**（`rgb(6 182 212 / 6%)` → `rgb(139 92 246 / 4%)`），脉冲关键帧 `rgb(6 182 212 / 40%)`（319-331）。在任何主题下这都是突兀的赛博色。
- `CodexAgentEditorModal.vue:324-370`：一整套**平行的硬编码紫蓝设计系统**——`linear-gradient(180deg, rgb(22 18 31 / 97%), …)` 外壳、`rgb(112 148 198 / 34%)` 发丝线、`0 34px 96px rgb(7 5 13 / 62%)` 投影。
- `ConfirmModal.vue:209-219`：同样硬编码的蓝紫渐变外壳 `linear-gradient(180deg, rgb(23 26 43 / 98%), …)`。
- `utilities.css:608-641`：`.text-gradient-cyan/.text-gradient-pink`（`#F472B6→#EC4899`）、`.text-gradient-purple`、`.text-gradient-rainbow` 五色渐变文字。
- **120 处 `linear-gradient` 散布在 50/182 个 `.vue` 文件**。
- `tailwind.config.ts:84-92`：`premium.pink/blue` + `premium-gradient` 映射到**从未定义**的 `--color-premium-pink/blue` —— 死 token，静默解析为空。
- phantom token 模式（`var(--platform-codex-rgb, 245 158 11)` 假 token 真硬编码）在 `CodexSettingsView.vue:899-900` 仍有残留（theme-token-contracts.md 已记录该案例）。

后果：界面里实际渲染的颜色远多于设计系统的调色板，且彼此不协调。"违和感"的很大一部分来自这种**调色板熵增**。

## 根因 3：Glow 家族 —— 发光糖果语法

`tokens.css:393-404` 的 `--glow-*` 标尺被广泛用于装饰性发光：

- 按钮 hover 发光（`utilities.css:286-289, 309-312, 320-323`）
- 状态点 `0 0 8px` 辉光（`utilities.css:424-441`）
- `.text-glow-*`：`text-shadow 0 0 20px + 0 0 40px`（`utilities.css:645-668`）
- `.tag-glow::before`：`filter: blur(8px); opacity: 0.5`（`utilities.css:725-738`）
- `.animate-pulse-glow`（`animations.css:359-371`）
- `Card.vue:159-171`：已标记 `@deprecated` 但仍存活的 radial-gradient + `mixBlendMode:'overlay'` + `blur(10px)` 辉光覆盖层
- `.config-card-active`：渐变底 + 2px accent 描边 + `0 0 40px` accent 光晕（`utilities.css:89-101`）

## 根因 4：Accent 稀缺性破产 + 紫橙对撞

DESIGN.md 的 Accent Scarcity Rule 要求粘土橙 < 屏幕 10%。实际 dashboard 一屏内 accent 大面积填充出现 5+ 处：

- 行动队列首行**整行实心橙**（`DashboardNextActions.vue:211-215`：`background: var(--color-accent-primary)`）
- usage 图表 range pill 激活态实心填充（`DashboardUsageMovement.vue:404-407`）
- 图表柱子渐变（`DashboardUsageMovement.vue:544-553`）
- 平台卡 accent 条（`DashboardPlatformMatrix.vue`）
- 导航激活态：渐变底 + accent 边框 + inset ring + **左侧 accent 竖条**（`MainLayout.vue:534-552`）——左侧竖条本身是 DESIGN.md 明令禁止的 side-stripe accent border

截图所示的 Catppuccin Mocha 主题把问题放大到极限：所有 surface 为蓝紫色相（crust `#11111b` / base `#1e1e2e` / surface0 `#313244` / surface1 `#45475a`，`tokens.css:855-951`），粘土橙 `#e8835b` 落在紫底上形成互补色对撞，整块橙色行动行因此格外刺眼。mocha 下 accent 还会再映射到更高彩度的 ctp 色（`tokens.css:1240-1248`）。

## 根因 5：排版违和 —— 营销语言误入操作台

- **Display 标题进操作台**：dashboard 标题 `font-size: clamp(1.8rem, 3.8vw, 3.2rem); font-weight: 640; letter-spacing: -0.06em; line-height: 0.96`（`DashboardView.vue:448`）——这是营销页 hero 排版，直接违反 DESIGN.md 的 Product Type Rule（"Do not use fluid hero typography inside app surfaces"）。
- **大写 micro-label 泛滥**：`uppercase + letter-spacing 0.12em~0.14em` 的 eyebrow recipe 在至少 5 个文件中复制粘贴（`base.css:289-296`、`utilities.css:762-771`、`DashboardView.vue:428-438`、`DashboardNextActions.vue:151-158`、`DashboardSignalStream.vue:202-209`、`DashboardUsageMovement.vue:437-443`、`home.css:23`）。全站 256 处 `uppercase` 散布 85 个 `.vue` 文件。密集操作界面里满屏"贴纸式"大写小标签，是"不现代"观感的重要来源。
- **字重 800 的 eyebrow**（`DashboardReadinessLedger.vue:207-213`）——标签比正文还重，层级倒挂。
- **全局正文负字距**：`--tracking-normal: -0.016em` 应用于 body（`tokens.css:363` + `base.css:39`），CJK 文本一并继承，中文显挤。
- **0.14em 字距无差别加到 CJK**：uppercase 对中文无效，但 `letter-spacing` 对中文照样生效。
- **数据数字无排版处理**：`32.2% / 60.9%`、`114.9K` 用品牌 sans 特大特粗渲染，无 `tabular-nums`，无语义克制（`114.9K` 用 accent 橙、`Connected` 用 success 绿做装饰性着色）。
- 非标准字重阶梯 560/620/640/800 混用（`tokens.css:348-349`）。

## 根因 6：形状语言单一 —— 糖果 pill 泛滥

- **183 处 `rounded-full` 散布 59 个 `.vue` 文件**；`.badge` 一律 pill（`utilities.css:377-384`）；临时值 `border-radius: 22px`（`TrayOverview.vue:273`）、`9999px` 字面量（`PageHeaderCard.vue:207`、`ConfigCard.vue:276`）。
- 一切皆可 pill：filter chip、range 切换、状态徽章、设置 dock、按钮。pill + 大圆角卡 + 内高光 + 发光 = 完整的"糖果塑料"词汇表。
- 嵌套卡违例：Ready 面板内再嵌 5 张 metric 小卡（`DashboardReadinessLedger.vue`），违反 DESIGN.md "Don't create nested cards"。

## 根因 7：结构债 —— 49 页三种惯用法，原语半死

- 49 个路由视图、约 3.9 万行视图代码，三种样式惯用法并存：(a) token 驱动的 scoped BEM；(b) 纯 Tailwind 零样式块；(c) 全局功能样式表（`codex-auth-shared.css` 658 行、`checkin-shared.css`、`profiles-page.css` 221 行）。
- 单页 ad-hoc `<style>` 块最大 809 行（CheckinView）；545（AppSettings）、533（GrokView）、524（Commands）、498（GrokSettings）、489（GeminiCli）、485（ClaudeCode）、482（Pricing）。
- 共享原语采用率极低：`ui/StatCard` **0 处引用**、`ui/Badge` 2 处、`PageHeaderCard` 6 处、`ui/Sparkline` 1 处；eyebrow 页头、统计瓦片、pill 切换组在每页手工重造。
- 每页节奏（页头高度、eyebrow 样式、卡片密度、留白）各自为政 → 全站没有统一的视觉心跳，这也是"不像一个现代产品"的深层原因。

## 附：主题组合爆炸

三层体系 theme(2) × flavor(3) × accent(4) = **24 种组合**，叠加字体偏好轴。每多一个风味，所有 surface/shadow/glass/对比度都要单独过质量线（mocha 有自己整段高优先级 remap 与 56% 黑投影）。组合爆炸使"每个主题都精致"在工程上不可达——这正是用户决策**退役 catppuccin + 收敛 accent**的依据。

## 证据计数汇总（初审，2026-08-18 上午）

| 现象 | 计数 | 分布 |
|---|---|---|
| 字面 `rgb()` | 184 处 | 31 个 .vue |
| `linear-gradient` | 120 处 | 50 个 .vue |
| `rounded-full` | 183 处 | 59 个 .vue |
| `uppercase` | 256 处 | 85 个 .vue |
| `backdrop-filter` | 46 处 | 24 个 .vue |
| 硬编码 `#hex` | 73 处 | 5 个 .vue（62 处为合理色板预览） |
| 单页 style 块 >300 行 | 8+ 个视图 | 最大 809 行 |
| 死 token | `premium.pink/blue/gradient` | tailwind.config.ts:84-92 |

## 复核附录（2026-08-18 规划补完）

方法：对 `ccr-ui/src` 重跑计数。字面 `rgb()` 排除 `rgb(var(--`。`*View.vue` 排除文件名误匹配的 `TrayOverview.vue`。

| 现象 | 复核计数 | 相对初审 |
|---|---|---|
| `*View.vue` | 51 | 初审写 49，漏 2 |
| 字面 `rgb()`（排除 token 包装） | 180 | 同量级 |
| `.vue` 内 `linear-gradient` | 119；`.css` 另 20 | 初审未计 css |
| `rounded-full` | 182 | 同量级 |
| `uppercase` | 263 / 88 个 .vue | 同量级 |
| `backdrop-filter` | 46 | 一致 |
| `#hex` | 41 / 9 个文件（AppSettings 色板 22） | 初审偏旧 |
| `<style>` ≥300 行 | 20 个文件，最大 CheckinView 807 | 初审低估非视图 |
| phantom-token | MCP 族、AgentIcons、CommandList、TokenDetailTab 等多处 | 初审只点了 CodexSettingsView |

Wave 7 对账用本附录，不用初审的 49 视图 / 73 hex。根因 1–7 不因此改变。
