# 添加 `project init` 项目初始化命令

## Goal

为 CCR 增加 `ccr project init`，在当前工作目录中一次完成 Git 仓库、Trellis 工作流和本地 Agent 目录忽略规则的初始化，减少新项目的重复手工配置，同时安全兼容已经由 `uv init` 等工具初始化过 Git 的项目。

## Background

- 现有顶层 `ccr init` 初始化的是用户级 CCR 配置，必须保持原语义和兼容行为。
- 本机 Trellis 0.6.8 已提供原生开发者用户名处理、Agent 平台复选框、`-u/--user`、平台开关和 `-y/--yes`。
- CCR 的命令定义、分发、外部进程编排及人类可读输出归属 `ccr-cli`；该 crate 已具备所需依赖和跨平台原子写入能力，不需要新增生产依赖。

## Requirements

### R1. Command Surface

- 新增两级命令 `ccr project init`，作用对象固定为进程当前工作目录。
- `ccr project --help`、`ccr project init --help` 和 `ccr help project init` 必须清楚说明 Git、Trellis、`.gitignore` 三个阶段。
- 裸 `ccr project` 不得隐式执行初始化，应显示帮助或缺少子命令错误。
- 现有顶层 `ccr init` 的解析、帮助和行为保持不变。

### R2. Git Initialization

- 先验证 `git` 可执行并判断当前目录是否已经位于 Git 工作树中。
- 当前目录不在任何 Git 工作树中时，在当前目录执行 `git init`。
- 当前目录本身是仓库根时，提示仓库已初始化、跳过 `git init` 并继续。
- 当前目录只是父级仓库的子目录时，显示检测到的仓库根、跳过 `git init` 并继续；不得创建嵌套仓库。
- `git` 不可执行或 `git init` 返回非零状态时，以外部命令错误停止，不执行 Trellis 和 `.gitignore` 阶段。

### R3. Trellis Initialization

- 默认执行原生交互式 `trellis init`，由 Trellis 自己处理用户名和 Agent 平台选择；CCR 必须继承 stdin、stdout 和 stderr。
- CCR 不硬编码 Trellis 的平台清单、显示名、默认选项或 `-u`/平台参数映射。
- 用户可以在 Trellis 原生交互中选择 Claude Code 与 Codex，达到等价于 `trellis init -u lyh --claude --codex` 的结果。
- 全局 `ccr -y project init` 必须向 Trellis 转发 `--yes`，由当前 Trellis 版本按自身默认值执行非交互初始化。
- `trellis` 不可执行、返回非零状态，或正常退出后未形成最低 Trellis 工作流结构时，CCR 必须返回失败且不得报告整体成功。
- 已存在有效 Trellis 工作流时仍调用原生 `trellis init`，由 Trellis 自身的重初始化流程处理；若用户取消但既有结构仍有效，CCR 可继续视为 Trellis 已就绪。

### R4. `.gitignore` Management

- 在当前目录的 `.gitignore` 中明确包含以下三条规则：`.agents/`、`.claude/`、`.codex/`。
- 文件不存在时创建；存在时保留原有内容、注释和行顺序，只追加缺失规则。
- 对已有目标规则不重复追加；重复运行命令必须幂等，目标内容无需变化时不得重写文件。
- 追加前必须补齐有效行边界，并沿用已有 CRLF 或 LF 换行风格。
- 使用仓库现有跨平台原子写入能力，避免写入失败留下截断文件。
- 读取或写入失败时返回文件错误，不报告整体初始化成功。

### R5. Sequencing, Output And Recovery

- 固定顺序为：Git 检测/初始化 -> Trellis 初始化 -> `.gitignore` 合并 -> 整体成功摘要。
- 每个阶段输出可识别的开始、跳过或成功状态；错误必须指出失败阶段和外部工具名称。
- 中途失败不回滚已经成功的 Git 或 Trellis 操作；重复运行必须能够安全继续剩余步骤。
- 仅在三个阶段均达到目标状态后输出整体成功。

### R6. Documentation

- 新增中英文 `project init` 命令参考，说明交互模式、`--yes`、父仓库行为、依赖项、写入内容和部分失败后的重试方式。
- 更新中英文命令索引，使新命令可发现，并明确区分 `ccr init` 与 `ccr project init`。

## Acceptance Criteria

- [ ] AC1: 顶层帮助包含 `project`；三种 project 帮助入口一致可用；`ccr init` 仍解析为用户级 CCR 配置初始化。
- [ ] AC2: 在不属于任何 Git 工作树的临时目录运行后，fake/real Git 收到 `init`，后续 Trellis 在同一目录运行。
- [ ] AC3: 当前目录已经是仓库根时不运行 `git init`；位于父仓库子目录时显示父仓库根、不创建嵌套 `.git`，并继续在当前目录运行 Trellis。
- [ ] AC4: 普通模式运行 `trellis init` 且继承终端；`ccr -y project init` 运行 `trellis init --yes`；CCR 不包含 Trellis 平台枚举。
- [ ] AC5: Trellis 缺失、非零退出或零退出但最低工作流结构缺失时，命令非零退出，不执行 `.gitignore` 阶段且不输出整体成功。
- [ ] AC6: `.gitignore` 不存在、已有其他内容、缺少末尾换行、只有部分目标规则、三条均已存在以及 CRLF 文件时，原内容得到保留、换行有效、目标规则各出现一次。
- [ ] AC7: 第二次运行不会重复写入目标规则；前一次在 Trellis 或 `.gitignore` 阶段失败后可通过重试完成，不要求回滚 Git/Trellis 文件。
- [ ] AC8: Git 缺失或 `git init` 非零退出时，命令非零退出且 Trellis 未被调用；`.gitignore` 读写失败时错误指向该文件阶段。
- [ ] AC9: 中英文命令文档和索引准确描述已实现行为，文档审计与构建通过。
- [ ] AC10: `ccr-cli` 单元测试、`ccr` 命令集成测试、格式、严格 Clippy、Rust 工作区测试及最终仓库门禁按实施计划通过。

## Out Of Scope

- 自动安装 Git、Trellis CLI、Python 或 Node.js。
- 增加项目路径参数；从当前目录向上或向下寻找“项目根”。
- 修改 Git 全局配置、创建初始提交、创建或切换分支、配置远端。
- 在父级仓库子目录中创建嵌套 Git 仓库。
- 在 CCR 中复制 Trellis 的用户名输入、Agent 注册表、平台默认值或重初始化菜单。
- 为命令增加桌面 UI、TUI、VS Code 入口、JSON 输出或 dry-run。
- 管理 `.gitignore` 中除 `.agents/`、`.claude/`、`.codex/` 外的规则，或忽略其他所选 Agent 平台目录。
- 跨外部工具事务、自动回滚或删除 `git init`/`trellis init` 已创建的文件。

## Key Decisions

- 使用单个 Trellis 任务交付：代码、测试和双语文档共同构成一个不可分割的命令契约，不拆父子任务。
- Trellis 原生交互是平台清单的唯一事实源；CCR 只负责进程编排和完成状态验证。
- 任意父级 Git 工作树都视为 Git 已初始化，避免嵌套仓库，但 Trellis 和 `.gitignore` 仍作用于调用时的当前目录。
- 全局 `--yes` 转发给 Trellis，保持 CCR 已有全局参数契约。
