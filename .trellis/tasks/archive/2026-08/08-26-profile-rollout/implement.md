# 三平台接线与集成验收 — 执行计划

前置：前三个子任务全部完成。

## 步骤

### 1. 前置确认

- [ ] 读 `SurfacePage` 与 `PageShell`，填写 design.md 的外壳对比表（loading / 错误 / runtime-unavailable / subnav），结论写入 `notes.md`（R4）。
- [ ] 确认 `listClaudeProfiles` / `listCodexProfiles` 的返回中是否含 `can_off`。不含则按 design.md 的回退条件处理并记录字段缺口。
- [ ] 确认 Claude / Codex 的 raw 命令名（`get_*_raw_text` / `*_save_profiles_raw`）与 domain wrapper 位置。
- [ ] 读 `useGrokProfilesPage.ts` 全文，逐项对照 design.md 的能力承接表，标出每项的现位置。

**审阅点**：外壳结论、`can_off` 来源、能力承接表的现位置三项需先记录，再进入编码。

### 2. Claude 与 Codex 控制器

- [ ] 新建 `features/claude/profiles/useClaudeProfilesPage.ts`：`useQuery` + `select`（剥离 → 投影）+ apply / delete / toggle / off / export / reload + raw-source capability。
- [ ] 新建 `features/codex/profiles/useCodexProfilesPage.ts`，同上。
- [ ] `canOff` 按步骤 1 结论取值。

### 3. Grok 控制器改造

- [ ] 在 `useGrokProfilesPage` 中加投影输出与 `ProfilesControllerResult` 形状的 props 组装，`recovery` 映射到 `notice`。
- [ ] 表单状态从 `useForm` + `GrokProfileEditorModal` 换为 `useProfileEditor` + `grokProfileEditorAdapter`。
- [ ] 不改删除分支判定、recovery 时序、`actionUnsupported` 守卫与全部文案 key。
- [ ] 逐项验证能力承接表，每项标注验证方式与结果。

**审阅点**：能力承接表全部为真且 `tests/grok-profiles-view.smoke.test.ts` 通过后，才允许进入删除步骤。

### 4. 三平台接线

- [ ] `ClaudeProfilesView` 改为组装控制器 + `ProfilesSurface` + `ProfileEditorModal`。
- [ ] `CodexProfilesView` 同上。
- [ ] `GrokProfilesView` 同上，带 subnav（形态由步骤 1 的外壳结论决定）。
- [ ] 三平台各自打开，验证列表、统计、QuickRail、筛选、双视图、Inspector、空态、新建、编辑、应用、停用、Off、source mode（Claude / Codex）全部可用。

**回滚点**：接线完成且验证通过后提交一次，删除步骤单独成提交。

### 5. antigravity 层二注册

- [ ] `configs/profiles.ts` 的 `profilesConfigs` 增加 `antigravity` 键（`list` 返回空快照）。
- [ ] 确认 `configs/profilePresentation.ts` 的 `antigravity` 实例已由 registry-tokens 建立并进入注册表。
- [ ] 不改 `config/platformDescriptors.ts`，不加路由。
- [ ] 命名不一致（antigravity vs descriptor id `gemini`）写入 `notes.md` 上报。

### 6. 删除与级联清理

按 design.md 删除清单执行，每项先确认前置条件成立。

- [ ] 删除 `features/grok/profiles/GrokProfilesPage.tsx`、`GrokProfileCard.tsx`、`GrokProfileEditorModal.tsx`。
- [ ] 删除 `components/profiles/ProfileListRow.tsx`。
- [ ] 处置 `BaseProfiles.tsx`：删除或退化为薄封装，理由写入 `notes.md`。
- [ ] 确认未删除 `useGrokProfilesPage.ts`、`grokEditorValidation.ts` 与全部已接入的共享组件。
- [ ] 级联清理导出：`components/profiles/index.ts`、`features/platform/profiles/shared.ts`、`features/platform/index.ts`。
- [ ] 清理被删组件独有的 CSS 类与 i18n key（`zh-CN` 与 `en-US` 同步）。
- [ ] 合并 `08-26-profile-editor` 临时定义在 `profile-editor-shell.css` 的共享原子类到 `profiles-shared.css`，去重。

### 7. 待决项清结

- [ ] 逐条结清前序三个子任务 `notes.md` 中的待决项，按 design.md「前序子任务待决项清结」的三组清单记录结论（AC14）。

### 8. 测试

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profiles-platform-wiring.smoke.test.tsx tests/grok-profiles-view.smoke.test.ts tests/grok-profile-editor.smoke.test.ts tests/platform-surface-unify.smoke.test.ts
```

- [ ] 新建 `tests/profiles-platform-wiring.smoke.test.tsx`：三平台控制器渲染、`canOff` 传递、raw-source 入口存在性、Grok `notice`、antigravity 注册表取值。
- [ ] 跑全量 smoke：`bun run test:smoke`。
- [ ] 零消费组件检查：对 `components/profiles/index.ts` 每个导出 `rg` 确认存在非 barrel 消费方，清单写入 `notes.md`。
- [ ] `rg -l "smoke.test" ccr-ui/src` 结果为空。

### 9. 整体验收

```bash
just frontend-check-quick
```

```bash
just ui-check
```

走查条件按父任务 `design.md`「视觉与响应式验收条件」，共 3 平台 × 2 viewport × 4 主题组合 = 24 次。

- [ ] 24 次走查逐项对照 `../08-26-profile-design-language/research/design-source.md` 的结构清单，结论记入 `notes.md`。
- [ ] `900×800` 下记录三平台表格容器与 body 的 `scrollWidth` / `clientWidth` 实测值。
- [ ] 凭据 sentinel 端到端验证（父任务 AC5 的六处）。
- [ ] 硬编码 hex 自查：`rg -n "#[0-9a-fA-F]{3,8}" ccr-ui/src/components/profiles ccr-ui/src/features/platform/profiles ccr-ui/src/features/grok/profiles`，结果应为空。
- [ ] 逐条勾选本任务 AC1–AC21，以及父任务 `prd.md` Acceptance Criteria 一节的 28 条。

## 提交划分

1. Claude / Codex 控制器 + Grok 控制器改造
2. 三平台接线 + antigravity 注册
3. 删除与级联清理
4. 测试与文案清理

删除单独成提交，便于在发现能力丢失时精确回滚。

## 风险

- 删除步骤可能暴露前三个子任务未覆盖的依赖。发生时回到对应子任务修补，不在本任务内堆积临时兼容代码。
- Grok 表单状态迁移是风险最高的一步，回退方案见 design.md 风险一节。
- `just ui-check` 耗时较长，安排在删除与清理全部完成后跑，不在中途反复触发。
- 若某条 AC 无法达成，回到规划修订，不在本任务内改写验收口径。
