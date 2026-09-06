# 执行计划

## 文件白名单

- AGENTS.md
- CLAUDE.md
- code_map.md
- ccr-ui/CLAUDE.md
- ccr-ui/code_map.md
- crates/AGENTS.md
- crates/code_map.md
- crates/ccr/src/CLAUDE.md
- docs/agents/harnesses.md（新增）
- docs/en/agents/harnesses.md（新增，中英对齐）
- .codex/skills/ccr-gate-recovery/SKILL.md
- .codex/skills/ccr-ui-visual-workflow/SKILL.md
- .kimi-code/skills/trellis-start/SKILL.md
- .kimi-code/skills/trellis-implement/SKILL.md
- .kimi-code/skills/trellis-check/SKILL.md
- .kimi-code/skills/trellis-research/SKILL.md
- .grok/commands/trellis-start.md
- .grok/agents/trellis-implement.md
- .grok/agents/trellis-check.md
- .agents/skills/trellis-meta/references/platform-files/platform-map.md
- .claude/skills/trellis-meta/references/platform-files/platform-map.md
- .grok/skills/trellis-meta/references/platform-files/platform-map.md
- .omp/skills/trellis-meta/references/platform-files/platform-map.md

## 顺序

共享事实纠正可早做，最终收口等待已批准的 UI/CI/OMP 子项；不等待未批准P2。

- [x] 获取本子项的明确实施批准，检查最新计划，再 task.py start。
- [x] 阅读 implement.jsonl/check.jsonl 的真实规范、父报告和 design.md。
- [x] 重现对应失败/缺口，实施最小变更，保留有意义的正负回归。
- [x] 逐条运行下列检查，记录命令/cwd/退出码；预期失败的负例以非零为通过。
- [x] 强模型独立审查，向父任务/规则子任务交付适用工具与证据；缺证据明确列出。

## 检查

~~~text
git diff --check
cd docs
bun run audit
bun run build
人工逐条核对 research/harnesses.md 主源、源码实际依赖、入口加载与权限说明
定向检查上述 11 个 Trellis 本地文件；四份 platform-map 的本次 Grok/Kimi 行一致，保留其他内容
~~~

有生成/安装副作用的聚合命令先检查组成步骤；仅使用已有依赖。本地检查不证明当前 hosted 或真实客户端通过。必要检查缺条件时该项验收保持未完成，不能用其他命令的通过替代。

## 完成和权限

本项 AC 与父任务已批准集的交付约束均满足才算完成。用户 `/goal` 已批准本子任务在 check PASS 后提交并归档；仍不发布、不编辑用户全局文件。不更改用户全局AGENTS/账户/默认模型，不写外部知识库，不复制全部规则，不新增文档/模型validator，不启动五套客户端计费运行。gitignore 下的白名单文件仅做路径级 `git add -f`，禁止 `git add -f .trellis/`。
