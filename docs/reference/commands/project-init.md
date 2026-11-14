# `project init` - 初始化项目工作流

`ccr project init` 以四段式终端向导在当前工作目录中依次准备 Git 仓库、项目工作流引擎（`mini-trellis` 或 `trellis`）、被忽略的本地 `ref/` 目录以及固定 7 条 Agent `.gitignore` 规则，并在收尾展示结构化状态摘要卡与后续命令引导。它不会修改 CCR 的用户级配置；用户级配置仍由 [`ccr init`](./init) 管理。

## 用法

```bash
# 交互式向导：默认推荐 mini-trellis，并自动预填 git config user.name
ccr project init

# 显式指定工作流引擎（跳过引擎单选，直接进入平台复选与用户名确认）
ccr project init --workflow mini-trellis
ccr project init --workflow trellis

# 非交互默认兼容模式：默认采用 trellis 全部 5 项平台 + git config user.name
ccr -y project init

# 非交互初始化 mini-trellis：启用全部 4 项平台 + git config user.name
ccr -y project init --workflow mini-trellis
```

`--workflow <trellis|mini-trellis>` 大小写不敏感。命令不提供 `--json` 开关，也始终只作用于启动 CCR 时的当前目录，不接受项目路径参数。

## 选项与工作流引擎对比

| 维度 | `mini-trellis` | `trellis` |
|---|---|---|
| 定位 | 轻量规格、研究笔记与跨会话记忆层 | 任务驱动的完整多阶段工作流 |
| 下游可执行文件 | `mini-trellis` | `trellis` |
| 默认触发方式 | 交互式向导首项默认高亮推荐 | `ccr -y project init` 未传 `--workflow` 时默认采用（向后兼容） |
| 支持的宿主平台 | **4 项**：Claude Code（`--claude`）、Codex（`--codex`）、Cursor（`--cursor`）、Oh My Pi（`--omp`），**不含 Grok** | **5 项**：Claude Code（`--claude`）、Codex（`--codex`）、Cursor（`--cursor`）、Oh My Pi（`--omp`）、Grok（`--grok`） |
| 最低产物校验 | `.trellis/.mini-trellis-manifest.json`、`.trellis/scripts/add_session.py`，以及含 `Managed by mini-trellis.` 的 `AGENTS.md` | `.trellis/workflow.md` 与 `.trellis/scripts/task.py` |
| 收尾 `下一步` 建议 | `mini-trellis doctor` 与 `mini-trellis mem recall "project init"` | `trellis doctor` 与 `ccr project init --help` |

## 前置依赖

运行前需要安装并在 `PATH` 中提供：

- Git（`git`）
- 所选工作流引擎对应的 CLI：`mini-trellis` 或 `trellis`

CCR 不会自动安装这些外部工具。平台菜单仅包含上表列出的受支持集合，不展示或传递 `--kimi` 等列表外开关。

## 四阶段执行流程

启动时 CCR 会先输出头部上下文（`目标目录`、`运行模式`，以及显式传入 `--workflow` 时的 `工作流预设`），随后按固定顺序执行四个编号阶段：

### 1. `[1/4]` 检查 Git 仓库

CCR 先在当前目录运行 `git rev-parse --show-toplevel`：

- 当前目录不在任何 Git 工作树中时，运行 `git init`。
- 当前目录本身已是仓库根时，复用仓库并跳过 `git init`。
- 当前目录位于父级仓库中时，输出告警显示实际父仓库根并跳过嵌套 `git init`，避免创建嵌套仓库。

即使复用父级仓库，后续的工作流初始化、`ref/` 和 `.gitignore` 仍只作用于调用命令时的当前目录。Git 阶段失败时不会进入后续交互或外部命令。

### 2. `[2/4]` 配置工作流与宿主平台

进入本阶段后，CCR 先检测当前目录的特征文件指纹（见下文「跨引擎冲突阻断与幂等重试」）：

- **交互模式**：
  - 未传 `--workflow` 时显示单选菜单，默认高亮推荐第 1 项 `mini-trellis`；已传 `--workflow` 时直接采用指定引擎。
  - 目标引擎确定后立即校验是否存在跨引擎冲突；无冲突再显示对应引擎的宿主平台复选菜单（`trellis` 5 项、`mini-trellis` 4 项，默认全部勾选）。全部取消勾选时会提示警告并重新显示菜单，不会启动下游 CLI。
  - 静默读取当前目录 `git config user.name`：若存在非空值则预填为开发者用户名默认值（直接回车即可采用）；若未配置，则要求手动输入非空名称，空白输入会提示并重问。
- **非交互模式（`-y` / `--yes`）**：
  - 不显示任何交互菜单；未传 `--workflow` 时默认采用 `trellis`，并启用目标引擎支持的全部平台（`trellis` 5 项或 `mini-trellis` 4 项）。
  - 开发者用户名严格读取自当前目录 `git config user.name`；若该命令无法启动、退出非零或输出为空白，命令会在调用下游 CLI 前直接失败并提示配置 `git config user.name` 或去掉 `-y` 手动输入。

采集完成后，终端会打印 `工作流引擎`、`宿主平台` 与 `开发者` 三行确认摘要。

### 3. `[3/4]` 执行工作流初始化

CCR 使用当前终端标准流继承（`Stdio::inherit()`）启动下游 CLI，并在子进程输出前后打印视觉边界行（彩色/普通 TTY 使用 `┌─` / `└─` 边框，非 TTY 管道与 `TERM=dumb` 自动降级为 `---` 纯文本分隔线）：

```bash
# 选择 trellis 时（仅 -y 模式末尾附加 --yes）
trellis init -u <开发者用户名> --claude --codex --cursor --omp --grok [--yes]

# 选择 mini-trellis 时（不含 --grok；仅 -y 模式末尾附加 --yes）
mini-trellis init -u <开发者用户名> --claude --codex --cursor --omp [--yes]
```

交互模式下取消某项平台时，对应开关被省略，保留的开关严格按菜单顺序排列；未带 `--yes` 时下游 CLI 仍可能按自身逻辑询问附加项（例如 `trellis` 的 `statusLine` 或远程模板）。CCR **绝不传递 `-s` / `--skip-existing`**，确保同引擎重复运行走下游重初始化逻辑。

下游进程返回退出码 `0` 后，CCR 还会校验所选引擎的最低工作流产物；若任一必需产物缺失，仍判定为校验失败（退出码 `90`）并停止后续阶段。

### 4. `[4/4]` 准备 `ref/` 目录与 `.gitignore` 忽略规则

工作流最低产物校验通过后：

- `ref/` 已是目录时，保留其中的现有内容。
- `ref` 已是普通文件（或非目录条目）时，返回文件 I/O 错误（退出码 `51`），且不修改 `.gitignore`。
- `ref/` 不存在时，创建该目录。

随后无论本次选择了哪套工作流引擎或勾选了哪些平台，CCR 都会在保留现有内容、注释、顺序与 LF/CRLF 换行风格的前提下，按序补充缺失的固定 7 条规则：

```text
.agents/
.claude/
.codex/
.cursor/
.omp/
.grok/
ref/
```

`.cursor/`、`.omp/`、`.grok/` 对应各宿主平台的本地目录，`.agents/` 是 Codex 与跨宿主技能共用目录；`.trellis/` 属于需要纳入版本控制的工作流与规格资产，不在忽略列表中。7 条规则均已存在时跳过写入，保持 `.gitignore` 字节与时间戳不变（但缺失的 `ref/` 目录仍会补建）；存在缺失规则时通过原子写入更新，旧版仅含 4 条规则的 `.gitignore` 会在再次运行时自动补齐 `.cursor/`、`.omp/` 与 `.grok/`。

### 结果摘要卡与终端降级

四个阶段全部成功后，CCR 会输出：

1. **6 行状态摘要表**：汇总 `Git 仓库`、`工作流引擎`（区分 `首次初始化` 与 `幂等更新`）、`宿主平台`、`开发者`、`ref/ 目录` 与 `.gitignore` 的执行状态及结果详情。
2. **总结就绪状态行**与**引擎专属 `下一步` 命令建议**。

在管道重定向、`NO_COLOR=1`、`CLICOLOR=0` 或 `TERM=dumb` 环境下，交互主题、状态前缀、子进程边界线与摘要表格会自动去除 ANSI 颜色转义（`TERM=dumb` 下表格进一步降级为纯 ASCII 边框），保证日志采集与 CI 脚本输出干净可读。

## 跨引擎冲突阻断与幂等重试

### 前置工作区指纹阻断（退出码 `90`）

为防止跨引擎混跑破坏现有目录（特别是已有 `mini-trellis` 时误跑 `trellis init -y` 会以退出码 `0` 静默产生混合损坏），CCR 在启动外部工作流 CLI 前会检查 `Trellis` 与 `mini-trellis` 的特征指纹：

- **已有 `trellis` 资产，目标为 `mini-trellis`**：以退出码 `90` 阻断，提示先运行 `mini-trellis migrate --dry-run` 审阅迁移计划，确认后执行 `mini-trellis migrate`。
- **已有 `mini-trellis` 资产，目标为 `trellis`（含裸 `ccr -y project init` 默认行为）**：以退出码 `90` 前置阻断；如确需切换引擎，提示先运行 `mini-trellis uninstall` 清理后再重试。
- **同时命中两套引擎特征（混杂目录）**：在任何交互提示与外部调用前直接以退出码 `90` 阻断，提示先检查清理混杂资产。

所有冲突阻断均发生在调用 `trellis` / `mini-trellis`、创建 `ref/` 与写入 `.gitignore` 之前。

### 失败边界与同引擎幂等重试

任一阶段失败时命令立即停止，不输出收尾摘要卡或 `下一步` 建议，也不会回滚此前已完成的阶段：

- `git` 缺失或 `git init` 失败（退出码 `75`）：不进入平台/用户名阶段，不调用工作流 CLI，不改动 `ref/` 与 `.gitignore`。
- 交互终端异常（退出码 `74`）、`-y` 缺少 `git config user.name` 或触发工作流冲突（退出码 `90`）：不调用工作流 CLI，不改动 `ref/` 与 `.gitignore`。
- `trellis` / `mini-trellis` 缺失或返回非零状态（退出码 `75`），或缺少最低工作流产物（退出码 `90`）：保留已就绪的 Git 仓库，不创建 `ref/`，也不更新 `.gitignore`。
- `ref` 同名文件冲突或 `.gitignore` 读写失败（退出码 `51`）：保留已完成的 Git 与工作流初始化结果。

修复报错原因后，在同一目录使用相同工作流引擎重新运行 `ccr project init` 即可安全恢复。同引擎重跑会保留 `ref/` 内容、已有自定义规格/研究笔记/会话日志与模板哈希清单，并在摘要卡中标记为 `幂等更新`。

## 相关命令

- [`init`](./init) - 初始化用户级 CCR 配置
- [`commands`](./index) - 查看完整命令索引
