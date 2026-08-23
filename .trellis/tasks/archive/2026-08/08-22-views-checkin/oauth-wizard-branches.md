# OAuth 向导步骤与错误分支

来源：`OAuthWizardModal.vue` 抽取。迁移后状态机：`src/features/checkin/lib/oauthWizardReducer.ts`。
向导在凭据步骤收集 cookies / api_user，不走真实 OAuth token 交换（`getOAuthAuthorizeUrl` 在 Tauri 为失败桩）。

## 步骤

| step | 名称 | 进入条件 | 成功出口 |
| --- | --- | --- | --- |
| 0 | 选择提供商与登录方式 | 弹层打开 / RESET | `FETCH_URL_START`（已选 provider 与 oauth_type） |
| 1 | 获取授权链接 | `FETCH_URL_START` | 有 `authorize_url` 后 `GOTO_CREDENTIALS` |
| 2 | 粘贴凭据 | `GOTO_CREDENTIALS` | cookies 解析非空后 `GOTO_CONFIRM` |
| 3 | 确认创建 | `GOTO_CONFIRM` | `CREATE_SUCCESS`；本步用已粘贴 cookies 调 `createCheckinAccount`，不使用真实 OAuth 凭据 |

## 错误分支（reducer action）

| 分支 | action | 触发 | 呈现 |
| --- | --- | --- | --- |
| OAuth 未配置 | （停留 step 0） | 无 linuxdo/github client_id | `checkin.oauthWizard.oauthNotConfigured` |
| 拉取授权链接失败 | `FETCH_URL_ERROR` | `success === false` 或空 url | 错误面板 + 返回选择 |
| 网络失败 | `FETCH_URL_ERROR` | `getOAuthAuthorizeUrl` throw | 错误面板 |
| Tauri 桩失败 | `FETCH_URL_ERROR` | HTTP-only 桩返回失败消息 | 错误面板 |
| 凭据为空 | `PARSE_ERROR` | 解析后 cookie 数为 0 | `errors.emptyCookies` |
| 无法识别格式 | `PARSE_ERROR` | 非 JSON 且不含 `=` | `errors.unrecognizedCredentialsFormat` |
| 解析失败 | `PARSE_ERROR` | 其它 parse 异常 | `errors.parseFailed` |
| 缺少提供商 | `CREATE_ERROR` | confirm 时 provider 丢失 | `errors.providerRequired` |
| 创建失败 | `CREATE_ERROR` | `createCheckinAccount` throw | `errors.createFailed` |
| 复制完成超时 | `CLEAR_COPIED` | 复制成功 2s 后 | 按钮文案恢复 |
| 重复成功 | `FETCH_URL_SUCCESS` 在 step !== 1 时忽略 | StrictMode / 迟到响应 | 状态不变 |

## 外部事件等待

本向导无 OAuth 回调 `listen()`。WAF WebView bypass 不在向导内，而在 Provider 页 `openWafLogin` 与签到补救 `waitForCheckinJobResult`（`checkin:job-finished` / `checkin:job-timeout`）。
