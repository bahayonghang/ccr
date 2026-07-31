# Design: Profiles 页面重构总体设计

> 子任务的 `design.md` 只写各自差异点，本文件是两页共享的重构契约。

## 1. 新模式定位

Operate 模式。成功 = 用户快速、有把握地切到对的 profile。设计判断服从 `ccr-ui/DESIGN.md`「The Editorial Control Room」：克制表面、陶土 accent 稀缺、编辑式减法。本页的肌肉（键盘层、组件族、状态诚实）保留，只重做信息架构与交互表层。

## 2. 新页面骨架（两页同构）

```
ModuleSubnav
└─ main.cp-shell（≥1280px: 1fr + 340px 右栏）
   ├─ .cp-main
   │  ├─ ProfilesHeader        标题 + [Add Profile 主按钮] + [⌘K 入口] + [···溢出菜单: Reload / Export / Edit TOML]
   │  ├─ ProfilesStatStrip     同 schema 四槽: Current / Total / 平台特色槽 / Health
   │  ├─ ProfilesQuickRail     ≤8 chip（钉选 + 最近）+ 稳定编号 + 平台化快捷键提示
   │  ├─ ProfilesToolbar       搜索 + 状态 pill 组 + [Filters ⌄ n] + 视图切换
   │  ├─ 状态块（loading / error / empty / search-empty，含 Clear）
   │  └─ ProfilesSection ×2（Enabled / Disabled）→ 卡片网格 或 列表
   └─ ProfilesInspector（右栏，原 ContextRail 重构）
       ① Profile 预览面板（默认=当前；hover/聚焦切换目标；含与当前的 diff 高亮）
       ② Health Audit（问题项可点击 → 定位/编辑对应 profile）
       ③ Distribution（折叠，默认收起）
```

头部可见动作从 7 个收敛到 3 个（Add / ⌘K / ···）。Back 链接保留但弱化。

## 3. 关键交互契约

### 3.1 ProfilesQuickRail（瘦身）

- 内容 = 用户钉选（pin）的 profile + 最近使用填充；钉选状态持久化（localStorage 按平台键，例如 `ccr:profiles:pinned:claude` / `:codex`，最近列表 `ccr:profiles:recent:{platform}`）。
- 每个 chip：名称 + 钉选/取消钉选小操作；活动 profile 陶土填充；**序号角标只出现在钉选 chip 上**。
- **编号稳定性**：数字编号 = 钉选数组顺序（1..n，n≤8），与搜索/筛选/排序/启停状态完全解耦，也**不随 Apply 变化**——最近使用列表虽然在每次 Apply 成功后重排，但最近 chip 不编号，纯展示。`useProfilesHotkeys` 数字键目标 = 钉选数组。
- **钉选上限 8**：钉第 9 个时 toast 提示已达上限（不静默失败、不挤掉既有钉选）。
- **边界规则**：
  - 首次使用（无钉选无最近）：QuickRail 整体隐藏（沿用既有「无启用 profile 时隐藏」行为），引导走卡片 Apply 或 ⌘K。
  - 无钉选但有最近：显示最近 chip（不编号）。
  - 钉选/最近中的名称在当前 profile 列表不存在（已删除/已重命名）：加载时过滤并回写清理后的 localStorage；重命名成功后用新名替换旧名而非丢弃。
  - 被禁用的钉选 profile：chip 保留但置灰且不可 Apply（数字键跳过），不自动移除钉选。
- **Windows 适配**：复用既有 `getClientPlatform()`（`ccr-ui/src/utils/windowChrome.ts:32`，可测试），Windows/Linux 显示 `Ctrl`，macOS 显示 `⌘`；提示文案改为「Ctrl+1 快速切换」式完整表述（i18n 键带 `{modifier}` 插值）。既有 `useProfilesHotkeys` 已同时监听 ctrl/meta，无需改监听逻辑。
- 栏尾显示「+N more → ⌘K」入口 chip（当可用 profile 超出栏内容量时）。

### 3.2 ProfilesInspector（右栏从档案柜改为 X 光片）

- 预览目标由两个独立状态驱动（**不用单一 previewName**）：
  - `hoveredName`：行/卡片 `mouseenter` 写入；`mouseleave` 仅当离开目标是同一行时清空。
  - `focusedName`：行/卡片 `focusin` 写入；`focusout` 时检查 `relatedTarget`——若焦点移至同一卡片内部（如卡片上的按钮）则保留，移至卡片外才清空。
  - 预览优先级：`hoveredName ?? focusedName ?? 当前 profile`。
  - 预览中的 profile 被删除时：两个状态立即清空并回落到当前 profile；被重命名时：跟随新名。
- 预览面板展示完整字段（base_url 完整显示、model 解析后真实值、auth_mode、provider、account、tags），不截断。
- 当预览目标 ≠ 当前 profile 时，与当前值不同的字段行高亮（语义色 + `→` 对比行），直接复用为 3.3 确认框的 diff 数据。
- Health Audit 每条问题按钮点击后：滚动定位到对应 profile 卡片并短暂高亮（而非只开编辑器）。
- Distribution 面板 `details/summary` 折叠，默认收起。

### 3.3 确认框升级（`useConfirmAction` / ConfirmModal 扩展）

- Apply 确认框：三行 diff 表格——base_url / model / auth_mode，「当前 → 目标」，相同行弱化，不同行强调；文案说明将同步更新当前配置。
- Delete 确认框：danger 样式保留，内联一行**与真实行为一致**的备份信息：删除/写入前 `write_guarded` 会把快照轮换写入 `~/.ccr/backups/{platform}/`（`BackupPolicy::Dir`，`crates/ccr-config/src/platforms/base.rs:545`）；当前**无 UI 恢复入口**，文案只陈述快照位置与手动恢复方式（将快照内容写回对应 profiles.toml），不得出现「从 Sync 页恢复」之类不实承诺（Sync 同步的是 `~/.ccr/platforms/`，与本地快照无关）。UI 内恢复入口属后端+产品范围扩展，明确列为非目标。
- 复用并遵守 `.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md` 与 `sync-security-contracts.md`（备份行为表述以其为准）。

### 3.4 ProfilesToolbar（筛选收敛）

- 裸露控件：搜索框、状态 pill 组（All/Current/Enabled/Disabled）、Filters 按钮、视图切换。Filters 按钮带生效数徽标。
- Filters 弹层（popover）：标签 pills、provider 下拉（仅 Claude）、排序下拉；底部「清除全部」。
- **弹层行为契约**（Filters popover 与 Header `···` 溢出菜单统一遵守）：
  - `Esc` 关闭并把焦点还给触发按钮；点击弹层外部关闭；选中项后弹层保持打开（多选场景），仅「清除全部」/外部点击/Esc 关闭。
  - 打开时焦点进入弹层第一个可操作项；`Tab` 在弹层内循环（focus trap 轻量版）或由 `Esc` 退出；方向键在选项间移动。
  - 窄窗口（<1280px 右栏隐藏后）：popover 以触发按钮锚定，右对齐防溢出视口；<720px 时退化为底部抽屉或全宽面板（跟随仓库既有 BasePopover/抽屉模式，若有）。
- 右栏 tag cloud 改为可点击 → 写入标签筛选，与 Filters 弹层同源（同一 `tagFilter` ref）。
- 修复 Codex 页缺失的 stale filter watch（标签消失时自动重置），与 Claude 页对齐。

### 3.5 主列表（卡片/列表统一）

- `ClaudeProfileRow` 与 `ProfileCard` 统一到 `--cp-*` token，废弃卡片上的 Tailwind 任意值字号与 per-provider 动态色（provider 以色点/小徽章弱表达，不再整卡染色）。
- 字段策略两平台一致：base_url 显示完整 host（过长仅截断路径）、model 走统一 fallback 链（见 §5）、缺失显示规范占位。
- 卡片主操作 = Apply（非当前且启用时）；编辑/删除收进卡片角落 `···` 菜单或 hover 显现的图标按钮（保留 aria-label）。
- 列表模式补齐 busyAction 反馈（Claude 页目前未传）。

### 3.6 编辑器模态统一

- Claude 页内联编辑器（视图内 ~370 行 `--editor-*` 样式）抽取为 `components/claude/ClaudeProfileEditorModal.vue`，与 Codex 的 props/emit 架构对齐。
- 两平台编辑器统一消费一套 token：迁移到 `--cp-*` / 全局 `--color-*`，删除平行 `--editor-*` 体系与 `!important`、硬编码 light RGBA；暗色覆盖块随之移除。
- 保留 4 段导航 + scroll-spy、ProviderTemplateSelector（遵守 `provider-template-contracts.md`）。
- 新增：保存前校验失败时在模态顶部显示汇总条并自动滚动到第一个错误字段。
- Codex 表单双源真相清理：`requires_openai_auth` / `openai_login_method` 不再存表单状态，保存时由 `auth_mode` 派生（`syncDerivedAuthFields` 逻辑移入 `buildCodexProfileRequest`）。**注意 `syncDerivedAuthFields` 的隐藏职责**：它还会在退出 `provider_env_key` 模式时清空 `env_key`（`CodexProfilesView.vue:790`），而当前 request builder 无条件序列化 `env_key`（`codexProfileEditor.ts:150`）——新契约：`env_key` 仅在 `auth_mode === 'provider_env_key'` 时序列化，其余模式一律置空；必须补模式切换回归测试。

### 3.7 StatStrip 同 schema 四槽

`Current` / `Total（enabled · disabled  hint）` / `平台特色槽` / `Health`。
- Claude 特色槽 = Auth split；Codex 特色槽 = Config mode。Health 槽 = 问题数（点击滚动到 Inspector Health 区）。
- 移除 `LAST WRITE` 客户端时钟槽（无持久化、重载即失真）；最近写入时间改为 Inspector 预览面板内一行元信息（仍由本地 mutation 维护，标注「本次会话」）。
- 移除从未被消费的 `totalSpark` / `recentSpark` sparkline 死代码。

## 4. 视觉契约

- 所有新/改样式只用 `--cp-*`（页内别名）与全局 `--color-*` token；字号一律 rem，落在 DESIGN.md 字阶（Label 0.8125rem 为密排下限，不再出现 10.5px/11px 任意值——密排元信息用 0.75rem 并记录为字阶扩展点，需同步 DESIGN.md 或在 spec 登记）。
- 圆角使用 6/8/12/16px 刻度。
- 陶土 accent：Add 按钮、活动 chip、Current 标记、diff 强调，占比 <10%。
- 暗/亮双主题同步验证；`prefers-reduced-motion` 保留。
- 可访问性：QuickRail 使用 `role="toolbar"` + roving tabindex；状态 pill 组 `aria-pressed`；Inspector 预览切换不打断屏幕阅读器（预览区 `aria-live="polite"` 仅报 profile 名）；apply 结果 toast 已有，补充 `role="status"`。
- 平台检测一律复用 `getClientPlatform()`（`ccr-ui/src/utils/windowChrome.ts:32`），不新写 `navigator.platform` 直读逻辑，便于测试 mock。

## 5. 共享逻辑修正

- **Model fallback 链去重**：`model → sonnet → opus → haiku → subagent`（Claude）与 base_url 空→`officialBaseUrl`（Codex）等解析逻辑各保留一份，收进 `utils/claudeProfiles.ts` / `utils/codexProfileEditor.ts`，row/rail/insights 全部引用同一函数。
- **死代码清理**：Claude 页 ~20 个未引用 i18n 键（`consoleEyebrow`、`quickSwitchStrip*`、`providerNav*` 等，清单见 research）、`codex.profiles.commandPalette.actionImport`、sparkline props、`ProfilesSection` 内联组件在两视图重复定义（提取到 `components/profiles/ProfilesSection.vue`）、`.cp-list-head` 重复 markup。
- **i18n**：新键入 `claudeProfiles.*` / `codex.profiles.*` 对称子树；触及组件内的 `translateWithFallback` 硬编码中文回退改为正常键。

## 6. 数据流不变量

- 不新增 Pinia store；视图本地 ref + composable 架构保留。钉选/最近列表也放本地 composable（`useProfilesQuickSwitch`）+ localStorage 持久化。
- API 调用面不变（`listClaudeProfiles` / `applyClaudeProfile` / Codex 同族）。
- 既有行为保留：apply/delete/rename 确认、raw TOML 编辑器（local 环境门槛 + 冲突处理）、TTL 刷新、`onActivated` 重载。

## 7. 回滚策略

组件层改动以**纯新增**为主（新文件 + 既有组件的可选新 prop，默认值保持旧行为），平台视图接入时切换到新 API；旧代码路径（旧 props 分支、旧槽位）在两个平台页都迁移完成后，由父任务集成步骤统一删除。若子任务 ②/③ 受阻，回滚视图接入即可，共享层新增物可无害留置。

## 8. 验证协议

### 8.1 截图走查协议（暗/亮双主题）

- **运行环境**：Tauri 桌面运行态为准（`just ui-dev` 或既有桌面启动方式）；web preview（`bun run dev`）仅用于样式级走查，且需在报告中注明 Tauri-only 数据缺失的局限。
- **Fixture**：使用评审截图同量级的真实本地配置（≥20 个 profile、含禁用项、含多标签、含缺失字段问题项），两平台各一份；不得用空状态截图充当验收。
- **路由与视口**：`/claude-code/profiles` 与 `/codex/profiles`，桌面视口 2543×1373（与现状截图一致）+ 1280px 边界视口（右栏显隐临界点）各一组。
- **产出**：每页暗/亮 × 两视口共 4 张，附在子任务完成报告中。

### 8.2 自动化用例（新增，放 `ccr-ui/tests/`）

- `useProfilesQuickSwitch`：持久化读写、stale 名称清理（删除/重命名/禁用）、钉选上限 8、recent 不参与编号。
- 修饰键提示：mock `getClientPlatform()` 分别返回 windows / macos，断言文案 `Ctrl` / `⌘`。
- QuickRail roving tabindex：方向键移动、Home/End、Tab 序不穿过全部 chip。
- Codex `env_key`：`buildCodexProfileRequest` 在 `provider_env_key` 模式序列化、其余模式置空的回归测试（现有 `codex-profiles-view.smoke.test.ts` 只覆盖 rename payload，不足以承接本重构）。
- `buildProfileDiff`：相同/相异/缺失值的行级输出。
