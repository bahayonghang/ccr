# T03 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对 git status、关联来源与当前依赖版本。
- [x] 前置项 无前置子任务 满足其验收条件。
- [x] 本任务上下文通过 task.py validate 后才由主会话运行 task.py start。

## 步骤
- [x] 复核 F05 与设计文件白名单；发现新范围先报告。
- [x] 按 design.md 实施最小修改，并保留原失败证据。
- [ ] 逐项满足 prd.md 的 3 条 AC。
- [x] 强模型独立审查完成；T02/T03/T06 见父任务最终集成审查，T01/T04/T05/T08 见首轮审查。
- [x] 主会话已回写项目规范并注明五套工具适用；未提交或归档。

## 必须通过的检查
- [x] `python -m unittest scripts.ci.test_check_workflow_governance`
- [x] `just workflow-governance-check`
- [x] `just version-check`
- [x] `cd docs && bun install --frozen-lockfile && bun run audit && bun run build`
- [x] `cd ccr-ui && bun install --frozen-lockfile`
- [x] `actionlint`
- [ ] `下一次已授权或自然触发的 Dependabot 运行验证，不主动推送或触发`

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

实施记录：[research/implementation-results.md](research/implementation-results.md)。本地九项命令退出码均为 0；共享规范回写和独立审查已完成；未来真实 Dependabot 运行仍未验证。
