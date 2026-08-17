# GitHub Copilot 工作区支持

CCR 现在在仓库内补齐了 GitHub Copilot for VS Code 的官方工作区资产，并且明确把它和 Codex CLI 运行时配置分开管理。

## 资产位置

| 位置 | 用途 |
|------|------|
| `.github/copilot-instructions.md` | 仓库级默认说明 |
| `.github/instructions/*.instructions.md` | 按 Rust、UI、文档范围追加说明 |
| `.github/prompts/*.prompt.md` | 可复用的任务提示模板 |
| `.github/agents/*.agent.md` | 可复用的 Copilot 自定义 agent |
| `.claude/skills/` | 共享项目 skills 的单一事实来源 |

## 关键边界

### GitHub Copilot for VS Code

- 读取仓库内 `.github/*` 工作区资产
- 可以发现共享 skills
- 面向 VS Code Chat / Agent Mode 的协作体验

### Codex CLI

- 运行时配置位于用户目录 `~/.codex/`
- CCR 的 Unified profile 位于 `~/.ccr/platforms/codex/profiles.toml`
- 与 GitHub Copilot 工作区资产不是同一套机制

## 为什么不加 `.github/skills/`

GitHub Copilot 支持从 `.claude/skills/`、`.github/skills/`、`.agents/skills/` 读取共享项目 skills。这个仓库刻意保留 `.claude/skills/` 作为单一事实来源，避免把同一份 skills 复制到多个目录后产生漂移。

如果未来确实需要拆分 GitHub Copilot 专用 skill，再单独评估是否引入 `.github/skills/`。

## 当前提供的内容

- 仓库级 Copilot 指令文件
- Rust / UI / 文档三类 scoped instructions
- Rust / UI / 文档三类 prompt files
- `researcher`、`implementer`、`reviewer` 三个自定义 agents
- `just copilot-check` 与 `scripts/quality/check-copilot-assets.mjs` 用于校验这些资产是否齐全，且文档中没有把 GitHub Copilot 和 Codex CLI 混写

## 维护约定

1. 新增或重命名 `.github/*` 资产时，同时更新本页和 VitePress 侧边栏。
2. 共享项目 skills 默认只维护 `.claude/skills/`。
3. 文档里提到 GitHub Copilot 时，指的是 VS Code 工作区能力；提到 Codex 时，指的是 Codex CLI。
4. 提交前运行 `just copilot-check`。

## 官方参考

- [Custom instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Prompt files](https://code.visualstudio.com/docs/copilot/customization/prompt-files)
- [Custom agents](https://code.visualstudio.com/docs/copilot/customization/custom-agents)
- [Agent skills](https://code.visualstudio.com/docs/copilot/customization/agent-skills)
