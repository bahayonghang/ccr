# Codex auth off 断言清单（AC6 / R6）

来源：`.trellis/spec/ccr-cli/backend/auth-off.md`

| 断言 | 迁移后位置 |
| --- | --- |
| 前端用 `can_auth_off`，不复用 profile `can_off` | `useCodexAuthPage`：`canAuthOff` 来自 `listCodexAuthAccounts` / `getCodexAuthCurrent`；`canOff` 来自 `listCodexProfiles().can_off` |
| Codex 确认语义为 `warning` | `handleAuthOff` → `surfaceNotify.confirm({ type: 'warning' })` |
| 取消不调用 IPC | confirm 返回 false 后直接 return |
| 调用现有 `codexAuthOff` wrapper | `@/api/domains/codex`，不新增 IPC |
| `changed` 决定成功文案 | `result.changed ? auth.offSuccess : auth.offUnchanged` |
| warnings 逐条 toast | `for (const warning of result.warnings) surfaceNotify.warning(warning)` |
| 日志不含凭据 | 只记录 `Failed to log out Codex official session` + `extractErrorMessage` |
| 界面测试锚点 | `data-testid="codex-auth-off"` |
