# Implement: Claude Code Profiles 页面落地

前置：`07-29-profiles-shared-layer` 已归档或其组件已可用。每步后跑 `cd ccr-ui && bun run test`，收尾跑 `just ui-check`。

## 检查清单

1. [x] `utils/claudeProfiles.ts`：新增 `resolveClaudePrimaryModel()` / `formatClaudeBaseUrlDisplay()` 与 row/inspector/diff descriptor 组装函数；删除 4 份 fallback 重复。
2. [x] 新建 `components/claude/ClaudeProfileEditorModal.vue`：迁移视图内联编辑器模板/逻辑/段导航 scroll-spy；样式接共享编辑器基底（`pe-*`）；保存前校验汇总条 + 跳转第一个错误分段。
3. [x] 视图接入编辑器模态（替换内联 BaseModal 块），add/edit/rename 流程走通。
4. [x] 重构 `ClaudeProfileRow.vue`：`--cp-*` 迁移、字段策略（完整 host / fallback model）、编辑/删除收进角落 `···` 菜单、保留搜索高亮与 busyAction。
5. [x] 视图接入新 QuickRail + `useProfilesQuickSwitch('claude')` + 稳定编号 hotkeys（`getStableTargets`）+ apply 成功 `recordUse` + rename 成功 `renamePinned`。
6. [x] 视图接入四槽 StatStrip（特色槽 Auth split，第四槽换 Health 并可点击定位）。
7. [x] 视图接入新 Toolbar（`compactFilters`，provider 筛选入 Filters popover）。
8. [x] 视图接入 ProfilesInspector：hovered/focused 双状态预览 + Health `@locate` 滚动高亮 + tag-select 写筛选 + 会话写入时间。
9. [x] Apply 确认框接 `ProfileDiffRows`；Delete 确认框加备份 footnote。
10. [x] Header 收敛（Add / ⌘K / ··· 溢出：Reload/Export/Edit TOML），⌘/Ctrl 跟随 `getClientPlatform()`。
11. [x] 死代码清理：53 个未引用 i18n 键（含整个 `claudeProfiles.contextRail` 子树与 `statStrip.lastWrite*`）、内联 `ProfilesSection` 定义、视图内 `.cp-section*` 重复样式、`--editor-*` 令牌体系、`translateWithFallback` 硬编码中文回退、ProviderTemplateSelector 裸英文标签。
12. [x] i18n 键调整对称入 zh-CN / en-US（全量 3778 键双向一致），同步 `.keys.txt` 快照。
13. [ ] 行为回归走查 + 按父 `design.md` §8.1 截图协议产出暗/亮 × 2543px/1280px 四张走查图。

> 13 未完成：需要 Tauri 桌面运行态 + ≥20 条真实 profile fixture，当前会话没有该运行环境（浏览器预览工具在本次会话不可用，web preview 也拿不到 Tauri IPC 数据）。已用类型检查、ESLint、533 条 smoke 用例（含 10 条本页 DOM 级断言）与生产构建替代覆盖编译与行为面。

## 视图瘦身结果

- `ClaudeCodeProfilesView.vue` 1971 → 1108 行；模板 + 逻辑 933 行（目标 ≤900，超 33 行）。
- 表单装配下沉到新增 `utils/claudeProfileEditor.ts`（与 `utils/codexProfileEditor.ts` 对称）。
- 启用/停用两个分组合并为 `listSections` 单套行渲染，消除模板重复。

## 验证

- `cd ccr-ui && bun run type-check` ✅
- `cd ccr-ui && bun run lint` ✅（0 error；72 warning 全部落在共享层既有文件）
- `cd ccr-ui && bun run test` ✅ 110 files / 533 tests
- `cd ccr-ui && bun run build` ✅
- `just ui-check-frontend` ✅
- 手动走查：apply（含 diff 确认）/ delete（含备份提示）/ Ctrl+数字键稳定编号 / hover 预览 / Filters 徽标 / raw TOML 编辑 —— 待 13 一并完成

## 回滚点

- 步骤 2–3 编辑器抽取独立可回滚；步骤 5–8 若共享组件有缺陷，回滚视图接入到旧组件 props（旧分支仍带 `TODO(profiles-redesign)` 保留）。
