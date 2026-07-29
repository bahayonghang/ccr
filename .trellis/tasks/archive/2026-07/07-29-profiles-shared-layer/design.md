# Design: Profiles 共享组件层重构（差异点）

总体契约见父任务 `../07-29-profiles-redesign/design.md`。本文件只写共享层的落地形态决策。

## 组件改动清单与兼容策略

**总原则（P0 独立交付契约）**：纯新增。既有组件只加可选 props / 新模式，默认值 = 现状行为；平台页迁移前不需要任何改动。凡行为差异大到无法用可选 prop 表达的，新建 V2 组件文件，旧文件原样保留。旧路径删除统一推迟到父任务集成步骤 4。

| 组件/模块 | 改动 | 兼容策略 |
|-----------|------|----------|
| `ProfilesQuickRail.vue` | 新增可选 prop `quickSwitch`（`useProfilesQuickSwitch` 返回值）；提供时渲染新模式（钉选编号 + recent 无编号 + pin 操作 + more 入口 + 修饰键提示），缺省时完全走现有渲染 | 可选 prop，默认旧行为 |
| `composables/useProfilesQuickSwitch.ts`（新） | 钉选数组（编号唯一来源，≤8）+ 最近列表（不编号）、`pin/unpin/isPinned/canPin`、`recordUse(name)`、stale 名称清理（加载时对当前列表过滤回写；重命名跟随新名）、localStorage 键 `ccr:profiles:pinned:{platform}` / `ccr:profiles:recent:{platform}`、`modifier`（基于 `getClientPlatform()`） | 新文件 |
| `composables/useProfilesHotkeys.ts` | 数字键 handler 可注入 `getStableTargets: () => string[]`；未注入时保持现有行为 | 向后兼容 |
| `components/profiles/ProfilesInspector.vue`（新 V2） | 预览面板（`previewProfile` + `currentProfile` + diff 高亮）/ Health（`@locate`）/ Distribution（`<details>` 默认折叠）/ tag cloud 可点击 emit `tag-select`。**不动旧 `ProfilesContextRail.vue`**，平台页迁移时换用，旧文件在集成步骤 4 删除 | 新文件 |
| `components/profiles/ProfileDiffRows.vue`（新） | 三行 diff 展示（label / 当前值 → 目标值 / 相同弱化、不同强调），供 Inspector 预览与 Apply 确认框共用 | 新文件 |
| `ConfirmModal` / `useConfirmAction` | 新增命名 slot / `diffRows` prop 与 `footnote` 文案行支持 | 可选，默认旧行为 |
| `ProfilesToolbar.vue` | 新增可选模式 `compactFilters`：开启时 tag/provider/sort 移入 Filters popover（含生效数徽标 + 清除全部），缺省保持现有平铺 | 可选 prop，默认旧行为 |
| `ProfilesStatStrip.vue` | 新增可选四槽 props（`current / total / specialty / health`）；旧 props（Last Write、sparkline）标记废弃但保留至集成步骤 4 | 可选 props，默认旧行为 |
| `ProfilesSection.vue`（新文件） | 从两视图提取的内联函数式组件落地为共享文件 | 新文件 |
| `components/profiles/profile-editor-shell.css`（新） | 编辑器模态共享样式基底：消费 `--cp-*`/全局 token，rem 字阶 | 新文件，编辑器迁移时引用 |
| `ProfilesCommandPalette.vue` | `--palette-*` 变量映射到 `--cp-*`/全局 token（值不变，纯别名替换，视觉零变化） | 纯样式别名，视觉不变 |
| `ProfilesHeader.vue` | 新增可选 `actionsMenu` 模式（Add + ⌘K + ··· 溢出）；缺省保持现有平铺按钮 | 可选 prop，默认旧行为 |

## 技术决策

- **钉选持久化**用 localStorage 而非后端：钉选是 UI 偏好不是配置事实；同步到 CCR 后端属于非目标。
- **编号语义（P0 修正）**：数字编号 = 钉选数组顺序（1..n，n≤8），是唯一编号来源。`pinnedProfiles` 顺序 = 用户钉选操作顺序（数组尾插）；`recentNotPinned` 按 `recordUse` 时间倒序、**只展示不编号**。过滤/排序/搜索/Apply 均不改变钉选数组。栏内容 = 钉选 chip（带序号）+ 最近 chip（无序号），总数 ≤8，超出走「+N more → ⌘K」。
- **钉选上限**：第 9 次钉选拒绝并 toast（i18n 键），不挤掉既有钉选。
- **stale 名称清理**：`useProfilesQuickSwitch` 接收当前 profile 名列表，watch 中过滤不存在的名称并回写 localStorage；重命名由视图在 rename 成功后调用 `renamePinned(oldName, newName)`；禁用不清理仅置灰。
- **最近使用**：`recordUse` 在 apply 成功后调用。
- **平台修饰键**：复用 `getClientPlatform()`（`ccr-ui/src/utils/windowChrome.ts:32`），windows/linux → `Ctrl`，macos → `⌘`；不直读 `navigator.platform`。
- **diff 计算**：`buildProfileDiff(current, target)` 放 `utils/`（平台无关壳 + 平台字段提取函数注入），输出 `[{key,label,from,to,changed}]`，供 ProfileDiffRows 渲染。
- **预览驱动（状态机修正）**：视图持有 `hoveredName` 与 `focusedName` 两个 ref，优先级 `hoveredName ?? focusedName ?? activeProfile`；`focusout` 用 `relatedTarget` 判断是否移出卡片；预览目标被删除即清空回落，重命名跟随新名（父 design §3.2）。
- **字号扩展点**：密排元信息统一 0.75rem（Label 之下新增一级，在实施时登记到 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`）。

## 不做

- 不改命令面板交互范式（仅 token 与入口）。
- 不重建 Health Audit 规则本身（`useProfilesInsights` 逻辑保留，仅消费方式变化）。
