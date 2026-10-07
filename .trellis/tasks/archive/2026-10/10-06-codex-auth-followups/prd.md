# Codex Auth 参考审计后续加固（P1–P6）

## Goal

承接审计报告第 5 节类别 B 后续任务提案 P1–P6。这些提案改变运行逻辑、保留数量、命令结果或用户可见文本，因此不在 10-06-reference-hardening 范围内实施。父任务负责任务地图、子任务顺序与跨子任务验收；父任务本身无直接实现工作。

## Authorization and Status

2026-10-06 用户授权按顺序实施现有父子任务；实施顺序为 P2 → P1 → P3 → P4 → P5 → P6。父任务已进入集成协调阶段，子任务在设计审阅后逐项启动。P1 的缺失身份规则已确认：先由快照补全，完整身份仍不可得时跳过同步。P3 已确认保留全部旧备份，不运行清理；P6 已确认拒绝冲突输入、保留现有加密导出规则；P5 首先交付评估。用户补充要求修复旧账号查询，并参考 ref/repo/cockpit-tools 深入分析账号生命周期。详情见 execution-status.md。提交、归档、推送、PR 和发布等待单独授权。

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Task Map

| 提案 | 子任务 | 来源 finding | 初始优先级 |
| --- | --- | --- | --- |
| P1 身份模型 | 10-06-auth-identity-key | F12 | P2 |
| P2 注册表版本门与未知字段 | 10-06-auth-registry-schema | F14 | P2 |
| P3 备份与破坏性操作 | 10-06-auth-destructive-backup | F10、F11、F21、F22 | P2 |
| P4 权限与日志纵深防御 | 10-06-auth-defense-in-depth | F17、F18、F20、F23、F24 | P3 |
| P5 快照文件命名 | 10-06-auth-snapshot-naming | F13 | P3 |
| P6 导入/导出 | 10-06-auth-import-export | F15、F16 | P3 |

## Ordering Constraints

父子结构不表示依赖。以下顺序也写入各子任务 PRD：

- P2 先于 P1：P1 向注册表增加身份字段；P2 的版本门或未知字段保留防止较旧 CCR 重写时丢失该字段。
- P1 先于 P5：P5 的文件名由 P1 的身份键派生。
- P1 先于 P6：P6 的导入校验使用 P1 的身份推导规则。
- P3 先于 P4：两者都修改 `rename_account`（F22 在 P3，F23 在 P4），按顺序实施以避免同文件冲突。

## Cross-Child Acceptance Criteria

- [ ] 每个子任务各自验收通过并归档；决定不实施的子任务记录决定与原因后归档。
- [x] 子任务之间的改动遵守 Ordering Constraints。
- [x] 改变用户可见文本的子任务同步 EN/ZH，并更新对应断言。
- [x] 保持私有权限、原子写、secret 脱敏、backup-before-destructive-change；错误变体集冻结（ccr-error-freeze）。
- [x] 集成复核：`just lint-strict`、`just test` 通过；涉及 TUI 文本时 Codex Auth TUI EN/ZH 与尺寸矩阵通过。
- [x] 未推送、未创建 PR、未发布（除非用户另行要求）。

2026-10-06 本地技术验收：P1/P2/P3/P4/P6 独立检查与最终 `just ci` 通过；P5 评估验收通过，建议暂缓迁移，迁移未授权。最终 CI 16 阶段 PASS，Rust workspace 2038 passed、0 failed、16 ignored。TUI EN/ZH 六尺寸与 compact 矩阵通过。批准变更路径的权限、写入、备份和诊断合同及剩余边界见 `research/integration-validation.md`。首项包含归档要求，继续保持未完成；各任务保留 `in_progress`。

## Out of Scope

- 10-06-reference-hardening 已实施的类别 A 修复。
- 修改 codex CLI 本体、拦截外部 `codex login`、代理转发、自动账号轮换。
