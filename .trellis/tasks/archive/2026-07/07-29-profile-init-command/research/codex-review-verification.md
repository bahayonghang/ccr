# Codex 审阅（2026-07-29）逐条源码校验

审阅方结论：No-Go，6 项 P1。本文件记录逐条校验结果与采纳的修正方向。**全部 6 项属实。**

## 1. profiles.toml 写入方案违反红线 — 属实

- `crates/ccr-core/src/core/guarded_write.rs:129-163` `write_guarded_versioned`：文档明确「An empty token means the caller expects the target not to exist」——空 token 即 create-if-absent，持路径锁做检查+写入，无 TOCTOU；返回 `VersionedWriteOutcome::Conflict` 表示目标已被并发创建。
- `WriteOptions.secret`（guarded_write.rs:48-56）→ AtomicWriter 0o600（Unix），Windows no-op。`save_profiles_to_toml` 走的同一持久化层（base.rs:16 引入 `write_guarded`，base.rs:831-845 有 0o600 断言），init 用 `secret: true` 与之对齐。
- 原设计 exists() + fs::write：确有 TOCTOU 覆盖窗口 + 默认权限问题。
- **采纳**：init 写入改 `write_guarded_versioned(path, template, "", WriteOptions { secret: true, backup: None, .. })`；Conflict 分支按「已存在」幂等处理。补并发双 init 与 Unix 0600 测试。

## 2. 注册表无锁 RMW — 属实

- `crates/ccr-config/src/managers/platform_config.rs:340`、`:367`：`load` / `save` 文档均标注「此方法不加锁，调用方需要在外层使用 CONFIG_LOCK 保护 RMW 序列」。
- `crates/ccr-config/src/platforms/base.rs:30-69`：已有 `save_platform_registry(_with_paths)`（`platform_registry` 命名锁 + load_or_create_default + 变更前备份 + save），但为私有 `fn`。
- 旧 `platform_init_command`（ccr-cli）的注册流程本身就是无锁 RMW——不能照抄。
- **采纳**：在 `ccr-config/src/platforms/base.rs` 新增公开幂等 helper（`register_platform_if_missing` 形态：锁内 load → 已注册则不写不备份直接返回 false → 未注册则注册+备份+save 返回 true），init 调它。

## 3. 原样内嵌模板会伪造激活态 — 属实

- `examples/claude/profiles.example.toml:4-5`：`default_config = "anthropic"` / `current_config = "anthropic"`（非空）。
- `examples/codex/profiles.toml:6-7`：`current_config = "default"`（非空）。
- `crates/ccr-cli/src/platforms/claude.rs:150-161` + `:174-183` `stable_current_profile`：profiles.toml 的 `current_config` 非空且命中 profile → 视为当前 profile，且反向**修复注册表**（`update_registry_current_profile`）。codex（codex.rs:1611、:1937）、grok（grok.rs:691 `fallback_current_profile_from_file`）同构：空串 trim 后 → None。
- 即：init 写入带非空 current_config 的模板后，`profile current` 会立刻声称 profile mode 激活并污染注册表——违反 PRD「init 不进入 profile mode」。
- **采纳**：三份模板 `current_config = ""`（直接修改两份现有 examples 文件；`default_config` 非激活语义，保留）。验收补「init 后注册表 current_profile == None + 目标 CLI runtime 文件字节不变」。

## 4. Grok 双示例漂移 — 属实

- `docs/examples/grok-profiles.toml` 已存在（`current_config = ""`、official 走 session、relay 用 `https://api.example.com/v1` + `env_key`，全文无 inline token）——本身就是规范范本。
- 被 4 个 docs 文件以 raw.githubusercontent 链接引用：`docs/{,en/}reference/commands/grok.md`、`docs/{,en/}examples/index.md`。
- `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md:162-163`：「Copy-ready examples use `example.com` and `env_key`; inline secrets are disclosure documentation, not example values.」——原设计 D6（enabled 的 inline `xai-your-api-key-here`）直接违反规范。
- **采纳**：canonical = `examples/grok/profiles.toml`（内容以 docs 版为基底：session + env_key 两个可用示例；inline_api_key 仅以注释说明）；`docs/examples/grok-profiles.toml` 保持字节一致镜像（raw 链接不动），一致性用单测锁死（include_str! 两份 assert_eq）。

## 5. 退休命令提示遗漏 — 属实

- `crates/ccr-core/src/core/error.rs:316`：`PlatformNotFound` 用户提示仍推荐「运行 'ccr platform init <平台名>' 初始化平台」。
- `crates/ccr-cli/src/commands/lifecycle/init.rs:31`、`:156`：`ccr init` 两条路径的提示均推荐 `ccr platform init`。
- 原 PRD 文件范围没有 ccr-core。
- **采纳**：范围扩至 `crates/ccr-core`（仅文案）；两处提示改为 `ccr <platform> profile init`，并加回归断言（含 `ccr init` 输出与 PlatformNotFound 消息不再含 `ccr platform init`）。

## 6. jsonl 清单为空 seed — 属实

- `implement.jsonl` / `check.jsonl` 各只有 `_example` seed 行，不满足 Phase 1.3 门槛。
- **采纳**：填充 spec/research 清单（grok-profile-runtime、ccr-cli/ccr-config backend-guidelines、test-fixtures、本文件）。

## 产品决策（审阅方唯一待拍板项）

采用「安全未激活模板」：三平台模板 `current_config = ""`；Grok 可直接使用的示例仅 session / env_key，inline token 仅作注释说明。理由：该选项不是偏好而是被既有约束决定——PRD R1「init 不进入 profile mode」+ spec grok-profile-runtime.md:162 示例安全契约。用户可在 `task.py start` 前否决。

## 附带修正

- 验收「list 示例或空列表」收敛为：init 后 `profile list` 显示模板示例（grok：official/relay 两条），`current` 报「不在 profile mode」。
- 模板自检从「仅 parse + grok auth mode」升级为逐 profile 调平台 trait `validate_profile`（`crates/ccr-config/src/models/platform.rs:574`；claude.rs:371 / codex.rs:2091 / grok.rs:838 均有实现）。
- 测试面补齐：并发双 init、Unix 0600、未激活断言、三平台 `profile init --json` clap 解析与 help 路由。
