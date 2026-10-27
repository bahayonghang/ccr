# 任务树和依赖
父任务负责需求、设计决策和最终集成验收；不作为大包实施目标。所有任务当前为planning。

| 子任务 | 职责 | 前置 | 需求 |
|---|---|---|---|
| 10-26-ccr-ag01-foundation | 基线、规范、令牌治理、外壳/首屏、主题兼容 | 用户批准最新方案 | R1 R2 R4 R6 R7 R8 |
| 10-26-ccr-ag02-workspaces | 公共控件、配置/平台设置/认证/资源/同步/环境页面 | AG01通过 | R2 R3 R5 R6 R8 |
| 10-26-ccr-ag03-observability | 首页/Insights、平台概览、Usage/预算/价格、会话/日志/命令/签到/托盘 | AG02公共组件稳定 | R2 R3 R5 R6 R7 |
| 10-26-ccr-ag04-acceptance | 全路由、主题/状态矩阵、性能、原生及最终门禁 | AG01–03证据齐全 | R1–R8 |

顺序AG01→AG02→AG03→AG04。父子关系不是自动依赖调度，执行者检查前置证据。共享tokens/shell归AG01；primitives/shared editor归AG02。AG03需要变更公共文件时协调，不并行抢写。

详细路由家族见 research/surface-inventory.md。AG02负责所有配置操作页面，包括converter、sync、wsl、ssh、Skillport迁移引导；AG03负责各平台首页（不包括平台设置）、command/checkin动态状态和独立tray。AG04从真实flattenCatalog导出每条path/id（包括redirect），核对loader实际页面，不能把placeholder视为成功。

目录重叠以路由职责裁定：`/slash-commands`、各平台slash-commands、`/opencode/commands`属于AG02资源管理；`/commands/:client?`属于AG03动态命令工作台。`/codex/sessions`和`/agent-sessions`均归AG03。同目录不同页面不等于共享文件可并行修改。

性能harness首次校准与成功before采样归AG01，必须先于视觉修改；AG04只复核该方法并采after/完成比较，不承担首次建立基线的前置工作。

登记每个页面的path/id、参数fixture、子任务、主题、状态、截图、行为检查和PASS/FAIL/NOT_RUN原因。新增发现的路由补入矩阵，不以遗漏为由跳过。
