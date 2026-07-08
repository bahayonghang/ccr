# 执行计划:Claude/Codex Profiles 交互与视觉统一

## Checklist

1. [x] 抽 `useConfirmAction.ts`,Codex 页迁移到它(行为不变)。
   - 验证:Codex apply/delete 手测 + type-check。✅ Codex 已改用 `useConfirmAction` 解构 `isOpen/dialog/busy/openConfirmDialog/executeConfirmedAction`,行为未变(handleProfileAction/handleSave 仍走同一 confirm 流程)。
2. [x] Claude 页移除 5 处 confirm/alert,接入 useConfirmAction + toast。
   - 验证:`rg "\\b(confirm|alert)\\(" src/views/ClaudeCodeProfilesView.vue` 零命中(已跑,确认)。apply/delete/rename 均已改走 `openConfirmDialog` + `uiStore.showError`。三流程手测受限于 web 预览无真实 profile 数据未能人工点击,但 smoke test 新增 `opens a confirm dialog with interpolated copy and only applies after confirmation` 用例覆盖 apply 路径(点击 footer 确认按钮后才调用 API),已通过。
3. [x] CommandPalette / QuickRail 提升为 profiles/* 泛型组件,Codex 改引用。
   - 验证:`components/codex/CommandPalette.vue`/`ProfilesQuickRail.vue` 已删除,`rg "components/codex/CommandPalette|components/codex/ProfilesQuickRail"` 零命中(无残留引用);Codex 页改引用 `profiles/ProfilesCommandPalette.vue` + `profiles/ProfilesQuickRail.vue`,smoke test 已同步改造并通过。
4. [x] 抽 `useProfilesHotkeys.ts`,两页接入;Claude 页接 CommandPalette + QuickRail。
   - 验证:两页 script 均调用 `useProfilesHotkeys({ paletteOpen, focusSearch, getApplicableProfiles, onApply })`;浏览器内以 `window.dispatchEvent(KeyboardEvent('k', ctrlKey))` 手测 Claude 页,确认 ⌘K 能正确打开命令面板并聚焦搜索框。⌘1-9/Esc 的端到端人工验证受预览环境限制未完成(见收尾说明),但逻辑与旧实现等价迁移,smoke test 覆盖面板开合。
5. [x] 删除假 spark 入参 + StatStrip 无数据不渲染趋势;lastWrite 改 markWrite()。
   - 验证:`rg "total-spark" ccr-ui/src/views` 零命中(已跑,确认)。`ProfilesStatStrip.vue` 本身已有 `v-if="totalSpark"`/`v-if="recentSpark"` 守卫,无需改动。两页 `loadProfiles`/`ensureLoaded` 不再写 `lastWriteHint`,改为 `markWrite()` 仅在 save/apply/delete 成功回调后调用。浏览器验证:Claude/Codex 页 stat strip 均未见 spark 残留、"最近写入" 初始为 "—"。
6. [x] 栅格多列 + 卡片密度收紧 + accent 统一。
   - 验证:两页 `.cp-grid` 改 `repeat(auto-fill, minmax(420px, 1fr))` + `<1280px` 回退单列;`--cp-accent` 统一指向 `--color-accent-primary`,浏览器 `getComputedStyle` 核实 Claude 页 `--cp-accent === --color-accent-primary === #d97757`,平台识别色拆到独立的 `--cp-icon-color`。1920px 多列截图未能采集(见收尾说明),数据依赖 Tauri 后端在纯 web 预览下为空,无法渲染出多卡片网格做像素级核对。
6b. [x] 卡片信息设计(design.md §4b):Codex 假 input 改键值行、当前置顶、紧凑应用按钮、URL 中段省略、0 值分布隐藏、页头命名统一。
   - 验证:`ProfileCard.vue` 字段区已是 `.cp-field__value` 纯文本行(input 视觉的 padding/background/border 已删除);`useProfilesFilter.ts` 新增 `pinCurrent()` 将当前 profile 前移;`ClaudeProfileRow.vue` 应用按钮改 `h-7`/`text-xs` 紧凑尺寸,Codex `ProfileCard.vue` 应用按钮原本已是紧凑 icon-btn;`utils/text.ts` 的 `truncateMiddle` 已接入 `ProfileCard`/`ClaudeProfileRow`/`ProfileListRow` 三处 base_url 展示;`ProfilesContextRail.vue` 新增 `visibleAuthModeBreakdown` 过滤 0 值;两页 i18n 页头统一为 "<平台> Profiles 管理"(zh/en 均已核对)。截图级逐项核销未完成,已用 rg/DOM 结构核对替代。
7. [x] 编辑 modal:IntersectionObserver 分区同步 + 高级字段渐进披露 + floating 玻璃外壳(两平台)。
   - 验证:`ClaudeCodeProfilesView.vue` 的 `setupSectionObserver` 已替换旧 `@scroll` 逐帧计算;`ClaudeProfileEditorSections.vue` 新增 `advancedExpanded` 折叠高级模型映射/timeout/auto-compact/traffic/effort 字段组(`ADVANCED_FIELD_KEYS` 覆盖全部 15 个字段,均在同一个 `v-show` 容器内),编辑已有值时自动展开;Claude/Codex 两个编辑 modal 外壳均改用 `--material-glass-floating-*` 令牌。Performance 录制未做(见收尾说明),但逐帧 `@scroll` 监听已确认从代码中移除。
8. [x] i18n 补齐新增文案(zh/en);`bun run type-check && bun run lint`。
   - 验证:`just frontend-check-quick` 全绿 —— type-check ✅、eslint+stylelint ✅、i18n 一致性测试 23/23 ✅(zh/en namespace、占位符、覆盖率均对齐)。
9. [x] `bun run test:smoke` (全量,含 provider-templates + 主题 smoke)。
   - 验证:`just frontend-check-quick` 内 `vitest run --config vitest.smoke.config.ts`:81 files / 364 tests 全部通过。
10. [~] 亮/暗两页并排截图入 research/;review gate。
   - 未完成:本机 `preview_screenshot` 在当前会话中持续超时(诊断出该预览标签页 `document.hidden === true`,推测是环境级的标签可见性问题,截图/部分键盘模拟因此不可靠,与本次代码改动无关);且纯 web 预览无 Tauri IPC,`list_claude_profiles`/`list_codex_profiles` 返回空,无法渲染出有数据的卡片网格用于截图核对多列布局。
   - 已替代验证:用 `preview_eval`/`preview_snapshot`/`preview_inspect` 读取 DOM 结构、accessibility tree、`getComputedStyle`,确认两页标题、命令面板文案、accent token、空状态结构均符合预期;⌘K 打开命令面板的窗口级快捷键在浏览器中人工触发成功。
   - 遗留给用户/后续:如需真正的亮暗并排像素级截图与真实数据下的多列网格/⌘1-9/Esc/编辑 modal 折叠动画核对,建议在桌面 `just tauri-dev`(有真实 profiles.toml 数据)下人工过一遍,或换一个未被系统判定为隐藏标签的预览会话重试截图。review gate 待用户决定是否在此基础上放行。

## Rollback

按 design.md §7 的 5 个 commit 独立 revert。
