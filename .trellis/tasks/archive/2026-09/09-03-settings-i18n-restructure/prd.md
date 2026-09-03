# 设置页重构与全局中文化

## Goal

修复中英文切换失效（两个独立根因），落地"全局中文化"决策，并按新视觉世界重构设置页（全局设置）的布局与排版。

## Requirements

1. **stale memo 修复**：`features/configs/hooks/useAppSettings.ts:181-184` 的 `sections` 空依赖 `useMemo` 导致切换语言后栏目卡文案不更新。改为语言感知（`useAppT()` + locale 依赖，或去 memo）。
2. **全局中文化**（用户已拍板）：zh-CN 语言包中全部约 29 处英文 eyebrow 翻译为中文——`settings.eyebrow`（`zh-CN.ts:2501`）、`Appearance/Theme/Flavor/Typography/Language/Shell/Diagnostics`（L2515-2581）、`settings.summary.runtimeDesktop/runtimeWeb`（L2511-2512），以及全 app 其余 eyebrow（`zh-CN.ts` L1236、L1635、L1656、L1672、L1690、L3346、L4707-5163、L5224 等）与零散英文标签（`Base URL` L1007、`Warning` 等）。完整清单实施时以扫描为准。
3. **设置页重构**：hero 卡（eyebrow + meta chips）、左侧栏目卡列、右侧 Theme/Flavor/Typography 面板的布局与排版按方向契约重做；Flavor 预览卡（中性/暖陶）的预览缩略要真实反映新 token。
4. **回归测试**：新增 (a) zh-CN 值中文断言（至少覆盖 settings 域 + 全部 eyebrow key），(b) 设置页 live 语言切换 smoke test（切换后栏目卡/eyebrow 无英文残留）。现有门禁（`i18n.test.cjs`、`check-i18n.mjs` 的 4404 key 计数）保持绿色并同步计数。
5. **spec 沉淀**：把"组件内禁止对 `t()` 结果做空依赖 memo；统一 `useAppT()`"写入 `.trellis/spec/ccr-ui/frontend/react-rerender-discipline.md` 或新建 i18n 契约文件，并登记到 `frontend/index.md`。

## Acceptance Criteria

- [ ] zh-CN 模式下设置页零英文残留（含 hero eyebrow、meta chips、栏目卡、各面板）
- [ ] 语言切换后全页面文案实时更新，无需重挂载；zh ↔ en 往返无陈旧文案
- [ ] 全局 29 处 eyebrow 完成中文化，`check-i18n.mjs` key 对齐与计数保持绿色
- [ ] 新增两类回归测试并通过；`bun run type-check|lint|test|build` 全绿
- [ ] 设置页按方向契约完成视觉重构，四主题组合下观感一致
- [ ] spec 更新落地并被 `frontend/index.md` 引用

## Dependencies / Ordering

- 视觉层依赖 `09-03-theme-token-world` 的 token；i18n 修复（1/2/4/5）可独立先行。

## Notes

- 分析：`../09-03-ui-visual-world-replacement/research/settings-i18n-analysis.md`
- 关键文件：`features/configs/AppSettingsView.tsx`、`features/configs/settings/*.tsx`、`features/configs/hooks/useAppSettings.ts`、`features/configs/lib/settingsModel.ts`、`i18n/locales/{zh-CN,en-US}.ts`、`i18n/bootMessages.ts`、`styles/app-settings.css`
