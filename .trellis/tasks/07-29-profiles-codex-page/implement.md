# Implement: Codex Profiles 页面落地

前置：`07-29-profiles-shared-layer` 已归档或其组件已可用。每步后跑 `cd ccr-ui && bun run test`，收尾跑 `just ui-check`。

## 检查清单

1. [ ] `utils/codexProfileEditor.ts`：新增 `resolveCodexBaseUrl()` / `codexAuthModeLabel()`；`buildCodexProfileRequest` 内聚派生字段（`requires_openai_auth` / `openai_login_method` 由 `auth_mode` 计算；`env_key` 仅 `provider_env_key` 模式序列化，其余置空）；删除 `syncDerivedAuthFields` 与表单内派生存储；补 `env_key` 模式切换回归测试。
2. [ ] `CodexProfileEditorModal.vue`：样式迁移共享编辑器基底（删 `--editor-*` / `!important` / 硬编码 RGBA / 暗色覆盖块）；校验汇总条。
3. [ ] `ProfileCard.vue`：`--cp-*` 迁移；字段策略对齐；操作区 = Apply / env-export 复制 / 编辑 / 删除菜单；引用统一 fallback/label 函数。
4. [ ] 视图接入新 QuickRail + `useProfilesQuickSwitch('codex')` + 稳定编号 hotkeys。
5. [ ] 视图接入四槽 StatStrip（特色槽 Config mode）。
6. [ ] 视图接入新 Toolbar（Filters popover = 标签 + 排序）；补 stale tag filter watch。
7. [ ] 视图接入 ProfilesInspector（previewName + hover/focus + Health locate + tag-select）。
8. [ ] Apply 确认框接 `ProfileDiffRows`（解析后 base_url）；Delete 加备份 footnote。
9. [ ] 补内联 loadError + retry 状态块（对齐 Claude）。
10. [ ] Header 收敛（Add / ⌘K / ··· 溢出）。
11. [ ] 死代码清理：`actionImport`、内联 ProfilesSection、`.cp-list-head` 重复、`extractErrorMessage` 换 `getErrorMessage`、硬编码回退。
12. [ ] i18n 对称更新（zh-CN / en-US / `.keys.txt`）。
13. [ ] 行为回归走查 + 按父 `design.md` §8.1 截图协议产出暗/亮 × 2543px/1280px 四张走查图，并与 Claude 页并排核对骨架同构。

## 验证

- `cd ccr-ui && bun run test`
- `just ui-check`
- 手动走查：apply（busy + diff 确认）/ env-export 复制 / 废弃 auth_mode 遗留 profile 编辑 / Ctrl+数字键 / hover 预览

## 回滚点

- 步骤 1 的派生字段单源化独立可回滚（行为等价重构）；步骤 4–7 视图接入按组件逐个可回滚。
