# Design：GrokPlatform 切换引擎（rev2）

> rev2 变更：①官方路线改为"恢复原条目"而非删除（CORR-001）②auth.json 事实修正 + 官方默认模型语义（CORR-002）③写序/CAS/删除语义（CORR-003）④枚举影响图外链（ARCH-002）⑤明文披露矩阵对齐（SEC-001）。

## 1. 边界与放置

- **crates/ccr-config**：`Platform` 枚举扩展（`models/platform.rs`）。
- **crates/ccr-cli**：`src/platforms/grok.rs` 新模块 + `mod.rs` 工厂注册（对齐 claude/droid/gemini，不新建 crate）。
- **全 workspace 穷举 match**：按 `research/platform-enum-impact-map.md` 逐点落显式决策（doctor skip / MCP install 拒绝 / sessions 不支持 / 展示名补充），以 `cargo check --workspace` 为完备性清单。
- 不改：ccr-codex、ccr-tui 功能面、`auth_profile_supported()`（兄弟任务负责）。

## 2. 数据模型与映射

### ProfileConfig → 运行时映射

| ProfileConfig | grok config.toml | 说明 |
|---|---|---|
| `base_url` | `[model.custom].base_url` | 非空 ⇒ 第三方；空 ⇒ 官方（纯模型选择器） |
| `model` | 第三方：`[model.custom].model`（必填）；官方：`[models].default = <model>`，未设则**移除该键**回落上游默认 | 上游默认随版本漂移，禁止硬编码 grok-build/grok-4.5 |
| `description` | `[model.custom].name` | fallback profile 名 |
| `auth_token: Secret` | `[model.custom].api_key` | inline 模式 |
| `platform_data.env_key` | `[model.custom].env_key` | 单字符串；与 api_key 互斥 |
| `platform_data.api_backend` | `[model.custom].api_backend` | `chat_completions|responses|messages`，缺省 responses |
| `platform_data.context_window` | `[model.custom].context_window` | 可选正整数 |
| `platform_data.supports_backend_search` | `[model.custom].supports_backend_search` | 可选 bool |
| `provider_type` | 路线判定辅助（`official`/`third_party`），缺省按 base_url 推断 | 对齐 Codex canonical 思路 |

### 常量与认证模式

```rust
const GROK_MANAGED_MODEL_KEY: &str = "custom";   // [model.custom]（接管用户现网别名；[ui].fork_secondary_model 引用不悬空）
const PROFILE_ENTRY_CONFIG_STATE_FILE: &str = "profile_entry_config_state.json";

enum GrokProfileAuthMode {
    InlineApiKey, // auth_token → api_key
    EnvKey,       // env_key（CCR 侧零明文；推荐口径）
    Session,      // 官方：不经营凭据，auth.json / XAI_API_KEY 自然接管
}
```

推断：`auth_token` 有值 → InlineApiKey；`env_key` 有值 → EnvKey；否则 Session。同设 → ValidationError（Grok 本体 api_key 优先，但 CCR 留双凭据会造成"改 env 不生效"排障陷阱）。官方 profile 携带任一凭据字段 → ValidationError（否则按上游三层优先级会劫持官方请求）。

### auth.json 边界（CORR-002 修正）

`~/.grok/auth.json` **存在**（会话 token、0600、后台自动刷新、hot reload）。CCR **不读不写不备份不校验**：自动刷新意味着任何外部快照必然过期，回写还会制造 token 回滚（Codex refresh_token_reused 同类教训）。官方路线因此定义为纯模型选择器。

## 3. 入口状态与恢复（CORR-001）

```rust
struct ProfileEntryConfigState {
    exists: bool,                    // config.toml 当时是否存在
    content: Option<String>,         // 整份原文（兜底恢复用）
    original_custom_model: Option<toml::Value>, // 原始 [model.custom]（None = 原不存在）
    original_default_model: Option<String>,     // 原始 [models].default（None = 原无此键）
}
```

- 首次切换（文件不存在时）捕获，AtomicWriter secret 模式写入 `~/.ccr/platforms/grok/`，之后不覆盖。
- **官方切换 / off / clear_active_profile_runtime**：
  - `original_custom_model = Some(v)` → 恢复 `[model.custom] = v`；`None` → 删除该条目（表空则连 `[model]` 表清理）。
  - `models.default`：官方 profile 显式 model → 写该值；否则恢复 `original_default_model`（`None` → 移除键）。
  - `off`/clear 额外：清 registry 指针 + profiles.toml current_config，并删除入口状态文件（下轮切换重新捕获）。
- 往返不变量：第三方 → 官方 → 第三方后，用户原条目内容可再次恢复（入口状态在 off 前不消费销毁）。

## 4. 切换流程与并发（CORR-003）

```
apply_profile(name)
 ├─ load_profiles + validate_profile
 ├─ capture_entry_config_state()                 // 仅首次
 ├─ loop (≤2 次):
 │    (config, token) = 读 config.toml + content_version_token
 │    new_config = 按路线改写（只触碰 model.custom / models.default 两落点）
 │    write_guarded_versioned(config_path, new_config, token)
 │      ├─ Written  → break
 │      └─ Conflict → 第 1 次: 重读重建; 第 2 次: 报错"config.toml 被并发修改，请重试"
 ├─ base::update_current_config(profiles.toml, name)
 └─ base::update_registry_current_profile_with_paths(registry, locks, "grok", name)
```

- **写序即真相序**：config.toml 是运行时真相源；其后两步指针更新失败时不回滚 config（报错提示重试），`get_current_profile` 漂移检测保证不谎报——此部分失败自愈路径必须有测试。
- CAS 用 `ccr_core::core::guarded_write::{write_guarded_versioned, content_version_token}` 现成原语，防 Grok 自身（hot reload 场景下用户同时改配置）或其他 CCR 进程并发覆盖。
- **删除语义**：`delete_profile` 检测目标是否当前激活（registry 指针 + 运行时匹配）；激活中 → `CcrError::ValidationError`（提示先 `ccr grok profile off` 或 switch）。强制路径由 CLI 层组合"先 clear_active_profile_runtime 再删除"实现，引擎不提供跳过检查的后门。非激活删除 → base reconcile（其"指针指向首个剩余但不 apply"的语义对非激活场景无害——指针仅在无激活时是建议值，漂移检测兜底）。

## 5. 当前 profile 判定

对齐 Codex `stable_current_profile`：registry 指针（回退 profiles.toml current_config）→ 按 profile 重建期望态（default 别名/托管条目字段/凭据形态；官方路线含"default 键应缺省"情形）→ 与运行时逐字段比对 → 不匹配清指针返回 None。

## 6. 安全与披露（SEC-001）

- `auth_token` 全程 `Secret`；日志无凭据；`safe_base_url_for_display`（剥 userinfo/query/fragment，逻辑取自 Codex 同名函数）与 `profile_auth_mode()` 作为 **pub helper** 暴露（TUI/CLI 复用，CORR-005 契约）。
- 明文矩阵（与父 PRD 一致）：profiles.toml 及其轮换备份、入口状态文件、运行时 config.toml。`secret=true` 在 Windows 为 no-op（ccr-core 现状，测试注释已明示）——文档如实披露，不做本任务修复；env_key 为推荐口径。
- 入口状态/profiles 写入走 AtomicWriter/guarded write + 私有权限 best-effort。

## 7. 测试设计

`TestGrokEnv`（tempdir + `CCR_ROOT`/`GROK_HOME`，参考 ccr-codex TestCodexEnv；全仓 `--test-threads=1` 约定覆盖 env 串行）。用例矩阵见 implement.md 第 5 步；新增关键组：往返恢复、CAS 冲突、删除激活拒绝、部分失败自愈、官方 default 键移除/恢复。

## 8. 兼容与回滚

- 纯增量；registry platforms 表按名扩展（PlatformConfigManager 已支持任意平台名）。
- 回滚 = revert；用户侧可从入口状态原文恢复 config.toml。
- 真实 grok 二进制不在开发机 PATH：验收以结构化 TOML 断言为门槛（父 PRD 已登记证据缺口）。
