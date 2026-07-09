# codex-auth-shared.css 语义令牌迁移

## Goal

把 `ccr-ui/src/styles/codex-auth-shared.css`(658 行)中的硬编码表面/边框/文字色迁移到语义令牌,让 Codex/Claude Auth 相关页面不再游离于主题系统(theme/flavor/accent 三层)之外。

本任务自 07-07-ui-consistency-sweep 拆出(2026-07-09 决策):sweep 的 R2-2 只做 Auth 页面结构内的交互与表面对齐,**不动本文件**;两任务互不阻塞。

## Requirements

- 硬编码 hex/rgba 表面色、边框色、文字色迁移到 `tokens.css` 既有语义令牌(`--color-*` / `--surface-*` 系列);确属装饰性的保留原值并加中文注释说明。
- 遵守 theme-token-contracts:只改消费侧,不改共享语义别名定义;若需新增别名,加在 tokens.css Surface Contract 块并保持 flavor 独立性。
- 不引入新的 `backdrop-filter`(玻璃预算 ≤3,Auth 页非玻璃层级)。
- 不改页面布局与选择器结构,纯色值来源替换。

## Out of Scope

- Auth 页面 Vue 组件的交互改造(原生 confirm 清除、空态等,归 consistency-sweep)。
- tokens.css 令牌体系本身的增删重构。

## Acceptance Criteria

- [ ] `rg "#[0-9a-fA-F]{3,8}\b|rgba?\(" ccr-ui/src/styles/codex-auth-shared.css` 仅剩已注释登记的装饰性命中。
- [ ] 亮/暗主题 × 默认 clay flavor 下 Codex Auth、Claude Auth 页截图对比,可读性与层级不回退。
- [ ] `cd ccr-ui && bun run type-check && bun run lint` 通过。
- [ ] 主题 smoke 通过:`cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/apple-glass-surface-contract.smoke.test.ts tests/theme-bootstrap.smoke.test.ts tests/app-settings.smoke.test.ts`。

## Notes

- 轻量任务,PRD-only 即可;执行前读 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md`。

## Dependencies

- 07-07-ui-glass-tokens(语义令牌体系,已完成)。
- 与 07-07-ui-consistency-sweep 的 R2-2 并行安全:sweep 承诺不改本文件。
