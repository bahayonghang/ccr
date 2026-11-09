# Design：共享编辑用例与 `profile edit` 命令

## 1. 分层

```
ccr <p> profile edit ─┐
ccr <p> profile set-field ─┤→ application::profile_edit::edit_profile(EditProfileRequest)
TUI 编辑器（worker）──┘        ├─ Claude/Codex → ProfileLifecycleContext::update(.., ApplyPolicy)
                               └─ Grok         → GrokPlatform::update_profile_versioned(.., ApplyPolicy)
```

- 新文件 `crates/ccr-cli/src/application/profile_edit.rs`（用例、请求 / 结果类型、字段规则）。
- 字段规则从 `crates/ccr-cli/src/commands/platform/profile.rs:74-260`（`editable_fields`、`ensure_field_allowed`、`update_profile_field`）移入用例模块，命令层只做参数解析和输出。
- 命令定义加在 `crates/ccr-cli/src/cli/subcommands/{claude,codex,grok}.rs` 的 `*ProfileAction::Edit`，参数结构放 `profile_args.rs`（通用）与 `grok.rs`（Grok 凭据参数），分发在 `crates/ccr-cli/src/cli/dispatch.rs`。

## 2. 类型

```rust
pub struct EditProfileRequest {
    pub platform: Platform,
    pub name: String,
    pub rename: Option<String>,
    pub expected_revision: Option<String>,
    pub fields: Vec<FieldEdit>,          // Set { field, value: serde_json::Value } | Clear { field }
    pub secret: SecretEdit,              // Keep | Replace(Secret) | Clear | UseEnvKey(String)（后者仅 Grok）
    pub apply: ApplyPolicy,              // IfCurrent（默认）| Never（--no-apply）
}

pub struct EditProfileOutcome {
    pub platform: Platform,
    pub old_name: String,
    pub new_name: String,
    pub changed_fields: Vec<String>,     // 只有键名
    pub reapplied: bool,
    pub revision: Option<String>,        // 写入后的 catalog revision
    pub lifecycle: EditLifecycle,        // Profile(ProfileOutcome) | Grok(GrokProfileUpdateOutcome 的无载荷摘要)
}
```

- `SecretEdit::Replace` 持有 `ccr_core::Secret`；两个类型都不派生会打印密钥的 `Debug`。
- 空改动（`fields` 为空、`secret == Keep`、`rename` 为空或同名）返回 `ValidationError`，不进入锁。

## 3. 重新应用（C4）

- `ProfileLifecycleContext::update` 增加 `ApplyPolicy` 参数；`active` 由 `target != name && current == name` 改为 `policy == IfCurrent && current == name`（`profile_lifecycle.rs:851`）。现有公开入口 `update_profile` / `update_profile_versioned` 固定传入旧语义对应的策略，Tauri（`ccr-ui/src-tauri/src/commands/{claude,codex}_profiles.rs`）与 GPUI（`ccr-desktop/src/runtime/profiles.rs`）的行为不变；新用例走新的带策略入口。
- Grok：`GrokPlatform::update_profile_versioned` 同样增加策略参数；同名且 `was_active && IfCurrent` 时在保存后调用 `apply_profile_locked`，失败映射为新的 `GrokProfileUpdateOutcome::UpdatedApplyFailed`，提示与现有 `RenameApplyFailed` 同级。
- 重新应用会经过 `execute` 的激活分支，因此同时增加切换计数并写一条启用历史（`profile_lifecycle.rs:746-751`）。这与 spec "上次使用 = CCR 成功启用" 的定义一致，不做特殊豁免。
- 当前生效 profile 被设为禁用：在锁内、写入前拒绝（扩展 `profile_lifecycle.rs:852-856` 的规则到同名情形）。

## 4. 密钥输入

- `--auth-token-stdin`（Grok：`--api-key-stdin`）从标准输入读取到 EOF，去掉末尾换行；读到空串报错。标准输入是终端时打印一行提示并关闭回显读取一行。
- 不新增 `--auth-token <value>` 形式的编辑参数，避免进入 shell 历史；`create` 现有参数不动。
- Grok 凭据互斥沿用 Grok 规则：替换 API 密钥时移除 `env_key` 与 `auth_token`；改用环境变量时移除 `api_key` 与 `auth_token`；清除时三者都移除（与 `crates/ccr-app/src/config/profile_patch/grok.rs:67-113` 语义一致）。

## 5. 输出

- 人类可读：`已更新 <platform> Profile '<new>'：description, model, base_url（已重新应用）`；改名时显示 `old → new`。遵循 `cli-output-presentation.md`。
- `--json`：`{ ok, platform, old_name, new_name, changed_fields, reapplied, revision, status, warnings }`。
- `set-field` 保持原有 `PlatformProfileMutationOutput` 格式，内部改为构造单字段 `EditProfileRequest`（`apply: IfCurrent`）。

## 6. 兼容与回滚

- 现有调用方语义不变（见第 3 节）。`set-field` 获得版本校验与重新应用，这是有意的行为变化，写入 CHANGELOG。
- 回滚：revert 用例与命令；生命周期的策略参数是新增参数，旧入口默认值保持旧语义。
