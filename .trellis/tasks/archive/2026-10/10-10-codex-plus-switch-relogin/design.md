# Design: ccr codex auth relogin

## 流程

```
relogin_command()
  1. store = detect_codex_credential_store()
     store != File  → Err（不触碰任何状态，不启动 codex logout）
  2. codex_local_auth_off()
     = 现有 codex_auth_off 的 File 分支：
       sync_runtime_with_saved_account_best_effort → 备份 → 删除 auth.json
  3. if find_managed_daemon().is_some():
       stop_codex_daemon()
       非 Stopped → 警告 + 手动命令 `codex app-server daemon stop`，返回 Err，不启动登录
  4. run_codex_login(): `codex login`，继承 stdio，阻塞等待
       成功 → next steps: ccr codex auth save <name>
       失败/找不到 codex → Err，提示本地已登出、可手动运行 codex login
```

第 2 步先于第 3 步：如果先停守护进程，其他 codex 客户端可能在删除文件前把守护进程重新拉起，新守护进程会再次读入旧 token。先删除文件后，任何重新拉起的守护进程都读不到旧 token。`codex login`（CLI 浏览器流程）直接写文件，不经守护进程。

## 边界与所有权

| 层                                                                        | 改动                                                                                                                                                                                                                                                                        |
| ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/ccr-cli/src/application/auth_off.rs`                              | 新增 `pub fn codex_local_auth_off() -> Result<AuthOffResult>`：store 非 File 时返回 `CcrError`；File 时复用现有 File 分支逻辑（抽出共用私有函数，`codex_auth_off` 的 File 分支调用同一函数，行为不变）                                                                      |
| `crates/ccr-codex/src/services/codex_process_service.rs`                  | `restart_codex_daemon_at` 改为参数化子命令的私有 `run_daemon_command_at(bin, subcommand, timeout)`；新增 `pub async fn stop_codex_daemon() -> DaemonStopOutcome` 与 `pub enum DaemonStopOutcome { Stopped, Failed { detail }, Timeout, Unavailable }`；restart 公共签名不变 |
| `crates/ccr-codex/src/lib.rs` / `services/mod.rs`                         | 导出 `stop_codex_daemon`、`DaemonStopOutcome`                                                                                                                                                                                                                               |
| `crates/ccr-cli/src/commands/codex/auth/relogin.rs`                       | 新文件，`relogin_command()`                                                                                                                                                                                                                                                 |
| `crates/ccr-cli/src/commands/codex/auth/mod.rs`                           | 注册模块与导出                                                                                                                                                                                                                                                              |
| `crates/ccr-cli/src/cli/subcommands/codex.rs`                             | `CodexAuthAction::Relogin`（无参数）                                                                                                                                                                                                                                        |
| `crates/ccr-cli/src/cli/dispatch.rs`                                      | 分派                                                                                                                                                                                                                                                                        |
| `crates/ccr-cli/src/cli/help_config.rs`                                   | 常用任务加入 relogin                                                                                                                                                                                                                                                        |
| `docs/reference/commands/codex.md`、`docs/en/reference/commands/codex.md` | 命令参考                                                                                                                                                                                                                                                                    |

## 不变量

- relogin 的任何路径都不调用 `spawn_official_logout`，也不向 revoke 端点发请求。
- `auth off` 的现有行为不变（keyring/auto 仍调用 `codex logout`）。
- 备份、原子写、密钥遮蔽沿用 `delete_credential_files` 与 `AuthOffBackup`，不新增写路径。
- 守护进程停止只通过官方 `codex app-server daemon stop`，不发送信号。

## 回滚

单一提交，`git revert` 即可。新增命令之外只改动 `restart_codex_daemon_at` 的内部参数化和 `codex_auth_off` 的 File 分支抽取，二者行为由现有测试覆盖。

## 测试

- `auth_off.rs`：keyring/auto 下 `codex_local_auth_off` 返回错误且不删除文件、不启动 logout（沿用现有 store 注入 / 临时目录 fixture）。
- `codex_process_service.rs`：`run_daemon_command_at` 使用 `stop` 参数（沿用现有 restart 测试 seam）。
- `relogin.rs`：纯函数判定「守护进程停止结果 → 是否继续登录」，仅 `Stopped` 与「无守护进程」继续。
