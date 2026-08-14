# 执行计划：Grok Profile 编辑器滚动与排版

## 前置

- [x] 任务已创建，决策已写入 PRD：对齐 Claude/Codex 编辑器壳
- [x] 实现前读 `design.md`、`profiles-page-contracts.md` 场景三、`ClaudeProfileEditorModal.vue` / `CodexProfileEditorModal.vue` 外壳（限高、`pe-nav`、`pe-scroll`、壳内 `pe-footer`）
- [x] 走 `trellis-before-dev`；视觉验证走 `ccr-ui-visual-workflow`（web preview，不默认 Tauri）

## 步骤

1. [x] i18n：zh-CN / en-US 同步补 `grok.profiles.editor.status`、`statusHint`、`validationJump`。导航标题复用已有 `identity` / `connection` / `runtime`。
2. [x] 重排 `GrokProfileEditorModal` 外壳：
   - `content-class` 改为 `pe-modal`（可保留 `grok-profile-editor` 作用域钩子）
   - `#header` 改为 `pe-modal__head`（eyebrow、标题、enabled pill、关闭）；`show-close=false`
   - 默认槽内：`pe-shell max-h-[calc(90vh-9rem)] overflow-hidden`
   - `pe-summary` + 跳转、`pe-nav`、`ref="scrollRef"` 的 `pe-scroll`、四段 `id`
   - 页脚改到壳内 `pe-footer`，去掉 BaseModal `#footer`
   - 官方不渲染 connection 段与对应导航
3. [x] 接入 scroll-spy 与 `scrollToSection`（对齐 Claude：IntersectionObserver，`rootMargin: '-140px 0px -70% 0px'`）。kind 切换与打开时 `nextTick` 后重绑。校验错误带 section，保存失败跳首个错误段。
4. [x] 排版：status 段补标题；字段继续 `pe-field` / `pe-panel` / `pe-panel-muted`；双列在 `md` 断点保持。不改字段绑定与 `updateField`。
5. [x] 回归：现有 editor/view smoke 保持通过。补最小断言：第三方有 `.pe-nav` / `.pe-scroll` / `.pe-footer`；官方导航无 connection；校验失败出现 jump。

## 验证

- [x] `cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/grok-profile-editor.smoke.test.ts tests/grok-profiles-view.smoke.test.ts`
- [x] `cd ccr-ui && bun run type-check && bun run lint && bun run test:i18n`
- [x] web preview：`bun run dev:web -- --host 127.0.0.1 --strictPort` → `http://127.0.0.1:5173/` Grok Profiles 添加第三方。视口 1280×720、1440×900 × 暗/亮。确认可滚到 Tags/Enabled，页脚固定，段导航可跳。
- [x] 官方短表单：无连接段，页脚可见，无空滚动条。
- [x] 完成后走 `trellis-check`。

## 风险文件

- `ccr-ui/src/components/grok/GrokProfileEditorModal.vue` — 主改动
- `ccr-ui/src/i18n/locales/{zh-CN,en-US}.ts` — 仅新增 editor 文案
- `ccr-ui/tests/grok-profile-editor.smoke.test.ts` — 补外壳断言
- 默认不改 `profile-editor-shell.css` / `BaseModal.vue`

## 回滚

回退上述文件即可。不要改 `BaseModal` 默认值来“顺便”修其他模态。
