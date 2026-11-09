# 统一 Profile 编辑器外壳与三份平台 adapter — 技术设计

`ProfileEditorAdapter` 的类型定义在父任务 `design.md`「契约三」。本文件写外壳实现与三份 adapter 的映射。

## 文件边界

新增：

- `components/profiles/ProfileEditorModal.tsx` — 模态外壳、分区装配、汇总条、底部动作栏
- `components/profiles/ProfileEditorFields.tsx` — 七种 `kind` 的渲染分派
- `features/platform/profiles/useProfileEditor.ts` — 表单状态、校验调用、提交编排
- `features/claude/profiles/claudeProfileEditorAdapter.ts`
- `features/codex/profiles/codexProfileEditorAdapter.ts`
- `features/grok/profiles/grokProfileEditorAdapter.ts`
- `ccr-ui/tests/profile-editor-shell.smoke.test.tsx`
- `ccr-ui/tests/profile-editor-adapters.smoke.test.ts`

改造：

- `components/profiles/profile-editor-shell.css` — 按新结构重写
- `ccr-ui/tests/fixtures/profiles.ts` — 补 sanitized DTO 夹具（typed 部分由 registry-tokens 建立）
- `ccr-ui/src/i18n/locales/zh-CN/*`、`en-US/*`

不动：`utils/{claude,codex,grok}ProfileEditor.ts`、`features/grok/profiles/grokEditorValidation.ts`、`GrokProfileEditorModal.tsx`、`features/{claude,codex,grok}/*ProfilesView.tsx`、`src/configs/profiles.ts`。

adapter 放在 `features/{platform}/profiles/` 而不是 `configs/`：它们要 import `@/api` 的平台函数与 `utils/*ProfileEditor.ts`，`configs/` 不允许这个依赖方向（`layering-contracts.md`）。`configs/profileEditorAdapter.ts` 只有类型。

## useProfileEditor

```ts
export function useProfileEditor(args: {
  adapter: ProfileEditorAdapter;
  presentation: ProfilePresentation;
  /** null 为新建；否则为被编辑的 sanitized 记录 */
  target: unknown | null;
  originalName: string | null;
  existingNames: readonly string[];
  hasExistingBaseUrl: boolean;
  onDone(outcome: ProfileWriteOutcome, applied: boolean): void;
}): {
  form: unknown;
  issues: readonly ProfileEditorIssue[];
  saving: boolean;
  submitError: string | null;
  setField(key: string, value: unknown): void;
  submit(apply: boolean): Promise<void>;
};
```

初始化：新建 `adapter.createEmpty()`；编辑 `adapter.fromRecord(target)`。`target` 已由平台控制器经 `stripCredentials` 处理，因此密钥字段自然为空，外壳不需要额外清理。

`setField` 除写值外还把 key 记入 dirty 集合。dirty 集合传给 `adapter.submit` 的上下文——Grok 的 `buildGrokPatch` 需要它。

提交编排：

1. `adapter.validate(form, ctx)`；非空则写 `issues`，不发请求。
2. `adapter.submit(form, { isEditing, originalName, apply, dirtyFields })`。
3. 结果为 `ok` 且 `apply` 为真时调 `config.apply(form.name)`。
4. `ok` → `onDone(outcome, applied)` 并关闭；其余三种 status 保持打开，`error` 写 `submitError`，`recovery` / `blocked` 原样上抛给控制器。

外壳不解析后端 status 字符串，只按 `ProfileWriteOutcome` 的四个 tag 分支。

## 三份 adapter 的映射

### claude

| 契约项       | 实现                                                                                     |
| ------------ | ---------------------------------------------------------------------------------------- |
| `createEmpty` | `createClaudeProfileForm()`                                                              |
| `fromRecord` | 新建空表单后 `fillClaudeProfileForm(form, sanitizedProfile)`；`auth_token` 因剥离恒为 `''` |
| `sections`   | identity（name、description）/ connection（base_url、auth_mode、auth_token）/ runtime（model 系列）/ advanced（多档 model、effort、timeout、provider、account） |
| `validate`   | name 非空、name 唯一（编辑时允许自身）、`auth_mode === 'api_key'` 时 base_url 与 auth_token 必填 |
| `submit`     | `buildClaudeProfileRequest(form)` → `addClaudeProfile` / `updateClaudeProfile`            |

`buildClaudeProfileRequest` 当前恒把 `auth_token` 写入 request。密钥留空时 adapter 在 request 上删除该键后再发送（AC13），不改 `utils` 文件。

### codex

| 契约项       | 实现                                                                                       |
| ------------ | -------------------------------------------------------------------------------------------- |
| `createEmpty` | `createCodexProfileEditorForm()`                                                            |
| `fromRecord` | `codexProfileToEditorForm(sanitizedProfile)`；`auth_token` 因剥离恒为 `''`                   |
| `sections`   | identity / connection（base_url、auth_mode、auth_token、env_key）/ runtime（model、wire_api、reasoning effort）/ advanced（model_catalog_json、bearer 派生字段、sandbox、approval） |
| `visible`    | `env_key` 仅 `provider_env_key`；bearer 派生字段仅 `provider_bearer_token`；base_url 在 `usesOpenAiAuthMode` 时隐藏 |
| `required`   | 按 `profiles-page-contracts.md` §4 的表：`requiresBaseUrl = !usesOpenAiAuthMode`、`requiresSecret` 覆盖三种模式、`requiresEnvKey = provider_env_key`、model 恒必填 |
| `validate`   | 五条规则映射到 identity / auth / runtime 三个 section，文案 key 沿用 `codex.profiles.validation.*` |
| `submit`     | `resolveModelSelection` 得 `resolvedModel` → `buildCodexProfileRequest(form, resolvedModel)` → `addCodexProfile` / `updateCodexProfile` |

deprecated auth mode（`openai_chatgpt`、`provider_env_key`）在 `options` 中按 `isDeprecatedAuthMode` 标注，既有 profile 仍可保持该模式，新建不推荐。

### grok

| 契约项       | 实现                                                                                                   |
| ------------ | -------------------------------------------------------------------------------------------------------- |
| `createEmpty` | `createEmptyGrokForm()`                                                                                  |
| `fromRecord` | `fillGrokForm(dto)`。该函数已把 `baseUrl` 置空且不复制 `base_url_display`，符合规格                       |
| `sections`   | identity（name、description、profile_kind 只读）/ connection（baseUrl、credentialAction、apiKey、envKey）/ runtime（model、reasoningEffort、apiBackend、contextWindow、supportsBackendSearch）/ status（enabled、tags） |
| `visible`    | `profileKind === 'official'` 时 connection 与 provider 字段整体隐藏；`apiKey` 仅 `replace_api_key`；`envKey` 仅 `replace_env_key` |
| `validate`   | 直接调 `validateGrokEditor({ form, editingName, hasExistingBaseUrl, t })`，返回值 section 已对齐          |
| `submit`     | 新建 `addGrokProfile(buildGrokCreateRequest(form))`；编辑 `updateGrokProfile(name, buildGrokPatch(form, dirtyFields))` |

`submit` 的返回映射：

| `GrokProfileActionResponse.status`             | `ProfileWriteOutcome`                                  |
| ---------------------------------------------- | -------------------------------------------------------- |
| `created` / `updated` / `renamed`              | `{ status: 'ok' }`                                       |
| `rename_apply_failed` / `rename_cleanup_failed` | `{ status: 'recovery', kind: status, message }`          |
| `blocked`                                      | `{ status: 'blocked', message, forceAllowed }`           |
| `unsupported_environment`                      | `{ status: 'error', message }`                           |

`recovery` 由 `useGrokProfilesPage` 接管，recovery 提示条与重试逻辑保持现有实现，本任务不重写。

## 布局

外层用 `src/ui` 下既有的 Dialog 原语。面板宽 720px，`max-height: 88vh`。限高与唯一滚动根按 `profiles-page-contracts.md` 的约定：`pe-shell` 放 Dialog 默认槽并加 `max-h` + `overflow-hidden`，`.pe-scroll` 是壳内唯一 `overflow-y: auto` 根，汇总条与 footer 留在壳内不进滚动区，不开 Dialog 自身的 `scrollable`。

正文按 `adapter.sections` 顺序渲染，外壳按 `section.layout` 决定容器：

- `grid` → 两列
- `row` → 整行
- `group` → 带边框的分组框（认证区）

`advanced` 为真的 section 收进折叠区，默认折叠。

底部：左侧提示文案（随模式变化）+ 取消 + 保存 + 保存并应用（accent 主按钮）。保存中禁用三个按钮并在主按钮显示进行态。

## 字段渲染

`ProfileEditorFields` 按 `kind` 分派七个分支：

| kind          | 渲染                                                            |
| ------------- | ----------------------------------------------------------------- |
| `text`        | 单行输入                                                          |
| `mono-text`   | 单行输入 + `--font-mono`                                          |
| `choice`      | chip 快捷选项 + 自由输入框；chip 高亮由输入值等于候选值决定        |
| `secret`      | `type="password"`，无 `autoComplete`，编辑模式带「留空不修改」提示 |
| `multi-value` | 标签多选（chip 选中 + 自由输入追加）                              |
| `boolean`     | 开关                                                              |
| `number`      | `inputMode="numeric"` 的单行输入，非数字由 adapter 的 validate 拦截 |

`choice` 与 `multi-value` 满足 R14：候选只是快捷选项，输入框可任意编辑。

## 密钥处理

- `fromRecord` 的入参已经过 `stripCredentials`（`configs/profileCredentials.ts`），密钥字段初值恒为空。外壳不做二次清理，也不依赖「后端返回掩码」这一前提——后端实际返回明文，剥离在平台控制器完成。
- 留空提交时 adapter 从 payload 中删除密钥键，后端保持原值。
- `submitError` 只展示后端返回的错误文案，不拼接表单值。
- 密钥输入框不设 `data-*` 回显属性，不写入任何日志。

sentinel 测试（AC14）：以 `crypto.randomUUID()` 作为密钥输入，mock 后端返回错误，断言该串不出现在 `container.textContent`、`console.*` 的调用参数、`submitError` 文案、任意元素的 `data-*` 属性值四处。

## 样式

`profile-editor-shell.css` 当前服务旧结构，重写。共享的 chip、输入框、label、按钮类由 `08-26-profile-list-surface` 在 `profiles-shared.css` 落地，本文件只写模态外壳、遮罩、分区、认证分组框、折叠区、汇总条。

遮罩与阴影使用 token（`--color-scrim` 系列），不硬编码 rgba。

## 测试

| 文件                                    | 覆盖                                                                          |
| --------------------------------------- | ------------------------------------------------------------------------------- |
| `tests/profile-editor-shell.smoke.test.tsx` | AC1 AC2 AC3 AC4 AC11 AC12 AC14 AC15 AC16 AC17：外壳行为，用一份最小 stub adapter |
| `tests/profile-editor-adapters.smoke.test.ts` | AC5 AC6 AC7 AC8 AC9 AC10 AC13：三份真实 adapter 的 validate 与 submit payload |

adapter 测试 mock `@/api` 的平台函数，断言传入的 request / patch 结构，不发真实 IPC。

focused 命令：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/profile-editor-shell.smoke.test.tsx tests/profile-editor-adapters.smoke.test.ts tests/grok-profile-editor.smoke.test.ts
```

## 风险

- `buildClaudeProfileRequest` 与 `buildCodexProfileRequest` 恒写 `auth_token`。adapter 在 request 上删键的做法依赖后端把「字段缺席」当作保留。实施第一步先在 `commands/claude.rs` / `commands/codex.rs` 的更新路径确认这一语义；若后端把缺席当作清空，改为不删键而在 `notes.md` 记录字段缺口并交由 rollout 另立任务。这是本任务唯一需要读后端代码的点。
- Grok 的 dirty 集合原本由 `react-hook-form` 的 `formState.dirtyFields` 提供。`useProfileEditor` 不用 `react-hook-form`，需自行维护等价集合。若两者语义有差（如同值回写是否算 dirty），以现有 `useGrokProfilesPage` 的行为为准并补一条测试。
- Codex 的条件必填矩阵有五种 mode × 四类字段。实施时按规格表逐行写测试，不凭记忆推导。
