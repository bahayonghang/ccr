# `ccr project init` 技术设计

## 1. Architecture And Ownership

该功能只新增一个 CLI 编排入口，不引入新 crate 或跨层服务：

```text
Cli / Clap
  Commands::Project { action: ProjectAction }
    ProjectAction::Init
      CommandDispatcher
        project_init_command(current_dir, auto_yes)
          ensure_git_repository
          run_trellis_init
          ensure_project_gitignore
```

- `cli/subcommands/project.rs` 定义嵌套命令枚举。
- `cli/definitions.rs` 将 `Project` 加入顶层命令。
- `cli/dispatch.rs` 只传递全局 `auto_yes` 并调用 handler。
- `commands/project/init.rs` 拥有阶段编排、外部进程和 `.gitignore` 合并。
- 纯文本合并和 Git 状态解析保持为模块私有函数，不建立通用“项目初始化框架”。

## 2. Command Contract

建议 Clap 结构：

```rust
pub enum ProjectAction {
    Init,
}

Commands::Project {
    #[command(subcommand)]
    action: ProjectAction,
}
```

裸 `ccr project` 由 Clap 产生缺少子命令错误；`--help` 和 `ccr help project init` 继续复用现有增强命令树。`help_config.rs` 为 `project` 增加任务导向说明，并在根帮助常用任务中增加项目初始化入口。

全局 `auto_yes` 只影响 Trellis 调用：普通模式参数为 `init`，自动模式参数为 `init --yes`。Git 不需要确认，`.gitignore` 合并也是确定性幂等操作。

## 3. Git State Detection

### Flow

1. 以当前目录为 `current_dir` 运行 `git rev-parse --show-toplevel` 并捕获输出。
2. 子进程无法启动时返回 `CcrError::ExternalCommandError`，明确说明未找到或无法执行 Git。
3. 成功且 stdout 为非空路径时，认定当前目录已经位于 Git 工作树：
   - 规范化路径后与当前目录相同：输出当前项目已经初始化。
   - 不同：输出父级仓库根并说明跳过嵌套初始化。
4. `rev-parse` 非零时运行继承 stdout/stderr 的 `git init`。
5. `git init` 无法启动或非零退出时返回外部命令错误。

不依赖 `<cwd>/.git` 是目录，因为 worktree 场景中 `.git` 可以是文件。Git 输出路径只用于信息和比较，不改变 Trellis/`.gitignore` 的目标目录。

### Error Boundary

`rev-parse` 非零被视为“不在工作树”，最终仍由 `git init` 决定是否可初始化。该策略不解析可能本地化的 stderr 文本；异常 Git 环境若导致 `git init` 失败，会以 Git 阶段错误终止。

## 4. Trellis Process Contract

调用形态：

```rust
let mut command = Command::new("trellis");
command.arg("init").current_dir(root);
if auto_yes {
    command.arg("--yes");
}
command
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit());
```

CCR 不读取或转换 Trellis 的交互结果。进程完成后执行两个判定：

- 退出状态必须成功。
- 当前目录必须存在 `.trellis/workflow.md` 和 `.trellis/scripts/task.py`，作为最低可用工作流后置条件。

后置条件用于防止 Trellis 在“未选择 Agent”等早退路径返回成功状态却没有完成初始化。对于已有有效 Trellis 的重初始化取消，最低结构仍存在，因此命令可继续。

外部进程是前台 CLI 工作，直接同步等待并保持终端继承；不增加后台任务、超时或输出捕获，避免破坏交互式 TTY 行为。

## 5. `.gitignore` Merge Contract

固定规则：

```text
.agents/
.claude/
.codex/
```

算法：

1. `.gitignore` 不存在时以空 UTF-8 文本开始；其他读取错误直接返回。
2. 逐行去除行尾 `\r` 并忽略首尾空白后，与三条固定规则做精确比较。
3. 没有缺失规则时返回 `Unchanged`，不写文件、不改变 mtime。
4. 有缺失规则时：
   - 现有文本含 CRLF 则沿用 CRLF，否则使用 LF。
   - 非空文件没有行结束符时先补一个。
   - 依需求顺序只追加缺失规则，并以行结束符收尾。
5. 使用 `ccr_core::core::AtomicWriter` 原子替换文件，复用其 Windows replace/retry 语义。

不把 `/.agents/`、`.agents/**`、父级规则或否定规则推断为等价。验收目标是当前项目 `.gitignore` 中明确存在三条固定、可读、可重复验证的规则。

## 6. Result And Recovery Model

```text
Git failed       -> stop; Trellis not invoked; gitignore untouched
Trellis failed   -> keep Git result; gitignore untouched; retry is safe
gitignore failed -> keep Git + Trellis results; retry only repairs missing rules
all ready        -> print one final success summary
```

不实现回滚。Git 和 Trellis 都是外部工具且可能写入多个用户文件，自动删除无法可靠区分新文件与既有文件。幂等重试是恢复机制。

## 7. Testing Design

### Unit Tests In `ccr-cli`

- `.gitignore` 空文件、既有内容、无末尾换行、部分规则、全部规则、CRLF 和重复调用。
- 最低 Trellis 后置条件检查。
- 路径比较对当前根与父级根的分类。

### Command Integration Tests In `ccr`

在临时目录中为子进程构造隔离 PATH，并按平台生成 fake `git`/`trellis`（Windows `.cmd`，Unix 可执行脚本）。fake 工具记录 argv、cwd 和调用顺序，并按场景创建 `.git` 或最低 `.trellis` 结构。

覆盖：

- 新 Git 仓库完整成功。
- 当前目录仓库与父级仓库均跳过 `git init`。
- `--yes` 只为 Trellis 增加 `--yes`。
- Git 缺失/失败、Trellis 缺失/失败、Trellis 零退出但后置条件缺失。
- `.gitignore` 内容保留与重复执行幂等。

子进程级 PATH 通过 `Command::env` 设置，不修改测试进程全局 PATH；需要全局环境改动时才使用现有 `TestHostEnv`。

## 8. Documentation And Compatibility

- 新增 `docs/reference/commands/project-init.md` 与英文对应页。
- 更新两份命令索引，将 `project init` 放在配置/项目引导区域，并明确 `ccr init` 是用户级 CCR 配置。
- 不修改 README 快速开始的现有 `ccr init`，避免把两种初始化混为一谈。
- 不新增配置文件、环境变量、公开 Rust API 或生产依赖。

## 9. Risks And Mitigations

- **Trellis CLI 行为漂移**：不复制平台清单；只依赖稳定的 `init`、`--yes` 和最低 `.trellis` 结构。
- **Trellis 成功码假阳性**：增加最低结构后置校验。
- **意外嵌套 Git**：使用 `rev-parse --show-toplevel` 识别父仓库并跳过。
- **用户 `.gitignore` 损坏**：无变化不写；有变化使用现有 `AtomicWriter`。
- **跨平台测试差异**：fake 工具脚本按 `cfg(windows)` / `cfg(unix)` 生成，路径和参数记录使用原生 `Path`/`OsString`。

## 10. Rollback

实现尚未发布时可按本任务文件清单删除新命令、测试和文档，并恢复命令树/索引改动。运行时不提供自动回滚；用户可依据 Git 状态手动处理 Trellis 生成文件。
