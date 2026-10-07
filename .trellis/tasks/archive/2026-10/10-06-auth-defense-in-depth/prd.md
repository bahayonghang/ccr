# Codex Auth 权限与日志纵深防御

## Goal

补齐权限加固与敏感信息呈现的纵深防御缺口。审计未发现现存泄漏；本任务降低未来回归的风险。

## Authorization and Status

2026-10-06 用户已授权父子任务顺序实施，并要求深入完善账号管理。R1–R5 按现有设计实施；不增加公开错误变体或 DTO。P3 独立检查通过后启动。

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

## Decisions

- 配额错误保留 HTTP 状态、允许的固定错误码与 `RELOGIN_REQUIRED_PREFIX`；未知响应内容不回显。TUI EN/ZH 需重新登录提示保持。
- Unchanged 的执行观测点进行版本校验的元数据权限加固；planner 保持只读，凭据字节与 mtime 不变。
- rename 回退以私有窄函数和测试 closure 注入失败验证，不增加生产故障开关或公开 trait。

## Acceptance Criteria（草案）

- [x] Windows 上 `USERNAME` 为空时仍能设置私有 DACL。
- [x] `Unchanged` 路径的 bytes 与 mtime 不变。
- [x] 认证模型的 `Debug` 输出不含 token 明文。
- [x] quota error 不含响应体，仍含状态码、错误码与需重新登录提示。
- [x] `cargo test -p ccr-codex`、`just lint-strict`、`just test` 通过；TUI 文本变更时 EN/ZH 与尺寸矩阵通过。

2026-10-06 独立检查：前四项由 Windows native core/consumer、Debug 与 loopback HTTP 回归证明。当前源码 Codex 368 passed / 2 ignored、TUI 253 passed；EN/ZH 六尺寸及 60x18 compact 的 cells/color 断言通过。scoped strict Clippy、fmt 与 secret-write policy 通过。`just lint-strict`、`just test`、`just ci` 由父任务在 P6 后运行，最后一项保持未完成。详见 `research/independent-check.md`、`research/independent-hashes.json`。

2026-10-06 最终集成验收：前段为独立检查时的历史状态。父任务最终 `just ci` 的 16 阶段通过，Codex 381 passed、2 ignored，TUI 253 passed；EN/ZH 六尺寸与 60x18 compact 断言再次通过，最后一项现已完成。Windows native 权限证据与交互式 TUI、Unix 的 NOT_RUN 边界分别保留。详见父任务 `research/integration-validation.md`。

## Dependencies

- 在 10-06-auth-destructive-backup（P3）之后实施。
