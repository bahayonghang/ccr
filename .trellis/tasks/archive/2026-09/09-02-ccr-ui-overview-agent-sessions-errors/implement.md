# 修复 Overview 与 Agent Sessions 页面报错 — 执行计划

## Ordered checklist

1. **启动处理器生命周期**
   - 在壳层首次 mount 卸载 `installStartupErrorHandlers`。
   - 增加测试：卸载后 `ErrorEvent` 不替换 `#app`、不写 `[startup] Unhandled window error`；卸载前仍走 `reportStartupFailure`。
   - 若当前树仍能稳定复现 `reading 'toString'`，只修该调用点。

2. **Restore 错误映射**
   - 拆开 `restore_source` 的统一 `map_err`：缺失/root unavailable → `agent_session_source_unavailable`；形状/逃逸/非法 variant → `agent_session_source_validation_failed`。
   - 在 `agent_sessions` 服务测试中覆盖删除文件与错误扩展名两条。

3. **Windows 路径包含**
   - 修正 `validate_stored_source` / `valid_source_shape` 在 canonicalize 后的 root 包含判断。
   - `ccr-store` 测试：Codex live jsonl 在 tempfile 下 restore 成功；显式覆盖反斜杠或 verbatim 前缀。

4. **刷新对账回归**
   - 扩展 `refresh_archive_with_registry` 测试：现存源 upsert + 缺失旧行 `missing` + 随后 `get_detail` 对现存源成功。
   - 不改自动刷新产品行为。

5. **详情 i18n 空态**
   - en-US / zh-CN 增加 unavailable 与稳定码文案。
   - `AgentSessionTranscript` 对 unavailable 走 missing 空态，不把生码当 description。
   - smoke：断言不渲染 `agent_session_source_validation_failed` 原文。

6. **验证**
   - 跑下方窄命令。桌面 Local 点 Incremental refresh，打开当天 Codex jsonl；web preview 无 Tauri 时记 `UNVERIFIED`。
   - 不删除用户 `log_entries`。

## Validation commands

不要加 `rtk` 前缀。Windows 上用 `python` 调 Trellis 脚本，用 `just` / `cargo` / `bun` 跑门禁。

```powershell
cargo test -p ccr-store sessions::providers --no-fail-fast
cargo test --manifest-path ccr-ui/src-tauri/Cargo.toml agent_sessions --no-fail-fast

Set-Location ccr-ui
bun run test:smoke -- tests/agent-sessions/agent-sessions.smoke.test.tsx
bun run test:i18n
bun run type-check
bun run lint
Set-Location ..

just frontend-check-quick
```

Agent Session / 启动 handler 合同要求时再跑 `just ui-check`。`just ci` 仅在收口需要全量证据时运行。

桌面视觉：Local 环境 `/` Event stream 不再被新的 `[startup] Unhandled window error` 刷屏；`/agent-sessions` 刷新后能打开现存会话。Tauri 未跑时写 `UNVERIFIED`。

## Risky files and rollback

- `ccr-ui/src/main.tsx`、`startupRecovery.ts`：卸载过早会丢掉真正的启动崩溃 fallback。先测再改。
- `crates/ccr-store/src/sessions/providers.rs`：路径包含是安全边界。回滚该函数即可，不要改成信任渲染器路径。
- `ccr-ui/src-tauri/src/services/agent_sessions.rs`：错误映射与刷新对账；失败时只回退映射，不改 schema。
- i18n 键：en/zh 与 keys.txt 必须一起改。

## Follow-up before `task.py start`

- 规划已收敛，无未决产品问题。
- Oh My Pi 为 sub-agent-dispatch：批准后、`task.py start` 前再整理 `implement.jsonl` / `check.jsonl`（至少各一条真实 spec/research，不能只留 seed）。
- 用户必须在本最终摘要之后的消息里明确批准实施，才运行 `task.py start 09-02-ccr-ui-overview-agent-sessions-errors`。
