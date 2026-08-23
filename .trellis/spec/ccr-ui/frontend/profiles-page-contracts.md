# Profiles Page Contracts

> Claude Code / Codex / Grok Profiles 页面的共享骨架契约，以及平台 profile 表单的序列化契约。
>
> 适用范围：`ccr-ui/src/features/claude/ClaudeProfilesView.tsx`、`ccr-ui/src/features/codex/CodexProfilesView.tsx`、`ccr-ui/src/features/grok/GrokProfilesView.tsx`、
> `ccr-ui/src/components/profiles/*`、`ccr-ui/src/features/{claude,codex,grok}/` 下的 profile 卡片与编辑器模态、
> `ccr-ui/src/utils/{claudeProfiles,claudeProfileEditor,codexProfiles,codexProfileEditor,grokProfiles,grokProfileEditor}.ts`。

---

## 场景一：Codex profile 表单派生字段与 `env_key` 序列化

### 1. Scope / Trigger

触发条件（满足任一即需按本节执行）：

- 修改 `buildCodexProfileRequest` 或 `CodexProfileEditorForm` 字段集。
- 在 Codex 编辑器里新增/调整 `auth_mode` 相关控件。
- 新增依赖 `requires_openai_auth` / `openai_login_method` / `env_key` 的前端逻辑。

这是跨层请求契约变更（前端表单 → `codex_add_profile` / `codex_update_profile` → `profiles.toml`），必须写到 code-spec 深度。

### 2. Signatures

```ts
// ccr-ui/src/utils/codexProfileEditor.ts
export interface CodexProfileEditorForm {
  name: string
  description: string
  base_url: string
  auth_token: string
  provider: string
  provider_type: string
  tags_input: string
  enabled: boolean
  wire_api: string
  env_key: string
  auth_mode: CodexProfileAuthMode // also includes 'provider_bearer_token'
  model_reasoning_effort: string
  model_catalog_json: string
  preferred_auth_method: string
  forced_login_method: string
}

export const buildCodexProfileRequest: (
  form: CodexProfileEditorForm,
  resolvedModel: string,
) => CodexProfileRequest
```

后端落点：`crates/ccr-codex/src/platforms/codex.rs`
- `save_profile()` → `normalize_profile_for_storage()` → `normalize_auth_fields()`
- `resolve_profile_auth_mode()`：显式 `auth_mode` 优先；缺省时**会从 `env_key` 的存在推断 `provider_env_key`**。

### 3. Contracts

派生字段一律由 `auth_mode` 计算，**表单不得存这三个字段的独立副本**：

| 请求字段 | 取值来源 | 结果 |
| --- | --- | --- |
| `requires_openai_auth` | `usesOpenAiAuthMode(auth_mode)` | `openai_chatgpt` / `openai_api_key` → `true`；其余 → `false` |
| `openai_login_method` | `authModeToLoginMethod(auth_mode) ?? null` | `openai_chatgpt` → `'chatgpt'`；`openai_api_key` → `'api'`；其余 → `null` |
| `env_key` | 仅 `auth_mode === 'provider_env_key'` 时 `normalizeOptionalText(form.env_key)` | 其余模式恒为 `null` |
| `model_catalog_json` | `normalizeOptionalText(form.model_catalog_json)` | 与 auth mode 独立；空值为 `null` |
| `preferred_auth_method` | bearer 模式下显式值或 `'apikey'` | 离开 bearer 模式后为 `null` |
| `forced_login_method` | bearer 模式下显式值或 `'api'` | 离开 bearer 模式后为 `null` |

`env_key` 条件序列化是**表单侧唯一必须显式清理的字段**：表单在模式切走后仍保留旧值（用户切回来不丢输入），但请求里不能带。

### 4. Validation & Error Matrix

校验发生在 `CodexProfileEditorModal` 内（保存前，汇总条 + 跳转出错分段），视图侧不再重复：

| 条件 | 分段 | 文案 key |
| --- | --- | --- |
| `!form.name.trim()` | `identity` | `codex.profiles.validation.nameRequired` |
| `requiresBaseUrl && !form.base_url.trim()` | `auth` | `codex.profiles.validation.baseUrlRequired` |
| `requiresSecret && !form.auth_token.trim()` | `auth` | `codex.profiles.validation.authTokenRequired` |
| `requiresEnvKey && !form.env_key.trim()` | `auth` | `codex.profiles.validation.envKeyRequired` |
| `!resolvedModel.trim()` | `runtime` | `codex.profiles.validation.modelRequired` |

其中 `requiresBaseUrl = !usesOpenAiAuthMode(auth_mode)`、`requiresSecret` 覆盖 `openai_api_key` / `provider_env_key` / `provider_bearer_token`、
`requiresEnvKey = auth_mode === 'provider_env_key'`。校验未通过时 `save` 事件不发出。

bearer 派生字段在普通状态只显示有效值，高级入口允许显式选择；模板应用不得修改
`auth_token`、`auth_mode` 或这些认证派生字段。`model_catalog_json` 是独立运行时字段，切换
auth mode 时不自动清空。

后端侧独立校验：`provider_env_key` 模式缺少合法变量名 → `"provider_env_key 模式需要合法的 env_key 变量名"`。

### 5. Good / Base / Bad Cases

- **Good**：`auth_mode: 'provider_env_key'` + `env_key: 'MISTRAL_API_KEY'` → 请求带 `env_key: 'MISTRAL_API_KEY'`、`requires_openai_auth: false`、`openai_login_method: null`。
- **Good**：`auth_mode: 'provider_bearer_token'` + 空高级覆盖 → 请求带 `preferred_auth_method: 'apikey'`、`forced_login_method: 'api'`，且模板不提供 token。
- **Base**：`auth_mode: 'no_auth'`，表单里 `env_key` 为空 → 请求 `env_key: null`。
- **Bad**：用户先选 `provider_env_key` 填了 `env_key`，再切到 `no_auth` 保存 → 请求**不得**带上残留的 `env_key`。
- **Bad**：用户从 bearer 切到其他 auth mode 后，请求仍携带 bearer 派生字段。

### 6. Tests Required

`ccr-ui/tests/codex-profile-editor.smoke.test.ts`：

- `serializes env_key only in provider_env_key mode`：断言 `provider_env_key` 下透传，切到 `openai_api_key` / `no_auth` / `openai_chatgpt` 后 `env_key === null`。
- `derives the OpenAI auth flags from auth_mode instead of stored form state`：断言 `requires_openai_auth` / `openai_login_method` 随 `auth_mode` 变化。
- bearer 往返：断言新 auth mode 不回落、`model_catalog_json` 保留、默认值派生、显式覆盖保留、切离 bearer 后派生字段清空。
- Provider 模板：断言 DeepSeek 模板只填非密 endpoint/model，不包含 `auth_token` 或其他凭据字段。
- `blocks save behind a validation summary until the model is resolved`：断言 `resolvedModel: ''` 时不发 `save`，且 `.pe-summary` 出现对应文案。

### 7. Wrong vs Correct

#### Wrong —— 把清理职责藏在表单同步函数里

```ts
// 视图里维护一个 watch/同步函数，顺带清 env_key
const syncDerivedAuthFields = () => {
  form.openai_login_method = authModeToLoginMethod(form.auth_mode) ?? null
  form.requires_openai_auth = usesOpenAiAuthMode(form.auth_mode)
  if (!requiresEnvKey.value) form.env_key = '' // ← 隐藏职责
}

// builder 却无条件发送
env_key: normalizeOptionalText(form.env_key),
```

问题：清理只在「用户在 UI 里切过模式」这条路径上生效。任何绕过该函数的填充路径（模板填充、遗留 profile 直接编辑、把同步函数删掉的重构）都会让旧 `env_key` 进入请求体。

#### Correct —— 序列化点单源判定

```ts
env_key: form.auth_mode === 'provider_env_key' ? normalizeOptionalText(form.env_key) : null,
requires_openai_auth: usesOpenAiAuthMode(form.auth_mode),
openai_login_method: authModeToLoginMethod(form.auth_mode) ?? null,
```

> **Warning**：后端 `normalize_auth_fields()` 也会为非 `provider_env_key` 模式清掉 `env_key`，所以这条前端契约不是唯一防线。
> 但它仍然必须成立，原因有二：
> 1. 请求体是 UI「将要写入什么」的对外表述，Apply/Save 的 diff、日志与未来的 dry-run 预览都读它；带着不会落盘的字段就是在撒谎。
> 2. `resolve_profile_auth_mode()` 在**没有显式 `auth_mode`** 时会从 `env_key` 的存在反推 `provider_env_key`。当前前端总是显式发送 `auth_mode`，一旦哪天不发（或后端优先级调整），残留 `env_key` 就会把 profile 静默翻回弃用模式。

---

## 场景二：两页共享骨架的消费约定

### Convention：平台页只注入策略，不重写骨架

**What**：Claude Code 与 Codex 两个 Profiles 页面消费同一套 `components/profiles/*` 组件族，
布局骨架必须完全一致：`ProfilesHeader`（`actions-menu`）→ 可选 Off 横幅 → `ProfilesStatStrip`（四槽）→
`ProfilesQuickRail` → `ProfilesToolbar`（`compact-filters`）→ 主列表 → `ProfilesInspector` 右栏。
平台差异只允许出现在三个地方：StatStrip 的**特色槽**、字段集、i18n 前缀。
Off 横幅仅在后端 `can_off === true` 时出现，放在 Header 与 StatStrip 之间；确认框 `type=warning`。
命令面板可加 `__off`，不得把 Off 放进 Header 溢出菜单。

**Why**：两页此前各自演化出不同的信息架构与视觉语言，是 Profiles 重构要解决的核心问题。
骨架同构是可验证的验收标准，不是审美偏好。

**已落地的平台差异**：

| 维度 | Claude Code | Codex |
| --- | --- | --- |
| StatStrip 特色槽 | Auth 分布（订阅/API Key 计数） | Config mode（official / custom relay） |
| Filters 弹层内容 | 标签 + provider + 排序 | 标签 + 排序（无 provider 维度） |
| 卡片额外操作 | 无 | env-export 复制图标按钮 |
| 卡片额外字段 | 多模型回退链 | `auth_source` / `env_key` / `openai_login_method` 徽章 |

**策略注入位置**：行/检查器/diff 描述符统一由 `utils/{platform}Profiles.ts` 组装并注入组件，
不在组件内写平台分支。表单序列化留在 `utils/{platform}ProfileEditor.ts`，与展示策略分文件。

### Convention：`--cp-*` 与 `pe-*` 两套基底的边界

**What**：
- 页面内（视图 + 卡片 + 行 + 右栏）消费视图根元素注入的 `--cp-*` 作用域令牌。
- 编辑器模态消费 `components/profiles/profile-editor-shell.css` 的 `pe-*` 类，
  该文件内所有 `--cp-*` 引用都带全局 `--color-*` 回退。

**Why**：模态经 Teleport 脱离视图作用域，拿不到 `--cp-*`；回退保证两种挂载位置视觉一致。

**Don't**：不要为某个平台的编辑器再建一套平行令牌（历史上的 `--editor-*` / `--agent-*` 体系）。
那会同时带来 `!important` 覆盖、硬编码明暗 RGBA 和独立暗色覆盖块，
并让主题/口味切换在该模态内失效。

### Convention：编辑器模态的限高与唯一滚动根

**What**：Claude / Codex / Grok 编辑器把 `pe-shell` 放在 BaseModal **默认槽**内，并加上
`max-h-[calc(90vh-9rem)] overflow-hidden`。`.pe-scroll` 是该壳内唯一的 `overflow-y: auto`
根；段导航与 `pe-footer` 留在壳内、不进滚动区。`content-class` 只打 `pe-modal`（可加平台钩子），
不要把 `pe-shell` 打在 BaseModal 面板上。不要开 `BaseModal.scrollable`。

**Why**：`.pe-scroll` 的 `flex: 1; min-height: 0; overflow-y: auto` 只在祖先已限高时生效。
打开模态时 `document.body` 会被锁滚动。若面板无限高，整卡被视口裁切，页脚不可达。
`scrollable` 会让 BaseModal body 自己滚动；再套 `.pe-scroll` 就会双滚动。

**Don't**：不要用 `scrollable` 当长表单的快捷修复；不要指望把 `pe-shell` 写进 `content-class`
就能限高。

> **Warning**：Grok 第三方创建态曾把 `pe-shell` 打在面板上、内部再包 `.pe-scroll`、且未开
> `scrollable`。结果是内部滚不动、页面也滚不动，Enabled 与保存被裁切。

### Convention：QuickSwitch 持久化与稳定编号

**What**：`useProfilesQuickSwitch(platform)` 的 `pinned` 数组是 `⌘/Ctrl+1..8` 的唯一目标来源
（`getStableTargets: () => quickSwitch.stableTargets.value`）。最近使用 chip 展示但不编号。

**Storage**：按平台分离写入 `localStorage`：

- `ccr:profiles:pinned:{platform}`：用户钉选顺序，最多 8 项。
- `ccr:profiles:recent:{platform}`：Apply 成功后的最近使用顺序，不得参与数字键映射。

**Why**：编号跟随「当前显示顺序」会随筛选/排序/Apply 漂移，快捷键语义不稳定。

**不变量**：

- `stableTargets` 只是 `pinned` 的最多 8 项副本；搜索、筛选、排序和 `recordUse()` 都不得改变其指向。
- 首个成功 profile 快照就绪前，`getProfileNames()` 必须返回 `null`；只有后端已确认列表为空时才返回 `[]`，避免加载态误清持久化数据。
- 列表加载或刷新时清理已删除的 stale 名称并回写 storage；禁用 profile 仍保留钉选，但不可 Apply。
- 重命名必须同时替换 pinned/recent 中的旧名，不得将其静默丢弃。
- Windows/Linux 提示使用 `Ctrl`，macOS 使用 `⌘`；平台检测只调用 `getClientPlatform()`。

**配套调用点**（漏掉会导致持久化列表与真实 profile 脱节）：

- Apply 成功后 `quickSwitch.recordUse(name)`
- 重命名成功后 `quickSwitch.renamePinned(oldName, newName)`
- profile 列表刷新由 composable 内部 watch 自动清理 stale 名称，视图无需处理

**Tests**：`profiles-quick-switch.smoke.test.ts`、`profiles-hotkeys.smoke.test.ts` 和
`profiles-quick-rail.smoke.test.ts` 分别覆盖 storage/stale/rename/上限、数字键目标、roving tabindex。

### Convention：Profiles 弹层行为

**What**：`ProfilesHeader` 溢出菜单与 `ProfilesToolbar` Filters popover 共享以下交互契约：

- 打开后焦点进入第一个可操作项；`Tab` / `Shift+Tab` 在弹层内循环。
- `ArrowDown` / `ArrowUp` 在可操作项间循环；`Escape` 关闭并将焦点还给触发按钮。
- 点击弹层外关闭；Header 菜单执行项后关闭，Filters 选中项后保持打开，仅清空/外部点击/`Escape` 关闭。
- 桌面窄布局仍锚定触发按钮并右对齐；`<=720px` 改为距视口边缘 12px 的底部全宽面板。

**Why**：这两个弹层是页面的主要键盘操作入口；行为分歧会造成焦点丢失，也会让窄视口的控件溢出。

### Convention：`0.75rem` 密排元数据字阶

**What**：Profiles 工作台允许在短标签、键帽、字段元数据、列头和健康审计行使用 `0.75rem`。
正文、操作说明、表单标签和按钮文案仍遵循 DESIGN.md 的 `0.8125rem` Label 或更大字阶。

**Why**：Profiles 需要在专家密度下扫描多个配置字段。`0.75rem` 是有意的紧凑元数据层，
不是把所有文本缩小的通用逃生口。

**Don't**：不得新增 `10px` / `10.5px` / `11px` / `11.5px` 等任意字号，也不得把验证、备份或破坏性操作说明降到该字阶。

### Gotcha：删除确认框的备份文案必须与真实行为一致

> **Warning**：`{platform}.profiles.confirmDeleteBackupFootnote` 描述的是
> `write_guarded` + `BackupPolicy::Dir` 写入 `~/.ccr/backups/{platform}/` 的本地快照
> （`crates/ccr-config/src/platforms/base.rs`）。
>
> 当前**没有** UI 内恢复入口，文案不得承诺「从 Sync 页恢复」——
> Sync 同步的是 `~/.ccr/platforms/`，与本地快照是两回事。

---

## 场景三：Grok write-only patch 与状态信封

### 1. Scope / Trigger

- 修改 `GrokProfileEditorModal`、`grokProfileEditor.ts`、`GrokProfilesView` 或 Grok profile domain wrapper。
- 新增 Grok profile 字段、凭据动作、删除/改名分支或 Local-only 行为。
- 这是 UI → generated client → Tauri patch helper → `GrokPlatform::validate_profile` 的跨层契约。

### 2. Signatures

```ts
type GrokCredentialAction =
  | 'preserve'
  | 'replace_api_key'
  | 'replace_env_key'
  | 'clear'

buildGrokPatch(
  form: GrokProfileEditorForm,
  dirtyFields: ReadonlySet<keyof GrokProfileEditorForm>,
): GrokProfilePatch

updateGrokProfile(name: string, patch: GrokProfilePatch): Promise<GrokProfileActionResponse>
deleteGrokProfile(name: string, options?: { force?: boolean }): Promise<GrokProfileActionResponse>
```

DTO authority: `GrokProfileDto` exposes `profile_kind`, `base_url_display`,
`auth_mode`, `has_inline_credential`, and optional `env_key`; it never exposes
`api_key` or compatibility `auth_token`.

### 3. Contracts

- `profile_kind` comes only from the backend DTO. Editing keeps it read-only; the frontend never infers it from URL/provider/auth fields.
- `base_url_display` is display-only. The editable `baseUrl` and credential inputs start blank and no builder serializes `base_url_display`.
- Patch fields use presence semantics: absent preserves, `null` clears an optional field, and a value replaces it. Only dirty ordinary fields are serialized.
- Clearing an official profile's optional model sends `model: null`. The Tauri decoder accepts the clear; core validation still rejects a third-party profile without a model.
- Credential actions are mutually exclusive. Only `replace_api_key` sends `api_key`; only `replace_env_key` sends `env_key`; `preserve` and `clear` send neither value field.
- Delete branches only on `{ status, reason }`. `active|drifted` may offer one force retry; `unsafe_missing_entry_state` shows durable manual recovery and never offers force. A blocked force retry surfaces the backend message and must not reopen another force dialog.
- Rename pin migration follows the real intermediate state: `rename_apply_failed` keeps the old pin until retry apply succeeds; `rename_cleanup_failed` migrates immediately because the new name is already active.
- Before a successful local list snapshot, `getProfileNames()` returns `null`. An unsupported environment must not return `[]` or erase persisted local pins. Local-only mode closes write surfaces and guards action handlers.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| third-party create lacks base URL/model/credential action | editor validation summary; no save event |
| replace action lacks matching value | editor validation summary; no request |
| edited display URL/credential remains untouched | fields absent from patch |
| official model is cleared | `model: null`; Tauri clears it |
| third-party model is cleared | core `validate_profile` rejects save |
| delete returns `blocked(unsafe_missing_entry_state)` | manual recovery banner; no force call |
| force delete returns `blocked` again | one error; no confirmation loop |
| environment is not local | no Grok profile command; pins preserved; mutations disabled |

### 5. Good / Base / Bad Cases

- **Good**: changing only reasoning effort sends `{ reasoning_effort: 'high' }` and preserves URL/credentials.
- **Good**: `rename_apply_failed` keeps `old-name` pinned, then retry apply migrates pinned/recent to `new-name`.
- **Base**: an official profile with session auth renders no provider, base URL, or credential controls.
- **Bad**: copying `base_url_display` into `base_url`; the safe display form may omit query/userinfo and corrupt the stored URL.
- **Bad**: treating non-local as a successful empty snapshot; this deletes valid local quick-switch pins.

### 6. Tests Required

- `tests/grok-profile-editor.smoke.test.ts`: reasoning-only patch exclusion, display URL non-serialization, credential action field exclusivity, official-only controls, and explicit model clear.
- `tests/grok-profiles-view.smoke.test.ts`: Local-only fail-closed/pin preservation, delete blocked/force branches, no force loop, rename recovery pin timing, and enabled/total health summary.
- `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::grok::tests -- --test-threads=1`: Tauri patch/status/redaction/local-only contracts.
- Run the shared Profiles matrix, `bun run type-check`, `bun run lint`, `node scripts/check-i18n.mjs`, `just tauri-bindings-check`, and `just frontend-check-quick`.

### 7. Wrong vs Correct

#### Wrong

```ts
const patch = { ...profile, base_url: profile.base_url_display, api_key: form.apiKey }
```

This writes a lossy display URL and can resend a credential that the user did not choose to replace.

#### Correct

```ts
const patch = buildGrokPatch(form, dirtyFields)
// Only a selected replacement action adds exactly one credential value field.
```

---

## Quality Check

改动本文件覆盖的范围后运行：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/codex-profiles-view.smoke.test.ts tests/codex-profile-editor.smoke.test.ts tests/claude-profiles-view.smoke.test.ts tests/profiles-quick-switch.smoke.test.ts tests/profiles-quick-rail.smoke.test.ts tests/profiles-hotkeys.smoke.test.ts tests/profiles-toolbar.smoke.test.ts tests/profile-diff.smoke.test.ts
```

再跑 `cd ccr-ui && bun run type-check`、`bun run lint`、`bun run test:i18n`（改动 i18n 键时）。

Grok Profiles 改动还必须把 `tests/grok-profile-editor.smoke.test.ts` 与
`tests/grok-profiles-view.smoke.test.ts` 加入同一次矩阵。
