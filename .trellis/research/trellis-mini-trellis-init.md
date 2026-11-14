# trellis 与 mini-trellis 初始化参数、落盘产物与重复执行行为实测摸底

- **所属地图**：[ccr project init 双工作流引擎与结构化样式优化地图](https://github.com/bahayonghang/ccr/issues/180)
- **研究工单**：[trellis 与 mini-trellis 初始化参数、落盘产物与重复执行行为实测摸底](https://github.com/bahayonghang/ccr/issues/181)
- **实测环境**：Windows + PowerShell (`pwsh`) + Node.js `v26.7.0` + Python `3.14.7`
- **一手来源版本与源码路径**：
  - `trellis`（`@mindfoldhq/trellis` `0.6.17`）：入口 `C:\home\lyh\.npm-global\trellis.ps1` → `D:\Documents\Code\Agents\Trellis\packages\cli\bin\trellis.js`，TypeScript 源码位于 `D:\Documents\Code\Agents\Trellis\packages\cli\src\`
  - `mini-trellis`（`mini-trellis` `0.3.8`）：入口 `C:\home\lyh\.npm-global\mini-trellis.ps1` → `C:\home\lyh\.npm-global\node_modules\mini-trellis\bin\mini-trellis.js`，ESM 产物位于 `C:\home\lyh\.npm-global\node_modules\mini-trellis\dist\`
  - 隔离实测工作区：`$env:TEMP\ccr-research-181`（全部实测完成后已清理删除）

---

## 核心结论速览（面向后续实现切片的决策基线）

| 决策维度 | `trellis` (`0.6.17`) | `mini-trellis` (`0.3.8`) | 对 `ccr project init` 的直接约束 |
| --- | --- | --- | --- |
| **CCR 现有 5 平台兼容性** | 全部支持：`--claude`、`--codex`、`--cursor`、`--omp`、`--grok`（共支持 22 平台 + 1 弃用别名） | 仅支持 **4 项**：`--claude`、`--codex`、`--cursor`、`--omp`；**不支持 `--grok`**（共支持 10 平台） | 平台枚举与多选菜单必须按所选引擎动态过滤，选 `mini-trellis` 时不得展示或传递 `--grok`（传 `--grok` 退出码 `1`）。 |
| **裸 `-y` 默认平台差异** | 默认勾选 **Claude Code + Cursor**（`.claude/` + `.cursor/`） | 默认仅勾选 **Claude Code**（仅 `.claude/`，`--cursor` 的 `defaultChecked: false`） | `ccr -y project init` 若保持显式传平台 flag，不能依赖下游 CLI 自身默认勾选项，需由 CCR 自身维护各引擎默认平台集。 |
| **显式传 `-u` + 平台但省去 `-y` 的交互行为** | **仍会弹出多达 4 类交互提示**（Monorepo 确认、Claude `statusLine` 询问、远程模板拉取与选择器、文件冲突询问）；非 TTY 下直接崩溃 `ERR_USE_AFTER_CLOSE`（退出码 `1`） | 全新空目录 **0 交互提示**；仅在真实 TTY 且非 JSON 模板文件（如 `.codex/config.toml`）已存在且内容不同时触发冲突询问；非 TTY 自动降级 `skip`（退出码 `0`） | 交互模式下 CCR 必须保持 `Stdio::inherit()`；若用户希望子 CLI 也不额外追问 `statusLine`/远程模板，需区分交互透传与 `-y` 行为。 |
| **根目录落盘边界** | 写 `AGENTS.md`（整文件 `writeFile`，`-y` 下已存在则跳过）和 `.gitattributes`；**绝不创建根目录 `CLAUDE.md` 或根目录 `.gitignore`** | 合并 `AGENTS.md` 的 `<!-- TRELLIS:START -->...<!-- TRELLIS:END -->` 托管块（保留块外文本）和 `.gitattributes`；**绝不创建根目录 `CLAUDE.md` 或根目录 `.gitignore`** | 根目录 `.gitignore` 的宿主目录忽略规则完全由 CCR 自身负责维护，不能指望任一引擎生成。 |
| **最低工作流产物（Postcondition）** | `.trellis/workflow.md` + `.trellis/scripts/task.py` | `.trellis/scripts/add_session.py` + `.trellis/.mini-trellis-manifest.json` + `AGENTS.md`（含 `Managed by mini-trellis.`） | `validate_trellis_workflow` 必须改为按引擎分支校验，否则 `mini-trellis` 永远因缺少 `workflow.md`/`task.py` 报退出码 `51`。 |
| **同引擎重跑幂等性陷阱** | 不带 `-s`/`-f` 时走 `handleReinit`（完全保留自定义文件与 297 条哈希）；**一旦传 `-s` 会绕过 `handleReinit` 重置哈希清单**（哈希从 297 暴跌至 41） | 不带 `-s`/`-f` 时走 `handleReinit`，保留 `spec/`、`research/`、`journal`、自定义 JSON 键与 `AGENTS.md` 块外内容，仅刷新块内说明与受管资产 | 同引擎重跑时 **严禁向 `trellis init` 额外注入 `-s`/`--skip-existing`**，传 `-u <dev> <platforms> [-y]` 即可安全触发 `handleReinit`。 |
| **跨引擎混跑风险** | 在已有 `mini-trellis` 目录跑 `trellis init -y`：**退出码为 `0` 但产生严重混合损坏**（`AGENTS.md` 与宿主机钩子配置被 `skip` 保留为 `mini-trellis` 版本，两个 `doctor` 均报 `error` 退出码 `1`） | 在已有 `trellis` 目录跑 `mini-trellis init`（即使加 `-f -y`）：**硬报错退出码 `1`**（提示先运行 `mini-trellis migrate --dry-run`） | **CCR 必须在调用子进程前做前置工作区指纹检测**：一旦检测到跨引擎或混杂状态，必须在 CCR 层主动拦截并给出清晰迁移/清理指引，绝不能直接放行 `trellis init`。 |

---

## 1. CLI 版本、`init` 完整参数矩阵与互传不支持参数实测

### 1.1 版本号与源码入口核对

- **`trellis --version`** → `0.6.17`（定义于 `D:\Documents\Code\Agents\Trellis\packages\cli\package.json:3`，CLI 注册于 `src/cli/index.ts:73-158`）
- **`mini-trellis --version`** → `0.3.8`（定义于 `C:\home\lyh\.npm-global\node_modules\mini-trellis\package.json:3`，CLI 注册于 `dist/cli/index.js:18-48`）

### 1.2 平台参数完整对比表

依据 `trellis` 的 `src/cli/index.ts:78-100` + `src/types/ai-tools.ts:181-639` 与 `mini-trellis` 的 `dist/cli/index.js:22-32` + `dist/types/ai-tools.js:4-177`：

| CLI Flag | 平台名称 | `trellis init` (`0.6.17`) | `mini-trellis init` (`0.3.8`) | 裸 `-y` 默认勾选差异 | 落盘宿主目录 |
| --- | --- | --- | --- | --- | --- |
| `--claude` | Claude Code | ✅ 支持 | ✅ 支持 | **两者均为 `true`** | `.claude/` |
| `--cursor` | Cursor | ✅ 支持 | ✅ 支持 | `trellis`: **`true`**；`mini-trellis`: **`false`** | `.cursor/` |
| `--codex` | Codex | ✅ 支持 | ✅ 支持 | 均为 `false` | `.codex/` + `.agents/skills/` |
| `--omp` | Oh My Pi (OMP) | ✅ 支持 | ✅ 支持 | 均为 `false` | `.omp/` |
| `--grok` | Grok Build | ✅ 支持 | ❌ **不支持** | 均为 `false` | `.grok/` |
| `--opencode` | OpenCode | ✅ 支持 | ✅ 支持 | 均为 `false` | `.opencode/` |
| `--pi` | Pi Agent | ✅ 支持 | ✅ 支持 | 均为 `false` | `.pi/` |
| `--dsh` | DeepSeek Harness | ✅ 支持 | ✅ 支持 | 均为 `false` | `.dsh/` |
| `--kiro` | Kiro | ✅ 支持 | ✅ 支持 | 均为 `false` | `.kiro/` |
| `--zcode` | ZCode | ✅ 支持 | ✅ 支持 | 均为 `false` | `.zcode/`（`trellis` 另写 `.agents/skills/`） |
| `--codebuddy` | CodeBuddy | ✅ 支持 | ✅ 支持 | 均为 `false` | `.codebuddy/` |
| `--kimi` | Kimi Code | ✅ 支持 | ❌ 不支持 | `false` | `.kimi/` + `.agents/skills/` |
| `--gemini` | Gemini CLI | ✅ 支持 | ❌ 不支持 | `false` | `.gemini/` |
| `--antigravity` | Antigravity | ✅ 支持 | ❌ 不支持 | `false` | `.agent/` |
| `--devin` / `--windsurf` | Devin（`--windsurf` 为弃用别名） | ✅ 支持 | ❌ 不支持 | `false` | `.windsurf/` |
| `--kilo` | Kilo CLI | ✅ 支持 | ❌ 不支持 | `false` | `.kilocode/` |
| `--qoder` | Qoder | ✅ 支持 | ❌ 不支持 | `false` | `.qoder/` |
| `--copilot` | GitHub Copilot | ✅ 支持 | ❌ 不支持 | `false` | `.github/` |
| `--droid` | Factory Droid | ✅ 支持 | ❌ 不支持 | `false` | `.factory/` |
| `--reasonix` | Reasonix | ✅ 支持 | ❌ 不支持 | `false` | `.reasonix/` |
| `--trae` | Trae IDE | ✅ 支持 | ❌ 不支持 | `false` | `.trae/` |
| `--snow` | Snow CLI | ✅ 支持 | ❌ 不支持 | `false` | `.snow/` |

**实测验证裸 `-y` 默认落盘差异**（非 Git 空目录）：
- `trellis init -y` → 顶层生成 `[".claude", ".cursor", ".gitattributes", ".trellis", "AGENTS.md"]`（143 个受管模板哈希）
- `mini-trellis init -y` → 顶层仅生成 `[".claude", ".gitattributes", ".trellis", "AGENTS.md"]`

### 1.3 非平台控制参数对比表

依据 `trellis` `src/cli/index.ts:101-135` 与 `mini-trellis` `dist/cli/index.js:24,33-37`：

| 参数 | `trellis init` | `mini-trellis init` | 作用与源码行为 |
| --- | --- | --- | --- |
| `-y, --yes` | ✅ | ✅ | 跳过交互提示并使用默认值，写文件冲突策略设为 `skip`（`init.ts:1158-1163` / `init.js:339-342`） |
| `-u, --user <name>` | ✅ | ✅ | 指定开发者名称并调用 `.trellis/scripts/init_developer.py` |
| `-f, --force` | ✅ | ✅ | 冲突文件强制覆盖（`writeMode = "force"`），且绕过 `handleReinit` 走全量初始化 |
| `-s, --skip-existing` | ✅ | ✅ | 冲突文件直接跳过（`writeMode = "skip"`），且绕过 `handleReinit` 走全量初始化 |
| `-h, --help` | ✅ | ✅ | Commander 内置帮助输出（退出码 `0`） |
| `--with-statusline` | ✅ 独有 | ❌ | 为 Claude Code 安装 Trellis 状态栏脚本 `.claude/hooks/statusline.py` |
| `--monorepo` / `--no-monorepo` | ✅ 独有 | ❌ | 强制开启或跳过多包仓库检测（`init.ts:1271-1308`） |
| `-t, --template <name>` | ✅ 独有 | ❌ | 拉取远程 spec 模板 |
| `--overwrite` / `--append` | ✅ 独有 | ❌ | 远程 spec 模板与现有 `.trellis/spec` 冲突时的写入策略 |
| `-r, --registry <source>` | ✅ 独有 | ❌ | 自定义远程 spec 模板源 |
| `--workflow <id>` / `--workflow-source <source>` | ✅ 独有 | ❌ | 指定 `.trellis/workflow.md` 工作流模板及自定义源 |
| `--opencode-version <1\|2>` | ❌ | ✅ 独有 | 指定 OpenCode 插件协议版本（新项目默认 V2；**必须配合 `--opencode` 使用**，`init.js:315-317`） |

### 1.4 互传不支持参数与非法参数组合实测表

两个 CLI 均基于 `commander` 且未对 `init` 开启 `.allowUnknownOption(true)`，传入未知选项或非法组合时真实退出码与 `stderr` 如下：

| 实测命令 | 退出码 | 实测 `stderr` 原文 |
| --- | --- | --- |
| `mini-trellis init --grok` | `1` | `error: unknown option '--grok'` |
| `mini-trellis init --kimi` | `1` | `error: unknown option '--kimi'` + `(Did you mean --kiro?)` |
| `mini-trellis init --with-statusline` | `1` | `error: unknown option '--with-statusline'` |
| `mini-trellis init --monorepo` | `1` | `error: unknown option '--monorepo'` |
| `mini-trellis init -t electron-fullstack` | `1` | `error: unknown option '-t'` |
| `mini-trellis init --workflow tdd` | `1` | `error: unknown option '--workflow'` |
| `mini-trellis init --opencode-version 2 -y`（未传 `--opencode`） | `1` | `Error: --opencode-version requires --opencode during init` |
| `mini-trellis init --opencode --opencode-version 3 -y` | `1` | `Error: --opencode-version must be 1 or 2 (received "3")` |
| `trellis init --opencode-version 2` | `1` | `error: unknown option '--opencode-version'` |

---

## 2. 交互（不带 `-y`）vs 非交互（带 `-y`）及非 TTY / 环境变量实测

### 2.1 显式传入 `-u test-dev` + 平台参数但**不带 `-y`** 时的行为差异

#### A. `trellis init -u test-dev --claude --codex --cursor --omp --grok`（无 `-y`，全新目录）
源码核对（`D:\Documents\Code\Agents\Trellis\packages\cli\src\commands\init.ts`）表明，即使调用方已显式提供 `-u` 与平台参数，省去 `-y` 仍会按顺序触发 **4 类交互或网络动作**：
1. **Monorepo 确认与每包 spec 来源询问**（`init.ts:1303-1412`）：只要 `detectMonorepo(cwd)` 命中 `Cargo.toml [workspace]`、`pnpm-workspace.yaml`、`package.json workspaces`、`go.work`、`pyproject.toml [tool.uv.workspace]`、`.gitmodules` 或 ≥2 个子级 `.git`（例如在 CCR 这类 Rust Workspace 仓库中初始化！），且未传 `--no-monorepo`，就会调用 `inquirer.prompt` 询问 `Enable monorepo mode?` 及每个包的 spec 来源。
2. **Claude Code `statusLine` 询问**（`maybePromptStatuslineOptIn`，`init.ts:991-1008, 1483`）：只要平台包含 `--claude` 且未显式传 `--with-statusline`，必弹 `Install Trellis statusLine for Claude Code? (status bar: model, context, branch, rate limits) (y/N)`。
3. **远程 spec 模板索引网络拉取 + 列表选择**（`init.ts:1522-1752`）：单仓模式下未传 `--template` 时，主动请求 `https://raw.githubusercontent.com/mindfold-ai/marketplace/main/index.json`，向 `stdout` 输出 `Fetching available templates...` 及带 `\r\x1b[2K` 的动态计时器 `Loading... 0s/5s`，拉取成功后弹出 `Select a spec template:` 列表（即便前面不选 `--claude`，实测 `trellis init -u test-dev --codex` 也会在此处触发交互）。
4. **已存在文件冲突询问**（`init.ts:1151`，`src/utils/file-writer.ts:178-192`）：`writeMode = "ask"` 在真实 TTY 下对内容不同的已存在文件逐个弹出 6 选 1 菜单（`Skip` / `Overwrite` / `Append` / `*-all`）；仅在 `!process.stdin.isTTY` 时降级为 `skip`（`file-writer.ts:145-148`）。

**非 TTY 管道实测结果**：
- 运行 `trellis init -u test-dev --claude --codex --cursor --omp --grok`（stdin 空管道）：在 `Install Trellis statusLine for Claude Code?` 处向 `stdout` 写入 ANSI 控制序列（`\x1b[32m?...`），随后因 stdin EOF 抛出未捕获异常，**退出码 `1`**，目录未创建任何文件：
  ```text
  Error [ERR_USE_AFTER_CLOSE]: readline was closed
      at Interface.pause (node:internal/readline/interface:573:13)
  ```
- 运行 `trellis init -u test-dev --codex`（规避 `--claude` 的 `statusLine` 询问，stdin 空管道）：发起 GitHub Raw 网络请求后在 `Select a spec template:` 处同样抛出 `ERR_USE_AFTER_CLOSE`，**退出码 `1`**，且 `stdout` 含 `\r\x1b[2K` 与光标隐藏码 `\x1b[?25l`。

#### B. `mini-trellis init -u test-dev --claude --codex --cursor --omp`（无 `-y`，全新目录）
源码核对（`C:\home\lyh\.npm-global\node_modules\mini-trellis\dist\commands\init.js:314-458`）：
- `mini-trellis` **无** Monorepo 检测、**无** `statusLine` 询问、**无** 远程模板网络请求与选择器。
- 当传入 `-u test-dev` 且显式指定至少一个平台 flag 时：
  - **全新空目录**：无论 TTY 还是非 TTY，均 **0 次交互提示**，直接完成全部写入，**退出码 `0`**，且与带 `-y` 相比仅少打印一行 `Mode: Non-interactive (skip existing files)`。
  - **存在预置冲突文件时**：`AGENTS.md` 始终走托管块合并（不询问），`.claude/settings.json` / `.codex/hooks.json` / `.cursor/hooks.json` 等 JSON 宿主配置始终走 `mergeJsonFile` 深度合并（`dist/configurators/shared.js:413-441`，不询问）；但非 JSON 模板文件（如 `.codex/config.toml`、`.trellis/config.yaml`、`SKILL.md`、`session-start.py`）走 `writeFile`（`dist/utils/file-writer.js:88-175`）：在**真实 TTY** 下内容不同会弹出 `File "<path>" already exists. What would you like to do?`，而在**非 TTY** 下由 `file-writer.js:114` 自动将 `ask` 降级为 `skip` 并退出码 `0`。

#### C. 裸 `init`（既无 `-u`、无 Git `user.name`、无平台 flag、也无 `-y`）在非 TTY 下的特殊表现
- `trellis init` 与 `mini-trellis init` 在非 Git 空目录且无 `-u` 时，首先调用基于原生 `readline.question("Your name: ")` 的 `askInput`（`init.ts:1228, 2076-2088` / `init.js:374, 153-164`）。当 `stdin` 立即 EOF 时，`readline` 不会兑现 Promise，Node 事件循环直接排空并**以退出码 `0` 静默提前退出**，此时磁盘未落盘任何 `.trellis/` 文件！这也是 `ccr project init` 必须做执行后产物校验（Postcondition Check）的直接原因之一。

### 2.2 非 TTY、`NO_COLOR=1`、`TERM=dumb`、`FORCE_COLOR=1`、`TRELLIS_QUIET=1` 实测矩阵

两个 CLI 均使用 `figlet` 打印多行 ASCII Banner、固定输出 Unicode Emoji（`👤`、`📁`、`📝`、`📌`、`📄`、`📋`/`✓`），并使用 `chalk ^5.3.0` 着色：

| 场景（均带 `-u test-dev <platforms> -y`，管道捕获 stdio） | `trellis` 退出码 | `trellis` ANSI / `stderr` 表现 | `mini-trellis` 退出码 | `mini-trellis` ANSI / `stderr` 表现 |
| --- | --- | --- | --- | --- |
| 默认非 TTY 管道（无颜色环境变量） | `0` | `stdout` 无 ANSI；**`stderr` 输出 1 段 `⚠️  Codex hooks require...` 提示**（无 ANSI，`src/configurators/codex.ts:342-351`） | `0` | `stdout` 无 ANSI；`stderr` 为空 |
| `NO_COLOR=1` | `0` | `stdout`/`stderr` 均无 ANSI；`stderr` 仍有 Codex hooks 提示 | `0` | `stdout` 无 ANSI；`stderr` 为空 |
| `TERM=dumb` | `0` | `stdout`/`stderr` 均无 ANSI（ASCII Banner 与 Emoji 仍原样输出）；`stderr` 仍有 Codex hooks 提示 | `0` | `stdout` 无 ANSI；`stderr` 为空 |
| `FORCE_COLOR=1` | `0` | `stdout` 与 `stderr` **均含 ANSI 颜色码** | `0` | `stdout` **含 ANSI 颜色码**；`stderr` 为空 |
| `FORCE_COLOR=1` + `NO_COLOR=1` 同时存在 | `0` | Node `v26.7.0` 向 `stderr` 额外打印 `Warning: The 'NO_COLOR' env is ignored due to the 'FORCE_COLOR' env being set`，并保留 ANSI 颜色 | `0` | 同左，`stderr` 出现 Node 内置环境变量冲突告警 |
| `TRELLIS_QUIET=1` | `0` | **抑制 `--codex` 在 `stderr` 上的 `⚠️  Codex hooks require...` 告警**（`stderr` 变为空） | `0` | `mini-trellis` 未使用该变量，`stderr` 本身为空 |

**对 `ccr project init` 的输出样式与子进程控制启示**：
- 若交互模式继续使用 `Stdio::inherit()`，子 CLI 的 `figlet` Banner 和步骤日志会直接写向终端；CCR 自身外围阶段标题与分割线应保持简洁清晰，避免与子 CLI Banner 叠床架屋。
- 若未来在 `-y` 模式下考虑捕获子进程输出做结构化折叠，需注意：切勿同时注入 `FORCE_COLOR` 与 `NO_COLOR`；对 `trellis` 可视情况考虑 `NO_COLOR=1`，且不能把 `trellis` 的 `stderr` 非空直接视为失败（`--codex` 成功时也会写 `stderr`，应以进程退出码 + Postcondition 文件为准）。

---

## 3. 全新初始化落盘产物对比、Postcondition Files 与特征指纹

### 3.1 根目录文件对比

在全新空目录分别执行：
- `trellis init -u test-dev --claude --codex --cursor --omp --grok -y`
- `mini-trellis init -u test-dev --claude --codex --cursor --omp -y`

| 根路径 | `trellis` (`0.6.17`) | `mini-trellis` (`0.3.8`) | 关键差异说明 |
| --- | --- | --- | --- |
| `AGENTS.md` | ✅ 创建（整文件写入，含 `<!-- TRELLIS:START -->` + `# Trellis Instructions` + `Managed by Trellis.`） | ✅ 创建/合并（`mergeManagedBlockFile`，含 `<!-- TRELLIS:START -->` + `# mini-trellis` + `Managed by mini-trellis.`） | **两者共用完全相同的起止注释标记 `<!-- TRELLIS:START -->` / `<!-- TRELLIS:END -->`！** 区别在块内标题与尾注，以及 `trellis` 用整文件 `writeFile`（`-y` 下若 `AGENTS.md` 已存在则直接跳过不改！），而 `mini-trellis` 只替换标记块并保留块外用户自定义内容。 |
| `.gitattributes` | ✅ 创建/追加 `journal-*.md merge=union` | ✅ 创建/追加 `journal-*.md merge=union` | 两者行为一致（`ensureGitattributes`）。 |
| `CLAUDE.md` | ❌ **不创建** | ❌ **不创建** | 两个引擎均不在根目录生成 `CLAUDE.md`。 |
| 根 `.gitignore` | ❌ **不创建** | ❌ **不创建** | 两个引擎均只写 `.trellis/.gitignore`（及启用 `--opencode` 时写 `.opencode/.gitignore`），**从不触碰项目根目录 `.gitignore`**。 |

### 3.2 `.trellis/` 目录完整产物对比

| `.trellis/` 子路径 | `trellis` | `mini-trellis` | 用途与归属 |
| --- | --- | --- | --- |
| `.trellis/.developer` | ✅ | ✅ | 当前开发者身份记录（传入 `-u` 或检测到 Git 用户名时由 `init_developer.py` 生成） |
| `.trellis/.gitignore` | ✅ | ✅ | 忽略 `.developer`、`.current-task`、`tasks/` 或 `.runtime/` 等本地状态 |
| `.trellis/config.yaml` | ✅ | ✅ | 同路径不同模板（`trellis` 为工作流/会话配置；`mini-trellis` 为 memory 配置） |
| `.trellis/workspace/index.md` + `<dev>/{index.md, journal-1.md}` | ✅ | ✅ | 开发者会话日志目录 |
| `.trellis/scripts/{__init__.py, add_session.py, get_context.py, init_developer.py}` | ✅ | ✅ | 同名入口脚本，内部实现各自独立 |
| `.trellis/scripts/common/{__init__, active_task, config, developer, git, git_context, io, log, packages_context, paths, safe_commit, session_context, tasks, trellis_config, types, workflow_phase}.py` | ✅ | ✅ | 同名公共 Python 模块 |
| **`.trellis/workflow.md`** | ✅ **独有** | ❌ 无 | `trellis` 四阶段工作流核心指南（`trellis doctor` 必检项） |
| **`.trellis/scripts/task.py`** | ✅ **独有** | ❌ 无 | `trellis` 任务状态机 CLI（现有 CCR 必检项） |
| **`.trellis/scripts/{get_developer.py, hooks/linear_sync.py}`** 及 `common/{cli_adapter, task_context, task_queue, task_store, task_utils, task_verification, verification}.py` | ✅ **独有** | ❌ 无 | `trellis` 任务与子代理支撑模块 |
| **`.trellis/.version`** | ✅ **独有** | ❌ 无 | 记录 `0.6.17`，供 `trellis update` 判断升级 |
| **`.trellis/.template-hashes.json`** | ✅ **独有** | ❌ 无 | `{"__version": 2, "hashes": {...}}`（5 平台初始化共记录 297 个 SHA-256 哈希） |
| **`.trellis/agents/{check.md, implement.md}`** | ✅ **独有** | ❌ 无 | 平台无关工作流角色提示词 |
| **`.trellis/tasks/00-bootstrap-guidelines/{prd.md, task.json}`** | ✅ **独有** | ❌ 无 | 首次带开发者初始化时自动创建的引导任务 |
| **`.trellis/spec/{backend, frontend}/*.md`** + `spec/guides/{code-reuse-thinking-guide, cross-layer-thinking-guide}.md` | ✅ **独有** | ❌ 无 | `trellis` 按项目类型生成的默认前后端规范骨架 |
| **`.trellis/.mini-trellis-manifest.json`** | ❌ 无 | ✅ **独有** | `{"version": 1, "files": {...}}`（4 平台初始化共记录 52 个受管模板哈希，`dist/utils/managed-manifest.js:5`） |
| **`.trellis/spec/guides/mini-memory.md`** | ❌ 无 | ✅ **独有** | `mini-trellis` 记忆层契约说明（配合 `spec/guides/index.md`） |
| **`.trellis/research/README.md` + `research/archive/`** | ❌ 默认不建 | ✅ 默认创建 | 主题研究收件箱（注意：`trellis` 项目用户在日常使用或迁移后也可能自建 `.trellis/research/`，故不能单靠该目录断定为 `mini-trellis`） |

### 3.3 宿主平台目录产物清单（CCR 相关平台及全平台概览）

| 宿主目录 | `trellis` (`--claude --codex --cursor --omp --grok`) | `mini-trellis` (`--claude --codex --cursor --omp`) |
| --- | --- | --- |
| **`.claude/`** | **54 个文件**：`settings.json`、`agents/trellis-{check,implement,research}.md`、`commands/trellis/{continue,finish-work}.md`、`hooks/{session-start,inject-subagent-context,inject-workflow-state,trellis_hook_input,trellis_hook_paths}.py`、`skills/trellis-{before-dev,brainstorm,break-loop,channel,check,meta,session-insight,spec-bootstrap,update-spec}/` | **8 个文件**：`settings.json`、`commands/mini-trellis/remember.md`、`hooks/session-start.py`、`skills/mini-trellis-{session-insight,update-spec}/` |
| **`.codex/`** | **11 个文件**：`config.toml`、`hooks.json`、`agents/trellis-{check,implement,research}.toml`、`hooks/{session-start,inject-subagent-context,inject-workflow-state,trellis_hook_input,trellis_hook_paths}.py`、`skills/.system/`（如有） | **3 个文件**：`config.toml`、`hooks.json`、`hooks/session-start.py` |
| **`.agents/`** | **46 个文件**（由 `--codex`/`--kimi`/`--zcode` 触发）：`skills/trellis-{before-dev,brainstorm,break-loop,channel,check,continue,finish-work,meta,session-insight,spec-bootstrap,start,update-spec}/`（共 12 个技能目录） | **6 个文件**（由 `--codex` 触发）：`skills/mini-trellis-{remember,session-insight,update-spec}/`（共 3 个技能目录） |
| **`.cursor/`** | **54 个文件**：`hooks.json`、`agents/trellis-*.md`、`commands/trellis-{continue,finish-work}.md`、`hooks/{session-start,inject-shell-session-context,inject-subagent-context,trellis_hook_*}.py`、`skills/trellis-*/`（9 个技能） | **8 个文件**：`hooks.json`、`hooks/session-start.py`、`skills/mini-trellis-{remember,session-insight,update-spec}/` |
| **`.omp/`** | **49 个文件**：`agents/trellis-*.md`、`commands/trellis-{continue,finish-work}.md`、`extensions/trellis/index.ts`、`skills/trellis-*/`（9 个技能） | **7 个文件**：`extensions/mini-trellis/index.ts`、`prompts/mini-trellis-remember.md`、`skills/mini-trellis-{session-insight,update-spec}/` |
| **`.grok/`** | **49 个文件**：`agents/trellis-*.md`、`commands/trellis-{start,continue,finish-work}.md`、`skills/trellis-*/`（9 个技能） | **不支持（0 个文件）** |

> 补充：实测 `mini-trellis` 开启全部 10 个支持平台时，其余 6 个平台的宿主目录分别为 `.opencode/`（10 文件，含 `.gitignore`、`package.json`、`commands/mini-trellis/remember.md`、`lib/trellis-context.js`、`plugins/mini-trellis/index.js`、`skills/`）、`.pi/`（8 文件）、`.dsh/`（6 文件）、`.kiro/`（8 文件）、`.zcode/`（8 文件）、`.codebuddy/`（8 文件）。若 CCR 未来仅扩展双引擎而不扩充新平台，现有 7 条 `.gitignore` 规则（`.agents/`、`.claude/`、`.codex/`、`.cursor/`、`.omp/`、`.grok/`、`ref/`）对 `mini-trellis` 的 4 个支持平台完全覆盖（可选不为 `mini-trellis` 写入 `.grok/`，或保持统一 Agent 忽略规则集）。

### 3.4 推荐的最低工作流产物（Postcondition Files）与四态特征指纹（Fingerprint）

#### A. 执行后最低工作流产物校验（Postcondition Files）
对齐两个 CLI 自身 `doctor` 的核心健康判据（`trellis` `src/commands/doctor.ts:584-632`；`mini-trellis` `dist/commands/doctor.js:134-154`）：
- **`trellis` 模式**：
  1. `.trellis/workflow.md`（存在且为普通文件）
  2. `.trellis/scripts/task.py`（存在且为普通文件）
  > 与 CCR 现有 `validate_trellis_workflow` 完全兼容，无需破坏既有 `trellis` 契约。
- **`mini-trellis` 模式**：
  1. `.trellis/scripts/add_session.py`（存在且为普通文件，对应 `mini-trellis doctor` 的 `memory-layer` 检查）
  2. `.trellis/.mini-trellis-manifest.json`（存在且为普通文件，确认 `updateManagedManifest` 已走完，排除半截中断）
  3. `AGENTS.md`（存在且 `<!-- TRELLIS:START -->...<!-- TRELLIS:END -->` 块内包含 `Managed by mini-trellis.`，排除误把 `trellis` 旧块当作 `mini-trellis` 就绪）

#### B. 无歧义四态特征指纹判定（Engine State Fingerprint）
为前置识别当前项目处于何种引擎状态，定义两组互斥核心指纹集合：
- **`TrellisMarkers`（命中任一即为真）**：
  - 存在 `.trellis/workflow.md`
  - 存在 `.trellis/scripts/task.py`
  - 存在 `.trellis/.template-hashes.json` 或 `.trellis/.version`
  - `AGENTS.md` 的 `<!-- TRELLIS:START -->...<!-- TRELLIS:END -->` 块内包含 `Managed by Trellis.`（或存在受管块但不含 `Managed by mini-trellis.`，与 `migrate.js:225-237` 的 `hasLegacyManagedBlock` 一致）
- **`MiniTrellisMarkers`（命中任一即为真）**：
  - 存在 `.trellis/.mini-trellis-manifest.json`
  - 存在 `.trellis/spec/guides/mini-memory.md`
  - `AGENTS.md` 的 `<!-- TRELLIS:START -->...<!-- TRELLIS:END -->` 块内包含 `Managed by mini-trellis.`
  - 存在宿主专属入口（如 `.claude/commands/mini-trellis/remember.md`、`.omp/extensions/mini-trellis/index.ts`、`.agents/skills/mini-trellis-remember/SKILL.md`）

| `TrellisMarkers` | `MiniTrellisMarkers` | 判定状态 | `ccr project init` 推荐处理 |
| --- | --- | --- | --- |
| `false` | `false` | **未初始化（Clean / Uninitialized）** | 允许用户自由选择 `trellis` 或 `mini-trellis`（即使存在空的 `.trellis/` 目录也不误判）。 |
| `true` | `false` | **已初始化 `trellis`** | 同引擎重跑安全放行（不加 `-s`）；若用户请求 `mini-trellis`，前置报错拦截并提示迁移风险。 |
| `false` | `true` | **已初始化 `mini-trellis`** | 同引擎重跑安全放行；若用户请求 `trellis`，**必须前置报错拦截**（防止 `trellis init` 静默产出损坏混杂态）。 |
| `true` | `true` | **双引擎混杂损坏态（Mixed / Corrupted）** | 立即报错中止，列出命中的双侧指纹文件，提示用户先运行 `doctor` / 清理或迁移，拒绝继续叠加写入。 |

---

## 4. 同引擎二次运行、跨引擎混跑与 `mini-trellis migrate` 实测

### 4.1 `trellis` 同引擎二次运行矩阵（含自定义文件保护与 `-s` 哈希陷阱）

在已完成 5 平台初始化的 `trellis` 目录中，预先向以下 6 处写入自定义内容：
`.trellis/spec/backend/index.md`、`.trellis/workspace/test-dev/journal-1.md`、`.trellis/workflow.md`、`AGENTS.md`（块内 + 块外）、`.claude/settings.json`（新增 `"env": {"CUSTOM_ENV": "1"}`）、`.codex/config.toml`（追加注释），初始 `.template-hashes.json` 共 **297** 条（其中非 `.trellis/` 平台/根文件 **263** 条）。实测 7 种重跑组合结果如下：

| 组合 | 命令 | 退出码 | 走的源码分支 | 自定义文件保留情况 | `.template-hashes.json` 条目变化 |
| --- | --- | --- | --- | --- | --- |
| **T1** | `trellis init -u test-dev --claude --codex --cursor --omp --grok -y` | `0` | `handleReinit`（`init.ts:1202-1215`） | **6 处全部保留**（5 个已配置平台全部打印 `already configured, skipping`） | 保持 **297**（平台哈希 263 条完整保留） |
| **T2** | `trellis init -u test-dev --claude --codex --cursor --omp --grok`（无 `-y`，非 TTY） | `0` | `handleReinit` | **6 处全部保留**（因 5 个平台均已存在，待新增列表为空，不触发 `statusLine` 询问） | 保持 **297** |
| **T2b** | 在已有 5 平台上追加 `--kimi`（无 `-y`，非 TTY） | `0` | `handleReinit`（增量添加平台） | **6 处全部保留**，仅新增 `.kimi/` 与对应哈希 | `merge: true` 合并增至 **306** |
| **T2c** | 原先仅 `--codex`，重跑追加未配置的 `--claude`（无 `-y`，非 TTY） | **`1`** | `handleReinit` → `maybePromptStatuslineOptIn` | 因 `--claude` 尚未配置且无 `-y`，触发 `statusLine` 询问导致非 TTY `ERR_USE_AFTER_CLOSE` | 未变更 |
| **T3** | `trellis init -u test-dev --claude --codex --cursor --omp --grok -s -y` | `0` | **绕过 `handleReinit`**（`!options.skipExisting` 为假），走全量 `skip` | 6 处文件内容保留 | ⚠️ **从 297 暴跌至 41（非 `.trellis/` 仅剩 1 条 `.codex/hooks.json`）！** 因 `writtenPaths` 未记录被 skip 的宿主文件且 `initializeHashes` 未开 `merge: true`（`template-hash.ts:364`），直接清空所有已装平台的模板归属哈希！ |
| **T4 / T5** | 裸 `trellis init -y` / 裸 `trellis init`（非 TTY） | `0` / `1` | `handleReinit` 无显式参数分支 | T4 打印 `Already initialized with: ...` 无改动；T5 在菜单处 `ERR_USE_AFTER_CLOSE` | 保持 **297** |
| **T6** | `trellis init -u test-dev --claude --codex --cursor --omp --grok -f -y` | `0` | 全量 `force` 覆盖 | **仅 `journal-1.md` 保留**；`spec/backend/index.md`、`workflow.md`、`AGENTS.md`（含块外）、`.claude/settings.json`（丢失自定义 `env`）、`.codex/config.toml` **全部被覆盖** | 仅重写有差异的文件，哈希降为 **44** |

### 4.2 `mini-trellis` 同引擎二次运行矩阵

在已完成 4 平台初始化的 `mini-trellis` 目录中，预先修改 `.trellis/spec/guides/index.md`、新建 `.trellis/research/custom-topic.md`、修改 `journal-1.md`、`.trellis/config.yaml`、`.codex/config.toml`、`.omp/prompts/mini-trellis-remember.md`、`AGENTS.md`（块内 + 块外）、`.claude/settings.json`（新增 `"env"`），初始 `.mini-trellis-manifest.json` 共 **52** 条。实测结果：

| 组合 | 命令 | 退出码 | 自定义文件保留与更新行为 |
| --- | --- | --- | --- |
| **M1** | `mini-trellis init -u test-dev --claude --codex --cursor --omp -y` | `0` | 走 `handleReinit`（`init.js:206-309`）：显式传入的平台会刷新受管 JSON（`.claude/settings.json` 深度合并，**保留自定义 `env`**），但非 JSON 冲突文件因 `writeMode=skip` 自动跳过（**保留 `.codex/config.toml`、`.omp/prompts/*`、`spec/`、`research/`、`journal`、`config.yaml`**）；`AGENTS.md` **块外自定义内容完整保留，块内说明同步刷新为标准受管块**；manifest 保持 52 条。 |
| **M2** | 同上但**不带 `-y`**（非 TTY） | `0` | 行为与 M1 完全一致（非 TTY 下 `file-writer.js:114` 将 `ask` 自动降级为 `skip`）。在真实 TTY 下则会对被改过的 `.codex/config.toml` 与 `.omp/prompts/mini-trellis-remember.md` 弹出冲突询问。 |
| **M3** | 同上加 `-s -y` | `0` | 绕过 `handleReinit` 走全量 `skip`，保留结果与 M1 一致，manifest 仍为 52 条（因 `mini-trellis` 的 `updateManagedManifest` 基于静态模板表而非当次写入记录，`init.js:184-198`）。 |
| **M4 / M5** | 裸 `mini-trellis init -y` / 裸 `mini-trellis init`（非 TTY） | `0` / `1` | M4 打印 `Already initialized with: Claude Code, Codex, Oh My Pi (OMP), Cursor` 无改动；M5 非 TTY 抛出 `ERR_USE_AFTER_CLOSE`。 |
| **M6** | 同上加 `-f -y` | `0` | **保留 `research/custom-topic.md`、`journal-1.md`、`AGENTS.md` 块外内容、`.claude/settings.json` 自定义 `env`**；覆盖 `spec/guides/index.md`、`config.yaml`、`.codex/config.toml`、`.omp/prompts/mini-trellis-remember.md` 与 `AGENTS.md` 块内内容。 |

### 4.3 跨引擎混跑与 `mini-trellis migrate` 实测

#### 方向 A：先 `trellis init`，再跑 `mini-trellis init`
- 实测命令：`mini-trellis init -u test-dev --claude --codex --cursor --omp -y` 以及追加 `-f -y`
- 实测结果：**均立即失败，退出码 `1`**，磁盘零修改，`stderr` 输出：
  ```text
  Error: Trellis workflow detected; run mini-trellis migrate --dry-run before initializing.
  ```
- 源码依据：`dist/commands/init.js:323-325` 在任何文件写入前调用 `needsTrellisMigration(cwd)`（`dist/commands/migrate.js:238-242`），只要发现 `collectTrellisResidue`、`legacyHostPlatforms` 或非 `mini-trellis` 的 `AGENTS.md` 受管块即抛错中止。

#### 方向 B：先 `mini-trellis init`，再跑 `trellis init`（⚠️ 高危静默损坏！）
- 实测命令：`trellis init -u test-dev --claude --codex --cursor --omp --grok -y`
- 实测结果：**进程退出码竟为 `0`**，但生成了一个**双引擎严重混杂且两端不可用**的工作区：
  1. **为何未走 `handleReinit`**：`trellis` 在 `init.ts:1197-1208` 检查 `.trellis/tasks` 是否非空；由于 `mini-trellis` 不创建 `tasks/`，`tasksEmptyEarly === true`，`trellis` 误判为“上次初始化中途夭折”，强行进入全量初始化且 `writeMode = "skip"`！
  2. **核心文件错位跳过**：因 `writeMode = "skip"`，`trellis` 跳过了已存在的 `AGENTS.md`（保留 `# mini-trellis` 说明！）、`.trellis/config.yaml`、`.trellis/scripts/{add_session,get_context,init_developer}.py`、`.claude/settings.json`、`.claude/hooks/session-start.py`、`.cursor/hooks.json`、`.codex/hooks/session-start.py`、`.codex/config.toml`，同时又写入了 `.trellis/workflow.md`、`task.py`、`.template-hashes.json`、`.version` 以及全部 `trellis-*` 命令/技能/代理（与 `mini-trellis-*` 并存）！
  3. **两端健康检查双双报错（退出码均为 `1`）**：
     - 运行 `trellis doctor` → **退出码 `1`**，报 7 项 `error [configured]`（`.claude/settings.json`、`.cursor/hooks.json`、`.codex/hooks.json` 缺少 `PreToolUse` / `UserPromptSubmit` / `SubagentStart` / `beforeShellExecution` 等 Trellis 钩子注册）。
     - 运行 `mini-trellis doctor` → **退出码 `1`**，报 `✗ legacy-trellis: Trellis workflow detected; run mini-trellis migrate --dry-run`。
     - 再次运行 `mini-trellis init` → **退出码 `1`**，同样被 `legacy-trellis` 拦截。
  4. **即使改用 `trellis init ... -f -y`（强制覆盖）**：仍不会删除 `.trellis/.mini-trellis-manifest.json`、`.claude/commands/mini-trellis/remember.md`、`.claude/skills/mini-trellis-*`、`.omp/extensions/mini-trellis/` 等专属资产，且 `.codex/hooks.json` 因受保护合并逻辑仍使 `trellis doctor` 与 `mini-trellis doctor` 双双返回退出码 `1`。

#### `mini-trellis migrate` 参数、源码逻辑与 5 平台实测边界
- **`mini-trellis migrate --help` 参数表**（`dist/cli/index.js:50-70`）：
  - `--opencode-version <1|2>`：迁移 OpenCode 资产时指定插件协议版本
  - `-y, --yes`：跳过两次交互确认，任务文件默认归档至 `.trellis/research/<topic>/legacy/`
  - `--dry-run`：仅预览删除、替换与任务收敛计划，不修改磁盘（退出码 `0`）
  - 非 TTY 且不传 `-y`/`--dry-run` 时，打印计划后拒绝执行并向 `stderr` 输出 `Refusing to prompt for confirmation in a non-interactive shell. Pass --yes/-y to confirm or --dry-run to preview.`，**退出码 `1`**（`migrate.js:506-510`）。
- **核心迁移机制**（`migrate.js:429-567`）：删除 `SCAN_RULES` + `FIXED_DELETIONS` + `.trellis/.version` → 强制重写 `.trellis/scripts/` 与 `.trellis/config.yaml` → 替换 `AGENTS.md` 受管块（保留块外文本） → 重配检测到的宿主平台 → 将 `.trellis/tasks/<topic>/` 收敛至 `.trellis/research/<topic>/`（`prd.md`、`task.json` 默认移入 `legacy/` 子目录；保留 `.trellis/spec/` 与 `.trellis/workspace/`）。
- **实测发现的两个关键上游局限（在 5 平台 `trellis 0.6.17` 目录执行 `mini-trellis migrate -y` 后验证）**：
  1. **Codex 钩子残留导致迁移后 `mini-trellis doctor` 与后续 `init` 仍报 `legacy-trellis`（退出码 `1`）**：`trellis 0.6.17` 在 `.codex/hooks.json` 中使用 `python -X utf8 -c "import base64;exec(...)" .codex/hooks/inject-workflow-state.py` 包装命令；而 `mini-trellis 0.3.8` 的 `commandMatchesDeletedPath`（`dist/utils/uninstall-scrubbers.js:32-40`）遇到 `-c` 标志直接返回 `false`，未能剥离该条目。迁移结束后 `.codex/hooks.json` 仍含 `inject-` 字符串，触发 `legacyHostPlatforms` 持续命中 `"codex"`！
  2. **Cursor / OMP / Grok 残留文件未清理**：`migrate.js:48-94` 的 `SCAN_RULES` 未纳入 `.cursor/`、`.omp/`、`.grok/` 和 `.trellis/agents/`，迁移后实测残留 `.cursor/` 下 52 个 `trellis-*`/`inject-*` 文件、`.omp/` 下 49 个 `trellis-*` 文件（且因 `DETECT_FILES.omp` 未命中而不自动装入 `mini-trellis` 的 OMP 资产）、`.grok/` 下 49 个文件以及 `.trellis/agents/{check,implement}.md`。
  - **结论**：`ccr project init` **绝不应自动静默代跑 `mini-trellis migrate`**；检测到已有 `trellis` 时应中止并提示用户手工审阅迁移计划及上述宿主目录残留。

---

## 5. 本仓内 `ccr project init` 耦合点全量盘点（文件与行号）

以下为当前仓库中所有绑定了单引擎 `trellis`、固定五平台名、固定命令参数或固定输出文案/断言的位置：

### 5.1 Rust CLI 参数定义、路由与帮助配置（`crates/ccr-cli/`）

1. **[crates/ccr-cli/src/cli/subcommands/project.rs](D:/Documents/Code/Github/ccr/crates/ccr-cli/src/cli/subcommands/project.rs)**
   - `L5-12`：`ProjectAction::Init` 文档注释硬编码 `Trellis platforms`、固定五平台名（`Claude Code, Codex, Cursor, Oh My Pi, and Grok`）、`trellis init`、`--yes` 以及 7 条 `.gitignore` 规则；且 `Init` 变体目前无任何子命令参数（如工作流引擎选择参数或平台覆盖参数）。
2. **[crates/ccr-cli/src/cli/definitions.rs](D:/Documents/Code/Github/ccr/crates/ccr-cli/src/cli/definitions.rs)**
   - `L201-210`：`Commands::Project` 顶层注释硬编码 `初始化当前项目的 Git、Trellis 工作流与本地忽略规则`、默认五平台与 `ccr -y project init` 说明。
   - `L594-608`：单测 `project_init_and_legacy_init_parse_as_distinct_commands` 断言 `ProjectAction::Init` 的解析结构。
3. **[crates/ccr-cli/src/cli/dispatch.rs](D:/Documents/Code/Github/ccr/crates/ccr-cli/src/cli/dispatch.rs)**
   - `L158`：`ProjectAction::Init => crate::commands::project_init_command(auto_yes)`，仅透传全局 `auto_yes: bool`。
4. **[crates/ccr-cli/src/cli/help_config.rs](D:/Documents/Code/Github/ccr/crates/ccr-cli/src/cli/help_config.rs)**
   - `L33-36`：`ROOT_AFTER_LONG_HELP` 示例硬编码 `使用 Git 用户名和默认五项: ccr -y project init`。
   - `L75-99`：`PROJECT_LONG_ABOUT` 与 `PROJECT_AFTER_LONG_HELP` 硬编码 `Trellis 工作流`、`trellis` 可执行名、默认五平台列表、`trellis init --yes`、7 条 `.gitignore` 规则。
   - `L101-116`：`PROJECT_INIT_LONG_ABOUT` 与 `PROJECT_INIT_AFTER_LONG_HELP` 硬编码四阶段说明、`trellis init`、默认五平台列表与 7 条忽略规则。
   - `L282-293`：`configure_project_command` / `configure_project_init_command` 将上述静态常量注入 `clap::Command`。

### 5.2 核心命令实现与内联单元测试（`crates/ccr-cli/src/commands/project/init.rs`）

**[crates/ccr-cli/src/commands/project/init.rs](D:/Documents/Code/Github/ccr/crates/ccr-cli/src/commands/project/init.rs)** 全文件围绕单一 `trellis` 构建：
- `L11-13`：`PROJECT_IGNORE_RULES` 固定 7 项 `[".agents/", ".claude/", ".codex/", ".cursor/", ".omp/", ".grok/", "ref/"]`。
- `L14-20`：交互提示常量 `DEVELOPER_HINT = "用于 Trellis 工作区目录"`、`EMPTY_PLATFORM_SELECTION_MESSAGE` 等。
- `L22-58`：`enum TrellisPlatform`（`Claude`, `Codex`, `Cursor`, `Omp`, `Grok`）、`TrellisPlatform::ALL`、`label()` 与 `flag()`。
- `L60-64`：`struct TrellisInitRequest`（无工作流引擎字段）。
- `L66-89`：`pub fn project_init_command(auto_yes: bool)`：使用 `ColorOutput::title("初始化当前项目")`、`key_value`、`step("1/4 Git 仓库")`、`step("2/4 Trellis 工作流")`、`step("3/4 .gitignore 规则")`、`step("4/4 ref/ 目录")`、`separator()` 与最终汇总 `ColorOutput::success("Git、Trellis、Agent 忽略规则和 ref/ 均已就绪")`（对应结构化样式优化切片的直接改造面，样式契约受 `.trellis/spec/ccr-cli/backend/cli-output-presentation.md` 约束）。
- `L145-196`：`resolve_trellis_request`、`prompt_platforms`（`dialoguer::MultiSelect` 固定展示 `TrellisPlatform::ALL`）、`platforms_from_indices`。
- `L221-271`：`run_trellis_init`（写死 `resolve_tool("trellis", "Trellis")`、`Stdio::inherit()`、退出码提示 `Trellis 阶段：trellis init 退出码 ...`）与 `trellis_init_args`（写死 `["init", "-u", dev, ...flags, "--yes"?]`）。
- `L279-296`：`validate_trellis_workflow` 硬编码检查 `.trellis/workflow.md` 与 `.trellis/scripts/task.py`，缺失时报错 `Trellis 阶段：缺少最低工作流文件：<missing>`（返回 `CcrError::ValidationError` → 退出码 `51`）。
- `L437-611`：15 个内联单测锁死固定参数顺序 `["init", "-u", ..., "--claude", "--codex", "--cursor", "--omp", "--grok", "--yes"]`、7 条 `.gitignore` 规则、`gitignore_ready_message` 文案以及 `.trellis/workflow.md` + `.trellis/scripts/task.py` 校验逻辑。

### 5.3 CLI 集成测试与帮助快照测试（`crates/ccr/tests/commands/`）

1. **[crates/ccr/tests/commands/project_init.rs](D:/Documents/Code/Github/ccr/crates/ccr/tests/commands/project_init.rs)**
   - `L9-14`：顶部常量锁死 `TRELLIS_ARGS = "init -u fake-git-user --claude --codex --cursor --omp --grok --yes"`、7 项 `GITIGNORE_RULES`、`SKIP_GITIGNORE_WRITE = "跳过写入：7 条规则均已存在"`、`READY_SUMMARY = "Git、Trellis、Agent 忽略规则和 ref/ 均已就绪"`。
   - `L56-63` & `L617-640`：`install_trellis` 与 `fake_trellis_script` 仅伪造 `trellis`（`.cmd`/可执行脚本），成功模式下仅落盘 `.trellis/workflow.md` 与 `.trellis/scripts/task.py`。
   - `L105-500`：共 13 个端到端集成测试锁死 `trellis` 调用顺序、失败退出码（缺 `git`/`trellis` → `75`、`trellis` 非零退出 → `90`、缺 `workflow.md` 或 `task.py` → `51`）及中文输出子串（`"Trellis 阶段"`、`"缺少最低工作流文件"`、`READY_SUMMARY`）。
2. **[crates/ccr/tests/commands/help.rs](D:/Documents/Code/Github/ccr/crates/ccr/tests/commands/help.rs)**
   - `L52-53`：`root_help_includes_guided_tasks` 断言根帮助包含 `"初始化当前项目"` 与 `"project"`。
   - `L219-242`：`help_subcommand_supports_project_init_path` 断言 `ccr project --help` 包含 `"Git"`、`"Trellis"`、`".gitignore"`，且断言 `ccr project init --help` 与 `ccr help project init` 字节级相等并同时包含：`"git init"`、`"trellis init"`、`".agents/"`、`".cursor/"`、`".omp/"`、`".grok/"`、`"--yes"`、`"ref/"`、`"git config user.name"`、`"Claude Code"`、`"Codex"`、`"Cursor"`、`"Oh My Pi"`、`"Grok"`。
   - `L245-251`：`legacy_init_help_remains_user_configuration_command` 断言 `ccr init --help` **不包含** `"trellis init"`。
   - `L254-264`：`bare_project_requires_an_explicit_action` 断言裸 `ccr project` 退出码为 `2`。

### 5.4 文档、文档审计脚本与项目规范文件

1. **中英文命令参考文档**：
   - [docs/reference/commands/project-init.md](D:/Documents/Code/Github/ccr/docs/reference/commands/project-init.md)（`L1-109`）与 [docs/en/reference/commands/project-init.md](D:/Documents/Code/Github/ccr/docs/en/reference/commands/project-init.md)（`L1-109`）：完整描述单引擎 `trellis` 行为、固定五平台表、`-y` 参数透传、7 条 `.gitignore` 规则及 `.trellis/workflow.md` + `.trellis/scripts/task.py` 校验项。
   - [docs/reference/commands/index.md](D:/Documents/Code/Github/ccr/docs/reference/commands/index.md)（`L44-46`, `L79`）与 [docs/en/reference/commands/index.md](D:/Documents/Code/Github/ccr/docs/en/reference/commands/index.md)（`L44-46`, `L79`）：命令总览表将 `ccr project init` 描述为 `初始化项目 Git、Trellis 工作流、忽略规则与 ref/`。
   - [docs/reference/commands/init.md](D:/Documents/Code/Github/ccr/docs/reference/commands/init.md)（`L11`）与 [docs/en/reference/commands/init.md](D:/Documents/Code/Github/ccr/docs/en/reference/commands/init.md)（`L11`）：指向 `./project-init` 的交叉提示（文案通用，仅链接关联）。
2. **[docs/scripts/audit-docs.mjs](D:/Documents/Code/Github/ccr/docs/scripts/audit-docs.mjs)**
   - `L237-269`（特别是 `L240`）：`const commandPageOverrides = new Map([['project', 'project-init']])` 将 `Commands::Project` 映射到 `reference/commands/project-init.md`。
   - `L106-140`：`checkInternalLinks()` 与 `checkLocaleParity()` 强制要求 `docs/reference/commands/project-init.md` 与 `docs/en/reference/commands/project-init.md` 双语文件成对存在且内部链接有效（不检查正文具体平台名字符串）。
3. **项目规范契约（`.trellis/spec/`）**：
   - [.trellis/spec/ccr-cli/backend/backend-guidelines.md](D:/Documents/Code/Github/ccr/.trellis/spec/ccr-cli/backend/backend-guidelines.md) `L97-186`（`## Scenario: Project Workflow Bootstrap`）：明确锁死了单引擎 `project_init_command(auto_yes: bool)` 签名、`.trellis/workflow.md` + `.trellis/scripts/task.py` 后置条件、固定五平台菜单、固定 7 条忽略规则与错误码矩阵，实施双引擎切片后需同步更新该规范章节。
   - [.trellis/spec/ccr-cli/backend/cli-output-presentation.md](D:/Documents/Code/Github/ccr/.trellis/spec/ccr-cli/backend/cli-output-presentation.md) `L1-95`：定义了 `ccr-cli` 的 `ColorOutput` / `NO_COLOR` / TTY 宽度与结构化信息块样式契约，是后续“结构化样式优化”切片的直接规范依据。
4. **无关项排除验证**：
   - [scripts/quality/check_harness_contracts.py](D:/Documents/Code/Github/ccr/scripts/quality/check_harness_contracts.py) `L128` 检查的是 `docs/agents/harnesses.md` 中 CCR 仓库自身贡献者的 `trellis init --skip-existing` 指引，不耦合 `ccr project init` 实现。
