# 五套 harness 能力、集成与模型分工

> 历史快照：本文保留实施批准前的审查状态与失败记录。批准后的改动和验证见 [实施与验证记录](implementation-results.md)。

核对日期：2026-09-29（America/Chicago）。外部资料使用下列官方文档/维护方源码；本机文件与 Git 跟踪状态分别核对。当前会话承担规划和审查，没有启动写代码的子 agent，也没有修改默认模型。

## 1. 能力边界

| 工具 | 官方能力与原生结构 | 本仓库观察 | 适合的规划/审查工作 | 适合的低成本执行工作 |
|---|---|---|---|---|
| Claude Code | CLAUDE.md import；Markdown 自定义 subagent；tools、permissionMode、model 各有独立含义 [S1,S2] | 真实 @AGENTS.md；本机 .claude hooks/agents；这些目录未在 Git 交付 | T01 注入、T05 包范围、T07 并发与子进程、T08 安全复核；只读审查应使用明确受限角色 | T03 两处生态配置、T04 解析与负例、T02 双语同步。可选择账户可用的较低成本模型；具体价格未核验 |
| Codex | AGENTS 层级、TOML agents、sandbox；model/model_reasoning_effort 可按角色配置；hook 需受信 [S3,S4] | 本机 auto 分派配置；trellis-check 为 workspace-write、自修；本机 agent/hook 未跟踪 | 主规划、契约测试、原生测试证据与最终差异；审批前不能把可写 check 当只读 reviewer | 已批准白名单内的脚本、夹具、lockfile 定向执行。保持规划/审查强模型，仅明确指定执行角色时降成本 |
| Grok Build | 内置 explore/plan 无 shell、无 edit；general-purpose 及自定义 agents 能力另行配置；平台支持 hooks [S5] | .grok/agents 三个 Trellis 定义；无项目 hooks，采用 pull prelude；目录未跟踪 | plan/explore 适合 T02 规则冲突和模块边界的只读独立意见，不能负责跑测试验收 | 批准后使用具备 shell/edit 的自定义或 general-purpose 执行固定改动；不能把 plan 角色当 test runner |
| Kimi Code | 内置 coder 可读写/执行；explore 只读；plan 无 shell；支持项目 Markdown agents [S6] | 实际有 .kimi-code/agents 三个角色，docs 却写未安装；没有项目 hooks；目录未跟踪 | plan/explore 做范围、依赖和文档对齐审查；安全结论仍由强模型负责 | coder 或已核对权限的自定义执行角色适合 T03/T04/测试夹具。不能把 coder 名称等同低价模型 |
| OMP | 原生 .omp/agents；tools、spawns、model、thinkingLevel；extension 注入。跨 harness agent 格式不自动通用 [S7] | 本机 extension 存在但遗漏 design/implement；已跟踪测试依赖 ignored 源；research 写有 pi/task 模型别名 | T01 扩展生命周期与上下文预算需强模型审查，先恢复契约再用于实施 | 批准后的固定文件编辑和证据汇总。pi/task 是别名，未验证账户解析和价格；不得据别名断言便宜或强 |

能力取决于实际工具、权限和配置；成本取决于实际模型与账户。上述分工是基于本次文件范围的建议，没有横向模型基准或实付价格比较。Codex 的 effort 值不能复制为五工具通用参数。

## 2. 角色与授权

- 审批前 reviewer：可读取、运行经授权的现有检查、在本任务目录记录证据；不修产品和正式规则。带 shell 的“只读”声明不能替代实际权限限制。
- research：写范围限指定任务 research/；不会自动获得修改产品、全局设置或其他任务的权限。
- implement：用户批准后按子任务白名单、AC 和命令执行。
- trellis-check：本仓库是可写自修角色，属于批准后的执行阶段；安全/架构/范围变化交回主会话。
- 原生加载、hook trust、沙箱强制和具体账户模型可用性：本轮 UNVERIFIED。文件存在、PATH 存在和文档支持均不足以标记 PASS。

## 3. 成本分工约束

强模型保留：RCA 结论、注入/信任根、凭据与文件边界、依赖安全范围、异步生命周期、正式验收和最终 diff。

低成本模型可做：已确定的两处 YAML 修改；指定字段的双语镜像；LF/CRLF/BOM 夹具；包清单负例；确定版本集合的 lockfile 更新；运行并汇总固定检查。

交付给执行角色的最小上下文：Active task 路径、父报告发现编号、具体文件、不可变规则、AC、验证命令、需要返回的证据。若发现未知根因或扩大文件范围，停止该扩展并交强模型复核。

## 4. 官方来源

以下页面在本轮搜索并获取；本文只保留摘要，没有复制整页。工具版本升级后需复核。

- S1：Claude Code subagents，https://code.claude.com/docs/en/subagents 。独立上下文、工具约束、权限和模型配置。
- S2：Claude Code memory/import，https://code.claude.com/docs/en/memory 。用于真实 @AGENTS.md 语法；仓库该行也已直接检查。
- S3：OpenAI Codex subagents，https://developers.openai.com/codex/subagents 。项目 .codex/agents 的 TOML 字段与继承。
- S4：OpenAI Codex hooks，https://developers.openai.com/codex/hooks 。配置层与 trust；不是本机 hook 已运行的证据。
- S5：Grok Build subagents，https://docs.x.ai/build/features/subagents 。内置 plan/explore 无 shell/edit。
- S6：Kimi agents，https://www.kimi.com/code/docs/en/kimi-code-cli/customization/agents.html 。内置角色、自定义文件、tools 与分派边界。
- S7：OMP 维护方 task-agent-discovery 文档，https://github.com/can1357/oh-my-pi/blob/main/docs/task-agent-discovery.md 。.omp 原生发现与模型/权限字段。

S2 的 import 定义由官方文档与仓库引用交叉核对；原生客户端加载未实测。没有使用未经验证的“某平台不支持 hook/agent”结论。
