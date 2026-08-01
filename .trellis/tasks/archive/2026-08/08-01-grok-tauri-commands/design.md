# 技术设计:Grok Tauri 命令桥接层

前置阅读:父任务 `design.md`(D1-D9,契约权威)、`research/tauri-command-layer.md`、`research/grok-backend-capabilities.md`、spec `grok-profile-runtime.md`、`tauri-handler-registry.md`。
修订记录:2026-08-01 依据 Codex 审阅重写(§2 DTO、§3 状态机、§4 settings patch、§5 raw、新增 §0 核心层改动)。

## 0. 核心层改动(仅 Grok 模块,先行实施)

`crates/ccr-cli/src/platforms/grok.rs` 新增只读 API(父 design D8.1):

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum GrokActivationState {
    Inactive,
    Active { name: String },
    Drifted { name: String },
    UnsafeMissingEntryState { name: Option<String> },
}
impl GrokPlatform {
    pub fn inspect_activation_state(&self) -> Result<GrokActivationState> { ... }
}
```

实现要点:复用现有私有判定件——原始注册表指针(`current_profile_from_registry`,不走 `get_current_profile` 的 drift 清理)、profiles `current_config` fallback、`runtime_matches_profile` 等价比较、`load_entry_state` 存在性、`runtime_has_managed_shape`。判定顺序:入口状态缺失且(意图存在或受管形态)→ `UnsafeMissingEntryState`;意图存在且运行时匹配 → `Active`;意图存在但不匹配 → `Drifted`;否则 `Inactive`。**零写入**(与 delete_profile 的 fail-closed 判定同源,杜绝语义分叉)。
配套:四态单测 + `grok-profile-runtime.md` Signatures/Contracts 增补(标注 read-only、no side effects、errors 不含凭据)。导出经 `platforms/mod.rs` → `ccr` facade。

实现审查发现共享 `save_profiles_to_toml` 会在文件新建或 `current_config=""` 时自动选择第一个 profile,与本任务 CRUD 不改变 activation 的前提冲突。`GrokPlatform::save_profile` 因此在 `grok_profile_operation` 锁内保存,且保存前注册表与 profiles 指针均为空时在同一锁作用域内恢复空 `current_config`;不修改其他平台的共享 serializer。

## 1. Tauri 层文件布局

- 新建 `ccr-ui/src-tauri/src/commands/grok.rs`(单文件起步,含 DTO + 命令 + 单测);`mod.rs` 声明;`handler_registry.rs` 注册。
- 每个命令体第一步:Local-only 检查(复用现有 `ensure_local_env` 式 helper / `EnvironmentRegistry` active env_type 判定);非 local 返回 `{status:"unsupported_environment", env_type}` 信封(读类)或中文错误(写类,保持与 raw 契约一致的形态,实现时统一)。

## 2. DTO(derive Serialize + TS,export_to `../../src/types/generated/grok/`)

```rust
#[serde(rename_all = "snake_case")] enum GrokAuthModeDto { InlineApiKey, EnvKey, Session }
#[serde(rename_all = "snake_case")] enum GrokActivationDto { Inactive, Active, Drifted, UnsafeMissingEntryState }  // + name 字段平铺在响应里

struct GrokProfileDto {
    name, description, provider,
    profile_kind: String,              // "official" | "third_party",后端权威判定
    base_url_display: Option<String>,  // 仅展示,永不回写
    has_base_url: bool,
    model, api_backend, context_window, supports_backend_search, reasoning_effort,
    auth_mode: GrokAuthModeDto, env_key: Option<String>, has_inline_credential: bool,
    enabled: bool, tags: Vec<String>,
}
struct GrokProfilesResponse { profiles: Vec<GrokProfileDto>, current_profile: Option<String>,
    activation: GrokActivationDto, activation_name: Option<String>, default_profile: Option<String> }
struct GrokDashboardOverview { activation: GrokActivationDto, current_profile: Option<String>,
    auth_mode: Option<GrokAuthModeDto>, profiles_total: u32, profiles_enabled: u32,
    config_exists: bool, config_path_display: String }
```

映射:`profile_to_dto` 用 `profile_auth_mode` + `safe_base_url_for_display`;`profile_kind` 判定 = 存在 base_url 或非 session 凭据 → third_party(与核心 validate 语义一致,实现时对照 `validate_profile` 的官方/第三方分支)。**禁止 `serde_json::to_value(&ProfileConfig)` 直出。**

### Update 请求(patch)

`grok_update_profile(name, patch: OpenJsonValueDto)`,Tauri 层解析:

- 普通字段(description/base_url/model/api_backend/context_window/supports_backend_search/reasoning_effort/tags/enabled/name):key 缺席=保留;`null`=删除该 platform_data 键(name/model 等必填项 null → 校验错误);有值=替换。
- `credential_action`: `"preserve"`(缺省)| `"replace_api_key"`(需 `api_key` 值)| `"replace_env_key"`(需 `env_key` 值)| `"clear"`。替换时先移除全部旧凭据字段再写入新字段。
- 组装后走 `validate_profile` → `save_profile`。

## 3. 操作命令状态机

### delete(D8.2)

```rust
// 伪代码
let state = platform.inspect_activation_state()?;
let blocked = matches!(&state, Active{name: n} | Drifted{name: n} if n == &name)
    || matches!(&state, UnsafeMissingEntryState{..});
if blocked && !force { return Ok(json!({"status":"blocked", "reason": reason(&state)})); }
if blocked && force {
    if matches!(state, UnsafeMissingEntryState{..}) { return Ok(blocked_envelope); } // 永不自动处理
    platform.clear_active_profile_runtime()?;
}
platform.delete_profile(&name)?;          // 核心守卫兜底(inspection 与执行之间的竞态)
Ok(json!({"status":"deleted"}))
```

核心 `delete_profile` 仍是最终权威;若它在 inspection 说可删后仍拒绝(竞态),把核心错误原样上抛(中文,已脱敏)。

### rename(D8.3)

```rust
platform.save_profile(&new_name, &patched)?;
let was_active = matches!(inspect, Active{name} if name == old);
if was_active {
    if let Err(e) = platform.apply_profile(&new_name) {
        return Ok(json!({"status":"rename_apply_failed", "message": ...}));  // 新旧并存,旧仍激活
    }
}
if let Err(e) = platform.delete_profile(&old_name) {
    return Ok(json!({"status":"rename_cleanup_failed", "message": ...}));    // 新已激活,旧残留
}
Ok(json!({"status":"renamed"}))
```

非激活改名退化为存新→删旧。所有部分失败状态的 message 含下一步指引(重试切换/重试删除旧名)。

## 4. Settings typed patch(D4)

- `grok_get_settings`:读 config.toml(不存在 → 默认空 + `exists:false`)→ 提取五 section 白名单字段现值 + `[model.*]` 脱敏摘要(`custom_models`)+ `activation` + `managed_keys_locked`(= activation ∈ {active,drifted,unsafe})。
- `grok_update_settings(patch: { set: Map<String, Value>, unset: Vec<String> })`:
  1. 白名单校验(dotted key ∈ `models.default`、`models.default_reasoning_effort`、`ui.theme`、`session.auto_compact_threshold_percent`、`session.load_envrc`、`cli.auto_update`、`cli.channel`、`cli.show_tips`、`hints.new_session_worktree_mode`、`hints.fork_worktree_mode`)+ 值域校验(0-100、枚举、bool)。
  2. 重试循环(≤3):读原文+token → `inspect_activation_state` 重查托管锁(锁定且 patch 触及 `models.*` 两键 → 拒绝返回引导错误)→ 解析 `toml::Value` 全文档 → 应用 set/unset(建缺失中间表;unset 后空表移除)→ `write_guarded_versioned(secret:true, BackupPolicy::None)` → Conflict 则 continue。
  3. 3 次冲突 → 返回 `{status:"conflict"}` 信封,前端提示重载。
- 全文档树变异保证未知表/键零丢失;不引入 `GrokConfig` typed 整量结构(评审废弃 section 覆盖方案)。

## 5. Config raw 通道(D5)

`settings_raw.rs` 增 grok 分支:`grok_get_config_raw_text` / `grok_save_config_raw_text` / `grok_list_config_layers`。保存:TOML 语法校验(错误只给行/列)→ `write_guarded_versioned(..., BackupPolicy::None)`。现有 helper 若写死 `BackupPolicy::Dir`,加按 kind 策略参数(默认原值,codex/claude 零变化)。layers 枚举:user(`$GROK_HOME/config.toml`,editable)、project(`./.grok/config.toml`,readonly)、managed(`~/.grok/managed_config.toml`、`/etc/grok/managed_config.toml`,readonly)、requirements(`~/.grok/requirements.toml`、`/etc/grok/requirements.toml`,readonly),各带 exists。

## 6. system.rs / platform/local.rs

- `CLI_VERSION_TOOLS` += `"grok"`;tool→program `grok → grok`;`normalize_cli_tool` 别名 `grok|grok-build|grok-cli`。(版本探测本身仍走 active env——前端只在 local 环境调用,见父 D9。)
- `local.rs`:`list_platforms`/`detect_cli_status`/`config_base_dir`(`GROK_HOME` 优先)补 grok。

## 7. 测试矩阵

1. 核心层:inspection 四态(含 drift 后 registry 清理前/后一致性);只读性(调用前后文件零变化)。
2. DTO 脱敏:含 api_key/auth_token/userinfo-URL 的 profile → JSON 无明文;`profile_kind` 判定与 validate 一致。
3. patch:三态字段语义、credential_action 四态、必填项 null 拒绝。
4. rename:非激活/激活/apply 失败/delete 失败四种结局(`from_parts` 注入)。
5. delete 信封:inactive 直删、active 阻断、force-active 成功、force-unsafe 拒绝、竞态兜底。
6. settings:未知表 round-trip、白名单外 key 拒绝、托管锁拒绝、**并发 CAS**(保存循环中模拟外部写,验证重试收敛且不丢外部键)。
7. raw:保存后 backups 目录无新增;token 冲突;layers 枚举。
8. Local-only:注入非 local env → 全命令拒绝/信封。

## 8. 回滚

核心层 API 单独 commit;Tauri 层 + 生成物同 commit。逆序 revert 干净退出。
