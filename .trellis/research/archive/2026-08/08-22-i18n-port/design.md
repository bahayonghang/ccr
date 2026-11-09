# 技术设计：i18n 运行时迁移到 react-i18next

> 父任务：`08-22-react-migration`。词条 417.7 KB 是前端最大的单类资产，迁移时避免任何批量替换操作（PRD Notes）。本文件的核心是插值语法处理与调用形式约定。

## 1. 插值语法差异（本任务最关键的决策）

`vue-i18n` 的插值语法为 `{name}`。`i18next` 的默认插值语法为 `{{name}}`。

两种处理路径：

| 路径     | 操作                                                                       | 风险                                                                         |
| -------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| A 改词条 | 把 417.7 KB 词条中的 `{name}` 批量替换为 `{{name}}`                        | 批量替换在 4,164 个 key 上执行，误伤无法逐条核对。PRD Notes 明确禁止批量替换 |
| B 改配置 | 配置 i18next 的 `interpolation.prefix = '{'`、`interpolation.suffix = '}'` | 词条零改动。需确认词条中无字面量花括号会被误当插值                           |

**选路径 B。** 词条文件 git diff 为空（AC2 的首选形态），符合 PRD Notes 的禁止批量替换要求。

路径 B 的前置核对：`rg -o '\{[^}]*\}' src/i18n/locales/*.ts` 抽出全部花括号内容，确认每一处都是插值变量名而非字面量文本。若存在字面量花括号（如展示 JSON 示例的词条），那几条单独处理并在 AC2 的「改动项逐项登记」中记录。

## 2. 调用形式约定（七个并行子任务的共同依赖）

PRD Notes：需与七个并行子任务约定统一的调用形式，避免出现多种写法。该约定需先行落定并通知各子任务（协同点 I）。

约定：

```tsx
// 组件内
const { t } = useTranslation();
t("views.foo.bar");
t("views.foo.count", { n: 3 });

// 组件外（工具函数、store）
import i18n from "@/i18n";
i18n.t("errors.network");
```

禁止的写法：

- `<Trans>` 组件。本仓词条无嵌入 HTML 的需求（词条未使用 linked message 与复数管道），`<Trans>` 会引入第二种写法。
- 命名空间拆分（`useTranslation('views')`）。当前词条是单一扁平命名空间，拆分会改变 4,164 个 key 的形态。
- `withTranslation` HOC。

约定落盘为 `i18n-call-convention.md`，在阶段 4 → 5 门前通知七个视图子任务。约定晚于视图迁移开始，则七个子任务各自选形式，收尾时需返工。

## 3. 词条可移植性

PRD 已确认：词条未使用 vue-i18n 的 linked message（`@:`），未使用复数管道（`|`）。因此词条数据可直接移植，仅运行时与调用点需要转换。

i18next 的复数处理（`_one` / `_other` 后缀）与 context 特性均不需要启用——启用会改变 key 集合，破坏 AC4 的「两个 locale 的叶子 key 集合一致，均为 4,164 个」。

## 4. `bootMessages.ts`（45.6 KB）

R4：启动期文案在 i18n 运行时初始化之前可用，行为不变。

`bootMessages` 是独立于 i18n 运行时的静态映射——它服务于 i18n 尚未初始化的时间窗口。因此本任务对它只做导入路径调整，不接入 i18next。

`deferLocaleHydration` 的 handle 字段（`08-22-shell-port` §1）与 `bootMessages` 配合：标记该字段的路由延迟 locale 加载，此期间用 `bootMessages`。该配合关系在迁移后保留。

AC8：启动期文案在 i18n 初始化前正确显示。

## 5. `formatMessage.ts`（1.1 KB）

R5：格式化行为在新运行时下等价。

该文件的职责需在实施时确认——若它只是包装 `$t` 调用，改为包装 `t`；若它有自定义格式化逻辑（数字、日期），确认 i18next 的对应能力或保留其自有实现。

## 6. 语言切换与持久化

R6：切换在运行时生效，无需刷新。i18next 的 `changeLanguage()` 提供该能力，`react-i18next` 的 `useTranslation` 在语言变化时触发重渲染。

R7：语言偏好持久化行为不变，刷新后保留。现状的存储键与读写位置需确认并沿用。不引入 `i18next-browser-languagedetector`——它会改变偏好读取的来源与优先级。

## 7. 静态检查的等价方案（R9）

`@intlify/eslint-plugin-vue-i18n` 移除后，未使用 key 与缺失 key 的静态检查能力不下降。

方案：扩展现有 `scripts/check-i18n.mjs`，不引入新工具。

现状：`check-i18n.mjs` 校验两个 locale 的 key 集合一致；`tests/i18n.test.cjs` 是其测试封装。

扩展内容两项：

1. **缺失 key**：扫描 `src/**/*.tsx` 与 `.ts` 中的 `t('...')` 与 `i18n.t('...')` 调用，抽出字面量 key，断言每个 key 存在于词条中。
2. **未使用 key**：反向断言——词条中的 key 若不在扫描结果中，报为未使用。

两项的共同限制：动态 key（`t(someVar)`）无法静态抽出。现状是否存在动态 key 需在实施时统计。存在则那些 key 加白名单，白名单逐条登记原因。

选择扩展现有脚本而非引入 `i18next-parser` 的理由：`check-i18n.mjs` 已有词条解析逻辑与 CI 接入点（`bun run check:i18n`），扩展的增量小于接入新工具并适配其配置。

## 8. key 原文泄漏检测（AC6）

PRD Notes：key 原文泄漏（界面显示 `views.foo.bar`）是本次迁移最可能出现的回归形式，AC6 的逐路由检查不可省略。

两层检测：

1. **开发期**：配置 i18next 的 `parseMissingKeyHandler` 在开发模式抛出或显著标记，使缺失 key 在开发时即暴露而非静默显示 key 名。
2. **逐路由检查**：遍历 75 条路由，在渲染后扫描页面文本，命中 key 形态的字符串即为泄漏。中英文各跑一遍。

### 8.1 检测器不能用手写正则

手写正则会漏。实测：对 `zh-CN.ts` 递归展开得到 4,164 个叶子 key，用 `^[a-z][a-zA-Z]*(\.[a-zA-Z0-9]+)+$` 只匹配 4,059 个，漏 105 个——全部因含下划线，例如 `checkin.stats.total_accounts`、`checkin.stats.today_checkins`、`checkin.stats.success_rate`。这些 key 即使原文直接显示在页面上，检查也会通过。

**检测器由实际叶子 key 集合生成**，不由字符类推断：

1. 递归展开 `src/i18n/locales/zh-CN.ts`，得到叶子 key 集合（4,164 个）。
2. 页面文本按空白与标点切分为候选串，逐个查该集合是否命中。集合命中即为泄漏。
3. 集合来自词条本身，key 命名习惯变化（新增下划线、数字、更深层级）不需要改检测器。

若因性能需要先用正则做粗筛，正则的每个分段必须接受仓库实际采用的字符集（至少 `[A-Za-z0-9_]`），粗筛后仍以集合命中为准。

### 8.2 检测器自身的正反例

检测脚本需带自测，至少四例：

| 例                                  | 期望   | 覆盖的失效模式                       |
| ----------------------------------- | ------ | ------------------------------------ |
| `checkin.stats.total_accounts`      | 命中   | 含下划线的 key（先前正则漏掉的 105 个） |
| `common.save`                       | 命中   | 两段无下划线 key                     |
| `保存`                              | 不命中 | 正常译文                             |
| `example.com` / `package.json` 等   | 不命中 | 形似 key 但不在词条集合内的普通文本   |

第 4 例是用集合命中而非正则的直接收益：正则会把 `package.json` 误报为泄漏。

第 2 层需要应用可运行且全部视图已迁移，因此在阶段 5 结束后执行。检测脚本落在 `ccr-ui/scripts/` 下，可重复运行，也供 `08-22-regression-release` 复用。

## 9. `vite.config.ts` 的 alias 移除

现状 `ccr-ui/vite.config.ts:22–24` 有 `vue-i18n` 的 dev / build 双入口 alias，注释说明原因：dev 需要 message compiler（否则 locale 字符串直接回退成 key），build 用 runtime-only 避免桌面壳 CSP 与 runtime compiler 冲突。

`08-22-dep-upgrade` 段 1 已删除该 alias。本任务需确认 i18next 在桌面壳 CSP 下无等价问题：i18next 无 runtime compiler，词条是普通 JS 对象，不涉及运行时编译，因此 CSP 的 `unsafe-eval` 类限制不适用。该确认需实测一次——在打包产物中切换语言，确认无 CSP 报错。

该确认也是 `08-22-regression-release` AC4 的 CSP 验证项之一。

## 10. 不变量

- 4,164 个叶子 key 与两个 locale 的词条内容不变（AC2 的首选形态为 git diff 为空）。
- 只有 zh-CN 与 en-US 两个 locale，不新增语言。
- 不新增 i18n key。
- 词条不拆包、不按路由懒加载（Out of Scope；若判定有必要，另开任务）。

## 11. 未决项

- 词条中是否存在字面量花括号（第 1 节末段）。
- `formatMessage.ts` 是否有超出 `$t` 包装的自定义逻辑（第 5 节）。
- 语言偏好的存储键与读写位置（第 6 节）。
- 是否存在动态 key（第 7 节末段）。
- 词条不拆包的判定：417.7 KB 全量加载对首屏的影响由 `08-22-arch-quality-perf` 的 bundle 预算数据判定。若超预算，拆包另开任务，不在本任务范围。
