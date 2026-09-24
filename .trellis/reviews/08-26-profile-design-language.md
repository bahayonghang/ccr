---
skill: trellis-plan-review
version: 0.3.0
task_dir: D:/Documents/Code/Github/ccr/.trellis/tasks/08-26-profile-design-language
task_name: 08-26-profile-design-language
task_status: planning
verdict: 需返回规划
blocking: 12
should_fix: 2
notes: 2
generated_at: 2026-08-26T09:17:49.4881536+08:00
---

# Trellis 规划审阅报告

## 结论

需返回规划 — 阻断 12 / 应修 2 / 提示 2

本报告覆盖父任务 `08-26-profile-design-language` 及四个 `planning` 子任务：`08-26-profile-registry-tokens`、`08-26-profile-list-surface`、`08-26-profile-editor`、`08-26-profile-rollout`。由于全部任务仍为 `planning`，Pass 7 实现漂移不适用。

## 问题清单

### TPR-01 · 阻断 · 通用读取投影丢失编辑器和 Grok 展示所需字段

- Location: 父任务 `design.md:29-83`；`08-26-profile-registry-tokens/design.md:20-53`；`08-26-profile-editor/design.md:18-41`；`08-26-profile-rollout/design.md:21-30`
- Claim: `ProfilePresentation.fromRecord(profile: ProfileRecord)` 能回填三平台编辑器，`fieldSlots` 还能展示 Claude/Codex/Grok 的平台专属字段。
- Evidence: `ccr-ui/src/configs/profiles.ts:26-34` 的 `ProfileRecord` 只有 `name/description/enabled/tags/model/baseUrl/authMode`；`toProfile()` 在 `ccr-ui/src/configs/profiles.ts:84-98` 丢弃其他字段，三份 `list()` 在 `:128-133`、`:168-175`、`:201-206` 都经过该投影。真实 `GrokProfileDto` 还含 `profile_kind/base_url_display/context_window/reasoning_effort/env_key/has_inline_credential`（`ccr-ui/src/types/generated/grok/GrokProfileDto.ts:5`），Claude/Codex 的高级字段分别见 `ccr-ui/src/types/claude.ts:53-85`、`ccr-ui/src/types/codex.ts:70-101`。
- Impact: Grok 的 `profile_kind`、recovery/activation 相关展示和三平台高级编辑字段无法由计划中的 `ProfileRecord` 恢复；按计划实现会显示空值，或编辑保存时丢失未投影字段。
- Route: 二选一：让统一读取契约携带平台原始 typed record，并由 presentation 做只读投影；或为展示快照与编辑详情分别定义 typed loader。无论哪条路线，都需明确每个平台字段的来源、空值语义及往返测试。

### TPR-02 · 阻断 · 单一 `toDraft` 不能表达三平台的创建、更新和 dirty-patch 语义

- Location: 父任务 `design.md:41-81`；`08-26-profile-editor/design.md:18-42,101-104`；`08-26-profile-editor/implement.md:22-28`
- Claim: `presentation.toDraft(form)` 后统一调用 `config.create(draft)` 或 `config.update(target.name, draft)`，即可吸收三平台签名差异并复用现有 utils。
- Evidence: Claude 的 `fillClaudeProfileForm` 是就地写入且参数为完整 `ClaudeProfile`（`ccr-ui/src/utils/claudeProfileEditor.ts:63-98`）；Codex 的 `buildCodexProfileRequest` 还要求第二个 `resolvedModel` 参数（`ccr-ui/src/utils/codexProfileEditor.ts:143-173`）；Grok 分成 `buildGrokCreateRequest` 与 `buildGrokPatch(form, dirtyFields)`（`ccr-ui/src/utils/grokProfileEditor.ts:110-176`）。`grokProfilesConfig` 在 `ccr-ui/src/configs/profiles.ts:155-187` 根本没有 `create`/`update`，而现有更新流程明确把 `react-hook-form` 的 dirty fields 交给 `buildGrokPatch`（`ccr-ui/src/features/grok/profiles/useGrokProfilesPage.ts:129-137`）。
- Impact: Grok 无法按计划提交；若强行转成全量 draft，会破坏 absent=preserve、`null`=clear 的 patch 语义并覆盖未编辑字段。
- Route: 为 config/presentation 定义独立的 create 与 update adapter，更新 adapter必须能接收 dirty-field/credential-action 上下文和结构化响应；或保留平台 editor controller，仅共享纯呈现外壳。为三平台分别锁定创建、同名更新、重命名、未编辑字段保留测试。

### TPR-03 · 阻断 · 编辑器 schema 与认证/校验模型无法表示真实平台契约

- Location: 父任务 `design.md:41-78`；`08-26-profile-registry-tokens/design.md:43-53`；`08-26-profile-editor/prd.md:12-20`；`08-26-profile-editor/design.md:43-84`
- Claim: 四种字段 kind 加通用 `api_key/oauth/no_auth` 段控件、统一的 name/Base URL 校验，即可覆盖 Claude、Codex、Grok。
- Evidence: Codex 实际有五种 auth mode（`ccr-ui/src/types/codex.ts:61-66`），其 Base URL、secret、env key、model 都是按 auth mode 条件校验（`.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md:71-83`）；Claude 只有 `subscription/api_key`（`ccr-ui/src/types/claudeProfileEditor.ts:8-10`）；Grok 使用 `preserve/replace_api_key/replace_env_key/clear` credential action，并区分 official/third-party（`ccr-ui/src/utils/grokProfileEditor.ts:27-43,101-176`）。计划的 `kind` 也没有 multi-value/boolean/number/conditional 分支，却要求标签多选、`supportsBackendSearch`、`contextWindow` 等字段。`design.md:62` 还规定与 Grok 既有规则冲突时“以既有实现为准”，直接与 R6 的统一必填/唯一性规则冲突。
- Impact: 有效的 official/OpenAI profile 会被错误拦截，provider env/bearer 与 Grok credential action 无法录入，标签和高级字段无法可靠序列化；AC26-31 没有可执行机制。
- Route: 让 registry 声明平台认证状态机、条件必填、字段 cardinality/type 与序列化责任；或保留平台专属表单模型/validator，仅复用布局组件。同步改写逐平台 good/base/bad 测试矩阵。

### TPR-04 · 阻断 · “后端返回掩码密钥”这一前提为假，泄露验收也没有闭环

- Location: 父任务 `prd.md:64-69`；`08-26-profile-editor/prd.md:15-17,28-29`；`08-26-profile-editor/design.md:64-69`；`08-26-profile-editor/implement.md:57-59`
- Claim: 后端返回的是掩码值，因此编辑时清空 secret 即可；API key 不会进入日志、错误信息或 DOM 非掩码位置。
- Evidence: Claude 与 Codex 当前序列化均显式 `Secret::expose` 原文，注释还写明“编辑表单预填需要原文”（`ccr-ui/src-tauri/src/commands/claude.rs:479-491`、`ccr-ui/src-tauri/src/commands/codex.rs:1617-1622`）。计划只要求人工“确认” console/data 属性，并在 `design.md:69` 原样展示后端错误，没有定义 sentinel、日志/错误扫描或平台 read sanitizer。
- Impact: 规划建立在错误的安全边界上；一旦为解决 TPR-01 改用完整 record，明文凭据可能进入表单状态、DOM、错误或测试快照，AC28-29 无法证明。
- Route: 明确 typed read adapter 在进入 UI 前剥离明文 secret，定义空 secret 的 preserve 语义，并增加随机 sentinel 的 DOM、console、toast/error、导出与 payload 测试；或先单独完成后端 masked DTO，再接统一编辑器。

### TPR-05 · 阻断 · Grok 能力盘点漏掉安全删除和状态信封分支

- Location: `08-26-profile-rollout/prd.md:12-17,21-24`；`08-26-profile-rollout/design.md:21-31`；`08-26-profile-rollout/implement.md:12-28`
- Claim: Grok 旧路径只有 `profile_kind`、启停切换、recovery 提示三项需承接，完成后即可删除旧 hook/page。
- Evidence: 现有 `useGrokProfilesPage` 还处理 Local-only fail-closed、`active/drifted/unsafe_missing_entry_state`、blocked delete 的单次 force 分支、rename apply/cleanup recovery 等状态（`ccr-ui/src/features/grok/profiles/useGrokProfilesPage.ts:23-41,129-162,189-250,285-321`）。权威规格明确要求这些分支及测试（`.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md:259-324`）。通用 `ProfilesConfig.remove` 只返回 `Promise<void>`（`ccr-ui/src/configs/profiles.ts:70-76`），当前 Grok wrapper 也直接丢弃 action response（`:178-186`）。
- Impact: 按删除门执行会静默丢失 unsafe delete 禁止强制、blocked-force 不循环、rename recovery 时序和 Local-only pin 保留等已受保护行为。
- Route: 先列全 Grok 状态/动作矩阵并为 config 定义结构化结果 adapter；每个分支有 R、AC 和现有 smoke/Tauri 测试映射后再允许删除。否则保留 `useGrokProfilesPage` 作为平台 controller。

### TPR-06 · 阻断 · 默认删除/改位方案与现行 Profiles 契约冲突，且无被批准的行为变更验收

- Location: 父任务 `design.md:118-132`；`08-26-profile-list-surface/design.md:63-85`；`08-26-profile-rollout/design.md:33-60`
- Claim: QuickRail、Inspector、CommandPalette 可默认删除，状态/provider/排序筛选可移除，profile-off 可移入页头；R10 只要消除零消费组件即可。
- Evidence: 现行规格把 `ProfilesQuickRail`、Filters popover、Inspector 和 Off 横幅列为共享骨架契约，并规定 Off 仅在后端 `can_off === true` 时显示、位于 Header 与 StatStrip 之间且 `type=warning`，不得放进 Header 菜单（`.trellis/spec/ccr-ui/frontend/profiles-page-contracts.md:143-163,198-220`）。当前 `ProfilesSnapshot` 不携带 `can_off`（`ccr-ui/src/configs/profiles.ts:43-46`）。规划没有新增“接受删除这些能力”的 requirement/AC，也没有把 `profiles-page-contracts.md` 放入 change list；rollout `design.md:60` 还允许保留零消费组件并临时改写 R4 验收口径。
- Impact: checker 按 manifest 读取规格时会与实现目标直接冲突；用户可见快捷切换、筛选、检查器和 Off 安全语义可能无验收地退化，R10 也可在同一实现上既通过又失败。
- Route: 二选一：把这些能力纳入新页面并保留现行规格；或把每项删除/改位写成明确需求与可观察 AC，并在同一任务中更新规格、测试和 `can_off` 数据流。不得在 rollout 运行时临时改变 AC。

### TPR-07 · 阻断 · Raw Editor 的保留结论没有数据、状态或安全接线机制

- Location: 父任务 `design.md:127-132`；`08-26-profile-list-surface/design.md:47-64`；`08-26-profile-rollout/design.md:62`；`08-26-profile-rollout/implement.md:30-35`
- Claim: `ProfilesRawEditorPanel` 保留并通过页头 `onEditSource` 接入即可完成原始配置出口。
- Evidence: 组件实际要求 `getRaw/saveRaw/onSaved/onClose`（`ccr-ui/src/components/profiles/ProfilesRawEditorPanel.tsx:29-39,76-84`），而 `ProfilesConfig` 没有 raw handler 或 source-mode state（`ccr-ui/src/configs/profiles.ts:57-77`）。Raw Config 规格还要求进入前明文警告、version token/conflict/activation-conflict 处理及保存后 full refresh（`.trellis/spec/ccr-ui/frontend/raw-config-editor-contracts.md:17-39`）；rollout 只有“一行接入”，没有这些机制和测试。
- Impact: 该组件无法由计划中的通用页面实际组装；即使临时接上 API，也可能跳过凭据警告、冲突保护或保存后全量刷新。
- Route: 在 config 中定义 typed raw-source capability 和 source-mode controller，逐项映射 Raw Config contract 与 focused tests；或撤销“保留并接入”决定并给出另一个经过验收的原始编辑出口。

### TPR-08 · 阻断 · “新增平台只需一份 registry”与两层真实架构及测试样例相互矛盾

- Location: 父任务 `prd.md:5,38,53-58`；父任务 `design.md:81-83`；`08-26-profile-rollout/prd.md:16,27`；`08-26-profile-rollout/design.md:72-76`
- Claim: 新增平台只需一份 registry 条目，无需新增页面组件；antigravity 可证明该结论。
- Evidence: 父任务 `design.md:83` 自己写明成本是“一条 `ProfilePresentation` + 一条 `ProfilesConfig`”；rollout 测试也显式使用 `antigravity presentation + mock config`。现行平台规范还要求一条 `platformSurfaceDescriptors` row 加每个 surface module 的 export（`.trellis/spec/ccr-ui/frontend/platform-surface-contracts.md:24-41,71-74`）。Grok shell/subnav/recovery 差异又在 rollout `design.md:17-20` 通过 props/config，而非单一 presentation registry 表达。
- Impact: 父任务 AC57 按文字不可通过；mock config 测试只能证明组件可注入，不能证明真实新增平台只改一处。
- Route: 将验收口径改为仓库既有的 descriptor + per-surface config + presentation 两层成本；或真正合并为一个注册源并由它生成 descriptor/config/presentation，随后测试真实注册链而不是手写 mock。

### TPR-09 · 阻断 · 计划中的核心测试文件不会被当前 Vitest 门发现

- Location: `08-26-profile-registry-tokens/design.md:3-13,101-109`；其余三个子任务 `design.md` 的测试章节与 change list
- Claim: `ccr-ui/src/configs/__tests__/profilePresentation.test.ts` 会保护结构、往返和 antigravity 可扩展性，`just frontend-check-quick`/`just ui-check` 可作为最终门。
- Evidence: `ccr-ui/vitest.smoke.config.ts:31-35` 只包含 `tests/**/*.smoke.test.{ts,tsx}`；`ccr-ui/package.json:30-32` 与 `justfile:758-800` 的 frontend test/quick gate只运行该 config。registry 子任务把测试放到未被发现的 `src/configs/__tests__`；list/editor/rollout 的 change list 又没有给出任何测试文件路径。
- Impact: 核心 AC 可以在测试从未执行的情况下显示门禁全绿；实现者也无法据 change list 确认需要新增/修改哪些现有 Profiles smoke tests。
- Route: 把所有契约测试落到 `ccr-ui/tests/*.smoke.test.ts(x)` 并列入各 design change list，或显式扩展测试 config/命令；给出逐任务 focused command，同时保留最终 quick/ui gate。

### TPR-10 · 阻断 · 视图模式持久化允许静默降级且没有验收标准

- Location: 父任务 `prd.md:43`；`08-26-profile-list-surface/prd.md:19,22-33`；`08-26-profile-list-surface/design.md:36-43,129-132`
- Claim: `viewMode` 按平台在会话内保持/持久化；若既有 UI 状态存储不支持则退化为组件 state。
- Evidence: `ccr-ui/src/api/domains/uiState.ts:1-15` 只有收藏与最近项 API，并无通用 view state；仓库真正的逐平台持久化先例是 `ccr-ui/src/features/profiles/stores.ts:23-70` 的 localStorage + Zustand。组件 state 在路由卸载后不满足“会话内保持”。同时 list-surface 的十条 AC 没有一条验证持久化或按平台隔离。
- Impact: 实现可按 design 的 fallback 完成并勾完全部 AC，却违反父 R6/子 R8；同一会话返回页面时视图选择会丢失。
- Route: 复用/扩展已有 Profiles Zustand+localStorage 模式并增加按平台、卸载重挂载、storage 不可用降级测试；或明确放弃持久化并同步修改 requirement/AC，不能静默降级。

### TPR-11 · 阻断 · 平台色计划漏掉 antigravity 基础 token，且新增名称治理不完整

- Location: `08-26-profile-registry-tokens/prd.md:11-17,21-27`；`design.md:55-109`；`implement.md:20-32`
- Claim: 六个平台都将具有 dot/surface/border/text 四角色，antigravity 可作为完整 presentation 样例。
- Evidence: 当前 `tokens.css` 的平台色只有 claude/codex/grok/gemini/opencode（`ccr-ui/src/styles/tokens.css:107-117`），不存在 `--color-platform-antigravity` 或 `-rgb`。实施步骤只为六平台补后三角色，并只更新 codex/grok 的 dot/rgb，没有任何一步新增 antigravity dot/rgb。与此同时，新增至少 18 个 token 名称，却未按 `.trellis/spec/ccr-ui/frontend/theme-token-contracts.md:24-49` 登记名称增量、更新冻结段、判断 `core.css`/四作用域/bridge 归属及运行规定的主题测试；`design.md:89` 还允许 `color-mix()`，但测试机制只说解析 CSS 后自行算 WCAG，没有定义如何求值 `color-mix()`。
- Impact: AC24 的 antigravity 四角色必然缺一，token checker 无法按现有规划验收；即使补值，也会留下治理规格漂移或不可计算的对比度测试。
- Route: 把 antigravity dot/rgb 纳入明确 change list；为所有新名称完成 name-delta、spec 冻结更新、作用域/映射/bridge 判定与 focused theme suite。对明色取值锁定为可直接计算的 hex，或用真实 CSS computed-style 求值后再算对比度。

### TPR-12 · 阻断 · 父任务的 implement/check 上下文仍是模板空载体

- Location: 父任务 `implement.jsonl:1`、`check.jsonl:1`
- Claim: 父任务负责跨子任务门禁与最终集成审查。
- Evidence: 两份文件都只含 `_example`。`plan_precheck.py` 对父任务退出 1 并报告两项 blocking；`task.py validate 08-26-profile-design-language` 却显示 `implement.jsonl: ✓ (0 entries)`、`check.jsonl: ✓ (0 entries)`，说明结构 validator 的绿灯没有提供任何集成上下文。
- Impact: 父任务若进入执行/检查，子 agent 收不到父 PRD、设计源、关键 Profiles/Theme/Raw Editor 规格；“最终集成出口”没有可执行上下文。
- Route: 删除 `_example` 并填入父任务真正需要的 task/spec/research 路径，随后同时跑 precheck 与 validate；不要把 validate 的 0-entry 绿灯当作 readiness。

### TPR-13 · 应修 · 供应商去重的“host”算法和验收口径未定义

- Location: 父任务 `prd.md:41`；父任务 `design.md:110-114`；`08-26-profile-list-surface/prd.md:13,25`；`08-26-profile-list-surface/design.md:36-41,123-127`
- Claim: 去协议后取第一段并用 `Set` 就等于 Base URL host 去重计数。
- Evidence: 计划表达式 `stripProtocol(p.baseUrl).split('/')[0]` 没有定义大小写、默认端口、userinfo、IPv6、尾点和无协议输入的规范化；测试只列“空值与带路径”，没有这些等价/非等价边界。`baseUrl` 当前只是可空字符串（`ccr-ui/src/configs/profiles.ts:26-34`），计划也没有 URL 格式校验。
- Impact: 相同供应商可能因大小写/端口写法被重复计数，或非法/带凭据 authority 被当作供应商；AC25 在边界数据上没有唯一的预期。
- Route: 明确“供应商”的 canonical key（例如经 URL 解析后的 hostname，是否保留端口也需决定）及非法输入策略，并补等价类/边界测试；或把文案改为与简单字符串算法一致的口径。

### TPR-14 · 应修 · 视觉与响应式验收缺少可复现前置条件，也漏了中间态旧页面回归

- Location: 父任务 `prd.md:55,59`、`implement.md:37-45`；list-surface `prd.md:29-33`、`implement.md:60-63`；editor `prd.md:33-35`、`implement.md:62-64`；rollout `prd.md:28-31`、`implement.md:55-66`
- Claim: “明暗主题渲染正确/符合成品稿层级”“窄窗口自身横向滚动且 body 不滚动”“逐页走查通过”可给出确定 pass/fail。
- Evidence: 所有手工步骤都未给 viewport、zoom、数据夹具、主题/flavor/accent dataset、滚动位置、测量方法、截图/计算样式锚点。list-surface 会重写全局 `profiles-shared.css`，而该 CSS 被当前仍在线的 `ProfileListRow`、`ProfilesHeader` 等直接 import（如 `ccr-ui/src/components/profiles/ProfileListRow.tsx:4`、`ProfilesHeader.tsx:4`）；该子任务只验证新组件，没有打开仍由 `BaseProfiles` 渲染的 Claude/Codex 页面。
- Impact: 同一构建可在不同窗口/数据下既通过又失败；前序 child commit 也可能在 rollout 前破坏现有页面而门禁仍绿。
- Route: 固定至少宽/窄 viewport、zoom、主题×flavor、代表性长文本/多标签数据、检查的 computed style/scrollWidth 判据与截图路径；list-surface 完成时额外走查当前 Claude/Codex 旧页面，rollout 再走统一页面。

### TPR-15 · 提示 · R/AC 写法没有被审阅工具识别，显式追溯标注也缺失

- Location: 五份 `prd.md` 的 Requirements 与 Acceptance Criteria 章节
- Claim: 现有任务已经能由工具验证 R/AC 交叉引用。
- Evidence: 五次 `plan_precheck.py` 均报告 `requirements=0 criteria=0`，同时把正文引用的 `R1...` 列为 referenced-but-undefined；原因是当前使用 `- **R1 ...**` 和无 `(R1,R2)` 注解的 checklist，未采用 parser 识别的定义/注解语法。语义人工追溯仍发现 TPR-10 等缺口。
- Impact: 后续修订即使漏掉某个 requirement/criterion，机械预检也不会报警；`task.py validate` 的全绿只证明 JSONL carrier 结构。
- Route: 按项目/技能可识别格式定义 R/AC，并为每条 AC 标注所证明的 requirement；修订后确认 precheck 的非零 R/AC 数与人工清单一致。

### TPR-16 · 提示 · 五个任务的 `dev_type` 均为空

- Location: 五份 `task.json` 的 `dev_type`
- Claim: 无；这是审阅适用 pass 的元数据缺口。
- Evidence: 父任务与四个子任务均为 `"dev_type": null`，而交付内容明显包含 feature/refactor、删除与跨平台迁移。
- Impact: 后续工具/审阅者无法由 task metadata 选择 feature scope-boundary 检查；不会直接改变产品行为。
- Route: 选择仓库支持且最贴近本树的 dev type，并在父/子一致性需要时同步；若 Trellis 有意允许 null，则在 notes 解释。

## 未能核实

- Claude Design 项目 `0a3d3dfa-8ad5-4bdf-861d-305f1e2c6389` 与 `CCR UI Profile 成品稿.dc.html` 的原始内容是否和 `research/design-source.md` 完全一致 — 本会话没有可调用的 DesignSync/登录态源文件，只能核对本地提取物及其内部使用。
- Claude slot3 候选字段在真实 profile 样本中的填充率与 `account` 敏感性 — 真实用户配置不在审阅范围，且计划尚未产出 `notes.md` 结论。
- 明暗主题、窄窗口、键盘焦点、滚动、reduced motion 与 WebView2/Tauri 下的真实视觉行为 — 当前只有 planning artifacts，没有实现可运行。
- 用户是否明确批准删除 QuickRail、Inspector、Provider/排序筛选等已规格化能力 — 五个任务工件中没有这项批准记录。

## 可靠部分

- 五个 `task.json` 都是 `planning`，父子关系与顺序（registry-tokens → list/editor → rollout）明确写入 task metadata、父 PRD 和 child 前置条件；Pass 7 正确跳过。
- 现状表的三条页面入口、Claude/Codex `noop` 新建编辑、Grok 独立页面路径与列出的主要文件行数均与当前仓库一致；`tokens.css:108-117` 引用也解析到正确构造。
- 四个子任务的 JSONL 都已替换为真实且存在的 task/spec/research 路径，`task.py validate` 对四者分别识别到非零 entries；只有父任务仍为空模板。
- 路由不变、Tauri 命令签名不变、先接线验证再删除、删除单独提交等边界在各工件中一致，回滚粒度与任务依赖基本匹配。
- 设计源提取物明确记录了暗色-only、Claude 无 `last_used`、模型/标签必须自由输入、平台切换器不落地、导入改用仓库真实能力等偏差，避免了直接照抄原型。
- `useQuery` key 继续沿用 `['platform-profiles', config.cacheKey]` 的断言与当前 `BaseProfiles` 实现一致（`ccr-ui/src/features/platform/profiles/BaseProfiles.tsx:24-33`）。

## 盲区

An agent reviewing an agent's plan is not an independent second opinion. The reviewer and the
author share most of the same blind spots. A clean report means "this pass found nothing", not
"the plan is complete". Treat the findings as a triage list, not as an approval.
