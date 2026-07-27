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
- [x] 由单一声明 codegen 出 handler + docs + frontend client（不再手工镜像）

### Typed IPC 扩展（P2-12, E4）
- [x] 建立"新增/变更 command 必须 typed"门禁（与 ci-governance 的 bindings drift 协调）
- [x] 按优先级迁移：install/process → sync → SSH → auth/provider → config write → 其余 read-only
- [x] typed domains command 边界内 `Value` 归零；生成 command client
- [x] 目标 2 sprint ≥80%，长期 100%

## Acceptance Criteria

- [x] high-risk 域（install/sync/SSH/auth/process）全部 typed，DTO/client 由 ts-rs/registry 生成
- [x] typed domains command 边界内 hand-written `Value` = 0
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
| Generated command inventory | PASS：metadata 315/315；typed 252/315（80.00%）；精确单一声明 252/252；Windows 总数 323 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml handler_registry -- --test-threads=1` | PASS：20/20 |
| `cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml runtime_policy -- --test-threads=1` | PASS：3/3；cooperative deadline、真实 future permit 生命周期、三类 timeout ownership |
| `just tauri-bindings` / `just tauri-bindings-check` | PASS：21 CLI + 6 usage + 141 desktop export tests；隔离并恢复 6 个用户已有 generated whitespace hunk 后 drift 为零 |
| `just tauri-command-inventory` / `just tauri-command-inventory-check` | PASS：manifest/docs/client 生成结果确定性一致 |
| Typed/API facade focused smoke | PASS：6 files / 15 tests；全量前端 smoke 另行覆盖 |
| `just frontend-check` | PASS：104 files / 464 smoke tests；type-check、lint、build、docs audit/build PASS |
| Tauri / workspace lint | PASS：Tauri `cargo clippy -- -D warnings`；`just lint-strict` |
| `just test` | PASS：workspace all-features tests + doctests |
| Source boundary | PASS：323 managed command attributes、0 direct `#[tauri::command]`、仅 `invokeRuntime.ts` 直接导入 core invoke |
| Scoped `git diff --check` | PASS；全局仅被明确排除的 6 个用户已有 generated TS 尾随空格阻断，原 SHA 已复核保持不变 |
| Single declaration owns exact input/output type names | PASS：manifest schema v2；252/252 typed command 的 handler、精确类型与 client declaration 同处 registry 行；三个历史 pilot 豁免已删除 |
| Generated client ownership | PASS：`stats.ts` / `install.ts` / `claudeObserver.ts` 不再 direct invoke；API facade smoke 无 typed exception |
| Runtime capability enforcement | PASS：Tauri AppManifest 从 registry manifest v2 生成 323 条 app permissions；主窗口全量、Codex tray 仅 6 条；confirmation 在 dispatch 前校验；属性宏把真实 async future 纳入 runtime policy；module/singleton permit 持有至完成；queue deadline、cooperative hard deadline、completion-aware 与 business-owned timeout ownership 均有回归测试 |

`OpenJsonValueDto` 只解决 JSON 可序列化边界；稳定响应仍需具名 DTO，不能用递归 JSON 联合冒充最终 typed 契约。`desktop-confirm:<command>` 是 UI 确认后的 action-scoped transport proof，不是秘密或授权令牌；授权由 Tauri ACL 执行，install/SSH 的高价值确认使用后端签发并消费的 opaque capability。
