# 统一五套 harness 项目规则与技能路由

## Goal

修复 F4/F6，把五套工具的项目事实、加载链、审查权限和现有技能路由统一到真实源码及官方能力，记录批准项的适用工具与证据。

## Confirmed Facts

AGENTS.md:4 与 CLAUDE.md:53 的 llmusage 事实冲突；CLAUDE.md:43 的 imports 在 code span；.codex/skills/ccr-ui-visual-workflow/SKILL.md:25 仍为 Vue。

父任务需求映射：根 R2/R4/R5，工具上下文与规则项同时映射根 R3；证据见父任务 research/audit.md 与 research/harnesses.md。

## Requirements

- R1：修复 F4/F6，把五套工具的项目事实、加载链、审查权限和现有技能路由统一到真实源码及官方能力，记录批准项的适用工具与证据。
- R2：不更改用户全局AGENTS/账户/默认模型，不写外部知识库，不复制全部规则，不新增文档/模型validator，不启动五套客户端计费运行。
- R3：审批后按本项文件边界执行，将真实验证和适用工具交父任务整合；当前保持 planning。

## Acceptance Criteria

- [x] AC1（R1）：AGENTS 描述 React 和 CLI+SQLite/local ccr-usage；CLAUDE 实际 import AGENTS，OpenSpec限定和失效 scoped import 得到纠正；ccr-ui 的目录说明服从 DESIGN.md 行情终端/React，Rust目录入口指向实际crate所有者；事实有当前源码依据。
- [x] AC2（R1）：五工具各列入口、当前集成机制、官方能力来源、只读reviewer与self-fix check区别；Kimi/Grok过时能力声明纠正，保留现有pull方案。
- [x] AC3（R1）：项目说明/现有技能注明适用工具，区分只读检查、生成/自动改写与安装命令；保持并行测试，UI操作需明确授权。
- [x] AC4（R1）：新增 harness 说明中英镜像齐全，docs audit/build、diff check通过；每个批准项回写命令/退出码/实际模型或未验证信息；真实五工具会话未运行时明确UNVERIFIED。
- [x] AC5（R2）：diff 不超出 implement.md 白名单，未执行所列排除动作。
- [x] AC6（R3）：交付记录含适用工具、实际执行模型、检查结果与 UNVERIFIED；未经用户批准不启动。

## Dependencies

共享事实纠正可早做，最终收口等待已批准的 UI/CI/OMP 子项；不等待未批准P2。

## Out of Scope

不更改用户全局AGENTS/账户/默认模型，不写外部知识库，不复制全部规则，不新增文档/模型validator，不启动五套客户端计费运行。
