# 执行计划：i18n 运行时迁移到 react-i18next

> 父任务：`08-22-react-migration`（阶段 5，与七个视图子任务并行）。
> 分支：`feature/react-migration/i18n-port`，PR 目标 `feature/react-migration`。
>
> 调用形式约定（批次 1）必须在七个视图子任务动手前落定，否则各子任务自选写法，收尾需返工。

## 前置确认

- [ ] `08-22-shell-port` 已交付（运行时接入需要外壳）。
- [ ] `08-22-dep-upgrade` 段 1 已装 `react-i18next` 17.0.12 + `i18next` 26.4.0，已移除 `vue-i18n` 与其 vite alias。
- [ ] `git checkout -b feature/react-migration/i18n-port feature/react-migration`

## 批次 0：前置核对

四项核对决定后续实现形态，先做。

- [ ] 词条中的花括号内容全量抽出（`rg -o '\{[^}]*\}' src/i18n/locales/*.ts`），确认每处都是插值变量名。字面量花括号逐条记录。
- [ ] `formatMessage.ts`（1.1 KB）的职责确认：纯包装还是含自定义格式化。
- [ ] 语言偏好的存储键与读写位置确认。
- [ ] 动态 key（`t(someVar)` 形态）统计。存在则准备白名单。

结论落盘为 `i18n-probe.md`。

## 批次 1：调用形式约定（最高优先级）

- [ ] 按 `design.md` §2 落定约定：`useTranslation` 的 `t()`、组件外用 `i18n.t()`。
- [ ] 明确禁止项：`<Trans>`、命名空间拆分、`withTranslation`。
- [ ] `i18n-call-convention.md` 落盘。
- [ ] 通知七个视图子任务（协同点 I）。

该批次不含代码改动，但是阶段 4 → 5 门的实际前置。

## 批次 2：运行时接入

- [ ] `src/i18n/index.ts`（3.7 KB）改为 i18next 初始化。
- [ ] **配置 `interpolation.prefix = '{'`、`interpolation.suffix = '}'`**（`design.md` §1 路径 B）。词条零改动。
- [ ] 不启用复数后缀与 context 特性（会改变 key 集合，破坏 AC4）。
- [ ] `zh-CN.ts`（204.8 KB）与 `en-US.ts`（212.9 KB）只改导入形态，内容不动。
- [ ] `formatMessage.ts` 按批次 0 的结论处理。
- [ ] `bootMessages.ts`（45.6 KB）只调导入路径，不接入 i18next（`design.md` §4）。
- [ ] `deferLocaleHydration` handle 字段与 `bootMessages` 的配合关系保留。
- [ ] `I18nextProvider` 装入 `main.tsx` 的 Provider 栈（`08-22-react-foundation` §1 预留的位置）。
- [ ] 语言切换用 `changeLanguage()`，持久化沿用批次 0 确认的存储键。不引入 `i18next-browser-languagedetector`。
- [ ] 开发模式配置 `parseMissingKeyHandler` 显著标记缺失 key（`design.md` §8 第 1 层）。

验证：`git diff --stat src/i18n/locales/`（应为空或只有登记项，AC2）；`bun run type-check`。

## 批次 3：调用点收尾

调用点转换在七个视图子任务内随视图迁移同步进行（R3）。本批次负责遗漏排查与收尾。

- [ ] `rg 'vue-i18n' ccr-ui/package.json ccr-ui/src` 无匹配（AC1）。
- [ ] `rg '\$t\(|useI18n\(' src` 无匹配（AC5）。
- [ ] 不符合批次 1 约定的写法（`<Trans>`、命名空间参数、HOC）排查并统一。
- [ ] 遗漏的调用点补转。

## 批次 4：静态检查扩展

- [ ] 按 `design.md` §7 扩展 `scripts/check-i18n.mjs`：加缺失 key 检查与未使用 key 检查。
- [ ] 动态 key 加白名单，逐条登记原因。
- [ ] `tests/i18n.test.cjs` 适配新运行时，继续校验两个 locale 的 key 集合一致（R8）。

验证：`bun run check:i18n` 退出码 0（AC3）；`bun run test:i18n` 退出码 0，两个 locale 均 4,164 个叶子 key（AC4）；AC9（未使用与缺失 key 检查可运行并通过）。

## 批次 5：key 原文泄漏逐路由检查

在阶段 5 结束后执行（需全部视图已迁移）。

- [ ] 按 `design.md` §8.1 写检测脚本：**检测器由 `zh-CN.ts` 递归展开的 4,164 个叶子 key 集合生成**，页面候选串查集合命中。不用手写正则做判定（实测某个合理正则漏 105 个含下划线的 key）。
- [ ] 按 `design.md` §8.2 给脚本加自测四例：`checkin.stats.total_accounts` 命中、`common.save` 命中、`保存` 不命中、`package.json` 不命中。自测不过则脚本不可用。
- [ ] 遍历 75 条路由，中英文各跑一遍（AC6）。
- [ ] 脚本落 `ccr-ui/scripts/` 下，可重复运行，供 `08-22-regression-release` 复用。
- [ ] 命中项逐条修复后重跑。

## 批次 6：CSP 与收尾验证

- [ ] 按 `design.md` §9 在打包产物中切换语言，确认无 CSP 报错。该确认同时是 `08-22-regression-release` AC4 的一项。
- [ ] 刷新页面后语言偏好保留（AC7）。
- [ ] 启动期文案在 i18n 初始化前正确显示（AC8）。

## 验证命令

| 时机        | 命令                                                                        |
| ----------- | --------------------------------------------------------------------------- |
| 每批次后    | `bun run type-check`、`bun run lint`（AC10）                                |
| 批次 2–4 后 | `bun run check:i18n`、`bun run test:i18n`                                   |
| 批次 3 后   | `rg 'vue-i18n' ccr-ui/package.json ccr-ui/src`、`rg '\$t\(\|useI18n\(' src` |
| 批次 5 后   | key 原文泄漏检测脚本（中英文各一遍）                                        |
| 批次 6 后   | `just tauri-build` 产物内切换语言                                           |
| 交付前      | `just frontend-check-quick`                                                 |

## 交付门（父任务视图门的一部分）

- [ ] AC1–AC10 全部满足。
- [ ] `i18n-probe.md` 与 `i18n-call-convention.md` 落盘。
- [ ] 词条 git diff 为空，或改动项逐条登记（AC2）。
- [ ] 两个 locale 的叶子 key 集合一致，均为 4,164 个（AC4）。
- [ ] 75 条路由的 key 原文泄漏检查中英文各通过（AC6）。
- [ ] 静态检查扩展可运行，动态 key 白名单逐条登记（AC9）。
- [ ] CSP 下切换语言无报错。

## 回滚点

| 批次 | 回滚方式                                                    |
| ---- | ----------------------------------------------------------- |
| 0–1  | 只产出文档                                                  |
| 2    | 运行时接入。单独提交。回滚后应用无 i18n，但词条未动，可重做 |
| 3    | 调用点收尾。按文件分次提交                                  |
| 4    | 脚本扩展，单独提交                                          |
| 5–6  | 检测脚本与验证                                              |

词条文件全程零改动（批次 2 的路径 B 决策），因此不存在词条损坏的回滚场景。这是选路径 B 的附带收益。

## 协同点

| 编号 | 内容                                                       | 对方                       | 时机                      |
| ---- | ---------------------------------------------------------- | -------------------------- | ------------------------- |
| I    | 调用形式约定先行，七个子任务按此转换                       | 七个视图子任务             | 批次 1，须在阶段 5 开始前 |
| —    | `deferLocaleHydration` handle 字段与 `bootMessages` 的配合 | `08-22-shell-port`         | 批次 2                    |
| —    | i18next 在 CSP 下无 runtime compiler 问题的确认            | `08-22-dep-upgrade`        | 批次 6                    |
| —    | key 原文泄漏检测脚本复用                                   | `08-22-regression-release` | 批次 5 后                 |
| —    | 词条 417.7 KB 全量加载对首屏的影响                         | `08-22-arch-quality-perf`  | 批次 2                    |
