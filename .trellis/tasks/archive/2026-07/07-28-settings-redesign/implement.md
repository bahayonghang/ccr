# 设置系统重设计 — 执行计划

> 前置：`07-28-color-system-rebuild` 令牌与值域已就绪。测试与实现红→绿同步走。

## Step 1 — i18n 先行

1. `en-US.ts` / `zh-CN.ts`：新增 neutral/catppuccin flavor 键、重写 accent 描述、新增分段控件文案；删除旧 flavor/accent 键。
2. `bootMessages.ts` settings 副本同步（双 locale）。
3. 验证：`bun run test:i18n`；新增 bootMessages ↔ 语言包键集合一致性断言。

## Step 2 — 外观区重构

1. 主题三选项 → 分段控件（保留 `settings-theme-*` testid + 解析结果指示）。
2. flavor 选择器 → 3 项真实 token 预览卡（作用域覆写 mini 预览，注释标注与 tokens.css 同步）。
3. accent 选择器 → 4 项实心按钮预览。
4. 字体卡样式接入新契约（行为不动）。
5. 验证：`app-settings.smoke.test.ts` 先改断言（红）→ 实现（绿）。

## Step 3 — 页面其余部分

1. Hero 摘要 pill 行重排；section 导航选中态重设计。
2. language / shell / diagnostics 三 Card 样式接入新契约。
3. `MainLayout.vue` dock 摘要映射与样式更新 → `main-layout-theme-stage.smoke.test.ts` 同步。

## Step 4 — 全量验证

1. `cd ccr-ui && bun run type-check && bun run lint && bun run test:i18n`。
2. `bunx vitest run --config vitest.smoke.config.ts tests/app-settings.smoke.test.ts tests/main-layout-theme-stage.smoke.test.ts tests/theme-bootstrap.smoke.test.ts`。
3. 视觉核验：light/dark × 3 flavor 的 /settings 截图矩阵（6 张），证据存父任务 research/。

## 回滚点

- Step 1（i18n）独立可还原；Step 2/3 同文件按 hunk 还原；测试与实现同批还原。

## Review gates

- Step 2 后：预览卡在三 flavor 下视觉确认再继续。
- Step 4 后：向父任务回报 AC 证据。
