# Codex Auth 权限与日志纵深防御

## Goal

补齐权限加固与敏感信息呈现的纵深防御缺口。审计未发现现存泄漏；本任务降低未来回归的风险。

## Authorization and Status

2026-10-06 用户要求为 10-06-reference-hardening 审计报告的后续任务提案 P1–P6 创建 Trellis 任务。本版为规划种子，未开始实施；需求与待决问题需经 brainstorm 收敛，复杂任务补齐 design.md 与 implement.md，并经用户评审后 `task.py start`。规划基线：dev @ 73b47563。证据锚点取自审计报告（.trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md），实施前需复核行号。

## Background and Evidence

- F24（低）：`utils.rs:87-118` 的 Windows 分支读取 `USERNAME` 后执行 `icacls /inheritance:r /grant:r`，失败时静默跳过。`ccr-core` 的 `AtomicWriter::secret` 已使用进程令牌用户 SID 的 DACL。
- F20（低）：`codex_oauth_token_service.rs:455-457` 在 tokens 相同时返回 `Unchanged`，不加固权限。codex-auth 在内容相同时也执行 `hardenSensitiveFile`。规范要求 `Unchanged` 时 bytes 与 mtime 不变。
- F23（低）：`codex_auth_service.rs:1529-1540` 在同目录 `fs::rename` 失败时回退到 `fs::copy` + `remove_file` + `ensure_private_permissions`，写入后再加固，存在权限窗口。
- F17（低）：`models/codex_auth.rs:572-589` 的 `CodexAuthJson`、`CodexAuthTokens` 派生明文 `Debug`；`rg` 未发现现存 Debug 输出。
- F18（低）：`openai_quota_core.rs:446-454`、`:486-494` 的错误文本拼接前 200/300 字节响应体，经 quota error 显示在 TUI。规范对桌面 OAuth 控制器要求只报告状态或固定诊断，不报告响应体。

## Requirements（草案）

| ID | 要求 |
| --- | --- |
| R1 | `ensure_private_permissions` 的 Windows 分支改用 SID DACL 帮助函数，不依赖 `USERNAME` 与 `icacls`。 |
| R2 | `Unchanged` 路径只做元数据级权限加固，不改变 bytes 与 mtime；Windows 上不为每次观测启动外部进程。 |
| R3 | rename 回退分支改为私有原子写。 |
| R4 | 认证模型实现脱敏 `Debug`。 |
| R5 | 配额与刷新错误只保留 HTTP 状态码与错误码，不含响应体（改变 TUI 可见文本，EN/ZH 同步）。 |

## Open Decisions

- R5 改变 quota error 文本，需用户确认；`RELOGIN_REQUIRED_PREFIX` 与错误码识别（如 `[refresh_token_invalidated]`）必须保留。
- R2 是否值得在读路径增加元数据副作用。
- R3 的回退分支难以在测试中稳定触发，需要确定等价验证方式。

## Acceptance Criteria（草案）

- [ ] Windows 上 `USERNAME` 为空时仍能设置私有 DACL。
- [ ] `Unchanged` 路径的 bytes 与 mtime 不变。
- [ ] 认证模型的 `Debug` 输出不含 token 明文。
- [ ] quota error 不含响应体，仍含状态码、错误码与需重新登录提示。
- [ ] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过；TUI 文本变更时 EN/ZH 与尺寸矩阵通过。

## Dependencies

- 在 10-06-auth-destructive-backup（P3）之后实施。
