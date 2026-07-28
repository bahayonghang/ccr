# Implement：`ccr grok` 命令面（rev2）

> 执行前置：core 已合入；`python ./.trellis/scripts/task.py start 07-28-grok-cli-surface`；读 `.trellis/spec/ccr-cli/backend/index.md` checklist。

## 步骤清单

### 1. 门控与共享机制（CORR-004）

- [ ] `auth_profile_supported()` → 含 Grok；ccr-config 内测试 + `crates/ccr/tests/platforms/auth_profile_surface.rs` 同步。
- [ ] `parse_platform` 文案、`platform_profile_create_command` 硬门放开。
- [ ] `editable_fields(Grok)`（常量在 core grok.rs，此处接线）。
- [ ] `update_profile_field` 四键类型化臂（api_backend 枚举 / env_key 单字符串守卫 / context_window 正整数 / supports_backend_search bool；各含 --clear 与中文错误）。
- [ ] `PlatformProfileCreateArgs` 扩展四个 Option 字段（claude/codex wrapper 传 None）。
- 验证：`cargo test -p ccr-cli platform -- --test-threads=1`；`cargo test -p ccr --test platforms -- --test-threads=1`

### 2. 子命令树与 wrapper

- [ ] `cli/subcommands/grok.rs`：GrokAction/GrokProfileAction（clap + 双语 doc 示例，对齐 claude.rs 风格）。
- [ ] `commands/grok/{mod.rs,profile.rs}`：current/list/switch/create/set-field/enable/disable/delete/off wrapper（复用共享命令 + `create_platform(Grok)`；off → `clear_active_profile_runtime`；delete --force = off→delete 组合）。
- [ ] `cli/dispatch.rs` `dispatch_grok` + 顶层命令注册；help 系统补 grok 条目（双语）。
- 验证：`cargo check -p ccr-cli`；手动（临时 env）`ccr grok profile create/switch/current --json`

### 3. 文档

- [ ] 新增 `docs/reference/commands/grok.md`（命令表/示例/env_key 推荐/off 语义/auth.json 边界）。
- [ ] `docs/reference/commands/platform.md` 迁移映射补 grok 行。
- [ ] 新增 `docs/examples/grok-profiles.toml` 与 `docs/examples/grok-cli-config.toml`，同步中英文示例索引；确认无真实域名/账号/凭据。
- [ ] 以临时 `GROK_HOME` 放置 `grok-cli-config.toml` 并运行 `grok inspect`，记录本机 Grok 配置发现结果且不访问用户真实 `~/.grok`。
- [ ] `rg -n "claude 和 codex|claude and codex|仅支持 claude / codex" crates/ docs/` 扫尾（限 profile/auth 命令语境）。
- 验证：`cd docs && npm run build`

### 4. 测试与收尾门

- [ ] 集成链路测试（create→switch→set-field→off→delete、--force、JSON 掩码、legacy platform switch 仍拒绝）。
- [ ] `just fmt` → 查 diff → `just fmt-check` → `just lint-strict` → `just test`
- [ ] 提交：`feat(cli): ✨ add ccr grok profile command surface`

## 回滚点

- 步骤 1（共享机制）与步骤 2-3（新命令树/文档）分 commit，各自可 revert。

## 明确不做

- 复活 `ccr platform switch/profile`；GrokPlatform 引擎逻辑修改（缺陷回报 core）
- `ccr grok` 直跳 TUI（tui-tab 合入后另议）；ccr-ui
