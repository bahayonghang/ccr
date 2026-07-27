# `ccr project init` 实施计划

## Pre-Implementation Gate

- [x] 用户已批准本 PRD、设计和实施计划。
- [x] 运行 `python ./.trellis/scripts/task.py start 07-23-project-init-command`，确认状态变为 `in_progress`。
- [x] 实施代理加载 `implement.jsonl` 中的 CLI 规范和研究文件。
- [x] 保持当前 `dev` 分支既有提交和任何新增用户改动不受影响。

## Implementation Steps

### 1. Add The Command Tree

- [x] 新增 `crates/ccr-cli/src/cli/subcommands/project.rs`，定义 `ProjectAction::Init`。
- [x] 在 `cli/subcommands/mod.rs` 注册并导出 `ProjectAction`。
- [x] 在 `cli/definitions.rs` 新增 `Commands::Project`，保留现有 `Commands::Init` 不变。
- [x] 在 `cli/dispatch.rs` 分发 `ProjectAction::Init`，把全局 `auto_yes` 传给 handler。
- [x] 在 `cli/help_config.rs` 增强 project 帮助和根帮助可发现性。

### 2. Implement Project Initialization

- [x] 新增 `commands/project/mod.rs` 与 `commands/project/init.rs`，并在 `commands/mod.rs` 注册模块。
- [x] 获取并保留当前目录作为所有阶段的目标根。
- [x] 实现 Git 工作树检测、父仓库提示和缺失仓库的 `git init`。
- [x] 实现继承终端的 `trellis init`；`auto_yes` 时追加 `--yes`。
- [x] 校验 Trellis 退出状态和最低 `.trellis` 文件后置条件。
- [x] 实现 `.gitignore` 纯合并函数、无变化短路和 `AtomicWriter` 写入。
- [x] 为阶段开始、跳过、成功及最终摘要添加符合现有 `ColorOutput` 风格的输出。
- [x] 错误使用现有 `CcrError::{ExternalCommandError, FileIoError, ValidationError}` 等合适变体，不新增共享错误枚举。

### 3. Add Focused Tests

- [x] 在 `init.rs` 单元测试中覆盖 `.gitignore` 合并、CRLF、缺失末尾换行、部分/全部规则和幂等。
- [x] 新增 `crates/ccr/tests/commands/project_init.rs`，构造跨平台 fake Git/Trellis 并验证 argv、cwd、调用顺序和文件后置条件。
- [x] 覆盖新仓库、当前仓库、父级仓库、`--yes`、Git 失败、Trellis 失败、假成功和重复运行。
- [x] 在 `crates/ccr/tests/commands.rs` 注册测试模块。
- [x] 扩展 `commands/help.rs`，验证 project 帮助以及旧 `ccr init` 兼容入口。

### 4. Document The Command

- [x] 新增中文 `docs/reference/commands/project-init.md`。
- [x] 新增英文 `docs/en/reference/commands/project-init.md`。
- [x] 更新两份 `commands/index.md`，区分用户级 `ccr init` 和项目级 `ccr project init`。
- [x] 文档覆盖依赖、交互/`--yes`、父仓库、固定 ignore 规则、失败阶段和幂等重试。

## Validation Ladder

按顺序执行，失败时只修复本任务范围内的根因并从最窄门禁重跑：

1. `cargo fmt --all -- --check`
2. `cargo test -p ccr-cli project -- --test-threads=1`
3. `cargo test -p ccr --test commands project_init -- --test-threads=1`
4. `cargo test -p ccr --test commands help -- --test-threads=1`
5. `cargo clippy -p ccr-cli --all-targets --all-features -- -D warnings`
6. `just docs-check`
7. `just version-check`
8. `just fmt-check`
9. `just lint-strict`
10. `just test`
11. `just ci`（代码与双语文档跨表面交付的最终门禁）

若 `just ci` 出现与本任务无关的既有或外部失败，记录精确失败步骤和证据，不将其误报为通过，也不扩展修复范围。

## Review Gates

- [x] `git diff --check` 无空白错误。
- [x] 最终 diff 只包含 task 规划明确列出的 CLI、测试、帮助和文档文件。
- [x] 搜索确认 CCR 中没有复制 Trellis 平台 flags/平台列表。
- [x] 搜索确认现有 `Commands::Init`、`ccr init` 文档语义未被替换。
- [ ] 手工在临时新目录运行一次真实 `ccr project init`，选择 Claude Code + Codex，确认 TTY 交互、Git、Trellis 和三条 ignore 规则；该检查会创建临时文件，执行前使用隔离临时目录。
- [x] 手工在父级临时 Git 仓库的子目录运行一次，确认没有生成嵌套 `.git`。

## Rollback Points

- 命令树未完成时：恢复 `definitions.rs` / `dispatch.rs` / `help_config.rs` 并删除新增 project 模块。
- handler 或测试失败时：保留规划文件，回退本任务新增产品文件后按设计重新实现；不触碰用户既有变更。
- 文档门禁失败时：只修正新增双语页面和索引，不改动无关文档。

## Completion

- [ ] 所有 AC1-AC10 都有自动或明确的手工证据。
- [x] 运行 `trellis-check` 做全范围规范、测试和一致性复核。
- [x] 评估是否有值得写回 `.trellis/spec/` 的新约定。
- [x] 按仓库流程给出本任务的中文 emoji 原子提交计划，等待用户确认后再提交；不 push。
- [ ] 完成 Trellis archive 和开发日志收尾。
