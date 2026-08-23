# 执行计划：Profiles 与配置视图迁移

> 父任务：`08-22-react-migration`。**批次 1 属阶段 4a（共享层前置），批次 2 起属阶段 5（七个视图子任务并行）。**
> 分支：`feature/react-migration/views-profiles-config`，PR 目标 `feature/react-migration`。批次 1 单独开 PR 并先行合入，`08-22-platform-unify` 批次 4 依赖它。

## 外壳接口公示（08-22-shell-port 批次 6）

- 共享接口：`.trellis/tasks/08-22-shell-port/shared-interfaces.md`。阶段 4a 之后不改 `MasterDetailLayout` 与 `src/ui/` 原语接口。
- **AC11**：`configs` 表单草稿的界面级验证（离开后返回草稿仍在）由本任务批次 2 执行。store 侧 `useConfigsViewStore.formDrafts` 已按配置 id 可读写（memory-only）。

## 前置确认

### 批次 1 的前置（阶段 3 外壳门通过后即可开工）

- [x] 父任务外壳门已通过（`08-22-shell-port` 与 `08-22-state-logic-port` 已交付）。
- [x] `08-22-design-system` 已迁移 `profiles-page.css`（28 变量），`0.75rem` 例外已在 `hardcode-exemptions.md` 中登记。
- [x] 通读 `profiles-page-contracts.md`（19.9 KB）。PRD Notes：不先通读会导致迁移后大量返工。
- [ ] `git checkout -b feature/react-migration/views-profiles-config feature/react-migration`（批次 1 按派遣留在 `react-migration/react-foundation`，不另开此分支）

### 批次 2 起的前置

- [ ] 父任务统一层门已通过。本域范围行数不变，无需回填。
- [ ] 另读 `provider-template-contracts.md`（8.9 KB）、`raw-config-editor-contracts.md`（7.4 KB）。
- [ ] 前置阅读完成（`08-22-views-claude/design.md` §1 末段的五份文档）。
- [ ] 前置阅读：`.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md`（R8，动手前必读）。
- [ ] `08-22-state-logic-port` 已提供 `configsKeys` 的 Query hook 与表单草稿的 Zustand 键（配置 id）。

## 批次 1：`components/profiles/` 迁移与接口定稳（阶段 4a，最高优先级）

必须在 `08-22-platform-unify` 批次 4 建 `BaseProfiles` 之前完成（协同点 F）。React base 组件无法复用未迁移的 Vue 组件，因此这是**框架迁移**而非仅接口定义。

- [x] 10 个文件（4,040 行）迁移。多数情况是把现有 Vue 接口如实映射到 React，不重新设计（Out of Scope：复用不改造）。
- [x] 按 `design.md` §3 公示接口：10 个文件各自的 props 列表与类型、slot → children / render props 映射、状态责任划分。
- [x] 公示文档落盘为 `profiles-shared-interfaces.md`，通知 `08-22-platform-unify` 与三个平台视图子任务。
- [x] 改造需求登记为独立缺陷，不在本批次做。
- [x] `0.75rem` 字号例外在此层保留。

验证：`bun run type-check` 退出码 0。该批次单独开 PR，先行合入迁移分支（父任务阶段 4a 门的准出项）。

## 批次 2：配置管理视图与弹层（阶段 5 起）

- [x] `ConfigsView`(520)、`EditConfigModal`(406)、`AddConfigModal`(342)、`ConfigCard`(331)、`components/configs/`（3 文件 654 行）。
- [x] `configs` 路由的缓存行为：数据走 Query，选中态、搜索词、表单草稿入 Zustand（草稿键为配置 id）。**补 `08-22-shell-port` AC11 的界面级表单草稿验证**——该项在外壳门时因视图未迁移而无法做，已由对方 AC11 单列并移交本批次。
- [ ] 弹层走 `08-22-design-system` 的单一 Dialog 底座（或其 `BaseModal` 适配器）。
- [ ] 嵌入原始配置编辑器的部分等 `08-22-views-sync-tools` 交付桥接组件后再做（`design.md` §8）。该依赖需与对方协调时序。

验证：配置列表浏览、新增、编辑、切换、删除走通；缓存行为验证。

## 批次 3：配置表单

- [x] `AppSettingsView`(1,399)：react-hook-form 非受控注册 + zod schema。该页是 `08-22-arch-quality-perf` 场景 1 的测量页之一。
- [ ] 超过行数上限，需拆分。拆分边界按设置分组（section）划分。
- [x] **字段清单落盘**（AC4）：每个字段的名称、类型、默认值、校验规则、读写验证结果。覆盖全部表单页，无空缺。
- [x] API key 与 auth token 字段掩码显示。

验证：AC4 的字段清单逐项读写正确；AC5 的掩码与日志无明文（smoke 测试断言）。

## 批次 4：Provider 模板与转换

- [x] `ProviderTemplateSelector`(1,275)：超过行数上限，需拆分。
- [x] `ConverterView`(915)。核对是否与 `profileDiff` 共用渲染组件。
- [ ] `provider-template-contracts.md` 的模板选择与应用行为验证（R5）。

## 批次 5：安全行为与契约验证

- [x] 配置切换前生成备份，备份文件可用于恢复（AC6）。
- [x] 配置写入原子性：确认前端只调现有 wrapper，无新增直写路径。中断写入的验证方法确定并执行（AC7、`design.md` §5 末段）。
- [x] 前端绕过路径排查：若存在，登记为独立缺陷，不在本任务修复。
- [x] `profileDiff` 的差异展示视觉与语义一致（R9）。
- [ ] 三份契约的验证项全部通过（AC8）。`raw-config-editor-contracts.md` 只验本任务的调用路径，桥接实现侧由 `08-22-views-sync-tools` 验。

## 批次 6：收口

- [x] 本批次组件内 px 与 `rgba()` 归零，`0.75rem` 例外保留并在豁免登记中（AC9）。
- [x] `rg --files -g '*.vue' src/components/profiles src/components/provider-templates src/components/configs src/views/AppSettingsView.vue src/views/ConverterView.vue src/views/ConfigsView.vue src/components/EditConfigModal.vue src/components/AddConfigModal.vue src/components/ConfigCard.vue` 无匹配（AC1）。
- [ ] `src/api`、`src/config`、`src/configs` 的 diff 检查：范围限本任务分支的提交，排除 `08-22-platform-unify` 对 `src/config` / `src/configs` 的改动（AC10、`design.md` §10 末段）。

## 验证命令

| 时机        | 命令                                                                      |
| ----------- | ------------------------------------------------------------------------- |
| 每批次后    | `bun run type-check`、`bun run lint`（AC11）                              |
| 批次 1–4 后 | `bun run test:smoke`（AC12）                                              |
| 批次 6 后   | 上表 AC1 的 `rg` 命令                                                     |
| 交付前      | `just frontend-check-quick`、`bun run lint:ci`、`just secret-write-check` |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC12 全部满足。
- [ ] AC3 的 9 条核心操作路径逐条验证并记录：配置列表浏览、配置新增、配置编辑、配置切换、配置删除、Provider 模板选择与应用、配置格式转换、应用设置读写、Profile 差异查看。
- [ ] `profiles-shared-interfaces.md` 落盘并已通知，且 `components/profiles/` 10 文件已迁为 React（批次 1，属阶段 4a 门，早于 `08-22-platform-unify` 批次 4）。
- [ ] 字段清单落盘，覆盖全部表单页无空缺（AC4）。
- [ ] 掩码、备份、原子写入三项验证通过（AC5–AC7）。
- [ ] 三份契约验证通过（AC8）。
- [ ] `0.75rem` 例外已保留并登记。
- [ ] `08-22-shell-port` AC11 的界面级表单草稿验证已补做（批次 2）。
- [ ] 前端绕过路径排查结论记录（有则登记为独立缺陷）。

## 回滚点

批次 1 属阶段 4a，单独开 PR 并先行合入。**定稳后不回滚**——`08-22-platform-unify` 的 `BaseProfiles` 依赖它，回滚会同时废掉统一层批次 4。

批次 2–6 各自独立提交。批次 3 与 4 内的大文件拆分按拆分边界分多次提交。

## 协同点

| 编号 | 内容                                          | 对方                          | 时机                            |
| ---- | --------------------------------------------- | ----------------------------- | ------------------------------- |
| F    | `components/profiles/` 迁移 + 接口公示        | `08-22-platform-unify`        | 批次 1（阶段 4a），须早于对方批次 4 |
| D    | 三份契约的重写稿                              | `08-22-test-contract-rebuild` | 前置                            |
| I    | i18n 调用形式                                 | `08-22-i18n-port`             | 全程                            |
| P    | 原始配置编辑器桥接组件的交付时序              | `08-22-views-sync-tools`      | 批次 2                          |
| —    | `profiles-page.css` 迁移与 `0.75rem` 例外登记 | `08-22-design-system`         | 批次 1 前置                     |
| —    | `configsKeys` 与表单草稿键                    | `08-22-state-logic-port`      | 批次 2 前置                     |
| —    | 表单草稿的界面级验证补做（对方 AC11）         | `08-22-shell-port`            | 批次 2                          |
