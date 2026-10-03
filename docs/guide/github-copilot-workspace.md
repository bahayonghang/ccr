# GitHub Copilot 工作区支持

CCR 现在在仓库内补齐了 GitHub Copilot for VS Code 的官方工作区资产，并且明确把它和 Codex CLI 运行时配置分开管理。

## 资产位置

| 位置 | 用途 |
|------|------|
| `.github/copilot-instructions.md` | 仓库级默认说明 |
| `.github/instructions/*.instructions.md` | 按 Rust、UI、文档范围追加说明 |
| `.github/prompts/*.prompt.md` | 可复用的任务提示模板 |
| `.github/agents/*.agent.md` | 可复用的 Copilot 自定义 agent |
| `AGENTS.md` | 已跟踪的共享项目规则 |
| `.github/skills/*/SKILL.md` | 已跟踪的共享检查与工作流说明来源 |

## 关键边界

### GitHub Copilot for VS Code

- 读取仓库内 `.github/*` 工作区资产
- 可以发现共享 skills
- 面向 VS Code Chat / Agent Mode 的协作体验

### Codex CLI

- 运行时配置位于用户目录 `~/.codex/`
- CCR 的 Unified profile 位于 `~/.ccr/platforms/codex/profiles.toml`
- 与 GitHub Copilot 工作区资产不是同一套机制

## 共享规则与本地安装

仓库共享规则引用 `AGENTS.md` 和 Git 已跟踪的 `.github/skills/*/SKILL.md`。Claude Code、Codex、Grok Build、Kimi Code、OMP 均可读取这些说明并运行相同检查命令；各客户端把它们安装到自己的本地技能目录（如 `.codex/skills/`，已按 `.gitignore` 忽略），自动发现与加载行为由各自配置决定。

`.claude/skills/` 属于被 Git 忽略的本地安装内容。全新检出的资产检查直接验证已跟踪共享来源。新增 GitHub Copilot 专用 skill 时，需要单独确认文件归属与交付范围。

## 当前提供的内容

- 仓库级 Copilot 指令文件
- Rust / UI / 文档三类 scoped instructions
- Rust / UI / 文档三类 prompt files
- `researcher`、`implementer`、`reviewer` 三个自定义 agents
- `just copilot-check` 与 `scripts/quality/check-copilot-assets.mjs` 用于校验这些资产是否齐全，且文档中没有把 GitHub Copilot 和 Codex CLI 混写

## 维护约定

1. 新增或重命名 `.github/*` 资产时，同时更新本页和 VitePress 侧边栏。
2. 共享规则维护在 `AGENTS.md` 和已跟踪的 `.github/skills/*/SKILL.md`；本地安装目录按工具配置管理。
3. 文档里提到 GitHub Copilot 时，指的是 VS Code 工作区能力；提到 Codex 时，指的是 Codex CLI。
4. 修改资产或检查器后运行 `node --test scripts/quality/check-copilot-assets.test.mjs`、`just copilot-check` 和 `just docs-check`。五套执行工具均使用这些入口。

检查器在内存中统一 LF 与 CRLF，并接受文件开头的单个 UTF-8 BOM。文档字节保持不变。Frontmatter 使用独立的 `---` 行作为首尾分隔符；结束分隔符可以位于文件末尾。当前资产使用单行标量字段，必填值必须非空。缺字段、空白或注释空值、未闭合分隔符、重复字段和格式错误的引号均使命令非零退出。

## 官方参考

- [Custom instructions](https://code.visualstudio.com/docs/copilot/customization/custom-instructions)
- [Prompt files](https://code.visualstudio.com/docs/copilot/customization/prompt-files)
- [Custom agents](https://code.visualstudio.com/docs/copilot/customization/custom-agents)
- [Agent skills](https://code.visualstudio.com/docs/copilot/customization/agent-skills)
