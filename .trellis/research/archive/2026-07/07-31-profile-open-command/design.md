# 技术设计：profile open 子命令

## 1. 模块边界

新增一个专职模块 `crates/ccr-cli/src/commands/platform/profile_open.rs`，与既有 `profile_init.rs` 平级。它承担：

1. `pub async fn platform_profile_open_command(platform_name: &str, json: bool)` — 唯一的命令实现
2. `resolve_editor()` / `spawn_editor()` — 编辑器解析与启动
3. `pub(super) fn template_for(platform: Platform) -> &'static str` — 三平台模板集中点

三个平台 handler（`commands/{claude,codex,grok}/profile.rs`）各加一个 3 行的 `open_command(json)`，转调共享实现，与它们现有的 `init_command` 完全同构。

**WIP 四文件改动全部删除**，回到基线。这确保 `ccr platform profile *` 的退役契约（`migration.rs:15` + `platform_profile_surface.rs:115` 测试）不被破坏。

## 2. 模板来源

`profile_init.rs` 的签名是 `(platform_name, template, json)`：模板由各 handler 通过 `include_str!` 注入，因为 `include_str!` 需要编译期字面量路径。三个模板常量分别位于：

- `commands/droid/profile.rs:25` → `examples/droid/profiles.example.toml`
- `commands/codex/profile.rs:25` → `examples/codex/profiles.toml`
- `commands/grok/profile.rs:23` → `examples/grok/profiles.toml`

`open` 需要 ensure-exists，也就需要模板。`profile_open.rs` 内建 `pub(super) fn template_for(platform: Platform) -> &'static str`，集中三个 `include_str!`——这是运行时函数而非 const，调用方在函数体内按需获取，不需要常量初始化。

同步删除三个 handler 的 `PROFILE_TEMPLATE` 常量，把 `init_command` 的调用点从 `PROFILE_TEMPLATE` 改为 `crate::commands::platform::profile_open::template_for(Platform::Claude)`（各自平台）。这样全 crate 的 `include_str!` 模板路径只在 `profile_open.rs` 出现一次，满足 spec `profile-init.md:27` 的"不得维护第二份 command-only 模板"约束。

## 3. ensure-exists 复用

`platform_profile_init_command` 已经做了全部需要的事：`ensure_directories()` → 模板解析校验 → `write_guarded_versioned`（secret 权限、原子写）→ `register_platform_if_missing`。

但它直接 `println!` 输出初始化报告，`open` 不该打印一整段 init 文案。方案：从 `profile_init.rs` 抽出一个纯逻辑函数

```rust
pub(super) struct ProfileFileEnsured {
    pub path: PathBuf,
    pub created: bool,
    pub registered: bool,
}

pub(super) fn ensure_profiles_file(
    platform: Platform,
    platform_name: &str,
    template: &str,
) -> Result<ProfileFileEnsured>
```

`platform_profile_init_command` 与 `platform_profile_open_command` 都调它，各自负责输出。这是纯重构，init 的外部行为与 JSON 形状不变。

## 4. 编辑器解析与启动

```rust
enum EditorTarget {
    /// 来自 $VISUAL / $EDITOR，阻塞等待退出
    Configured { program: String, args: Vec<String>, source: &'static str },
    /// 平台关联程序，非阻塞
    SystemAssociation,
}

fn resolve_editor() -> EditorTarget
```

规则：依次读 `VISUAL`、`EDITOR`；`trim()` 后为空视作未设置。命中后按空白切分成 program + args，支持 `EDITOR="code --wait"`。

不做 shell 引号解析（`EDITOR="'/path/with space/ed' -w"`）：POSIX 惯例里 `$EDITOR` 按空白切分，引入引号解析会带来 Windows 路径歧义。**带空格路径的用户设 `VISUAL` 指向无空格的 wrapper 脚本，此限制写进帮助文本。**

启动：

```rust
fn spawn_editor(target: &EditorTarget, path: &Path) -> Result<()>
```

- `Configured` → `Command::new(program).args(args).arg(path).status()?`，阻塞等待。非零退出返回 `CcrError::ExternalCommandError("编辑器退出码 N")`。
- `SystemAssociation` → `open::that(path)?`（`open` crate v5.4.0）。该 crate 在 Windows 调用 `ShellExecuteW`、macOS 调用 `open`、Linux 调用 `xdg-open`，避免 `cmd /c` 的元字符注入风险。启动失败或系统调用返回错误时自动传播为 `Err`，映射到 `CcrError::ExternalCommandError`。

**`open` crate 新增依赖治理**：在根 `Cargo.toml` 的 `[workspace.dependencies]` 加 `open = "5.4.0"`，在 `ccr-cli/Cargo.toml` 的 `[dependencies]` 加 `open = { workspace = true }`。该 crate 核心无厚重传递依赖（Windows 零额外依赖、macOS 零额外依赖、Linux 仅 `which` 用于 PATH 探测），专注文件关联程序启动，符合治理约束。`just version-check` 须通过依赖 drift 检查。

**`--json` 输出时序契约**：
- `Configured` 分支：ensure-exists → 启动编辑器（阻塞，输出走 terminal tty）→ 编辑器退出 → 打印 JSON 到 stdout。终端编辑器写 `/dev/tty` 不污染 stdout，JSON 是最后的完整输出。
- `SystemAssociation` 分支：ensure-exists → 调用 `open::that`（立即返回）→ 打印 JSON 到 stdout。

所有分支：启动或等待失败时返回 `ExternalCommandError`，不输出 JSON（命令以非零退出）。

## 5. 输出契约

```rust
#[derive(Serialize)]
struct ProfileOpenOutput<'a> {
    ok: bool,
    platform: &'a str,
    profiles_file: String,
    created: bool,
    registered: bool,
    editor: &'a str,   // "$VISUAL" | "$EDITOR" | "system"
}
```

与 `ProfileInitOutput` 同族（`ok` / `platform` / `profiles_file` / `created` / `registered`），新增 `editor` 让脚本能判断命令是否阻塞过。`registered` 反映 `ensure_profiles_file` 是否注册了平台。

人类可读输出：
- created 时：`已创建 profiles 模板: <path>`
- registered 时（首次初始化）：`已注册平台: <platform>`
- 随后：`正在用 $EDITOR 打开: <path>`（或 `正在用系统关联程序打开`）

## 6. CLI 定义

三处各加一个变体，紧跟 `Init` 之后（语义相邻）：

```rust
/// Open the platform profiles.toml in your editor
Open {
    #[arg(long)]
    json: bool,
},
```

- `cli/subcommands/claude.rs` → `ClaudeProfileAction`
- `cli/subcommands/codex.rs` → `CodexProfileAction`
- `cli/subcommands/grok.rs` → `GrokProfileAction`

三处 doc comment 用英文（clap help 属公开 API 面）。

`dispatch.rs` 三段 match 各加一个 arm，模式与相邻的 `Init` arm 一致。

## 7. 兼容性

- 纯新增子命令，无既有命令的行为变更。
- WIP 四文件改动完整删除，`ccr platform profile *` 维持退役契约。`platform_profile_surface.rs:115` 测试不回退。
- `profile_init.rs` 重构不改变 `ccr <platform> profile init` 的 CLI 与 JSON 契约。
- 新增 `open` crate 依赖须通过 `just version-check` 的依赖 drift 检查。

## 8. 权衡记录

| 决策 | 取舍 |
|---|---|
| `open` 不收 profile 名 | 单文件承载全部 profile，参数无法定位条目；收下只会制造"能跳转"的错误预期 |
| `$VISUAL`/`$EDITOR` 优先于系统关联 | CLI 用户的期待是终端编辑器；GUI 用户通常不设这两个变量，回退自然 |
| 按空白切分而非 shell 解析 | 符合 POSIX 惯例，避免 Windows 路径的引号歧义；带空格路径需用 wrapper 脚本 |
| 抽 `ensure_profiles_file` 而非在 open 里重写 | 原子写、secret 权限、模板校验、平台注册四项逻辑不该有第二份 |
| 新建 `profile_open.rs` 而非塞进 `profile.rs` | 后者已 720 行且职责是字段增删改 |
| 使用 `open` crate 而非 `cmd /c` | Windows `ShellExecuteW` 绕过 cmd 元字符解释，避免注入；macOS/Linux 同样安全 |
| 所有分支检查启动/退出失败 | 非零退出或启动失败时应报错，不能静默成功误导用户 |
| 删除 WIP 而非保留隐藏入口 | 退役契约明确要求 `platform profile *` 返回 migration error；WIP 破坏该契约和回归测试 |

## 9. 副作用与回滚

**副作用**：
- `ensure_profiles_file` 在 profiles.toml 不存在时创建该文件（`~/.ccr/platforms/{platform}/profiles.toml`），权限 Unix `0o600`（或 Windows 等价），内容为验证过的模板。
- 首次初始化时向 `~/.ccr/config.toml` 的 platform registry 写入平台条目（带锁、备份后原子写）。
- 启动系统关联程序或终端编辑器，进程退出后无 CCR 持久化状态变更（用户可能编辑了 profiles.toml 内容，但那是用户操作，不是命令副作用）。

**回滚**：
- 代码回滚：`git revert` 移除三个新增 `Open` 变体、dispatch arm、`profile_open.rs` 模块、`open` crate 依赖、文档改动。
- **磁盘文件不随代码回滚自动删除**：已创建的 `profiles.toml` 和 registry 条目会保留。用户若要清理，需手动删除 `~/.ccr/platforms/{platform}/profiles.toml` 与 `~/.ccr/config.toml` 中的对应平台条目。
- ensure-exists 只在文件缺失时写入，不覆盖既有内容（`write_guarded_versioned` + `exists()` 前置判断），因此不存在用户数据损坏路径。
