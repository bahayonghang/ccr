# Implement: profile init 子命令与平台示例配置补全

前置：`design.md` v2 定稿；任务 `task.py start` 后才动手。
（v2 修订依据：`research/codex-review-verification.md` 的 6 项 P1 采纳结论）

## 步骤清单

### Step 1: examples / docs 示例文件（安全未激活形态）

- [x] `examples/claude/profiles.example.toml`：`current_config = "anthropic"` → `""`（其余不动）。
- [x] `examples/codex/profiles.toml`：`current_config = "default"` → `""`（其余不动）。
- [x] 新建 `examples/grok/profiles.toml`（design.md D6：official=session、relay=env_key+example.com、inline 仅注释；`current_config = ""`）。
- [x] `docs/examples/grok-profiles.toml` 内容同步为与 `examples/grok/profiles.toml` 字节一致（raw 链接不动）。
- [x] 更新 `examples/README.md`：目录树补 grok/droid；「快速使用」段替换 `ccr platform init <p>` → `ccr <p> profile init`；补 grok 平台条目。

验证：`rg -n "platform init" examples/` 无残留。

### Step 2: 持久化与注册 helper

- [x] `crates/ccr-config/src/platforms/base.rs`：新增 `pub fn register_platform_if_missing(platform_name, description) -> Result<bool>`（`platform_registry` 命名锁内 load → 已注册直接 false（不写不备份）→ 未注册则注册+备份+save 返回 true；复用 base.rs:23-56 锁常量与 `save_platform_registry` 骨架）。附单测：新注册 / 幂等二次 / 并发。
- [x] 新建 `crates/ccr-cli/src/commands/platform/profile_init.rs`：`platform_profile_init_command(platform_name, template, json)`，按 design.md D1（ensure_directories → 模板自校验 → `write_guarded_versioned` 空 token + `secret: true` + `BackupPolicy::None`，`Conflict`/已存在 → 幂等路径 → `register_platform_if_missing` → 人读/JSON 输出 `{ok, platform, profiles_file, created, registered}`）。
- [x] `crates/ccr-cli/src/commands/platform/mod.rs` 导出；三个平台命令模块各加 `init_command`（各自 `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/..."))` 模板常量）：
  - `crates/ccr-cli/src/commands/claude/profile.rs`
  - `crates/ccr-cli/src/commands/codex/profile.rs`
  - `crates/ccr-cli/src/commands/grok/profile.rs`

### Step 3: CLI 枚举 + dispatch + 迁移文案（全部可达提示）

- [x] `subcommands/{claude,codex,grok}.rs`：各加 `Init { #[arg(long)] json: bool }` variant（Help 之后 Current 之前）。
- [x] `cli/dispatch.rs`：`dispatch_claude` / `dispatch_codex` / `dispatch_grok` 三处 profile match 各加 Init 臂。
- [x] `commands/migration.rs`：新增 `legacy_platform_init_error()`（含三条 `ccr <p> profile init`）；`dispatch_platform` 的 `PlatformAction::Init` 分支改调它。
- [x] `crates/ccr-core/src/core/error.rs:316`：`PlatformNotFound` 建议行改 `ccr claude/codex/grok profile init`（仅文案）。
- [x] `crates/ccr-cli/src/commands/lifecycle/init.rs:31`、`:156`：提示改为新命令。
- [x] `cli/help_config.rs`：`PLATFORM_AFTER_LONG_HELP` 补一行 init 指引。

### Step 4: 测试

- [x] `profile_init.rs` 单测：三份模板 parse 成功；逐 profile 过平台 `validate_profile`（platform.rs:574 trait）；三份模板 `current_config` 为空；`examples/grok/profiles.toml` == `docs/examples/grok-profiles.toml`（字节一致）。
- [x] `crates/ccr/tests/commands/grok_profile.rs`（沿用 `GrokProfileFixture` 隔离）：
  - init 首次：文件创建、`list` 两条模板项、`current` 未激活、注册表 `current_profile == None`、`GROK_HOME/config.toml` 字节不变；
  - init 二次：文件字节不变、backups 无新增、输出含已存在语义；
  - 并发双 init：两个子进程同时执行，均退出 0，最终文件 == 模板；
  - `--json`：`created` 首次 true / 二次 false，`ok/platform/profiles_file/registered` 齐全；
  - cfg(unix)：产物权限 0o600。
- [x] `claude_profile.rs` / `codex_profile.rs`：各加 init 首次 + 幂等 + 未激活（含 runtime 文件不变）用例。
- [x] `cli/definitions.rs`：三平台 `profile init --json` clap 解析测试。
- [x] `crates/ccr/tests/commands/help.rs`：`platform init` 报错断言改新文案（含 `ccr grok profile init`）；三平台 `profile --help` 含 `init`；`ccr init` 输出与 `PlatformNotFound` 消息不含 `ccr platform init` 的回归断言。

### Step 5: docs

- [x] `docs/{,en/}reference/commands/platform.md`：init 段改为新命令。
- [x] `rg -n "platform init" docs/` 逐处判断：教程型用法（multi-platform-setup、troubleshooting、examples/index、gemini/droid 平台页、commands/init.md、commands/sync.md）改为新命令或加迁移提示；`migration.md` 属历史记述，仅补一句新命令指向。gemini/droid 页面注意：这两个平台本任务不加 init，文案改为「目录会在 profile create 时自动创建」或指向手工复制示例，**不得虚构 `ccr gemini/droid profile init`**。
- [x] grok 相关 docs（`docs/{,en/}reference/commands/grok.md`）核对 raw 链接指向的示例内容更新后语义仍成立。

### Step 6: 验证与收尾

- [x] `just fmt-check`；`just version-check` 的版本一致性通过，随后被任务外既有 `ccr-ui/README.md` 文档漂移（仍为 `version-7.0.0`）阻断，未改该文件。
- [x] `just lint-strict && just test`（直测时带 `-- --test-threads=1`）
- [x] docs 改动：`cd docs && npm run build`
- [x] 手工冒烟（临时 `CCR_ROOT` + `GROK_HOME`）：`ccr grok profile init` → `list` → `current` → 再次 `init`；`ccr platform init grok` 与 `ccr init` 文案含新命令、无退休命令。
- [x] Trellis 3.3 spec update：init 命令契约与「安全未激活模板」约定已沉淀进 `.trellis/spec/ccr-cli/backend/profile-init.md`；3.4 commit 在最终门禁后执行。

## 回滚点

- 每个 Step 独立可编译可测试；Step 3 完成前 CLI 面无任何行为变化（新函数未接线）。
- Step 1 的 examples 头部修改独立成立（手工复制流程同样受益），可单独保留或回滚。
- 出问题 revert 对应 commit 即可，无持久化状态迁移。

## Review gates

- Step 2 完成后：秘钥/持久化红线复查（guarded versioned write、secret 权限、注册表锁、无明文密钥打印、无覆盖路径）。
- Step 5 完成后：`rg -n "platform init"` 全仓过一遍，确认残留均为「历史记述/迁移文档」性质；`rg -n "current_config = \"" examples/ docs/examples/` 确认模板均为空激活态。
