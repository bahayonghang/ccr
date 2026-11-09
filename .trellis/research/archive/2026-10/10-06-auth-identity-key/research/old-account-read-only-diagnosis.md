# 旧账号只读诊断

2026-10-06 用户报告：k12 已通过 `ccr codex auth save k12` 保存，切换后仍无法查询。截图显示当前 runtime 为 khanh，k12 配额错误要求重新登录。截图属于初始证据，不据该文案认定具体吊销原因。

## 当前事实

- k12 注册表记录和自有账号快照均存在。快照包含一致的完整用户身份；注册表 account_id 与快照相同，快照邮箱脱敏后与注册表邮箱相同。未发现该记录被当前账号覆盖的证据。
- k12 与当前 khanh runtime 的用户和 workspace 均不同。配置明确使用 file credential store。
- k12 的 last_refresh 为 2026-10-06T08:19:04Z，access token 的本地 exp 为 2026-10-16T08:19:06Z。诊断时尚未到期。JWT 解析不验证服务器授权。
- 支持的 runtime/托管备份位置仅找到一个匹配 k12 完整身份的备份，其 last_refresh 为 2026-10-01T12:14:47Z，比现有快照旧。本次未尝试提交旧备份的 refresh token。
- 对已实现的 `https://chatgpt.com/backend-api/wham/usage` 进行 GET-only 请求：k12 返回 HTTP 401；以相同方法请求当前 khanh runtime 返回 HTTP 200，配额结构存在。没有请求 OAuth refresh 端点，没有执行登录或切换。k12/registry/runtime 原始字节哈希在首个诊断请求前后相同。
- k12 的错误未识别到已有允许的固定错误码，也没有匹配 `authentication token has been invalidated` 固定短语。远端拒绝原因未查明，不把 401 描述为已确认 refresh-family 吊销。

凭据、完整身份值、原始响应体和个人邮箱未输出到日志或证据。本文件只记录脱敏状态。Python HTTPS 诊断采用固定端点、禁止重定向、有限响应读取；结果只输出状态、固定分类和必要布尔值。

## 结论与验证边界

保存成功意味着本地账号快照存在；保存不保证远端以后仍接受其中的凭据。该账号的信息仍在，当前认证请求未被远端接受。

共享 quota core 的手动查询缺陷已用合成 stub 重现并修复：有效 access token 不因手动查询先触发 OAuth 刷新。该修复不能将 k12 的实际 401 自动变成成功。

用户要求参考 cockpit-tools 深入完善保存、切换、轮换与写回；源码对照见 `cockpit-account-lifecycle.md`，新增凭据操作互斥和保存同源读取通过合成回归验收后再更新状态。不能将新的本地检查等同真实账号恢复。

用户已选择稍后完成 k12 本人登录，先完成代码修复。真实账号重新登录、OAuth refresh、切换与修复后额度验证均为 NOT_RUN；当前 k12 查询恢复为 UNVERIFIED。
