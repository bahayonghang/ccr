# 旧账号手动配额查询实施证据

状态：`LOCAL_CORE_PASS_PENDING_INDEPENDENT_GATE`。日期：2026-10-06。工作区：`dev`，既有 P1/P2 变更保留。产品写入范围仅为 `crates/ccr-codex/src/services/openai_quota_core.rs`。

## 结果与原因

手动配额查询的 `force_refresh` 原先同时绕过缓存并强制 OAuth 刷新。有效 access token 因失效 refresh token 提前失败，配额 GET 未执行。修复将提前刷新条件限定为 `is_token_expired`，保留本地过期与额度端点认证拒绝后的刷新路径。公开签名、调用参数、缓存期限和错误分类保持。

主会话另行完成真实账号的只读诊断，并报告 `k12` 的额度 GET 返回 HTTP 401。该诊断未由本子代理运行。共享核心修复不能证明该真实账号恢复；被端点拒绝的 access token 仍进入已有刷新流程，永久刷新失败仍返回错误。用户选择稍后重新登录。

## 改动

- `openai_quota_core.rs:180`：明确 `force_refresh` 仅绕过配额缓存。
- `openai_quota_core.rs:226`：有效 access token 不再因手动查询标志提前刷新。
- `openai_quota_core.rs:761`：新增仅测试的 loopback stub。所有 JWT 携带完整合成用户及账号 claims，每个夹具使用 UUID 隔离全局缓存；HTTP 端点通过 `TEST_ENDPOINTS.scope` 注入。记录配额 GET、token POST 与持久化回调次数。
- `openai_quota_core.rs:1175`：既有缓存到期测试写入实际已过期的自有条目，并用真实 `now` 读取。原测试将全局缓存读取时间推进 31 秒，会清除其他并行测试的有效条目。保留默认并行、30 秒 TTL 和原到期断言。

## 回归覆盖

| 场景 | 断言 |
| --- | --- |
| 有效 access + 永久失效 refresh + force | GET 1 次、刷新 0 次、持久化 0 次，取得配额 |
| 已有缓存 + force | 普通查询命中缓存；force 再发 GET，取得变化后的配额，刷新和持久化均为 0 |
| 本地过期 access | 刷新 1 次、持久化 1 次、用新 access 执行 GET 1 次 |
| 有效 access 被 usage 401/token_invalidated 拒绝 | 先 GET，刷新 1 次、持久化 1 次，再用新 access GET |
| 被拒绝的 access + 永久刷新失败 | 返回可识别的永久错误；无成功配额、无缓存条目、持久化 0 次 |
| 本地过期 access + 永久刷新失败 | 返回永久错误；GET 0 次、持久化 0 次、无成功缓存 |

## 命令和证据

运行采用 Rust 测试默认并行与 `--skip export_bindings`。没有读取真实凭据、访问真实服务或更改运行时账号。

1. 修复前：`cargo test -p ccr-codex --all-features services::openai_quota_core::tests::manual_quota -- --skip export_bindings`。退出 101，1 passed、4 failed。证明有效 access 被强制刷新阻断，且认证拒绝场景未先执行 GET。原始日志：`old-account-quota-first-failure.log`，SHA256 `58F747B3A360E51720B296BB9D5A30DB5E9EB9C9B72A5309F4B370722246C539`。
2. 条件修复后首轮核心门：`cargo test -p ccr-codex --all-features services::openai_quota_core::tests -- --skip export_bindings`。退出 101，15 passed、1 failed；新增测试全部通过，失败来自上述既有缓存到期夹具的并行干扰。原始日志：`old-account-quota-validation.log`，SHA256 `A31775849EA65F171B03F8BE4FE465B48ECBCADFDF59C8D049D60DAE4E7ECAC4`。
3. 夹具修复后同一核心门：退出 0，16 passed、0 failed、0 ignored。日志：`old-account-quota-retest.log`，SHA256 `2D5E89C06A8AD3C5FC264BAC9C67DA869711E7210FFF409E68C0502EEE5C7970`。
4. `rustfmt --edition 2024 crates/ccr-codex/src/services/openai_quota_core.rs`：退出 0。
5. `git diff --check -- crates/ccr-codex/src/services/openai_quota_core.rs`：退出 0。

完整 Codex crate、clippy 和全工作区门由 `check_p1` 与主会话协调运行，以其最终冻结源码证据为准。本子代理未提交、归档或替换已安装二进制。
