# `ccr project init` 平台、用户名与 `ref/` 设计

## 边界

改动留在现有项目初始化编排里，不新增 crate，也不把 Trellis 全量平台注册表搬进 CCR。

```text
project_init_command(auto_yes)
  ensure_git_repository(root)
  resolve_trellis_request(auto_yes, git, root)
  run_trellis_init(root, request)
  ensure_ref_dir(root)
  ensure_project_gitignore(root)
```

- `crates/ccr-cli/src/commands/project/init.rs` 拥有菜单、参数、`ref/` 和忽略规则。
- `dialoguer` 已在 `crates/ccr-cli/Cargo.toml` 中，版本 0.12。交互控件的 import 放在模块顶部。
- 用户名输入复用 `crate::commands::common::prompt_required`。
- Clap 命令树不增加新参数。全局 `auto_yes` 继续从 `dispatch.rs` 传入。

## 平台菜单

固定顺序的私有枚举，只包含这五项，避免 Kimi 或其他开关被拼进参数：

| 显示名 | 开关 | 默认 |
|---|---|---|
| Claude Code | `--claude` | 勾选 |
| Codex | `--codex` | 勾选 |
| Cursor | `--cursor` | 勾选 |
| Oh My Pi | `--omp` | 勾选 |
| Grok | `--grok` | 勾选 |

交互模式使用 `dialoguer::MultiSelect`，`defaults` 五项都是 `true`。提示语为「选择要初始化的平台」。返回的下标映射回上表顺序，取消勾选的项不出现在参数里。下标为空时打印警告并重新显示菜单，不调用 Trellis。

`MultiSelect::interact` 失败，包括没有终端或操作者取消，返回 `CcrError::UiError`，退出码 74。此时 Trellis 尚未启动。

## 用户名

交互模式在菜单得到至少一项之后调用 `prompt_required("开发者用户名", "用于 Trellis 工作区目录")`。空行继续询问。得到的字符串去掉首尾空白后作为单独一个参数传给 `-u`，保留名称内部的空格。

`-y` 不调用 `MultiSelect` 和 `prompt_required`。Git 阶段成功后，用同一 Git 可执行文件在当前目录运行 `git config user.name` 并捕获输出：

- 进程无法启动：`ExternalCommandError`，退出码 75。
- 退出非零，或修剪后的 stdout 为空：`ValidationError`，退出码 90。消息说明需要 `git config user.name`，并说明可以去掉 `-y` 后手动输入。
- 成功：该文本成为 `-u` 的值，平台固定为全部五项。

交互模式不读取 Git 用户名。

## Trellis 参数

纯函数把请求变成参数，供单元测试和真正的 `Command` 共用：

```text
init -u <name> [--claude] [--codex] [--cursor] [--omp] [--grok] [--yes]
```

开关只从枚举按上表顺序发出。`auto_yes` 时最后一个参数是 `--yes`。名称或平台列表为空时，函数返回 `ValidationError`，作为菜单和 Git 用户名检查之后的保护。

进程仍继承 stdin、stdout、stderr，工作目录仍是调用命令时的当前目录。退出成功后继续检查 `.trellis/workflow.md` 和 `.trellis/scripts/task.py`。

因此交互模式会把平台复选从 Trellis 移到 CCR，但 statusLine 和 spec 模板仍由不带 `--yes` 的 Trellis 询问。`-y` 会跳过这两项，并让 Trellis 按自身 `--yes` 语义跳过已有文件。

已有 `.trellis/` 时，因为同时传入 `-u` 和平台开关，Trellis 0.6.17 会跳过重初始化菜单。已有 `.developer` 不会被覆盖。

## `ref/` 与 `.gitignore`

Trellis 后置条件通过之后：

1. `root.join("ref")` 已是目录：保留内容，提示已存在。
2. 该路径是文件：`FileIoError`，退出码 51，不改 `.gitignore`。
3. 路径不存在：`create_dir`。失败同样是 `FileIoError`。
4. 然后合并忽略规则。`PROJECT_IGNORE_RULES` 增加最后一项 `ref/`。匹配仍是整行修剪后的精确相等。已有正文、顺序和 LF/CRLF 保持不变；有缺失规则时才用 `AtomicWriter` 写入。

规则已经完整时不重写 `.gitignore`，但第 1 到第 3 步仍然执行。所以「忽略规则已在、目录缺失」会补目录。

成功摘要改为「Git、Trellis、Agent 忽略规则和 ref/ 均已就绪」。忽略规则无需写入时的提示改为「.gitignore 已包含全部项目忽略规则，跳过写入」。

## 帮助、文档和规范

同步这些用户可见文字：

- `crates/ccr-cli/src/cli/subcommands/project.rs` 的 `Init` 文档注释。
- `crates/ccr-cli/src/cli/help_config.rs` 里 `PROJECT_LONG_ABOUT`、`PROJECT_AFTER_LONG_HELP`、`PROJECT_INIT_LONG_ABOUT`、`PROJECT_INIT_AFTER_LONG_HELP`。
- `docs/reference/commands/project-init.md` 与 `docs/en/reference/commands/project-init.md`。
- 两份 `docs/**/reference/commands/index.md` 的 `project init` 一行用途，补上本地 `ref/`。

帮助和参考需要写明：交互模式默认勾选五项且可以取消，然后输入用户名；`-y` 使用这五项和 `git config user.name`，并传给 `trellis init --yes`；当前目录会创建被忽略的 `ref/`。

`.trellis/spec/ccr-cli/backend/backend-guidelines.md` 的 Project Workflow Bootstrap 改成同一契约：五项菜单、两种用户名来源、`ref/` 规则，以及「裸 `trellis init` 或传入 `--kimi`」作为错误示例。保留 Git 父仓库检测、最低 Trellis 文件、原子合并和失败不回滚。

## 测试

进程测试没有终端。`dialoguer` 在这种环境下会失败，因此现有到达 Trellis 的 `project init` 用例改为 `ccr -y project init`，并由 fake Git 提供用户名。Git 失败和 Git 缺失的用例仍停在 Git 阶段，断言没有平台询问和 Trellis 调用。

`crates/ccr/tests/commands/project_init.rs` 的 Windows 与 Unix fake Git 都增加 `config user.name`：

- 环境变量 `FAKE_GIT_USER` 非空时打印该值并成功退出。
- 未设置时非零退出。
- 继续记录完整参数，避免把 `config` 误判成 `init`。

进程断言：

- `-y` 且用户名可用：调用顺序包含 `rev-parse`、必要时 `init`、`config user.name`，然后是精确的 Trellis 参数。当前目录有 `ref/`，`.gitignore` 含四条规则。
- `-y` 且没有用户名：退出码 90，日志里没有 `trellis|`，没有 `ref/` 和 `.gitignore`。
- Trellis 失败或最低文件缺失：没有 `ref/`，`.gitignore` 不变。
- `ref/` 已有文件：文件保留。
- `ref` 是普通文件：退出码 51，发生在 Trellis 成功之后。
- 四条忽略规则已在而 `ref/` 不存在：创建目录，`.gitignore` 修改时间不变。

`crates/ccr-cli/src/commands/project/init.rs` 的单元测试覆盖参数函数：五项全选、去掉中间一项后的顺序、拒绝空平台、`-y` 的 `--yes` 位置，以及包含 `ref/` 的忽略规则合并。部分取消不通过进程测试驱动复选框。

帮助测试继续检查 `trellis init`、`.agents/` 和 `--yes`，并增加 `ref/` 与五项平台名的断言。

## 取舍

CCR 固定五项，Trellis 以后新增平台不会自动出现在这个菜单里。这换来默认勾选和可取消，同时不复制完整注册表。

交互模式仍会进入 Trellis 自己的 statusLine 和模板询问。`-y` 会跳过它们。这和「交互模式不追加 `--yes`」的需求一致。
