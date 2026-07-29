# Profiles Page Contracts

> Claude Code / Codex 两个 Profiles 页面的共享骨架契约，以及 Codex profile 表单的序列化契约。
>
> 适用范围：`ccr-ui/src/views/{ClaudeCodeProfilesView,CodexProfilesView}.vue`、
> `ccr-ui/src/components/profiles/*`、`ccr-ui/src/components/{claude,codex}/` 下的 profile 卡片与编辑器模态、
> `ccr-ui/src/utils/{claudeProfiles,claudeProfileEditor,codexProfiles,codexProfileEditor}.ts`。

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
  auth_mode: CodexProfileAuthMode // 'openai_chatgpt' | 'openai_api_key' | 'provider_env_key' | 'no_auth'
  model_reasoning_effort: string
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

其中 `requiresBaseUrl = !usesOpenAiAuthMode(auth_mode)`、`requiresSecret = auth_mode === 'openai_api_key'`、
`requiresEnvKey = auth_mode === 'provider_env_key'`。校验未通过时 `save` 事件不发出。

后端侧独立校验：`provider_env_key` 模式缺少合法变量名 → `"provider_env_key 模式需要合法的 env_key 变量名"`。

### 5. Good / Base / Bad Cases

- **Good**：`auth_mode: 'provider_env_key'` + `env_key: 'MISTRAL_API_KEY'` → 请求带 `env_key: 'MISTRAL_API_KEY'`、`requires_openai_auth: false`、`openai_login_method: null`。
- **Base**：`auth_mode: 'no_auth'`，表单里 `env_key` 为空 → 请求 `env_key: null`。
- **Bad**：用户先选 `provider_env_key` 填了 `env_key`，再切到 `no_auth` 保存 → 请求**不得**带上残留的 `env_key`。

### 6. Tests Required

`ccr-ui/tests/codex-profile-editor.smoke.test.ts`：

- `serializes env_key only in provider_env_key mode`：断言 `provider_env_key` 下透传，切到 `openai_api_key` / `no_auth` / `openai_chatgpt` 后 `env_key === null`。
- `derives the OpenAI auth flags from auth_mode instead of stored form state`：断言 `requires_openai_auth` / `openai_login_method` 随 `auth_mode` 变化。
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
布局骨架必须完全一致：`ProfilesHeader`（`actions-menu`）→ `ProfilesStatStrip`（四槽）→
`ProfilesQuickRail` → `ProfilesToolbar`（`compact-filters`）→ 主列表 → `ProfilesInspector` 右栏。
平台差异只允许出现在三个地方：StatStrip 的**特色槽**、字段集、i18n 前缀。

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

### Convention：数字快捷键只绑定钉选顺序

**What**：`useProfilesQuickSwitch(platform)` 的 `pinned` 数组是 `⌘/Ctrl+1..8` 的唯一目标来源
（`getStableTargets: () => quickSwitch.stableTargets.value`）。最近使用 chip 展示但不编号。

**Why**：编号跟随「当前显示顺序」会随筛选/排序/Apply 漂移，快捷键语义不稳定。

**配套调用点**（漏掉会导致钉选列表与真实 profile 脱节）：
- Apply 成功后 `quickSwitch.recordUse(name)`
- 重命名成功后 `quickSwitch.renamePinned(oldName, newName)`
- profile 列表刷新由 composable 内部 watch 自动清理 stale 名称，视图无需处理

### Gotcha：删除确认框的备份文案必须与真实行为一致

> **Warning**：`{platform}.profiles.confirmDeleteBackupFootnote` 描述的是
> `write_guarded` + `BackupPolicy::Dir` 写入 `~/.ccr/backups/{platform}/` 的本地快照
> （`crates/ccr-config/src/platforms/base.rs`）。
>
> 当前**没有** UI 内恢复入口，文案不得承诺「从 Sync 页恢复」——
> Sync 同步的是 `~/.ccr/platforms/`，与本地快照是两回事。

---

## Quality Check

改动本文件覆盖的范围后运行：

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/codex-profiles-view.smoke.test.ts tests/codex-profile-editor.smoke.test.ts tests/claude-profiles-view.smoke.test.ts tests/profiles-quick-switch.smoke.test.ts tests/profiles-quick-rail.smoke.test.ts tests/profile-diff.smoke.test.ts
```

再跑 `cd ccr-ui && bun run type-check`、`bun run lint`、`bun run test:i18n`（改动 i18n 键时）。
