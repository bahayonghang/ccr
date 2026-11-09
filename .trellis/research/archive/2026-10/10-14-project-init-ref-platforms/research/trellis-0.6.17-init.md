# Trellis 0.6.17 `init` 交互契约

本机命令：`trellis init --help`。安装包版本 0.6.17，路径 `C:\home\lyh\.npm-global\node_modules\@mindfoldhq\trellis`。项目内 Trellis 文件版本更高时，下面的判断仍以这次读到的安装包为准。

## 平台复选

- `dist/types/ai-tools.js` 里只有两处 `defaultChecked: true`：Claude Code（第 23 行，`cliFlag: "claude"`）和 Cursor（第 39 行，`cliFlag: "cursor"`）。
- Codex、Oh My Pi（`omp`）、Grok、Kimi 的 `defaultChecked` 都是 `false`。
- `dist/commands/init.js:1092-1118`：存在任一显式平台开关时直接使用这些开关；没有任何开关且带 `--yes` 时使用 `defaultChecked`；两者都不是时才弹出 “Select AI tools to configure”。
- 因此 CCR 不能既把 `--claude --codex --cursor --omp --grok` 传给 Trellis，又保留 Trellis 自己的预勾复选框。预勾这五项必须由 CCR 完成，再把最终选择作为显式开关传入。

## 用户名

- `dist/commands/init.js:886-904`：未传 `--user` 且当前目录存在 `.git` 时，执行 `git config user.name` 并直接采用。
- `dist/commands/init.js:926-936`：没有名称且没有 `--yes` 时才提示 `Your name:`。
- `ccr project init` 先确保 Git 仓库，所以不传 `-u` 时，已配置 Git 用户名的环境不会再询问。
- `.trellis/scripts/init_developer.py:36-42`：`.developer` 已存在时打印现有身份、成功退出，不覆盖。

## 重初始化与其余提示

- `dist/commands/init.js:637-642`：已有 `.trellis/` 时，只要带了平台开关或 `-u`，就跳过 “already initialized” 菜单，并补这些平台和开发者身份。
- `dist/commands/init.js:786-799` 与模板分支：未带 `--yes` 时，Claude Code 仍可能确认 statusLine，单仓库流程仍可能选择 spec 模板。
- `--yes` 把写入冲突模式设为跳过已有文件（`dist/commands/init.js:878-882`），并跳过 statusLine 与模板询问。

## 对 CCR 的约束

- 交互模式由 CCR 显示五项复选和用户名输入，然后运行不带 `--yes` 的 `trellis init -u <name> <保留的开关>`。
- `-y` 由 CCR 读取 `git config user.name`，固定五项，并追加 `--yes`。
- Kimi 不进入 CCR 菜单。
