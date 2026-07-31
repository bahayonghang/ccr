# 为 claude/codex/grok profile 添加 open 子命令

## Goal

为三个显式平台入口新增 `ccr <claude|codex|grok> profile open`，用用户配置的编辑器（或系统关联程序）打开该平台的 `profiles.toml`。

## Background

工作区当前有一份未提交的 WIP（4 个文件），尝试在隐藏迁移入口 `ccr platform profile open <platform>` 上实现该功能：

- `crates/ccr-cli/src/cli/subcommands/platform.rs:168` — `PlatformProfileAction::Open`
- `crates/ccr-cli/src/commands/platform/profile.rs:683` — `platform_profile_open_command` + `open_with_default_editor`
- `crates/ccr-cli/src/cli/dispatch.rs:367` — 路由
- `crates/ccr-cli/src/commands/platform/mod.rs` — 导出

**该 WIP 将被完整删除**，原因：
1. `ccr platform profile *` 已退役并通过 `legacy_platform_command_error` 返回迁移错误（`migration.rs:15`）；`platform_profile_surface.rs:115` 已有回归测试断言该行为。WIP 的特例路由会破坏该测试。
2. WIP 使用 `cmd /c start "" <path>` 打开文件，路径仍经过 cmd.exe 解释，`&`、`|`、`%` 等元字符会被展开，违反注入约束。
3. WIP 不做 ensure-exists，文件缺失时直接失败；输出复用了语义不匹配的 `PlatformProfileMutationOutput`（`name` 字段被误用为文件名）。

路径解析已统一：`PlatformPaths::new(platform)?.profiles_file` → `~/.ccr/platforms/{platform}/profiles.toml`。三平台一致，**单个 toml 承载该平台全部 profile**。

## Requirements

### R1 三平台对称子命令

- `ccr claude profile open`、`ccr codex profile open`、`ccr grok profile open` 均可用。
- 三者均支持 `--json`，输出结构与同族 profile 命令保持一致。
- **不接受 profile 名参数**：单文件承载全部 profile，传名字无法定位到条目，只会产生误导性语义。

### R2 编辑器解析策略

按顺序回退，首个命中即使用：

1. `$VISUAL` 已设置且非空 → 阻塞运行 `$VISUAL <path>`
2. `$EDITOR` 已设置且非空 → 阻塞运行 `$EDITOR <path>`
3. 平台关联程序（立即返回）：
   - Windows: `cmd /c start "" <path>`
   - macOS: `open <path>`
   - 其他: `xdg-open <path>`

### R3 打开前确保文件存在

- 若 `profiles.toml` 不存在，先创建（含平台对应的 example 模板），再打开。
- 复用现有 init 能力（`platform_profile_init_command`），不要新写一套模板写入逻辑。
- 创建行为需在输出中可见，让用户知道文件是新建的还是已存在的。

### R4 清理 WIP

完整还原 WIP 四文件到基线，确保 `ccr platform profile *` 的退役契约和 `platform_profile_surface.rs` 回归测试不被破坏。

### R5 文档与帮助同步

- `docs/reference/commands/{claude,codex,grok}.md` 的 `profile` 支持面列表新增 `open` 条目（Codex 和 Grok 用的是列表，Claude 用的是表格）。
- `docs/en/reference/commands/{claude,codex,grok}.md` 同步。
- `crates/ccr-cli/src/cli/help_config.rs`：`PLATFORM_AFTER_LONG_HELP`（或对应平台级字符串）提及新入口；`ccr platform profile *` 退役边界文案不变。
- `CHANGELOG.md` 记录。

## Constraints

- 内部实现注释用中文，公开 clap doc comment 和 API 文档用英文（项目约定）。
- 生产路径禁止 `unwrap` / `expect`，统一 `Result` 错误处理。
- 打开文件不经过任何 shell 解释器；Windows 用 `open` crate（`ShellExecuteW`），macOS/Linux 同样走 `open` crate 的对应实现。
- `$EDITOR` 可能带参数（如 `code --wait`），按空白切分为 program + args；带空格的路径需要用 `$VISUAL` 指向无空格 wrapper，此限制写入帮助文本。
- 启动或等待编辑器/关联程序失败时返回 `CcrError::ExternalCommandError`，不输出 `ok: true`。
- `--json` 模式下，JSON 在编辑器进程退出后才打印到 stdout；blocking 编辑器的输出走终端 tty，不污染 stdout。
- 输出不得打印 profile 内的完整 token。
- 通过 `cargo fmt` 与 `just lint-strict`。
- 新增 `open` crate 须在根 `[workspace.dependencies]` 声明；`just version-check`（依赖 drift 检查）须仍通过。

## Acceptance Criteria

- [ ] `ccr claude profile open`、`ccr codex profile open`、`ccr grok profile open` 三条命令均能解析并执行
- [ ] 三条命令均支持 `--json`，输出含 `ok` / `platform` / `profiles_file` / `created` / `registered` / `editor` 字段
- [ ] `$VISUAL` 设置时使用 `$VISUAL`；仅 `$EDITOR` 设置时使用 `$EDITOR`；都未设置时回退到 `open` crate（系统关联程序）
- [ ] `$EDITOR="code --wait"` 这类带参数的值能正确拆分执行
- [ ] `profiles.toml` 不存在时先创建再打开，`created: true` 且 `registered: true`；已存在时幂等，`created: false`
- [ ] 启动或等待编辑器失败时命令以非零退出，不输出 `ok: true`
- [ ] `--json` 模式下，JSON 在编辑器退出后打印，不与编辑器输出交错
- [ ] `ccr platform profile open` 返回退役错误（`platform_profile_surface.rs` 测试仍通过）
- [ ] `crates/ccr-cli/src/cli/definitions.rs` 新增三条 clap 解析测试并通过
- [ ] 编辑器解析逻辑有单元测试覆盖三级回退与带参数拆分
- [ ] `crates/ccr/tests/commands/` 新增集成测试，覆盖首次创建、幂等打开、JSON 字段、平台注册
- [ ] 6 个文档文件（中英各 3）与 `help_config.rs`、`CHANGELOG.md` 已同步
- [ ] `just version-check`、`just fmt-check`、`just lint-strict`、`just test`、`just docs-check` 全部通过
