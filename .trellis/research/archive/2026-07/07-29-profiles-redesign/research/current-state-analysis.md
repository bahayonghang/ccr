# 现状分析：Profiles 页面（Claude Code / Codex）

> 来源：2026-07-29 双 explore 代理代码剖析 + impeccable 双评估（A 设计评审 / B 检测器）。评审快照：`ccr-ui/.impeccable/critique/2026-07-29T09-29-59Z__ccr-ui-src-views-claudecodeprofilesview-vue.md`（Nielsen 23/40）。

## 架构事实

- 两页共享 `ccr-ui/src/components/profiles/` 八件组件：ProfilesHeader / ProfilesStatStrip / ProfilesQuickRail / ProfilesToolbar / ProfilesContextRail / ProfilesCommandPalette / ProfileListRow / ProfilesRawEditorPanel。平台差异经 `i18n-prefix` + descriptor 对象注入，样式经 `--cp-*` 页内 token 别名层。
- 无 Pinia profile store；状态全在视图本地 ref，仅用 `useUIStore`（toast/confirm）。
- 共享 composable：`useProfilesFilter`（平台无关核心）+ `useClaudeProfilesFilter` / `useCodexProfilesFilter` 薄封装；`useProfilesInsights` 同理；`useProfilesHotkeys`（⌘K / ⌘1-9 / `/` / Esc）；`useConfirmAction`。
- API 走 `@/api`（`api/domains/claude.ts` / codex 同族）：list / add / update / delete / apply / export / getRaw / saveRaw（乐观 token 冲突处理）。
- `ProfilesSection` 是两视图各自复制定义的内联函数式组件；`.cp-list-head` markup 同样重复。

## 文件规模

- `ClaudeCodeProfilesView.vue` 1971 行（含内联编辑器 + ~620 行 CSS）
- `CodexProfilesView.vue` 1142 行
- `ProfilesContextRail.vue` 885 行 / `ProfilesCommandPalette.vue` 643 行 / `ProfileCard.vue` 574 行 / `ProfilesRawEditorPanel.vue` 381 行 / `ProfilesToolbar.vue` 368 行 / `ClaudeProfileRow.vue` 320 行
- `CodexProfileEditorModal.vue` 1021 行（props 驱动，~250 行非 scoped 样式含硬编码 light RGBA + `!important`）

## 两页分歧点（重构须收敛）

- 编辑器：Claude 内联在视图（BaseModal + ClaudeProfileEditorSections，独立 `--editor-*` token）；Codex 已抽取为 props 驱动模态（但同样 `--editor-*` + 硬编码色）。
- 卡片：`ClaudeProfileRow`（Tailwind + per-provider 动态色 + 搜索高亮 `<mark>`）vs `ProfileCard.vue`（2 列字段网格 + env-export 复制按钮，Codex 独有）。
- Provider 筛选：仅 Claude 接线。Codex 缺 stale filter watch。
- 错误面：Claude 有内联 loadError/refreshError + retry；Codex 仅 toast。
- Busy 粒度：Codex 有 per-profile busyAction；Claude 未传（列表行无加载反馈）。
- 头部 ⌘K 按钮：Codex 有，Claude 无（仅快捷键可达）。
- StatStrip 特色槽：Claude=Auth split + Last Write；Codex=Config mode + Last Write。
- base_url 显示：Codex 卡片完整显示，Claude 截断为 `https…`。

## 检测器证据（detect.mjs，100 条）

- 72 条 px 字面字号偏离 rem 字阶（重灾区 ProfilesContextRail.vue 18 处；ClaudeProfileRow.vue 为 Tailwind `text-[11px]` 任意值）。
- 27 条 radius 偏阶（多为检测器内置刻度与 DESIGN.md 12–16px 卡片区间不一致的半误报）。
- 1 条 width transition（ProfilesContextRail.vue:737，进度条，低风险）。
- 硬编码颜色 / `!important` / 裸 inline style / 缺 aria 图标按钮：**零发现**。**注意口径**：detect.mjs 的这几条规则只扫描模板 markup 与 inline 上下文，不覆盖 `<style>` 块；上方「文件规模」中 `CodexProfileEditorModal.vue` 非 scoped `<style>` 块内的硬编码 light RGBA 与 `!important` 是**人工走查**发现，两者不矛盾。结论修正为：模板层配色干净，样式块层存在遗留硬编码（编辑器模态）。

## 死代码 / 重复清单

- Claude 页未引用 i18n 键（仅 locale 文件匹配，代码零引用）：`breadcrumbProfiles`、`consoleEyebrow`、`quickSwitchStrip*`、`searchProvidersCount`、`providerSectionsCount`、`providerNav*`、`providerSectionEyebrow/Summary`、`overview*`、`identity*`、`metrics*`、`currentProfileMissingHint`、`currentProfileSummaryFallback`、`readonlyNameHint`、`editorSummaryTitle`、`editorSectionsTitle/Hint`、`tagsPreviewEmpty`、`smallFastModelBadge`、`providerTypeChip`、`enabledCount`、`searchHint`、`quickSwitchHint`。
- `codex.profiles.commandPalette.actionImport` 键存在但未接线。
- `ProfilesStatStrip` 的 `totalSpark` / `recentSpark` sparkline props：两页均未传，死能力。
- Model fallback 链（model→sonnet→opus→haiku→subagent）在 Claude 页重复 4 份（rowDescriptor / railDescriptor.activeFields / railDescriptor.runtimeSummary / useClaudeProfilesInsights.primaryRuntimeModel）。
- Codex base_url 空→officialBaseUrl fallback 重复 3 份（ProfileCard / rowDescriptor / railDescriptor）；authModes 标签查表 ~4 处。
- Codex 表单 `requires_openai_auth` / `openai_login_method` 双源真相（存表单又总由 auth_mode 重算）。
- `cp-spin` keyframes 在 ProfilesHeader 与 ProfileListRow 重复定义。
- `ProfilesCommandPalette` 自带第三套 `--palette-*` token。
- i18n 债务：`translateWithFallback` 硬编码中文回退若干处；Claude 页 ProviderTemplateSelector 部分标签为裸英文。

## 关键行为约束（重构不得破坏）

- apply/delete/rename 均经确认框；⌘/Ctrl+数字键切换也走确认。
- raw TOML 编辑器：`getCurrentEnvironment()` 门槛（仅 local）+ 警告确认 + 乐观 token 冲突处理 + dirty 拦截。
- Codex 页 `onActivated` TTL 刷新（REFRESH_TTL_MS）。
- Claude 页 `normalizeClaudeProfilesState` 修复不一致 is_current。
- `filterClaudeProfiles` 搜索字段：name/desc/provider/provider_type/account/base_url/model/small_fast_model/tags。
