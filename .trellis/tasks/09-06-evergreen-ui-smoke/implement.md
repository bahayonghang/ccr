# 执行计划

## 文件白名单

- ccr-ui/tests/shell/route-view-mount.smoke.test.tsx

## 顺序

独立于 CI/OMP，优先执行；通过后把结果交规则子任务。

- [x] 获取本子项的明确实施批准，检查最新计划，再 task.py start。
- [x] 阅读 implement.jsonl/check.jsonl 的真实规范、父报告和 design.md。
- [x] 重现对应失败/缺口，实施最小变更，保留有意义的正负回归。
- [x] 逐条运行下列检查，记录命令/cwd/退出码；预期失败的负例以非零为通过。
- [x] 强模型独立审查，向父任务/规则子任务交付适用工具与证据；缺证据明确列出。

## 检查

~~~text
cd ccr-ui
bun run test:smoke -- tests/shell/route-view-mount.smoke.test.tsx tests/agent-sessions/agent-sessions.smoke.test.tsx
bun run type-check
bun run lint:ci
bun run test
~~~

有生成/安装副作用的聚合命令先检查组成步骤；仅使用已有依赖。本地检查不证明当前 hosted 或真实客户端通过。必要检查缺条件时该项验收保持未完成，不能用其他命令的通过替代。

## 完成和权限

本项 AC 与父任务已批准集的交付约束均满足才算完成。用户 `/goal` 已批准本子任务在 check PASS 后提交并归档；仍不发布、不编辑用户全局文件。不改 AgentSessionsView 产品容错，不改变 IPC DTO、后端返回或创建通用 mock 引擎。

## 交付记录

- 适用工具：Cursor 主会话派发 `trellis-implement` / `trellis-check`；不改产品容错。
- 实施：trellis-implement（子代理），验证：trellis-check 判定 PASS。
- 检查证据：`check-ledger.md`。原始 focused smoke 由 exit=1 变为 exit=0。
- UNVERIFIED：hosted CI、原生 Tauri/真实本地数据集、五套 harness 实机会话。
