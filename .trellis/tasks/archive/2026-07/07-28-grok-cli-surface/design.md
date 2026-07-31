# Design：`ccr grok` 命令面（rev2 新增）

## 1. 架构对齐：per-platform 命令树，不复活退休面

现状核验：

- `ccr platform switch/current/info/init/profile` 在 `crates/ccr-cli/src/cli/dispatch.rs:348-361` 统一返回 `legacy_platform_command_error`，仅 `platform list` 存活（注册表视图）。`docs/reference/commands/platform.md` 明确其为退休入口并给出迁移映射。
- 现行架构：`crates/ccr-cli/src/cli/subcommands/claude.rs`（`ClaudeAction`/`ClaudeProfileAction`）+ `crates/ccr-cli/src/commands/claude/profile.rs` wrapper → 复用 `commands/platform/profile.rs` 的共享 `platform_profile_*_command`（create/set-field/enable/disable/delete）。codex 同构。

Grok 照抄该形态：

```
cli/subcommands/grok.rs      GrokAction / GrokProfileAction（clap 定义 + doc 注释示例）
cli/dispatch.rs              dispatch_grok 分支（Command::Grok）
commands/grok/mod.rs         模块组织
commands/grok/profile.rs     wrapper：组装 PlatformProfileCreateArgs{platform_name:"grok"}、
                             current/list/switch/off 的 grok 实现（调 create_platform(Grok)）
```

`ccr grok`（无子命令）行为：MVP 打印 help（不做 TUI 直跳；TUI tab 归 tui-tab 任务，后续可加）。

## 2. 共享机制改造点（CORR-004 落地）

| 位置                                                         | 改造                                                                                                         |
| ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| `ccr-config/models/platform.rs:108` `auth_profile_supported` | `[Claude, Codex, Grok]`；本文件测试 + `crates/ccr/tests/platforms/auth_profile_surface.rs` 同步              |
| `commands/platform/profile.rs` `parse_platform`              | 错误文案 "support only claude, codex and grok"                                                               |
| `commands/platform/profile.rs:212` create 硬门               | `matches!(Claude                                                                                             | Codex | Grok)` |
| `editable_fields()`                                          | 增加 `Platform::Grok => GrokPlatform::editable_fields()`（常量定义在 core 的 grok.rs，含 8 通用 + 4 特有键） |
| `update_profile_field()`                                     | 新增 Grok platform_data 键臂（见下）                                                                         |

### 类型化字段解析

`update_profile_field` 是跨平台共享函数，Grok 键臂需平台守卫（仅当 `editable_fields(platform)` 放行后进入——现有 `ensure_field_allowed` 已在入口做白名单，键名不冲突即可安全共存；claude/codex 白名单不含这四键，天然隔离）：

```rust
"api_backend"             => string，枚举校验 chat_completions|responses|messages（小写归一）
"env_key"                 => string；输入以 '[' 开头或含 ',' → 明确报错"MVP 仅支持单个环境变量名"
"context_window"          => value 解析 u64 且 > 0 → Number；失败报"context_window 需为正整数"
"supports_backend_search" => true/false/1/0 → Bool
```

四键写入 `profile.platform_data`；`--clear` → `shift_remove`。终校验由 `GrokPlatform::validate_profile`（create/set-field 命令尾部既有调用链）兜底，错误文案单一来源。

### Create 参数

`PlatformProfileCreateArgs` 增加 `api_backend/env_key/context_window/supports_backend_search: Option<...>` 字段（claude/codex wrapper 传 None，不影响既有调用方）；grok wrapper 全量传递。互斥校验（auth-token vs env-key）交给 validate_profile。

## 3. Off / Delete 组合语义

- `off`：`GrokPlatform::clear_active_profile_runtime()`（core 契约：恢复入口原条目 + 清指针）→ 成功输出恢复摘要（不含凭据）。幂等：无激活时提示已处于官方态。
- `delete <name>`：直接调引擎；引擎对激活项返回拒绝错误 → CLI 原样呈现（提示 off/switch）。`--force`：CLI 先调 off 再 delete（两步各自可失败，失败即停并如实报告）。
- `switch`/`current`/`list`：`create_platform(Grok)` 通用 trait 调用 + 输出卡片复用 `print_status_card` 等既有渲染（`current --json` 结构对齐 claude/codex 字段命名）。

## 4. 帮助与文档

- help 系统：`cli/help.rs`（或等价注册点）补 grok 顶层 + 嵌套条目，双语；clap doc 注释含示例（`\ 示例: ccr grok profile switch relay`风格对齐）。
- `docs/reference/commands/grok.md`：命令表、create 示例（env_key 推荐 + inline 披露一句话）、off 语义（恢复入口原条目）、与 `grok login` 的边界（auth.json 归 Grok 自身）。
- `docs/reference/commands/platform.md` 迁移映射补一行。docs 属 VitePress 包：`cd docs && npm run build` 验证。
- `docs/examples/grok-profiles.toml`：CCR profile 输入，覆盖官方纯模型选择器和第三方 env_key 模式；示例值只使用 `example.com` 与占位环境变量。
- `docs/examples/grok-cli-config.toml`：Grok 运行时目标形态，覆盖 `[model.custom]` 与 `[models].default`，保留一个非托管段证明并存语义；用临时 `GROK_HOME` 执行本机 `grok inspect`，不得触碰用户真实配置。
- `docs/examples/index.md` 与 `docs/en/examples/index.md` 同步新增两份示例入口。

## 5. 测试

- 单测：update_profile_field 四键类型化解析（合法/非法/clear）；editable_fields 白名单；parse_platform 门控。
- 集成（临时 CCR_ROOT+GROK_HOME）：create→switch→current(json)→set-field→off→delete 链路；--force 删除激活项；JSON 输出掩码断言；`ccr platform switch grok` 仍走 legacy 错误。
- 固定面：auth_profile_surface.rs 新集合。
