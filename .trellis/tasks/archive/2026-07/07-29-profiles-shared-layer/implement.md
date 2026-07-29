# Implement: Profiles 共享组件层重构

执行顺序即依赖顺序。每步完成后跑 `cd ccr-ui && bun run test`（快速反馈），全部完成后跑 `just ui-check`。

## 前置阅读

- `.trellis/spec/ccr-ui/frontend/index.md` + `theme-token-contracts.md` + `confirm-interaction-contracts.md`
- 父任务 `prd.md` / `design.md` / `research/current-state-analysis.md`
- 现有源码：`components/profiles/*.vue`、`composables/useProfiles*.ts`

## 检查清单

1. [ ] 新建 `composables/useProfilesQuickSwitch.ts`（钉选数组编号唯一来源 / recent 不编号 / pin 上限 8 / stale 清理与 rename 跟随 / localStorage / `getClientPlatform()` 修饰键），配套 Vitest 用例（持久化、清理、上限、修饰键提示）。
2. [ ] 改 `useProfilesHotkeys.ts`：数字键目标来源可注入 `getStableTargets`，未注入保持旧行为。
3. [ ] 新建 `utils/profileDiff.ts`（`buildProfileDiff` 平台无关壳）+ 单测。
4. [ ] 新建 `components/profiles/ProfileDiffRows.vue`。
5. [ ] 扩展确认流（`ConfirmModal` / `useConfirmAction`）：可选 diff slot + `footnote`，默认旧行为。
6. [ ] `ProfilesQuickRail.vue` 新增可选 `quickSwitch` 模式（钉选编号 chip + recent 无编号 + pin 操作 + more 入口 + `role="toolbar"` roving tabindex + 修饰键提示）；roving tabindex 单测。
7. [ ] 新建 `components/profiles/ProfilesInspector.vue`（预览面板 + diff + Health `@locate` + Distribution 折叠 + tag cloud 可点击）；旧 `ProfilesContextRail.vue` 不动。
8. [ ] `ProfilesToolbar.vue` 新增可选 `compactFilters` 模式（Filters popover + 生效数徽标 + 清除全部 + 弹层行为契约：Esc/外部点击/焦点返回/键盘导航/窄窗口）。
9. [ ] `ProfilesStatStrip.vue` 新增可选四槽 props；旧 Last Write/sparkline 标 `TODO(profiles-redesign): 集成步骤删除`。
10. [ ] 提取 `components/profiles/ProfilesSection.vue`（视图接入在子任务 ②③）。
11. [ ] `ProfilesCommandPalette.vue` token 别名归并（`--palette-*` → `--cp-*`/全局，值不变）。
12. [ ] 新建 `components/profiles/profile-editor-shell.css` 共享编辑器样式基底。
13. [ ] `ProfilesHeader.vue` 新增可选 `actionsMenu` 模式（Add / ⌘K / ··· 溢出，溢出菜单遵守弹层行为契约）。
14. [ ] i18n：新增键对称入 `zh-CN.ts` / `en-US.ts`；本任务触及面内无硬编码回退。
15. [ ] **纯新增证明**：平台页零改动跑 `bun run build` / typecheck / `bun run test` 全绿；逐组件核对默认渲染无变化。

## 验证

- `cd ccr-ui && bun run test`
- `cd ccr-ui && bun run typecheck`（若 script 名不同以 package.json 为准）
- `just ui-check`（最终门禁）

## 回滚点

- 步骤 1–5 为纯新增，随时可回滚。
- 步骤 6–13 为既有组件加可选模式，默认路径不变，异常时回滚单文件即可。
