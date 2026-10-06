# 参考 codex-auth 审计并加固 Codex Auth 实现

## Goal

对照 Loongphy/codex-auth，审计 CCR Codex Auth 的存储、身份、备份、权限、恢复与导入导出实现，产出带证据的审计报告；在保持运行逻辑不变的前提下应用安全加固修复。

## Authorization and Status

2026-10-06 用户要求"参考 codex-auth 优化实现（保持运行逻辑不变）"。本版为规划产物，实施需评审后 `task.py start`。执行顺序：本任务不依赖子任务 1/2 的代码，可在其后启动以避免文件冲突（见 implement.md）。

## Background and Confirmed Facts

参考机制见父任务 research/codex-auth-reference.md。初步对照已识别差异候选（design.md 维度表展开）：
- CCR 备份在每次写前创建（无内容变化去重），保留为 auth 前缀共享池 10 个（所有 auth.*.bak 标签共用，codex_config.rs:41、:271-299）；codex-auth 仅内容变化时备份、保留 5。
- CCR 身份仅 account_id（codex_auth_service.rs:714、codex_oauth_token_service.rs:333）；codex-auth 为 user_id::account_id。
- 权限加固：CCR 写路径有 ensure_private_permissions，读取/无变化路径覆盖度待审计。
- 注册表未知字段保留、导入/导出、API-key 指纹策略待审计。

## Requirements

| ID | 要求 |
| --- | --- |
| R1 | 审计报告：维度 × CCR 现状（file:line）× codex-auth 做法 × 风险 × 修复类别；输出到本任务 research/ 或 checks/。 |
| R2 | 修复类别 A（不改变运行逻辑与用户可见文本，仅日志级文本可调整）：内容无变化不备份（去重命中必须返回既有相同备份路径，保持 commit_plan 回滚语义）、备份保留不得低于现状（auth 前缀共享池 10 个）、权限加固覆盖、日志脱敏缺口、原子写覆盖、dead code；错误信息一致性仅限日志级文本，不得改变 TUI 可见 toast/quota error 文本。 |
| R3 | 修复类别 B（行为变化候选，含用户可见文本变更、保留池缩减、身份模型）：单独列出并转为后续任务提案（含位置、证据、建议），记录在审计产物；不转交子任务 1；本任务不实施。 |
| R4 | 每个类别 A 修复有回归测试或等价验证；既有测试语义不变。 |
| R5 | 结论分级（高/中/低）且每条 finding 带可复核证据（file:line 或实验输出）；未验证项标 NOT_RUN。 |

## Acceptance Criteria

- [ ] AC1（R1）：审计报告覆盖 design.md 全部维度，每条 finding 带证据与分级。
- [ ] AC2（R2）：类别 A 修复完成，`cargo test -p ccr-codex`（及相关 crate）通过，无用户可见行为变化（对比既有断言）；保留池数量不缩减；TUI 可见错误文本不变；备份去重含回滚测试（写失败时回滚到既有备份路径）。
- [ ] AC3（R3）：类别 B 清单存在，以「后续任务提案」形式输出（位置/证据/建议）；不要求子任务 1 接收。
- [ ] AC4（R5）：无未标注证据的结论。
- [ ] AC5：`just lint-strict`、`just test` 通过。

## Out of Scope

- 行为变更修复（切换/同步语义 → 子任务 1）；面板显示（子任务 2）。
- 修改第三方 API 行为、代理转发、自动轮换。

## Risks and Evidence Boundaries

- "保持运行逻辑不变"边界：用户可见文本变更（toast/quota error）、恢复来源保留池缩减一律类别 B；文件副作用类变更（如备份去重）默认类别 A，但必须保持 commit_plan 回滚契约（去重命中返回既有备份路径）并在报告说明；其余影响认证/配额/切换语义的变更一律类别 B。
- 审计范围以静态代码与合成实验为限；真实账户行为不纳入。
