# Design: profile init 子命令与平台示例配置补全

> 2026-07-29 v2：按 Codex 审阅 6 项 P1 修订（校验记录见 `research/codex-review-verification.md`）。
> 主要变化：写入改 guarded versioned write、注册走加锁幂等 helper、模板改「安全未激活」形态、
> grok 双示例收敛为镜像同步、迁移文案范围扩至 ccr-core 与 lifecycle init。

## 总体思路

在既有 per-platform profile 命令面（claude / codex / grok）上各加一个 `Init` 子命令，三者共享一个实现函数；模板内容用 `include_str!` 内嵌仓库 `examples/` 下对应示例文件，实现「示例 = init 产物」的单一来源。不复活全局 `ccr platform init` 路由；所有仍推荐退休命令的可达输出全部改指新命令。

## 关键决策

### D1: 共享实现放在 platform 命令层，写入走 guarded 持久化层

新增 `crates/ccr-cli/src/commands/platform/profile_init.rs`：

```rust
pub async fn platform_profile_init_command(
    platform_name: &str,   // "claude" | "codex" | "grok"
    template: &str,        // include_str! 的 examples 内容
    json: bool,
) -> Result<()>
```

流程：

1. `Platform::from_str` → `PlatformPaths::new` → `paths.ensure_directories()`。
2. 模板自校验：`parse_profiles_from_str(template)` 失败即返回 `ConfigError`（防御 examples 被改坏后写出坏文件；正常情况下单测已在 CI 拦截）。
3. **profiles.toml 写入（修订）**：不用 `exists() + fs::write`（TOCTOU + 权限缺陷），改用
   `write_guarded_versioned(&paths.profiles_file, template.as_bytes(), "", &WriteOptions { secret: true, backup: BackupPolicy::None, ..Default::default() })`
   （`crates/ccr-core/src/core/guarded_write.rs:134`）：
   - 空 `expected_token` = create-if-absent，检查与写入同持路径锁，无覆盖窗口；
   - `secret: true` → Unix 0o600（与 `save_profiles_to_toml` 的权限口径一致）；
   - 返回 `Written` → `created = true`；返回 `Conflict` 或预读发现文件已存在 → `created = false`（幂等「已存在」路径，不备份不改写）。
   - 预读仅用于省去已存在场景的无谓锁竞争，最终判定以 versioned write 结果为准。
4. **注册表注册（修订）**：不照抄旧 `platform_init_command` 的无锁 `load_or_create_default → register_platform → save`（违反 `platform_config.rs:340/:367` 的调用方持锁契约）。在 `crates/ccr-config/src/platforms/base.rs` 将既有私有 `save_platform_registry` 体系扩展出公开幂等 helper：

   ```rust
   /// Registers a platform in the unified registry if absent.
   /// Locked RMW; returns whether a new entry was written.
   pub fn register_platform_if_missing(platform_name: &str, description: &str) -> Result<bool>
   ```

   语义：`platform_registry` 命名锁内 load → 已注册 → 不写、不备份、返回 `false`；未注册 → 注册 + 变更前备份 + save → 返回 `true`。锁常量与备份 tag 复用 base.rs:23-56 现有实现。
5. 输出：human 模式 `ColorOutput` 打印路径、created/registered 状态与下一步指引；`--json` 输出 `{ok, platform, profiles_file, created, registered}`。

### D2: 安全未激活模板（修订，含产品决策）

**决策：三平台模板 `current_config = ""`；grok copy-ready 示例仅 session / env_key，inline token 只作注释说明。**
依据：三平台都把 profiles.toml 的非空 `current_config` 当激活态信号并反向修复注册表（claude.rs:150-161/174-183、codex.rs:1611/1937、grok.rs:691），原样内嵌现有示例会让 init 直接伪造 profile mode，违反 R1；grok 的示例安全形态由 spec `grok-profile-runtime.md:162-163` 强制。

- claude → `examples/claude/profiles.example.toml`：头部 `current_config` 改空串（`default_config = "anthropic"` 保留，无激活语义）；其余内容不动。
- codex → `examples/codex/profiles.toml`：`current_config` 改空串；其余内容不动。
- grok → 新建 `examples/grok/profiles.toml`（形状见 D6）。
- include 路径用 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/<...>"))`；ccr-cli 编译依赖仓库内 examples 文件（workspace 构建成立，本 crate 不单独发布，模块注释写明）。
- `examples/claude/profiles.toml`（历史乱码遗留）不碰、不引用。
- 效果收敛：init 后 `profile list` 显示模板示例条目（占位符明显、无激活标记），`profile current` 报「不在 profile mode」，注册表 `current_profile == None`，目标 CLI runtime 文件不被触碰。

### D3: CLI 枚举与 dispatch

三个枚举各加一个 variant（英文 doc comment，风格与邻近 variant 一致）：

- `crates/ccr-cli/src/cli/subcommands/claude.rs` `ClaudeProfileAction::Init { #[arg(long)] json: bool }`
- `crates/ccr-cli/src/cli/subcommands/codex.rs` `CodexProfileAction::Init { ... }`（enum 在 codex.rs:150）
- `crates/ccr-cli/src/cli/subcommands/grok.rs` `GrokProfileAction::Init { ... }`

variant 放在 `Help` 之后、`Current` 之前（bootstrap 动作排最前）。dispatch（`crates/ccr-cli/src/cli/dispatch.rs`）三处 match 各加一臂，转发到各平台 `commands/<platform>/profile.rs::init_command`，后者调 `platform_profile_init_command("<platform>", TEMPLATE, json)`。

### D4: 迁移指引闭环（修订：覆盖全部可达提示）

1. `crates/ccr-cli/src/commands/migration.rs` 新增专用函数（不动既有通用文案，避免影响 switch/current/info/profile 的既有断言）：

   ```rust
   pub fn legacy_platform_init_error() -> CcrError  // 文案含三条 ccr <p> profile init
   ```

   `dispatch_platform` 的 `PlatformAction::Init` 分支改调它；`crates/ccr/tests/commands/help.rs:128-130` 断言同步。
2. `crates/ccr-core/src/core/error.rs:316`：`PlatformNotFound` 建议行 `ccr platform init <平台名>` → `ccr claude/codex/grok profile init`（仅文案，枚举不动）。
3. `crates/ccr-cli/src/commands/lifecycle/init.rs:31`、`:156`：两处 `ccr platform init` 提示改为新命令。
4. 以上三处均加「输出不含 `ccr platform init`」的回归断言。

### D5: 帮助文本

clap derive 自动把新 variant 纳入 `--help` 与 `help::print_nested_subcommand_help(&["<platform>", "profile"])`。`help_config.rs` 的 `PLATFORM_AFTER_LONG_HELP` 增加一行「初始化平台配置目录: ccr <platform> profile init」。

### D6: grok 示例内容与双文件收敛（修订）

**canonical = `examples/grok/profiles.toml`；`docs/examples/grok-profiles.toml` 为字节一致镜像。**
docs 版已被 `docs/{,en/}reference/commands/grok.md` 与 `docs/{,en/}examples/index.md` 以 raw 链接引用——保留文件与链接，内容同步为新 canonical，单测 `assert_eq!(include_str!(examples), include_str!(docs))` 锁死漂移。
（备选「删 docs 版 + 改链接」被否：破坏 docs/examples 目录对其余平台的既有约定，churn 更大。）

内容以现有 docs 版为基底（其头部 `current_config = ""`、relay 用 `https://api.example.com/v1` + `env_key`、official 走 session，已合规）扩展：

1. `[official]` session 模式：无凭据字段，注明认证交给 Grok 自身会话 / `XAI_API_KEY` 全局环境变量。
2. `[relay]` env_key 模式：`example.com` base_url、`env_key = "GROK_RELAY_API_KEY"`、`api_backend` / `context_window` / `supports_backend_search` 平台字段示范。
3. inline_api_key 模式**仅注释说明**（含 `ccr grok profile create --auth-token ...` 指引），不出现在任何解析生效的 section 中——遵守 spec「inline secrets are disclosure documentation, not example values」。

字段名对齐 `GROK_EDITABLE_FIELDS`（grok.rs:22-35）；头部注释说明目标路径与 `ccr grok profile init` 用法。

## 影响面与兼容性

- 纯新增子命令 + 文案修订，不改任何既有子命令行为；`profile create` 的自建目录路径不受影响。
- 修改两份现有 examples 头部 `current_config`：影响「手工复制示例」用户——原行为（复制即伪激活）本就是坑，改后复制文件不再伪造 profile mode，属修复。
- 秘钥红线：模板仅含占位符 / env_key；init 不读不写真实凭据，不触碰 `~/.claude` / `~/.codex` / `~/.grok`。
- 回滚：revert 单个 commit 即可，无数据迁移。

## 测试策略

- 集成测试（`crates/ccr/tests/commands/{claude,codex,grok}_profile.rs`，沿用各自 fixture 的 `CCR_ROOT` / `*_HOME` 隔离）：
  - 首次 init：文件存在、`list` 显示模板条目、`current` 报未激活、注册表 `current_profile == None`、runtime 文件字节不变、退出码 0；
  - 二次 init：profiles.toml 字节不变、备份目录无新增、输出含「已存在」语义；
  - 并发双 init：同一 fixture 下同时拉起两个 ccr 进程执行 init，均退出 0，最终文件 == 模板；
  - `--json` 字段断言（`created` 首次 true / 二次 false）；
  - cfg(unix)：init 产物权限 0o600。
- 模板自洽单测（`profile_init.rs` 内 `#[cfg(test)]`）：三份模板 parse 成功；逐 profile 过对应平台 `validate_profile`（trait `platform.rs:574`）；三份模板 `current_config` 为空；grok examples 与 docs 镜像字节一致。
- CLI 面：三平台 `profile init --json` clap 解析测试（definitions.rs 既有模式）；`help.rs` 更新 `platform init` 报错断言 + 新增三平台 `profile --help` 含 `init` 断言 + `ccr init` / `PlatformNotFound` 输出不含退休命令的回归断言。
