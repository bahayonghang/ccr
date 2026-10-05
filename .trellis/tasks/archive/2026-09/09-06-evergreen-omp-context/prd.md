# 补齐 OMP Trellis 三件套上下文

## Goal

修复 F5，使 OMP Trellis 主会话与各子角色获得当前复杂任务的 PRD、设计和执行计划，并在自动注入不可用时明确补读。

## Confirmed Facts

.omp/extensions/trellis/index.ts:274 仅注入 prd/info/jsonl；:456 的 session_start 是实际注入入口，按 PI_BLOCKED_AGENT 识别子角色。

父任务需求映射：根 R2/R4/R5，工具上下文与规则项同时映射根 R3；证据见父任务 research/audit.md 与 research/harnesses.md。

## Requirements

- R1：修复 F5，使 OMP Trellis 主会话与各子角色获得当前复杂任务的 PRD、设计和执行计划，并在自动注入不可用时明确补读。
- R2：不改 Trellis 上游仓库或用户全局配置、不新增依赖/信任机制、不重构上下文缓存/压缩流程、不擅自运行付费模型。
- R3：审批后按本项文件边界执行，将真实验证和适用工具交父任务整合；当前保持 planning。

## Acceptance Criteria

- [x] AC1（R1）：默认 extension 入口的行为测试证明主会话/implement/check/research 均收到存在的 prd/design/implement 标记；轻量任务缺 design/implement 时正常读取 PRD。
- [x] AC2（R1）：implement 仅读 implement.jsonl，check 仅读 check.jsonl，research 不自动读两者；现有越界路径拒绝仍成立。
- [x] AC3（R1）：三份 agent 说明要求自动上下文缺失时从明确 task path 拉取三件套和匹配manifest；无新增信任根或隐式写权限。
- [x] AC4（R1）：本地行为测试通过；真实 OMP 会话的加载/实际模型证据另列，未执行必须保留 UNVERIFIED，不能称运行对齐完成。
- [x] AC5（R2）：diff 不超出 implement.md 白名单，未执行所列排除动作。
- [x] AC6（R3）：交付记录含适用工具、实际执行模型、检查结果与 UNVERIFIED；未经用户批准不启动。

## Dependencies

可以独立实现；规则子任务的 OMP 部分在本项结果确定后收口。

## Out of Scope

不改 Trellis 上游仓库或用户全局配置、不新增依赖/信任机制、不重构上下文缓存/压缩流程、不擅自运行付费模型。
