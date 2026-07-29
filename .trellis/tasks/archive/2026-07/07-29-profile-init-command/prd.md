# profile init 子命令与平台示例配置补全

## Goal

`ccr platform init` 退休后迁移指引断裂：错误信息把用户导向 `ccr grok profile ...`，但该命令面没有任何初始化入口（`ccr grok profile init` 报 `unrecognized subcommand`）。需要为 claude / codex / grok 三个 profile 命令面新增 `init` 子命令，用于初始化平台目录结构与 `profiles.toml` 模板；同时补齐 `examples/grok/` 示例配置，并让 init 模板以 examples 为单一来源。

## Background / 复现

```
❯ ccr platform init grok
[ERR] 配置文件错误: legacy command retired: `ccr platform init` ... Use ... `ccr grok profile ...` for Grok profiles.

❯ ccr grok profile init
error: unrecognized subcommand 'init'
```

断裂点：

1. 旧 `platform_init_command`（`crates/ccr-cli/src/commands/platform/init.rs`）仍在代码里，但 dispatch 层（`crates/ccr-cli/src/cli/dispatch.rs` `dispatch_platform`）对 `PlatformAction::Init` 一律抛 `legacy_platform_command_error("init")`，没有等价的新入口。
2. `examples/` 只有 claude / codex / droid / gemini，没有 grok 示例；`examples/README.md` 的快速使用章节仍教用户跑已退休的 `ccr platform init <platform>`。
3. 除迁移错误本身外，仍有**可达的用户提示在推荐退休命令**：`crates/ccr-core/src/core/error.rs:316`（`PlatformNotFound` 建议）与 `crates/ccr-cli/src/commands/lifecycle/init.rs:31`、`:156`（`ccr init` 输出）。
4. 新用户在全新机器上没有「一条命令搭好 `~/.ccr/platforms/<platform>/` 骨架 + 模板 profiles.toml」的路径，只能靠 `profile create` 逐个传 flag。

审阅记录：Codex 审阅 6 项 P1 已逐条源码校验属实并全部采纳，见 `research/codex-review-verification.md`。

## Requirements

### R1: 新增 `init` 子命令（claude / codex / grok 三平台）

- `ccr claude profile init`、`ccr codex profile init`、`ccr grok profile init` 均可执行，语义一致。
- 行为：
  1. 创建平台目录结构（`PlatformPaths::ensure_directories`：platform_dir / history / backups）。
  2. 若 `profiles.toml` 不存在：写入内置模板（来源见 R2）。写入必须走 guarded 持久化层：`write_guarded_versioned` 空 token（create-if-absent，无 TOCTOU 覆盖窗口）+ `secret: true`（Unix 0o600），并发下最多一个进程成功写入，其余按「已存在」处理。
  3. 若 `profiles.toml` 已存在：**不覆盖、不报错**，提示已存在并展示路径（幂等；重复执行安全）。
  4. 在 `~/.ccr/config.toml` 注册表注册平台。注册必须是加锁的幂等 RMW（`platform_config.rs` 的 `load`/`save` 文档明确要求调用方持锁）；已注册时不产生任何写入或备份。
  5. 输出下一步指引（编辑模板 / `profile create` / `profile list` / `profile switch`）。
- **init 不进入 profile mode**：不触碰目标 CLI 运行时配置（`~/.claude/`、`~/.codex/`、`~/.grok/config.toml`）；init 后注册表 `current_profile == None`，`profile current` 报「不在 profile mode」。
- 支持 `--json` 输出（与 profile 命令面其余子命令一致），JSON 至少包含 `ok`、`platform`、`profiles_file`、`created`（bool，本次是否新建）、`registered`（bool，本次是否新注册）。
- 不提供 `--force` 覆盖选项（MVP 明确排除破坏性路径；后续如需覆盖必须走 backup-before-destructive-change）。

### R2: 安全未激活模板，examples 单一来源

- **三份模板 `current_config = ""`**（安全未激活）：三平台均以 profiles.toml 的 `current_config` 非空作为激活态信号并据此反向修复注册表（claude.rs:174-183 等），因此模板携带非空 `current_config` 会把「刚初始化」伪造成「已激活」。需同步修改两份现有示例文件的头部：
  - `examples/claude/profiles.example.toml`：`current_config = "anthropic"` → `""`
  - `examples/codex/profiles.toml`：`current_config = "default"` → `""`
  - `default_config` 无激活语义，保留原值。
- 新增 `examples/grok/profiles.toml` 作为 grok 的 **canonical** 示例，遵守 `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md:162` 契约：copy-ready 示例只用 `example.com` + `env_key`（及无凭据的 session 模式）；inline API key 仅以注释说明，不作为示例值。内容以既有 `docs/examples/grok-profiles.toml` 为基底扩展。
- **消除 grok 双示例漂移**：`docs/examples/grok-profiles.toml` 已被 4 个 docs 页面以 raw 链接引用，保留该文件但内容与 `examples/grok/profiles.toml` 字节一致（镜像），一致性由单测锁死；raw 链接无需变更。
- init 模板通过 `include_str!` 内嵌 examples 文件：
  - claude → `examples/claude/profiles.example.toml`
  - codex → `examples/codex/profiles.toml`
  - grok → `examples/grok/profiles.toml`
- 模板自检：每份模板 `parse_profiles_from_str` 解析成功，且逐 profile 通过对应平台的 `validate_profile`（trait 见 `crates/ccr-config/src/models/platform.rs:574`）；grok 模板不得出现 `auth_token` 与 `env_key` 同设。

### R3: 迁移指引与帮助文本闭环（含全部可达提示）

- `ccr platform init` 的迁移报错文案明确给出 `ccr claude/codex/grok profile init`（新增专用文案，不动其余 platform 子命令的通用文案）。
- 清理仍推荐退休命令的可达输出，并加回归断言：
  - `crates/ccr-core/src/core/error.rs:316`（`PlatformNotFound` 建议行）
  - `crates/ccr-cli/src/commands/lifecycle/init.rs:31`、`:156`（`ccr init` 提示）
- `ccr claude/codex/grok profile help` 与 `--help` 列出 `init` 子命令。
- `examples/README.md` 更新：目录树补 grok（及现状缺失的 droid）；快速使用章节改为 `ccr <platform> profile init` 流程，移除已退休用法。
- `docs/` 中直接教用户执行 `ccr platform init` 的中英文页面同步为新命令（至少 `docs/{,en/}reference/commands/platform.md`；其余出现处以指引方式指向新命令，历史记述类页面仅补指向）。

## Non-goals / Out of scope

- gemini / droid / opencode 的 profile init（opencode 无 profile 命令面；gemini / droid 暂无对应子命令 enum）。相关 docs 不得虚构这两个平台的 init 命令。
- TUI / ccr-ui (Tauri) / VSCode 扩展的 init 入口（TUI 空态已在 07-29-grok-tui-order-empty-state 清理）。
- 删除旧 `platform_init_command` 死代码。
- `--force` 覆盖已有 profiles.toml。

## Constraints

- 红线：profiles.toml 写入必须无 TOCTOU（guarded versioned write）+ secret 权限；注册表 RMW 必须持锁；不得引入明文密钥打印（模板占位符除外，grok 遵守 example-safety 契约）。
- 内部实现注释用中文；公共 API 文档用英文。
- 改动范围：`crates/ccr-cli`、`crates/ccr-config`（公开注册 helper）、`crates/ccr-core`（仅提示文案）、`crates/ccr`（tests）、`examples/`、`docs/`。

## Acceptance Criteria

- [ ] `ccr grok profile init` 首次执行：创建 `~/.ccr/platforms/grok/` 结构 + 模板 `profiles.toml`，退出码 0；`ccr grok profile list` 显示模板示例（official / relay 两条，均无激活标记）；`ccr grok profile current` 报「不在 profile mode」；注册表 `current_profile == None`；`~/.grok/config.toml`（如存在）字节不变。claude / codex 同构验收（各自 runtime 文件不变）。
- [ ] 二次执行 init：`profiles.toml` 字节不变、无新备份产生，提示已存在，退出码 0。
- [ ] 并发双 init（两个进程同时执行）：均退出码 0，最终文件内容 == 模板，无损坏。
- [ ] Unix 下 init 产物 `profiles.toml` 权限为 0o600（cfg(unix) 测试）。
- [ ] `--json` 输出含 `ok/platform/profiles_file/created/registered` 且可解析；三平台 `profile init --json` 的 clap 解析测试通过。
- [ ] `ccr platform init grok` 报错文案含 `ccr grok profile init`；`ccr init`（已初始化路径）与 `PlatformNotFound` 提示不再含 `ccr platform init`（回归断言）。
- [ ] `ccr grok profile help` / `--help` 输出含 `init`（claude / codex 同理）。
- [ ] 模板自检测试：三份模板 parse 成功 + 逐 profile `validate_profile` 通过 + 三份模板 `current_config` 为空；`examples/grok/profiles.toml` 与 `docs/examples/grok-profiles.toml` 字节一致（同步测试）。
- [ ] `examples/grok/profiles.toml` 存在且符合 grok-profile-runtime spec 的示例安全契约；`examples/README.md` 更新且 `rg "platform init" examples/` 无残留。
- [ ] `just version-check`、`just fmt-check`、`just lint-strict`、`just test` 通过；docs 改动通过 `cd docs && npm run build`（或 `just frontend-check`）。
