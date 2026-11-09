# 规划审阅裁定（2026-08-25）

对外部审阅报告 TPR-01..TPR-12 的逐条核验。每条给出：判定、证据、与报告的差异、修订动作。
证据为本轮直接读取仓库文件所得，行号为读取时快照。

---

## TPR-01 jsonl 仍为种子 — 属实（阻断）

证据：7 个任务目录的 `implement.jsonl` 与 `check.jsonl` 共 14 个文件，首行均为
`{"_example": "Fill with ..."}`，无真实条目。

与报告一致。修订：14 个文件全部填真实 spec/research 条目。

---

## TPR-02 令牌治理未闭环 — 属实，且比报告更严重（阻断）

三个独立事实：

1. `ccr-ui/tests/apple-glass-surface-contract.smoke.test.ts` 断言
   `--material-glass-chrome-bg: var(--color-bg-elevated)`，并在
   `prefers-reduced-transparency` 块中再次断言同一取值。
   原 `implement.md` 第 6 步「把 `--material-glass-chrome-bg` 回退目标改为
   `--color-bg-chrome`」会同时打断这两处断言。
2. `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md:26`：448 个变量名集合冻结，
   `src/styles/**` 的变量名并集必须等于迁移前集合，新增名称需要专门的 token 治理任务。
3. `ccr-ui/src/utils/themeBootstrap.ts:343` `CUSTOM_ACCENT_VARIABLE_FAMILY` 恰为 8 个变量。
   自定义强调色只重写这 8 个，新增的 accent 相关令牌不会随自定义强调色重算。

报告未指出的第四点：`tokens.css` 已经具备设计稿要的四层表面阶梯，
`--surface-shell-bg: var(--material-glass-chrome-bg)` → `--color-bg-elevated`，
`--surface-card-bg: var(--color-bg-surface)`。clay 暗色下为
base `#17120f` / elevated `#221b18`（shell）/ surface `#2a221e`（卡片）/ overlay `#342b26`。
即 `--color-bg-chrome` 是重复造名，不需要新增。

报告未指出的第五点：圆角与边框可以做成**纯取值修改**，不新增名称。
`core.css:153-159` 的 `@theme inline` 把 `--radius-sm..full` 映射到同名第 1 层变量，
把 7 个既有令牌的取值收敛到 4 个不同值即可达成设计目标，无需 `--radius-chip|control|card|pill`。
原 `design.md` 的角色令牌方案会无谓地触发名称冻结。

修订：令牌子任务改为先做名称增量审计，逐项分类为「改值 / 复用 / 真新增」；
真新增项走该子任务自身承担的治理流程（更新名称集记录 + 更新 spec），
并显式列出需要迁移的既有测试断言。

---

## TPR-03 readiness 信息去向未定 — 属实（阻断）

证据：`DashboardReadinessLedger.tsx` 消费 `readiness`（`status` / `labelKey` /
`titleKey` / `descriptionKey` / `reasons[]`）与 `statusMetrics[]` 两组数据。
`dashboardPresentation.ts:652-654` 两者均由 `buildDashboardPresentation` 产出。
原计划把 ledger 从首页移除，但只把 `statusMetrics` 交给用量指标行，
`reasons[]` 与三个文案键无承接方。

同时 `home-runtime-layout/prd.md` 要求 pill 在 Topbar，
其 `design.md` §4 又采用方案 A 把 pill 放在首页内容区，两者矛盾。

与报告一致。修订：逐项列出 ledger 每块信息的新归属；PRD 措辞改为与方案 A 一致。

---

## TPR-04 未跟踪平台判断无效 — 属实（阻断）

证据：`ccr-ui/src-tauri/src/services/usage.rs:616` `empty_home_platform_map()`
为 `home_usage_platforms()` 的每个平台插入 `HomeOverviewPlatformStats::default()`；
同文件 1148 行起，`series` 由 `build_home_date_range_from(start_day, days)` 逐日补齐，
缺失日期取 `empty_home_platform_map()`。因此未跟踪平台同样得到长度等于 `days` 的全零序列。
`DashboardPlatformRow.state` 表示 CLI 安装/运行状态，与用量跟踪无关。

可用的真实信号：`UsageArchiveDiagnostics.source_health: Array<UsageSourceHealth>`，
其中 `UsageSourceHealth = { source: string, state: "live" | "degraded" | "missing", ... }`。

与报告一致。修订：占位判据改为 `source_health`，并把「`source` 字段取值到 `usageKey`
的对照」列为子任务第一项研究步骤（当前未验证该字符串取值域）。

---

## TPR-05 成本区间与延迟机制未定 — 属实，修法比报告简单（阻断）

证据：`ccr-ui/src/features/usage/queries.ts:80`
`useUsageSummary(platform?, startDate?, endDate?)`，
`queryKey: usageKeys.summary(platform, startDate, endDate)`，无 `enabled` 参数。

报告称「query key 不随 activeDays 变化」——这只对无参调用成立。
显式传入区间即可让 key 随区间变化并触发重取，无需改 hook 签名。
区间口径可由 `usage.rs` 的 `local_usage_date_window(days)` 反推：
`end = 本地今天`，`start = end - (days - 1)`。

延迟发起没有现成开关。改 `useUsageSummary` 会波及所有既有调用方，
因此改为条件挂载子组件：父组件在首屏 perf mark 之后翻转标志位再渲染成本子组件。

修订：成本项显式传区间；延迟用条件挂载；change list 增加子组件文件。

---

## TPR-06 事件流既有能力会被删掉 — 属实（阻断）

证据：`DashboardSignalStream.tsx` 现有能力：

- `PillToggleGroup` 三档筛选 `all` / `warn` / `error`，标签自带计数。
- `channel` 独立列（`dashboard-signal__channel`）。
- 相邻同 `message` + `channel` + `level` 聚合为一行并显示 `×N`。
- 空态含 `/monitoring` CTA，非空时页脚另有 `/monitoring` 链接。

计数口径（现状）：`aggregatedEntries` 上计数，即**聚合后、筛选前、截断前**；
可见行为 `aggregatedEntries.filter(...).slice(0, limit)`，`limit` 默认 6。

设计稿的三列事件行不含筛选与 channel。与报告一致。
修订：逐项决定保留方式，并把上述计数口径写进验收标准，取代「计数与列表一致」。

---

## TPR-07 外观子任务不成立 — 属实（阻断）

证据：`AppearanceSection.tsx` 已具备主题三选、flavor 卡片与预览、UI/代码字体选择与自定义输入、
重置提示。`ccr-ui/src/features/configs/lib/flavorPreview.ts` 用
`FLAVOR_PREVIEW_TOKENS` 硬编码 20 个十六进制值，取值与 `tokens.css` 的
neutral/clay × light/dark 表面与文本令牌重复，无一致性机制。

与报告一致。补充：父任务 XC1 的检查命令包含 `ccr-ui/src/features/configs/`
整个目录，会命中 `flavorPreview.ts` 这 20 个值，属误报。

修订：该子任务升级为复杂任务，补 `design.md` / `implement.md`，
明确前后差异与 flavor 预览的单一取值来源。

---

## TPR-08 回归测试未落到文件 — 属实（阻断）

证据：6 个子任务的 change list 均无测试文件。
既有相关测试：`dashboard-presentation.smoke.test.ts`（`buildDashboardPresentation` 契约）、
`apple-glass-surface-contract.smoke.test.ts`、`theme-contrast-contract.smoke.test.ts`、
`theme-switch.smoke.test.tsx`、`token-single-point.smoke.test.tsx`、
`hardcode-px-rgba.smoke.test.ts`（仅覆盖 `.ts` / `.tsx` 的 px 与 `rgba()`，不覆盖 CSS，不覆盖十六进制）。

无任何既有测试覆盖 sparkline、成本行、side rail DOM、外观页重排、响应式断点。

与报告一致。修订：R8 拆到各子任务，每个子任务列出测试文件与断言。

---

## TPR-09 断点不可用 — 属实，修法比报告简单（应修）

证据：`tokens.css:522-530` 的 `--breakpoint-sm..2xl` 位于一个标注
「仅用于参考，主要针对桌面」的 `:root` 块，不在 `core.css` 的 `@theme` 中，
既不能生成 Tailwind 变体，也不能用于 `@media` 条件。

仓库既有写法为 CSS Media Queries Level 4 区间语法加 px 字面量，
例如 `ccr-ui/src/styles/components/profiles-page.css:184` `@media (width >= 1280px)`、
`:190` `@media (width <= 1279px)`、`:196` `@media (width <= 1024px)`。

修订：沿用该写法并固定三档验收视口，不引入构建期常量。

---

## TPR-10 门禁与基线 — 属实（应修）

证据：

- 父任务 `implement.md:21` 的 XC1 命令包含 `ccr-ui/src/features/configs/`，
  会命中 `flavorPreview.ts` 既有十六进制值。
- XC4 用 `git diff --stat`，只看工作区，看不到已 `git add` 的改动。
- 父任务 `task.json` `base_branch: "main"`；`git rev-list --count main..dev` = 197。

修订：XC1 改为只检查本任务改动的 CSS 文件；XC4 同时检查工作区与暂存区；
`base_branch` 改为 `dev`。

---

## TPR-11 依赖图不一致 — 属实（应修）

证据：父任务 `prd.md:72` 把 `home-side-rail` 前置写为「令牌层、运行时布局」，
`home-usage-chart` 前置写为「令牌层」；但 `implement.md:11` 把 usage 排在阶段 3，
即 runtime（阶段 2）之后，且 `home-usage-chart/implement.md:3` 明写前置含 runtime。
`home-runtime-layout/design.md` change list 声称 8 个文件，
§7 又允许删除 `DashboardReadinessLedger.tsx` 与 `dashboard-readiness-ledger.css`。

修订：统一为 usage 依赖 token + runtime；把可能删除的文件计入 change list 与回滚清单。

---

## TPR-12 AC 格式 — 属实（提示）

证据：全部 PRD 使用 `- AC1 → R1:` 行式；机械预检按 checkbox 抽取，得数为 0。
父任务 `task.json` 的 `dev_type` / `scope` / `package` 均为 `null`。

修订：AC 改 checkbox；补齐任务元数据。

---

## 报告中需要更正的表述

1. TPR-02「只修改 tokens.css 即可通过 frontend-check-quick」被判为计划的主张。
   实际计划的更大问题是新增了本不需要的名称（圆角角色令牌、`--color-bg-chrome`），
   而这两项都能用取值修改或复用既有别名达成。缩小范围比补治理材料更正确。
2. TPR-05「query key 也不随 activeDays 变化」只在无参调用下成立。
   传入 `startDate` / `endDate` 即可让 key 随区间变化，不需要改 hook 签名。
3. TPR-09 建议「固定媒体查询字面值或共享构建期常量」。
   仓库已有既定写法（Level 4 区间语法 + px 字面量），沿用即可，不需要新机制。

## 未验证项

- `UsageSourceHealth.source` 的字符串取值域与 `usageKey`（`claude` / `codex` /
  `gemini` / `opencode`）的对照关系：`UNVERIFIED`，列为令牌层之后的第一项研究步骤。
- 四套主题在真实浏览器下的对比度与级联结果：`UNVERIFIED`，无实现可测。
- `shell.css` 中 `sidebar-glass` / `topbar-glass` 是否已经消费 `--surface-shell-*`：
`UNVERIFIED`，列为 runtime 子任务第一项研究步骤。若已消费，chrome 实色化为零改动。
</content>

</invoke>
