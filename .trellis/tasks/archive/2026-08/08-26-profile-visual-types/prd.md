# 全局按钮 / 标签 / URL 视觉类型

## Goal

让 CCR UI 操作页上的**动作、字段标签、URL、枚举值**共用一套可扫描的视觉类型：主次与风险按钮一眼可分，Base URL 与 `api_key` 这类值不再长成同一段 muted 小字。Profile 列表是旗舰验收面。其它操作页的迁移范围以 `08-26-visual-type-rollout` 的逐文件清单为准（含全部 `ui-classes` 按钮消费方、五个 platform Base，以及清单内的 configs/checkin/sync/usage/claude 站点），而不是「所有页面上任意长得像按钮的节点」。

## Background

- 标注截图：`research/claude-profiles-annotation.png`。红框：页头 `Reload` / `+ New Profile`、Off 横幅 `Turn profile off`、卡片 `BASE URL`、`AUTH`、`PROVIDER`。
- 视觉规格：`research/visual-language.md`。
- React 迁移在 `08-22-design-system` 判定表里要把 Vue `Button` / `Badge` 落到 `src/ui/button.tsx` 与 `src/ui/badge.tsx`。`src/ui/index.ts` 至今没有这两项。`layering-contracts.md` 仍以 `src/ui/Button.tsx` 为合法原语示例。
- 结果：Profiles 用 `cp-btn` / `cp-chip`，Codex / OpenCode / Grok 各有一份几乎相同的 `ui-classes.ts`，Configs / Checkin / Sync / Usage / 平台 Base 再各写 primary class。ghost 描边和 primary 边框权重接近，Off 动作没有 warning 语义。
- 三平台 Profile 列表已由 `08-26-profile-design-language` 接到共享呈现层。类型化显示做在共享层，Claude / Codex / Grok 一起变。

## Decisions

1. **全局抽取（有界）**：原语落在 `src/ui/`（`ui-primitive` 层），样式进 `src/ui/primitives.css`。不引入 `class-variance-authority` / shadcn Slot。操作页迁移不是全仓扫尾，清单由 rollout 子任务逐文件给出；遗漏的同类 platform Base（Commands/Plugins）必须进清单，不得靠「所有消费方」概括。
2. **不复活** Vue Button 的 `glass` / `surface` / `elevation` / `motion` / `density` 轴。
3. **变体封闭集**：`primary` / `secondary` / `ghost` / `quiet` / `warning` / `danger` / `accent-soft`；尺寸 `sm` | `md`。
4. **URL 只展示**：`UrlText` 使用 `formatBaseUrlDisplay`，`title` 为原文。不做外链、不点击复制。
5. **AUTH 与 PROVIDER**（及同类枚举）用 `Badge` static；MODEL 空值 `-` 保持纯文本。
6. **不加 token 名**。颜色只用已有 `--color-accent-*`、`--color-warning-tint`、`--color-text-*`、`--color-border-*`。
7. **任务树**：父任务只做规格与集成门禁。实施顺序：原语 → Profile 旗舰面 → 操作页迁移。

## Task map

| 子任务 | 交付 | 依赖 |
| --- | --- | --- |
| `08-26-visual-type-primitives` | `Button` / `buttonClass` / `Badge` / `FieldLabel` / `UrlText`；`EmptyState` 与 `ConfirmModal` 改用 `Button` | 无 |
| `08-26-visual-type-profiles` | Profile 页头、Off 横幅、卡片、表格、空态、编辑器脚采用原语；字段槽增加 `kind` | primitives |
| `08-26-visual-type-rollout` | 按逐文件清单迁移动作按钮；删除三份 `primaryBtnClass` 等导出 | primitives；可与 profiles 并行 |

## Requirements

- R1：`src/ui` 提供 Button、Badge、FieldLabel、UrlText，导出组件与 `buttonClass()`。原语不导入 `features/` / `api/` / store。
- R2：变体、尺寸、状态、减动效与排版以 `research/visual-language.md` 为准。Button 七变体的背景/边框/字色、同尺寸同高、focus/active/disabled、FieldLabel 三项、新 `.ui-*` 规则无 hex/px，均需 CSS 或 computed-style 契约，不能只断言 class 名。
- R3：Profile 列表（Claude / Codex / Grok）页头主次按钮、Off 横幅按钮 **与容器** warning 表面、卡片/表格字段类型化显示符合该规格；`fieldSlots` 用 `kind: 'text' | 'url' | 'chip'` 取代仅有的 `chip?: boolean`。行状态徽章与 `record.badges` 均为 `Badge mode="static"`（运行中状态徽章 `tone=accent`；`record.badges` 的 tone 原样映射）。
- R4：rollout 逐文件清单内的操作页动作按钮改用原语或 `buttonClass()`，不再各写一套 primary/ghost/danger class。清单必须包含 `BaseCommands` / `BasePlugins`。旧 `bg-accent-secondary` 主确认不得按「不存在的 `bg-accent-primary px-4 py-2`」推断，映射见 rollout `design.md`。
- R5：不改路由、Tauri、凭据处理、Profiles 骨架顺序、表格列数。
- R6：契约测试在 `ccr-ui/tests/ui/` 与 `ccr-ui/tests/profiles/`。`rg -l "smoke.test" ccr-ui/src` 为空。
- R7：`just frontend-check-quick` 在每个子任务通过；父任务集成跑 `just ui-check`。
- R8：Profile 旗舰面在 1440×900 与 900×800、`light|dark` × `neutral|clay` 共 8 组合下对照标注截图与 `visual-language.md` 走查。每个组合必须对必查项给出 PASS/FAIL；**8 行全部 PASS** 才算本条成立。失败项写入 `notes.md` 并附截图路径。只存在记录文件不算通过。

## Acceptance Criteria

跨子任务，在 rollout 完成后整体成立。

- [ ] AC1（R1）：`@/ui` 导出 `Button`、`buttonClass`、`Badge`、`FieldLabel`、`UrlText`；`tests/quality/` 或 `tests/ui/` 的分层断言覆盖这些文件不得导入 `features` / `api` / store。
- [ ] AC2（R2）：Button 七变体的 token 背景/边框与同 `size` 等高、focus/active/disabled 契约由 primitives 子任务 AC2 锁定；FieldLabel 三项由 primitives AC10 锁定；`prefers-reduced-motion: reduce` 下无 `transform` 过渡。
- [ ] AC3（R2）：`Badge` static 的计算 `cursor` 不是 `pointer`；interactive 才是。
- [ ] AC4（R3）：Claude 卡片 Base URL 走 `UrlText`（展示值来自 `formatBaseUrlDisplay`）；AUTH 与 PROVIDER 走 static `Badge`；MODEL 空值不是 Badge。行状态徽章与 `record.badges` 为 static Badge；运行中状态徽章 `tone=accent`；这些节点 computed `cursor` 不是 `pointer`。
- [ ] AC5（R3）：页头 `新建` 为 `primary`，Reload / Export / Edit source 为 `ghost`；Off 横幅动作为 `warning`，容器背景为 `--color-warning-tint`、边框为 `--color-warning`。确认框仍为 `type=warning`。
- [ ] AC6（R3）：Codex / Grok 列表同一套组件，差异只来自 `fieldSlots` 入参。
- [ ] AC7（R4）：`features/codex/ui-classes.ts`、`features/opencode/ui-classes.ts`、`features/grok/ui-classes.ts` 不再定义 `primaryBtnClass` / `ghostBtnClass` / `secondaryBtnClass` / `dangerBtnClass`。
- [ ] AC8（R4）：rollout 逐文件清单中的动作按该子任务映射表改为 `Button` 或 `buttonClass`（含 `BaseCommands` / `BasePlugins` 的 `bg-accent-primary` 主按钮，以及 `AgentEditModal` 保存=`primary`、Add tool=`secondary`，`McpPresetsPanel` 确认安装=`primary`）。不得把未列入映射表的 `bg-accent-secondary` 一律改成 primary。
- [ ] AC9（R5）：表格仍为六列，不出现 Claude PROVIDER 第四数据列；骨架顺序不变。
- [ ] AC10（R6、R7）：`just frontend-check-quick` 与 `just ui-check` 通过；无新增 token 名（`theme-token-contracts.md` 冻结段 unique-name 数不变）。
- [ ] AC11（R8）：profiles 子任务 `notes.md` 含 8 行走查表，每行必查项均为 PASS；任一 FAIL 则本条不成立。
- [ ] AC12（R2）：Profile 相关与新 `.ui-*` 规则无硬编码 hex，且新 `.ui-*` 规则无 `px` 字面量。

## Out of scope

- 窗口标题栏、托盘、分页、`PillToggleGroup`、Configs `FilterChip`、Codex 账号卡 icon-only `ActionButton`
- `SyncInfoSidebar.tsx`、`WslManagementView.tsx`、`SshManagementView.tsx` 上未列入 rollout 清单的一次性 accent 按钮
- Base URL 外链或点击复制
- 点 AUTH / PROVIDER / tag 触发筛选
- 新增 CSS token 名；若对比度不够，停下来另开 token-governance，不在本树里加名
- 输入框 / 卡片容器原语（`fieldInputClass` / `panelCardClass` 可留在各域）

## Notes

- 子任务不得在实施中改写本文件的 AC。做不到就回到规划。
- 测试目录按领域：`tests/ui/`、`tests/profiles/`。不要写回 `tests/` 根。
