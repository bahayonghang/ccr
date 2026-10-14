# `project init` - 初始化项目工作流

`ccr project init` 在当前工作目录中依次准备 Git 仓库、Trellis 平台与开发者用户名、Trellis 工作流，以及被忽略的本地 `ref/` 目录。它不会修改 CCR 的用户级配置；用户级配置仍由 [`ccr init`](./init) 管理。

## 用法

```bash
# 交互模式：五项默认勾选，然后手动输入用户名
ccr project init

# 非交互：五项平台 + git config user.name，并传给 trellis init --yes
ccr -y project init
```

命令固定作用于启动 CCR 时的当前目录，不接受项目路径参数。

## 依赖

运行前需要安装并在 `PATH` 中提供：

- Git（`git`）
- Trellis CLI（`trellis`）

CCR 不会自动安装这些工具。平台菜单固定为下面五项，不复制 Trellis 的完整平台注册表。

## 执行阶段

### 1. Git

CCR 先运行 `git rev-parse --show-toplevel`：

- 当前目录不在 Git 工作树中时，运行 `git init`。
- 当前目录本身是仓库根时，跳过 `git init`。
- 当前目录位于父级仓库中时，显示实际仓库根并跳过 `git init`，避免创建嵌套仓库。

即使复用父级仓库，后续平台选择、Trellis、`ref/` 和 `.gitignore` 仍只作用于调用命令时的当前目录。Git 失败时不询问平台和用户名。

### 2. 平台与用户名

交互模式在 Git 成功后显示复选。菜单默认全部勾选，可以取消任意项：

- Claude Code（`--claude`）
- Codex（`--codex`）
- Cursor（`--cursor`）
- Oh My Pi（`--omp`）
- Grok（`--grok`）

全部取消时会重新显示菜单，不会启动 Trellis。确认至少一项后，手动输入开发者用户名。空输入会再次询问，直到得到非空名称。

`ccr -y project init` 不显示复选，也不询问用户名。用户名来自当前目录的 `git config user.name`。该命令无法启动时命令失败。退出非零、输出为空或只有空白时，命令失败并提示设置 `git config user.name`，或去掉 `-y` 后手动输入。这次失败发生在启动 Trellis 之前。

### 3. Trellis

交互模式继承当前终端，按菜单顺序只附加保留的开关：

```bash
trellis init -u <输入的用户名> --claude --codex --cursor --omp --grok
```

取消某一项时，对应开关不出现，其余开关仍按上述顺序排列。未带 `--yes` 时，Trellis 仍可能询问 Claude Code statusLine 和 spec 模板。

`ccr -y project init` 在 Git 用户名可用时运行：

```bash
trellis init -u <git config user.name> --claude --codex --cursor --omp --grok --yes
```

CCR 会检查 Trellis 的退出状态，并确认当前目录至少包含 `.trellis/workflow.md` 和 `.trellis/scripts/task.py`。即使 Trellis 返回成功码，缺少这些文件也会视为失败。

### 4. `ref/` 与 `.gitignore`

Trellis 最低文件就绪后：

- `ref/` 已是目录时，保留其中的内容。
- `ref` 已是文件时，命令失败，并且不修改 `.gitignore`。
- `ref/` 不存在时，创建该目录。

然后保留现有内容、注释、顺序和 LF/CRLF 换行风格，只补充缺少的固定规则：

```text
.agents/
.claude/
.codex/
ref/
```

四条规则都已存在时不会重写 `.gitignore`。即使忽略规则已经完整，缺失的 `ref/` 目录仍会创建。新增或更新使用原子写入，重复执行不会重复追加规则。

## 失败与重试

阶段顺序固定为 Git、平台与用户名、Trellis、`ref/`、`.gitignore`。任一阶段失败后立即停止，不报告整体成功，也不会回滚此前已经完成的外部操作：

- Git 缺失或 `git init` 失败时，不询问平台和用户名，不运行 Trellis，不创建 `ref/`，也不更新 `.gitignore`。
- 平台选择失败，或 `-y` 读不到 Git 用户名时，不运行 Trellis。
- Trellis 缺失、返回非零状态或缺少最低工作流文件时，不创建 `ref/`，也不更新 `.gitignore`。
- `ref` 为普通文件时，不更新 `.gitignore`。
- `.gitignore` 读取或写入失败时，保留已经完成的 Git、Trellis 和 `ref/` 结果。

修复错误原因后，在同一目录重新运行 `ccr project init` 即可继续。Git 检测、Trellis 重初始化、`ref/` 目录检查和 ignore 规则合并都支持安全重试。

## 相关命令

- [`init`](./init) - 初始化用户级 CCR 配置
- [`commands`](./index) - 查看完整命令索引
