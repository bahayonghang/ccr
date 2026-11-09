# Design — Claude 第三方模型 Profile auth_mode 自动纠正

## 1. 现状链路验证（为什么 CLI 映射本身没问题）

| 环节                     | 位置                                                                                                                            | 结论                                                                            |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| 数据结构含扩展字段       | `ProfileConfig` crates/ccr-config/src/models/platform.rs:167；`ConfigSection` crates/ccr-config/src/managers/config/types.rs:65 | ✅ `default_opus_model` 等齐全                                                  |
| Profile↔Section 双向转换 | `base::profile_to_section` / `section_to_profile` crates/ccr-config/src/platforms/base.rs:129,155                               | ✅ 扩展字段全部 clone                                                           |
| env 写出                 | `ClaudeSettings::update_from_config` crates/ccr-cli/src/managers/settings.rs:104                                                | ✅ `default_opus_model → ANTHROPIC_DEFAULT_OPUS_MODEL`，有测试 settings.rs:1006 |
| Tauri 持久化             | `patch_profile_with_config` / `profile_to_json` ccr-ui/src-tauri/src/commands/claude.rs:328,409                                 | ✅ 读写扩展字段                                                                 |
| 前端绑定                 | ClaudeProfileEditorSections.vue:178                                                                                             | ✅ Opus 输入框绑定 `default_opus_model`                                         |

→ 失效仅发生在 **apply 时的 auth_mode 分支** 与 **存量数据落在孤儿字段**。

## 2. 失效点定位

### 2.1 apply 分支（主因）

crates/ccr-cli/src/platforms/claude.rs:278

```rust
match auth_mode {
    ClaudeProfileAuthMode::Subscription => settings.clear_managed_vars(), // 只清空，不写任何 env
    ClaudeProfileAuthMode::ApiKey       => settings.update_from_config(&section),
}
```

`auth_mode` 来自 `Self::profile_auth_mode` → `ClaudeAuthService::resolve_profile_auth_mode`（claude_auth_service.rs:737）：**显式 `auth_mode` 优先**，`infer_profile_auth_mode`（:753，会从 base_url/token 推断 api_key）被旁路。表单默认值 subscription（ClaudeCodeProfilesView.vue:584/795/835）→ 第三方 profile 被当订阅 → 清空。

### 2.2 孤儿字段（次因）

`[chy]` 的 Opus 值落在 platform_data 的 `custom_model_option`，既不是 typed `default_opus_model`，`update_from_config` 也不处理 platform_data，故永不写 env。

## 3. 设计方案

### 3.1 核心契约：`is_api_key_shaped` + `effective_auth_mode`

在 ccr-cli 内新增**纯函数**（建议置于 `ClaudeAuthService` 或 `ClaudePlatform`，与 `resolve_profile_auth_mode` 同层，便于复用）：

```rust
/// 判定 profile 是否为「API-key 形态」（第三方/中转必然形态）
/// 保守规则: provider_type=third_party_model 或 (base_url 与 auth_token 同时非空)
fn is_api_key_shaped(profile: &ProfileConfig) -> bool {
    fn filled(v: &Option<String>) -> bool {
        v.as_deref().is_some_and(|s| !s.trim().is_empty())
    }
    profile.provider_type.as_deref() == Some("third_party_model")
        || (filled(&profile.base_url) && filled(&profile.auth_token))
}

/// 在 resolve 之上叠加自愈：API-key 形态 + 解析为 subscription → 纠正为 api_key
/// 纯函数、不打日志（warn 在 apply/normalize 纠正点发出，避免只读渲染路径刷屏）
pub fn effective_auth_mode(profile: &ProfileConfig) -> ClaudeProfileAuthMode {
    let resolved = Self::resolve_profile_auth_mode(profile);
    if matches!(resolved, ClaudeProfileAuthMode::Subscription) && Self::is_api_key_shaped(profile) {
        return ClaudeProfileAuthMode::ApiKey;
    }
    resolved
}
```

> **关键边界 1（避免假阳性）**：**不把「模型映射字段非空」纳入判定**。`ANTHROPIC_DEFAULT_*_MODEL` 在官方订阅下也可用于钉某个快照；若以「填了模型映射」即判 api_key，会误伤合法的「订阅 + 快照钉选」profile 并触发 `section.validate()`（要求 base_url/token）失败。chy 这类真实第三方必然带 base_url+token，已被覆盖，无需模型映射信号。
> **关键边界 2**：不修改 `resolve_profile_auth_mode` 本体——它表达「存储态的字面解析」，既有单测 `test_resolve_profile_auth_mode_prefers_explicit_platform_data`（claude_auth_service.rs:974）依赖该语义。纠正逻辑作为独立叠加层，零破坏。
> **日志安全**：`effective_auth_mode` 不打日志；纠正发生时由 `apply_profile` / `normalize_profile` 各自 `tracing::warn`，仅打印 profile 名/provider，绝不打印 `auth_token`/`base_url` 全量。

### 3.2 两处接入点

| 接入点                 | 位置                                                                          | 改动                                                                                          | 作用                                                         |
| ---------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ------------------------------------------------------------ |
| **应用时（防御自愈）** | `ClaudePlatform::apply_profile` claude.rs:269                                 | 将 `let auth_mode = Self::profile_auth_mode(profile)` 改为走 `effective_auth_mode`            | 既有 `chy` 等存量 profile 无需重存即可正确 apply（AC1/AC3）  |
| **保存时（权威纠正）** | `ClaudePlatform::normalize_profile` claude.rs:175（已在 `save_profile` 调用） | 计算 `effective_auth_mode`，若与存储显式值不同，则写回 `platform_data["auth_mode"]="api_key"` | 落盘存储态被纠正，UI 列表/表单刷新即显示 api_key（AC2/R2.3） |

`profile_auth_mode`（claude.rs:158）是 `apply_profile` 与 `profile_to_json` 共用的入口；为最小化扩散，让 `profile_auth_mode` 直接委托 `ClaudeAuthService::effective_auth_mode`，则 apply 与 UI 回显**自动一致**。需复核所有 `profile_auth_mode` 调用方（auth_source 等）语义无回归。

### 3.3 custom_model_option 正规化（R3）

**typed 化**（对齐 default\_\*\_model 既有写法）：

- `ProfileConfig`（platform.rs）+ `ConfigSection`（types.rs）新增：
  `custom_model_option: Option<String>`、`custom_model_option_name: Option<String>`。
- `base::profile_to_section` / `section_to_profile`（base.rs）补 clone。
- `update_from_config`（settings.rs）新增映射：
  `custom_model_option → ANTHROPIC_CUSTOM_MODEL_OPTION`、`custom_model_option_name → ANTHROPIC_CUSTOM_MODEL_OPTION_NAME`；
  `clear_managed_vars` 的清理集合同步纳入这两个 key（它们带 `ANTHROPIC_` 前缀，已被 `clear_anthropic_vars` 覆盖，确认无遗漏即可）。
- `get_env_var_names`（claude.rs:312）追加两项。
- Tauri `patch_profile_with_config` / `profile_to_json`（claude.rs）补字段读写。
- 前端 `ClaudeProfileEditorForm`（types/claudeProfileEditor.ts）+ 表单（ClaudeProfileEditorSections.vue 高级映射区）补输入框与 helper。

**迁移自愈**：在 profile 加载或保存归一处（建议 `normalize_profile`），若 typed 字段为空而 `platform_data` 含同名 key，则抬升为 typed 并从 platform_data 移除。保证 `chy` 这类存量被一次性清理。

- 语义红线（R3.4）：仅做字段归位，**不**把 `custom_model_option` 写进 `default_opus_model`。

### 3.4 前端默认与内联校验（R2/R4）

- 默认值：`createEmptyForm`/模板应用处，当存在第三方信号时，`auth_mode` 初值取 `api_key`。最稳妥实现：保留 select，但在「检测到 base_url/provider/模型映射且仍为 subscription」时显示 `editor-banner--warn` 内联提示（复用 ClaudeProfileEditorSections.vue 既有 banner 样式），文案见 R4.1。
- 由于后端会权威纠正，前端提示定位为「提前告知 + 体验」，即使用户忽略也不会出错（后端兜底）。
- provider 模板（R4.2）：遵循 ccr-ui provider-template-contracts，新增/标注「第三方模型」模板预置 `auth_mode=api_key`、`provider_type=third_party_model`、空的 `default_opus_model` 占位。

## 4. 数据流（修复后）

```
表单(api_key 默认 + 内联校验)
  └─ save_claude_profile ─> patch_profile_with_config ─> normalize_profile
                                                          ├─ effective_auth_mode → 落盘 api_key
                                                          └─ 迁移 custom_model_option → typed
  └─ apply_claude_profile ─> apply_profile
                              └─ effective_auth_mode == ApiKey
                                  └─ update_from_config ─> settings.json:
                                       ANTHROPIC_BASE_URL / AUTH_TOKEN /
                                       DEFAULT_OPUS_MODEL / CUSTOM_MODEL_OPTION /
                                       CLAUDE_CODE_EFFORT_LEVEL ...
  └─ claude 启动读取 settings.json → 命中第三方模型 ✅
```

## 5. 兼容性 / 回滚

- **向后兼容**：新增 typed 字段均 `Option` + `skip_serializing_if=Option::is_none`，旧 profiles.toml 解析不受影响；platform_data flatten 仍兜底未知键。
- **存量自愈**：apply 防御层使既有 subscription-误标 profile 立即可用；save/load 迁移逐步清理孤儿键。无需用户手动迁移。
- **回滚**：纯逻辑叠加 + 新增字段，回退只需 revert；不涉及数据销毁（迁移是「抬升+删冗余键」，原值仍在 typed 字段中）。
- **安全**：沿用 `SettingsManager::save_atomic`（tempfile+rename+lock）与 `base.rs` 备份轮转；不新增明文落点；日志脱敏。

## 6. 取舍记录

- **自动纠正 vs 阻断/警告**（已选自动纠正）：第三方 profile 配置意图明确，自动纠正最省心；以「后端权威纠正 + 前端内联提示」兜住「静默」风险，避免纯静默修改带来的不可解释性。
- **纠正点放 apply 还是 save**：两者都做。仅 save 无法自愈存量；仅 apply 则 UI 仍显示错误 auth_mode。共用 `effective_auth_mode` 一处实现，避免逻辑分叉。
- **custom_model_option 是否纳入 is_api_key_shaped**：纳入——它是 `ANTHROPIC_*` 覆盖之一，存在即说明用户在配第三方 picker。
