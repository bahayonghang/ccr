# 首页 Insights 界面设计定稿

父任务：`09-24-home-insights-redesign`（决策 D1-D3 与现状事实见父任务 `prd.md`）。

## Goal

在写前端代码之前，确定首页 Insights 区块的视觉与交互规则，并写入 `ccr-ui/DESIGN.md` 与 `ccr-ui/src/styles/tokens.css`，使 frontend 子任务只按规则实现、不做设计判断。同时复现并定位截图中的首屏图表越界问题。

## 使用的技能

- `frontend-design`：确定 Insights 区块在行情终端（market terminal）世界中的构图与层级。
- `dataviz`：为热力图、堆叠周趋势、小时分布、榜单条选择图形形式、颜色梯度与标注规则；用其颜色校验器检查亮/暗两套主题的对比度。
- `design`：产出多画板设计画布（Artifact），供用户在画布上调整后定稿。
- `artifact-design`：画布发布前的设计检查。

## Requirements

- R1 构图：首屏构图不变（`DESIGN.md` Layout 段，`ccr-ui/DESIGN.md:199-205`）。Insights 位于首屏之下，与首屏共用同一滚动容器与底部命令状态栏。
- R2 区块清单与顺序（可调整顺序，不可删减）：总量行、最近 7 天对比、53 周活动热力图（含 streak/busiest day 说明行）、按周分 agent 堆叠趋势、按小时/星期/月分布（同一面板三视图切换）、Agents/Projects/Models 榜单（度量切换：请求数/token/会话数；会话数只对 Agents 可用）。
- R3 规避被否决的套路：`DESIGN.md:306` 禁止"指标卡片行 + 无界主图"。总量行与 7 天对比必须采用非卡片形式（例如行情带式单行数值、发丝线分隔），并在 `DESIGN.md` 中写明形式。
- R4 所有图表有界：给出每个图表的高度规则（`clamp()` 或固定行高），与现有 `clamp(10rem, 26vh, 16rem)` 规则并列写入 `DESIGN.md`。
- R5 颜色：
  - 为 `kimi_code`、`pi`、`zcode`、`deepseek_harness` 四个来源补身份色，亮/暗两套，写入 `tokens.css` 与 `DESIGN.md` Platform Identity 段（现仅 6 个，`ccr-ui/DESIGN.md:163-165`）。
  - 周趋势超过 6 个来源时的合并规则（例如前 N 个来源独立着色，其余并为 "其他" 中性色）。
  - 热力图梯度：遵守 Amber Scarcity Rule 与 Status-Only Color Rule（`ccr-ui/DESIGN.md:167-173`）。梯度不得使用 amber 与状态色；给出 0 值格与 4-5 级梯度的 token。
  - 亮/暗两套主题的对比度满足 WCAG AA 非文本 3:1（图形）与文本 4.5:1，校验结果写入任务 `research/`。
- R6 空态：为父任务 AC-P5 的四种状态各给出一个画板。
- R7 数字格式：大数缩写沿用首屏 `compactLabel`（跟随运行时默认 locale，即 WebView 系统语言，不跟随应用界面语言），完整值放 `title`（用户决定，2026-09-24）；百分比变化的正负表示（不使用红绿以外的额外颜色；变化方向用符号 + 文字，颜色只在状态语义成立时使用）。
- R8 首屏图表越界：在 Web 预览（`bun run dev:web -- --host 127.0.0.1 --strictPort`）中复现截图中的用量图高度超过 16rem 的现象，定位原因（CSS 未加载、层叠覆盖或布局拉伸），把结论与修复方案写入 `research/chart-overflow.md`。只定位，不改代码；修复在 frontend 子任务执行。若 Web 预览无法复现（例如只在 Tauri 运行时出现），记录环境差异。

## Acceptance Criteria

- AC1 设计画布已发布为 Artifact，包含：首屏 + Insights 全页（暗色）、同页亮色、四种空态、1280px 窄宽度下的 Insights 布局。用户已在画布上确认。
- AC2 `ccr-ui/DESIGN.md` 新增 Insights 签名组件条目（热力图、周趋势、分布面板、榜单、总量行、7 天对比），每条含：形式、高度规则、颜色 token、空态、Do/Don't。
- AC3 `tokens.css` 新增 4 个来源身份色与热力图梯度 token；`.impeccable/design.json`（DESIGN.md 的机器可读副本）同步。本子任务同时是这批 token 名的治理登记任务（`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md:49` 要求新增 token 名必须有专门的治理登记）：在该规格文件追加登记段（新增名清单与数量、唯一名总数变化），并在 `research/token-names-before.txt` / `token-names-after.txt` 留下名集对比证据。新增名只放 layer 1（`tokens.css` 的 `:root` 与 `[data-theme='dark']`）；需要 Tailwind 工具类时才加 `@theme inline` 映射，且值只能是 `var()` 引用。
- AC4 `research/palette-validation.md` 记录 dataviz 校验器对亮/暗主题的输出。
- AC5 `research/chart-overflow.md` 给出越界现象的复现步骤与原因，或说明无法复现的环境条件。
- AC6 `bun run test`（含 theme token smoke 测试）通过；`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 规定的检查通过。

## Out of Scope

- 组件实现与数据接入（frontend 子任务）。
- 首屏平台行情带扩展到 9 个来源（父任务不在范围内）。
