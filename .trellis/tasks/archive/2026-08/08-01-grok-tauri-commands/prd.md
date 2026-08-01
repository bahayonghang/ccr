# Grok Tauri 命令桥接层

> 父任务:`08-01-grok-ui-platform`(需求集/凭据边界/命令清单以父任务 prd.md + design.md 为准)。
> 依赖:无,任务树中最先实施。本任务交付后即冻结前后端契约,前端子任务方可 start。
> 修订记录:2026-08-01 依据 Codex 审阅修订(删 profiles raw;新增 activation inspection/patch 语义/status 信封/Local-only/CAS 重试)。

## Goal

在 `ccr-ui/src-tauri` 暴露 Grok 平台的全部 IPC 命令(profiles CRUD/apply/off、settings typed patch + config raw、首页聚合、版本探测与平台清单接入),复用 `ccr::platforms::GrokPlatform`;核心 crate 改动限定为只读 activation inspection API,以及保证 CRUD 不制造激活意图的 Grok 专用空 `current_config` 保留修复(含 spec 更新与测试)。

## Requirements

1. 实现父 design §2 命令清单(**不含 profiles raw**),注册进 `handler_registry.rs` 的 `grok:` 模块([SecretMutation, Generated]),再生命令清单与生成式 TS client;新增 ts-rs DTO 后再生 bindings。
2. 全部命令用 `#[ccr_tauri_command_macros::command]`,`async fn` + `Result<T, String>`,文件 I/O `spawn_blocking`,错误消息中文含上下文;**所有命令入口统一 Local-only 门控**(父 design D9),非 local 返回 `unsupported_environment` 信封。
3. 核心层新增 `GrokPlatform::inspect_activation_state()`(父 design D8.1):纯只读四态(inactive/active/drifted/unsafe_missing_entry_state),零副作用;配套单测与 `grok-profile-runtime.md` spec 更新。Tauri 层所有 active/drift 判定只用它,禁用 `get_current_profile()` 作意图判定。
4. Grok profile 保存必须保留保存前的 inactive 状态:空/缺席 `current_config` 且注册表无 current 时,create/update 后仍为空;保存与修正过程持 `grok_profile_operation` 锁,不得影响其他平台的共享 serializer。
5. 脱敏 DTO 按父 design D3:含 `profile_kind`/`has_base_url`/`auth_mode`/`has_inline_credential`;响应永不含 `api_key`/`auth_token` 明文;`base_url_display` 仅展示。
6. update 为 **patch 语义**(父 design D3):缺席=保留、null=清除、有值=替换;凭据走 `credential_action` 枚举(preserve/replace_api_key/replace_env_key/clear)。
7. rename 顺序「存新 → apply 新(若原为激活)→ 删旧」,部分失败返回结构化信封(`renamed | rename_apply_failed | rename_cleanup_failed`,父 design D8.3)。
8. delete 返回 `{status: deleted | blocked, reason}` 信封(父 design D8.2);force 仅在 `blocked(active|drifted)` 时 off+重删;`unsafe_missing_entry_state` fail-closed。
9. settings typed 按父 design D4:get 含五 section 现值 + `custom_models` 脱敏摘要 + `activation` + `managed_keys_locked`;update 为**白名单 set/unset 字段 patch + read/merge/CAS 重试(≤3,每次重查托管锁)**,未知表/键零丢失。
10. config raw 通道按父 design D5:CAS + `BackupPolicy::None`;`grok_list_config_layers` 含 user/project/**managed/requirements** 层存在性。
11. `system.rs` 与 `platform/local.rs` 补 grok(program=`grok`,配置目录 `GROK_HOME` 优先、缺省 `~/.grok`)。
12. `grok_get_dashboard_overview` 返回 `{activation, current_profile, auth_mode, profiles_total, profiles_enabled, config_exists, config_path_display}`(无 version 字段)。

## Acceptance Criteria

- [x] `cd ccr-ui/src-tauri && cargo check` 通过;`just tauri-command-inventory-check` 与 `just tauri-bindings-check` 通过
- [x] 核心层:inspection API 四态单测全过;`cargo test -p ccr-cli grok -- --test-threads=1` 与 `cargo test -p ccr --test commands grok_profile -- --test-threads=1` 回归全绿;spec `grok-profile-runtime.md` 已补签名与契约
- [x] 命令层测试:DTO 脱敏断言(无 api_key/auth_token/不安全 URL)、patch 三态语义、credential_action 四态、rename 三种结局(含激活改名)、delete 信封与 force 编排(inactive 不触发 off;unsafe 态拒绝)、settings 未知表 round-trip 保留、**CAS 并发测试**(typed 保存与 apply/外部写并发,验证重试与不丢键)、config raw 保存无备份产物、非 local 环境全命令拒绝
- [x] 隔离冒烟(临时 `CCR_ROOT`/`GROK_HOME`):CLI create→字段 patch→apply→off→delete;Tauri 激活改名/settings patch 由聚焦状态机与真实文件 CAS 测试覆盖,写入形态与 CLI 一致
- [x] `just fmt-check`、`just lint-strict`、`just test` 全绿;核心层 diff 仅含 inspection API、Grok inactive 指针保留修复、spec 与测试
- [x] 父 prd「凭据边界」「状态机与数据完整性」逐条自查通过

## Out of scope

- profiles.toml 源码编辑命令(评审裁剪)
- 前端任何文件(generated 产物除外)
- usage/sessions/auth 命令;托盘;WSL/SSH 支持
