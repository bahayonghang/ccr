# Implement: ccr codex auth relogin

## 顺序

1. `codex_process_service.rs`：参数化 `run_daemon_command_at`，新增 `stop_codex_daemon` / `DaemonStopOutcome`，导出。补测试。
2. `auth_off.rs`：抽出 File 分支共用函数，新增 `codex_local_auth_off`。补拒绝路径测试。
3. `relogin.rs` + `mod.rs` + `CodexAuthAction::Relogin` + `dispatch.rs`。补继续/中止判定测试。
4. `help_config.rs` 与两份命令参考文档。

## 验证

```bash
just version-check
just fmt-check
just lint-strict
just test
cd docs && bun install --frozen-lockfile && bun run build
```

手工检查（file 存储、有守护进程）：`ccr codex auth relogin` 后 `~/.codex/backups` 出现 auth 备份，守护进程 PID 消失，`codex login` 被拉起；codex 日志 `logs_2.sqlite` 在该时段没有 `account/logout`。

## 回滚点

每步完成后 `just test` 通过再进入下一步；整体单提交，可 `git revert`。
