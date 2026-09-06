# 五套 harness 能力与项目接入证据

核对日：2026-09-06。官方能力、仓库配置、当前会话可用工具是三种不同证据。此处不把某 harness 等同于某价位或能力的模型，也不声称启动五工具实测过。

| 工具 | 官方能力边界 | 仓库接入与缺口 | 本项目分工建议 |
|---|---|---|---|
| Claude Code | CLAUDE.md、真实 @imports、scoped rules；子代理可限制工具和模型 | .claude settings 有 hooks/3 Trellis agents；CLAUDE 中代码 span 的 imports 不生效，共享事实冲突 | 强模型审查架构、Rust敏感持久化/IPC；明确文件和AC后便宜模型做 fixture/说明 |
| Codex | 分层 AGENTS、自定义 agents、独立子上下文、agent 模型/effort | .codex config/hooks/agents + .agents skills；native injection 还依赖用户启用/信任，文件存在不等于生效 | 强模型执行失败链调查、测试/CI复核；子代理低成本档执行固定契约修改 |
| Grok Build | 自定义 .grok/agents、hooks、兼容规则；内置 plan/explore 没有 shell/edit | .grok agents/skills/commands 使用 pull prelude；缺本地 hooks 不代表平台不支持 | 强模型独立反证计划/规则；测试要交主会话或有 shell 角色，不能给内置 plan/explore |
| Kimi Code | coder/explore/plan；项目 agents/hooks；子代理不继承父对话 | .kimi-code 目前用技能交内置 coder，暂无项目 agents/hooks；“平台不支持”文案过时 | 完整 handoff 后做单模块任务/资料整理；强模型也可规划，不能默认 Kimi 一定便宜 |
| OMP (Oh My Pi) | .omp agents/skills/extensions；task 独立上下文、批量与可选隔离；模型依配置 | .omp Trellis extension 已有自动注入，但漏 design/implement；pi/task 是 selector 不是明确廉价模型 | 补齐上下文后适合独立批次执行与多 provider 分配；记录实际解析模型后再作成本判断 |

## 主要来源

- [Claude 项目说明及 import](https://code.claude.com/docs/en/memory)
- [Claude subagents](https://code.claude.com/docs/en/sub-agents)
- [Codex AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [Codex subagents](https://developers.openai.com/codex/subagents)
- [Grok subagents](https://docs.x.ai/build/features/subagents)
- [Grok hooks](https://docs.x.ai/build/features/hooks)
- [Grok compatibility](https://docs.x.ai/build/features/skills-plugins-marketplaces)
- [Kimi agents](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents)
- [Kimi hooks](https://www.kimi.com/code/docs/en/kimi-code-cli/customization/hooks.html)
- [OMP task 官方仓库](https://github.com/can1357/oh-my-pi/blob/main/docs/tools/task.md)
- [OMP context 官方仓库](https://github.com/can1357/oh-my-pi/blob/main/docs/context-files.md)

## 规则差异的最小解决方案

1. AGENTS 维护当前工程事实；CLAUDE 以实际 import 复用，保留 Claude 专属加载说明。其他入口明确读取共享文件，不复制五份规则正文。
2. docs/agents/harnesses.md（计划新增）列实际加载链、当前项目集成方式、命令副作用、只读 reviewer 与实施后 self-fix check 的区别。
3. Kimi/Grok 的手工拉取继续有效，修正“平台没有能力”的描述即可；本轮后续计划也不要求铺设新 hooks/agents。
4. OMP loader 补设计/执行计划，是有源码缺口支撑的局部变更；测试不扩张为跨平台通用框架。
5. 审批前只读 reviewer；批准实施后 implement/self-fix check。任何工具的文件/模型/权限继承都以当前运行证据为准。

## 强模型与低成本执行

“更便宜模型”是相对主规划模型、同一 provider/账户的实际成本而言；价格和解析模型本轮未查验，不给货币数值或绝对便宜排名。

| 工作 | 规划/最终审查 | 可交低成本模型的范围 | 升级回强模型的条件 |
|---|---|---|---|
| UI route mock | Codex 或 Claude Code 的强模型核对 DTO 与失败链 | 明确 DTO 后改一个 test 文件、typed fixture，跑指定 smoke | 拟改产品容错、API形状或出现第二类失败 |
| CI shell/path | Codex 强模型追踪失败传播和输入消费者；Grok plan 可独立检查逻辑 | shell: bash、精确路径集合、已规定的正负测试 | coverage语义、跨平台shell、权限或输入范围改变 |
| OMP context | Codex/Claude/OMP 的强模型审查上下文与信任边界 | 接收固定设计后局部字段加载和行为测试 | 引入路径信任新规则、跨进程/compaction变化 |
| 说明/skill 路由 | 强模型确定唯一事实和各工具官方边界 | Claude/Codex/Kimi/OMP 上的低成本档逐项同步文字与链接 | 规则冲突、来源不确定、需要修改全局配置 |
| 历史诊断 | Codex 或 Claude 强模型读日志/设计最小复现；Grok作假设反证 | 收集脱敏日志、按已定参数运行验证 | 需要解释并发/IO/timeout、决定产品语义 |

本环境可见低成本候选可从 gpt-5.6-terra/luna 等档选择，但只在 provider 的真实价格和可用模型确认后采用；不更改默认 agent 模型。其他 harness 同理按可用低成本档选择。所有最终验收回到独立强模型，不把测试通过当成无证据的硬件/账户/权限证明。

## 批准后的回写与证据格式

首选项目内 docs/agents/harnesses.md + 现有 skills，不写用户全局配置或外部知识库。每个批准项交付记录：适用工具、源文件、执行角色/实际模型、命令与退出码、审查结论、未验证项。无需新 schema 或自动路由引擎。

五套真实会话验收是单列的运行证据：同一安全只读任务，返回 task path、PRD/design/implement 三个标记、允许工具范围和是否编辑。需要本机可用且已授权的客户端/账户；当前仅有源码/文档证据，未来缺客户端或调用许可时标 UNVERIFIED，不能阻塞局部文件/单测的完成，也不能冒称五工具运行验收完成。
