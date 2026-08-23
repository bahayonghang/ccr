# 明暗对比度与取值一致性核对（AC8 证据，批次 8）

- 基准：`git show 98b08252:ccr-ui/src/styles/tokens.css`（批次 1 之前，未受设计体系任务影响）。
- 对比范围：两版本中取值含颜色字面量的全部变量（基线 77 个 / 现行 77 个）。
- 取值差异：0 个；仅存在于基线：0 个；仅存在于现行：0 个。

## WCAG 对比度（四组合 × 契约色对，阈值同 theme-contrast-contract）

| 组合 | 色对 | 对比度 | 阈值 | 判定 |
| --- | --- | --- | --- | --- |
| light/neutral | `--color-text-primary` vs `--color-bg-surface` | 13.11:1 | ≥12.0:1 | PASS |
| light/neutral | `--color-text-secondary` vs `--color-bg-surface` | 9.84:1 | ≥7.0:1 | PASS |
| light/neutral | `--color-text-muted` vs `--color-bg-surface` | 6.65:1 | ≥4.5:1 | PASS |
| light/neutral | `--color-accent-primary` vs `--color-accent-primary-contrast` | 6.86:1 | ≥3.5:1 | PASS |
| light/clay | `--color-text-primary` vs `--color-bg-surface` | 13.11:1 | ≥12.0:1 | PASS |
| light/clay | `--color-text-secondary` vs `--color-bg-surface` | 9.84:1 | ≥7.0:1 | PASS |
| light/clay | `--color-text-muted` vs `--color-bg-surface` | 6.65:1 | ≥4.5:1 | PASS |
| light/clay | `--color-accent-primary` vs `--color-accent-primary-contrast` | 6.86:1 | ≥3.5:1 | PASS |
| dark/neutral | `--color-text-primary` vs `--color-bg-surface` | 13.11:1 | ≥12.0:1 | PASS |
| dark/neutral | `--color-text-secondary` vs `--color-bg-surface` | 9.84:1 | ≥7.0:1 | PASS |
| dark/neutral | `--color-text-muted` vs `--color-bg-surface` | 6.65:1 | ≥4.5:1 | PASS |
| dark/neutral | `--color-accent-primary` vs `--color-accent-primary-contrast` | 6.86:1 | ≥3.5:1 | PASS |
| dark/clay | `--color-text-primary` vs `--color-bg-surface` | 13.11:1 | ≥12.0:1 | PASS |
| dark/clay | `--color-text-secondary` vs `--color-bg-surface` | 9.84:1 | ≥7.0:1 | PASS |
| dark/clay | `--color-text-muted` vs `--color-bg-surface` | 6.65:1 | ≥4.5:1 | PASS |
| dark/clay | `--color-accent-primary` vs `--color-accent-primary-contrast` | 6.86:1 | ≥3.5:1 | PASS |

**结论**：四组合全部达标；取值与迁移前一致。
