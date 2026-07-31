# PRD: Codex Profiles 页面落地

父任务：`.trellis/tasks/07-29-profiles-redesign/`。依赖：子任务 `07-29-profiles-shared-layer` 完成。

## 范围

`ccr-ui/src/views/CodexProfilesView.vue`（1142 行）与 `components/codex/ProfileCard.vue`、`components/codex/CodexProfileEditorModal.vue`（1021 行）接入重构后的共享组件层，对齐 Claude 页骨架，清理 Codex 特有技术债。

## 需求

1. **接入新骨架**：Header 收敛、四槽 StatStrip（特色槽 = Config mode）、瘦身 QuickRail（平台键 `codex`）、Filters popover 工具栏、ProfilesInspector 右栏。
2. **ProfileCard 视觉统一**：迁移 `--cp-*`，与 Claude 卡片同语言；保留 Codex 独有的 env-export 复制按钮与 auth_source 显示（以徽章/菜单形式融入新卡片规范）；字段策略与 Claude 一致。
3. **CodexProfileEditorModal 样式迁移**：删除 ~250 行非 scoped `--editor-*` 样式、硬编码 light RGBA 与 `!important`、独立暗色覆盖块，接共享编辑器基底；新增保存前校验汇总条。
4. **表单双源真相清理**：`requires_openai_auth` / `openai_login_method` 不再存表单 state，保存时由 `auth_mode` 派生（逻辑并入 `buildCodexProfileRequest`，见 `utils/codexProfileEditor.ts`）。
5. **共享逻辑去重**：base_url 空→`officialBaseUrl` fallback 3 份重复与 `authModes` 标签查表 ~4 处各保留一份（`utils/codexProfileEditor.ts`）；提取 `extractErrorMessage` 换用共享 `getErrorMessage`。
6. **筛选对齐**：补上 stale tag filter watch（与 Claude 一致）；Filters popover 内仅标签 + 排序（Codex 无 provider 筛选）。
7. **Apply/Delete 确认升级**：diff 行 = base_url（含 official fallback 解析后值）/ model / auth_mode；delete 加备份 footnote。
8. **错误面对齐**：保留 toast，同时补内联 loadError + retry（与 Claude 一致）。
9. **死代码清理**：未接线的 `commandPalette.actionImport`、内联 `ProfilesSection`、`.cp-list-head` 重复、触及面内 `translateWithFallback` 硬编码中文回退。

## 验收标准

- 父任务跨子任务验收标准 1–11 中适用于本页的项全部满足（含 `env_key` 序列化回归测试）。
- `cd ccr-ui && bun run test` 通过；`just ui-check` 通过。
- 行为回归：apply（含 busy spinner + diff 确认）/ delete / env-export 复制 / raw TOML / 模板填充 / 废弃 auth_mode 遗留 profile 编辑路径。
- 与 Claude 页并排截图走查：骨架同构，仅特色槽与字段集不同。
