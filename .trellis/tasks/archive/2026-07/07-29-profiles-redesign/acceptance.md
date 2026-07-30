# Acceptance: Profiles 页面重构

**Date**: 2026-07-30
**Scope**: `07-29-profiles-redesign` 父任务跨子任务集成验收

## 验收标准 1–11

| # | 结论 | 证据 |
| --- | --- | --- |
| 1 | PASS | `ClaudeCodeProfilesView.vue` 与 `CodexProfilesView.vue` 同序消费 `ProfilesHeader` → `ProfilesStatStrip` → `ProfilesQuickRail` → `ProfilesToolbar` → `ProfilesSection` → `ProfilesInspector`；页面公共布局只保留在 `src/styles/profiles-page.css`。`claude-profiles-view.smoke.test.ts` / `codex-profiles-view.smoke.test.ts` 覆盖四槽统计、QuickRail、Inspector 和主列表。 |
| 2 | PASS | `useProfilesQuickSwitch.ts` 以 `pinned.slice(0, 8)` 作为 `stableTargets`，recent 不编号；`useProfilesHotkeys.ts` 必须注入 `getStableTargets`，已删除显示顺序回退。`profiles-quick-switch.smoke.test.ts` / `profiles-hotkeys.smoke.test.ts` 覆盖持久化、稳定编号、上限和超界数字键。 |
| 3 | PASS | 两页 Apply 调用 `buildProfileDiff` 生成 base_url/model/auth_mode 三行差异，确认框由 `ProfileDiffRows.vue` 渲染；删除脚注的 zh/en 文案均指向 `~/.ccr/backups/{platform}/`，并明示无 UI 恢复入口。`profile-diff.smoke.test.ts` 与两个 view smoke 覆盖该流程。 |
| 4 | PASS | 完整字段详情集中在 `ProfilesInspector.vue`；两页以 `hoveredName ?? focusedName ?? current` 解析预览目标，删除/重命名后清理预览状态。当前态轻量标记仅由 StatStrip、QuickRail 和列表行消费。 |
| 5 | PASS | `ProfilesToolbar.vue` 常态只裸露搜索、状态 pill 组和 Filters 按钮，有效筛选数以 badge 显示；标签/provider/排序已全部进入 popover。 |
| 6 | PASS | 视图、行/卡片、Inspector 统一消费 `--cp-*`，编辑器消费共享 `profile-editor-shell.css`；旧 `--editor-*` / `--palette-*` 平行体系已清除。最终扫描确认本次新增/修改样式无 px 字面 `font-size`，`0.75rem` 密排元数据字阶已登记到 spec。 |
| 7 | PASS | `useProfilesQuickSwitch.ts` 复用 `getClientPlatform()`：Windows/Linux 返回 `Ctrl`，macOS 返回 `⌘`；双语提示使用 `{modifier}` 插值，旧的 `+ number key` / `+ 数字键` 键已删除。 |
| 8 | PASS | `bun run type-check`、`bun run lint`、`bun run test` 与根目录 `just ui-check` 通过；最终全量为 i18n 23/23、Vitest 112 files / 547 tests，Tauri `cargo check` 通过。lint 为 0 error / 3 条已有 raw-text warning。 |
| 9 | PASS | zh-CN / en-US 的 Profiles 键与两份 `.keys.txt` 同步删除旧 `contextRail`、Last Write 和模糊快捷键键；`bun run test:i18n` 23/23 通过，本次触及组件无新增 `translateWithFallback` 硬编码回退。 |
| 10 | PASS (manual) | 2026-07-30 用户在真实 Tauri 运行态人工确认 Claude/Codex 两页、暗/亮主题与两档视口均无问题。本结论不使用代理生成截图作为验收证据。 |
| 11 | PASS | `profiles-quick-switch.smoke.test.ts` 覆盖 localStorage、stale 清理、首个成功快照前不误清数据、rename/delete/disable 边界与平台修饰键；`profiles-quick-rail.smoke.test.ts` 覆盖 roving tabindex；`codex-profile-editor.smoke.test.ts` 覆盖 `env_key` 仅在 `provider_env_key` 模式序列化。 |

## 集成清理与复评

- 删除 `ProfilesContextRail.vue`、`codex.profiles.contextRail` zh/en 子树与 `.keys.txt` 键。
- 删除 ProfilesHeader / ProfilesToolbar / ProfilesQuickRail / ProfilesStatStrip 的兼容分支、旧 props 和 `TODO(profiles-redesign)`。
- `.cp-state*` / `.cp-grid` / `.cp-list-head` 只保留在 `src/styles/profiles-page.css`，两个 view 不再各自维护重复样式。
- Impeccable 复评快照：`ccr-ui/.impeccable/critique/2026-07-30T01-49-38Z__ccr-ui-src-views-claudecodeprofilesview-vue.md`，34/40，P0=0，P1=0。快照保留 `⚠️ DEGRADED: single-context (project AGENTS requires sequential execution)` 声明。
- detector 最终报告 73 条，全部为 design-system font-size/radius advisory（56 + 17）；无 layout-transition 或功能级规则。由于 detector 对这些记录没有另设 `advisory: true`，进程退出码仍为 2，不表述为 clean run。

## 最终验证

```text
bunx vitest run ...profiles...     8 files / 53 tests passed
bun run type-check                 passed
bun run lint                       passed (0 errors, 3 existing warnings)
bun run test                       i18n 23/23; 112 files / 547 tests passed
just ui-check                      passed, including Tauri cargo check
git diff --check                   passed (CRLF notices only)
```
