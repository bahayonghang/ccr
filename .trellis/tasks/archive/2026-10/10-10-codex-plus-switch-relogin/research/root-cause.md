# 根因分析：Plus 账号切换后需重新登录

日期：2026-10-10。时间均为 UTC（本机本地时间 = UTC−5）。证据只记录 claim、时间戳和 refresh token 的 sha256 前 10 位指纹，不记录 token 原文。

## 结论

Plus 账号失效的直接原因是 codex 的 logout 流程在服务端吊销了 refresh token。ccr 快照保存的是同一个 refresh token，所以快照随之失效。之后 ccr 把快照写回 `~/.codex/auth.json`，codex 和 ccr 配额查询都拿到已吊销的 token，表现为「一切换就报错」。

切换动作本身不吊销 token。切换只把更早发生的吊销暴露出来。

账号套餐类型不是区分因素。截图中的 `teacher` 账号 id_token 的 `chatgpt_plan_type` 是 `k12`（不是 team）。该账号的快照 token 自 10-07 04:05 保存后，没有在任何一次 codex logout / login 时处于 `auth.json` 中，所以没有被吊销。

## 上游行为

openai/codex PR #17825（commit 22f7ef1，"Revoke ChatGPT tokens on logout"）：

- `codex logout`、TUI `/logout`、app-server `account/logout` 都调用 `logout_with_revoke`，向 `https://auth.openai.com/oauth/revoke` 发送存储的 refresh token（`token_type_hint: refresh_token`），再删除本地 `auth.json`。
- `codex login`（浏览器流程和 device code 流程）在登录前调用 `clear_existing_auth_before_login`，内部同样是 `logout_with_revoke`。

因此在 `auth.json` 里有已保存账号时运行 `codex login` 或 `/logout`，该账号的 ccr 快照必然失效。

来源：
- https://github.com/openai/codex/pull/17825
- https://github.com/openai/codex/blob/5af85998/codex-rs/cli/src/login.rs
- https://github.com/openai/codex/blob/f802f0a3/codex-rs/login/src/auth/revoke.rs

## 本机证据

### 快照与 runtime 状态（10-10 检查时）

| 文件 | plan | auth_time | last_refresh | rt 指纹 | 文件 mtime |
|---|---|---|---|---|---|
| `~/.codex/auth.json` | plus（khanh） | 06:38 | 06:38:24 | c4273cfb3a | 07:19:24 |
| `auth/khanh.json` | plus | 06:38 | 06:38:24 | c4273cfb3a | 06:39:37 |
| `auth/163.json` | plus | 07:11 | 07:11:42 | 0af2ec2dde | 07:12:02 |
| `auth/teacher.json` | k12 | 10-07 04:05 | 10-07 04:05:16 | acf1af840e | 10-07 04:05:40 |

khanh 的 refresh token 从 06:38 登录后没有轮换过。runtime 与快照一致，所以失效原因不是「快照未回写导致 refresh_token_reused」。

### codex 日志时间线（`~/.codex/logs_2.sqlite`）

| 时间 | 进程 | 事件 |
|---|---|---|
| 06:38 | — | khanh 登录（codex 日志中没有 `codex_login::server` 记录，推断经 ccr-ui 内置 OAuth 登录） |
| 06:39:37 | ccr | 保存 khanh 快照（rt c4273cfb3a） |
| 07:11:11–07:11:14 | daemon 66184，client codex-tui 0.162.1 | `account/logout`，`Reloaded auth, changed: true`。此时 `auth.json` = khanh，khanh 的 rt 被吊销 |
| 07:11:39–07:11:42 | daemon 66184 | 浏览器 OAuth 登录 163，`oauth token exchange succeeded` |
| 07:12:02 | ccr | 保存 163 快照（rt 0af2ec2dde） |
| 07:13:59–07:14:01 | daemon 66184，client codex-tui 0.162.1 | 第二次 `account/logout`，`changed: true`。此时 `auth.json` = 163，163 的 rt 被吊销 |
| 07:19:24 | ccr | 切换到 khanh，写入 `auth.json`（ccr.log：`Codex auth.json 已原子保存`） |
| 07:20:28 | codex 0.162.1 进程 62532 | `/models` 返回 401 `token_revoked`（"Encountered invalidated oauth token for user"） |
| 07:20:32 | 进程 62532 | `Failed to refresh token status=401 ... error_code: Some("refresh_token_invalidated"), error_message: Some("Your session has ended. Please log in again.")` |

163 的 ERR 与 khanh 同因：163 的 rt 在 07:13:59 的 logout 中被吊销。

## 未查明事项

- 02:09 起，三个 codex 0.146.0 进程（pid 42520、40208、62620）和一个 0.161.0 进程（pid 57412）持续收到 `token_revoked`。02:09 之前的日志中没有 login / logout 记录，该次吊销的触发原因未查明。0.146.0 进程版本较旧，来源未查明（可能是其他宿主应用内置的 codex），它们在 07:24 仍在轮询。
- 未验证 OpenAI 对不同套餐（plus / k12 / team）的 refresh token 策略是否有差异。现有证据不需要套餐差异即可解释全部 Plus 失效。

## ccr 现有能力

- Codex Auth TUI 的 `o` 键调用 `auth_off_for_platform(Platform::Codex)`（`crates/ccr-cli/src/application/auth_off.rs:227`）。该路径备份并删除本地 `auth.json`，不调用服务端吊销。之后 `codex login` 找不到旧 token，不会吊销已保存账号。
- 配额错误文案已经提示 "press o before codex login"（`crates/ccr-tui/src/tui/codex_auth/ui.rs:974`）。该提示只在失效发生后显示，不能阻止失效。
- ccr-ui 有内置 OAuth 登录（`ccr-ui/src-tauri/src/commands/codex_auth.rs:33`），不经过 codex 的 logout 流程。

## 用户侧规避

1. 添加新账号前，在 ccr Codex Auth 按 `o` 本地登出，再运行 `codex login`；或使用 ccr-ui 内置登录。
2. 不要在 codex TUI 中执行 `/logout` 来换号。
3. 已失效的 khanh、163 需要重新登录一次后重新保存快照。
