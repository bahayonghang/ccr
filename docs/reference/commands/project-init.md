# `project init` - 初始化项目工作流

`ccr project init` 在当前工作目录中依次准备 Git 仓库、Trellis 工作流和本地 Agent 目录忽略规则。它不会修改 CCR 的用户级配置；用户级配置仍由 [`ccr init`](./init) 管理。

## 用法

```bash
# 交互式运行 Trellis 初始化
ccr project init

# 将全局 --yes 转发给 Trellis
ccr -y project init
```

命令固定作用于启动 CCR 时的当前目录，不接受项目路径参数。

## 依赖

运行前需要安装并在 `PATH` 中提供：

- Git（`git`）
- Trellis CLI（`trellis`）

CCR 不会自动安装这些工具，也不会复制 Trellis 的用户名输入、Agent 平台清单或平台参数。

## 执行阶段

### 1. Git

CCR 先运行 `git rev-parse --show-toplevel`：

- 当前目录不在 Git 工作树中时，运行 `git init`。
- 当前目录本身是仓库根时，跳过 `git init`。
- 当前目录位于父级仓库中时，显示实际仓库根并跳过 `git init`，避免创建嵌套仓库。

即使复用父级仓库，后续 Trellis 和 `.gitignore` 仍只作用于调用命令时的当前目录。

### 2. Trellis

普通模式继承当前终端运行：

```bash
trellis init
```

用户名和 Agent 平台由 Trellis 自己询问或推导。例如，可在 Trellis 的原生交互中选择 Claude Code 和 Codex。

使用全局 `-y` / `--yes` 时，CCR 运行：

```bash
trellis init --yes
```

CCR 会检查 Trellis 的退出状态，并确认当前目录至少包含 `.trellis/workflow.md` 和 `.trellis/scripts/task.py`。即使 Trellis 返回成功码，缺少这些文件也会视为失败。

### 3. `.gitignore`

CCR 保留现有内容、注释、顺序和 LF/CRLF 换行风格，只补充缺少的固定规则：

```text
.agents/
.claude/
.codex/
```

规则已完整存在时不会重写文件。新增或更新使用原子写入，重复执行不会重复追加规则。

## 失败与重试

阶段顺序固定为 Git、Trellis、`.gitignore`。任一阶段失败后立即停止，不报告整体成功，也不会回滚此前已经完成的外部操作：

- Git 缺失或 `git init` 失败时，不运行 Trellis。
- Trellis 缺失、返回非零状态或缺少最低工作流文件时，不更新 `.gitignore`。
- `.gitignore` 读取或写入失败时，保留已经完成的 Git 和 Trellis 结果。

修复错误原因后，在同一目录重新运行 `ccr project init` 即可继续。Git 检测、Trellis 重初始化和 ignore 规则合并都支持安全重试。

## 相关命令

- [`init`](./init) - 初始化用户级 CCR 配置
- [`commands`](./index) - 查看完整命令索引
