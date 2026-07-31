# 执行计划：profile open 子命令

## 前置状态

工作区有 4 个文件的未提交 WIP。**Step 0 先完整删除这些改动**，回到基线后再从头实现。

```bash
git diff --stat   # 应显示 4 files changed
```

---

## Step 0 — 删除 WIP，回到基线

```bash
git checkout -- \
  crates/ccr-cli/src/cli/dispatch.rs \
  crates/ccr-cli/src/cli/subcommands/platform.rs \
  crates/ccr-cli/src/commands/platform/mod.rs \
  crates/ccr-cli/src/commands/platform/profile.rs
```

**验证**：`cargo check -p ccr-cli` 编译通过，`git diff --stat` 输出为空，`cargo test -p ccr -- platform_profile_surface -- --test-threads=1` 中的 `legacy_platform_profile_gemini_reports_migration_instead_of_mutating` 仍通过。

---

## Step 1 — 新增 `open` crate 依赖

**文件 1**：根 `Cargo.toml` 的 `[workspace.dependencies]` 加：

```toml
open = "5.4.0"
```

**文件 2**：`crates/ccr-cli/Cargo.toml` 的 `[dependencies]` 加：

```toml
open = { workspace = true }
```

**验证**：`cargo metadata --no-deps --format-version 1 -q` 无错误；`just version-check` 通过依赖 drift 检查。

---

## Step 2 — 抽出 `ensure_profiles_file`

**文件**：`crates/ccr-cli/src/commands/platform/profile_init.rs`

从 `platform_profile_init_command` 中抽出纯逻辑部分为：

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

`platform_profile_init_command` 改为调它，再做原有输出。保留 `Platform::from_str` + `auth_profile_supported` 校验在 command 层。init 的外部行为与 JSON 字段（`ok` / `platform` / `profiles_file` / `created` / `registered`）不变。

**验证**：`cargo test -p ccr -- grok_profile -- --test-threads=1`，`embedded_profile_templates_are_inactive_and_valid` 与 init 相关测试仍通过。

---

## Step 3 — 新建 `profile_open.rs`

**文件**：`crates/ccr-cli/src/commands/platform/profile_open.rs`（新建）

实现结构按 `design.md` §2–§5：

**3a. 模板集中**

```rust
pub(super) fn template_for(platform: Platform) -> &'static str {
    match platform {
        Platform::Claude => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/droid/profiles.example.toml"
        )),
        Platform::Codex => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/codex/profiles.toml"
        )),
        Platform::Grok => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/grok/profiles.toml"
        )),
        _ => unreachable!("template_for 只支持三个 auth-profile 平台"),
    }
}
```

**3b. 编辑器解析**

```rust
enum EditorTarget {
    Configured { program: String, args: Vec<String>, source: &'static str },
    SystemAssociation,
}

fn resolve_editor() -> EditorTarget {
    resolve_editor_from(
        std::env::var("VISUAL").ok().as_deref(),
        std::env::var("EDITOR").ok().as_deref(),
    )
}

fn resolve_editor_from(visual: Option<&str>, editor: Option<&str>) -> EditorTarget {
    for (value, source) in [(visual, "$VISUAL"), (editor, "$EDITOR")] {
        if let Some(s) = value {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                let mut parts = trimmed.split_whitespace();
                let program = parts.next().unwrap().to_string();
                let args = parts.map(str::to_string).collect();
                return EditorTarget::Configured { program, args, source };
            }
        }
    }
    EditorTarget::SystemAssociation
}
```

**3c. 启动**

```rust
fn spawn_editor(target: &EditorTarget, path: &Path) -> Result<()> {
    match target {
        EditorTarget::Configured { program, args, source } => {
            let status = std::process::Command::new(program)
                .args(args)
                .arg(path)
                .status()
                .map_err(|e| CcrError::ExternalCommandError(
                    format!("{source} 无法启动: {e}")
                ))?;
            if !status.success() {
                return Err(CcrError::ExternalCommandError(
                    format!("{source} 退出码: {}", status.code().unwrap_or(-1))
                ));
            }
            Ok(())
        }
        EditorTarget::SystemAssociation => {
            open::that(path).map_err(|e| CcrError::ExternalCommandError(
                format!("系统关联程序启动失败: {e}")
            ))
        }
    }
}
```

**3d. 输出结构与命令**

```rust
#[derive(Serialize)]
struct ProfileOpenOutput<'a> {
    ok: bool,
    platform: &'a str,
    profiles_file: String,
    created: bool,
    registered: bool,
    editor: &'a str,
}

pub async fn platform_profile_open_command(platform_name: &str, json: bool) -> Result<()> {
    let platform = parse_platform(platform_name)?;
    let template = template_for(platform);
    let ensured = ensure_profiles_file(platform, platform_name, template)?;
    let editor = resolve_editor();

    if !json {
        if ensured.created {
            ColorOutput::success(&format!("已创建 profiles 模板: {}", ensured.path.display()));
        }
        if ensured.registered {
            ColorOutput::success(&format!("已注册平台: {platform_name}"));
        }
        let editor_label = match &editor {
            EditorTarget::Configured { source, .. } => *source,
            EditorTarget::SystemAssociation => "系统关联程序",
        };
        ColorOutput::info(&format!("正在用 {editor_label} 打开: {}", ensured.path.display()));
    }

    spawn_editor(&editor, &ensured.path)?;

    if json {
        let output = ProfileOpenOutput {
            ok: true,
            platform: platform_name,
            profiles_file: ensured.path.display().to_string(),
            created: ensured.created,
            registered: ensured.registered,
            editor: match &editor {
                EditorTarget::Configured { source, .. } => source,
                EditorTarget::SystemAssociation => "system",
            },
        };
        println!("{}", serde_json::to_string(&output).map_err(CcrError::JsonError)?);
    }

    Ok(())
}
```

注意：JSON 在 `spawn_editor` 之后打印——blocking 编辑器的 tty 输出先完成，JSON 最后输出，不污染。`SystemAssociation` 分支立即返回，JSON 紧随其后，同样干净。

**单元测试**（同文件内 `#[cfg(test)]`）：
- `resolve_editor_from(Some("nano"), None)` → `Configured { program="nano", source="$VISUAL" }`
- `resolve_editor_from(None, Some("code --wait"))` → `Configured { program="code", args=["--wait"] }`
- `resolve_editor_from(Some("  "), Some("vim"))` → `Configured { source="$EDITOR" }`（空白跳过）
- `resolve_editor_from(None, None)` → `SystemAssociation`
- `resolve_editor_from(Some(""), Some(""))` → `SystemAssociation`

---

## Step 4 — 更新 `platform/mod.rs` 与三个 handler

**`crates/ccr-cli/src/commands/platform/mod.rs`**：

加 `mod profile_open;` 和 `pub use profile_open::platform_profile_open_command;`。

**三个 handler**（`commands/{droid,codex,grok}/profile.rs`）：

1. 删除各自的 `const PROFILE_TEMPLATE`。
2. 把 `init_command` 中 `PROFILE_TEMPLATE` 替换为 `crate::commands::platform::profile_open::template_for(Platform::Claude)`（Codex/Grok 各自对应平台）。
3. 新增 `open_command`：

```rust
pub async fn open_command(json: bool) -> Result<()> {
    crate::commands::platform::platform_profile_open_command("claude", json).await
}
```

---

## Step 5 — CLI 定义 + dispatch

**定义**（各在 `Init` 之后插入）：

- `cli/subcommands/claude.rs:83` `ClaudeProfileAction`
- `cli/subcommands/codex.rs:150` `CodexProfileAction`
- `cli/subcommands/grok.rs:25` `GrokProfileAction`

```rust
/// Open the platform profiles.toml in your editor.
/// Creates the file from the example template if it does not exist.
Open {
    #[arg(long)]
    json: bool,
},
```

**dispatch**（`crates/ccr-cli/src/cli/dispatch.rs`，三段 profile match）：

各在 `Init` arm 之后插入（注意 Claude/Grok 是 `Box`ed action，Codex 不是）：

```rust
ClaudeProfileAction::Open { json } => {
    crate::commands::claude::profile::open_command(*json).await
}
```

---

## Step 6 — 测试

**clap 解析测试** — `crates/ccr-cli/src/cli/definitions.rs`，仿 `platform_profile_init_json_flags_parse`（:588）：

```rust
#[test]
fn profile_open_flags_parse_for_all_platforms() {
    // claude / codex / grok × (无 flag, --json)
}
```

**集成测试** — `crates/ccr/tests/commands/profile_open.rs`（新建），使用与 `grok_profile.rs` 相同的 `CCR_ROOT` fixture 模式：

- `profile_open_creates_file_when_missing`：三平台各自调 `open --json`，断言 `created: true`、`registered: true`、`profiles_file` 路径存在、`editor: "system"`
- `profile_open_idempotent_when_file_exists`：预写任意内容后再调，断言 `created: false`、文件内容不变
- `profile_open_json_reports_all_fields`：断言 `ok` / `platform` / `profiles_file` / `created` / `registered` / `editor` 六个字段齐全

注意：集成测试不测实际编辑器启动（CI 环境无编辑器），通过 `VISUAL=` `EDITOR=` 清空确保走 `SystemAssociation` 分支；`open::that` 在 CI 可能返回错误，此时测试仍只验证 ensure-exists 的 JSON 输出（在 spawn_editor 之前）。若 `open::that` 失败，命令整体非零退出，测试断言 `stderr` 含 `ExternalCommandError` 字样。

**验证**：

```bash
cargo test -p ccr-cli -- --test-threads=1
cargo test -p ccr -- --test-threads=1
```

---

## Step 7 — 文档与帮助

**中文文档**：

- `docs/reference/commands/claude.md` — Profile Runtime **表格**新增一行：
  ```
  | `ccr claude profile open` | 用 $VISUAL/$EDITOR 或系统关联程序打开 profiles.toml；文件不存在时先从模板创建 |
  ```
- `docs/reference/commands/codex.md` — `profile` 支持面**列表**新增 `open` 条目（当前第 30 行起是 `-` 列表，不是表格）。
- `docs/reference/commands/grok.md` — 同 codex 格式，先读确认结构再改。

**英文文档**：`docs/en/reference/commands/{claude,codex,grok}.md` — 格式与中文对应版本一致，先读再改。

**帮助文本** — `crates/ccr-cli/src/cli/help_config.rs`：

`help_config.rs` 无 profile 专属 long-help 段落；帮助来自 clap 命令树本身（`help.rs:18` 直接调 `--help`）。无需改动 `help_config.rs`——`Open` 变体的 doc comment 就是帮助文本。

`PLATFORM_AFTER_LONG_HELP`（:121）的边界段目前写着"旧的平台路由入口已退休"，与 open 功能无交集，保持不变。

**CHANGELOG.md** — 顶部 `Unreleased`（若无则新建）加 `feat` 条目。

## 验证顺序

```bash
just version-check           # 含依赖 drift 检查
just fmt-check
just lint-strict
cargo test -p ccr-cli -- --test-threads=1
cargo test -p ccr -- --test-threads=1
just test
just docs-check              # VitePress 构建，不用 cd docs && npm run build
just ci                      # 最终全链路
```

## 手工验收

在独立终端（不影响真实配置）：

```bash
# 用 CCR_ROOT 重定向到临时目录
export CCR_ROOT=$(mktemp -d)

# 首次打开：should create file
VISUAL= EDITOR= cargo run -p ccr -- claude profile open --json

# 幂等
VISUAL= EDITOR= cargo run -p ccr -- claude profile open --json

# 编辑器拆分
EDITOR="echo test" cargo run -p ccr -- codex profile open

# platform 退役契约不回退
cargo run -p ccr -- platform profile open claude  # 应返回退役错误
```

## 回滚点

- Step 0 可独立回滚：`git stash` WIP（已删），无代码变更。
- Step 1 可独立回滚：删除 `open` crate 依赖。
- Step 2 纯重构，init 行为不变，可单独提交。
- Steps 3–5 是功能整体，需一起提交。
- Step 6–7 文档与测试，独立可回滚。

任一步 `just lint-strict` 失败且两次修不好：停下记录到 `research/`，不要继续堆叠。
