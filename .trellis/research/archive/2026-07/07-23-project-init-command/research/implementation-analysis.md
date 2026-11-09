# `ccr project init` 实施研究

## 1. 仓库内架构落点

- `crates/ccr-cli/src/cli/definitions.rs:83` 是顶层 `Commands` 枚举；现有 `Init` 在该文件约 202 行，语义是初始化 CCR 用户配置，因此新功能应新增独立的 `Project` 嵌套命令而不是复用或改名。
- `crates/ccr-cli/src/cli/dispatch.rs:41` 是统一命令分发入口；新命令应从这里进入项目初始化 handler。
- `crates/ccr-cli/src/commands/` 负责用户可见命令处理。建议新增 `commands/project/{mod.rs,init.rs}`，把 CLI 定义留在 `cli/subcommands/project.rs`。
- `crates/ccr-cli` 已依赖 `dialoguer = 0.12.0`，也已有 `std::process::Command` 配合继承终端的先例，无需新增生产依赖。
- 帮助正文由 `crates/ccr-cli/src/cli/help_config.rs` 对 Clap 命令树增强；集成帮助测试在 `crates/ccr/tests/commands/help.rs`。

## 2. Trellis 0.6.8 的实际初始化契约

本机已验证：

```text
trellis --version
0.6.8
```

`trellis init --help` 暴露 `-u/--user <name>` 和 20 个平台开关。安装包源码进一步证明：

- `dist/commands/init.js:885-936`：未传 `--user` 时，若当前目录含 `.git`，Trellis 先读取 `git config user.name`；读取成功则显示该用户名而不再次询问，读取不到才提示 `Your name:`。
- `dist/commands/init.js:1090-1117`：未显式传平台开关且未使用 `--yes` 时，Trellis 从自身平台注册表动态生成复选框 `Select AI tools to configure:`。
- `dist/commands/init.js:1121-1122`：选择为空时 Trellis 输出至少选择一个工具的提示并返回。
- 当前注册表默认勾选 Claude Code 和 Cursor；Codex 默认未勾选。用户可手动改为 Claude Code + Codex，从而达到等价于 `trellis init -u lyh --claude --codex` 的调用结果。

### 设计方案比较

| 方案 | 行为 | 优点 | 风险 |
|---|---|---|---|
| A. CCR 直接运行 `trellis init`（推荐） | Trellis 自己询问/推导用户名并展示全平台复选框 | 平台集合、显示名和默认值永远由安装的 Trellis 版本决定；CCR 无重复交互代码 | 已有 Git 用户名时只显示、不强制重新输入；提示语言由 Trellis 决定 |
| B. CCR 收集用户名与 Agent 后拼参数 | CCR 用 `dialoguer::Input` + `MultiSelect` 生成 `-u` 和平台 flags | 可提供统一中文体验，也能固定默认 Claude + Codex | 必须在 CCR 硬编码平台清单；每次 Trellis 新增、改名、废弃开关都可能漂移 |
| C. CCR 只收用户名，再交给 Trellis 选 Agent | 调用 `trellis init -u <name>` | 用户名体验可控，平台清单不漂移 | 出现两层交互，体验割裂；价值低于 A |

推荐 A。它把 CCR 定位为项目初始化编排器，而不是 Trellis CLI 的参数镜像。CCR 应使用 `Command::new("trellis").arg("init").current_dir(root).stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit()).status()`，让真实终端交互完整透传。

### 已确认决策

用户选择方案 A：CCR 直接运行原生交互式 `trellis init` 并透传终端。CCR 不复制用户名输入、Agent 平台注册表或参数映射。

## 3. Git 初始化检测

有两种不同语义：

- 检查 `<cwd>/.git`：只认当前目录自身的仓库元数据，符合 Trellis 自身判断用户名时的逻辑；父仓库中的普通子目录仍会执行嵌套 `git init`。
- `git rev-parse --is-inside-work-tree`：父仓库中的任何子目录都被视为已经初始化，不会形成嵌套仓库。

建议使用 `git rev-parse --show-toplevel` 并比较规范化后的仓库根与当前目录：

- 根相同：提示当前项目已初始化并跳过。
- 当前目录只是父仓库子目录：明确报错或询问，而不是静默创建嵌套仓库。
- 不在工作树：运行 `git init`。

该边界是产品选择，不能仅由代码仓库事实决定。

### 已确认决策

用户选择避免嵌套仓库：只要当前目录已位于某个 Git 工作树中，就显示实际仓库根、跳过 `git init`，然后仍以当前目录作为 Trellis 与 `.gitignore` 的作用目录继续执行。

不建议只检查 `.git` 是否为目录，因为 worktree/submodule 场景中 `.git` 可以是文件。若最终选择“只认当前目录”，检查应使用 `Path::exists()`，同时接受文件和目录。

## 4. `.gitignore` 写入策略

目标规则固定为：

```gitignore
.agents/
.claude/
.codex/
```

建议抽出纯函数处理文本并进行原子式文件替换：

1. 读取现有文件（不存在视为空）。
2. 逐行只对去除行尾 `\r` 后的精确规则去重；不把 `/.agents/`、`.agents/**` 或带否定的规则推断为等价，以免错误理解复杂 Git ignore 语义。
3. 仅追加缺失规则；保留原字节顺序和现有换行风格，确保追加前有一个完整行边界。
4. 写入同目录临时文件并持久化替换，避免进程中断留下截断文件。

精确匹配比调用 `git check-ignore` 更适合这里：后者会把父级或通配规则也视为已忽略，但验收要求是项目 `.gitignore` 明确包含三条可读规则。

## 5. 顺序、失败与重试

推荐流程：

```text
解析当前目录
  -> 检测/执行 git init
  -> 执行交互式 trellis init
  -> 幂等更新 .gitignore
  -> 输出整体成功摘要
```

- Git 失败：立即停止，避免在非预期目录初始化 Trellis。
- Trellis 失败：保留已成功的 Git 初始化；不写整体成功。重复运行可继续。
- `.gitignore` 失败：Trellis 已落盘，报告该阶段失败；重复运行只补忽略规则。
- 不做跨外部工具的伪事务或回滚。`git init` 与 `trellis init` 都可能创建大量文件，回滚会扩大破坏面。

需要注意：Trellis 现有初始化在部分早退路径可能只打印错误提示后正常返回，因此集成测试应使用可控的 fake `trellis` 可执行文件验证 CCR 对进程退出码的处理；真实 Trellis 的业务成功只能由退出码和关键落盘结果共同判断时，再评估是否增加 `.trellis/` 后置校验。

## 6. 可测试性建议

- 将外部工具执行与纯逻辑拆开，但保持在 `ccr-cli` 内，不新增跨 crate 抽象。
- 单元测试覆盖 Git 状态分类和 `.gitignore` 文本合并。
- 命令集成测试通过临时目录与 PATH 前置 fake `git`/`trellis`，记录参数和调用顺序；使用项目 `TestHostEnv`/串行环境锁，避免并行污染 `PATH`。
- 至少验证：新仓库、当前目录已有仓库、Git 失败、Trellis 缺失/失败、三种 `.gitignore` 增量状态、无末尾换行、重复运行。

## 7. 预计文件范围

- `crates/ccr-cli/src/cli/definitions.rs`
- `crates/ccr-cli/src/cli/dispatch.rs`
- `crates/ccr-cli/src/cli/subcommands/mod.rs`
- `crates/ccr-cli/src/cli/subcommands/project.rs`（新增）
- `crates/ccr-cli/src/commands/mod.rs`
- `crates/ccr-cli/src/commands/project/mod.rs`（新增）
- `crates/ccr-cli/src/commands/project/init.rs`（新增）
- `crates/ccr/tests/commands.rs`
- `crates/ccr/tests/commands/help.rs`
- `crates/ccr/tests/commands/project_init.rs`（新增）
- 可能更新中英文命令文档；是否纳入本任务在最终规划中明确。
