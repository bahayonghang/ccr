# 切换不可用与账号失效复现分析（D5）

日期：2026-10-06。基线：dev @ 797c1877。方法：本机只读检查（只输出键名、时间戳、token 的 sha256 前 8 位，不输出 token 值）+ 代码走查 + 上游 openai/codex 源码与 issue 核对。

## 1. 本机证据（只读）

环境：codex-cli 0.160.1；`CODEX_HOME`、`CCR_CODEX_DIR` 未设置；`~/.codex/config.toml` 含 `cli_auth_credentials_store = "file"`、`forced_login_method = "chatgpt"`。

| 文件                                        | account_id 哈希 | refresh_token 哈希 | last_refresh            | 键                                              |
| ------------------------------------------- | --------------- | ------------------ | ----------------------- | ----------------------------------------------- |
| runtime auth.json                           | 01360a80        | c6f2dc7f           | 2026-10-06T08:19:04.84Z | OPENAI_API_KEY, auth_mode, last_refresh, tokens |
| 快照 k12.json                               | 01360a80        | c6f2dc7f           | 2026-10-06T08:19:04Z    | 同上                                            |
| 快照 khanh.json                             | 978ea5eb        | bc30d07f           | 2026-10-06T02:23:19Z    | 同上                                            |
| backups/auth.runtime_switch.20261006_031731 | 978ea5eb        | bc30d07f           | 2026-10-06T02:23:19.08Z | 同上                                            |
| backups/auth.runtime_switch.20261006_031816 | 01360a80        | d221f0d9           | 2026-10-01T12:14:47Z    | last_refresh, tokens（CCR normalize 写入形态）  |
| 其余 8 个 auth.runtime_switch 备份          | —               | —                  | —                       | 仅 OPENAI_API_KEY（无 OAuth tokens）            |

结论：

- 03:17:31 CCR 把 khanh 换出时，runtime 中 khanh 的 refresh_token 与 khanh 快照相同（bc30d07f）。khanh 失效时不存在「轮换后未回写」的丢失。
- 03:18:16 的备份是 CCR 写入的旧 k12 状态（d221f0d9，10-01）。当前 runtime 与 k12 快照持有新 token（c6f2dc7f，含 codex 写入的 `auth_mode` 键），说明之后执行过 `codex login`（k12）并由 CCR 保存/同步。
- 时间线与用户报告一致：khanh 处于 runtime 时执行 `codex login` 登录 k12，随后 khanh 报 `refresh_token_invalidated`。

## 2. 上游行为（openai/codex）

- PR #17825「Revoke ChatGPT tokens on logout」：`codex logout`、TUI `/logout`、app-server logout 先向 `https://auth.openai.com/oauth/revoke` 吊销 refresh_token，再删除本地凭据。
- PR #21747「revoke superseded auth tokens on relogin」；issue #22577 评论：已有托管 ChatGPT 凭据时，普通 `codex login` 会在新登录前吊销旧凭据；临时规避为 `CODEX_REVOKE_TOKEN_URL_OVERRIDE=http://127.0.0.1:65535/oauth/revoke codex login`。
- codex manager.rs 把 `refresh_token_invalidated` 归类为 Revoked（「refresh token was revoked」），`refresh_token_reused` 归类为 Exhausted（已被使用）。
- 本机 codex.exe 0.160.1 二进制含字符串 `oauth/revoke`、`CODEX_REVOKE_TOKEN_URL_OVERRIDE`、`failed to revoke auth tokens during logout`、`failed to clear existing auth before login`。
- issue #40541：账号切换后，仍在运行的 codex 进程（CLI/app-server/Desktop）因 account id 不匹配跳过 reload，不会采用新账号，只能重启恢复。

## 3. 根因结论

**根因 A（成立，用户主场景）**：外部 `codex login`（及 `codex logout`）吊销 runtime 中当前账号的 refresh_token。被吊销的 token 在服务端失效，所有本地副本（快照、runtime、备份）同时不可用。CCR 的回写、repair 无法恢复，只能重新登录该账号。规划假设的「轮换失同步」不是本次失效的原因。

**根因 B（成立，代码缺陷，独立于 A）**：规划 research §3 的两条失同步链路仍是真实缺陷：

- 换出未回写：switch_account 不回写换出账号 runtime tokens（codex_auth_service.rs:1106）。
- CCR 自身轮换分裂：活动已保存账号的配额刷新消费快照 refresh_token（codex_quota_service.rs:172、:255-258），runtime 仍持被消费值；codex 下次刷新得到 `refresh_token_reused`。

## 4. 候选逐项结论

| #   | 候选                | 方法                                                                                                    | 结论                                                                                                                                    |
| --- | ------------------- | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | codex 进程运行中    | 上游 issue #40541 + TUI 代码（app.rs:1283 仅提示）                                                      | 成立（上游行为）：运行中的 codex/Desktop 不采用新账号，需重启。CCR 已提示；不新增阻断（R6 约束）。                                      |
| 2   | 非 file 凭据存储    | 本机 config 为 file；代码走查 commit_plan 拒绝 Write 的文案                                             | 本机不成立；报错文案已给出可执行指引（codex_runtime_service.rs:236-243）。                                                              |
| 3   | normalize 字段裁剪  | 本机 runtime 键对照；上游 AuthDotJson.auth_mode 为 Option 并按字段推断                                  | 不成立（静态推断）：normalize 去掉 `auth_mode` 与空 `OPENAI_API_KEY`，codex 按 tokens 推断 ChatGPT 模式。未做原生启动实验，标 NOT_RUN。 |
| 4   | config/profile 交互 | 代码走查：TUI 切换先调 profile_off_for_platform（app.rs:1292），仅在存在 profile 指针或第三方路由时清理 | 本机不成立（`Profile: Not bound`，无第三方路由时 profile_off 不改文件）。                                                               |
| 5   | 快照 token 已失效   | §1 本机证据 + §2                                                                                        | 成立：表现为切换后目标账号不可用；原因是根因 A（吊销），不是轮换丢失。                                                                  |
| 6   | 切换后退出 TUI      | 代码走查 app.rs:1310                                                                                    | 行为如设计；改变退出行为属行为变更，本任务不改。                                                                                        |
| 7   | CCR 自身轮换分裂    | 代码走查                                                                                                | 成立（根因 B 第二链路），由 D1 配额路由 + 新鲜度定向修复。                                                                              |

## 5. 对规划的影响

- D1–D4（换出回写、观测点同步、配额路由、repair 谓词、身份规则）修复根因 B，仍然有效。
- 根因 A 不能由同步或 repair 修复。可行缓解只能防止吊销发生：在外部 `codex login` 前让 runtime 不含旧账号凭据（先同步快照，再本地删除 runtime auth.json，不调用 revoke），或使用隔离 CODEX_HOME 登录。该缓解超出原规划范围，需用户决定。
- AC6(b)「检测并呈现需重新登录」覆盖根因 A 的结果呈现。

## 6. D6 窗口边界

修复后仍存在以下理论丢失窗口，代码不能关闭这些窗口：

| 窗口 | 条件 | 结果 | 缓解 |
| --- | --- | --- | --- |
| W1 外部覆盖 | codex 在 CCR 未运行期间轮换账号 A 的 refresh_token，之后外部 `codex login B` 覆盖 runtime，期间无 CCR 观测点 | A 快照仍持有已被消费的旧 token；codex login 还会吊销 runtime 中的新 token（根因 A） | 观测点最大化：TUI 加载/reload、switch 换出、配额查询前路由、`ccr codex auth sync`、本地登出（TUI `o` / `ccr codex auth off`）均先同步。外部登录前先按 `o` 本地登出，可避免吊销。结果呈现为「需重新登录」，快照与账号保留。 |
| W2 备份池上限 | `~/.codex/backups/` 中 auth 前缀备份为共享池，保留上限 10 个（codex_config.rs MAX_BACKUPS），runtime_switch 与其他 auth 备份标签共用 | 超过 10 个后最旧的备份被删除，repair 可用来源减少 | repair 按新鲜度选源，不依赖单一标签；快照同步是主路径，备份只是兜底。超限行为按共享池语义，不另加测试。 |
| W3 keyring/auto 存储 | `cli_auth_credentials_store` 不是 file | CCR 不读写 runtime 文件，同步为 NoOp；`ccr codex auth off` 调用 `codex logout`，该命令会吊销 token | 文档说明；本任务不改变 keyring 行为。 |
| W4 运行中 codex 进程 | 切换后已运行的 codex/Desktop 进程 | 进程不重新加载账号（上游 issue #40541） | 维持现有提示；重启进程。 |

codex 自身是否在 login 前保存可恢复备份：未核实（原因未查明），本任务不依赖该行为。
