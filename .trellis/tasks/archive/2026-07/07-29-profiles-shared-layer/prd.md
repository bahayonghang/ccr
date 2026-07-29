# PRD: Profiles 共享组件层重构

父任务：`.trellis/tasks/07-29-profiles-redesign/`（总体设计见其 `design.md`，本任务实现其中 §2–§5 的全部共享部分）。

## 范围

`ccr-ui/src/components/profiles/` 组件族 + 相关共享 composable/utils 的重构，为两个平台页提供统一新骨架。本任务不含具体平台视图的接入（接入由子任务 ②③ 完成）。

**独立交付契约（P0 约束）**：本任务的一切改动必须是**纯新增、向后兼容**——新文件（composable/组件/css 基底）随意加；对既有组件（QuickRail/Toolbar/StatStrip/ContextRail/Header/ConfirmModal）只允许增加**可选 props 或新模式**，默认值必须保持现有渲染与行为不变。两个平台页在本任务完成后**不需要任何修改**即可继续正常工作（以 `bun run build` / typecheck 证明）。旧 props 分支与旧代码路径的删除不在本任务，统一由父任务集成步骤（`07-29-profiles-redesign/implement.md` 步骤 4）在两个平台页迁移完成后执行。

## 需求

1. **ProfilesQuickRail 瘦身**（父 design §3.1）：钉选 + 最近 ≤8 chip；新 composable `useProfilesQuickSwitch`（稳定编号数组、pin/unpin、最近使用记录、localStorage 持久化、平台修饰键检测与提示插值）；「+N more → ⌘K」入口。
2. **useProfilesHotkeys 稳定编号**：数字键目标改由 `useProfilesQuickSwitch` 的稳定数组提供，与过滤/排序解耦；提示文案带 `{modifier}` 插值（Windows=Ctrl，mac=⌘）。
3. **ProfilesContextRail → ProfilesInspector**（父 design §3.2）：预览面板（hover/focus 驱动 + diff 高亮）、Health Audit 联动定位、Distribution 默认折叠；tag cloud 可点击写筛选。
4. **确认框 diff 能力**（父 design §3.3）：ConfirmModal/确认流支持结构化 diff 内容（当前 → 目标，base_url/model/auth_mode 三行）与 delete 备份提示行。
5. **ProfilesToolbar 筛选收敛**（父 design §3.4）：裸露 ≤3 控件 + Filters popover（标签/provider/排序 + 清除全部 + 生效数徽标）。
6. **StatStrip 同 schema 四槽**（父 design §3.7）：Current / Total / 平台特色槽 / Health；移除 Last Write 槽与 sparkline 死代码。
7. **列表/卡片视觉统一**：`ProfileListRow` 保持 `--cp-*`；提供统一的字段格式化/占位约定供平台卡片遵循；列表行 busyAction 反馈可用。
8. **编辑器外壳统一**：定义编辑器模态共享样式基底（消费 `--cp-*`/全局 token，rem 字阶，无 `!important`/硬编码色），供 Claude（新抽取）与 Codex（迁移）两个编辑器模态使用。
9. **共享逻辑去重**（父 design §5）：提取 `ProfilesSection.vue`；删除 `cp-spin` 重复定义；`--palette-*` 并入 `--cp-*` 或全局 token。
10. **可访问性**：QuickRail `role="toolbar"` + roving tabindex；pill 组 `aria-pressed`；Inspector `aria-live` 策略（父 design §4）。

## 验收标准

- 父任务跨子任务验收标准 1、2、5、6、7、9 中属于共享层的部分全部满足（以新 API 形态验证，平台页接入在子任务 ②③ 验证）。
- **纯新增证明**：两个平台页零改动通过 `bun run build` / typecheck 与 `bun run test`——既有组件的默认渲染与行为逐 prop 核对无变化。
- 需求 1–10 中以「新模式/新组件」形态交付的能力，有最小 demo 或测试证明可用（不依赖平台页接入）。
- 新组件/composable 有对应 i18n 键（zh-CN + en-US 对称），无硬编码回退文案。
- 旧代码路径（旧 props 分支、sparkline、Last Write 槽等）标记 `TODO(profiles-redesign): 集成步骤删除` 注释，但**不在本任务删除**。
