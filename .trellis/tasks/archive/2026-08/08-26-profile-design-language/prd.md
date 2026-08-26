# 统一 Profile 页面设计语言

## Goal

让 `claude` / `codex` / `grok` 三个 Profile 页面收敛到同一套**呈现层**：同样的页壳、统计条、筛选栏、卡片与表格双视图、空态、编辑器外壳。平台各自的读取投影、写入语义、表单模型与状态机保留在平台控制器内，不进入呈现层。

设计基线为 Claude Design 项目 `0a3d3dfa-8ad5-4bdf-861d-305f1e2c6389` 的 `CCR UI Profile 成品稿.dc.html`。规格提取见 `research/design-source.md`。

## 现状

三个平台目前是三条互不共享的实现路径。

| 维度 | Claude | Codex | Grok |
|---|---|---|---|
| 页面入口 | `BaseProfiles(claudeProfilesConfig)` | `BaseProfiles(codexProfilesConfig)` | 独立 `GrokProfilesPage` |
| 页壳 | `SurfacePage` | `SurfacePage` | `PageShell` + `GrokSubnav` |
| 列表形态 | `ProfileListRow` 密度行 | `ProfileListRow` 密度行 | `GrokProfileCard` 两列卡片 |
| 统计条 | 无 | 无 | 无 |
| 搜索与筛选 | 无 | 无 | 无 |
| 视图切换 | 无 | 无 | 无 |
| 新建 / 编辑 | `onAdd` / `onEdit` 传 `noop`，无 UI | 同 Claude，无 UI | `GrokProfileEditorModal` |
| 停用（profile off） | 页头下方裸按钮 | 同 Claude | 同 Claude |

仓库中已存在但零页面消费的组件（React 迁移遗留）：`ProfilesStatStrip`、`ProfilesToolbar`、`ProfilesCommandPalette`、`ProfilesQuickRail`、`ProfilesInspector` 系列、`ProfilesRawEditorPanel`、`ProfileDiffRows`，以及 `profiles-shared.css`、`profile-editor-shell.css`。同类情况还有 `utils/claudeProfileEditor.ts`、`utils/codexProfileEditor.ts` 两份表单模型，均无对应 UI。这些资产只在 `features/platform/profiles/shared.ts` 里被 re-export。

`ProfilesConfig`（`src/configs/profiles.ts`）是 registry 雏形，但存在三处硬约束：

1. `ProfileRecord` 只有 `name / description / enabled / tags / model / baseUrl / authMode` 七个字段，`toProfile()` 丢弃其余全部字段，三平台 `list()` 都经过该投影。Grok 的 `profile_kind`、Claude 与 Codex 的高级字段都不在其中。
2. `ProfileDraft` 只有 `name / description / model / tags` 四个字段，装不下任何一个平台的真实写入 payload。
3. `grokProfilesConfig` 没有 `create` / `update`，Grok 写入走 `buildGrokCreateRequest` 与 `buildGrokPatch(form, dirtyFields)` 的 dirty-patch 语义。

后端 `profile_to_json`（`commands/claude.rs`、`commands/codex.rs`）对 `auth_token` 显式 `Secret::expose` 返回明文，注释写明「编辑表单预填需要原文」。掩码化属另一任务，本任务必须在前端侧建立剥离边界。

Token 侧：`--color-platform-{claude,codex,grok,gemini,opencode}` 已存在，无 antigravity；`--font-mono` 已存在。

## 已确认决策

1. **平台色**：更新 `--color-platform-codex` 与 `--color-platform-grok` 对齐设计稿，并补 antigravity。首页等已消费这些 token 的位置一并变化，属于预期内的溢出。
2. **路由**：保持 `/claude-code/profiles`、`/codex/profiles`、`/grok/profiles` 三条独立路由。设计稿左栏平台切换器视为全局侧栏的演示，不新增 `/profiles` 聚合页。
3. **编辑器字段**：核心字段严格按成品稿排布（名称、描述、Base URL、模型、平台字段、认证、标签），平台专属的其余字段收进可折叠「高级」区。

## 修订决策（审阅返回后新增）

以下决策回应 `.trellis/reviews/08-26-profile-design-language.md` 的阻断项，与上面三条同级。

4. **统一的是呈现层，不是数据层**（TPR-01 / TPR-02 / TPR-03 / TPR-05）。呈现层组件与编辑器外壳三平台共用；typed 读取投影、写入 adapter、表单模型、校验器、状态机留在平台控制器。`useGrokProfilesPage` 作为 Grok 平台控制器保留，不退役。
5. **现行 Profiles 规格的既有能力全部保留**（TPR-06）。QuickRail、Filters 弹层的 provider 与排序维度、Inspector 右栏、Off 横幅的位置与 `type=warning` 语义按 `profiles-page-contracts.md` 原样纳入新页面。删除任何一项都是独立的产品决策，需要用户批准后另立任务，本任务不做。
6. **明文凭据在进入 UI 状态前剥离**（TPR-04）。平台控制器在 `useQuery` 的 `select` 阶段调用读取 sanitizer，剥离后的记录才进入 React state、DOM、日志与错误信息。
7. **可扩展性口径改为仓库真实的两层成本**（TPR-08）：一条 `platformSurfaceDescriptors` row + 每个 surface 一份 config/presentation 导出。不宣称「只改一处」。
8. **契约测试一律落在 `ccr-ui/tests/*.smoke.test.ts(x)`**（TPR-09）。`vitest.smoke.config.ts` 的 `include` 只有 `tests/**/*.smoke.test.{ts,tsx}`，放在 `src/**/__tests__/` 的测试不会被任何门禁执行。

## Requirements

- R1：呈现层统一。页壳、统计条、筛选栏、卡片视图、表格视图、空态、编辑器外壳七类组件三平台共用一套实现，组件内不得出现平台名字面量分支（`platform-surface-contracts.md` 的 ESLint 规则同样适用）。
- R2：typed 读取投影。呈现层只消费 `ProfileDisplayRecord`。该记录由平台 `ProfilePresentation.project()` 从平台 typed DTO（`ClaudeProfile` / `CodexProfile` / `GrokProfileDto`）生成，不经过 `configs/profiles.ts` 的 `ProfileRecord` 七字段投影。
- R3：凭据剥离边界。平台控制器在数据进入 React state 前剥离明文凭据字段。剥离后的记录不含任何平台的密钥原文。`ProfileDisplayRecord` 与 `searchText` 均不含凭据。
- R4：平台色 token 与命名治理。为 claude / codex / grok / antigravity / opencode / gemini 六平台补 `-surface` / `-border` / `-text` 三角色，antigravity 另补 dot 与 `-rgb`；按 `theme-token-contracts.md` 完成名称增量登记、冻结段更新与作用域归属判定。明色取值写为可直接计算的 hex。
- R5：统计条。四卡等宽网格——总数（含供应商去重计数）、运行中（accent 高亮）、标签分布、认证方式。供应商 key 的规范化算法由 R12 定义。
- R6：搜索与筛选。搜索覆盖名称、描述、Base URL、标签；标签 pill 单选；provider 与排序维度保留在 Filters 弹层内（决策 5）；结果为空时进入空态。
- R7：双视图与视图模式持久化。卡片视图（三列）与表格视图（六列，窄屏容器内横向滚动）共用同一套行状态计算。视图选择按平台 key 持久化，复用 `features/profiles/stores.ts` 既有的 Zustand + localStorage 模式，storage 不可用时降级为纯内存并保持当前会话可用。
- R8：空态。区分「该平台无任何 profile」与「筛选无结果」两种文案与动作。
- R9：编辑器外壳统一，平台表单模型保留。一个 `ProfileEditorModal` 提供分区外壳、字段渲染原语、底部动作栏；字段集合、条件必填、认证状态机、序列化与提交由平台 `ProfileEditorAdapter` 提供。Codex 五种 auth mode、Claude 两种 auth mode、Grok 四种 credential action 与 official / third-party 分支各自保留现有语义。
- R10：写入结果结构化。平台 adapter 的提交返回结构化结果，能表达 Grok 的 `rename_apply_failed` / `rename_cleanup_failed` / `blocked`。呈现层只发出意图并渲染控制器给出的状态，不解释后端 status。
- R11：原始配置出口。`ProfilesRawEditorPanel` 由平台控制器提供的 raw-source capability 驱动，逐项满足 `raw-config-editor-contracts.md` 的明文警告、version token、`conflict`、`activation_conflict`、保存后全量刷新五项要求。Grok 无 raw source，入口不渲染。
- R12：供应商 canonical key。定义 Base URL 到供应商 key 的规范化算法与非法输入策略，并以等价类测试锁定。
- R13：三平台接线与零消费清零。三个平台页面切到统一呈现层；`components/profiles/` 下每个导出组件都有非 barrel 消费方。
- R14：可扩展性口径。新增平台的 profiles 页面成本按两层表述：层一 `platformSurfaceDescriptors` 的 row 或 `surfaces` 条目（与上线决策绑定），层二 profiles surface 模块的 `ProfilesConfig` + `ProfilePresentation` 两条注册项。本任务只交付并验证层二，验证走真实注册表取值，不用手写 mock config 冒充。不表述为「只需注册元数据即可得到完整页面」。
- R15：测试落位与门禁。全部新增契约测试位于 `ccr-ui/tests/*.smoke.test.ts(x)`，在各子任务 design 的 change list 中逐个列出路径，并给出逐任务 focused 命令，最终仍跑 `just frontend-check-quick` 与 `just ui-check`。
- R16：视觉与响应式验收前置条件。明暗主题、窄窗口、走查类验收必须给出固定 viewport、zoom、theme × flavor 组合、数据夹具与测量判据，同一构建在不同环境下结论一致。

## Acceptance Criteria

以下为跨子任务验收，需在 `08-26-profile-rollout` 完成后整体成立。

- [ ] AC1（R1）：三页渲染同一套页壳、统计条、筛选栏、列表与编辑器外壳组件，平台差异只来自 presentation 与 adapter 入参。
- [ ] AC2（R1）：`tests/platform-surface-unify.smoke.test.ts` 的「Base 文件无平台名字面量分支」断言覆盖新增的 `features/platform/profiles/**` 与 `components/profiles/**`，且通过。
- [ ] AC3（R2）：三平台的 `project()` 各有一条往返测试，断言 `ProfileDisplayRecord` 的四个 slot、`vendorKey`、`authKey`、`badges` 取值来自 typed DTO 而非七字段投影。
- [ ] AC4（R2）：Grok 的 `profile_kind` 在统一卡片与表格中均可见，取值来自 `GrokProfileDto.profile_kind`。
- [ ] AC5（R3）：以随机 sentinel 作为 `auth_token` 走完整 list 流程后，sentinel 不出现在 `ProfileDisplayRecord`、DOM 文本、`console` 输出、toast/错误文案、导出内容与编辑器提交 payload 六处，测试断言。
- [ ] AC6（R4）：六个平台的 dot / surface / border / text 四角色在明暗两套主题下都有取值，无 `undefined` 回退；antigravity 的 dot 与 `-rgb` 同时存在。
- [ ] AC7（R4）：明色主题下每个平台 `-text` 对 `-surface` 的对比度不低于 4.5:1，由 hex 直接计算断言。
- [ ] AC8（R4）：token 名称增量已登记进 `theme-token-contracts.md` 的冻结段，`tests/theme-switch.smoke.test.tsx` 与 `tests/token-single-point.smoke.test.tsx` 通过。
- [ ] AC9（R5）：统计条四卡数值与列表一致；供应商数按 R12 的 canonical key 去重。
- [ ] AC10（R6）：搜索对四字段生效；标签 pill 与 Filters 弹层内的 provider、排序维度均可用，与搜索可叠加。
- [ ] AC11（R6、R8）：筛选无结果时展示带「清除筛选」的空态，点击后恢复全量列表；平台本身无 profile 时展示另一套文案。
- [ ] AC12（R7）：卡片与表格展示同一组数据，运行中态在两个视图中的高亮一致。
- [ ] AC13（R7）：视图选择在同一会话内跨路由卸载重挂载后保持，且 claude 与 codex 互不影响；localStorage 抛错时页面仍可切换视图。
- [ ] AC14（R9）：三个平台均可完成新建与编辑，使用同一个 `ProfileEditorModal` 外壳。
- [ ] AC15（R9）：Codex 五种 auth mode 的条件必填矩阵（base URL / secret / env key / model）与 `profiles-page-contracts.md` 的表格逐行一致，good / base / bad 各一条测试。
- [ ] AC16（R9）：Grok 的 credential action 互斥序列化、display URL 不回填、official 无 provider/base URL/凭据控件三条断言保持通过。
- [ ] AC17（R9、R3）：编辑既有 profile 时密钥输入为空并显示「留空不修改」，提交后既有密钥不被清空。
- [ ] AC18（R10）：Grok 的 `rename_apply_failed`、`rename_cleanup_failed`、delete `blocked`、`unsafe_missing_entry_state`、force 不循环五条分支在统一页面下行为不变，现有 `tests/grok-profiles-view.smoke.test.ts` 通过。
- [ ] AC19（R11）：Claude 与 Codex 的原始配置入口可进入 source mode，明文警告、`conflict` 只给重载/取消、`activation_conflict` 需显式危险确认、保存后全量刷新四项各有一条断言；Grok 页面无该入口。
- [ ] AC20（R12）：供应商 key 的等价类测试覆盖大小写、默认端口与显式端口、userinfo、IPv6、尾点、无协议输入、空值与非法输入八类。
- [ ] AC21（R13）：`rg` 验证 `components/profiles/` 下每个导出组件至少有一个非 barrel 消费方。
- [ ] AC22（R13）：`features/grok/profiles/` 中被统一呈现层取代的文件已删除，`useGrokProfilesPage` 保留为平台控制器。
- [ ] AC23（R14）：antigravity 的 `ProfilesConfig` 与 `ProfilePresentation` 在注册表中存在，测试按 key 从注册表取出后渲染出完整 `ProfilesSurface`，不使用手写 mock config；`tests/platform-surface-unify.smoke.test.ts` 的 75 路径断言仍通过（descriptor 未改动）。
- [ ] AC24（R15）：新增契约测试全部位于 `ccr-ui/tests/*.smoke.test.ts(x)`，`bun run test:smoke` 能发现并执行；`just frontend-check-quick` 与 `just ui-check` 通过。
- [ ] AC25（R16、R1）：三页在 1440×900 与 900×800 两个 viewport、`light|dark` × `neutral|clay` 四种组合下按固定夹具走查通过，对照 `research/design-source.md` 的结构清单。
- [ ] AC26（R16、R7）：900×800 下表格容器 `scrollWidth > clientWidth` 且 `document.body.scrollWidth <= document.body.clientWidth`，测量值记录在 rollout `notes.md`。
- [ ] AC27（R1、R4）：Profile 相关 `.tsx` 与 `.css` 中无硬编码 hex，`rg` 结果为空。
- [ ] AC28（R1）：新增与改动的用户可见文案在 `zh-CN` 与 `en-US` 两份 locale 中都有 key，`bun run check:i18n` 通过。

## Constraints

- 明暗双主题都必须达到高对比，颜色一律走 `--color-*` token。
- 动效遵循仓库既有的 reduced motion 降级约定。
- API key 只允许掩码显示，不得进入日志、导出或错误信息。剥离边界见 R3。
- `ClaudeProfile` DTO 无 `last_used`，成品稿 Claude 的「最近使用」槽位必须改用后端已有字段，不得为此新增后端字段。
- 模型与标签候选必须允许自由输入，registry 候选值只作为快捷选项。
- 本任务不改 Tauri 后端命令签名，也不改后端凭据序列化。若发现字段缺口，记录在 `08-26-profile-rollout` 的 notes，另立任务。
- 不得在实施或 rollout 阶段临时改写验收口径。若某条 AC 无法达成，回到规划修订，不在运行时降级。
- 现行 `profiles-page-contracts.md`、`raw-config-editor-contracts.md`、`platform-surface-contracts.md`、`theme-token-contracts.md` 的既有条款是约束而非参考。需要变更条款时，在同一任务内更新规格并补测试。

## Task Map

| 子任务 | 交付物 | 依赖 |
|---|---|---|
| `08-26-profile-registry-tokens` | presentation / adapter 契约 + 凭据 sanitizer + 平台色 token 与治理 | 无 |
| `08-26-profile-list-surface` | 页壳、统计条、筛选栏、双视图、空态、QuickRail 与 Inspector 接线、source mode | registry-tokens |
| `08-26-profile-editor` | 编辑器外壳 + 三份平台 adapter | registry-tokens |
| `08-26-profile-rollout` | 三平台接线、Grok 呈现层退役、遗留组件清零、整体验收 | 前三者 |

`list-surface` 与 `editor` 在 `registry-tokens` 完成后可并行。

## Notes

- 设计稿只能通过 `DesignSync` 工具读取，WebFetch 与浏览器直连返回 403。规格已提取到 `research/design-source.md`，实施期以该文件为准，不必重新拉取。
- 同一 Design 项目内的首页设计稿对应已归档任务 `08-25-react-home-style-redesign`，其 token 结论是本任务的上游基线。
- 命名不一致：设计稿用 antigravity，代码 `config/platformDescriptors.ts` 用 descriptor id `gemini` 承载 `rootPath: '/antigravity'`。同一概念两个名字，本任务不改代码也不改设计稿，在 rollout `notes.md` 记录并上报。
- 审阅报告 `.trellis/reviews/08-26-profile-design-language.md` 的「未能核实」四项中，Claude slot3 字段填充率由 `registry-tokens` 实施期补核；设计稿一致性由 `research/design-source.md` 承担；真实视觉行为由 AC25 / AC26 的固定条件走查覆盖；能力删除的用户批准由决策 5 规避（本任务不删除任何已规格化能力）。
