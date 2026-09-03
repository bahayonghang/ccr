# 主题 Token 体系与配色世界替换

## Goal

在 `tokens.css` 单一权威内落地新视觉世界的配色/token 体系（方向以方向契约为准），完成中性深色暖化、平台色确权、启动 Loader 修色，并在建成后重写 `ccr-ui/DESIGN.md`。这是其他三个子任务的地基。

## Requirements

1. **深色暖化**：中性深色族（现 `#131316/#1a1b1f/#22242a/#2c2f37`，`tokens.css:161-170`）向暖棕微调，与 clay 深色（`#17120f` 系）及暖强调色同源；浅色中性族保持阅读面属性。
2. **新方向配色落地**：按方向契约替换 surface/accent/semantic/chart 各组 token 值；保留 `data-theme/data-flavor/data-accent` 属性模型与 `index.html` 首帧 IIFE 的 key 兼容（`ccr-theme`/`ccr-flavor`/`ccr-accent`）。
3. **平台色确权**：Antigravity 启用自己的 `--color-platform-antigravity`（当前全 `src/` 无人消费，处处映射到 Gemini 蓝：`dashboard-usage-movement.css:123-125,200-202`、`dashboard-platform-matrix.css:51-52`、`DashboardView.tsx:164`、`MainLayoutNav.tsx:15`）；平台色只作身份识别，不重涂整屏。
4. **启动 Loader 修色**：`index.html:60-62` 的 `#app-loader` 深色底 `#000000` 与蓝色 `#2997ff` 转圈改为与新世界 token 一致的值，消除深色启动闪色。
5. **死代码清理**：`applyCustomAccent/clearCustomAccent`（`themeBootstrap.ts:421-453`，无调用方）与 `data-resolved-flavor` 残留（`themeBootstrap.ts:67-72`）按 spec 评审决定去留并执行。
6. **DESIGN.md 重写**：本任务只更新 `theme-token-contracts.md` 等相关 spec；`ccr-ui/DESIGN.md` 与 `ccr-ui/AGENTS.md` 的 Aesthetic Direction 重写**移至父任务收尾步骤**（ impeccable 规则：规则书从建成后的世界书写，需等首页/设置页/设置坞全部落地后由 documenter 子代理执行）。

## Acceptance Criteria

- [ ] 四组合（light/dark × neutral/clay）截图对比评审通过；中性深色与 clay 深色并置气质一致
- [ ] Antigravity 在导航色样、平台卡、图表段、图例四处使用同一平台色，不再是 Gemini 蓝
- [ ] 深色启动无黑→近黑闪烁，Loader 配色来自 token
- [ ] `bun run type-check`、`bun run lint`、`bun run test`、`bun run build`、`bun run tauri:check` 全绿
- [ ] `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md` 按新体系更新（DESIGN.md / AGENTS.md 重写归父任务收尾）

## Dependencies / Ordering

- 无上游依赖；**必须先于** overview-home-restructure、settings-i18n-restructure、settings-dock-polish 三个子任务完成 token 落地（它们消费新 token）。

## Notes

- 分析：`../09-03-ui-visual-world-replacement/research/theme-shell-analysis.md`
- 方向契约：impeccable surface brief（方向轮锁定后写入），seed key `19fe1fa0`
