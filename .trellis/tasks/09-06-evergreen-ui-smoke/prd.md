# 修复 Agent Sessions 路由 smoke 的类型化夹具

## Goal

修复 F1 的 route smoke 测试夹具，使 Agent Sessions 的启动/状态/分页数据遵守真实 IPC DTO，保留双语路由和错误边界验证。

## Confirmed Facts

全套 720 pass/2 fail，focused 2 pass/2 fail；ccr-ui/tests/shell/route-view-mount.smoke.test.tsx:70 匹配 session 并在第 76 行返回 []，ccr-ui/src/api/generated/agentSessions.ts:15 返回 StartSessionIndexJobResponse。

父任务需求映射：根 R2/R4/R5，工具上下文与规则项同时映射根 R3；证据见父任务 research/audit.md 与 research/harnesses.md。

## Requirements

- R1：修复 F1 的 route smoke 测试夹具，使 Agent Sessions 的启动/状态/分页数据遵守真实 IPC DTO，保留双语路由和错误边界验证。
- R2：不改 AgentSessionsView 产品容错，不改变 IPC DTO、后端返回或创建通用 mock 引擎。
- R3：审批后按本项文件边界执行，将真实验证和适用工具交父任务整合；当前保持 planning。

## Acceptance Criteria

- [x] AC1（R1）：focused route 的 4 项测试全部通过，zh-CN/en-US 均不出现 ErrorBoundary，完整 smoke 不再出现本次 2 项失败。
- [x] AC2（R1）：显式 fixture 使用生成 DTO 类型约束；start refresh 含 job_id/snapshot，list 含 items/next_cursor，不改变产品 API/错误处理。
- [x] AC3（R1）：type-check、lint:ci 和现有 agent-sessions 测试通过，完整 bun run test 通过；未删减失败断言或降低覆盖率。
- [x] AC4（R2）：diff 不超出 implement.md 白名单，未执行所列排除动作。
- [x] AC5（R3）：交付记录含适用工具、实际执行模型、检查结果与 UNVERIFIED；未经用户批准不启动。

## Dependencies

独立于 CI/OMP，优先执行；通过后把结果交规则子任务。

## Out of Scope

不改 AgentSessionsView 产品容错，不改变 IPC DTO、后端返回或创建通用 mock 引擎。
