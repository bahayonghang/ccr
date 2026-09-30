# 五套 harness 入口、加载链与角色权限

本页记录 Claude Code、Codex、Grok Build、Kimi Code、OMP（Oh My Pi）在本仓库的入口、当前集成方式、官方能力来源，以及审批前只读 reviewer 与批准后 implement / self-fix check 的区别。产品行为规则仍以根目录 `AGENTS.md` 为唯一事实来源；不要把用户全局规则复制五份进仓库。本页不进入 VitePress 产品导航，由 `AGENTS.md` / `CLAUDE.md` 链接。

English: [harnesses](/en/agents/harnesses.md).

## 角色（所有工具共用）

| 角色 | 何时 | 权限 |
|---|---|---|
| 只读 reviewer | 审批前：核对计划、源码、官方文档 | 读取批准范围；仅运行已授权检查，并在批准的任务目录记录证据。不改产品代码、全局规则、账户或默认模型。 |
| research | 按授权调查问题 | 读取相关材料；写入指定任务的 research 目录。 |
| implement | 用户批准实施后 | 按 `implement.md` 白名单改文件并跑列明检查。 |
| Trellis check | 批准实施之后、作为执行角色 | **可写、可自修**（lint/typecheck/漏测），限已批准文件与验收。原生角色定义位于各工具的本地 agents 目录。 |

Grok 内置 `plan` / `explore` 没有 shell/edit，不能承担需要跑测试的验收；自定义 `.grok/agents` 可以有工具，与内置 plan/explore 不是同一角色。

`xhigh` / `low` / `medium` 是 **Codex** `model_reasoning_effort`，不是五工具通用参数。不要在 Claude Code、Grok、Kimi、OMP 上当作官方 effort 开关。

浏览器、Playwright 或 UI 工具**可用**不等于已授权操作界面。

## 官方能力 vs 本仓库集成

官方能力、Git 交付、本地生成配置和原生运行证据分别记录。下表的本地集成观察日期为 2026-09-29；新检出需另行初始化。文件存在不能证明原生客户端已加载或已信任。

| 工具 | 仓库入口 | 当前项目集成 | 官方能力来源 |
|---|---|---|---|
| Claude Code | `CLAUDE.md`（真实 `@AGENTS.md` import）、`.claude/settings.json`、`.claude/hooks/`、`.claude/agents/`、`.claude/skills/` | SessionStart / PreToolUse / PostToolUse **hooks** 注入 Trellis 上下文；三个 Trellis agents。共享事实只维护在 `AGENTS.md`。 | [memory / import](https://code.claude.com/docs/en/memory)、[subagents](https://code.claude.com/docs/en/sub-agents) |
| Codex | `AGENTS.md`、`.codex/hooks.json`、`.codex/agents/`、`.agents/skills/`、`.codex/skills/` | **hooks**（SessionStart / UserPromptSubmit / SubagentStart）。文件存在不等于用户已启用/信任。 | [AGENTS.md](https://developers.openai.com/codex/guides/agents-md)、[subagents](https://developers.openai.com/codex/subagents) |
| Grok Build | `.grok/agents/`、`.grok/skills/`、`.grok/commands/trellis-*.md` | **本仓库用手拉 prelude**，未安装项目 hooks。官方支持 [agents](https://docs.x.ai/build/features/subagents) 与 [hooks](https://docs.x.ai/build/features/hooks)。不要写成平台没有 agents/hooks。 | [subagents](https://docs.x.ai/build/features/subagents)、[hooks](https://docs.x.ai/build/features/hooks)、[compatibility](https://docs.x.ai/build/features/skills-plugins-marketplaces) |
| Kimi Code | `.kimi-code/agents/`、`.kimi-code/skills/`、共享 `.agents/skills/` | 本机有 `trellis-implement`、`trellis-check`、`trellis-research` 项目 agents，使用 pull 上下文；未安装项目 hooks。新检出若尚无项目 agents，可由具备执行工具的角色显式加载对应技能与任务文档。 | [agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents)、[hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html) |
| OMP | `.omp/agents/`、`.omp/skills/`、`.omp/extensions/trellis/` | TypeScript **extension** 自动注入任务上下文（存在时含 `prd.md` / `design.md` / `implement.md` 与角色 jsonl）。无项目 `settings.json`；扫描 `.omp/`。 | [task](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/task.md)、[context files](https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md) |

本地 Trellis 文件允许按 `.agents/skills/trellis-meta/references/local-architecture/generated-files.md` 定制：可改 workflow/spec/平台入口，不要手改 `.trellis/.template-hashes.json` 或上游模板。

## 共用技能与命令副作用

下列技能路径在 `.codex/skills/` 下，**五套工具都适用**（不只 Codex）。不要为对齐官方能力去新铺 hooks 或复制五份正文。

| 技能 | 适用 | 注意 |
|---|---|---|
| `.codex/skills/ccr-ui-visual-workflow/SKILL.md` | 五工具的 `ccr-ui` 视觉工作 | React + `DESIGN.md`。默认网页预览，不要默认 Tauri 桌面壳。UI 工具可用 ≠ UI 操作授权。 |
| `.codex/skills/ccr-gate-recovery/SKILL.md` | 五工具的本地门禁恢复 | Rust 默认并行，使用 `--skip export_bindings`；共享环境由测试夹具局部隔离，绑定由独立生成门禁负责。 |
| Trellis start / implement / check / research | 各工具自己的 agents 或 Kimi/Grok 的 pull 技能 | Grok/Kimi 的「无 hook」指**本仓库未安装**，不是平台上限。 |

命令分类：

- **只读检查**：`just version-check`、`just fmt-check`、`bun run type-check`、`cd docs && bun run audit`。
- **会改文件**：`just fmt`、`just version-sync`、部分 `lint`/`lint:fix`。跑完看 diff。
- **工具前置条件**：验证命令缺少工具时应明确失败；全局安装必须与聚合检查分开。缺少条件的门禁保留未验证状态。

## Git 交付与新检出

共享说明、两项 CCR 专用 skill 和 `.omp/extensions/trellis/index.ts` 属于仓库交付范围。其余本地 agents、hooks、skills 和设置按 `.gitignore` 保持本地生成；审查机器上的文件不能代表新检出已经安装。

1. 核对安装的 Trellis CLI 与项目版本，并查看 `trellis init --help`。版本不匹配时保留环境差异；不要自动升级全局工具。
2. 在已授权初始化的工作区，用匹配版本选择所需平台：`--claude`、`--codex`、`--grok`、`--kimi` 或 `--omp`，配合 `--skip-existing` 保留仓库定制。例如 `trellis init --kimi --skip-existing`。
3. 检查生成结果和 diff，分别验证客户端加载与 trust。保持用户的信任设置，不复制个人配置、凭据或缓存。

本地生成的 `platform-map.md` 用于检查生成层漂移；共享契约以本页和 `AGENTS.md` 为准。OMP 的受控测试验证文档注入、缓存刷新、路径边界和预算；原生 OMP 会话仍需独立验证。

## 模型分工

强模型负责根因、安全、权限与最终审查。较低成本模型执行已确定的 YAML、双语同步、测试夹具和兼容版本更新。每次分派提供任务路径、文件白名单、不可变规则和验收命令；发现未知根因或扩大范围时交回主会话。工具名称与角色名称不能推导账户价格。
