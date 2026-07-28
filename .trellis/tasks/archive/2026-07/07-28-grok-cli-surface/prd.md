# Grok CLI 命令面：`ccr grok profile` 子命令树

## Goal

按现行 per-platform 命令架构（`ccr claude` / `ccr codex` 模式）为 Grok 提供 CLI 命令面：`ccr grok profile <current|list|switch|create|set-field|enable|disable|delete|off>`，复用共享 `platform_profile_*_command` 机制并为 Grok 特有字段提供类型化解析。**不复活已退休的 `ccr platform switch/profile` 路径**。

> rev2：吸收审阅 ARCH-001（原计划建立在已退休命令面上）与 CORR-004（grok 特有字段无法经现有 update_profile_field 写入）。已核验：`dispatch.rs:348-361` 对 platform switch/profile 返回 `legacy_platform_command_error`；`ccr claude/codex profile` 经 `commands/{claude,codex}/profile.rs` wrapper 复用共享机制；`update_profile_field` 白名单不含任何 platform_data 键（除 claude auth_mode）。任务升级为非轻量，补 design.md。

## Requirements

### R1 子命令树（对齐 claude.rs/codex.rs 模式）

- `crates/ccr-cli/src/cli/subcommands/grok.rs`：`GrokAction { Help, Profile }` + `GrokProfileAction { Help, Current{json}, List{json}, Switch{name}, Create{...}, SetField{...}, Enable{name}, Disable{name,force}, Delete{name,force}, Off }`。
- `Create` 参数：通用（name/description/base-url/auth-token/model/provider/provider-type/account/--tag/--disabled/--json）+ Grok 特有 `--api-backend`、`--env-key`、`--context-window`、`--supports-backend-search`。
- `Off`：退出 profile mode，调用 core 的 `clear_active_profile_runtime`（恢复入口原条目），对齐 `ccr claude profile off` 先例。
- `Delete` 当前激活项：默认拒绝（core 引擎行为）；`--force` = 先 off 再删（CLI 层组合，不绕过引擎检查）。
- dispatch：`dispatch_grok` + 顶层 `ccr grok` 注册 + help 系统（`ccr help grok` / `ccr help grok profile`）双语。

### R2 共享机制放开与类型化字段解析（CORR-004）

- `Platform::auth_profile_supported()` → `[Claude, Codex, Grok]`；`crates/ccr/tests/platforms/auth_profile_surface.rs` 固定面测试同步。
- `parse_platform` 错误文案、`platform_profile_create_command` 内 `matches!(Claude | Codex)` 硬门放开至含 Grok。
- `editable_fields(Platform::Grok)` 白名单：`description`/`base_url`/`auth_token`/`model`/`provider`/`provider_type`/`account`/`tags` + `api_backend`/`env_key`/`context_window`/`supports_backend_search`。
- `update_profile_field` 增加 Grok platform_data 键的**类型化解析**：
  - `api_backend`：string，取值 `chat_completions|responses|messages`
  - `env_key`：string（单字符串；array 输入明确拒绝）
  - `context_window`：正整数（字符串解析为 u64 > 0，写 Number）
  - `supports_backend_search`：bool（`true/false/1/0`）
  - 每项均支持 `--clear`；解析失败给中文校验错误
- 深度校验一律由 `GrokPlatform::validate_profile` 承担（create/set-field 后即时调用，沿既有流程），CLI 层不复制业务规则。

### R3 文档与帮助

- 新增 `docs/reference/commands/grok.md`（VitePress，结构对齐 `docs/reference/commands/CLAUDE.md` 的 claude 命令页；含 env_key 推荐口径与明文披露一句话）。
- `docs/reference/commands/platform.md` 迁移映射表补 grok 行（`ccr grok profile ...`）。
- 新增 `docs/examples/grok-profiles.toml`（CCR 输入，含官方与第三方/env_key 示例）和 `docs/examples/grok-cli-config.toml`（Grok 运行时 `[model.custom]`/`[models].default` 示例）；不得包含真实域名、账号或凭据，并同步中英文示例索引。
- help 文案、`ccr platform list` 展示（grok 自动进入 registry 视图，验证即可）。

### R4 约束

- 输出（含 `--json`）auth_token 恒掩码；base_url 展示走 core 的 `safe_base_url_for_display`。
- 仅命令面接线：切换/恢复/校验逻辑全部来自 GrokPlatform；发现引擎缺陷回报 core 任务返工。

## Acceptance Criteria

- [ ] `ccr grok profile create relay --base-url ... --auth-token ... --model grok-4.5 --api-backend responses` 成功；`switch`/`current --json`/`list --json` 全链路正确。
- [ ] `ccr grok profile set-field relay context_window --value 1000000` 等四个 Grok 特有字段类型化写入与 `--clear` 正确；非法值（api_backend 越界、context_window 非正整数、env_key 传 array）给中文错误。
- [ ] `ccr grok profile off` 恢复入口原条目并清指针；删除当前激活项默认拒绝、`--force` 走 off+删除。
- [ ] `ccr platform switch grok` 仍返回 legacy 迁移指引（退休面不复活）；`ccr platform list` 正常显示 grok。
- [ ] `auth_profile_surface.rs` 更新通过；help/docs 双语无遗留"仅 claude/codex"表述；`cd docs && npm run build` 通过（触碰 docs 时）。
- [ ] 两份 Grok 示例配置可直接复制、无真实凭据，并由中英文 `examples` 索引引用；`grok-cli-config.toml` 在临时 `GROK_HOME` 下可被本机 `grok inspect` 发现。
- [ ] 命令面集成测试（临时 CCR_ROOT/GROK_HOME）：create→switch→set-field→off→delete 全链路 + JSON 掩码断言；`just lint-strict` + `just test` 通过。

## Notes

- 前置依赖：`07-28-grok-platform-core` 合入（off/delete 语义、validate、safe-url helper 均为其契约）。
- rev2 起为完整任务（prd + design + implement）。
