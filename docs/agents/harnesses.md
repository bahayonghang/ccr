# 五套 harness 入口、加载链与角色权限

本页记录 Claude Code、Codex、Grok Build、Kimi Code、OMP（Oh My Pi）在本仓库的入口、当前集成方式、官方能力来源，以及审批前只读 reviewer 与批准后 implement / self-fix check 的区别。产品行为规则仍以根目录 `AGENTS.md` 为唯一事实来源；不要把用户全局规则复制五份进仓库。本页不进入 VitePress 产品导航，由 `AGENTS.md` / `CLAUDE.md` 链接。

English: [harnesses](/en/agents/harnesses.md).

## 角色（所有工具共用）

| 角色 | 何时 | 权限 |
|---|---|---|
| 只读 reviewer | 审批前：核对计划、源码、官方文档 | 读仓库与文档。不改代码、不改全局 AGENTS/账户/默认模型。 |
| implement | 用户批准实施后 | 按 `implement.md` 白名单改文件并跑列明检查。 |
| Trellis check | 批准实施之后、作为执行角色 | **可写、可自修**（lint/typecheck/漏测）。不是审批前只读审查。见 `.codex/agents/trellis-check.toml`、`.claude/agents/trellis-check.md`。 |

Grok 内置 `plan` / `explore` 没有 shell/edit，不能承担需要跑测试的验收；自定义 `.grok/agents` 可以有工具，与内置 plan/explore 不是同一角色。

`xhigh` / `low` / `medium` 是 **Codex** `model_reasoning_effort`，不是五工具通用参数。不要在 Claude Code、Grok、Kimi、OMP 上当作官方 effort 开关。

浏览器、Playwright 或 UI 工具**可用**不等于已授权操作界面。

## 官方能力 vs 本仓库集成

官方文档已支持的能力，不得写成「平台不能做」。本仓库可能仍用手拉（pull）prelude、未安装项目 hooks/agents。下表「当前集成」只描述本仓库现状。

| 工具 | 仓库入口 | 当前项目集成 | 官方能力来源 |
|---|---|---|---|
| Claude Code | `CLAUDE.md`（真实 `@AGENTS.md` import）、`.claude/settings.json`、`.claude/hooks/`、`.claude/agents/`、`.claude/skills/` | SessionStart / PreToolUse / PostToolUse **hooks** 注入 Trellis 上下文；三个 Trellis agents。共享事实只维护在 `AGENTS.md`。 | [memory / import](https://code.claude.com/docs/en/memory)、[subagents](https://code.claude.com/docs/en/sub-agents) |
| Codex | `AGENTS.md`、`.codex/hooks.json`、`.codex/agents/`、`.agents/skills/`、`.codex/skills/` | **hooks**（SessionStart / UserPromptSubmit / SubagentStart）。文件存在不等于用户已启用/信任。 | [AGENTS.md](https://developers.openai.com/codex/guides/agents-md)、[subagents](https://developers.openai.com/codex/subagents) |
| Grok Build | `.grok/agents/`、`.grok/skills/`、`.grok/commands/trellis-*.md` | **本仓库用手拉 prelude**，未安装项目 hooks。官方支持 [agents](https://docs.x.ai/build/features/subagents) 与 [hooks](https://docs.x.ai/build/features/hooks)。不要写成平台没有 agents/hooks。 | [subagents](https://docs.x.ai/build/features/subagents)、[hooks](https://docs.x.ai/build/features/hooks)、[compatibility](https://docs.x.ai/build/features/skills-plugins-marketplaces) |
| Kimi Code | `.kimi-code/skills/trellis-*.md`、共享 `.agents/skills/` | **本仓库用手拉技能**，主会话把内置 `coder` 派成 Trellis 角色；未安装项目 agents/hooks。官方支持 [agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents) 与 [hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html)。 | [agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents)、[hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html) |
| OMP | `.omp/agents/`、`.omp/skills/`、`.omp/extensions/trellis/` | TypeScript **extension** 自动注入任务上下文（存在时含 `prd.md` / `design.md` / `implement.md` 与角色 jsonl）。无项目 `settings.json`；扫描 `.omp/`。 | [task](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/task.md)、[context files](https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md) |

本地 Trellis 文件允许按 `.agents/skills/trellis-meta/references/local-architecture/generated-files.md` 定制：可改 workflow/spec/平台入口，不要手改 `.trellis/.template-hashes.json` 或上游模板。

## 共用技能与命令副作用

下列技能路径在 `.codex/skills/` 下，**五套工具都适用**（不只 Codex）。不要为对齐官方能力去新铺 hooks 或复制五份正文。

| 技能 | 适用 | 注意 |
|---|---|---|
| `.codex/skills/ccr-ui-visual-workflow/SKILL.md` | 五工具的 `ccr-ui` 视觉工作 | React + `DESIGN.md`。默认网页预览，不要默认 Tauri 桌面壳。UI 工具可用 ≠ UI 操作授权。 |
| `.codex/skills/ccr-gate-recovery/SKILL.md` | 五工具的本地门禁恢复 | 按现有门禁**并行**能跑的保持并行；Rust `--test-threads=1` 是已有 flake 规避，不是新的串行恢复引擎。不要发明第二套 CI。 |
| Trellis start / implement / check / research | 各工具自己的 agents 或 Kimi/Grok 的 pull 技能 | Grok/Kimi 的「无 hook」指**本仓库未安装**，不是平台上限。 |

命令分类：

- **只读检查**：`just version-check`、`just fmt-check`、`bun run type-check`、`cd docs && bun run audit`。
- **会改文件**：`just fmt`、`just version-sync`、部分 `lint`/`lint:fix`。跑完看 diff。
- **可能安装工具**：聚合 `just ci` 中的 audit 等步骤。不要用「别的命令绿了」代替缺条件的验收。

## 已批准子项回写（P2 未做完）

每条含工具、命令、退出码、角色/模型、UNVERIFIED。真实五工具会话未在本项启动。

### ui-smoke · F1

- **白名单**：`ccr-ui/tests/shell/route-view-mount.smoke.test.tsx`
- **命令**：原 `bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx` 为 exit **1**，现为 **0**；`type-check` / `lint:ci` / 全量 `bun run test` 为 **0**。
- **工具 / 角色**：dispatch `trellis-implement` 然后 `trellis-check` **PASS**。实际解析模型 UNVERIFIED。
- **UNVERIFIED**：hosted Frontend CI、原生 Tauri 桌面。

### ci-verdict · F2 / F3

- **变更要点**：`.github/workflows/vscode-ci.yml` coverage 步骤 `shell: bash`；`.cargo/tauri-ci.toml` → tauri 面；`.cargo/config.toml` → root+tauri；`.cargo/audit.toml` → root。
- **命令**：unittest **24 OK**；`check_workflow_governance` exit **0**；vscode-coverage exit **0**（70% 阈值未改）。
- **UNVERIFIED**：hosted GitHub Actions、branch protection。

### omp-context · F5

- **行为**：`buildTaskContext` 在存在时注入 `design.md` / `implement.md`。
- **命令**：`bun test scripts/trellis/omp-context.test.ts` **5/5** exit **0**。
- **UNVERIFIED**：真实 OMP 会话。四个 `.omp` 白名单文件已用路径级 `git add -f` 跟踪；`.gitignore` 仍包含 `.omp/`。

### ci-history

已批准、**尚未执行**。保持 pending P2。本页不声称历史 CI 证据复核完成。

## 平台地图中的 Grok / Kimi 行

四份 `platform-map.md`（`.agents` / `.claude` / `.grok` / `.omp` 下 `trellis-meta/references/platform-files/`）只更正 Grok/Kimi：**官方有 agents/hooks，本仓库仍用 pull**。其他工具原文不动。
