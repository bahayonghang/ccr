# CCR 常青项目五套 harness 审查与改造

## 目标
使 CCR 的本地门禁、托管流程、五套 agent 入口及团队说明一致，并保持安全和验证边界。审查计划已获批准，当前阶段按子任务实施、验证和回写。

## 基线与授权
- 审查日期：2026-09-29，America/Chicago。
- 分支 dev，HEAD fe9d65972376dbb4c332fde9860c1e42ff2f2005；开始时工作区干净。
- 用户已在最终计划后回复「批准实施」，授权 T01–T08 的代码、检查、说明和 skill 回写。
- 审查阶段仅新增父子任务；实施阶段按子任务启动。提交、归档、推送、PR 和发布未获授权。

## 要求
- R1：核对代码结构、关键入口、领域归属与关键安全边界。
- R2：实际执行现有测试，保留命令、退出码、计数、失败和未验证边界。
- R3：追踪本地与托管失败到证据支持的责任层；根因未查明时明确记录。
- R4：五工具分别说明官方能力、本仓库集成、授权和强/低成本模型分工。
- R5：建立优先级、文件白名单、前置依赖和必须通过的检查。
- R6：审查 AGENTS、CLAUDE、工具入口、skill 和工作流中的冲突或缺失。
- R7：批准项完成后回写项目说明、已跟踪 skill 或规范，并标注适用工具。

## 父级验收
- AC1（R1–R3）：research/audit-report.md 与 check-results.md 记录当前基线及失败；不以旧 SHA 结果代替 HEAD 验证。
- AC2（R4、R6）：harness-matrix.md 和 T02 覆盖五工具；只读 reviewer、research 输出及可写 check 权限无混淆。
- AC3（R5）：8 个子任务各有 PRD、设计、实施计划和真实 JSONL 上下文；子任务验收均可独立核对。
- AC4（R5、R7）：批准后按依赖实施并完成对应检查与知识回写；安全公告或正式门槛未关闭时保留未完成状态。
- AC5（全部）：未经后续明确批准不进入实施；没有全局账户、模型、trust、推送或发布副作用。

## 子任务与优先级
- T01 / P1：[OMP 上下文完整性与可复现交付](../09-29-harness-omp-context/prd.md) — F02、F03
- T02 / P1：[五套 harness 共享说明与角色权限对齐](../09-29-harness-contract-alignment/prd.md) — F03、F04
- T03 / P1：[Dependabot Bun 生态配置修复](../09-29-harness-dependabot-bun/prd.md) — F05
- T04 / P2：[Copilot 检查的跨平台换行兼容](../09-29-harness-copilot-crlf/prd.md) — F06
- T05 / P1：[VSIX 打包范围与本地文件排除](../09-29-harness-vsix-package/prd.md) — F07
- T06 / P2：[只读聚合门禁与遗漏检查接入](../09-29-harness-readonly-gates/prd.md) — F08
- T07 / P2：[历史 CI 失败复验与诊断证据](../09-29-harness-ci-evidence/prd.md) — H01、H02、H03、O01
- T08 / P1：[前端依赖安全审计修复](../09-29-harness-frontend-security/prd.md) — F01

## 范围外
不重构 CLI/Tauri/UI 架构，不改品牌视觉，不自动批量升级依赖，不新增五套重复规则正文，不将本轮原始会话导入 Basic Memory，不提交或归档。新 GUI 操作、账户授权、外部工作流触发另行确认。

## 决策状态
执行设计采用仓库现有 just、工作流和安全规范。最终计划已获用户批准；按已确定依赖执行，范围变化另行记录。
