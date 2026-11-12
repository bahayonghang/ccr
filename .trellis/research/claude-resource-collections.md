# Claude 资源集合键（agents / slashCommands / plugins）是否被 Claude Code 读取

- 工单：[#155](https://github.com/bahayonghang/ccr/issues/155)（父图 #138，阻塞 #149）
- 性质：只读核对，没有改代码。官方文档与 schema 抓取于 2026-11-10（文档中出现的最新版本号到 v2.1.307 左右）；仓库代码基于 `origin/dev`（`bb0bd86f`）。
- 一手来源：code.claude.com 官方文档、官方 CHANGELOG、SchemaStore 上的 `claude-code-settings.json`（官方 settings 文档用它做 `$schema` 示例）、本仓库代码与 git 历史。

## 结论

Claude Code 不读取 settings.json 顶层的 `agents`、`slashCommands`、`plugins`，历史上也没有读过。CCR 的 Agents、Slash commands、Plugins 三个页面只是在 settings.json 里维护 CCR 自己的数组，对 Claude Code 的行为没有任何影响。真实机制分别是 `agents/*.md`、`commands/*.md` 加 `skills/<name>/SKILL.md`，以及插件市场安装加 `enabledPlugins`。同一页的规则和输出样式写在 `~/.claude/rules/`、`~/.claude/output-styles/`，与官方的用户级位置一致，写入会生效。但这两类只覆盖用户级，而且没有跟随 `CLAUDE_CONFIG_DIR`。

| 页面条目 | CCR 读写位置 | 判定 | 依据摘要 |
| --- | --- | --- | --- |
| 子代理 Agents | settings.json 顶层 `agents` 数组 | **无效** | 官方子代理是 `agents/*.md`（YAML frontmatter 加正文）。settings 里只有单数 `agent`（指定主线程用哪个代理）。CCR 条目缺官方必填的 `description`，`disabled` 也没有官方对应字段 |
| 斜杠命令 Slash commands | settings.json 顶层 `slashCommands` 数组 | **无效** | 官方自定义命令是 `commands/*.md`，现已并入 skills（`skills/<name>/SKILL.md`）。正文是提示词，不是 CCR `command` 字段那样“要执行的命令” |
| 插件 Plugins | settings.json 顶层 `plugins` 数组 | **无效** | 官方插件从 marketplace 安装：settings 用 `enabledPlugins`（对象 `"name@marketplace": bool`）和 `extraKnownMarketplaces`，磁盘记录在 `~/.claude/plugins/`。CCR 的 `enabled` 开关不会启用或禁用任何插件 |
| 规则 Rules | `~/.claude/rules/**/*.md` | **部分有效** | 与官方用户级位置一致，递归扫描、写入生效。缺项目级 `.claude/rules/`；设置 `CLAUDE_CONFIG_DIR` 时会写错目录 |
| 输出样式 Output styles | `~/.claude/output-styles/*.md` | **部分有效** | 与官方用户级位置一致，写入生效。缺项目级和托管级；同样没有跟随 `CLAUDE_CONFIG_DIR` |

## 1. Claude Code 是否读取这三个键

**官方设置索引里没有。** [settings-reference](https://code.claude.com/docs/en/settings-reference) 的索引表共 249 行，与这三类相关的键只有这些：

- `agent`：每个会话都以某个具名子代理启动；
- `enabledPlugins`、`extraKnownMarketplaces`、`pluginConfigs`；
- `appendPlugins`、`prependPlugins`：mods 用，限用户或托管；
- `skillOverrides`、`disableSkillShellExecution`、`disableBundledSkills`；
- `strictPluginOnlyCustomization.agents` 和 `.skills`：只在托管设置里有效，用来锁定来源。

`agents` 这个词只作为 `strictPluginOnlyCustomization` 的锁定面出现。`plugins` 只作为内联 marketplace（`extraKnownMarketplaces` 的 `settings` 来源）里的字段出现，都不是顶层键。

**SchemaStore schema 里也没有。** `https://json.schemastore.org/claude-code-settings.json` 顶层共 142 个 `properties`，不含 `agents`、`slashCommands`、`plugins`；根级 `additionalProperties` 为 `true`，所以编辑器校验不会因为这三个键报错。官方 [settings](https://code.claude.com/docs/en/settings) 页也说明 schema 可能落后于 CLI，所以这里只把它当佐证，主证据是官方文档本身。

**历史上也没有支持过。** 官方 [CHANGELOG](https://github.com/anthropics/claude-code/blob/main/CHANGELOG.md) 里，这三类功能从一开始就是文件或插件机制：

- 0.2.31：“Markdown files in .claude/commands/ directories now appear as custom slash commands”；
- 1.0.60：引入自定义子代理（`/agents`）；
- 2.0.12：“Plugin System Released”，同时引入 `/plugin install`、`/plugin enable/disable`、`/plugin marketplace`。

全文找不到 `slashCommands`，也找不到 settings 顶层 `agents` 或 `plugins` 的条目。`agents` 和 `plugins` 只以另外两种形态出现：

- CLI 参数 `--agents`：JSON 对象，键是代理名，值包含 `description`、`prompt` 等；
- Agent SDK 选项 `agents`、`plugins`：CHANGELOG 写作 “`--agents` / SDK `agents`”，[plugins/loading](https://code.claude.com/docs/en/plugins/loading) 写作 “an Agent SDK app passed the `plugins` option”。

这两种都不是 settings.json，形状也和 CCR 的数组不同。

**未知键的处理：文件照常加载，键本身不起作用。**

- [settings](https://code.claude.com/docs/en/settings#fix-a-broken-settings-file) 把问题分成两档。“Settings Error”指非法 JSON 或 “a value the schema rejects”。“Settings Warning”指个别条目失败（例子是格式错误的权限规则、未知 hook 事件名），这些值会被跳过，文件其余部分照常生效。
- settings-reference 的 [`sshHostCommands`](https://code.claude.com/docs/en/settings-reference#sshhostcommands) 一节给了先例：在 v2.1.307 之前该键尚未被识别，Claude 的 Edit 工具拒绝把它加进 settings 文件（报 `Unrecognized field: sshHostCommands`），而 “Claude Code loads a file that holds the key”。
- 推论：含 `agents`、`slashCommands`、`plugins` 的 settings.json 会照常加载，这三个键被忽略。如果让 Claude 用自己的 Edit 工具往 settings 里添加这类未识别键，可能会被拒绝。文档没有写 `claude doctor` 是否会提示未知顶层键，本次也没有实测（见第 8 节）。

## 2. 官方的真实机制

### 子代理

来源：[sub-agents](https://code.claude.com/docs/en/sub-agents)、[claude-directory](https://code.claude.com/docs/en/claude-directory)。

- 位置与优先级，同名时高者生效：
  1. 托管设置目录下的 `.claude/agents/`；
  2. `--agents` CLI JSON，仅当前会话，不落盘；
  3. 项目 `.claude/agents/`：从工作目录向上直到仓库根，每一层都扫描，离工作目录最近的生效；`--add-dir` 加入的目录也会加载；
  4. 用户 `~/.claude/agents/`；
  5. 插件的 `agents/`：名字带插件前缀，如 `my-plugin:reviewer`。
- 项目级和用户级都递归扫描子目录。代理的身份只取 frontmatter 的 `name`，与文件名和子目录无关；同一目录树里名字重复时，只加载其中一个。
- 格式是 YAML frontmatter 加 Markdown 正文，正文就是系统提示词。只有 `name` 和 `description` 必填。其余字段有 `tools`、`disallowedTools`、`model`、`permissionMode`、`maxTurns`、`skills`、`mcpServers`、`hooks`、`memory`、`background`、`effort`、`isolation`、`color`、`initialPrompt` 等。不认识的字段会被静默忽略。
- 相关 settings 键：`agent`（默认主线程代理，可放在任意 settings 文件）；托管键 `strictPluginOnlyCustomization.agents`。
- 和 CCR 模型的差异：CCR 的 `Agent` 是 `{name, model, tools, system_prompt, disabled}`（`crates/ccr-types/src/claude_settings.rs:417-436`）。它没有必填的 `description`；用 `system_prompt` 而不是 Markdown 正文（`--agents` JSON 里对应的字段叫 `prompt`）；官方 frontmatter 也没有 `disabled` 字段。

### 斜杠命令与 skills

来源：[skills](https://code.claude.com/docs/en/skills)。抓取到的 `/docs/en/slash-commands.md` 与 `/docs/en/skills.md` 内容完全相同，旧页面已并入 skills。

- 文档原话：“Custom commands have been merged into skills.” `.claude/commands/deploy.md` 和 `.claude/skills/deploy/SKILL.md` 都会生成 `/deploy`，旧的 `commands/` 文件继续可用。
- skills 的位置：
  - 托管目录下 `.claude/skills/<name>/SKILL.md`；
  - 用户 `~/.claude/skills/<name>/SKILL.md`；
  - 项目 `.claude/skills/<name>/SKILL.md`，以及嵌套的 `<subdir>/.claude/skills/`；
  - `--add-dir` 加入的目录；
  - 插件 `<plugin>/skills/<name>/SKILL.md`，以 `/plugin-name:skill-name` 调用。
- 同名时的优先级：托管高于个人，个人高于项目；skill 与 `commands/` 文件同名时 skill 生效。
- 不要写 `~/.claude/skills/synced/`：这是 claude.ai 同步下来的目录，下次同步会覆盖或删除其中的修改。
- frontmatter：skill 支持 `name`、`description`、`argument-hint`、`disable-model-invocation`、`user-invocable`、`allowed-tools`、`model`、`effort`、`context`、`agent`、`hooks`、`paths` 等；`commands/*.md` 支持其中除 `name`、`paths` 外的全部字段（claude-directory 的 frontmatter 表）。
- 相关 settings 键：`skillOverrides`、`disableSkillShellExecution`、`disableBundledSkills`；托管键 `strictPluginOnlyCustomization.skills`。
- 和 CCR 模型的差异：CCR 的 `SlashCommand` 是 `{name, description, command, args, disabled}`（`claude_settings.rs:397-415`），`command` 的注释是 “Command to execute”。官方命令是一段提示词模板，不是一条可执行命令。

### 插件

来源：[discover-plugins](https://code.claude.com/docs/en/discover-plugins)、[plugins/loading](https://code.claude.com/docs/en/plugins/loading)、settings-reference 中的 `enabledPlugins`、`extraKnownMarketplaces` 两节。

- 安装范围决定启用记录写进哪个文件：用户级写 `~/.claude/settings.json` 的 `enabledPlugins`；项目级写 `.claude/settings.json`；本地级写 `.claude/settings.local.json`。
- `enabledPlugins` 的类型是对象，把 `plugin-name@marketplace-name` 映射到布尔值。`extraKnownMarketplaces` 把 marketplace 名映射到 `{source, autoUpdate?}`；放在仓库 settings 里时，要等工作区被信任后才生效。`pluginConfigs`（用户或托管）保存插件配置对话框里填的值。
- 磁盘状态由 `claude plugin` 命令维护，根目录是 `~/.claude/plugins`，可被 `CLAUDE_CODE_PLUGIN_CACHE_DIR` 改写：
  - `installed_plugins.json`：每次安装的 `scope`、`installPath`、`version`；
  - `known_marketplaces.json`：已拉取的 marketplace，claude.ai 托管的记在 `known_marketplaces_claudeai.json`；
  - `cache/<marketplace>/<plugin>/<version>/`、`marketplaces/<name>/`、`data/<plugin-id>/`；
  - `synced/`、`.trash/`、`flagged-plugins.json`。
- 管理入口：会话里的 `/plugin`，或终端里的 `claude plugin install|enable|disable|uninstall --scope user|project|local`。
- 和 CCR 模型的差异：CCR 的 `Plugin` 是 `{id, name, version, enabled, config}`（`claude_settings.rs:438-456`），只是元数据。CCR 自己的 spec 研究也确认它不执行任何包管理（`.trellis/research/archive/2026-11/10-17-gpui-04-profiles/research/resources-source-map.md` 中 “Claude plugins 4” 一行）。

## 3. CCR 集合模型的来历

| 时间 / 提交 | 状态 |
| --- | --- |
| `68c829ca`（2025-10-14），axum 后端 `ccr-ui/backend` | 处理器读写真实文件：`handlers/agents.rs` 有 4 处 `MarkdownManager::from_home_subdir("agents")`，`handlers/slash_commands.rs` 对 `"commands"` 同样有 4 处，`handlers/plugins.rs` 用 `PluginsManager` 操作 `~/.claude/plugins/config.json`。同时 `settings_manager.rs:47-55` 的 settings 结构里已经定义了 `slash_commands`、`agents`、`plugins` 三个数组 |
| `6a42fdaa` 的父提交，axum 后端末期 | 混合模式。agents 和 slash 的列表先读项目 `.claude/{agents,commands}` 和用户 `~/.claude/{agents,commands}` 的 md 文件，都为空才回退到 settings 数组；新增、修改、删除都写 settings 数组（`api/handlers/agents.rs` 第 163、291 行，`slash_commands.rs` 第 161、215 行）。plugins 的读写都先走 settings 数组，失败才回退到 `config.json` |
| `6a42fdaa`（2026-03-04，`refactor(workspace): ♻️ 拆分 ccr-db 并重构 ccr-ui 运行架构`），迁到 Tauri | Tauri 命令只读写 settings 数组，读 md 文件的逻辑被删除。`ccr-db/src/managers/markdown.rs`、`plugins.rs` 留了下来，但仓库里已没有调用方 |
| `e8329cd8`（2026-11-06，`feat(原生桌面): ✨ 接通资源管理与敏感同步闭环`） | 新增共享策略 `crates/ccr-app/src/resources/claude_collections.rs`，GPUI 与 Tauri 共用。GPUI 资源研究（上表引用的 `resources-source-map.md`）按现状记录了“typed `ClaudeSettings.agents` 数组”，但没有核对 Claude Code 是否读取 |

所以这套集合模型并不对应 Claude Code 的任何机制。它是 axum 后端时期的 settings 结构建模，在迁到 Tauri 时取代了原来基于真实文件的实现。

## 4. 仓库里依赖这三个键的代码

| 位置 | 依赖方式 |
| --- | --- |
| `crates/ccr-types/src/claude_settings.rs:143-157` | `ClaudeSettings` 的 `slash_commands`（序列化为 `slashCommands`）、`agents`、`plugins` 都是带类型的 `Vec`。元素的必填字段：`Agent` 要 `name`、`model`；`SlashCommand` 要 `name`、`description`、`command`；`Plugin` 要 `id`、`name`、`version` |
| `crates/ccr-app/src/resources/claude_collections.rs:12-16`、`19-28`、`85-152`、`172-204` | 键名映射、按 ccr-types 校验、增删改（`Add`/`Patch`/`Replace`/`Delete`），列表时由 `enabled`/`disabled` 推出开关状态 |
| `ccr-desktop/src/runtime/resources/claude_files.rs:12-35`、`96-109`、`174-183` | GPUI 资源页的类型映射、新建模板，以及 settings 文档路径（`roots.claude.settings_file`） |
| `ccr-desktop/src/features/resources/claude_files.rs:268-270`、`365-367`，`claude_files/common_form.rs:44-132` | GPUI 页面标签、路由 id、表单字段 |
| `ccr-ui/src-tauri/src/commands/claude_agents.rs`、`claude_slash.rs`、`claude_plugins.rs`（全文） | Tauri 的 list/add/update/delete。update 用 `Mutation::Replace` |
| `ccr-ui/src/api/generated/claude.ts:12-23`，`ccr-ui/src/configs/{agents,slashCommands,plugins}.ts` | 前端 IPC 封装和页面配置 |
| 解析 `ClaudeSettings` 的整文件路径：`crates/ccr-cli/src/managers/settings.rs:187`、`crates/ccr-cli/src/services/doctor_service.rs:969`、`ccr-ui/src-tauri/src/commands/settings_raw.rs:105`、`crates/ccr-app/src/platform_settings/json.rs:47`、`56` | 不直接使用这三个键，但整份 settings 都要能反序列化成 `ClaudeSettings`。如果同名键是别的形状（比如对象，或缺必填字段），CLI 加载 settings、doctor、Tauri 原始编辑校验、GPUI 保存校验都会失败 |
| `crates/ccr-sync/src/sync/assets.rs:55-75` | 同步只把整个 `~/.claude/settings.json`（标为 sensitive）和 `~/.claude/CLAUDE.md` 当文件同步，三个数组随文件一起走，没有单独依赖。`agents/`、`commands/`、`skills/`、`rules/`、`output-styles/`、`plugins/` 都不在同步资产里 |
| `crates/ccr-db/src/services/converter_service.rs:28-34`、`81` | 转换器只解析 `mcpServers`，统计里的 `slash_commands`/`agents` 恒为 0，不读这三个键 |
| CLI/TUI（`crates/ccr`、`crates/ccr-cli`）、`ccr-vscode` | 没有直接读写这三个键。`ccr project init` 里的 `.agents/` 是 gitignore 规则，与此无关 |

## 5. 规则与输出样式的位置核对

| 类别 | CCR 读写位置 | 官方位置 | 差异 |
| --- | --- | --- | --- |
| 规则 | GPUI：`roots.home.join(".claude")` 下的 `rules/`，递归列出 `.md`（`crates/ccr-app/src/resources/claude_files.rs:23-28`、`164-172`，`ccr-desktop/src/runtime/resources/claude_files.rs:167-173`）。Tauri 系统提示页只读列出 `~/.claude/rules`（`crates/ccr-app/src/resources/system_prompts.rs:123`） | 用户 `~/.claude/rules/`，项目 `.claude/rules/`，都递归发现 `.md`；frontmatter 只有 `paths`（[memory](https://code.claude.com/docs/en/memory)，用户级规则在项目规则之前加载） | 用户级一致。缺项目级 |
| 输出样式 | GPUI：同一个 store 的 `output-styles/`，递归列出，也允许在子目录里新建。Tauri：`~/.claude/output-styles`（`ccr-ui/src-tauri/src/commands/claude.rs:217-223`），`list_output_styles` 只取顶层文件（`claude_files.rs:297-310`）。选择样式用的是 settings 的 `outputStyle` 键（`claude_settings.rs:121-129`，兼容旧写法 `output_style`），与官方键一致 | 用户 `~/.claude/output-styles`、项目 `.claude/output-styles`（从工作目录到仓库根的每一层）、托管目录下的 `.claude/output-styles`，插件也可以带 `output-styles/`；frontmatter 有 `name`、`description`、`keep-coding-instructions`、`force-for-plugin`（[output-styles](https://code.claude.com/docs/en/output-styles)） | 用户级一致。缺项目级和托管级。官方没有说明是否递归子目录，GPUI 允许建在子目录里的样式能否被加载未证实 |
| 两者共有 | rules/output-styles 的根目录按 home 拼出 `.claude`；settings 文件路径则跟随 CCR 的运行时路径解析（`crates/ccr-config/src/models/claude_runtime_paths.rs:29` 处理 `CLAUDE_CONFIG_DIR`） | claude-directory：设置 `CLAUDE_CONFIG_DIR` 后，所有 `~/.claude` 路径都移到该目录下 | 设置 `CLAUDE_CONFIG_DIR` 时，CCR 把规则和输出样式写进 Claude Code 不读的目录 |

## 6. 改造为管理真实机制时需要读写的位置

下表的位置和格式都来自第 2、5 节引用的官方文档；“建议”一列是本研究的推断，不是官方要求。

| 类别 | 可写（用户级 / 项目级） | 只读展示 | 相关 settings 键 | 建议 |
| --- | --- | --- | --- | --- |
| 子代理 | `~/.claude/agents/**/*.md`；`<project>/.claude/agents/**/*.md` | 托管目录 `.claude/agents/`；插件 `agents/`；`--agents`（会话级，不落盘） | `agent`；托管键 `strictPluginOnlyCustomization.agents` | frontmatter 按原文保存（必填 `name`、`description`），正文是系统提示。身份按 `name` 判断，不按文件名，并检测同名冲突 |
| 斜杠命令 / skills | `~/.claude/skills/<name>/SKILL.md` 和 `~/.claude/commands/**/*.md`；项目 `.claude/skills/<name>/SKILL.md`、`.claude/commands/**/*.md` | 托管目录 `.claude/skills/`；插件 `skills/`、`commands/`；`~/.claude/skills/synced/`（不要写） | `skillOverrides`、`disableSkillShellExecution`、`disableBundledSkills`；托管键 `strictPluginOnlyCustomization.skills` | CCR 已有 skills 子系统（`crates/ccr-skills`，`crates/ccr-cli/src/managers/skills_manager.rs:30` 使用 `~/.claude/skills`），可考虑与它合并，避免出现两个入口 |
| 插件 | settings 的 `enabledPlugins`（`~/.claude/settings.json`、`.claude/settings.json`、`.claude/settings.local.json`）、`extraKnownMarketplaces`、`pluginConfigs`（用户级） | `~/.claude/plugins/installed_plugins.json`、`known_marketplaces.json`、`cache/`、`marketplaces/`（根目录可被 `CLAUDE_CODE_PLUGIN_CACHE_DIR` 改写） | 另有托管键 `strictKnownMarketplaces`、`blockedMarketplaces` 等 | 安装和卸载交给 `claude plugin install` / `enable` / `disable` / `uninstall`（带 `--scope`），不要直接改安装记录文件；页面主要负责展示和启用开关 |
| 规则 | `~/.claude/rules/**/*.md`；`<project>/.claude/rules/**/*.md` | — | — | 保留 `paths` frontmatter |
| 输出样式 | `~/.claude/output-styles/*.md`；`<project>/.claude/output-styles/*.md` | 托管目录 `.claude/output-styles/`；插件 `output-styles/` | `outputStyle`（`/config` 把它写进 `.claude/settings.local.json`） | — |
| 通用 | 所有 `~/.claude` 路径都要按 `CLAUDE_CONFIG_DIR` 解析 | — | — | 复用 `ClaudeRuntimePaths`。继续保留备份、文件锁、原子写入和版本冲突检测 |

迁移时还要处理一件事：用户 settings.json 里可能已经有这三个数组，而且 `plugins[].config` 可能含凭据。这些数据对 Claude Code 无效，却会随 settings.json 一起被同步（标为 sensitive）。改造时要决定是保留不动、导出，还是迁移成文件；任何删除都要先备份。

## 7. 旁支发现（不属于本票）

- `crates/ccr-skills/src/skills_ext/plugins.rs:24-41` 读 `~/.claude/plugins/config.json` 的 `repositories[].installLocation` 来找插件里的 skills。当前官方插件根目录布局里没有这个文件（安装路径记在 `installed_plugins.json` 的 `installPath`），所以插件 skills 的发现很可能一直是空的。
- `crates/ccr-db/src/managers/markdown.rs`、`crates/ccr-db/src/managers/plugins.rs` 是 axum 时期的遗留代码，仓库里没有调用方。
- 第 5 节提到的 `CLAUDE_CONFIG_DIR` 不一致：rules/output-styles 用 home 拼路径，settings 路径却跟随运行时解析。
- 测试夹具 `ccr-ui/src-tauri/src/commands/claude.rs:535`、`582` 把 `"enabledPlugins"` 写成数组 `["alpha"]`，官方类型是对象。它只是作为未知字段原样透传，测试本身没错，但容易误导读者。

## 8. 没能确认的事项

- `claude doctor` 或 Settings Warning 会不会提示未知顶层键（如 `agents`）。文档没写，本次也没运行 Claude Code。
- Claude 自己的 Edit 工具修改一份已经含未识别键的 settings.json 时会不会被拒。现有先例只涉及“新增”未识别键。
- `output-styles/` 子目录里的样式会不会被加载，官方没有说明。
- 早期插件版本是否用过 `~/.claude/plugins/config.json`（CCR 最初的 `PluginsManager` 就以它为目标），没有核实。
- “历史上从未支持”的判断依据是 CHANGELOG、官方文档和 schema 里都没有出现，没有逐个检查旧版 CLI 的二进制。
