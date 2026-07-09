# 技术设计:Claude/Codex Profiles 交互与视觉统一

## 1. 组件提升(codex/* → profiles/*)

- `components/codex/CommandPalette.vue` → `components/profiles/ProfilesCommandPalette.vue`:泛型化 props(`profiles: T[]`, `descriptor` 提供 label/busy 判定),actions 由调用方以 `{ id, icon, labelKey, handler }[]` 注入;codex 原文件改为薄壳 re-export(或直接改引用,视引用面而定,当前仅 CodexProfilesView 引用 → 直接移动+改引用)。
- `components/codex/ProfilesQuickRail.vue` → `components/profiles/ProfilesQuickRail.vue`:同上;`enabled !== false` 过滤逻辑本就平台无关。
- 两视图的快捷键处理合并为 `composables/useProfilesHotkeys.ts`:参数 `{ palette, toolbar, enabledProfiles, onApply }`,统一 ⌘K/⌘1-9//、Esc 行为与 isEditableTarget 守卫(现两份重复实现)。

## 2. 确认对话框统一

- Claude 页移除 5 处 confirm/alert:apply/delete 复用 Codex 页的 `openConfirmDialog` 模式——把该模式抽为 `composables/useConfirmAction.ts`(reactive dialog state + executeConfirmedAction),两页共用,ConfirmModal 组件不动。
- rename 确认:handleSave 中的 rename 分支改走同一 dialog(warning 型),保存流程在 confirm 回调中继续;错误路径 `alert(...)` → `uiStore.showError(...)`。

## 3. StatStrip / lastWrite

- `ProfilesStatStrip.vue`:`total-spark`/`recent-spark` props 已可选则直接删除 Codex 页的字面量入参;若组件在无数据时渲染空轨道,改为 `v-if` 不渲染。
- lastWrite:两视图新增 `markWrite()`(写操作成功回调中调用,存 ISO 时间),loadProfiles 不再触碰;显示层保留现有 hint 格式,空值显示"—"。

## 4. 布局与视觉

- `.cp-grid`:`repeat(auto-fill, minmax(420px, 1fr))`,配合 ProfileCard/ClaudeProfileRow 收紧内边距与次要字段(baseUrl 截断、tags 最多 3 个 +N);1280px 以下自动回单列。
- accent:两页 scoped 块 `--cp-accent: var(--color-accent-primary)` 对齐(改 Claude 页一行);Claude 品牌识别保留在页头图标 `--color-platform-claude`。
- modal:`claude-profile-editor-modal` 的 shell 背景改 floating 档材质(`--material-glass-floating-*`),内部 panel 维持不透明;Codex 的 CodexProfileEditorModal 同步。

## 4b. 卡片信息设计(截图复核 P9-P13)

- `ProfileCard.vue`(Codex):BASE URL/MODEL/认证 的 input 样式容器改为键值文本行(label 小写 mono 淡色 + value 正文/等宽),与 `ClaudeProfileRow` 的字段呈现对齐——两平台卡片用同一套字段行样式(可抽 `ProfileFieldRow` 小组件)。
- 当前 profile:filtered 列表渲染前将 `is_current` 项固定提到首位(排序层处理,不破坏用户所选排序的其余顺序),卡片加"当前"边框强调;其"应用"按钮不渲染。
- 非当前行的"应用"按钮:从大号实心降为紧凑 outline(高度 ~28px),hover/focus 时提升为 accent;键盘可达性不变。
- base_url 显示:CSS 无中段省略,用工具函数 `truncateMiddle(url, head, tail)` + `title` 属性全文;栅格列宽给 URL 列更高权重。
- `ProfilesContextRail` 分布条:`v-if="item.count > 0"` 过滤 0 值;全 0 时该分组整体隐藏。
- `ProfilesHeader` 标题:两页 i18n 统一为"<平台> Profiles 管理"(改 codex.profiles.title 文案)。

## 5. 编辑 modal 渐进披露

- `ClaudeProfileEditorSections.vue`:分区容器加 `collapsed` 状态,`basic`/`connection` 默认展开,`auth`/`status` 中的高级字段组(模型 4+4 映射、timeout、auto-compact、traffic 开关)折叠为 `<details>` 风格分组(自绘,带计数徽章"已配置 n 项");编辑已有 profile 时,凡有值的分组自动展开。
- 滚动同步:`syncActiveFormSection` 改 IntersectionObserver(rootMargin 顶部 -140px),移除 @scroll 绑定;nav 点击滚动逻辑不变。

## 6. 权衡

- 不合并两个编辑 modal 为一个泛型组件——字段模型差异大(Claude 20+ 字段 vs Codex auth 模式机),泛型化成本高于收益;只统一外壳材质与分区交互模式。
- CommandPalette 泛型化时不引入新依赖,沿用现有模糊过滤实现。

## 7. 回滚

commit 划分:①对话框/快捷键统一 ②组件提升+Claude 接入 ③StatStrip/lastWrite ④栅格与材质 ⑤modal 渐进披露;各自独立可 revert。
