# Profile 列表呈现层 — 执行计划

前置：`08-26-profile-registry-tokens` 已完成，`ProfileDisplayRecord`、`toVendorKey`、`ProfilePresentation` 与平台色四角色可用。

## 步骤

### 1. 前置确认

- [ ] 读 `features/profiles/stores.ts`，确认 pinned / recent 的 localStorage 读写与水合写法，`viewMode` 按同一形态扩展。
- [ ] 确认 Profile 页面现有的搜索热键绑定，据此定筛选栏的快捷键提示文案。
- [ ] `rg` 列出 `profiles-shared.css` 中仍被 `ProfileListRow`、`ProfilesHeader` 等在线组件使用的类名，写入 `notes.md`，重写时保留。
- [ ] 读 `ProfilesRawEditorPanel.tsx` 的 props 契约与 `raw-config-editor-contracts.md`，确认 source mode 流程的六项映射无遗漏。
- [ ] 读 `profiles-page-contracts.md` 的共享骨架一节，确认骨架顺序与 Off 横幅约束。

**审阅点**：在线类名清单与 source mode 映射表需先记录，再进入编码。

### 2. 状态层

- [ ] 新建 `features/platform/profiles/useProfilesSurface.ts`，接收记录数组，持有筛选与视图状态，派生四项统计。
- [ ] 新建 `features/platform/profiles/resolveRowState.ts`。
- [ ] 在 `features/profiles/stores.ts` 扩展 `viewByPlatform` + `setView`，沿用现有 `try/catch` 读写与前缀水合。
- [ ] `tests/profiles-view-mode.smoke.test.ts`：跨卸载保持、按平台隔离、`setItem` 抛错降级。

### 3. 页壳、Off 横幅与统计条

- [ ] 新建 `ProfilesPageHeader.tsx`：面包屑 + 页头 + 主次操作按钮组。`onEditSource` 为可选。
- [ ] 新建 `ProfilesOffBanner.tsx`：`canOff` 为假时返回 `null`，确认框 `type=warning`。
- [ ] 改造 `ProfilesStatStrip.tsx`：换为 `{ current, stats, labels }` 契约，四卡两种内容 slot 共用一个卡片外壳。
- [ ] i18n：新增文案同步进 `zh-CN` 与 `en-US`。

### 4. QuickRail、命令面板与筛选栏

- [ ] 适配 `ProfilesQuickRail.tsx` 的 props 到 `ProfileDisplayRecord`，热键与持久化语义不动。
- [ ] 适配 `ProfilesCommandPalette.tsx`，保留 `__off` 条目。
- [ ] 改造 `ProfilesToolbar.tsx`：按设计稿重排单行布局；Filters 弹层、provider 下拉、排序下拉、焦点陷阱逻辑全部保留。
- [ ] 跑 QuickRail 三份既有 smoke 测试确认无回归。

### 5. 列表视图与 Inspector

- [ ] 新建 `ProfileCardGrid.tsx`：三列网格，Inspector 展开时两列。
- [ ] 新建 `ProfileTable.tsx`：六列固定网格，容器 `min-width: 1024px` + `overflow-x: auto`。
- [ ] 两者均通过 `resolveRowState` 取状态。
- [ ] 适配 `ProfilesInspector*.tsx` 与 `ProfileDiffRows.tsx` 到 `ProfileDisplayRecord` + 注入的 descriptor。
- [ ] 新建 `ProfilesEmptyState.tsx`：两种 variant。

### 6. 页面装配与 source mode

- [ ] 新建 `features/platform/profiles/ProfilesSurface.tsx`，按骨架顺序装配。
- [ ] 实现 source mode：警告、token 保持、`conflict`、`activation_conflict` + `force`、`invalid` 透传、保存后顺序。
- [ ] 编辑器入口以 props 预留（`onAdd` / `onEdit` 透传）。
- [ ] 更新 `components/profiles/index.ts` 导出。
- [ ] `tests/profiles-raw-source.smoke.test.tsx`。

### 7. 样式

- [ ] 按新结构重写 `profiles-shared.css`，保留步骤 1 记录的在线类名。
- [ ] 颜色全部走 token，平台色用四角色，运行中高亮用全局 accent。

### 8. 规格同步

- [ ] 更新 `.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md`：StatStrip 特色槽合并为「认证方式」卡；骨架新增双视图与 source mode 入口。
- [ ] 不改动 Off 横幅位置、QuickSwitch 语义、Filters 维度、Inspector 存在性四项条款。

### 9. 验证

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profiles-surface.smoke.test.tsx tests/profiles-view-mode.smoke.test.ts tests/profiles-raw-source.smoke.test.tsx tests/profiles-quick-switch.smoke.test.ts tests/profiles-hotkeys.smoke.test.ts tests/profiles-quick-rail.smoke.test.ts tests/platform-surface-unify.smoke.test.ts
```

```bash
just frontend-check-quick
```

- [ ] 硬编码 hex 自查：`rg -n "#[0-9a-fA-F]{3,8}" ccr-ui/src/components/profiles ccr-ui/src/features/platform/profiles`，结果应为空。
- [ ] `rg -l "smoke.test" ccr-ui/src` 结果为空。
- [ ] 确认 `git diff --stat` 中不含 `features/{claude,codex,grok}/` 与路由文件。
- [ ] 确认未删除任何文件。

### 10. 走查

前置条件按父任务 `design.md`「视觉与响应式验收条件」：viewport `1440×900` 与 `900×800`，zoom 100%，`light|dark` × `neutral|clay` 四组合，夹具 `tests/fixtures/profiles.ts`。

- [ ] 新 `ProfilesSurface` 在 8 种（2 viewport × 4 主题组合）条件下各渲染一次，检查统计卡、运行中高亮、chip、表格边框层级、Inspector 展开态。
- [ ] `900×800` 下记录表格容器与 body 的 `scrollWidth` / `clientWidth` 实测值，写入 `notes.md`。
- [ ] **旧页面回归**：本任务重写了 `profiles-shared.css`，仍由 `BaseProfiles` 渲染的 `/claude-code/profiles` 与 `/codex/profiles` 在同一 8 种条件下各走查一次，确认无样式回归。结论逐项写入 `notes.md`（AC18）。

## 验收对照

完成后逐条勾选 `prd.md` 的 AC1–AC20。

## 与并行任务的约定

`08-26-profile-editor` 与本任务并行。本任务负责在 `profiles-shared.css` 中落地共享的 chip、按钮、输入框、label 等原子类；编辑器任务引用这些类，只在 `profile-editor-shell.css` 写模态专属样式。本任务完成这些原子类后即在 `notes.md` 记录类名清单，供编辑器任务对接。
