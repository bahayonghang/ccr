# 审批后实施总计划

状态：已批准实施。按子任务依赖启动；实施、检查与回写进行中。

## 执行顺序与验收
| 次序 | 子任务 | 优先级 | 前置 | 主要验收 |
|---|---|---|---|---|
| 1 | [T08 前端依赖安全审计修复](../09-29-harness-frontend-security/implement.md) | P1 | 无 | cd ccr-ui && bun install --frozen-lockfile；just frontend-audit |
| 2 | [T05 VSIX 打包范围与本地文件排除](../09-29-harness-vsix-package/implement.md) | P1 | 无 | node --test ccr-vscode/scripts/check-package-files.test.mjs (新增)；just vscode-ci |
| 3 | [T01 OMP 上下文完整性与可复现交付](../09-29-harness-omp-context/implement.md) | P1 | 无 | bun test scripts/trellis/omp-context.test.ts；使用 git ls-files 所列源文件构建自有临时夹具，执行同一 Bun 测试；不得以本机 ignored 文件补齐 |
| 4 | [T03 Dependabot Bun 生态配置修复](../09-29-harness-dependabot-bun/implement.md) | P1 | 无 | python -m unittest scripts.ci.test_check_workflow_governance；just workflow-governance-check |
| 5 | [T04 Copilot 检查的跨平台换行兼容](../09-29-harness-copilot-crlf/implement.md) | P2 | 无 | node --test scripts/quality/check-copilot-assets.test.mjs (新增)；just copilot-check |
| 6 | [T02 五套 harness 共享说明与角色权限对齐](../09-29-harness-contract-alignment/implement.md) | P1 | T01 | python -m unittest scripts.quality.test_check_harness_contracts (新增)；python scripts/quality/check_harness_contracts.py (新增) |
| 7 | [T06 只读聚合门禁与遗漏检查接入](../09-29-harness-readonly-gates/implement.md) | P2 | T01, T02, T03, T04, T05, T08 | python -m unittest scripts.ci.test_check_workflow_governance scripts.ci.test_architecture_contract_gates；just workflow-governance-check |
| 8 | [T07 历史 CI 失败复验与诊断证据](../09-29-harness-ci-evidence/implement.md) | P2 | T06 | cargo test -p ccr-cli non_dry_run_doctor_persists_sanitized_report；just tauri-process-smoke |

## 检查点
- [x] 用户明确批准本最终计划及全部八个子任务；授权记录见 approval.json。
- [x] 从对应子任务启动，不把父任务层级当作执行依赖。
- [x] 实施前读取子任务 JSONL；低成本执行不改变判断与验收范围。
- [x] 每项保留原始失败、修复后结果和文件白名单。
- [x] 完成 T02 的共享说明和各项规范回写，标明适用工具。
- [ ] 强模型核对全部已改文件和相关正式检查，不隐藏告警或遗漏项。
- [ ] 最终 diff 仅含批准范围；检查新建文件的空白与引用。
- [ ] 提交、归档、推送和发布待相应授权。

## 审查阶段验证界限（历史基线）
本轮运行 Windows 本地 Rust、Tauri 后端、React/jsdom、扩展、文档、治理和安全检查。未运行新鲜五客户端会话的 hook/trust、原生 GUI、Linux/macOS/1.95 MSRV、Rust/Tauri coverage、release 构建或当前 HEAD 托管 CI。原 just ci 含修复副作用，未整条运行；没有将组成项检查冒充完整 CI PASS。详见 research/check-results.md。

## 审批后实施进度

当前结果见 [实施与验证记录](research/implementation-results.md)。本节后续结果不覆盖上方审查阶段的原始失败记录。
