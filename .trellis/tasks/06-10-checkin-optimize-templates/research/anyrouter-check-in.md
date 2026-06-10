# Research: anyrouter-check-in 签到可靠性分析

- **Query**: ref/repo/anyrouter-check-in 如何把 AnyRouter / NewAPI 类站点签到做得可靠（流程、判定、WAF、配置模型、错误处理）
- **Scope**: internal（只读参考仓库 `ref/repo/anyrouter-check-in`）
- **Date**: 2026-06-10
- **说明**: 本文件由主会话直接调研撰写（research 子代理两次因代理端 400 错误失败）

## 项目概览

Python + httpx(HTTP/2) + cloakbrowser（反检测 Playwright 衍生）。面向 AnyRouter/AgentRouter 双内置 provider 的多账号定时签到，GitHub Actions 驱动。核心文件：`checkin.py`（主流程 655 行）、`utils/config.py`（Provider/Account 配置模型）、`utils/browser.py`（浏览器登录 + WAF 等待）、`utils/notify.py`（多渠道通知）。

## 签到流程与成功失败判定

### 主流程（checkin.py:353-473）

```
账号循环（串行）→ 认证（邮箱密码浏览器登录 优先 | session cookies + WAF cookies）
→ httpx HTTP/2 客户端 + 完整浏览器头
→ GET user_info（签到前余额）
→ 有 sign_in_path: POST 签到；无 sign_in_path（AgentRouter）: 再 GET user_info 即视为自动签到
→ GET user_info（签到后余额）→ 计算奖励
```

### 成功判定（execute_check_in, checkin.py:280-315）

1. `ret == 1 || code == 0 || success` → 成功（兼容多种 NewAPI 分叉响应风格）。
2. 失败 message 含已签到关键词 `['已经签到','已签到','重复签到','already checked','already signed']` → **归一为成功**。
3. JSON 解析失败时退化为 `'success' in response.text.lower()` 文本嗅探。
4. 非 200 → 失败 `HTTP {status}`。

### 请求构造（run_check_in_requests, checkin.py:409-473）

- **httpx `http2=True`**——更像真实浏览器，降低 WAF 嫌疑。
- 完整浏览器头：UA(Chrome 138)/Accept/Accept-Language/Accept-Encoding/Referer/Origin/Connection/`Sec-Fetch-Dest|Mode|Site`；签到 POST 额外加 `Content-Type: application/json` + `X-Requested-With: XMLHttpRequest`。
- `new-api-user` 头：取账号配置 `api_user`，或邮箱登录时从浏览器拦截 `/api/user/self` 响应自动解析出的用户 id（api_user_override）。

### 奖励计算（main, checkin.py:531-559）——比"解析签到响应"更可靠

- 签到前后各查一次余额：`reward = (after_quota+after_used) - (before_quota+before_used)`（总额差，排除期间消耗干扰）；`usage_increase`、`balance_change` 分开展示。
- 通知文案能区分「签到获得 +$X」vs「今日已签到（期间有使用）」vs「无变化」。

## WAF-CF 绕过策略

- **cloakbrowser**（stealth 浏览器）headless 访问 login 页 → `wait_for_waf_ready`（站点就绪轮询）→ 从 **`page.context.cookies()`（完整 cookie store，非 document.cookie）** 中只挑 `waf_cookie_names` 声明的必需 cookie（anyrouter: `acw_tc/cdn_sec_tc/acw_sc__v2`；agentrouter: `acw_tc`）。
- **缺任何一个必需 cookie 即整体失败返回 None，绝不部分使用**（checkin.py:125-134）——与 ccr WAF 任务的 `is_complete()` 契约一致。
- WAF cookies 与账号 cookies 合并：`{**waf_cookies, **user_cookies}`（用户 cookie 优先）。
- 邮箱密码登录路径天然不需要单独 WAF 步骤（同一浏览器会话直接过盾 + 登录 + 拦截 api_user），且 `persist_profile=True` 时复用浏览器 profile 目录，下次免登录。

## 配置与数据模型

### ProviderConfig（utils/config.py:12-72）——与 ccr BuiltinProvider 最可比

| 字段                            | 默认                | 语义                                                                               |
| ------------------------------- | ------------------- | ---------------------------------------------------------------------------------- |
| `domain`                        | 必填                | 站点 base URL（唯一必填项）                                                        |
| `login_path`                    | `/login`            | 浏览器登录/WAF 获取入口                                                            |
| `sign_in_path`                  | `/api/user/sign_in` | **`None` = 查询用户信息即自动签到**（AgentRouter 模式，对应 ccr `checkin_bugged`） |
| `user_info_path`                | `/api/user/self`    | 余额/用户信息                                                                      |
| `api_user_key`                  | `new-api-user`      | 用户 id 头名                                                                       |
| `bypass_method`                 | `None`              | 仅 `'waf_cookies'` 一种；`waf_cookie_names` 为空时自动归零                         |
| `waf_cookie_names`              | `None`              | 必需 WAF cookie 名单                                                               |
| `use_proxy` / `persist_profile` | false               | 站点级代理 / 浏览器 profile 持久化                                                 |

- 内置仅 2 个 provider（anyrouter/agentrouter），用户经 `PROVIDERS` 环境变量 JSON **增量覆盖**：`ProviderConfig.from_dict(name, data, defaults=providers.get(name))` 逐字段回退到同名内置默认——内置目录是「默认值」不是「封闭清单」。
- AccountConfig：`provider`（引用 provider 名）+ `cookies | email+password` 二选一 + `api_user` + `name`。校验时给出具体错误（哪个账号缺哪个字段、JSON 解析失败的常见原因提示）。

## 错误处理与重试

- **无自动重试**：单账号单次尝试，失败记入统计；异常 message 截断 50 字符防泄漏。
- 邮箱登录失败**不回退**到可能过期的 session cookie（显式打印原因后放弃，checkin.py:384-385）。
- 登录失败时保存截图（debug 模式）上传 Actions artifact，给出 run URL——故障可追溯。
- **通知降噪**：余额数据 sha256 hash 持久化到文件，仅「有失败 或 余额变化」才推送通知（checkin.py:577-636）；全部成功且余额没变 → 静默。
- 退出码：**至少一个账号成功即 exit 0**（部分失败不算任务失败）。

## 对 ccr 的启示

1. **请求指纹完整度是签到可靠性的第一杠杆**：HTTP/2 + Sec-Fetch-\* + Referer/Origin + X-Requested-With + 真实 UA。ccr 的 reqwest 默认请求头远比这少——「签到报错（被 WAF 拦）」的一部分可能直接来自请求特征太像 bot。reqwest 开 http2 + 补齐浏览器头是低成本高收益修复。
2. **`sign_in_path: Option<String>` 单字段表达「自动签到站」**：None = 查 user_info 即签到。ccr 已有等价物（checkin_path: None + checkin_bugged），可对齐语义并在 UI 明示「该站查询即签到」。
3. **必需 cookie 清单 + 完整 cookie store 提取 + 不齐全即失败**——与 ccr 刚完成的 WAF 任务契约互为佐证（来源一致：本 repo 就是该任务的调研对象之一）。
4. **奖励用余额差推断**而不是解析签到响应 message：`(after_quota+after_used)-(before_quota+before_used)`。ccr 的 record 里 reward 解析失败时可用余额快照差兜底（metapi 也是同样结论）。
5. **内置 provider 目录 = 可被用户增量覆盖的默认值**：from_dict + defaults 模式。ccr 的 builtin_providers 目前是封闭硬编码，用户无法改 `builtin-anyrouter` 的 waf_cookie_names 这类字段——目录统一时应支持「内置 + 用户 override」合并。
6. **已签到关键词归一**（已经签到/已签到/重复签到/already checked/already signed）→ 成功。ccr 已有 `[ALREADY_CHECKED_IN]` 机制，但关键词表可对齐扩充。
7. **通知/提示降噪**：只在失败或余额变化时打扰用户。对 ccr 的桌面通知/结果面板同样适用（全部成功可轻量化展示）。
8. **邮箱密码登录 + api_user 自动拦截**是 ccr 没有的认证模式（ccr 是 cookies/token + OAuth 向导）。它解决「session 过期要手动抓 cookie」的根痛点，但引入凭据存储责任（ccr 有加密层可承接），可作为远期可选项而非 MVP。

## Caveats / Not Found

- 本 repo **无 cf_bypass.py / Cloudflare 处理**（CF 逻辑在 Newapi-checkin repo）；WAF 等待 `wait_for_waf_ready` 实现为通用「站点就绪」轮询（utils/browser.py:479-480），未读完整实现细节。
- `dingtalk_notifier.py` 不存在于本 repo（通知走 utils/notify.py 多渠道，未逐行分析）。
- `test_checkin.py` 在 tests/ 目录（pytest），未逐个用例分析；测试覆盖配置解析与判定逻辑。
- cloakbrowser 是第三方反检测浏览器封装，桌面应用打包它的可行性/体积未评估（ccr 用 Tauri WebView 路线更轻）。
