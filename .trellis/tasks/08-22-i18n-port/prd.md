# i18n 运行时迁移到 react-i18next

> 父任务：`08-22-react-migration`

## Goal

将 i18n 运行时从 `vue-i18n` 迁到 react-i18next，4,164 个叶子 key 的词条数据原样复用，调用点完成转换。

## Scope

| 文件 | 体积 / 数量 |
|---|---|
| `src/i18n/locales/zh-CN.ts` | 204.8 KB |
| `src/i18n/locales/en-US.ts` | 212.9 KB |
| `src/i18n/locales/zh-CN.keys.txt` | 4,261 行（**源码行数，不是 key 数**）|
| `src/i18n/locales/en-US.keys.txt` | 4,402 行（校验用，同上）|
| 两个 locale 的叶子 key 数 | 各 4,164（递归展开 `zh-CN.ts` / `en-US.ts` 实测）|
| `src/i18n/bootMessages.ts` | 45.6 KB |
| `src/i18n/index.ts` | 3.7 KB |
| `src/i18n/formatMessage.ts` | 1.1 KB |
| `scripts/check-i18n.mjs` | 校验脚本 |
| `tests/i18n.test.cjs` | 校验测试 |

**可移植性结论**：词条未使用 vue-i18n 的 linked message（`@:`），未使用复数管道（`|`）。词条数据可直接移植，仅运行时与调用点需要转换。

## Requirements

- R1 `vue-i18n` 从 `package.json` 移除，替换为 `react-i18next` 17.0.12 + `i18next` 26.4.0（选型见父任务 `design.md` §1）。
- R2 `zh-CN.ts` 与 `en-US.ts` 的词条内容不做改动。若格式需调整（如插值语法差异），改动逐项登记并说明。
- R3 4,164 个叶子 key 的调用点全部转换。调用点转换在各 `08-22-views-*` 子任务内随视图迁移同步进行，本任务负责运行时切换、遗漏排查与收尾校验。
- R4 `bootMessages.ts`（45.6 KB）的启动期文案在 i18n 运行时初始化之前可用，行为不变。
- R5 `formatMessage.ts` 的格式化行为在新运行时下等价。
- R6 语言切换在运行时生效，无需刷新页面。
- R7 语言偏好持久化行为不变，刷新后保留。
- R8 `scripts/check-i18n.mjs` 与 `tests/i18n.test.cjs` 适配新运行时，继续校验两个 locale 的 key 集合一致。
- R9 `@intlify/eslint-plugin-vue-i18n` 移除后，未使用 key 与缺失 key 的静态检查能力不下降。等价方案在 `design.md` 中给出。

## Acceptance Criteria

- [ ] AC1 `rg 'vue-i18n' ccr-ui/package.json ccr-ui/src` 无匹配。
- [ ] AC2 `zh-CN.ts` 与 `en-US.ts` 的 git diff 为空，或改动项逐条有登记说明。
- [ ] AC3 `bun run check:i18n` 退出码 0。
- [ ] AC4 `bun run test:i18n` 退出码 0，两个 locale 的叶子 key 集合一致，均为 4,164 个。
- [ ] AC5 全仓无残留的 `$t(` / `useI18n()` 等 vue-i18n 调用形式。
- [ ] AC6 在应用内切换中英文，全部 75 条路由的页面文案正确切换，无 key 原文泄漏（形如 `views.foo.bar` 的字符串出现在界面上）。
- [ ] AC7 刷新页面后语言偏好保留。
- [ ] AC8 启动期文案（`bootMessages`）在 i18n 初始化前正确显示。
- [ ] AC9 未使用 key 与缺失 key 的静态检查可运行并通过。
- [ ] AC10 `bun run type-check` 与 `bun run lint` 退出码 0。

## 前置与后续

- 前置：`08-22-shell-port`（运行时接入）。调用点转换与 `08-22-views-*` 七个子任务并行进行。
- 后续：`08-22-test-contract-rebuild`。

## Out of Scope

- 新增语言。当前只有 zh-CN 与 en-US 两个 locale。
- 词条文案的改写与润色。
- 新增 i18n key。
- 词条拆包与按路由懒加载。若 `design.md` 判定有必要，另开任务。

## Notes

- 调用点分布在 7 个视图子任务的全部批次中。本任务需与七个并行子任务约定统一的调用形式，避免出现多种写法。该约定需在本任务的 `design.md` 中先行落定并通知各子任务。
- key 原文泄漏（界面显示 `views.foo.bar`）是本次迁移最可能出现的回归形式，AC6 的逐路由检查不可省略。
- 词条文件合计 417.7 KB，是前端最大的单类资产。迁移时避免任何批量替换操作，防止大面积静默损坏。
