---
skill: trellis-plan-review
version: 0.4.0
task_dir: D:/Documents/Code/Github/ccr/.trellis/tasks/08-26-profile-visual-types
task_name: 08-26-profile-visual-types
task_status: planning
review_scope: task-tree
task_count: 4
task_members:
  - 08-26-profile-visual-types
  - 08-26-visual-type-primitives
  - 08-26-visual-type-profiles
  - 08-26-visual-type-rollout
task_statuses:
  08-26-profile-visual-types: planning
  08-26-visual-type-primitives: planning
  08-26-visual-type-profiles: planning
  08-26-visual-type-rollout: planning
verdict: 需返回规划
blocking: 2
should_fix: 7
notes: 1
generated_at: 2026-08-26T15:12:23.1558835+08:00
---

# Trellis 规划审阅报告

## 审阅范围

- 根任务：`08-26-profile-visual-types`
- 模式：`task-tree`
- 任务数量：4
- 有序成员（根优先；顺序不代表依赖）：
  - `08-26-profile-visual-types` — `planning`
  - `08-26-visual-type-primitives` — `planning`
  - `08-26-visual-type-profiles` — `planning`
  - `08-26-visual-type-rollout` — `planning`

## 结论

需返回规划 — 阻断 2 / 应修 7 / 提示 1

## 问题清单

### TPR-01 · 阻断 · ConfirmModal 的类型映射会丢失 warning / danger 语义

- Task: `08-26-visual-type-primitives`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-primitives`
- Location: `08-26-visual-type-primitives/prd.md:17`, `08-26-visual-type-primitives/prd.md:27`, `08-26-visual-type-primitives/design.md:13-14`, `08-26-profile-visual-types/research/visual-language.md:24-25`
- Claim: 原语子任务 R6 写成“确认跟随 `type`：danger/warning/info→primary”，而 AC5 只要求 ConfirmModal 渲染出 `.ui-btn`。
- Evidence: 当前 `ccr-ui/src/ui/confirm-modal.tsx:42-47` 对 danger、warning、info 使用三套不同颜色；`.trellis/spec/ccr-ui/frontend/confirm-interaction-contracts.md:29-35` 也把 danger、warning、info 定义为不同风险等级。父任务权威视觉规格同样把 `warning` 与 `danger` 定义为不同 Button 变体。子任务设计只写“footer 改 Button”，没有给出逐类型映射，AC5 即使把三种确认都改成 primary 也会通过。
- Impact: 按 R6 字面实施会把删除/不可逆与可逆高影响操作折叠为普通主按钮，造成现有风险语义回归；按现有代码自行猜测又会超出计划机制。
- Route: 二选一：明确并验证逐类型映射（例如保留 danger、warning、info 各自语义）；或把“全部 primary”作为显式产品决定，先同步父任务权威规格与确认交互契约并取得决策。两条路线都要让 AC 按三种 `type` 分别判定，不能只查 `.ui-btn`。

### TPR-02 · 阻断 · Rollout 对必迁站点的现状断言为假，封闭变体映射无法决定输出

- Task: `08-26-visual-type-rollout`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-rollout`
- Location: `08-26-visual-type-rollout/prd.md:18-19`, `08-26-visual-type-rollout/prd.md:30-32`, `08-26-visual-type-rollout/design.md:9-17`, `08-26-visual-type-rollout/prd.md:40`
- Claim: Closed inventory 第 4 项把 `AgentEditModal.tsx` 归为一次性 `bg-accent-primary px-4 py-2` 提交/新建按钮，设计映射表据此只覆盖 primary、secondary、ghost、danger 旧类；第 5 项同时要求迁移 McpPresets 确认按钮。
- Evidence: `ccr-ui/src/features/platform/agents/AgentEditModal.tsx:80-89` 的保存按钮实际是 `bg-accent-secondary px-4 py-3`，同文件 `:123-125` 的 Add tool 也是 `bg-accent-secondary`；`ccr-ui/src/features/mcp/McpPresetsPanel.tsx:161-165` 的确认安装同样是 `bg-accent-secondary`。这些旧形态都不在 `design.md:11-17` 的映射表中，AC2 只笼统要求“主 CTA”为 primary，不能决定 Add tool、保存、确认安装分别属于哪个封闭变体。
- Impact: 必迁站点没有设计机制；不同实现者可把所有 accent-secondary 都改成 primary、保留 secondary，或按局部判断混用，而同一份计划与 AC 均无法判定哪一种正确。
- Route: 逐个列出这些现有动作的语义与 old→new 映射，并为每一类补可判定 AC；或者从 closed inventory 移除这些站点并同步缩小父任务范围。不要以不存在的 `bg-accent-primary px-4 py-2` 作为迁移依据。

### TPR-03 · 应修 · “全局/封闭”范围与真实同类调用点及 change list 不一致

- Task: `cross-task`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-rollout`
- Location: `08-26-profile-visual-types/prd.md:5`, `08-26-profile-visual-types/prd.md:17`, `08-26-visual-type-rollout/prd.md:9-26`, `08-26-visual-type-rollout/design.md:3-17`, `08-26-visual-type-rollout/implement.md:5-12`
- Claim: 父任务称“全局抽取”并让其它操作页迁到同一原语，rollout 则称其清单为 closed inventory；但 `design.md` 没有逐文件 change list。
- Evidence: `rg -l 'primaryBtnClass|ghostBtnClass|secondaryBtnClass|dangerBtnClass' ccr-ui/src/features` 返回 26 个文件（含 3 个定义文件）；平台一次性 `bg-accent-primary px-4 py-2` 搜索返回 5 个 Base 文件。Closed inventory 第 4 项只列 `BaseMcp`、`BaseSettings`、`BaseAgents`，但同属平台 Base 且同形的 `ccr-ui/src/features/platform/commands/BaseCommands.tsx:77-78,108-112` 与 `ccr-ui/src/features/platform/plugins/BasePlugins.tsx:75-76,107-111` 既未列入，也未出现在 Out of scope。rollout 的设计和执行清单仍只写域级概括，无法与这 26 个真实文件逐项核对。
- Impact: 计划可在满足现有“封闭清单”AC 的同时，在紧邻的 BaseCommands/BasePlugins 保留同一套一次性 primary/secondary 按钮，从而与父任务的“全局/其它操作页”承诺冲突；实施漂移和回滚范围也无法按设计 change list 判断。
- Route: 二选一：把真实调用点解析为精确 change list，并补入遗漏的同类 Base 文件；或把父任务目标改为明确的部分迁移，并把 BaseCommands/BasePlugins 等写入显式 Out of scope。无论选择哪条路线，rollout 设计都应列出完整文件清单而不是“所有消费方”。

### TPR-04 · 应修 · 权威规格要求的 static 状态徽章没有子任务机制

- Task: `08-26-visual-type-profiles`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-profiles`
- Location: `08-26-profile-visual-types/prd.md:36-37`, `08-26-profile-visual-types/research/visual-language.md:34-45`, `08-26-visual-type-profiles/prd.md:16-24`, `08-26-visual-type-profiles/design.md:11-13`
- Claim: 父任务 R2 把 `visual-language.md` 设为权威规格；其中 static Badge 的代表用法包含卡片 AUTH、PROVIDER、tags 和状态徽章。profiles 子任务只覆盖字段 chip 与卡片 tags，没有覆盖状态徽章。
- Evidence: 当前 `ccr-ui/src/components/profiles/ProfileCardGrid.tsx:49-50,71-78` 分别以 `.cp-card__badge` 渲染行状态、以带 pointer 的 `.cp-chip` 渲染 `record.badges`；`.cp-chip` 的 pointer 与 hover 定义在 `ccr-ui/src/components/profiles/profiles-shared.css:381-399`。子任务 R4/R8、设计的 `ProfileFieldValue` 和 AC1-AC9 都没有这些两类节点的 Badge 机制或验收子句。
- Impact: 计划全部执行后仍可把部分静态状态/枚举留在旧视觉类型，其中 `record.badges` 继续呈现可点击光标；父任务 R2 可以被宣称完成，但权威规格并未完整落地。
- Route: 为状态徽章与 `record.badges` 增加明确的迁移机制和 AC；或从父任务权威规格/范围中明确排除这些节点。两处节点要分别处理，不能只改 fieldSlots。

### TPR-05 · 应修 · 原语核心视觉契约只验证 class 存在，R5 与像素禁令没有可判定 AC

- Task: `08-26-visual-type-primitives`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-primitives`
- Location: `08-26-profile-visual-types/research/visual-language.md:18-32`, `08-26-profile-visual-types/prd.md:35-36,48-50,59`, `08-26-visual-type-primitives/prd.md:12-19,23-31`, `08-26-visual-type-primitives/design.md:16-20,30-33`
- Claim: 父任务要求变体、尺寸、状态、减动效与排版全部以视觉规格为准；子任务又要求 `FieldLabel` 的 0.75rem/muted/0.08em 和新增 CSS 无 px。现有 AC2 只证明 7×2 个 class 可区分，AC7 只扫描 hex，AC8 只查 reduced-motion；AC9 没有 R 注解，R5 没有任何 AC。
- Evidence: 一个只有 `.ui-btn--primary` 等 class 名、但没有正确背景/边框/hover/focus/active/disabled/同高规则的实现仍能通过 AC2；写入 px 也不会被 AC7 的 hex regex 捕获。Pass 0 已机械确认 `08-26-visual-type-primitives` 的 `criteria_without_requirement=['AC9']`、`requirements_without_criterion=['R5']`。设计测试只写“render Button/Badge/UrlText/FieldLabel”，没有 CSS 或 computed-style 判据。
- Impact: 这是视觉类型任务的核心结果；当前门禁可以在 Button/FieldLabel 外观和状态错误时整体变绿，也不能证明无 px 字面量。
- Route: 按规格子句补确定性的 CSS/DOM/computed-style 契约（至少覆盖 7 变体语义、两尺寸同高、focus/active/disabled、FieldLabel 三项和 px 禁令），并为 R5 增加 AC、给 AC9 标注其实际覆盖的需求。若某项只做人工走查，必须把它写入可判定的人工步骤。

### TPR-06 · 应修 · Off 横幅容器的 warning 表面没有验收子句

- Task: `08-26-visual-type-profiles`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-profiles`
- Location: `08-26-visual-type-profiles/prd.md:15`, `08-26-visual-type-profiles/prd.md:28`, `08-26-visual-type-profiles/design.md:15-19`, `08-26-profile-visual-types/prd.md:52`
- Claim: profiles R2 同时要求 Off 按钮为 warning、容器使用 warning-tint + warning 边、确认继续 `type='warning'`；子任务 AC1 只查按钮 class，父任务 AC5 也只补了按钮与确认类型。
- Evidence: 当前 `ccr-ui/src/components/profiles/profiles-shared.css:1782-1792` 的 `.cp-off-banner` 仍是 `--color-border-subtle` + `--color-bg-elevated`。设计虽然在 `design.md:18` 写了目标 token，但父子两级没有任何 AC 判断容器是否真正改成 warning 表面。
- Impact: 实现可以完全漏掉容器 CSS，所有现有 AC 仍通过；R2 的一个独立用户可见子句无法验收。
- Route: 为容器背景和边框增加 token/computed-style 或源码契约；或者删除该容器子句，只保留按钮 warning。不要把按钮 class 当作容器表面的证据。

### TPR-07 · 应修 · UrlText 用例没有进入截断分支，也未覆盖非法 URL 回退

- Task: `08-26-visual-type-primitives`
- Affected tasks: `08-26-visual-type-primitives`
- Location: `08-26-visual-type-primitives/prd.md:15`, `08-26-visual-type-primitives/prd.md:26`, `08-26-visual-type-primitives/design.md:22-28`
- Claim: R4 要求 UrlText 调用 `formatBaseUrlDisplay` 并在非法 URL 时展示原文；AC4 用 `https://api.example.com/very/long/path` 证明长路径展示缩短。
- Evidence: `ccr-ui/src/utils/text.ts:11-18` 的默认截断阈值是 path 长度大于 18；该 AC 向量的 pathname `/very/long/path` 长度只有 15，因此不会进入截断分支。它之所以短于原文只是因为去掉了 `https://`。AC4 也没有非法 URL 向量。
- Impact: 一个只删除 scheme、从不截断长路径、也不保留非法输入的错误实现仍可能满足现有 AC。
- Route: 用 pathname 明确大于 18 的 URL 验证截断边界，并增加非法 URL 原文回退向量；或者缩小 R4，不再声称覆盖这两种行为。

### TPR-08 · 应修 · Profiles 清理策略同时允许 alias/跳过死代码，又要求全目录零残留

- Task: `cross-task`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-profiles`
- Location: `08-26-profile-visual-types/design.md:71-74`, `08-26-visual-type-profiles/prd.md:22,34`, `08-26-visual-type-profiles/prd.md:47`
- Claim: 父设计允许把 `cp-btn` / `pe-btn` 改成 `.ui-btn` alias 过渡；子任务 Notes 又允许无生产消费方的 `ProfilesHeader.tsx` 不迁，但 AC7 要求 `components/profiles` 下 `.cp-btn|.pe-btn` 为空或仅剩注释。
- Evidence: `ProfilesHeader` 当前只由 barrel/re-export 与测试消费，没有生产调用；但 `ccr-ui/src/components/profiles/ProfilesHeader.tsx:128,143,169,225` 有真实 `cp-btn` class，`ccr-ui/src/components/profiles/index.ts:13-17` 与 `ccr-ui/src/features/platform/profiles/shared.ts:8` 仍导出它。保留 alias 或按 Notes 跳过该文件都会使 AC7 的目录扫描失败。
- Impact: 计划给出彼此不兼容的完成路径；实施者必须自行选择删除死组件、迁移它、保留 alias 或放宽 AC，任何选择都会违背另一处规划文本。
- Route: 明确选择一种策略并同步父子产物：删除 `ProfilesHeader` 及导出/测试、迁移它到原语，或把 alias/死代码作为显式 AC 例外。不要同时保留“可 alias/可跳过”和“全目录零残留”。

### TPR-09 · 应修 · 8 组合视觉走查只要求记录存在，不给同一构建唯一结论

- Task: `08-26-visual-type-profiles`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-profiles`
- Location: `08-26-profile-visual-types/prd.md:42,58`, `08-26-visual-type-profiles/prd.md:23,35`, `08-26-visual-type-profiles/design.md:21-23`, `08-26-visual-type-profiles/implement.md:20`
- Claim: 父 R8 要求 4 个 theme×flavor 组合 × 2 个 viewport 对照截图和视觉规格走查，但父 AC11/子 AC8 的判据只是 `notes.md` “记录 8 张/8 组合”。
- Evidence: AC 没有要求每个组合为 PASS、没有列出必须逐项判断的页头主次、Off 表面、四字段类型、溢出/截断等结果，也没有规定截图或失败项路径。即使 `notes.md` 记录八个组合全部失败，字面 AC 仍成立。`just ui-check` 是自动化门，不提供这些视觉结论。
- Impact: 同一构建可以因记录文件存在而通过，也可以因视觉不符而被人工判失败；验收结果依赖审阅者临场解释。
- Route: 把每个组合的必查项、PASS/FAIL 结论、失败处理和截图/证据位置写成确定格式；或把“记录存在”降为证据收集项，并另设要求所有必查项通过的 AC。

### TPR-10 · 提示 · task.json 的 base_branch 与当前规划基线未解释

- Task: `cross-task`
- Affected tasks: `08-26-profile-visual-types`, `08-26-visual-type-primitives`, `08-26-visual-type-profiles`, `08-26-visual-type-rollout`
- Location: 四个成员的 `task.json:15-17`
- Claim: 四个任务都记录 `base_branch: main`、`branch: null`，规划没有解释实际实施/差异基线。
- Evidence: 当前检出分支为 `dev`；`git rev-list --left-right --count main...dev` 返回 `0 226`。`ProfileCardGrid.tsx`、`profilePresentation.ts`、`AgentEditModal.tsx`、三平台 `ui-classes.ts` 等计划依据文件在 `main` 均不存在，只在当前 `dev` 基线上存在。
- Impact: 如果后续按 task metadata 生成 PR 或 Pass 7 差异，范围会混入 226 个既有提交或无法以计划所引用的文件为基线；如果仓库约定直接在 dev 实施，则该字段只是未解释的陈旧默认值。
- Route: 在开始前把 base/branch metadata 对齐实际工作流；若 `main` 是刻意的最终 PR 目标，记录如何从 `dev` 取得干净任务 diff。此项是元数据提示，不单独阻止实施。

## 未能核实

- 1440×900 与 900×800、`light|dark` × `neutral|clay` 下的真实层级、对比度、基线和溢出结果 — 任务仍在 planning，目标组件与样式尚未实现；本次只读审阅不能产生未来浏览器截图证据。
- `Badge` 的 computed cursor、Button 的 7×2 视觉矩阵及 `prefers-reduced-motion` 最终行为 — 只能核对计划机制，目标代码尚不存在。
- `just frontend-check-quick` 与 `just ui-check` 的未来结果 — 本次没有产品改动，计划中的实现与测试尚未发生，不能把未来门禁写成已验证。

## 可靠部分

- Pass 0 机械预检通过：根任务和 3 个递归子任务解析唯一、parent backlink 正确、无环/重复成员、4 个 PRD 与复杂任务产物齐全、jsonl 无 `_example`/TBD/坏路径，报告目标受 git ignore 管理。
- 父任务对当前仓库结构的主要定位成立：`ccr-ui/src/ui/index.ts:6-34` 尚未导出 Button/Badge/FieldLabel/UrlText；`.trellis/spec/ccr-ui/frontend/layering-contracts.md:43-50,74-76,92-94` 的 ui-primitive 边界与大小写示例均存在；package manifest 没有把 cva 或 Radix Slot 声明为直接依赖。
- 三份 `features/{codex,opencode,grok}/ui-classes.ts` 的四类按钮定义确实高度同构，且现有消费者可由精确标识符搜索完整枚举。
- Profile 现状断言大体准确：`ProfileFieldSlot` 当前仍是 `chip?: boolean`，四份 presentation 都有 4 个 slot；卡片渲染 4 个字段，表格固定 6 列且不渲染 slot 3；Claude/Codex 搜索仍使用各自原始 `base_url`。
- 数量与单位复核通过：Button 封闭集 7 个变体 × 2 个尺寸 = 14；`light|dark` × `neutral|clay` × 2 viewport = 8；当前 `ccr-ui/src/styles/**/*.css` 自定义属性 unique-name union 重新计算为 452。
- 所需现有 token 均存在，`--color-warning-tint` 已由独立治理任务登记；父子任务明确禁止新增 token 名并把 452 作为集成门。
- 跨子任务顺序不是靠树顺序推断：父 PRD 和 implement.md 都明确写出 primitives 先行，profiles 与 rollout 后续可并行，并给出互斥路径边界。
- 全部成员仍为 `planning`，因此 Pass 7 实现漂移不适用；本次没有把工作区中用户已有的 `.gitignore` / 任务草稿改动当成产品实现。

## 盲区

An agent reviewing an agent's plan is not an independent second opinion. The reviewer and the
author share most of the same blind spots. A clean report means "this pass found nothing", not
"the plan is complete". Treat the findings as a triage list, not as an approval.
