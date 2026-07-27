# Typed IPC 扩展与 Command 能力元数据

> 父任务：`07-24-audit-remediation` ｜ 覆盖：P2-11、P2-12 ｜ 报告 Epic A4/E4

## Goal

把 command registry 从 handler list 升级为 capability manifest（带 risk/timeout/audit 元数据），并把 typed IPC 从 ~8% 试点扩展到高危域全覆盖。

## 背景 / 证据（已核实）

- `ccr-ui/src-tauri/src/commands/handler_registry.rs:6-11` — `CommandModule` 仅 key/title/commands，315/323 commands 无 risk/authorization/rate-limit/timeout/input schema（P2-11）
- typed IPC 已覆盖 Usage V2（17）+ Claude Observer（9）+ install（8）= 34/315 ≈ 10.79%，其余仍有大量手写 TS / `Value`（P2-12）
- 前端当前有 346 个 direct `invoke` 调用，分布在 26 个源文件；registry metadata 覆盖 0/315
- spec：`.trellis/spec/ccr/backend/typed-ipc-bindings.md`（typed IPC 契约）；ts-rs 生成绑定在 `ccr-ui/src/types/generated/`

## Requirements

### Command 能力元数据（P2-11, A4）
- [x] `CommandDescriptor` 扩展 risk / input schema / output / timeout / concurrency / confirmation / audit 字段
- [ ] 由单一声明 codegen 出 handler + docs + frontend client（不再手工镜像）

### Typed IPC 扩展（P2-12, E4）
- [x] 建立"新增/变更 command 必须 typed"门禁（与 ci-governance 的 bindings drift 协调）
- [x] 按优先级迁移：install/process → sync → SSH → auth/provider → config write → 其余 read-only
- [ ] typed domains 内 `Value` 归零；生成 command client
- [x] 目标 2 sprint ≥80%，长期 100%

## Acceptance Criteria

- [ ] high-risk 域（install/sync/SSH/auth/process）全部 typed，DTO/client 由 ts-rs 生成
- [ ] typed domains 内 hand-written `Value` = 0
- [x] binding drift 为 required check
- [x] Command risk metadata 覆盖 315/315（生成报告核对）
- [x] `just frontend-check` + `just lint-strict` + `just test` 通过

## Out of Scope

- 不改变已经稳定且无安全需求驱动的 Tauri command 名称
- 不新增 generic `Value`/raw invoke 兼容逃生口
- 不在 install/sync/SSH 后端契约稳定前重复迁移相应域

## Notes

- **顺序约束**：建议在 install-plan-handle / webdav-sync / ssh-hardening 整改落地后再迁移对应域的 typed IPC，避免接口变更导致迁移返工
- 大规模 TS diff 风险：per-domain compat shim，不允许新 generic escape hatch（报告 §5 2E 回滚）
- 触发 frontend-quality-reviewer + tauri-ipc-reviewer 复查

## Verification evidence (2026-07-27)

| Evidence | Result |
| --- | --- |
| Generated command inventory | PASS：metadata 315/315；typed 252/315（80.00%）；Windows 总数 323 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml handler_registry -- --test-threads=1` | PASS：15/15 |
| `just tauri-bindings` / `just tauri-command-inventory` | PASS：141 desktop export tests；生成结果确定性一致 |
| Typed/API facade focused smoke | PASS：8 files / 31 tests；另有 typed JSON boundary 5 tests |
| `just frontend-check` | PASS：101 files / 453 smoke tests；docs audit/build PASS |
| `just lint-strict` / `just test` | PASS |
| Single declaration owns exact input/output type names | INCOMPLETE：descriptor 仍只有 module-level `Generated` / `LegacyJson`；client 类型仍由手写 generator 字符串维护 |
| Runtime capability enforcement | INCOMPLETE：audit metadata 进入运行时日志，但 timeout/concurrency/confirmation/authorization 仍未由统一后端策略执行 |

本状态仅为已验证 checkpoint，不得归档本子任务。`OpenJsonValueDto` 只解决 JSON 可序列化边界；稳定响应仍需具名 DTO，不能用递归 JSON 联合冒充最终 typed 契约。
