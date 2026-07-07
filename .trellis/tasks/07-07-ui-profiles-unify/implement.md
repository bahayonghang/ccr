# 执行计划:Claude/Codex Profiles 交互与视觉统一

## Checklist

1. [ ] 抽 `useConfirmAction.ts`,Codex 页迁移到它(行为不变)。
   - 验证:Codex apply/delete 手测 + type-check。
2. [ ] Claude 页移除 5 处 confirm/alert,接入 useConfirmAction + toast。
   - 验证:`rg "\\b(confirm|alert)\\(" src/views/ClaudeCodeProfilesView.vue` 零命中;apply/delete/rename 三流程手测。
3. [ ] CommandPalette / QuickRail 提升为 profiles/* 泛型组件,Codex 改引用。
   - 验证:Codex 页 ⌘K 面板与 QuickRail 行为不回归。
4. [ ] 抽 `useProfilesHotkeys.ts`,两页接入;Claude 页接 CommandPalette + QuickRail。
   - 验证:两页快捷键矩阵手测记录(⌘K//、⌘1-9、Esc)。
5. [ ] 删除假 spark 入参 + StatStrip 无数据不渲染趋势;lastWrite 改 markWrite()。
   - 验证:`rg "total-spark" src/views` 零命中;写操作后 hint 才更新。
6. [ ] 栅格多列 + 卡片密度收紧 + accent 统一。
   - 验证:1920px 截图 ≥2 列;data-accent 切换两页同步变色。
6b. [ ] 卡片信息设计(design.md §4b):Codex 假 input 改键值行、当前置顶、紧凑应用按钮、URL 中段省略、0 值分布隐藏、页头命名统一。
   - 验证:截图对照 P9-P13 逐项核销;键盘操作应用流程可达。
7. [ ] 编辑 modal:IntersectionObserver 分区同步 + 高级字段渐进披露 + floating 玻璃外壳(两平台)。
   - 验证:滚动无逐帧 JS(Performance 录制);新建/编辑两场景折叠状态正确。
8. [ ] i18n 补齐新增文案(zh/en);`bun run type-check && bun run lint`。
9. [ ] `bun run test:smoke -- tests/provider-templates.smoke.test.ts` + 主题 smoke。
10. [ ] 亮/暗两页并排截图入 research/;review gate。

## Rollback

按 design.md §7 的 5 个 commit 独立 revert。
