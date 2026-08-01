# Grok 后端能力现状(UI 复用评估)

> 调研日期 2026-08-01,基于 dev 分支。结论:**核心能力已 100% 可复用,缺口全部在 Tauri 命令层与前端。**

## 1. 核心实现位置

| 文件                                              | 内容                                                                                                     |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `crates/ccr-cli/src/platforms/grok.rs`(~1770 行)  | `GrokPlatform`、`GrokProfileAuthMode`(`InlineApiKey`/`EnvKey`/`Session`)、私有 `ProfileEntryConfigState` |
| `crates/ccr-cli/src/commands/grok/profile.rs`     | CLI 处理器 + **私有** JSON DTO(`GrokProfileSummary` 等,已做脱敏,不可直接复用)                            |
| `crates/ccr-cli/src/commands/platform/profile.rs` | 共享 profile CRUD;`editable_fields(Platform::Grok)`                                                      |
| `crates/ccr-config/src/models/platform.rs`        | `Platform::Grok`(短名 `grok`、显示名 `Grok Build`、`is_implemented()==true`)                             |

`GrokPlatform` 公开 API(UI 直接可用):

- `new()`;`PlatformConfig` trait:`load_profiles / save_profile / delete_profile / apply_profile / validate_profile / get_current_profile / get_settings_path`
- `clear_active_profile_runtime()`(= off)
- `profile_auth_mode(&ProfileConfig) -> GrokProfileAuthMode`(不暴露凭据)
- `safe_base_url_for_display(&str)`(剥离 userinfo/query/fragment)
- `editable_fields()`(17 字段白名单)、`normalize_reasoning_effort()`(7 级:none/minimal/low/medium/high/xhigh/max)
- `get_env_var_names()` → `["XAI_API_KEY", "GROK_CODE_XAI_API_KEY"]`

## 2. 文件路径与格式(全 TOML)

| 文件            | 路径                                                    | 说明                                                                                                                                    |
| --------------- | ------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Grok 运行时配置 | `$GROK_HOME/config.toml`(默认 `~/.grok/config.toml`)    | CCR 只管理 `[model.custom]`、`[models].default`、`[models].default_reasoning_effort`;保留未知表;不碰 `auth.json`/`mcp_credentials.json` |
| CCR profiles    | `~/.ccr/platforms/grok/profiles.toml`                   | 共享 `ConfigSection` 编码;模板 `examples/grok/profiles.toml`                                                                            |
| 入口状态快照    | `~/.ccr/platforms/grok/profile_entry_config_state.json` | 首次 apply 捕获原始 runtime,off/官方 profile 用于恢复                                                                                   |
| 注册表指针      | `~/.ccr/config.toml` → `platforms.grok.current_profile` | 当前 profile 意图                                                                                                                       |
| 备份            | `~/.ccr/backups/grok/`(仅 profiles.toml)                | **runtime config.toml 有意不备份**(spec 红线)                                                                                           |

Profile 字段(第三方):`base_url` + `model` 必填,凭据三选一(`api_key` / 兼容 `auth_token` / 单字符串 `env_key`);可选 `api_backend`(chat_completions|responses|messages)、`context_window`(正整数)、`supports_backend_search`(bool)、`reasoning_effort`(7 级)。官方 profile:仅 `model` 可选,拒绝 base_url 与一切凭据。

## 3. 切换写入序列(UI 无需重复实现)

`apply_profile` 全程持跨进程锁 `grok_profile_operation`(10s 超时):

1. 入口状态 create-if-absent CAS(已存在 Conflict 不覆盖基线)
2. runtime config.toml 内容 token CAS,冲突重试 1 次,二次冲突报「请重试」;`secret:true` + `BackupPolicy::None`
3. profiles.toml `current_config`(带备份轮换)
4. 注册表指针

`off` 反向恢复;入口状态缺失但有激活意图时 **fail-closed**。`get_current_profile` 做 drift 检测。delete 拒绝激活/漂移中的 profile;「off 再删」的 force 编排在 CLI 层 `delete_command`(profile.rs:298-329),**核心层不含,Tauri 层需照抄 ~10 行 match**(spec 禁止无条件 off)。

## 4. Spec 红线(`.trellis/spec/ccr-cli/backend/grok-profile-runtime.md`)

- **禁止直接序列化 `ProfileConfig` 到对外 JSON**:`platform_data["api_key"]` 是明文 `serde_json::Value`(`auth_token` 有 `Secret` 掩码,platform_data 没有)。
- 对外 JSON 只暴露 `auth_mode` + `safe_base_url_for_display` 结果 + `env_key` 名,永不回传 `api_key`/`auth_token` 明文。
- 错误/日志不含 token、不回显凭据 TOML 解析原文、URL 一律过 safe helper。
- runtime config.toml 写入 `BackupPolicy::None`(备份会新增未披露明文凭据位置)。
- `auth.json` / `mcp_credentials.json` 永不读写。
- force delete:仅在收到 active-profile 拒绝时才 off + 重删。

## 5. crate 关系(关键结论)

- `crates/ccr-cli` 是 **lib crate**(`[lib] name = "ccr_cli"`);`crates/ccr` 是 facade,`ccr::platforms::* = ccr_cli::platforms::*`(`crates/ccr/src/lib.rs:187`),`grok::{GrokPlatform, GrokProfileAuthMode}` 已导出(`ccr-cli/src/platforms/mod.rs:26`)。
- **`ccr-ui/src-tauri` 已依赖 `ccr-cli`**(Cargo.toml:28)与 `ccr` facade → `use ccr::platforms::GrokPlatform` 今天就可用,**无需下沉共享 crate、无需改 Cargo.toml**。旁证:ccr-tui 已有 GrokProfile tab。
- 若未来 grok 长出 auth/usage/sessions,再仿 `ccr-codex` 抽域 crate(可选重构,不在本任务)。

## 6. CLI 命令面(对照参考)

`ccr grok profile`:`current`/`list`(--json)、`switch`、`create`(--base-url/--api-key/--env-key/--model/--api-backend/--context-window/--supports-backend-search/--reasoning-effort/…)、`set-field`(--value/--value-json/--clear)、`enable`/`disable`、`delete`(--force)、`open`、`init`、`off`。文档:`docs/reference/commands/grok.md`。

## 7. 测试基线

- 黑盒:`crates/ccr/tests/commands/grok_profile.rs`(458 行):init 幂等/并发/0600 权限、全流程 create→switch→current(脱敏)→set-field→off(完整还原)→delete、force delete、数组 env_key 拒绝。
- 单测:grok.rs 内嵌 ~25 个(CAS 冲突/锁互斥/fail-closed/drift/TOML 错误不回显密钥)。
- 运行方式:`cargo test -p ccr --test commands grok_profile -- --test-threads=1`。

## 8. Grok Build 官方配置面(settings 可视化范围参考)

来源:docs.x.ai/build/settings(2026-07)+ xai-org/grok-build 仓库 README。

- 用户配置 `~/.grok/config.toml` sections:`[models]`(default、web_search、default_reasoning_effort)、`[model.<id>]`(model/base_url/name/api_key/env_key/api_backend/context_window/supports_backend_search/supports_reasoning_effort/reasoning_effort)、`[ui]`(theme)、`[session]`(auto_compact_threshold_percent 0-100 默认 85、load_envrc 默认 true)、`[cli]`(auto_update、channel stable|alpha、show_tips)、`[hints]`(new_session_worktree_mode、fork_worktree_mode:ask|always|never)、`[permission]`、`[features]`、`[tools]`、`[toolset.*]`、`[mcp_servers]`、`[sandbox]`、`[auth]`、`[telemetry]`、`[skills]`、`[plugins]`、`[compat.*]`、`[subagents]`、`[memory]`
- 项目级 `.grok/config.toml` 只贡献 `[mcp_servers]`、`[plugins]`、`[permission]`。
- **完整配置层级(优先级降序)**:CLI flags → 环境变量(`GROK_*`)→ 用户 `~/.grok/config.toml` → 项目 `.grok/config.toml` → **Managed**(`~/.grok/managed_config.toml`、`/etc/grok/managed_config.toml`,企业下发默认值)→ **Requirements**(`~/.grok/requirements.toml`、`/etc/grok/requirements.toml`,策略钉死)→ 内置默认。UI 的 layers 面板须列出 managed/requirements(只读),存在时提示用户设置可能被策略覆盖。
- `GROK_HOME` 环境变量覆盖配置目录;`grok inspect` 显示发现的配置源。
