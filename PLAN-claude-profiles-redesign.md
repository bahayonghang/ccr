# Claude Code Profiles 页面重设计 — 对齐 Codex 编辑式工作台

> 状态：**全部完成 ✅** ｜ 验证：type-check / eslint / stylelint / i18n test / build 均通过

## 背景 Context

CCR UI 里有两个 Profile 页面，视觉语言已经分叉：

- **Codex Profiles**（参考标杆）：单列主区 + 右侧上下文侧栏的「编辑式工作台」布局 —— Header → 4 列统计条 → 快速切换条 → 工具条（搜索 + 状态/标签 pill + 排序 + 列表/卡片切换）→ 启用/禁用分组列表 → 右侧 3 面板上下文侧栏（当前 Profile 详情 / 分布洞察 / 健康审计）。
- **Claude Code Profiles**（待重设计）：面包屑 → `PageHeaderCard` + 指标 ticker → 搜索栏 → 快速切换条 → 左侧 Provider 导航 + Provider 分组区块的两列布局。无右侧上下文侧栏、无统计条、无筛选工具条。

目标：把 Claude 页面重做成与 Codex 一致的编辑式工作台风格（CLAUDE.md 设计原则 #4 "One Visual Language"），同时尊重两边数据模型差异。**纯布局/表现层重做，编辑器流程不动。**

确认方向（与用户对齐）：
1. 完整对齐 Codex 布局（单列 + 右侧上下文侧栏，移除左侧 Provider 导航），并在工具条中额外加 Provider 筛选。
2. 右侧侧栏保留三个面板全部。
3. 工具条包含：状态 pill + 标签 pill + 排序下拉 + 列表/卡片切换。

数据模型差异：`ClaudeProfile` 多模型字段（opus/sonnet/haiku/subagent）、`auth_mode` 只有 `subscription | api_key`（无废弃模式）；`CodexProfile` 则有 `openai_login_method`/`credential_store`/`env_key` 和 4 值含废弃项的 auth。

---

## 任务清单 Checklist

- [x] **新建 `useClaudeProfilesFilter` composable** — `src/composables/useClaudeProfilesFilter.ts`
  状态/标签/Provider 过滤 + 排序 + 派生 `allTags`/`allProviders`/`enabledList`/`disabledList`/`activeProfile`；查询委托现有 `filterClaudeProfiles`，Provider 过滤用 `getClaudeProfileProviderKey`。
- [x] **新建 `useClaudeProfilesInsights` composable** — `src/composables/useClaudeProfilesInsights.ts`
  分布洞察（provider top5 / auth 2 桶 / tag top8）+ 健康审计（去掉废弃 auth 检测）：
  - api_key 模式缺 base_url 才报
  - `model` 与 `default_opus/sonnet/haiku_model`+`subagent_model` 全空才报"无模型"
  - 缺 account
  - 运行时重复 key=`base_url | (model||default_sonnet_model||default_opus_model)`，组 ≥2
- [x] **新建 `ClaudeProfilesHeader` 组件** — `src/components/claude/profiles/ClaudeProfilesHeader.vue`
  标题/副标题 + 返回/重载/导出/添加；去掉 ⌘K 命令面板按钮；含共享 `.cp-btn` 样式。
- [x] **新建 `ClaudeProfilesStatStrip` 组件** — `src/components/claude/profiles/ClaudeProfilesStatStrip.vue`
  4 列：当前 Profile / 配置总数 / 认证分布（订阅·API Key）/ 最近写入；不含 Sparkline。
- [x] **新建 `ClaudeProfilesToolbar` 组件** — `src/components/claude/profiles/ClaudeProfilesToolbar.vue`
  搜索（/ 快捷键）+ 状态 pill + 标签 pill + **Provider `<select>` 下拉** + 排序 + 列表/卡片切换；`defineExpose({ focusSearch })`。
- [x] **新建 `ClaudeProfilesContextRail` 组件** — `src/components/claude/profiles/ClaudeProfilesContextRail.vue`
  3 面板（当前 Profile 详情 / 分布洞察 / 健康审计），消费 `useClaudeProfilesInsights`，仅 ≥1280px 显示，问题项点击打开编辑器。
- [x] **新建 `ClaudeProfileListRow` 组件** — `src/components/claude/profiles/ClaudeProfileListRow.vue`
  密集列表行，含 Claude 多模型摘要列。
- [x] **新增 i18n 键（zh-CN + en-US）** — `src/i18n/locales/{zh-CN,en-US}.ts`
  `claudeProfiles.*` 下新增 `officialBaseUrl`/`reloadAction`/`groups`/`statStrip`/`toolbar`/`fields`/`contextRail`（含 `issues`）；两语言键树一致。
- [x] **重写 `ClaudeCodeProfilesView.vue` 视图** — `src/views/ClaudeCodeProfilesView.vue`
  - 根 `.claude-profiles-view` 声明作用域 `--cp-*` 令牌（主色 = 暖中性 `accent-secondary`，与统一身份色系统一致）
  - `ModuleSubnav module="claude-code"` 替代面包屑
  - `cp-shell` grid（1fr ｜ 320px @≥1280px）：Header / StatStrip / Toolbar / 三态 / 启用·禁用分组（card=`ClaudeProfileRow`，list=`ClaudeProfileListRow`）/ ContextRail
  - 接线 `useClaudeProfilesFilter` / `useClaudeProfilesInsights`
  - `<BaseModal>` 编辑器整块 + 编辑器全局样式逐字保留；卡片行无 payload emit 正确接线 `@apply="handleApply(profile.name)"`
  - `findProfile()` 把列表行的 name emit 还原成完整记录给 `openEditForm`
  - 移除左导航机制（sectionObserver/sectionRefs/registerSectionRef/scrollToSection 等），保留模态自身的 `modalScrollRef`+`syncActiveFormSection`
- [x] **删除废弃组件并清理 + 验证**
  - 删除 `ClaudeProfilesProviderNav.vue` / `ClaudeProfilesSectionList.vue` / `ClaudeProfilesOverview.vue`
  - 删除 `claudeProfiles.ts` 中 `createClaudeProfilesOverviewSummary` / `ClaudeProfilesOverviewSummary`
  - 无悬空 import/引用

---

## 验证结果 Verification

| 检查 | 命令 | 结果 |
|------|------|------|
| 类型检查 | `bun run type-check` | ✅ exit 0 |
| ESLint | `eslint --fix`（改动文件）| ✅ exit 0 |
| Stylelint | `stylelint --fix`（Vue 文件）| ✅ exit 0 |
| i18n 测试 | `bun run test:i18n` | ✅ exit 0 |
| 生产构建 | `bun run build` | ✅ exit 0，无错误 |
| 悬空引用 | grep 已删除组件/函数 | ✅ 0 命中 |

### 仍建议手测（`bun run tauri dev` → Claude Code → Profiles）

- 卡片 / 列表两视图渲染启用·禁用分组；apply/edit/delete 正常；当前徽章 + provider 配色
- 工具条：搜索、状态 pill、标签 pill、Provider 下拉、排序、视图切换、`/` 与 ⌘K 聚焦、`n/total` 计数
- 右侧侧栏（≥1280px）：当前 Profile 面板 / 分布柱状 / 审计列表（api_key 缺 base_url、无模型、缺账号、运行时重复），点击问题项打开编辑器
- 编辑器模态不变：新增、编辑、改名冲突二次确认、保存、取消、分区滚动联动、错误横幅
- 导出 profiles.toml、重载、loading/empty/no-results 三态

---

## 关键文件 Touched Files

**新增**
- `ccr-ui/src/composables/useClaudeProfilesFilter.ts`
- `ccr-ui/src/composables/useClaudeProfilesInsights.ts`
- `ccr-ui/src/components/claude/profiles/ClaudeProfilesHeader.vue`
- `ccr-ui/src/components/claude/profiles/ClaudeProfilesStatStrip.vue`
- `ccr-ui/src/components/claude/profiles/ClaudeProfilesToolbar.vue`
- `ccr-ui/src/components/claude/profiles/ClaudeProfilesContextRail.vue`
- `ccr-ui/src/components/claude/profiles/ClaudeProfileListRow.vue`

**修改**
- `ccr-ui/src/views/ClaudeCodeProfilesView.vue`（模板 + 视图 chrome 样式重写，编辑器逻辑/模板/全局样式保留）
- `ccr-ui/src/utils/claudeProfiles.ts`（移除 overview summary）
- `ccr-ui/src/i18n/locales/zh-CN.ts`、`ccr-ui/src/i18n/locales/en-US.ts`

**删除**
- `ccr-ui/src/components/claude/ClaudeProfilesProviderNav.vue`
- `ccr-ui/src/components/claude/ClaudeProfilesSectionList.vue`
- `ccr-ui/src/components/claude/ClaudeProfilesOverview.vue`

**复用未改**
- `ccr-ui/src/components/claude/ClaudeProfileRow.vue`（卡片视图）
- `ccr-ui/src/components/claude/ClaudeProfileEditorSections.vue`（编辑器分区）
