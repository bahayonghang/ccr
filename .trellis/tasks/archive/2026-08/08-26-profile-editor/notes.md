# 08-26-profile-editor notes

## 后端 `auth_token` 缺席语义

`ccr-ui/src-tauri/src/commands/claude.rs` `patch_profile_with_config` 与 `codex.rs` `apply_profile_config` 均为：

```
if let Some(raw) = obj.get("auth_token") { profile.auth_token = ... }
```

键缺席时不进入分支，保留原值。空字符串由 `parse_string_field` 处理，与缺席不同。adapter 在密钥留空时 **delete 键** 是安全的。

## Codex 条件必填矩阵

| auth_mode | requiresBaseUrl (`!usesOpenAiAuthMode`) | requiresSecret (`openai_api_key` / `provider_env_key` / `provider_bearer_token`) | requiresEnvKey | model |
| --- | --- | --- | --- | --- |
| `openai_chatgpt` | 否 | 否 | 否 | 是 |
| `openai_api_key` | 否 | 是 | 否 | 是 |
| `provider_env_key` | 是 | 是 | 是 | 是 |
| `provider_bearer_token` | 是 | 是 | 否 | 是 |
| `no_auth` | 是 | 否 | 否 | 是 |

编辑态密钥留空不视为 `requiresSecret` 失败（AC13 留空不序列化）。新建仍拦截。

序列化：`env_key` 仅 `provider_env_key`；bearer 派生字段仅 `provider_bearer_token`。由 `buildCodexProfileRequest` 保证。

## Grok 已解决交互

- `hasExistingBaseUrl`：编辑且后端已有 URL 时 `baseUrl` 留空放行（`validateGrokEditor`）。
- official：隐藏 connection / provider / 凭据；`credential_action` 强制 `preserve`。
- credential action 互斥：`addCredentialFields` 只在对应 action 写 `api_key` / `env_key`。
- `fillGrokForm` 不复制 `base_url_display`，`baseUrl` 恒空。
- dirty：`useGrokProfilesPage` 用 `Object.keys(formState.dirtyFields)`；同值回写仍算 dirty（RHF 默认）。本任务 `setField` 一律记 dirty。
- `profile_kind` 在统一外壳中为只读展示；新建默认 `third_party`。
- `tests/grok-profile-editor.smoke.test.ts` 本任务重建为对 `validateGrokEditor` / `buildGrokPatch` / `fillGrokForm` 的行为锁，因仓库中原先不存在该文件。

## 共享原子类（list-surface）

`cp-btn` `cp-btn--ghost` `cp-btn--primary` `cp-btn--accent-soft` `cp-chip` `cp-pill` `cp-pill--active` `cp-label` `cp-input`

Dialog 原语：`BaseModal`（`scrollable` 保持 false）。
