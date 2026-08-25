# 技术设计：首页 1b 重建与全局令牌层收敛

设计输入：`research/claude-design-source.html`（Claude Design 项目 `0a3d3dfa-8ad5-4bdf-861d-305f1e2c6389`）。
本文件是父任务的设计映射，子任务的实现细节写在各自的 `design.md` / `implement.md`。

---

## 1. 边界与契约

### 不变更项

- Tauri IPC 命令签名与返回类型不变，不新增 `#[tauri::command]`。
- `src/api/` 下的 invoke 封装不新增、不改签名。
- 路由表 `src/shell/routeCatalog.ts` 的路径与 id 不变。
- 不新增前端运行时依赖。

### 允许变更项

- `src/views/dashboard/dashboardPresentation.ts` 的导出类型允许**加字段**（不删不改已有字段）。
- 首页组件的内部 DOM 结构与 CSS 类名。
- `src/styles/tokens.css` 的令牌取值与新增语义令牌。

### 数据契约核对结论

| 设计元素                 | 数据来源                                                                                        | 结论               |
| ------------------------ | ----------------------------------------------------------------------------------------------- | ------------------ |
| 平台卡 7 日 sparkline    | `HomeUsageOverviewResponse.series[].{claude,codex,antigravity,opencode}.requests`               | 可派生，无需改契约 |
| 堆叠日柱图（按平台分层） | 同上                                                                                            | 可派生，无需改契约 |
| 请求 / TOKEN / 会话 指标 | `HomeUsageOverviewResponse.summary`                                                             | 直接可用           |
| 预估成本                 | `HomeUsageOverviewResponse` **不含**；需 `useUsageSummary()` → `UsageSummaryDto.total_cost_usd` | 见 D-3             |
| 就绪 `3/3`               | `buildDashboardPresentation().readiness`                                                        | 直接可用           |
| 顶栏 `PROFILE / default` | **无对应概念**                                                                                  | 见 D-4             |
| 事件流                   | `useDashboardSignals()`                                                                         | 直接可用           |

---

## 2. 令牌层设计（对应子任务 `08-25-design-token-consolidation`）

### 2.1 表面阶梯

设计稿 clay 暗色四级表面与现有 `tokens.css` 的对照：

| 设计稿 | 值 | 现有令牌 | 差异 |
|---|---|---|---|
| base | `#17120F` | `--color-bg-base` | 一致 |
| chrome | `#1C1613` | `--color-bg-elevated`（经 `--surface-shell-bg` → `--material-glass-chrome-bg`） | 层级已存在，取值 `#221B18` |
| card | `#221B18` | `--color-bg-elevated` | 一致 |
| raised | `#2A221E` | `--color-bg-surface` | 一致 |

设计稿把 chrome 与 card 画成两个不同取值。仓库现状是二者共用 `--color-bg-elevated`，
card 实际落在 `--color-bg-surface`（`#2A221E`），因此 chrome（elevated）与 card（surface）在渲染上仍然可辨：

```
base #17120f  <  shell/chrome #221b18  <  card #2a221e  <  overlay #342b26
```

四层阶梯成立。**不新增 `--color-bg-chrome`**（决策 D7）：

- 新增名称会打破 `theme-token-contracts.md:26` 的 448 名称冻结。
- `--material-glass-chrome-bg: var(--color-bg-elevated)` 被 `apple-glass-surface-contract.smoke.test.ts`
  在主块与 `prefers-reduced-transparency` 块中各断言一次，改回退目标会同时打断两处。
- 现有 chrome 档已是 `blur: none` 的实色，设计稿要的「实色 chrome」已经成立。

组件侧是否真的消费了 `--surface-shell-*`，由 `08-25-home-runtime-layout` 在其第一步核查；
若绕过语义别名，那是组件侧问题，改法是接回别名，不是新建令牌。

### 2.2 边框：alpha → 实色

设计稿的判断是半透明边框在深底上糊成一团。现有 clay 暗色为 `rgb(243 234 223 / 14% | 22% | 34%)`，改为：

| 令牌                     | clay 暗色新值 |
| ------------------------ | ------------- |
| `--color-border-subtle`  | `#322A25`     |
| `--color-border-default` | `#3A302A`     |
| `--color-border-strong`  | `#4A3D35`     |

`--color-border-*-rgb` 伴随令牌必须同步为实色的 RGB 分量，否则以 `rgb(var(--...-rgb) / x%)` 组合的调用点会取到错误底色。neutral 明暗与 clay 亮色按同样口径各出一组实色值。

### 2.3 圆角：8 档收到 4 档

设计稿保留 `6 chip / 8 控件 / 12 卡片 / pill`，删除 `4 / 10 / 16`。

**不新增角色令牌**（决策 D7）。`core.css:153-159` 已把 7 个既有圆角令牌映射进 `@theme inline`，
改这 7 个的**取值**即可达成四档收敛，既不打破名称冻结，也不需要改 `core.css`：

```
--radius-none: 0;        /* 不变 */
--radius-sm: 6px;        /* was 4px  → chip */
--radius-md: 6px;        /* 不变      → chip */
--radius-lg: 8px;        /* 不变      → control */
--radius-xl: 12px;       /* was 10px → card */
--radius-2xl: 12px;      /* 不变      → card */
--radius-3xl: 12px;      /* was 16px → card */
--radius-full: 9999px;   /* 不变      → pill */
```

新代码的规范入口是四个既有名：`--radius-md`（chip）、`--radius-lg`（control）、
`--radius-2xl`（card）、`--radius-full`（pill）。这条约定写在文档里，不靠新令牌名承载。

副作用是原本 4px 的 chip 变 6px、原本 16px 的容器变 12px，属于预期的视觉收敛，
回归走查时按此判定，不当作缺陷。

### 2.4 语义色与平台色

语义色（前景 / 底色对）取设计稿值：

| 角色    | 前景      | 底色      |
| ------- | --------- | --------- |
| accent  | `#E8835B` | `#33231B` |
| success | `#7CAB82` | `#25332A` |
| warning | `#D6A76D` | `#3A2A20` |
| danger  | `#DB8A73` | `#2B1F1C` |
| info    | `#98AFC9` | `#252D33` |

现有令牌已有 `--color-{success,warning,danger,info}` 前景，缺配套的 tint 底色。
是否新增 `--color-{role}-tint` 由令牌子任务的名称增量审计决定：既有
`--color-stage-chip-neutral-{bg,border,text}` 是单一中性档，能否承载五色语义待判。

**accent tint 的额外约束**：`themeBootstrap.ts:343` 的 `CUSTOM_ACCENT_VARIABLE_FAMILY` 恰为 8 个变量，
`applyCustomAccent()` 只重写这 8 个。新增 `--color-accent-tint` 不会随自定义强调色重算，
会出现主色变了而 tint 不变的色相不一致。令牌子任务在两个选项中二选一并记录理由：
用 `rgb(var(--color-accent-primary-rgb) / 12%)` 表达（自动跟随，不新增名称），
或新增名称并同步扩展 `CUSTOM_ACCENT_VARIABLE_FAMILY` 与其生成逻辑。

平台色：`claude #D97757` / `codex #7CAB82` / `grok #8B839C` / `antigravity #98AFC9` / `opencode #735F52`。
现有令牌缺 `--color-platform-opencode`（`ccr-ui/src/styles/` 中无任何 `opencode` 命中），
这是**确需新增**的名称，走名称治理流程。1b 的四张卡中 OpenCode 是其一，当前代码用 `text-accent-info` 顶替，属于要修的语义错配。

命名漂移记录：令牌名为 `--color-platform-gemini`，但产品与路由已改称 antigravity。本任务**不重命名**，只在 `design.md` 记录；重命名单独排期。

使用规则（写入子任务验收）：平台色只出现在 3–4px 的识别标记与图表分层里，不作为整块卡片底色。

### 2.5 排版

| 角色 | 设计稿规格 | 既有最近档 | 差 |
|---|---|---|---|
| 页标题 | 17px / 600 / -0.01em | `--text-lg` = 1.0625rem (17px) | 0 |
| 正文 | 14px / 400 | `--text-sm` = 0.8125rem (13px) | 1px |
| 次要文本 | 13px | `--text-sm` = 0.8125rem (13px) | 0 |
| 标签 | mono 10px / 600 / 0.16em | `--text-xs` = 0.75rem (12px) | 2px |
| 数据 hero | mono 28px / 600 / -0.02em | `--text-2xl` = 1.625rem (26px) | 2px |
| 数据次级 | mono 20px / 600 | `--text-xl` = 1.3125rem (21px) | 1px |

四项差异都在 1–2px 内。**默认结论是复用既有档位，不新增 `--text-data-*` 角色令牌**（决策 D7）。
`theme-token-contracts.md:10` 已登记一条字号例外（Profiles 共享层可用 `0.75rem`，唯一的次 Label 档），
新增更小档位会与该例外冲突。
令牌子任务在名称增量审计中逐档确认；只有实施者在真实浏览器中确认某档近似造成明显视觉问题时才走新增流程。

mono 使用范围收敛为：**数字、版本号、时间戳、路径、代码**。界面文案、标题、说明文字一律 sans。
这条规则写在文档里，不靠令牌名承载。约束「一屏只允许一个 hero 档数字」写进首页子任务验收。

---

## 3. 首页 1b 组件映射（对应三个 home-* 子任务）

```
MainLayoutTopbar        ← 面包屑 + 环境切换（均保留，不新增元素；决策 D5）
└─ DashboardView
   ├─ 区块标题行                 → 就绪 pill + 主行动按钮（设计稿画在顶栏，实际落这里）
   ├─ DashboardPlatformMatrix   → 改写为四张运行时卡（原地改写，props 契约不变）
   ├─ DashboardUsageMovement    → 改写为指标行 + 堆叠日柱图 + 7/30/90 切换
   │  └─ DashboardCostMetric    → 新增：唯一调用 useUsageSummary 的组件，首屏后条件挂载
   ├─ DashboardNextActions      → 右侧栏上半
   └─ DashboardSignalStream     → 右侧栏下半（保留筛选、channel、聚合；决策 D9）
   DashboardReadinessLedger     → 从首页移除，信息逐项落位见 D-2
```

| 设计元素                               | 目标文件                                                                                                              |
| -------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| 侧栏实色化、分组标题 mono、平台色点    | `src/shell/MainLayoutChrome.tsx`、`src/shell/MainLayoutNav.tsx`、`src/shell/shell.css`                                |
| 顶栏面包屑 / 环境 / 就绪 pill / 主行动 | `src/shell/MainLayoutChrome.tsx`、`src/shell/EnvironmentSwitcher.tsx`                                                 |
| 四张平台卡                             | `src/features/usage/dashboard/DashboardPlatformMatrix.tsx`、`src/features/usage/styles/dashboard-platform-matrix.css` |
| 用量与成本大图                         | `src/features/usage/dashboard/DashboardUsageMovement.tsx`、`.../styles/dashboard-usage-movement.css`                  |
| 下一步                                 | `src/features/usage/dashboard/DashboardNextActions.tsx`                                                               |
| 事件流                                 | `src/features/usage/dashboard/DashboardSignalStream.tsx`                                                              |
| 首页栅格                               | `src/features/usage/dashboard/DashboardView.tsx`、`.../styles/dashboard-view.css`                                     |
| 外观设置                               | `src/features/configs/settings/AppearanceSection.tsx`、`FlavorCard.tsx`、`ThemeOption.tsx`                            |

### sparkline 数据落点

`DashboardPlatformRow` 增补可选字段：

```ts
export interface DashboardPlatformRow extends DashboardPlatformSource {
  state: DashboardPlatformState;
  stateKey: string;
  version?: string;
  versionKey?: string;
  metrics: DashboardMetricValue[];
  sparkline?: number[]; // 新增：按天的 requests，长度 = activeDays
}
```

派生逻辑放在 `buildDashboardPresentation`，输入已有的 `overview`。这是加字段，不破坏现有调用点，符合 R6。

**注意**：后端 `empty_home_platform_map()` 与逐日补齐保证未跟踪平台也得到全零序列，
因此 `sparkline` 在 `series` 非空时总会是数组。`sparkline === undefined` **不是**未跟踪的判据，
未跟踪判据用 `archive.source_health[]`（决策 D8）。

---

## 4. 设计决策与偏离

- **D-1 平台卡数量为 4。** 设计稿侧栏列 5 个平台（含 Grok），卡阵列只有 4 个。现有 `DashboardView.platforms` 也是 4 个，与设计稿一致。是否把 Grok 纳入首页卡阵列不在本任务范围。
- **D-2 `DashboardReadinessLedger` 退出首页，信息逐项落位。** 1b 没有独立的就绪台账区。
  该组件消费 `readiness`（`status`、`labelKey`、`titleKey`、`descriptionKey`、`reasons[]`）与 `statusMetrics[]` 两组数据。
  就绪 pill 只能承载 `status` 与一个聚合数字；`statusMetrics` 含 CPU、内存、后端、CLI 计数，
  **与用量指标行不是同一组数据**，用量指标行是 requests / tokens / cost / sessions，不承接 `statusMetrics`。
  `08-25-home-runtime-layout` 的 `design.md` §7.2 六行落位表已于 2026-08-25 填完（父任务 AC9 关闭）：

  | 来源字段 | 落位结论 |
  |---|---|
  | `readiness.status` | 迁移到区块标题行就绪 pill 的 `data-status` |
  | `readiness.labelKey` | 迁移到就绪 pill 文案 |
  | `readiness.titleKey` | 迁移到 `dashboard-header__readiness-title` |
  | `readiness.descriptionKey` | 迁移到 `PageHeader` 描述 |
  | `readiness.reasons[]` | 迁移到标题行下方 checklist；失败条数写入 pill |
  | `statusMetrics[]` | 删除。1b 首屏是平台运行时卡；CPU/内存不在 1b；CLI/后端已出现在平台卡与 `reasons[]`；用量指标行不承接本组。presentation 仍产出该数组 |

  组件文件已删除。归档路径：`.trellis/tasks/archive/2026-08/08-25-home-runtime-layout/design.md`。
- **D-3 成本走 `useUsageSummary()`，显式传区间。** home overview DTO 无 cost。
  `useUsageSummary(platform?, startDate?, endDate?)` 的 query key 随参数变化，
  因此传 `end = 本地今天` / `start = end - (activeDays - 1)` 即可与首页同区间并在切档时重取。
  该 hook 无 `enabled` 参数，延迟发起用条件挂载 `DashboardCostMetric` 实现，不改 hook 签名。
  取不到显示 `—`，有数据且为零显示 `$0.00`，两者可区分。
- **D-4 顶栏 PROFILE 下拉不实施。** 应用中没有全局 profile 概念，profile 是各平台各自的（codex profiles、claude profiles）。造一个假的全局选择器会与 AC5 的诚实要求冲突。顶栏只保留既有的 `EnvironmentSwitcher`。若后续要做全局 profile，另开任务。
- **D-5 chrome 层不新建令牌，就绪 pill 不上顶栏。**
  chrome 档已是 `blur: none` 的实色（`--surface-shell-bg` → `--material-glass-chrome-bg` → `--color-bg-elevated`），
  设计稿要的实色 chrome 已成立，见 §2.1。
  `MainLayoutTopbar` 在 shell 层不持有 dashboard presentation，把 readiness 提上去需要跨层 store 依赖，
  因此 pill 落在 `DashboardView` 的区块标题行，视觉高度相邻，满足「同屏可见」。
- **D-6 设计稿的内联 style 不搬运。** 设计稿是 canvas 文档，全部样式是内联字面值。实现一律走语义令牌与既有类名体系；子任务的检查项包含「新增 CSS 中无硬编码十六进制颜色」。
- **D-7 事件流不做能力削减。** 设计稿的三列事件行不含筛选、`channel` 列与聚合 `×N`，
  但现有 `DashboardSignalStream` 三者都有。设计稿是视觉参考，不是能力清单；三者全部保留，
  行栅格由三列扩为四列。计数口径固定为聚合后、筛选前、截断前——这是既有正确行为，
  「标题计数 ≠ 可见行数」由聚合与 `limit` 截断造成，不判为缺陷。

---

## 5. 兼容性与回滚

- 令牌层改动是纯 CSS 变量取值变更，回滚 = 还原 `tokens.css` 单文件。
- 首页组件改写按子任务分文件落地，单个子任务可独立 revert 而不影响其他子任务，前提是各子任务不越界改动 `DashboardView.tsx` 的同一区块——栅格骨架由 `08-25-home-runtime-layout` 单独负责，其余子任务只改自己的组件内部。
- `dashboardPresentation.ts` 只加字段，旧消费者不受影响。

---

## 6. 验证口径

- 类型与静态检查：`just frontend-check-quick`。
- 完整前端门禁：`just frontend-check`；UI 全量：`just ui-check`。
- 视觉验证走 Web 预览（`npm run dev`）+ 浏览器工具，覆盖明/暗 × neutral/clay 四组合。
- 响应式三档：宽桌面、常规桌面、窄窗口，判定无横向滚动、无重叠、无截断。
- `prefers-reduced-motion` 下无非必要动画。
