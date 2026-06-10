# 签到组件拆分与死代码清理

> 父任务: [06-10-checkin-optimize-templates](../06-10-checkin-optimize-templates/prd.md) · 工作包 6 · 依赖: 前 4 个子任务完成后收尾（纯重构，行为不变）

## Goal

治理签到页面的巨型组件与样式重复，删除无路由引用的遗留视图，行为零变化。

## Requirements

1. **CheckinAccountsTab.vue 拆分**（现 2062 行，其中 ~1050 行 scoped CSS）：
   - 拆出 `AccountFormModal`（账号编辑大弹窗，含 CDK 三种凭证字段区 + session↔cookies JSON 互转逻辑，script :660-678/:794-831）。
   - 拆出 `AccountActionsMenu`（浮动操作菜单 + 手写定位算法，:740-791）。
   - 拆出账号卡片组件（template 网格区）。
   - 主文件目标 ≤600 行。
2. **CSS 公共层**：AccountsTab 与 ProvidersTab 雷同的卡片/徽章样式抽到共享样式（遵循 theme-token-contracts；不引入新视觉，不动主题 token 语义）。
3. **死代码删除**（删除前 `rg` 确认无引用）：
   - `views/checkin/CheckinManageView.vue`（无路由注册、无引用）。
   - 其专属子组件 `views/checkin/components/{CheckinStats,AccountManager,CheckinHistory}.vue`（仅被 CheckinManageView 使用；TokenConfig.vue 等其他组件先核实引用再决定）。
   - `stores/checkin.ts`（仅被上述死组件使用）。
4. **行为不变约束**：现有 smoke 测试（checkin-accounts-tab 8 用例、checkin-progress-modal、checkin-state）随组件迁移更新引用路径后全绿；不顺手改文案/逻辑/样式语义。

## Acceptance Criteria

- [ ] CheckinAccountsTab.vue ≤600 行；新组件职责单一、props/emits 类型完整。
- [ ] `rg -i "CheckinManageView|stores/checkin" ccr-ui/src` 无残留引用；删除文件列入 PR 描述。
- [ ] `bun run test`（含迁移后的 smoke）+ `bun run type-check` + `just frontend-check-quick` 绿。
- [ ] 视觉零变化（卡片/徽章在明暗主题下对比检查）。

## Out of Scope

- CheckinView.vue（1599 行）与 CheckinProvidersTab、ProviderTemplateSelector 的拆分（如有需要另行任务）。
- 任何行为/视觉/文案变更。

## Technical Notes

- 拆分方向与行数构成：[`../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md`](../06-10-checkin-optimize-templates/research/internal-checkin-architecture.md)（§CheckinAccountsTab.vue 62KB 拆解 / §旁路与遗留）。
- 样式约束：`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`。
- 全局规范：删除仅限本任务明确列出的死代码，不扩大化。
