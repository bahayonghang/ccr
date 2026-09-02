# 修复 Overview 与 Agent Sessions 页面报错 — 设计

## Boundaries

- Overview：仅 `ccr-ui/src/main.tsx`、`ccr-ui/src/utils/startupRecovery.ts` 及其测试。不改 `dashboardPresentation.ts` 的 frontend 通道门控。
- Agent Sessions：`crates/ccr-store/src/sessions/providers.rs`（restore/validate）、`ccr-ui/src-tauri/src/services/agent_sessions.rs`（错误映射与刷新对账）、详情/i18n 表面。不改 IPC 形状、不改 archive schema、不改 provider 集合。
- 前端继续只发 `archive_id`；生成 client 仍是唯一 `invoke` 所有者。

## Overview startup handlers

当前：

```
installStartupErrorHandlers()  // disposer 丢弃
window.error → reportStartupFailure('Unhandled window error')
  → logger.error('[startup] … failed')
  → renderFatalStartup() 替换 #app
```

目标：

1. `installStartupErrorHandlers` 继续返回 disposer。`main.tsx` 必须保留该返回值。
2. React 根提交成功后卸载这两个 listener。卸载点放在壳层首次 mount 的 `useEffect`（一次性），不要用固定 timeout。
3. 卸载后的窗口错误：若仍需记录，走普通 `logger.error`，stage 不得为 Unhandled window error，且不得 `renderFatalStartup`。
4. 卸载前的启动失败路径保持不变，以便真正的挂载崩溃仍有 fallback。

不在本设计中清用户 DB。Event stream 继续渲染 frontend error；`isCoreSignal` 继续排除 `frontend`。

`undefined.toString()` 无堆栈。实施时先用卸载后的复现判断：当前树若不再抛，则只修处理器生命周期；若 Dashboard/壳层仍抛，再对调用点加空值保护，禁止无证据地改 ApexCharts 或大面积 Presentation。

## Agent Sessions restore errors

`validate_stored_source` 已区分：

| Provider 错误 | 语义 |
| --- | --- |
| session source is missing / root is unavailable | 磁盘源不在 |
| session source escaped its canonical root | 路径逃逸 |
| session source shape does not match its provider | 形状非法 |
| unknown session source variant / unknown provider | 元数据非法 |

`restore_source` 在 Tauri 层把以上全部压成 `agent_session_source_validation_failed`。改为按类映射：

- 缺失 / root unavailable → `agent_session_source_unavailable`（与 `read_message_page` 失败同码，符合合同「source unavailable」）
- 逃逸、形状、非法 member/variant/provider → `agent_session_source_validation_failed`

不要把 provider 的英文 `CcrError` 原文或 `file_path` 传给渲染器。

详情 UI：

- unavailable：标题用 missing/source-unavailable i18n，说明不展示生码，保留 Retry。
- 其它稳定码：`agentSessions.errors.<code>`，缺键时回退到通用错误文案，仍不展示生码。

列表继续自动选中第一行。刷新对账后现存会话会因 `updated_at` 排到前面；未刷新时 unavailable 空态替换当前 Error 生码。

## Refresh reconciliation

`refresh_archive_with_registry` 已在 discover 成功时把 agent 插入 `authoritative_agents`，并调用 `mark_agent_session_archive_missing_by_identity`。本机 `usage_session_source_state` 为 0 行，说明这条路径从未成功提交，或用户从未点刷新。

实施核对：

1. 刷新 job 失败时不得把未完成的 agent 标成 authoritative（已有）。
2. 补测试：registry 含现存 jsonl + DB 中一条指向已删路径的 live 行 → 刷新后现存可 detail，旧行 `missing`。
3. 不把进入页面改为自动刷新。

## Windows path containment

`validate_stored_source` 对 root 与 `physical_path` 都 `canonicalize` 后做 `starts_with` + `valid_source_shape` 的 `strip_prefix`。Windows 上 `\\?\` 前缀会导致「明明在 root 下却校验失败」。

修复保持只读校验，不接受渲染器路径：

- 在 canonicalize 之后比较前，把两条路径规范成同一前缀形态（去掉多余的 verbatim 前缀，或双方都保留后再比 component）。
- 比较必须仍是路径组件包含，禁止字符串 `starts_with` 绕过 `..`。
- 文件必须仍是 regular file。

用 tempfile 在 Windows 上测 Codex live jsonl；再加一条显式 `\\?\` 或反斜杠路径夹具。

## Compatibility

- 错误码对旧前端：多一个 unavailable 分支；未知码走通用 i18n。
- 不迁移 archive ID，不改 DTO 字段。
- 回滚：还原 handler 生命周期与 `restore_source` 映射即可；不写 DB migration。
