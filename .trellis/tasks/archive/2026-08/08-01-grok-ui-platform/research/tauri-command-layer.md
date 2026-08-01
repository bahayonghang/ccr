# Tauri 命令层模式(src-tauri)

> 调研日期 2026-08-01。目标:为 grok 新增命令。

## 1. 注册结构

- 入口:`ccr-ui/src-tauri/src/main.rs:390` `.invoke_handler(commands::generate_handler())`;`commands/mod.rs` 只做模块声明。
- **单一事实来源**:`commands/handler_registry.rs` 的 `define_command_registry!`,每模块 = `key: "标题" [默认Risk, Generated|LegacyJson] => [ super::<mod>::<command> => ["输入TS","输出TS","TS client 声明"], ... ]`。同时生成:
  - `tauri::generate_handler![]`(Windows 分支追加 wsl)
  - 运行时 capability manifest:`audit_invoke` 拒绝未注册命令;按 risk 推导 timeout/并发/审计;**mutation 命令要求前端 payload 带 `confirmationToken: 'desktop-confirm:<command>'`**
  - 生成物:`permissions/command-inventory.toml`、`docs/reference/tauri-command-inventory.md`(中英)、`src/api/generated/command-manifest.json`、`commandCapabilities.ts`、各模块生成式 TS client。再生:`just tauri-command-inventory`;校验:`just tauri-command-inventory-check`
- 命令属性宏:`#[ccr_tauri_command_macros::command]`(非裸 `#[tauri::command]`),强制 `async fn` + `Result<T, String>`,函数体包进 `runtime_policy::execute`。
- 命名:`<platform>_<verb>_<noun>` snake_case;risk 前缀启发式(`list_/get_/detect_` → ReadOnly;`delete_/clear_` → Destructive)。
- 文件组织:Claude/Codex = 门面文件 + `#[path]` 子模块;Gemini/OpenCode = 单文件(grok 起步仿单文件)。

## 2. 参照命令面

**Claude**(`claude_profiles.rs`,模块 [SecretMutation, Generated],全部 `Result<OpenJsonValueDto, String>` + `spawn_blocking`):
`claude_list_profiles`(返回 `{profiles: [...], current_profile}`)/ `claude_get_profile` / `claude_add_profile(request)` / `claude_update_profile(name, request)`(改名 + 当前 profile 时重新 apply)/ `claude_delete_profile` / `claude_apply_profile` / `claude_export_profiles(include_secrets)` / `claude_get_profiles_raw(state)` / `claude_save_profiles_raw(state, content, token, force)`。

**Codex**:同套 + `codex_get_profile_env` / `codex_list_models`;mutation 结束调 `invalidate_codex_dashboard_overview_cache(&state)`。Settings:`codex_get_settings`/`codex_update_settings` — `CodexConfig` 结构体(**`#[serde(flatten)] other: HashMap<String, toml::Value>` 保留未知字段**),写入 `ccr_core::AtomicWriter::new(path).secret(true)`。

**Raw 通道**(`settings_raw.rs`):`codex_get/save_config_raw_text` 用 `write_guarded_versioned` + `BackupPolicy::Dir{backups_dir}` + `content_version_token`,返回 `status: saved|conflict|invalid`;校验错误不回显文件内容。⚠️ grok 的 runtime config 按 spec 必须 `BackupPolicy::None`(与 codex raw 默认不同,需专门处理)。

**共享 helper**:`commands/profile_lifecycle.rs` 的 `profiles_raw_payload_from_paths`/`save_profiles_raw_to_paths` 平台无关(传 `PlatformPaths::new(Platform::Grok)` 即可),含语法/语义校验、激活冲突检测、CAS、备份(profiles.toml 可备份,无红线)。

## 3. 平台首页数据命令

- `commands/system.rs`:`get_cli_versions`/`get_cli_version(state, options)`(模块 system_extended [ProcessExecution]);`CLI_VERSION_TOOLS = ["ccr","claude","codex","gemini"]` **缺 grok**;tool→program 映射与 `normalize_cli_tool` 别名表需补 grok(program = `grok`)。带 AppState 缓存与超时探测,返回 `CliVersionEntry{status: ok|timeout|error|not_installed}`。
- `platform/local.rs`:`list_platforms()`/`detect_cli_status()`/`config_base_dir()` 硬编码 claude/codex/gemini/opencode,grok 均缺席(影响 `env_list_platforms`/`env_detect_cli`)。
- Codex 首页聚合范式:`codex_get_dashboard_overview(force?)` — 指纹缓存 + `spawn_blocking` 构建 `{auth, profiles, config, inventory}`。

## 4. DTO/序列化惯例

- 开放 JSON:`commands/wire.rs` 的 `OpenJsonValueDto`(untagged,ts-rs 导出 `src/types/generated/common/`),平台 CRUD 默认类型。
- 强类型 DTO:定义在命令模块内,derive `Serialize + TS`,`#[ts(export, export_to = "../../src/types/generated/<domain>/")]`;枚举 `#[serde(rename_all = "snake_case")]`;**struct 字段保持 snake_case**;invoke 参数 key 在生成 client 中 camelCase(Tauri 2 自动转换)。
- 错误映射:统一 `.map_err(|e| format!("中文上下文: {e}"))`;spawn_blocking join 错误 → `"任务执行失败: {e}"`。
- 掩码:核心层 `ccr_core::Secret` 负责 auth_token;⚠️ claude/codex 的 `profile_to_json` 目前有意 `Secret::expose` 原文给编辑表单预填(注释声明掩码化属 typed-ipc 任务)——**grok 按 spec 不得沿用**,凭据只写不读。

## 5. grok 新增命令文件清单

| #   | 文件                                         | 动作                                                              |
| --- | -------------------------------------------- | ----------------------------------------------------------------- |
| 1   | `src-tauri/src/commands/grok.rs`(新建)       | 命令实现,调 `ccr::platforms::GrokPlatform`                        |
| 2   | `src-tauri/src/commands/mod.rs`              | `pub mod grok;`                                                   |
| 3   | `src-tauri/src/commands/handler_registry.rs` | 新增 `grok:` 模块条目 [SecretMutation, Generated] + wire contract |
| 4   | `src-tauri/src/commands/system.rs`           | `CLI_VERSION_TOOLS`、tool→program、`normalize_cli_tool` 补 grok   |
| 5   | `src-tauri/src/platform/local.rs`            | `list_platforms`/`detect_cli_status`/`config_base_dir` 补 grok    |
| 6   | `src-tauri/src/commands/settings_raw.rs`     | grok raw 读写(注意 BackupPolicy 决策)                             |
| 7   | `just tauri-command-inventory`               | 再生 inventory/manifest/生成式 TS client                          |
| 8   | `Cargo.toml`                                 | **无需改动**                                                      |

验证:`cd ccr-ui/src-tauri && cargo check`、`just tauri-command-inventory-check`、`cd ccr-ui && bun run type-check`。相关 spec:`.trellis/spec/ccr/backend/tauri-handler-registry.md`;可用 `tauri-command-scaffold` skill。
