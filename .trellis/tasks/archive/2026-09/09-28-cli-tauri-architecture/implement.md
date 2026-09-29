# 父任务执行与最终验收计划

## 本轮已完成的规划交付

- [x] 阅读当前规范、依赖和代表调用链，保存三域研究与治理证据。
- [x] 执行只读基线和隔离前端反例，记录正式 lint 的已有阻断。
- [x] 将问题映射到 11 个子任务，建立 PRD、设计、执行计划和 context manifests。
- [x] 独立规划复核的六项修订闭合，12 个任务机械检查及上下文校验通过；结果见 research/planning-validation.md、plan-precheck.json 和 context-validation.json。

## 后续实施门槛

- [x] 用户于 2026-09-28 批准最新规划；子任务按依赖激活并派实施代理。
- [ ] 核对当时的 baseline、其他任务与工作区；父任务只协调，按子任务单独实施。
- [ ] 第一阶段：T01/T05/T06/T08。
- [ ] 第二阶段：T02、T11。
- [ ] 第三阶段：T03/T04/T07；T03 与 T08 就绪后 T09。
- [ ] 最后 T10：完整契约矩阵、规范与门禁收敛。

## 每个子任务的必需证据

对 R1/AC1 至 R11/AC11：逐项运行子任务全部验收，记录旧反例、修复测试、平台、测试数、退出码、commit、剩余限制。前置依赖通过对应契约后才进入下游。研究中的缺陷刻画测试要迁为正确行为回归并反转旧失败断言，不把旧缺陷测试通过当作已修复。

## 最终综合验证

以下命令仅在获批实施、处理原有工作区边界后执行：

```text
just version-check
just fmt-check
just lint-strict
just test
just frontend-check
just tauri-ci
just ci
git diff --check
```

just ci 有 version-sync/fmt 等改写；本轮未运行。最终门禁在 clean worktree 或明确保留用户改动的受控工作区执行。Formal lint、受控排除诊断、native、visual 分开记录；任何正式失败仍为失败，不降低规则或删用户文件。

## 完成条件

- [ ] 所有父 AC 和子 AC 通过；Local/WSL/SSH 按支持范围验证。
- [ ] Windows/Linux/macOS 的敏感权限与进程清理按相关契约通过。
- [ ] 受影响 Web 交互与视觉确认、原生桌面 smoke 完成；未验证不宣称发布就绪。
- [ ] 规范和生成物同步、独立检查通过、原有 Insights 任务和文件状态无越权变化。
- [ ] 依用户后续授权和 Trellis 流程处理提交与归档。
