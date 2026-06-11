# Research: Newapi-checkin 通用签到逻辑分析

- **Query**: ref/repo/Newapi-checkin 如何用一套通用逻辑给多个 NewAPI 系站点签到（站点配置模型、成功判定、错误处理、CF 绕过、新站点接入成本）
- **Scope**: internal（只读参考仓库 `ref/repo/Newapi-checkin`，对照 `crates/ccr-checkin/src/managers/checkin/builtin_providers.rs`）
- **Date**: 2026-06-10
- **上游项目**: Jasonliu-0/Newapi-checkin（Python 3.11 + requests + Playwright，GitHub Actions 定时执行）

## 项目概览

仓库极小（核心代码约 1100 行），文件分工：

| 文件 | 行数 | 说明 |
|---|---|---|
| `checkin.py` | 616 | 主流程：账号解析 → 逐账号签到 → 汇总/通知 |
| `cf_bypass.py` | 243 | CF 拦截检测 + Playwright 同会话绕过签到 |
| `config_helper.py` | 218 | 交互式命令行配置生成器（含账号有效性测试） |
| `config_generator.html` | - | 网页版配置生成器（GitHub Pages 托管，localStorage 保存） |
| `test_checkin.py` | 138 | 单站点三步自检脚本（用户信息 / 签到 / 历史） |
| `debug_session.py` | 96 | Session Cookie Base64 解码分析工具 |
| `dingtalk_notifier.py` | 332 | 钉钉通知（与签到逻辑无耦合） |
| `.github/workflows/checkin.yml` | 37 | 每天 UTC 0:10 定时跑，预装 `playwright install chromium --with-deps` |
| `requirements.txt` | 5 | 仅 `requests` / `pytz` / `playwright`（**没有 pyproject.toml**） |

**核心设计哲学**：不维护任何"站点列表"。它赌的是所有 NewAPI 系站点共享同一套 API 契约（`/api/user/self` + `/api/user/checkin` + `session` cookie + `new-api-user` 头），因此**站点差异不在配置时声明，而在运行时检测**（CF 拦截是签到时发现的，不是配置里标记的）。

## 站点配置模型

### 每个账号只有 5 个字段（其中 2 个必填）

`checkin.py:340-387 parse_accounts()` 支持三种等价格式：

1. 单账号简单格式：`BASE_URL#SESSION_COOKIE`
2. 多账号逗号分隔：`URL1#SESS1,URL2#SESS2`
3. JSON 数组（推荐）：

```json
[{"url": "https://api.example.com", "session": "MTc2...", "user_id": "123", "name": "主力站", "cf_clearance": "可选"}]
```

| 字段 | 必填 | 用途 |
|---|---|---|
| `url` | 是 | base_url，唯一的"站点身份" |
| `session` | 是 | `session` cookie 值（登录态） |
| `user_id` | 推荐 | 填进 `new-api-user` 请求头；`config_helper.py:85-91` 把它设为必填并警告"缺少会导致签到失败" |
| `name` | 否 | 显示备注 |
| `cf_clearance` | 否 | 手动提供的 CF cookie，作为 Playwright 自动绕过的备用（`checkin.py:68-69` 直接 set 进 session） |

**没有** checkin_path / balance_path / auth_header / WAF 标记 / 分类等任何 per-site 字段。所有路径硬编码为 NewAPI 上游约定：

- `GET {base_url}/api/user/self` — 用户信息（`checkin.py:131`）
- `POST {base_url}/api/user/checkin` — 签到（`checkin.py:215`）
- `GET {base_url}/api/user/checkin?month=YYYY-MM` — 签到历史统计（`checkin.py:325-329`）

### 认证方式（统一一种）

`checkin.py:60-85 NewAPICheckin.__init__`：

- Cookie：`session=<值>`（+ 可选 `cf_clearance`）
- 请求头：`new-api-user: <user_id>` + 浏览器 UA + `Accept: application/json`（无 Authorization/Bearer，与 ccr 的 token 方式不同——它走的是"网页会话"身份而不是 API token）

### user_id 三级自动发现链

1. 配置显式提供（最优先）；
2. **Base64 解码 session cookie 后正则提取**（`checkin.py:87-119`）：尝试 `linuxdo[_-](\d+)`、`"id"[:\s]+(\d+)`、`user[_-](\d+)`、`userid[:\s]+(\d+)` 四种模式（NewAPI 的 session 是 gorilla/sessions Base64 编码，常含 `linuxdo_988` 这类 OAuth 用户名）；
3. 调 `/api/user/self` 成功后从 `data.id` 回填并更新请求头（`checkin.py:164-172`）。

### 配置加载优先级

`CONFIG_URL`（云端 WebDAV：坚果云/群晖/NextCloud/直链，支持 Basic 或 `token:` Bearer 认证，`checkin.py:390-467`）> `NEWAPI_ACCOUNTS` 环境变量。云端配置 JSON 还可顺带携带钉钉 webhook 配置。改站点配置不需要改代码、不需要推代码。

### 添加新站点的成本

**一行 JSON（url + session），零代码改动。**`config_helper.py` 还会在保存前用 `get_user_info()` 实测账号有效性（`config_helper.py:38-52`）。

## 通用签到流程与判定

### 三级回退主流程（`checkin.py:195-259 checkin()`）

```
1. requests 直连 POST /api/user/checkin（快速路径，30s 超时）
2. 响应判定为 CF 拦截 → _cf_bypass_checkin()：Playwright 无头浏览器
3. 浏览器内 CF 验证通过后，直接在页面上下文 fetch 签到（终极回退）
```

注释明确说流程"借鉴 Chrome 扩展 background.js:115-248"。

### 直连路径的判定顺序（`checkin.py:215-250`）

1. `status == 401` → "认证失败: Session 可能已过期"（直接终止，不重试）
2. JSON 解析失败 → 调 `detect_cloudflare_block()`，命中则转 CF 绕过；否则报"响应格式错误 (HTTP xxx) + 前 200 字符预览"
3. JSON 解析成功但 `status in (403, 503)` → 再跑一次 CF 检测（防 JSON 形式的拦截页）
4. `status == 200` 且 `data.success == true` → 成功；取 `data.message`、`data.data.checkin_date`、`data.data.quota_awarded`
5. `status == 200` 且 `success == false` → 失败，message 取 `data.message`
6. 其他状态码 → `HTTP {code}: {message}`

### 浏览器路径的宽容判定（`cf_bypass.py:189-217`，页面内 JS）

这是全仓库最有参考价值的一段——多协议成功判定 + 已签到关键词归一：

```javascript
const success = data.success === true || data.status === 'success' || data.ret === 1 || data.code === 0;
const message = data.message || data.msg || data.data || '签到完成';
const alreadyKeywords = ['已签到', '已经签到', 'already', '重复签到'];
const alreadyCheckedIn = !success && alreadyKeywords.some(k => msgStr.includes(k));
return { success: success || alreadyCheckedIn, alreadyCheckedIn, message: msgStr, ... };
```

- 成功标志兼容四种响应风格：NewAPI (`success`)、`status:'success'`、`ret:1`、`code:0`；
- message 兼容 `message`/`msg`/`data` 三种字段名；
- **"已签到"被归类为成功**（`alreadyCheckedIn` 单独标记），`checkin.py:298-303` 把它映射回 `success=True`。

### 已知不一致（repo 自身的坑）

直连快速路径（`checkin.py:247-248`）**没有**做已签到关键词归一——NewAPI 上游对重复签到返回 `success=false` + "今日已签到"类 message，直连模式下会被记为失败 ❌ 并计入 fail_count。只有走了 CF 浏览器路径才有这个归一逻辑。这是它通用判定上的一个漏洞，ccr 借鉴时应该把已签到归一放在统一出口处。

## 错误处理与特殊站点

### 错误分类（无通用重试）

| 类别 | 触发 | 处理 |
|---|---|---|
| 认证过期 | HTTP 401 | 直接失败，message 提示重新获取 session |
| CF 拦截 | 见下方签名 | 升级到 Playwright（唯一的"重试"形态） |
| 响应格式错误 | JSON 解析失败且非 CF | 失败 + 原始响应前 200 字符 |
| 超时 | `requests.Timeout`（30s） | 失败"请求超时" |
| 网络错误 | `RequestException` | 失败 + 异常信息 |
| 未知 | 兜底 `Exception` | 失败 + 异常信息 |

- **没有指数退避、没有同请求重试**——单账号单次尝试，失败只升级不重复。
- 通知层的会话过期判定靠 message 关键词：`'session' in message.lower() or '认证' in message`（`checkin.py:589`）。
- 退出码策略：**全部账号失败才 `sys.exit(1)`**，部分失败仍算成功（`checkin.py:608-609`），避免 Actions 误报。
- 日志全程脱敏：`_mask_url()`（域名中段打码）、`_mask_user_id()`（全打码）、用户名只显示前 3 字符。

### Cloudflare 检测签名（`cf_bypass.py:22-47 detect_cloudflare_block`）

| 条件 | 判定 |
|---|---|
| 403 + body 含 "Just a moment" | CF JS Challenge |
| 403 + `<!DOCTYPE html` + "cloudflare" | CF HTML Challenge |
| 503 + "cloudflare" + ("challenge" 或 "checking your browser") | CF Challenge (503) |
| body 非 JSON + `<!DOCTYPE` + ("Just a moment" / "challenge-platform" / "cf-challenge") | 非 JSON HTML 拦截页 |

检测是**纯运行时**的：任何站点哪天突然挂上 CF，行为自动适配，配置不用动。

### CF 绕过流程（`cf_bypass.py:115-230 bypass_and_checkin`）

关键设计（`checkin.py:261-266` 注释点明）：**CF 绕过和签到必须在同一个浏览器会话内完成，不能拆成"取 cf_clearance cookie → 回 requests 重试"两步，因为 cf_clearance 绑定浏览器指纹**。

1. Playwright chromium 无头启动，`--disable-blink-features=AutomationControlled` 等参数；
2. stealth 注入：`navigator.webdriver=undefined`、伪造 `window.chrome`、`plugins`、`languages`（`cf_bypass.py:151-156`）；
3. 把 `session` cookie 种到目标域名，goto base_url；
4. 轮询页面 title 等 CF 自动放行：最多 6 次 × 8 秒，挑战特征 = title 含 "Just a moment" / "Checking your browser" / "Attention Required" / cloudflare+challenge（`cf_bypass.py:74-113`）；
5. 通过后向 `localStorage` 注入 `user = {"id": user_id}`（NewAPI 前端从 localStorage 读 user id 发 `new-api-user` 头）；若 localStorage 无 user 还会兜底访问 `/login` 页再试；
6. 在页面上下文 `fetch('/api/user/checkin', {method:'POST', credentials:'include'})`，用上文的宽容判定解析。

CF 验证即使没过也会"尝试直接签到"（`cf_bypass.py:171-172`），不直接放弃。

### README 中的支持站点与已知坑

- **没有站点列表**。FAQ Q4（README.md:519-521）："理论上支持所有基于 [New API](https://github.com/Calcium-Ion/new-api) 项目搭建的站点，只要 API 接口兼容即可。"——它从根上回避了"内置站点清单"这个维护负担。
- 已知坑（散布在 README 故障排除章节 + 代码注释）：
  - Session 7-30 天过期，需定期更新；正常长度 200-500 字符，过短=复制不完整（README.md:617-645）；
  - 新版 NewAPI 强制要求 `new-api-user` 头，缺失会签到失败 → 脚本自动从 `/api/user/self` 取 id 回填（FAQ Q7）；`config_helper.py` 直接把 user_id 设为必填；
  - 签到奖励额度随机（约 2.5M-10M tokens），不是错误（FAQ Q6）；
  - HTML 错误页/维护页会导致 JSON 解析失败，需看原始响应排查；
  - GitHub Actions 长期无活动会被自动禁用 → 专门有 `keepalive.yml` 工作流保活；
  - CF 站点本地跑需手装 `playwright install chromium`。

## 对 ccr 的启示

对照 `crates/ccr-checkin/src/managers/checkin/builtin_providers.rs`（22 个内置站点，每个声明 checkin_path/balance_path/user_info_path/auth_header/auth_prefix/category/WAF/CF/CDK/OAuth）：

1. **"标准 NewAPI 模板"应该成为一等公民，而不只是内部构造函数**。ccr 的 `standard_provider()`（builtin_providers.rs:121-149）已经证明 14/22 个站点完全同构（仅 base_url/OAuth client_id 不同）。Newapi-checkin 证明标准站只需要 base_url 一个站点级输入。建议：把"添加自定义 NewAPI 站点 = 粘贴一个 URL"做成用户可见能力，内置列表只是这个模板的预填实例。

2. **WAF/CF 从"配置时静态标记"改为"运行时检测 + 标记作提示"**。ccr 的 `requires_waf_bypass` / `requires_cf_clearance` 会过时（站点随时加/撤 CF）。Newapi-checkin 的 `detect_cloudflare_block(status, body)` 四条签名（403+Just a moment / 403+DOCTYPE+cloudflare / 503+challenge / 非JSON+challenge-platform）成本极低，可以对**所有**站点的每次签到响应都跑一遍：命中就走绕过链路，并可顺手提示用户"该站点现在需要 CF 绕过"。静态标记降级为首次请求的快捷提示。

3. **统一的宽容响应解释器，减少"签到报错"**。把 `cf_bypass.py:199-203` 的判定收敛成一个 Rust 函数：
   - 成功 = `success==true || status=="success" || ret==1 || code==0`；
   - message = `message || msg || data`；
   - **已签到关键词（已签到/已经签到/already/重复签到）一律归一为"成功（今日已签）"状态**——并且要放在所有路径的统一出口（Newapi-checkin 自己就栽在直连路径漏了这一步）。
   这能直接消灭一批"其实已签到却报失败"的用户报错。

4. **升级链模式与 cf_clearance 指纹绑定**。直连（便宜）→ 检测到拦截 → 浏览器会话内完成"过盾 + 签到"一条龙，**不要**把 cf_clearance 抠出来回 HTTP 客户端重放（指纹不匹配会被再次拦截）。这与 ccr 最近的 WAF cookie 自动恢复流（任务 06-10-ccr-ui-checkin-waf-cookie-flow）方向一致，可作为该流程的设计佐证：拿到 cookie 后若仍被拦，应在同一浏览器上下文里完成签到而不是反复回 HTTP 层重试。

5. **user_id / new-api-user 自动发现链**：显式配置 > session cookie Base64 正则（`linuxdo_(\d+)` 等 4 个模式）> `/api/user/self` 响应回填。ccr 如果支持 session-cookie 认证模式的站点，可复用这条链减少用户必填项。

6. **错误分类对齐**：401=会话过期（可触发"请重新登录"动作）、CF 拦截（触发绕过）、非 JSON 响应（附原始 body 前 200 字符帮助排查）、超时、网络错误。"会话过期"再用 message 关键词兜底（'session'/'认证'）。汇总层面"全部失败才算任务失败"。

7. **配置数据化**。builtin_providers.rs:361 注释提到标准站"同步自 PROVIDERS.json"——可以更进一步：内置站点清单整体下沉为数据文件（打包资源或远端可更新），新站点上线/站点行为变化（如新挂 CF）不需要发版。Newapi-checkin 的云端配置（WebDAV 直链 + 优先级覆盖）是同一思想的极简版。

8. **保留 ccr 的差异化优势**：Newapi-checkin 没有余额查询、CDK 充值、OAuth 登录、AgentRouter 式"查询即签到"等特殊机制的处理——这些 ccr 的 `category`/`cdk_config`/`oauth_config` 仍有必要，通用化应只针对 standard 类站点收敛配置，特殊站点保持显式声明。

## Caveats / Not Found

- 该 repo **没有 pyproject.toml**（提问中提到），依赖管理只有 `requirements.txt`（requests / pytz / playwright）。
- 该 repo **没有余额查询**功能（ccr 的 balance_path 在这里无对应物），也没有任何 token/Bearer 认证——全部走 session cookie，与 ccr 的 Authorization Bearer 模式是两种身份体系。
- `config_generator.html`（约 38KB 网页工具）和 `dingtalk_notifier.py` 剩余部分未逐行分析（与签到通用逻辑无关，前者只是表单生成 JSON，后者是通知渲染）。
- `cf_bypass.py` 源文件含少量乱码字符（如"阴护""磀到"），是上游文件编码问题，不影响逻辑判读。
- 直连路径"已签到判为失败"的不一致是从代码推断（NewAPI 上游对重复签到返回 `success=false`），未实际运行验证。
