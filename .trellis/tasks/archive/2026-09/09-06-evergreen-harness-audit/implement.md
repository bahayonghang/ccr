# 可批准执行计划

## 当前状态

五个子任务已实施、check PASS、独立提交并归档。父任务在 `just ci` 通过前保持未归档。

## 批准并已完成的子任务

用户 `/goal` 批准全部五项（含 P2）。归档路径：

- `archive/2026-09/09-06-evergreen-ui-smoke`
- `archive/2026-09/09-06-evergreen-ci-verdict`
- `archive/2026-09/09-06-evergreen-omp-context`
- `archive/2026-09/09-06-evergreen-harness-rules`
- `archive/2026-09/09-06-evergreen-ci-history`

## 执行步骤

- [x] 记录用户批准的子任务名单，再对对应子任务 task.py start；不启动未批准任务。
- [x] 每次派工注明活动子任务、独占文件、权限、真实模型；review阶段只读，实施后的check才允许在本范围self-fix。
- [x] 先最小回归、再子系统门；遇到额外失败先判断归属，不并入无关修复。
- [x] 所有批准项交强模型独立复核，不把廉价模型输出直接当最终结论。
- [x] 规则任务合并批准项的工具适用范围、验证和缺证据到项目说明/skills。
- [x] 执行跨模块终验 `just ci`，记录通过/失败/未执行；不得用历史CI结果替代当前HEAD。
- [x] 父任务规划产物提交后归档（仅在 `just ci` exit 0 之后）。

## 待批准批次

| 顺序 | 子任务与文件 | 必过检查 | 执行分工 |
|---|---|---|---|
| 1 | evergreen-ui-smoke：ccr-ui/tests/shell/route-view-mount.smoke.test.tsx | focused route + agent-sessions smoke、type-check、lint:ci、完整 bun run test | 强模型定 DTO；低成本档局部改动；强模型复核 |
| 1 | evergreen-ci-verdict：.github/workflows/vscode-ci.yml；scripts/ci/ci_surface_policy.py；现有治理测试 | shell失败/成功管道、path正负用例、workflow治理、vscode覆盖率 | 强模型定传播/消费者；低成本档实施 |
| 2 | evergreen-omp-context：.omp extension、三代理说明、scripts/trellis/omp-context.test.ts（新） | bun test scripts/trellis/omp-context.test.ts，含缺文件/角色分派/信任边界回归；真实OMP运行证据单列 | 强模型设计与review；界定后可低成本档写测试/局部实现 |
| 3 | evergreen-harness-rules：AGENTS/CLAUDE/code_map、docs/agents/harnesses.md（新）、列明skills | 逐项源码与官方来源核对、docs audit/build、diff check；不新造文档validator | 强模型确认事实；低成本档同步 |
| 独立 P2 | evergreen-ci-history：自身 research/verification.md | 现有原参数复核、保存首故障/环境/退出码，未复现明确记录 | 强模型诊断；低成本档仅收集 |

每个子任务的完整文件白名单与命令在自身 implement.md。用户可只批准前四项。P2 默认不包含产品修复；如复现需要改语义或生产代码，提交具体修复方案再批准。

本次推荐批准组合为四个 P1，已包含 rules 回写责任。若只批准 UI/CI/OMP 的某个子集而未批准规则回写，允许完成该子项局部验证，但父任务交付保持“待批准的最小回写”，不得自动启动完整 rules 改造或声称已满足回写要求。提交该子项结果时同时给出最小回写差异供批准；只有明确批准的回写范围才能落盘到现有说明/skill。

## 执行步骤

- [ ] 记录用户批准的子任务名单，再对对应子任务 task.py start；不启动未批准任务。
- [ ] 每次派工注明活动子任务、独占文件、权限、真实模型；review阶段只读，实施后的check才允许在本范围self-fix。
- [ ] 先最小回归、再子系统门；遇到额外失败先判断归属，不并入无关修复。
- [ ] 所有批准项交强模型独立复核，不把廉价模型输出直接当最终结论。
- [ ] 规则任务合并批准项的工具适用范围、验证和缺证据到项目说明/skills。
- [ ] 执行跨模块终验，记录通过/失败/未执行；不得用历史CI结果替代当前HEAD。
- [ ] 向用户交付结果；提交、归档和发布依另行明确授权。

## 规划检查

对六个任务分别 task.py validate；根任务运行 plan_precheck.py --include-descendants；独立强模型审阅整个树。结构通过不表示产品通过或实施授权。
