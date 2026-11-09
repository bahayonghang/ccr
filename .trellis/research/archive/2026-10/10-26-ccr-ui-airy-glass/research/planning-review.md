# Research: 父子任务规划一致性复核

- Query: 父任务及四个子任务是否存在阻止可靠实施的依赖、所有权或验收矛盾？
- Scope: internal，仅读本任务已有规划与研究；没有产品检查、UI操作或代码修改。
- Date: 2026-10-26

## Findings

### P2 — 性能 harness 校准的阶段/所有权与 before 基线前置倒置

- `../10-26-ccr-ag01-foundation/design.md:12`、`:14` 要求 AG01 在任何产品改造前采集成功 before 数据。
- 父 `verification.md:47` 已确认历史 harness 需要校准选择器、路由、错误分支，并允许最小修复。
- 但 `../10-26-ccr-ag04-acceptance/design.md:7` 把 `scripts/perf/` 必要校准的所有权分给 AG04，`:12` 又要求 AG01–03 已提供 before/after 后才进入，`:17` 才执行校准。

建议：将“验证脚本前置校准 + fixture/场景固定 + before 采集”的必要脚本所有权明确放入 AG01；AG04 仅复核同一 harness、同一场景及 before/after，后期校准若改变测量语义必须重建可复现 before，不能拿旧口径比较。同步父任务顺序、AG01/AG04 design 与 implement。无需用户额外决定，这是技术依赖修正。

### P2 — 资源路由与 commands 目录的所有权边界不完整

- 父 `task-map.md:13` 以页面语义分配：所有配置/资源操作页面归 AG02，动态 command 状态归 AG03。
- `research/surface-inventory.md` 表明 `/slash-commands` 的源码也在 `features/commands/`；平台资源还包括 `/codex/slash-commands`、`/opencode/commands`，不能凭目录名归为异步命令执行。
- `../10-26-ccr-ag02-workspaces/design.md:7` 未包含 `features/commands/` 的资源管理表面，而 `../10-26-ccr-ag03-observability/design.md:7` 整个 commands 目录列入候选所有权。
- `/codex/sessions` 位于 codex 家族，AG03范围写“会话”，但其源码候选只列 agent-sessions 和各平台 Dashboard，AG02又排除观测页，仍缺明确页级归属。

建议：在 task-map 与两个子任务显式分配：slash-command/command-definition 资源管理页面归 AG02（即使源码位于 commands 目录）；`/commands/:client?` 运行工作台归 AG03；`/codex/sessions` 归 AG03。共享组件与混合样式按语义选文件、后改者协调；AG01建立初始路由登记时就落这几项，不等 AG04再发现遗漏。无需用户额外决定。

## 已满足的审查项

- 全部状态仍为 planning；父 PRD 清楚说明具体适配尚待用户批准，没有把初始任务创建授权当作实施授权。
- 暗色精确值、合成对比度、token最小增量、真实路由导出和测量工具校准可在 AG01 技术验证完成；不应额外制造产品问题阻塞规划。
- 目录与旧规范冲突被明确记录；不降低对比度/coverage等门槛，不以替换所有测试绕过审美变化。
- 本轮产品、Web、native、性能状态均明确 NOT_RUN，没有虚报验收。
- 安装、提交、发行边界清楚；含安装的聚合门未授权时不执行、记录 NOT_RUN 并跑可运行的子门；没有将子门等同聚合全绿。
- Web/原生分开，macOS等不可用平台单列，核心原生未测不宣布桌面最终验收；不存在明显证据替代问题。

## External References

无；仅复核父子规划与已有研究。

## Caveats / Not Found

- 本复核不证明实现或验收通过；只指出当前计划可执行性问题。
- 以上两个问题修正后，剩余阶段阻塞是用户对最终具体方案的批准；技术配色与测量工作仍按AG01办理。

## 主会话处理记录

2026-10-26：两项已修正。父task-map/implement/verification及AG01明确前置harness校准和before所有权；AG04只复核同口径并采after，改变口径时须重建before。父task-map和AG02/03设计显式区分slash-command资源、动态commands和codex/sessions。修正后重跑五任务结构/上下文校验通过；此记录不代表产品测试通过。
