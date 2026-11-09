# Design — Overview 首页重构：图表比例与信息可信度

> 方向契约 FIRST VIEWPORT：顶部行情带 → 命令列 → 固定行高用量面板 → 右栏日志。本设计把首页从"卡片仪表板"翻译为"行情终端"，壳层侧栏/顶栏不动（那是全 app 共享面）。

## 边界

- 只动 Overview 页面域：`features/usage/dashboard/**`、`features/usage/styles/dashboard-*.css`、`views/dashboard/dashboardPresentation.ts`、必要时 `src-tauri` 的 usage 响应。
- 不改壳层（sidebar/topbar 归 settings-dock 与后续任务）；不动 i18n 架构；`dashboard.*` key 的增删见 §5。

## 1. 图表固定尺度（核心缺陷修复）

现状膨胀链：`.dashboard-lower{align-items:stretch}` + `.dashboard-usage{height:100%}` + `.dashboard-usage__chart{flex:1; min-height:8rem; 无上限}`。

方案（`features/usage/styles/dashboard-usage-movement.css` + `dashboard-view.css`）：
- `.dashboard-usage__chart`：去掉 `flex:1`，改 **`height: clamp(10rem, 26vh, 16rem)`**——图表高度只跟视口走，永不被右栏撑开；柱 100% 即这个盒子的 100%。
- `.dashboard-usage`：去掉 `height:100%`，改内容自适应；面板高度 = 头 + 指标行 + 固定图表 + 轴/页脚。
- `.dashboard-lower`：`align-items: stretch` → `start`；右栏 `.dashboard-rail` 自己定高，超长时 `.dashboard-signals__list` 加 `overflow-y:auto` + `max-height`（右栏内部滚动，不反哺左列）。
- 图表加终端刻度纪律：水平发丝网格线（3-4 条，`border-top` hairline，低对比），柱顶不设圆角（终端是方柱；若现有段有 radius 则去掉）。遵守 craft-floor：网格线合法因为图表本身就是量具。
- 数字字体：用量面板的指标数字加 `font-variant-numeric: tabular-nums`（等宽数字是数据正当用途，不是"技术感戏服"）。

## 2. 平台矩阵 → 行情带

`DashboardPlatformMatrix.tsx` 的四个大卡片改为**单行行情带**（视宽不足时横向滚动，不换行）：
- 每个平台一格：线色 tick（2px 顶条或左侧 4px 色块，用确权后的 `--color-platform-*`）+ 名称 + 版本（muted）+ `请求`/`TOKEN` 两组等宽数字 + 既有 `PlatformSparkline` 迷你柱（保留，固定 2.5rem 高）。
- 格间发丝分隔线（`border-right: 1px solid var(--color-border-subtle)`），去掉大卡片 padding 与独立表面——整带是一个 `--surface-card` 面板内的行。
- 状态徽标（Ready/需处理）保留，语义色点 + 文字。
- sessions 指标：`buildPlatformRows` 已构造未渲染 → 在格内渲染为第三组数字（小字 muted）。无数据时显示 `–` 而非 0（配合 §3）。

## 3. Sessions 诚实态

决策树（按序落实，取第一层可行者）：
1. 查 `src/api/generated/usageV2.ts` 的 overview 响应类型是否已含 `needs_session_index`（后端 `services/usage.rs:1151` 有内部计算）。若已在响应里 → 直接消费。
2. 若响应没有但后端结构体已有该字段 → 补序列化字段 + 重新生成/手写同步 TS 类型（遵循该 generated 文件的既有同步方式，先读文件头注释确认是否自动生成）。
3. 都不行的兜底：前端启发式——`requests>0 && sessions===0` 时显示"会话归档未索引"。
呈现：用量面板 `会话` 指标与平台格 sessions 位显示"未索引"状态（info 色 + 文案 + 指向用量页的链接），**不是静默 0**。新增 i18n key 进 `dashboard.*`（双语言，见 §5 归属）。

## 4. 页头与右栏的终端化排版（`DashboardView.tsx` 骨架重组）

- 状态检查项 pills（`.dashboard-header__reasons`）保留语义，视觉收敛为状态行：每项 = 语义色点 + 文字，发丝分隔，去掉 pill 盒子的重填充（颜色只住状态点——落选挑战者纪律）。
- Event stream（`DashboardSignalStream.tsx`）：日志行 = 等宽时间列 + 级别点 + 域标签 + 消息，行高分行固定，`tabular-nums`。
- Action queue（`DashboardNextActions.tsx`）：命令列表行 = 图标 + 命令名 + 一句说明 + 箭头；首条高亮用 `--color-bg-overlay` 级表面而非大面积琥珀填充。
- 页脚"打开完整报表/打开监控"链接保留。

## 5. i18n key 变动归属

- 本任务新增：`dashboard.usage.sessionsUnindexed` 等少数 key（双语言）。
- 死键删除（`dashboard.usage.peakLabel/hoverHint/metricSelectLabel/metricPlatforms`）**移交 settings-i18n 子任务**统一做（它独占 locale 大改，避免两子任务并改 `zh-CN.ts`/`en-US.ts` 冲突）。本任务新增 key 时同步更新 `check-i18n.mjs` 的 `EXPECTED_LEAF_COUNT`（当前 4404，每加一个 key 两个 locale 各 +1）。

## 6. 不做

- 不改壳层、顶栏、其他页面；不引入图表库（手写柱保留，修的是容器不是渲染器）；不动 `theme.css` 桥。

## 验证

- `bunx vitest run --config vitest.smoke.config.ts tests/dashboard/`
- `cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build`；若动了 Rust：`bun run tauri:check && bun run tauri:test`
- 视觉：dev:web 下 1600×1000 与窄窗截图——图表高度有界、行情带单行、右栏自滚动。
