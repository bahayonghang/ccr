# Overview 首页：铺满工作区 + 指标徽章

## Goal

去掉 Overview 与全部 `PageShell` 页在宽屏上的居中空底。就绪条和用量摘要里的数字改成可扫的徽章。视觉世界仍是 Editorial Control Room。

用户打开 Overview 时先看到状态数字，不再先看到两侧空底。走 `PageShell` 的平台页同样铺满主工作区。

## Background

`08-18-ui-visual-refactor` 把 Dashboard 锁在 `1440px` 居中，把 `PageShell` 锁在 `1480px` 居中。用户在约 2536px 宽窗口里标出：侧栏与内容之间、内容右侧的大块空底，以及就绪条五个数字没有徽章。

访客模式：Operate。本任务是在既有世界里改结构和原语，不换视觉世界。

2026-08-18 用户确认：Overview 与全部 `PageShell` 页一起放开居中上限。

## Confirmed Facts

- 首页是 `DashboardView.vue`，不走 `PageShell`。黑边主因：`.dashboard-workbench` 的 `width: min(100%, 1440px); margin: 0 auto;`（`DashboardView.vue:330-335`）。主滚动区另有 `content-scroll-area` 的 `p-4 sm:p-6`（`MainLayout.vue:228`）。约 2270px 可用宽里内容锁在 1440px，左右各剩约 390px `--color-bg-base`。
- `PageShell` 的 `max-width: 1480px; margin-inline: auto`（`PageShell.vue:36-44`）约束 40+ 个平台/工具页。`OpenCodePageShell` 包在 `PageShell` 上，随同一处修改。
- 设置页 `AppSettingsView` 自有 `max-w-[1440px]`，不走 `PageShell`。`profiles-page.css` 另有 `1680px`。二者不是本次截图来源。
- 就绪条五个指标来自 `dashboardPresentation.statusMetrics`。类型已有 `tone: DashboardTone`（`neutral | success | warning | danger | accent`），`buildStatusMetrics()` 已按主机、后端、CLI、用量、信号赋值。
- `DashboardReadinessLedger.vue:41-47` 只把 `label` / `value` / `hint` 传给 `StatTile`，丢掉 `tone`。
- `StatTile` 是裸排版：label + 1.5rem 主色数字 + hint，`tabular-nums`，无外壳。消费点还包括 Usage、Checkin、Budget、Grok、Codex、OpenCode、Tray。`ui-primitives.smoke.test.ts` 锁「bare tile、无 `ui-card`」。
- `Badge.vue` 已有 square / pill 与语义 variant。DESIGN.md：Badge 默认 6–8px 方角；pill 只给状态点 + 短词。
- 仍有效的禁令：禁嵌套卡、禁 side-stripe、禁语义色直接涂数字、禁实心 accent 填统计数字、静止卡无投影或仅 `--shadow-sm`、accent 占用 < 10%。
- `dashboardPresentation.ts` 的 signal / readiness / `isFirstRun` 受 `dashboard-presentation-contracts.md` 约束。

## Requirements

### R1 铺满主工作区

- 删除 `DashboardView` 工作区的 1440px 居中上限。内容在 `content-scroll-area` 内横向铺满。
- 删除 `PageShell` 的 `1480px` 与 `margin-inline: auto`。全部 `PageShell` 消费页（含 `OpenCodePageShell`）随同一处修改铺满。
- 保留 `content-scroll-area` 的 `p-4 sm:p-6` 与 `PageShell` 现有内边距。不在本任务里拆这两层 padding。
- 宽屏（≥1800 可用宽）下，由 1440/1480 居中造成的空底消失。侧栏到内容卡、内容卡到窗口右缘的空隙只剩上述已有内边距。
- `<=1180` 的 Dashboard 单列回退保持。
- 不改 `AppSettingsView` 的 `max-w-[1440px]`，不改 `profiles-page.css` 的 1680px，不改各页内部更窄的阅读列（定价、说明文）。

### R2 指标徽章

- `StatTile` 增加可选 `tone`，值域与 `DashboardTone` 对齐。未传 `tone` 时保持现有裸瓦片，Checkin / Budget / Grok / Codex / OpenCode / Tray / Usage 页头默认外观不变。
- 传入 `tone` 时，只把数值放进方角徽章壳；label 与 hint 留在壳外。
- 数字颜色保持 `--color-text-primary` + `tabular-nums`。状态只用壳的浅底（8–12%）+ 1px 边 + 可选小圆点。
- `accent` tone 用 clay 8–12% 浅底。禁止实心 accent 填数字或整块瓦片。
- 徽章壳 `inline-flex`，宽度跟数字走，不拉满栅格列。
- 禁止把整个 `StatTile` 再套一张卡。保持「无 `ui-card`」。
- `DashboardReadinessLedger` 把已有 `metric.tone` 传给 `StatTile`。
- `DashboardUsageMovement` 的四个摘要 `StatTile` 传 `tone="neutral"`，与就绪条共用原语。
- 不在本任务给其他页面的 `StatTile` 补 tone，除非那一页已经有现成 tone 字段且用户另开需求。

### R3 首页观感

- 就绪条五指标在宽屏上等分；长数字（如 `35.5% / 76.2%`）不挤乱邻格。
- 徽章圆角与卡内 padding 同心：卡 12px、卡内 pad 16px 时，内壳 8px。
- 徽章不可点。不加位移、不加 glow。hover / focus 只留在可点击控件上。
- 深浅色 × `neutral` / `clay` 下，徽章对比度不低于现有 text/surface 阈值。
- 不引入新 flavor / accent，不写字面 hex/rgb，不回退 inset 高光。

### R4 合同与验证

- 不改 `dashboardPresentation` 的 signal / readiness / first-run 判定。
- 更新 `ui-primitives.smoke.test.ts`：无 `tone` 仍是裸瓦片；有 `tone` 时数值在徽章壳内，源码仍含 `tabular-nums`，仍无 `ui-card`。
- 把「tone 只驱动壳、不驱动数字色」写入 `dashboard-presentation-contracts.md`。必要时同步 `ccr-ui/DESIGN.md` 中「禁语义色装饰数字」一句，改为禁止把数字本身涂成语义色。
- 视觉证据放本任务 `evidence/`，命名 `{route}-{locale}-{theme}-{flavor}-{width}.png`。至少：
  - `/`：zh-CN × dark/light × neutral × 1440 与 1920
  - 一个 `PageShell` 页（建议 `/claude-code`）：zh-CN × dark × neutral × 1920

## Acceptance Criteria

- [ ] Overview 在 ≥1920 可用宽下，不再出现 1440 居中留下的空底。
- [ ] 任一 `PageShell` 页在 ≥1920 可用宽下，不再出现 1480 居中留下的空底。
- [ ] 就绪条五个数字带 tone 徽章壳；`metric.tone` 不再被丢掉。
- [ ] 用量摘要四个数字带 `tone="neutral"` 的同一套壳。
- [ ] 未传 `tone` 的 `StatTile` 外观与改前一致。
- [ ] 数字本身是主文本色 + `tabular-nums`，不是语义色字，不是实心 accent。
- [ ] 无嵌套卡、无 side-stripe、无装饰 glow、无新字面色。
- [ ] `dashboardPresentation` 语义判定不变；现有 presentation smoke 仍绿。
- [ ] `cd ccr-ui && bun run type-check`、`bun run lint`、`ui-primitives` 与 `dashboard-presentation` smoke 通过。
- [ ] `evidence/` 截图清单齐。

## Out of Scope

- 换视觉世界、新 flavor / accent、营销式大数字英雄区。
- 改路由、Tauri、`llmusage`、监控判定、first-run 启发式。
- `AppSettingsView`、`profiles-page.css`、各页内部阅读列的 max-width。
- 给 Checkin / Budget / Grok / Codex / OpenCode / Tray 补徽章。
- 平台矩阵、事件流、行动队列的交互或文案。
- `home.css` 遗留 vw clamp 的全量清理。
- 合并或取消 `content-scroll-area` 与 `PageShell` 的双层 padding。
