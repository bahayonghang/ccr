# 配色系统重构 — 执行计划

> 每步完成后跑对应验证命令；失败即回滚该步再继续。全程遵循 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`。

## Step 1 — 迁移逻辑与类型（TS 先行）

1. `themeBootstrap.ts`：`FlavorMode`/`AccentMode` 收窄；加 `FLAVOR_MIGRATION` / `ACCENT_MIGRATION`；`readStoredFlavor`/`readStoredAccent` 走迁移+白名单；`resolveFlavorMode` 支持 `catppuccin → latte|mocha`；`DEFAULT_FLAVOR='neutral'`。
2. `shellPreferences.ts`：`initializeTheme` 检测存储值 ≠ 迁移值时写回 localStorage。
3. `index.html` 首帧 IIFE：内联同一迁移表与解析（无 import）；保持"任何 CSS 加载前注水"。
4. 验证：`bunx vitest run --config vitest.smoke.config.ts tests/theme-bootstrap.smoke.test.ts`（先改测试锁定新行为，再实现，红→绿）。

## Step 2 — tokens.css 重建（主战场）

1. `:root` 写 neutral light 全套；`[data-theme='dark']` 写 neutral dark 全套（按 design.md §2 锚点）。
2. 重写 clay flavor 明暗两块（同几何）。
3. 删 paper/graphite 块、frappe/macchiato 调色板块；Catppuccin 共享重映射块拆成 `[data-resolved-flavor='latte']` / `html:root[data-resolved-flavor='mocha']` 两块完整重映射（修复 latte elevation 反转）。
4. stage 语义层全量改不透明/实心；删 `--stage-bg-mesh/aurora/orb/grid/noise-*` 死令牌；删 `premium-pink/blue`。
5. accent 块重写为 4 个（含 dark 变体 + `*-contrast` 令牌 + Catppuccin 作用域映射）。
6. 玻璃/表面契约按 design.md §3 重指；`prefers-reduced-transparency` 块重写（含 mocha 同级重置）。
7. `theme.css` bridge 审计：引用的令牌名全部仍存在；新增 `accent-primary-contrast` 桥接。
8. 验证：`bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts`。

## Step 3 — 氛围层与全局样式

1. `StageBackground.vue` / `AnimatedBackground.vue` 收敛为不透明基底。
2. `backgrounds.css` 死代码删除 + `deferred-decorations.css` / `base.css` 引用清理。
3. `base.css` 暗色字体平滑改 `auto`。
4. 验证：`rg "premium-bg-orb|premium-background|stage-bg-(mesh|aurora|orb|grid)" ccr-ui/src` 无结果；`bun run build` 通过。

## Step 4 — 确定性 bug

1. `PlatformMcpView.vue` / `PlatformPluginsView.vue`：JS 白背景 → CSS hover 类。
2. `OutputStylesView.vue`：`bg-white` → 语义类/暗色守卫。
3. 验证：rg 证据 + 手动暗色 hover 核验。

## Step 5 — 新守卫测试

1. 新增 `tests/theme-contrast-contract.smoke.test.ts`（design.md §7 静态解析方案）。
2. 红→绿：先对新令牌跑，不达标的锚点值微调（只允许改 tokens.css 值，不许降阈值）。
3. 验证：6 组组合全绿。

## Step 6 — 全量回归 + 视觉核验

1. `cd ccr-ui && bun run type-check && bun run lint && bun run test:i18n`。
2. 全量 smoke：`bun run test:smoke`（资源预算见 development-resource-contracts）。
3. Playwright 手动核验：dark+neutral 的 Overview / Profiles / Settings 截图，与诊断前对比（无雾、边界清晰、文本实心）；证据存 `../07-28-ccr-ui-visual-redesign/research/`。
4. i18n：新增 `neutral`/`catppuccin` 显示名键（双 locale + `bootMessages.ts` 副本同步）；旧键保留。

## 回滚点

- Step 2 之前：`git checkout -- ccr-ui/src/utils/themeBootstrap.ts ccr-ui/index.html` 可还原迁移。
- Step 2 为单文件主战场：tokens.css 整体可还原。
- Step 3-4 互相独立，可单独还原。

## Review gates

- Step 1 后：迁移表与值域经自检（单测红→绿）。
- Step 2 后：对比度测试先行暴露不达标锚点。
- Step 6 后：进入子任务 B / C 的并行实施。
