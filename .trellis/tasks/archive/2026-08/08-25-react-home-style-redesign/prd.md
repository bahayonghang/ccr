# 重新设计 React 首页与样式系统

## Goal

在不改变 CCR Desktop 现有业务能力、路由与 Tauri 数据契约的前提下，重新设计 React 首页和应用样式配置，使其成为面向高级 AI CLI 用户的现代、清晰、可信且高信息密度的操作工作台；同时建立可复用、可配置、可验证的视觉系统，解决当前页面单调、层级扁平、等宽字体滥用和整体观感陈旧的问题。

## Background and Confirmed Facts

- 当前前端已完成 Vue → React 迁移；架构事实以 `ccr-ui/src/main.tsx`、`ccr-ui/src/shell/router.tsx` 和 `.tsx` 组件树为准。
- 当前技术栈包括 React 19、React Router、TanStack React Query、Zustand、Radix UI、Motion、React i18next 与 Tauri 2。
- 首页组合入口为 `ccr-ui/src/features/usage/dashboard/DashboardView.tsx`，现有区块为 readiness、next actions、usage movement、signal stream 与 platform matrix。
- 样式由 `ccr-ui/src/styles/` 全局 tokens/theme、`ccr-ui/src/ui/` primitives 和 feature-local CSS 共同承担。
- 现有首页截图显示：大面积近似灰黑表面、卡片对比不足、排版层级偏弱、等宽字体覆盖过广、数据与动作缺少明确视觉焦点。
- 仓库要求保持 calm、precise、editorial workbench 方向，偏暖中性色、charcoal 文本、低饱和强调色；不得回到重玻璃拟态、紫色 SaaS、NEKO/二次元或国风视觉。
- `ccr-ui/src-tauri/Cargo.toml` 存在用户未提交改动，本任务不得覆盖或混入该改动。
- `code_map.md` 的 Vue 描述、`ccr-ui/package.json` 的包名 `ccr-ui-frontend-vue`、`ccr-ui/CLAUDE.md` 的 Vue 技术栈表格均属迁移残留。
- 设计输入已到位：Claude Design 项目 `0a3d3dfa-8ad5-4bdf-861d-305f1e2c6389` 的 `CCR UI 首页重设计.dc.html`，已存档为 `research/claude-design-source.html`（143.5 KB）。该产物取代原 OpenDesign 生成路径，OpenDesign 线程不再恢复。
- 设计稿含四个部分：`1a` 台账优先、`1b` 运行时卡阵列（两者为互斥的首页 IA 方案）、`1c` 令牌提案、外观设置页亮色重排，以及 `0a` 当前界面还原基线。
- 数据契约核对结果：`HomeOverviewSeriesItem`（`ccr-ui/src/types/generated/usage/HomeOverviewSeriesItem.ts`）按天携带 `claude`/`codex`/`antigravity`/`opencode` 四平台的 `sessions`/`requests`/`tokens`，因此平台卡 sparkline 与堆叠日柱图可纯前端派生，无需改动 IPC 契约。
- `HomeUsageOverviewResponse` 不含 cost 字段；成本只能来自既有命令 `get_usage_summary_v2`（`UsageSummaryDto.total_cost_usd`）。
- 现有令牌层已具备设计稿所需的语义结构：`tokens.css` 提供 `neutral`/`clay` 两个 flavor 与明暗两套主题，clay 暗色的表面阶梯（`#17120f` / `#221b18` / `#2a221e`）与设计稿一致；差异集中在边框（现为 alpha，设计稿要求实色）、圆角（现 8 档，设计稿要求 4 档）与 mono 使用范围。

以下事实来自 2026-08-25 的规划审阅核验，详见 `research/plan-review-adjudication.md`：

- 令牌名称集合被 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md:26` 冻结为 448 个：`src/styles/**` 的变量名并集必须等于迁移前集合，新增名称需要专门的 token 治理任务。本父任务把 `08-25-design-token-consolidation` 指定为该治理任务。
- 设计稿要求的「chrome 层实色」在仓库中已经成立：`--surface-shell-bg` → `--material-glass-chrome-bg` → `--color-bg-elevated`，且 chrome 档 `blur: none`。四层表面阶梯（base / elevated=shell / surface=card / overlay）已存在，不需要新增 chrome 令牌。
- `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts` 断言 `--material-glass-chrome-bg: var(--color-bg-elevated)`，并在 `prefers-reduced-transparency` 块中重复断言。改动 chrome 回退目标会同时打断这两处。
- `ccr-ui/src/utils/themeBootstrap.ts:343` 的 `CUSTOM_ACCENT_VARIABLE_FAMILY` 恰为 8 个变量。自定义强调色只重写这 8 个；任何新增的 accent 相关令牌不会随自定义强调色重算。
- 后端 `ccr-ui/src-tauri/src/services/usage.rs` 的 `empty_home_platform_map()` 与逐日补齐逻辑保证：未跟踪平台同样得到长度等于所选天数的全零序列。无法用「序列全零」或 `DashboardPlatformRow.state` 判断平台是否被跟踪。可用信号是 `UsageArchiveDiagnostics.source_health[]`，其 `state` 取值为 `live` / `degraded` / `missing`。
- `useUsageSummary(platform?, startDate?, endDate?)`（`ccr-ui/src/features/usage/queries.ts:80`）的 query key 随三个参数变化，因此显式传区间即可与首页天数联动；但该 hook 没有 `enabled` 参数，延迟发起只能靠条件挂载消费组件。
- 首页区间口径由 `local_usage_date_window(days)` 定义：`end = 本地今天`，`start = end - (days - 1)`。
- `DashboardSignalStream.tsx` 现有能力包括 `all`/`warn`/`error` 三档筛选（标签带计数）、独立 `channel` 列、相邻同条目聚合 `×N`、空态 CTA 与页脚 `/monitoring` 链接。计数口径为聚合后、筛选前、截断前。
- `ccr-ui/src/features/configs/lib/flavorPreview.ts` 用 20 个十六进制字面量镜像 `tokens.css` 的表面与文本取值，无一致性机制。
- 响应式的仓库既定写法是 CSS Media Queries Level 4 区间语法加 px 字面量（例：`@media (width <= 1279px)`）。`tokens.css` 的 `--breakpoint-*` 位于标注「仅用于参考」的 `:root` 块，不在 `@theme` 中，既不生成 Tailwind 变体也不能用于 `@media` 条件。
- `ccr-ui/tests/hardcode-px-rgba.smoke.test.ts` 只覆盖 `.ts` / `.tsx` 中的 px 与 `rgba()`，带豁免登记表；不覆盖 CSS 文件，也不覆盖十六进制颜色。

## Requirements

- R1：首页必须保留现有信息与操作能力，但重建信息层级、布局、留白、排版与状态语义，使首屏优先呈现系统 readiness、推荐下一步和关键使用趋势。
- R2：首页必须在常见桌面宽度下保持紧凑、快速扫描，并为窄桌面/窗口缩放提供明确响应式退化，不出现横向溢出、卡片拥挤或关键信息截断。
- R3：视觉语言必须覆盖浅色与深色主题，并继续支持现有 flavor、accent 与字体偏好；颜色、圆角、阴影、边框、间距、排版和动效应由明确 token/semantic layer 统一管理。
- R4：样式配置页必须让用户理解并预览 theme、flavor、accent、UI font 与 monospace font 的效果；不得暴露失效组合或让预览与实际页面不一致。
- R5：设计必须符合可访问性要求，包括键盘焦点、对比度、减少动效偏好、状态不只依赖颜色表达，以及可读的最小字号/行高。
- R6：实现必须复用现有 React 组件、数据 presentation 层和路由，不改变首页后端/Tauri API 契约，不新增外部运行时依赖。
- R7：Claude Design 产物必须作为视觉与交互设计输入映射到仓库现有 React/CSS 架构；设计稿的内联 style 不得直接搬运为实现，必须落到语义 token 与既有类名体系。
- R8：设计与实现必须有回归测试，至少覆盖首页区块/关键动作、主题 token、样式配置交互、响应式契约与中英文文案键完整性。每个改动 UI 行为的子任务必须在自己的 change list 中列出被新增或被修改的测试文件；「运行既有测试」不满足本项。
- R9：同步修正会误导后续开发的 Vue 架构残留文案，但不得扩大到与本次首页/设计系统无关的文档重写。

## Acceptance Criteria

- [x] AC1（R1）：在首页可清楚识别系统总体状态、最高优先级动作、使用趋势、近期信号和平台状态，且现有导航/动作仍可达。
- [x] AC2（R1,R2）：在宽桌面（≥1440px）、常规桌面（1280px）、窄窗口（≤1024px）三档视口中，无横向滚动、重叠、截断或不可操作控件；关键首屏层级保持稳定。
- [x] AC3（R3,R4）：浅色/深色主题及现有 flavor/accent/font 组合均由同一组语义 token 驱动，设置页预览与首页实际呈现一致。
- [x] AC4（R3,R5）：文本、边框、焦点、成功/警告/错误状态满足 `theme-contrast-contract.smoke.test.ts` 的既有阈值；`prefers-reduced-motion` 下无非必要动画。阈值不得为了让新配色通过而下调。
- [x] AC5（R6）：现有首页 presentation/data hooks 与 Tauri API wire contracts 不发生破坏性变化，Web 预览允许的 native-only unavailable 状态保持诚实。
- [x] AC6（R7）：`design.md` 逐项记录设计稿 `1b` 与 `1c` 到 React 组件与 CSS token 的映射，且每条映射指向具体文件；设计稿中无法由现有数据契约支撑的元素被显式标注处置方式。
- [x] AC7（R8）：相关 smoke tests、i18n、type-check、lint 和生产构建通过；最终 Web 预览完成截图/浏览器视觉检查。
- [x] AC8（R9）：`code_map.md` 和前端包元数据不再把当前 React 架构描述为 Vue，且修改保持在架构漂移修正范围内。
- [x] AC9（R1）：`DashboardReadinessLedger` 现有的每一项信息（`readiness.status` / `labelKey` / `titleKey` / `descriptionKey` / `reasons[]` 与 `statusMetrics[]`）在新首页上都有明确承接位置或明确的删除理由，逐项记录在 `design.md`。
- [x] AC10（R3）：`tokens.css` 变更后，`src/styles/**` 的变量名并集相对 448 名称集的增量被完整登记；每个新增名称都有分类、Tailwind/Core 映射结论、四作用域配对结论与自定义强调色影响结论。

## Out of Scope

- 修改 Rust/Tauri 后端业务逻辑、usage 数据模型或 IPC wire contract。
- 重设计所有平台详情页、Profiles、MCP、Commands、Sync、Check-ins 与 Usage 完整页面。
- 新增网络服务、云同步能力、遥测数据或新的第三方前端依赖。
- 使用 Tauri 桌面自动化替代 Web 视觉验证；仅在确认需要 native window API 时追加原生验证。

## Confirmed Scope

- 平台：跨平台，以桌面和窄窗口响应式状态为本任务的可执行解释，不扩展为独立移动端产品。
- 核心流程：首页概览 → 外观设置与实时预览 → 返回首页验证统一 token 效果。
- 完成度：生产导向，设计稿只作为视觉与信息架构输入。

## Confirmed Decisions（2026-08-25，用户确认）

- D1 首页 IA 采用设计稿 `1b` 运行时卡方案：顶栏常驻环境/Profile 切换 → 四张平台卡阵列 → 左侧用量大图 + 右侧「下一步 / 事件流」。`1a` 台账方案不实施。
- D2 `1c` 令牌提案落在全局 token 层（`ccr-ui/src/styles/tokens.css`），全站页面一次性统一；代价是 Profiles、MCP、Sync、Check-ins 等既有页面同步变样，必须做全页回归走查。
- D3 Trellis 组织为父子任务树：本任务为父任务，持有需求集、设计映射、任务地图与跨子任务验收；实施拆到 6 个子任务。

## Confirmed Decisions（2026-08-25 审阅后追加）

- D4 顶栏 `PROFILE / default` 下拉不实施。应用没有全局 profile 概念，profile 按平台各自持有。顶栏只保留既有 `EnvironmentSwitcher`。
- D5 就绪 pill 落在首页区块标题行，不提升到 shell 层顶栏。理由：`MainLayoutTopbar` 位于 shell 层、不持有 dashboard presentation，提升需要引入跨层 store 依赖。R1 与 AC2 的措辞按此对齐，不再要求 Topbar 位置。
- D6 成本经既有 `useUsageSummary()` 取，显式传入与首页天数一致的 `startDate` / `endDate`。不改 hook 签名；延迟发起用条件挂载子组件实现。取不到显示 `—`，有数据且为零显示 `$0.00`，两者必须可区分。
- D7 令牌层不新增 `--color-bg-chrome`，不新增圆角角色令牌。chrome 层复用既有 `--surface-shell-*` 链路；圆角收敛做成 7 个既有令牌的取值修改。`08-25-design-token-consolidation` 同时承担 448 名称集的治理职责，负责登记确有必要的新增名称。
- D8 平台卡的「未跟踪」判据用 `archive.source_health[]`，不用全零序列或 CLI 安装状态。若 `source` 字段取值无法对应 `usageKey`，则该子任务降级为不显示占位态并记录原因，不得用零值冒充占位。
- D9 事件流保留既有筛选、`channel` 列与聚合 `×N`；设计稿的三列布局按此扩充，不做能力削减。计数口径固定为聚合后、筛选前、截断前。

## Task Map

| 子任务 | 目录 | 交付物 | 前置 | 形态 |
|---|---|---|---|---|
| 设计令牌层收敛与治理 | `.trellis/tasks/08-25-design-token-consolidation` | `tokens.css` 边框/圆角/字体取值收敛、名称增量登记、既有测试迁移 | 无 | 复杂 |
| 首页 1b 运行时布局 | `.trellis/tasks/08-25-home-runtime-layout` | 平台卡阵列、`DashboardView` 重排、readiness 信息重新落位 | 令牌层 | 复杂 |
| 首页用量与成本图表区 | `.trellis/tasks/08-25-home-usage-chart` | 指标行、堆叠日柱图、7/30/90 切换、成本接入 | 令牌层、运行时布局 | 复杂 |
| 首页右侧栏 | `.trellis/tasks/08-25-home-side-rail` | 「下一步」与「事件流」呈现层，保留既有筛选与 channel | 令牌层、运行时布局 | 复杂 |
| 外观设置页重排 | `.trellis/tasks/08-25-appearance-settings-refresh` | 分区重排、flavor 预览取值单一来源 | 令牌层 | 复杂 |
| 修正 Vue 架构残留文案 | `.trellis/tasks/08-25-arch-drift-docs` | `code_map.md`、`package.json`、`ccr-ui/CLAUDE.md` | 无 | 轻量 |

排序不是依赖系统：前置关系写在各子任务的 `prd.md` / `implement.md` 中，每个子任务的验收标准独立可测。
`home-usage-chart` 与 `home-side-rail` 的前置都含运行时布局，因为两者的容器栅格由该子任务建立。

## Cross-Child Acceptance

- [x] XC1：本任务新增或改写的 CSS 文件中无硬编码颜色、圆角或边框字面量。判据只针对本任务改动的文件清单，既有历史代码与已登记豁免不计。
- [x] XC2：`just frontend-check` 在集成后通过（exit 0）；`just version-check` 与 `just fmt-check` 不因本任务失败。用户发布门不含 `just ui-check`，本项按用户目标核 `frontend-check`。
- [x] XC3：全页回归走查覆盖 Dashboard、Profiles、MCP、Commands、Sync、Check-ins、Usage、Settings 在明暗两主题 × neutral/clay 两 flavor 下的表现，记录到父任务 `research/regression-walkthrough.md`。
- [x] XC4：`ccr-ui/src-tauri/Cargo.toml` 的用户未提交改动在任何子任务的工作区、暂存区与最终提交中都不被覆盖或混入。
- [x] XC5：每个改动 UI 行为的子任务都有对应的测试文件改动，且这些文件在其 change list 中列出。仅运行既有测试不满足 R8。
- [x] XC6：`tokens.css` 的名称增量已按 AC10 登记，`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 同步更新；受影响的既有断言已迁移而非删除。

## Open Questions

- 无阻塞项。设计输入、IA 方向、令牌范围与任务组织均已确认；2026-08-25 审阅提出的 12 项已逐条裁定，见 `research/plan-review-adjudication.md`。
