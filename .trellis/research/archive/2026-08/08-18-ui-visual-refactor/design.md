# 设计方向：Editorial Control Room · 收敛实施版

## 0. 权威链

`ccr-ui/DESIGN.md` 定义视觉世界（Editorial Control Room）。本文件给出可执行配方，并列出 **本任务必须写入 DESIGN.md 的修订**。

Wave 0 与 token 改动同一批次修订 DESIGN.md。修订完成后，冲突以修订后的 DESIGN.md 为准。修订完成前，下表条款以本文件为准，避免实施时两套数字互斥。

| 条款 | 现行 DESIGN.md | 本任务写入 DESIGN.md 的值 |
|---|---|---|
| Headline / PageTitle | 560 / 2rem / tracking `-0.028em` | 600 / 1.5rem / `-0.01em` / lh 1.2 |
| Title / SectionTitle | 560 / 1.3125rem / `-0.016em` | 600 / 1.0625rem / lh 1.3 |
| Body tracking | `-0.016em` | `0` |
| Label | 560 / tracking `0.018em` | 500 / tracking `0` / lh 1.24 |
| 主按钮填充 | 「Clay gradient over the accent range」 | 实心 `--color-accent-primary`，禁止渐变 |
| 主按钮形状 | pill + 44px | 保留 pill + 44px |
| 次按钮 | pill | 8–10px 圆角，非 pill |
| 输入框圆角 | 16px（`rounded.xl`） | 8–10px |
| 静止卡投影 | 可用 `--shadow-md` | 无，或仅 `--shadow-sm` |
| Semantic Glow | 命名词汇，用于状态反馈 | 仅 focus ring；删除装饰性 glow |

世界级原则不改：Accent Scarcity、Product Type Rule、No Ghost Card、禁 side-stripe、禁嵌套卡、禁紫蓝渐变。

## 1. 表面配方

**现状**：1px 描边 + 大投影 + `inset 0 1px 0 white/38%` 同时打在每张卡上。

| 层级 | 背景 | 描边 | 投影 | 用途 |
|---|---|---|---|---|
| base | `--color-bg-base` | — | — | 应用底色 |
| workspace | `--color-bg-elevated` | — | — | 内容区 |
| card（静止） | `--color-bg-surface` | 1px `--color-border-subtle` | 无 | 内容卡 |
| card（hover） | 同上 | `--color-border-strong` | 无或 `--shadow-sm` | 可点击卡 |
| overlay/sticky | `--color-bg-overlay` | 1px subtle | `--shadow-sm` | 吸附工具条 |
| modal | `--surface-modal-*` | 1px | `--shadow-lg` | 模态（全屏 ≤1 个 blur） |

- 禁止任何 inset 白色高光（`--inner-glow` / `--glass-inner-glow` 连同消费点删除）。
- 深度用色调分层：base < elevated < surface < overlay。暗色 elevation 单调提亮，由 contrast smoke 守护。
- 卡内分组用留白 + 细分隔线，不套第二张卡。

## 2. 颜色与 accent

- 渲染色来自 `tokens.css` 语义 token。`.vue` 字面 hex/rgb 只留 `AppSettingsView` 色板预览。phantom-token 视同硬编码。
- 实心 accent 只允许：(a) 每屏 ≤1 个主按钮；(b) 小型激活标记（chip 选中点、focus ring）。列表首行、导航、图表柱、统计数字用 tonal（accent 8–12%）或中性色。
- success / warning / danger / info 只表达状态。
- 主题域：`neutral | clay` × `light | dark` × accent `clay`。`data-accent` 仍写入，值恒为 `clay`。
- 平台识别色只用小标记，走 `--color-platform-*`，不粉刷区域。
- 迁移表以 `prd.md` R1 为准。

## 3. 排版

| 档位 | 规格 | 用途 |
|---|---|---|
| PageTitle | 1.5rem / 600 / tracking -0.01em / lh 1.2 | 页头标题（固定 rem） |
| SectionTitle | 1.0625rem / 600 / lh 1.3 | 卡片 / 分区标题 |
| Body | 1rem / 400 / tracking 0 / lh 1.56 | 正文、描述 |
| Label | 0.8125rem / 500 / lh 1.24 | 表单与控件标签 |
| Eyebrow | 0.75rem / 600 / tracking 0.08em / uppercase | 仅拉丁短标签 |
| Data | tabular-nums，字重 500–600 | 统计数字；尺寸 ≤2 档 |

- `--tracking-normal` = 0。字重阶梯 400 / 500 / 600 / 700。
- eyebrow 单一来源。`:lang(zh)`、`:lang(zh-CN)` 关闭 tracking 与 uppercase。拉丁短标签在中文界面上设 `lang="en"`。
- 正文、按钮、表格头不大写。表格头用 Label 档。

## 4. 形状

- 卡 12px；面板 12–16px；控件 8–10px。
- pill（9999px）仅 toggle、chip、主按钮。
- Badge 默认 6–8px 圆角矩形。pill badge 只用于状态点 + 短词。

## 5. 共享原语

- `PageHeader`：eyebrow? + title + description? + 右侧状态 / 操作槽。替换 `PageHeaderCard` 与手抄 eyebrow。
- `StatTile`：label + value + hint?。无嵌套卡、无装饰着色。取代手造统计瓦片与 `ui/StatCard`。
- `PillToggleGroup`：单选切换。激活态为 tonal accent。
- `SectionLabel`：分区小标（非 uppercase）。
- `PageShell`：页头 + subnav + 内容栅格。先按 §1 改 `OpenCodePageShell`，再抽出通用壳。禁止把该文件现有字面 `rgb()` 复制进新壳。

适用规则见 `views-inventory.md`。没有统计的页不接 StatTile；没有单选切换的页不接 PillToggleGroup。

## 6. 结构重排

- **Dashboard**：页头一行（标题 + 状态）；行动队列为紧凑列表，首行 tonal；Readiness 改为数据行；Usage / Signals 双栏统一卡高；PlatformMatrix 用平台色小标记，不用 accent 条。
- **平台工作台**：主页 = 状态摘要 + 子模块入口卡阵；子页 = PageShell + ModuleSubnav。
- **设置页**：分区改为分组列表 + 锚点导航；flavor 双卡；无 accent 选择区。
- 信息层级：状态 > 下一步动作 > 数据洞察 > 入口。重排页在对应 Wave 记录前后结构。

## 7. 动效

- 100–200ms，只表达状态变化。删除 pulse-glow、tag-glow。hover 位移 ≤1px。
- `prefers-reduced-motion` 与 `prefers-reduced-transparency` 重置保持完整。mocha 作用域内的重置迁到剩余 flavor 选择器，不能随 mocha 块一起删掉后无人承担。

## 8. 实现层禁令

- 禁 inset 高光、装饰性 glow / text-shadow、`.text-gradient-*`。
- 禁 `.vue` 字面颜色与 phantom-token。
- 禁新增 `linear-gradient`（白名单见 `prd.md` AC）。
- 禁 vw 流体排版进 app 表面；禁对 CJK 加字距 / 大写。
- 禁 side-stripe；禁语义色装饰数字。
- 禁新增 flavor / accent；禁绕过 surface 语义别名直读 `--material-glass-*`。
- 禁把 OpenCode TUI 主题字符串当作 CCR flavor 删除或改写。

## 9. 边界与数据流

- 只改 `ccr-ui` 前端表面：`src/styles/*`、`src/components/**`、`src/views/**`、`themeBootstrap.ts`、`index.html` IIFE、`shellPreferences.ts`、相关 smoke、`DESIGN.md`、`theme-token-contracts.md`、主题相关 i18n。
- 不改：路由表结构、Tauri command、`dashboardPresentation.ts` 的 signal / readiness 语义、profiles 序列化、checkin 队列、ApexCharts 完整 CSS 加载路径、`requestConfirm` 门闩。
- 样式消费路径：语义 token → surface 别名 → 组件 class。页面不得再声明平行色板。
- 主题读取路径：localStorage `ccr-theme` / `ccr-flavor` / `ccr-accent` → 迁移函数 → `data-*` 属性 → CSS 选择器。IIFE 与 TS 走同一张表。

## 10. 兼容与迁移

- 迁移在读取时发生。`migratePersisted*` 仅在「存贮值 ≠ 迁移结果」时写回。键不存在时不写默认值。
- 写回后，旧版应用读到的是它已认识的 `neutral` / `clay`。配置不丢。用户若曾选 catppuccin / sage / sky，回滚到旧二进制后不会自动回到旧选项——偏好已写回新值。PRD 接受这一结果。
- `data-flavor` 仍独立于 `data-theme`。`data-accent="clay"` 始终存在，满足三层不合层。
- contrast smoke 的有效组合从 6 组（含 latte / mocha）减为 4 组（light/dark × neutral/clay）。阈值常量不变。
- `apple-glass-surface-contract.smoke.test.ts` 中 mocha 覆盖块「必须存在」的断言改为「不得存在」；`styleLockedPaths` 在改 Checkin 族时同步改期望，锁的是「禁止回退旧调色板」，不是「禁止改视觉」。

## 11. 取舍

| 选项 | 选择 | 代价 |
|---|---|---|
| 修订 DESIGN.md vs 撤回配方 | 修订 DESIGN.md | 设计文档与 07-28 时期的 560 字重 / 渐变主按钮不再一致 |
| 拆 6 个子任务 vs 单任务 | 单任务 + 发版冻结 | 一个 PR 很大；换来配方不在多个目录漂移 |
| 强制拆 >300 行 style vs 只清视觉 | 只清视觉 | Checkin / Grok 仍长；结构债另开任务 |
| 删 Sparkline / ConfigCard vs 留用 | 留用并改样式 | Wave 1 不能「清僵尸」这两件 |
| accent 层删除 vs 留属性 | 留 `data-accent=clay` | 选择器多一个恒真条件；遵守不合层契约 |

## 12. 回滚

| 点 | 动作 | 风险 |
|---|---|---|
| Wave 0 提交 | `git revert` 该提交。用户若已被写回 `neutral`/`clay`，回滚代码后不会自动恢复 catppuccin | 中段全站更难看，因此禁止把 Wave 0 单独合入 `main` |
| Wave 1–2 | 还原 `components/ui`、`MainLayout`、`AppSettingsView` | 已改过的页面会暂时对不齐新原语 |
| Wave 3+ 单页 | 按 `views-inventory.md` 单页还原 | 不影响 token 域 |
| 发版 | Wave 7 通过前，功能分支不合并 `main`，不打 release tag | 见 `implement.md` 冻结规则 |

回滚不修改用户 localStorage 之外的磁盘配置。不写回空 key。
