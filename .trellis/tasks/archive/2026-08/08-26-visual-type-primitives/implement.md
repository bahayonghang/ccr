# 原语落地 — 执行

## 清单

1. 在 `primitives.css` 写 `.ui-btn` / `.ui-badge` / `.ui-field-label` / `.ui-url-text` 与 reduced-motion。
2. 实现四个 tsx 与 `buttonClass`，从 `index.ts` 导出。
3. EmptyState、ConfirmModal 改用 `Button`。
4. 写 `tests/ui/action-visual-types.smoke.test.tsx`（变体 token、同高、三种 ConfirmModal type、UrlText 超长 pathname 与 `not-a-url`、FieldLabel 三项、`.ui-*` 无 hex/px）。
5. 跑 focused smoke + `bun run check:arch-boundaries` + `just frontend-check-quick`。

## 验证

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/ui/action-visual-types.smoke.test.tsx
cd ccr-ui && bun run check:arch-boundaries
just frontend-check-quick
```

## 回滚

只涉及 `src/ui/**` 与 `tests/ui/**`。revert 该提交即可。
