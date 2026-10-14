# 优化 `ccr project init`

## Goal

`ccr project init` 在当前目录完成 Git 和 Trellis 初始化后，留下本地 `ref/` 目录并把它写入 `.gitignore`。交互运行时先给出可修改的平台复选，Claude Code、Codex、Cursor、Oh My Pi 和 Grok 默认勾选；确认后再手动输入开发者用户名。`ccr -y project init` 跳过这两次询问，使用这五项和 `git config user.name`。

## Requirements

1. 成功结束后，调用命令时的当前目录存在 `ref/` 目录，且该目录的 `.gitignore` 含有精确行 `ref/`。已有目录内容保持不变。`ref/` 已是目录时不重建、不清空。`ref` 已是文件时，该阶段失败并说明冲突。
2. `.gitignore` 仍只追加缺失规则，保留原文、顺序和 LF/CRLF。固定规则为 `.agents/`、`.claude/`、`.codex/`、`ref/`。四条都已存在时不重写文件。`ref/` 目录缺失时，即使忽略规则已经完整，也要创建该目录。
3. 交互模式在 Git 阶段成功之后、启动 Trellis 之前，先显示平台复选。菜单只有五项，默认全部勾选：Claude Code（`--claude`）、Codex（`--codex`）、Cursor（`--cursor`）、Oh My Pi（`--omp`）、Grok（`--grok`）。操作者可以取消任意项。Kimi 和其他 Trellis 平台不出现。全部取消时留在这一步，不启动 Trellis。
4. 平台确认之后询问开发者用户名。空输入必须再次询问，直到得到非空名称。程序不得内置 `lyh` 或任何其他默认用户名。已有 `.trellis/.developer` 时仍询问。
5. 交互模式执行 `trellis init -u <name>`，并按菜单顺序只附加最终保留的平台开关。不追加 `--yes`。标准输入、输出和错误继承当前终端。
6. `ccr -y project init` 不显示复选，也不询问用户名。平台固定为上述五项。用户名来自在当前目录执行的 `git config user.name`；退出非零、输出为空或只有空白时失败，提示配置该名称，或去掉 `-y` 后手动输入。失败发生在启动 Trellis 之前。成功时的参数为 `init -u <git 用户名> --claude --codex --cursor --omp --grok --yes`。
7. 阶段顺序是 Git、平台与用户名、Trellis、然后 `ref/` 与 `.gitignore`。任一阶段失败即停止，不报告整体成功，不回滚已完成阶段。Trellis 失败或最低工作流文件缺失时，不创建 `ref/`，也不改 `.gitignore`。Git 失败时不询问平台和用户名。
8. 父级 Git 工作树、当前目录即仓库根、工具缺失、非零退出，以及 `.trellis/workflow.md` 与 `.trellis/scripts/task.py` 都存在这些既有边界保持不变。命令仍只作用于当前目录，不增加路径参数。
9. 帮助文案、中英文命令索引和 `project init` 参考要写明五项默认勾选、手动用户名、`-y` 的 Git 用户名来源，以及被忽略的 `ref/`。
10. `.trellis/spec/ccr-cli/backend/backend-guidelines.md` 的 Project Workflow Bootstrap 要改成这份五项菜单、两种用户名来源和 `ref/` 契约。

## Technical Notes

这些事实决定了上面的交互归属，实现时不能靠给裸 `trellis init` 加开关来代替 CCR 的复选：

- Trellis CLI 0.6.17 的交互复选框只预勾注册表里 `defaultChecked: true` 的项。全表只有 Claude Code 和 Cursor（`dist/types/ai-tools.js:23`、`dist/types/ai-tools.js:39`）。
- 传入任一平台开关后，Trellis 不再显示复选框（`dist/commands/init.js:1092-1098`）。
- 未传 `-u` 且当前目录已有 `.git` 时，Trellis 采用 `git config user.name`，不再询问（`dist/commands/init.js:886-904`）。本命令的 Git 阶段会先建立这个条件。
- 未带 `--yes` 时，Trellis 仍可能询问 Claude Code statusLine 和 spec 模板。
- 已有 `.trellis/.developer` 时，`init_developer.py:36-42` 成功退出且不覆盖原身份。
- 当前编排在 `crates/ccr-cli/src/commands/project/init.rs:7` 和 `crates/ccr-cli/src/commands/project/init.rs:86-94`。本仓库根 `.gitignore:99` 使用精确行 `ref/`。

## Acceptance Criteria

- [ ] AC1: 新目录执行成功后，当前目录有 `ref/` 目录，`.gitignore` 同时包含 `.agents/`、`.claude/`、`.codex/` 和 `ref/`。
- [ ] AC2: 已有 `.gitignore` 只补缺失行；四条规则已在时文件修改时间不变。已有 `ref/` 目录及其内容不变。规则已完整但 `ref/` 目录缺失时会创建目录且不重写 `.gitignore`。`ref` 为普通文件时命令失败，且错误指向这一阶段。
- [ ] AC3: 接受全部默认项并输入名称后，Trellis 参数是 `init -u <输入的名称> --claude --codex --cursor --omp --grok`。取消某一项后，对应开关不出现，其余开关仍按菜单顺序排列。参数中没有 `--kimi`。空名称或零个平台不会启动 Trellis。
- [ ] AC4: `ccr -y project init` 在 Git 用户名可用时记录到 `init -u <该名称> --claude --codex --cursor --omp --grok --yes`。Git 用户名缺失时退出码 90，不调用 Trellis，不产生 `ref/` 或 `.gitignore`。
- [ ] AC5: Git 失败不询问平台和用户名，也不调用 Trellis，不产生 `ref/` 或 `.gitignore`。Trellis 失败或成功退出但缺少 `.trellis/workflow.md`、`.trellis/scripts/task.py` 时，不产生 `ref/`，也不改 `.gitignore`。
- [ ] AC6: `ccr project init --help` 与中英文命令参考写明五项默认勾选、手动用户名、`-y` 读取 Git 用户名，以及 `ref/`。

## Out of Scope

- 自动安装 Git 或 Trellis。
- 把 Trellis 全量平台注册表复制进 CCR，或提供 Kimi 以及其他未列出的平台开关。
- 为 `.cursor/`、`.grok/`、`.kimi-code/`、`.omp/` 等其他平台目录新增忽略规则。
- 在 `ref/` 内写入说明文件、占位文件或克隆任何仓库。
- 修改用户级 `ccr init`。
- 增加项目路径参数，或在父仓库里改父级 `.gitignore`。
- 回滚 `git init` 或 `trellis init` 已创建的文件。
- 改变 Trellis 自己的 statusLine、模板选择和已有开发者身份不覆盖的行为。
