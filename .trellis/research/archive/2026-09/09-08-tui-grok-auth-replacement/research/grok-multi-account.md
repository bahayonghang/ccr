# Grok 多账号协议与 CCR 复用证据

核查日期：2026-09-08。官方源码固定 commit `72a61251fcffb464bcc687aeb5a998e5a98ec0c9`；仅读公开源码、本机 Grok docs 和 CCR 源码，未读取真实凭据/配置/日志，未运行 Grok。本文事实与规划提案分开。

## 官方协议事实

| 事实 | 一手依据 |
| --- | --- |
| auth.json 顶层是 scope -> GrokAuth，scope 不是账号名；OAuth scope 为 issuer 去尾斜杠后拼接 ::client_id | [model.rs:250](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/model.rs#L250)、[config.rs:187](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/config.rs#L187) |
| credential 中 key 是 access token；另含 auth_mode/create_time/user_id/email/refresh_token/expires_at/oidc_issuer/oidc_client_id/principal/team 等信息 | [model.rs:46](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/model.rs#L46) |
| auth_mode 是 web_login/oidc/external/api_key；旧 web_login 被当前读取逻辑忽略 | [model.rs:26](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/model.rs#L26)、[model.rs:292](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/model.rs#L292) |
| 默认官方 issuer https://auth.x.ai，client b1a00492-073a-47ea-816f-4c329264a828；个人/团队同 scope | [config.rs:122](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/config.rs#L122)、[config.rs:241](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/config.rs#L241) |
| API key 独立 scope xai::api_key，不应被账号恢复覆盖 | [storage.rs:357](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/storage.rs#L357) |
| user_id 初始可能 unknown 或来自 sub/principal，后续 enrichment 可改身份字段；email 不是稳定身份 | [protocol.rs:678](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/oidc/protocol.rs#L678)、[enrichment.rs:228](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs#L228) |
| refresh 需要 token/issuer/client；新 refresh_token 存在则替换，否则保留；磁盘失败时内存仍可能更新 | [refresh.rs:84](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/oidc/refresh.rs#L84)、[manager.rs:875](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/manager.rs#L875) |
| refresh 持 auth.json.lock；enrichment 同锁中重读并比较 key/refresh_token；API key 存储路径未遵守同锁 | [lock.rs:1](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/manager/lock.rs#L1)、[refresh_chain.rs:165](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/manager/refresh_chain.rs#L165)、[enrichment.rs:142](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/manager/enrichment.rs#L142)、[storage.rs:367](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/storage.rs#L367) |
| docs 宣称 hot reload，但 reloader 仅 key 变化发更新；另一磁盘采纳路径拒绝落后内存 create_time 超过60秒的 token | [认证文档](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-pager/docs/user-guide/02-authentication.md)、[reloader.rs:241](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/config/reloader.rs#L241)、[manager.rs:1075](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/manager.rs#L1075) |
| 官方 Windows secret 文件使用当前用户 DACL；不能照搬其某些 remove+rename/非原子 fallback 写法 | [secure_file.rs:92](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell-base/src/util/secure_file.rs#L92)、[storage.rs:171](https://github.com/xai-org/grok-build/blob/72a61251fcffb464bcc687aeb5a998e5a98ec0c9/crates/codegen/xai-grok-shell/src/auth/storage.rs#L171) |

## CCR 可复用与缺口

- crates/ccr-core/src/core/guarded_write.rs:102、125、134：write_guarded/content_version_token/write_guarded_versioned。每路径锁和最终内容核对不等于多文件事务，也不封锁不守锁的外部 writer。
- crates/ccr-core/src/core/lock.rs:264：操作级资源锁。FileLock::new 在同文件 138 先 truncate(true)，不能直接复用到官方锁文件，否则会清 holder 信息。计划改为非截断打开并测试；不新增依赖或通用锁选项。
- crates/ccr-core/src/core/atomic_writer.rs:236 已在 Windows 保留已有目标 DACL；新目标没有对应 owner-only 初始化。多账号新存储需要补新建 secret 文件保护；不能宣称当前实现已满足。
- crates/ccr-cli/src/services/claude_auth_service.rs:859、919、979：保存/切换/删除流程；685 是凭据 hash 匹配。只参考交互和流程，不泛化复制整个服务。
- crates/ccr-codex/src/services/codex_oauth_token_service.rs:333：Codex 回存依赖专有身份信息，不能用于 Grok。
- crates/ccr-codex/src/services/codex_auth_service.rs:153、169、858、898：Codex 将当前 auth.json 复制到 CCR 平台 auth 目录，并更新 CCR 元数据；save_current 不写回原生源文件。用户已要求 Grok 保存遵循这种只读复制语义、当前账号继续使用、存入 .ccr 对应目录。Grok 保存采用同一平台归属，持久化仍为 accounts.json 中的单 scope 完整副本。
- crates/ccr-cli/src/application/auth_off.rs:350：现有 Grok off 没有多账号操作锁。新 save/switch/delete/off 必须协调；off 仍保留原“删除整个官方运行时 auth.json”的动作语义，不动已保存账号。
- crates/ccr-cli/src/platforms/grok.rs:197 的 inspect_activation_state 只读；profile off 与 auth switch 分离，switch 不自动清第三方 route。
- crates/ccr-store 是 session/cache SQLite，本次不引入凭据数据库。
- crates/ccr-tui/src/tui/overlay.rs:153 的 Confirm 渲染硬编码“确认删除”，且并无默认取消选项选择状态。Grok 可复用几何/配色辅助，不能把保存覆盖/切换直接套成删除弹窗。
- crates/ccr-tui/src/tui/runtime.rs:63 有 spawn_blocking，Disabled 会丢任务；Grok 必须处理执行器不可用，防永久 Busy。
- crates/ccr-tui/src/tui/event.rs:48 已过滤 Repeat/Release，不需在每个服务/渲染层重复防抖。

## 规划推论与验收边界

采用官方 OAuth 个人/团队账号，保存一个 scope 的完整对象，不整文件恢复，不自行刷新 token。用户已确定停止当前 Grok 后切换，供后续新会话使用；旧账号快照在已运行进程中立即生效不能由静态源码保证，也不属于本次范围。

保存范围已由用户单独确定：Grok 运行时可只读保存到 CCR Grok 目录，不停止进程、不获取官方锁、不触碰原生文件，不调用登录/登出/refresh。读取后原生可继续刷新，CCR 保存捕获的完整快照；只有显式切换才讨论运行会话生效条件。

身份对比采用 scope + user_id + principal_type/principal_id + team_id（保守精确比较）；unknown/缺失/歧义时要求明确保存，不根据 current 指针自动覆盖。create_time/expires_at/未知字段原样保留，禁止改时间骗过热加载。

fs2 与 fs4 静态上均映射 OS 文件锁；实际跨库双进程竞争尚未执行。服务端 token 轮换/撤销规则、磁盘写失败后内存更新、真实新会话登录有效性、已安装 Grok 构建来源均不在本轮证据内。旧文档“任何快照必然过期”不是已证明的服务端规律，不再作为拒绝多账号的理由。
