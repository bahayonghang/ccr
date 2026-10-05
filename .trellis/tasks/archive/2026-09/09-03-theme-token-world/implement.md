# Implement — 主题 Token 体系与配色世界替换

> 执行前必读：`prd.md`、`design.md`（值映射表是唯一真源）、`research/theme-shell-analysis.md`、`.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`、`ccr-ui/.impeccable/surfaces/ui-src-features-usage-dashboard-dashboardview-tsx.md`（方向契约）。
> 铁律：对比阈值不动，只调色值；`@theme inline` 不写字面量；flavor 块不得设置 `--color-accent-primary` / `--color-border-accent`；IIFE 迁移表不动。

## 执行清单

### Step 1 — 中性深色暖化（dark 块）
- [ ] `ccr-ui/src/styles/tokens.css` `[data-theme='dark']` 块：按 design.md §1 替换背景坡道、文字五级、边框三级、功能色 tint。
- [ ] 验证：`bunx vitest run --config vitest.smoke.config.ts tests/theme/theme-contrast-contract.smoke.test.ts`；不达标只调值。

### Step 2 — 浅色中性暖化（:root 块）
- [ ] 按 design.md §7 替换 `:root` 背景/文字/边框。
- [ ] 验证：同上对比门禁（四组合全过）。

### Step 3 — 琥珀强调色
- [ ] 按 design.md §2 同步四处定义点（`:root`、`[data-theme='dark']`、`[data-accent='clay']`、`[data-theme='dark'][data-accent='clay']`），含 `-rgb`/`-glow`/`-contrast`/`--color-border-accent`。
- [ ] 验证：对比门禁 + `theme-switch.smoke.test.tsx`（锚点值需更新为新 hex，阈值不动）。

### Step 4 — 功能色对齐
- [ ] dark 块 success/danger/warning/info 及 `-contrast` 按 design.md §3；light 块 warning 微调。
- [ ] 验证：对比门禁。

### Step 5 — Antigravity 平台色确权（消费端）
- [ ] `features/usage/styles/dashboard-usage-movement.css`、`dashboard-platform-matrix.css`、`DashboardView.tsx`、`shell/MainLayoutNav.tsx` 四处从 gemini 改指 `--color-platform-antigravity`（或对应 utility）。
- [ ] `styles/chart-colors.css` ramp 按 design.md §5 同步新功能色。
- [ ] 验证：`rg "platform-gemini" ccr-ui/src` 仅剩 Gemini 平台自身消费点；`bun run type-check`。

### Step 6 — 启动 Loader 修色
- [ ] `ccr-ui/index.html:60-62`：dark 底 `#000000`→`#100f0c`，spinner `#2997ff`→`#f0a32b`；light 底色对齐 `#e9e4d8`。
- [ ] 验证：肉眼确认 `bun run dev:web` 深色启动无闪色。

### Step 7 — spec 与锚点收尾
- [ ] 更新 `theme-token-contracts.md`：新 palette 叙述 + `applyCustomAccent`/`data-resolved-flavor` 的 vestigial 注记（契约条款与阈值不动）。
- [ ] 全量验证：`cd ccr-ui && bun run type-check && bun run lint && bun run test && bun run build && bun run tauri:check`。
- [ ] 视觉验收：`bun run dev:web -- --host 127.0.0.1 --strictPort`，四组合（light/dark × neutral/clay）截图评审。

## 回滚点

- 每个 Step 独立可 revert；Step 1-4 是纯值替换，revert 即恢复。
- 若 Step 3 琥珀在大量消费端出现意外（如 clay 橙色被当作 Claude 平台语义使用），停在该 Step 上报，不要自行扩大改动面。

## Review gates

- Step 4 完成后：四组合对比门禁必须全绿，才允许进 Step 5。
- 全部完成后：父任务验收标准第 1/2/3 条（四组合评审、Antigravity 确权、Loader 无闪色）。
