# 统一 Profile 编辑器外壳与三份平台 adapter — 执行计划

前置：`08-26-profile-registry-tokens` 已完成，`ProfileEditorAdapter` 类型与 `stripCredentials` 可用。

## 步骤

### 1. 前置确认

- [ ] 读 `profiles-page-contracts.md` 的 Codex 条件必填矩阵与 Grok write-only patch 两节，抄出完整规则表写入 `notes.md`，作为 adapter 实现与测试的对照源。
- [ ] 读 `features/grok/profiles/grokEditorValidation.ts`、`GrokProfileEditorModal.tsx`，记录已解决的交互细节（`hasExistingBaseUrl` 留空放行、official 隐藏分支、credential action 联动）。
- [ ] 读 `utils/claudeProfileEditor.ts`、`utils/codexProfileEditor.ts`、`utils/grokProfileEditor.ts`，确认三份表单模型与构建器的入参签名。
- [ ] **后端语义确认**：读 `ccr-ui/src-tauri/src/commands/claude.rs` 与 `codex.rs` 的 profile 更新路径，确认 request 中 `auth_token` 键缺席时后端保留原值而非清空。结论写入 `notes.md`。
- [ ] 确认 `src/ui` 下既有的 Dialog 原语，选定复用对象。
- [ ] 确认 `08-26-profile-list-surface` 已落地的共享原子类清单（见其 `notes.md`）。

**审阅点**：Codex 矩阵表与后端 `auth_token` 缺席语义两项结论需先记录，再进入编码。若后端把缺席当作清空，停下记录字段缺口，按 design.md 风险一节处理。

### 2. 表单状态

- [ ] 新建 `features/platform/profiles/useProfileEditor.ts`。
- [ ] 维护 dirty 集合，语义对齐 `useGrokProfilesPage` 现有的 `formState.dirtyFields`。
- [ ] 提交编排：validate → submit → （`ok` 且 apply 时）`config.apply` → 按 outcome 四分支处理。
- [ ] 外壳不解析后端 status 字符串，只按 `ProfileWriteOutcome` 的 tag 分支。

### 3. 字段渲染与模态外壳

- [ ] 新建 `components/profiles/ProfileEditorFields.tsx`，七种 `kind` 各一个分支。
- [ ] 新建 `components/profiles/ProfileEditorModal.tsx`，按 `adapter.sections` 与 `section.layout` 装配。
- [ ] 限高与唯一滚动根按 `pe-shell` / `pe-scroll` 约定，不开 Dialog 的 `scrollable`。
- [ ] 汇总条：列出 `issues`，点击跳转到对应 section。
- [ ] 高级折叠区：`section.advanced` 为真的分区，默认折叠，为空时不渲染折叠控件。
- [ ] 底部：提示文案随模式变化 + 取消 / 保存 / 保存并应用，保存中禁用并显示进行态。
- [ ] i18n：新增文案同步进 `zh-CN` 与 `en-US`。
- [ ] `tests/profile-editor-shell.smoke.test.tsx`：用最小 stub adapter 覆盖外壳行为与 sentinel 泄漏断言。

### 4. 三份 adapter

- [ ] `features/claude/profiles/claudeProfileEditorAdapter.ts`：sections、validate、submit；密钥留空时从 request 删键。
- [ ] `features/codex/profiles/codexProfileEditorAdapter.ts`：按步骤 1 抄下的矩阵表实现 `visible` / `required` / `validate`；`submit` 内部算 `resolvedModel`。
- [ ] `features/grok/profiles/grokProfileEditorAdapter.ts`：validate 直接调 `validateGrokEditor`；submit 按 design.md 的响应映射表转 `ProfileWriteOutcome`。
- [ ] `tests/profile-editor-adapters.smoke.test.ts`：Codex 五 mode 矩阵、Grok credential 互斥与 reasoning-only patch、Claude 四类校验、三平台密钥留空不序列化。

### 5. 样式

- [ ] 重写 `profile-editor-shell.css`：模态外壳、遮罩、分区、认证分组框、折叠区、汇总条。
- [ ] 共享原子类引用 `profiles-shared.css`，不重复定义。
- [ ] 遮罩与阴影走 token，不写 rgba 字面量。

### 6. 验证

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profile-editor-shell.smoke.test.tsx tests/profile-editor-adapters.smoke.test.ts tests/grok-profile-editor.smoke.test.ts tests/platform-surface-unify.smoke.test.ts
```

```bash
just frontend-check-quick
```

- [ ] 硬编码 hex 自查：`rg -n "#[0-9a-fA-F]{3,8}" ccr-ui/src/components/profiles/ProfileEditor*.tsx ccr-ui/src/components/profiles/profile-editor-shell.css`，结果应为空。
- [ ] `rg -l "smoke.test" ccr-ui/src` 结果为空。
- [ ] 确认 `git diff --stat` 中不含 `features/{claude,codex,grok}/*ProfilesView.tsx`、路由文件，且未删除任何文件。

### 7. 主题走查

前置条件按父任务 `design.md`「视觉与响应式验收条件」的主题矩阵（`light|dark` × `neutral|clay`）。

- [ ] 四种组合下各打开一次新建与编辑，检查分区边界、认证分组框内嵌底色、chip 选中态、汇总条、主按钮对比度。
- [ ] Claude 高级区展开后若单列排布超过面板可视高度两屏，改为两列排布，不改核心区布局。

## 验收对照

完成后逐条勾选 `prd.md` 的 AC1–AC22。

## 与并行任务的约定

`08-26-profile-list-surface` 与本任务并行。本任务不写 `profiles-shared.css`。若发现需要的共享原子类缺失，在 `notes.md` 记录并临时在 `profile-editor-shell.css` 中定义，由 rollout 阶段合并去重。
