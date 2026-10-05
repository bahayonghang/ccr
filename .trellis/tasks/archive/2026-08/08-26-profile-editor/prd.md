# 统一 Profile 编辑器外壳与三份平台 adapter

父任务：`08-26-profile-design-language`
依赖：`08-26-profile-registry-tokens`

## Goal

用一个模态外壳同时服务三个平台的新建与编辑。分区布局与字段渲染原语按成品稿固定并三平台共用；字段集合、条件必填、认证状态机、序列化与提交由平台 `ProfileEditorAdapter` 提供，各自保留现有语义。

父任务决策 4：统一的是呈现层，不是表单模型。Codex 五种 auth mode 的条件必填矩阵、Claude 两种 auth mode、Grok 四种 credential action 与 official / third-party 分支无法压进一组通用字段 kind，因此保留平台表单模型与校验器。

## Requirements

- R1：单外壳双模式。同一个 `ProfileEditorModal` 处理新建与编辑，通过入参区分。标题、底部提示文案随模式变化。
- R2：分区布局。按成品稿固定——名称与描述两列、Base URL 整行、模型与平台字段两列、认证分组框、标签多选、高级折叠区。分区顺序由 `adapter.sections` 声明，外壳按 `layout` 渲染，不写平台分支。
- R3：字段渲染原语。`ProfileEditorFields` 按 `ProfileEditorFieldKind` 分派：`text`、`mono-text`、`choice`、`secret`、`multi-value`、`boolean`、`number` 七种各一个渲染分支。
- R4：条件可见与条件必填。字段的 `visible(form)` 与 `required(form)` 由 adapter 提供，外壳只调用，不判断平台。
- R5：高级区。`section.advanced` 为真的分区收进可折叠区，默认折叠。折叠区为空时不渲染折叠控件。
- R6：Claude adapter。auth mode 为 `subscription` / `api_key` 两值；`fromRecord` 复用 `fillClaudeProfileForm`，`submit` 复用 `buildClaudeProfileRequest` + `addClaudeProfile` / `updateClaudeProfile`。多档 model 等字段进高级区。
- R7：Codex adapter。auth mode 五值；base URL、secret、env key、model 的条件必填矩阵与 `profiles-page-contracts.md` 的表格逐行一致。`fromRecord` 复用 `codexProfileToEditorForm`，`submit` 复用 `buildCodexProfileRequest(form, resolvedModel)`，`resolvedModel` 由 adapter 内部经 `resolveModelSelection` 算出。派生字段 `requires_openai_auth` / `openai_login_method` / `preferred_auth_method` / `forced_login_method` 的序列化规则不变。
- R8：Grok adapter。`profile_kind` 只读，不从 URL / provider / auth 推断；`base_url_display` 不回填进可写 `baseUrl`；credential action 四值互斥序列化；`fromRecord` 复用 `fillGrokForm`；新建走 `buildGrokCreateRequest`，编辑走 `buildGrokPatch(form, dirtyFields)`，dirty 集合由 adapter 内部持有并维护。
- R9：校验。三份 adapter 各自的 `validate` 返回 `ProfileEditorIssue[]`。Grok 复用 `validateGrokEditor`（含 `hasExistingBaseUrl` 留空放行分支），Codex 按规格矩阵实现，Claude 实现名称非空、名称唯一、`api_key` 模式下 Base URL 与密钥必填。外壳渲染汇总条并支持跳转到出错分段。
- R10：结构化提交结果。`submit` 返回 `ProfileWriteOutcome`。`ok` 关闭模态；`recovery` / `blocked` / `error` 保持模态打开或交由控制器接管，外壳不解释后端 status 字符串。
- R11：密钥处理。`secret` 字段渲染为 `type="password"`，不设 `autoComplete`。`fromRecord` 的入参已经过 `stripCredentials`，密钥字段初值恒为空，编辑模式显示「留空不修改」提示。留空提交时 adapter 不序列化该字段。
- R12：凭据不外泄。密钥值不进入 `console`、错误文案拼接、`data-*` 属性、DOM 非掩码位置与导出内容。
- R13：双出口。「保存」与「保存并应用」两个提交动作，后者在 `submit` 返回 `ok` 后调用 `config.apply(name)`。
- R14：候选值可自由输入。模型与标签的 `options` 只是快捷选项，用户可输入候选外的值。
- R15：无平台名分支。`components/profiles/ProfileEditor*.tsx` 与外壳 hook 不得比较平台名字面量。
- R16：测试落位。新增测试位于 `ccr-ui/tests/*.smoke.test.ts(x)`。

## Acceptance Criteria

- [ ] AC1（R1、R2）：同一个 `ProfileEditorModal` 在 claude / codex / grok 三份 adapter 下均可打开、填写、提交。
- [ ] AC2（R1）：新建模式底部提示为「将追加到 {配置文件}」，编辑模式为「将覆盖 {配置文件} 中的 [{name}]」。
- [ ] AC3（R2、R5）：分区顺序与成品稿一致；`advanced` 分区默认折叠；无 `advanced` 分区时不渲染折叠控件。
- [ ] AC4（R3、R4）：七种 `kind` 各有渲染断言；`visible` 返回假的字段不出现在 DOM，`required` 返回真的字段带必填标记。
- [ ] AC5（R7、R9）：Codex 五种 auth mode × 四类条件必填（base URL / secret / env key / model）的矩阵测试，每种 mode 各一条 good、一条 bad，与 `profiles-page-contracts.md` 的表格逐行对齐。
- [ ] AC6（R7）：Codex 的 `env_key` 只在 `provider_env_key` 模式序列化；bearer 派生字段只在 `provider_bearer_token` 模式序列化；离开该模式后为 `null`。
- [ ] AC7（R8）：Grok 的 credential action 互斥断言——`replace_api_key` 只带 `api_key`，`replace_env_key` 只带 `env_key`，`preserve` 与 `clear` 两个值字段都不带。
- [ ] AC8（R8）：Grok 只改 reasoning effort 时 patch 为 `{ reasoning_effort: 'high' }`，不含 URL 与凭据字段；`base_url_display` 不出现在任何 patch 或 create request 中。
- [ ] AC9（R8）：Grok official profile 不渲染 provider、base URL、凭据控件；`profile_kind` 为只读展示。
- [ ] AC10（R6、R9）：Claude 名称为空、名称与既有 profile 重名、`api_key` 模式下 Base URL 为空或密钥为空四种情况均被拦截并给出分段级提示。
- [ ] AC11（R9）：外壳的汇总条列出全部 issue，点击 issue 跳转到对应分段。
- [ ] AC12（R11）：`fromRecord` 对三平台的 sanitized DTO 夹具各跑一次，断言返回表单的密钥字段为空串。
- [ ] AC13（R11）：密钥留空提交时，Claude 与 Codex 的 request 不含 `auth_token` 字段，Grok 的 patch 不含 `api_key` 与 `env_key`。
- [ ] AC14（R12）：以随机 sentinel 作为密钥输入并提交失败，断言 sentinel 不出现在 DOM 文本、`console` 调用参数、错误文案与 `data-*` 属性四处。
- [ ] AC15（R10）：`submit` 返回 `recovery` / `blocked` / `error` 时模态不关闭且不调用 `apply`；返回 `ok` 时关闭。四条各一测试。
- [ ] AC16（R13）：「保存并应用」在 `submit` 返回 `ok` 后调用 `config.apply`，返回非 `ok` 时不调用。
- [ ] AC17（R14）：模型与标签均可输入 `options` 之外的值并进入提交 payload。
- [ ] AC18（R15）：`tests/platform-surface-unify.smoke.test.ts` 的无平台名分支断言覆盖本任务新增文件并通过。
- [ ] AC19（R1-R15）：明暗两套主题下、`neutral|clay` 两种 flavor 下各打开一次新建与编辑，检查分区边界、认证分组框内嵌底色、chip 选中态、主按钮对比度；编辑器相关文件中无硬编码 hex。
- [ ] AC20（R1-R15）：新增文案在 `zh-CN` 与 `en-US` 中均存在，`bun run check:i18n` 通过。
- [ ] AC21（R16）：新增测试文件为 `tests/profile-editor-shell.smoke.test.tsx` 与 `tests/profile-editor-adapters.smoke.test.ts`；`rg -l "smoke.test" ccr-ui/src` 为空。
- [ ] AC22（R1-R16）：`just frontend-check-quick` 通过；`tests/grok-profile-editor.smoke.test.ts` 仍通过。

## Constraints

- 不删除 `GrokProfileEditorModal.tsx` 及其相关文件，退役由 rollout 执行。
- 不改路由，不改 `src/configs/profiles.ts` 已有字段，不改 Tauri 命令签名，不改后端凭据序列化。
- 不改 `features/{claude,codex,grok}/*ProfilesView.tsx`。
- 不重写 `utils/{claude,codex,grok}ProfileEditor.ts` 与 `grokEditorValidation.ts` 的规则语义。adapter 是这些实现体的包装层，不是替代。
- 样式只写 `components/profiles/profile-editor-shell.css`，共享原子类引用 `08-26-profile-list-surface` 在 `profiles-shared.css` 中落地的类名。
- 遮罩与阴影走 token（`--color-scrim` 系列），不写 rgba 字面量。
- 模态外壳复用 `src/ui` 下既有的 Dialog 原语，不新造。限高与唯一滚动根按 `profiles-page-contracts.md` 的 `pe-shell` / `pe-scroll` 约定。

## Notes

- 表单结构、字段顺序、提示文案的权威来源是 `../08-26-profile-design-language/research/design-source.md` 的「表单弹窗」一节。
- Claude 与 Codex 目前完全没有新建/编辑 UI（`BaseProfiles` 中 `onAdd` / `onEdit` 传 `noop`），本任务是这两个平台该能力的首次落地。
- `claudeProfilesConfig.create` / `update` 的入参类型 `ProfileDraft` 只有四个字段，装不下真实 request。adapter 的 `submit` 直接调用 `addClaudeProfile` / `updateClaudeProfile` 等 API 函数，不经过 `ProfilesConfig`。
- `grokProfilesConfig` 没有 `create` / `update`，Grok 的写入本来就只能走 adapter 内部的 `grokApi`。
- 本任务是审阅报告 TPR-02、TPR-03、TPR-04 的主要落地入口。
