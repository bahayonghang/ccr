# Research: 签到链路与 Provider 模板体系内部架构

- **Query**: 摸清签到链路与 Provider 模板体系，定位 (a) 签到报错产生/展示路径 (b) 性能体验热点 (c) 签到站目录与 Provider 模板打通的集成点
- **Scope**: internal（仓库内代码研究）
- **Date**: 2026-06-10

---

## 签到链路全景

```
SQLite (ccr-db checkin_repo)
   ↑ database::with_connection
managers/checkin/*  ← 全部是无状态 facade（ProviderManager / AccountManager / BalanceManager / RecordManager / WafCookieManager）
   ↑
CheckinService (crates/ccr-checkin/src/services/checkin_service.rs)
   ↑ CheckinService::with_client(checkin_dir, AppState.http_client)
Tauri commands (ccr-ui/src-tauri/src/commands/checkin.rs ~30 个命令; commands/waf.rs WAF WebView)
   + checkin_jobs.rs (CheckinJobSnapshot/Delta 状态机, 存于 AppState)
   ↑ invoke() / listen()
api/domains/checkin.ts (invoke 薄封装)
   ↑
useCheckinState (views/checkin/composables/useCheckinState.ts) —— 主状态容器（不是 Pinia store）
   ├─ checkinDataState.ts   页面数据加载（5 路并发 invoke）
   ├─ checkinJobRuntime.ts  签到 Job 启动 + 事件监听（delta/finished/timeout）
   └─ checkinWafRecovery.ts WAF 自动补救（开 WebView → 验证 → 重试）
   ↑ props 下发
CheckinView.vue → 4 个 v-if Tab（Providers / Accounts / Records / ImportExport）+ CheckinProgressModal + OAuthWizardModal
```

### 单账号签到执行流（`CheckinService::checkin`，checkin_service.rs:508-709）

1. 加载 account + provider，初始化 CryptoManager（:509-523）。
2. 本地判重 `RecordManager::has_checked_in_today`（:533-535；record_manager.rs:118-125，看今天是否有 Success/AlreadyCheckedIn 记录）。
3. 解密 cookies JSON → `CookieCredentials`（:565-570）。
4. **远程预查**：GET `{base_url}{user_info_path}` 解析 `check_in_today`/`is_checked_in`/`checkin_status` 字段（:573-609，实现 :712-789）。已签到则直接落 AlreadyCheckedIn 记录返回。
5. `do_checkin`：POST `{base_url}{checkin_path}`，Cookie = 账号 cookies + 缓存 WAF cookies + 缓存 CF cookies 合并（:806-1057）。
   - WAF/CF 挑战页检测（`is_waf_challenge` :280-286、`is_cf_challenge` :289-298）→ 尝试 `refresh_waf_cookies`/`refresh_cf_cookies`，**当前版本两者均直接返回 Err（无头绕过未实现，:383-397 / :411-427）**，软失败继续用原响应；真正的 WAF cookie 来源是 Tauri WebView（commands/waf.rs `open_waf_login`）写入的 24h 缓存（waf_cookie_manager.rs:161-184）。
   - JSON 成功判定：`ret==1 || code==0 || code==200 || success==true`（:967-970）；消息含「已/already」→ 返回 `[ALREADY_CHECKED_IN]` 前缀标记（:988-991）。
6. 落 CheckinRecord（成功/已签/失败含 error_code）+ `update_checkin_time`（:693-699）。
7. CDK 充值 `try_cdk_topup` 当前是 no-op（:795-803）。

### 批量签到 Job 流（主 UI 路径）

- 前端「一键签到/单账号签到」都走 `start_checkin_job`（checkinJobRuntime.ts:199-208 → api/domains/checkin.ts:124-126），**不走** `execute_checkin`/`batch_checkin` 直连命令（后两者仍存在，api 层有导出）。
- 后端 `start_checkin_job`（commands/checkin.rs:639-679）：去重账号 → 构造 pending 日志快照存入 AppState → `tauri::async_runtime::spawn(run_checkin_job)` 立即返回 `{job_id, snapshot}`。
- `execute_checkin_job_accounts`（commands/checkin.rs:200-287）：JoinSet + `Semaphore(5)` 并发，单账号 `timeout(90s)`，整个 Job `timeout(600s)`（:304）。
- 进度推送：`checkin:job-delta` 事件只带增量（CheckinJobDelta，checkin_jobs.rs:65-84，注释明确为避免 IPC O(N²)）；终态推 `checkin:job-finished` / `checkin:job-timeout` 全量快照。
- 前端 `startAndTrackCheckinJob`（checkinJobRuntime.ts:132-197）：注册 3 个 listen → 再拉一次 `get_checkin_job_status` 兜底（防事件早于监听注册的竞态，:180-187）→ 终态 `finalizeCheckinJob` 刷新数据 + 触发 `runWafRecovery`。
- WAF 自动补救（checkinWafRecovery.ts:236-357）：按 `error_code === 'waf_blocked'` 把失败结果按 provider_name 分组（:144-163）→ `open_waf_login` 打开隐藏 WebView 等用户/自动过 WAF（waf.rs:250-381，60s 超时，required cookies 齐全才落缓存）→ `validate_waf_cookie_for_account` 只读验证（waf.rs:439-449 → checkin_service.rs:1620-1720）→ 通过后对该组账号重新 `start_checkin_job` 并**轮询** `get_checkin_job_status`（500ms × 240 次，checkinWafRecovery.ts:210-223）→ 合并重试结果回展示模型。

### 旁路与遗留

- Pinia `stores/checkin.ts`（30s TTL 缓存）只被 `views/checkin/components/{CheckinStats,AccountManager,CheckinHistory}.vue` 使用，它们属于 `CheckinManageView.vue`——**该 View 未注册路由、无任何引用，是遗留死代码**（router/index.ts:307-318 只注册 CheckinView 与 CheckinAccountDashboardView）。
- `get_account_dashboard`（commands/checkin.rs:1188-1212 → checkin_service.rs:1454-1514）是「一个命令返回聚合体（account+streak+calendar+trend）」的好范式。

---

## 错误产生与展示路径（含每层代码位置）

### 第 1 层：reqwest → CheckinServiceError

| 来源 | 转换 | 位置 |
|---|---|---|
| `send()` / `bytes()` 失败 | `CheckinServiceError::Network(e.to_string())` | checkin_service.rs:453-462, 492-501 |
| HTTP 非 2xx 且 body 是 JSON | `Api("HTTP {status}: {msg/message/error}")` | checkin_service.rs:942-954 |
| 业务失败（success=false） | `Api(message)` | checkin_service.rs:993-995 |
| WAF 挑战页（HTML） | `Api("检测到 WAF 挑战页面…")`（消息内嵌 "WAF" 关键字） | checkin_service.rs:1023-1027, 1043-1048 |
| CF 挑战页 | `Api("检测到 Cloudflare 挑战页面…")` | checkin_service.rs:1030-1034 |
| 非 JSON 响应 | `Api("HTTP {}: 返回非 JSON 响应")` / `Api("无法解析响应…")` | checkin_service.rs:1037-1040, 1053-1055 |
| 余额缺字段 | `Api("无法解析余额响应，缺少 quota/used_quota/balance 字段…")` | checkin_service.rs:1347-1350 |
| manager 错误 | `Provider/Account/Crypto/Record/Balance(e.to_string())` | checkin_service.rs:512-523 等 |

### 第 2 层：error_code 分类（crates/ccr-checkin/src/core/error.rs:34-65）

枚举共 8 变体；`error_code()` 输出值全集：

- `provider_error` / `account_error` / `crypto_error` / `network_error`（直接按变体）
- `Api(msg)` 内部**靠消息字符串匹配再细分**：
  - 含 `WAF`/`waf` → `waf_blocked`
  - 含 `Cloudflare`/`cf_clearance`/`cloudflare` → `cf_blocked`
  - 含 `401`/`403`/`Unauthorized`/`cookie`/`Cookie`/`token`/`expired` → `cookie_expired`
  - 其余 → `api_error`
- `Record`/`Balance`/`Database` → 一律 `api_error`
- Tauri 层额外造出两个码：`task_error`（commands/checkin.rs:86-91）、`timeout`（:238；checkin_jobs.rs:253）

注意：分类依赖中文/英文消息关键词（如 WAF 错误消息必须含 "WAF"），是**隐式契约**，spec 已要求保留该分类（.trellis/spec/ccr-checkin/backend/backend-guidelines.md:32）。该分类逻辑本身无单元测试。

### 第 3 层：service 层是否保结构

- `CheckinService::checkin` **把 `do_checkin` 的 Err 捕获并转成 `Ok(CheckinExecutionResult{status:Failed, message, error_code})`**（checkin_service.rs:661-691）——结构化信息保留。`checkin()` 只在基础设施失败（账号不存在/解密失败/记录写入失败）时返回 Err。
- `batch_checkin`（service 内）对 Err 也保留 `e.error_code()`（checkin_service.rs:1374-1386）。

### 第 4 层：Tauri command —— 两条路径，两种命运

1. **Job 路径（主路径，保结构）**：`Ok(Ok(result))` 原样进快照；但 `Ok(Err(error))` 走 `build_failed_checkin_result` 时 message 变成 `"Checkin failed: {error}"` 且 error_code **硬编码 `task_error`**，丢掉了 `error.error_code()` 分类（commands/checkin.rs:231-239 + :86-91）——例如账号解密失败本应是 `crypto_error`，前端拿到的是 `task_error`。
2. **直连命令路径（压扁点）**：所有命令签名是 `Result<Value, String>`，错误统一 `map_err(|e| format!("…: {}", e))`：
   - `execute_checkin`：`format!("Checkin failed: {}", e)`（commands/checkin.rs:585-599）
   - `get_balance`：`format!("Failed to query balance: {}", e)`（:721-731）
   - 其余 list/add/update/delete 同模式。
   error_code 在此完全丢失，只剩拼接字符串。

### 第 5 层：事件载荷（保结构）

`CheckinJobLogEntry`/`CheckinJobDelta`/`CheckinJobSnapshot` 携带 `message` + `error_code` + `status`（checkin_jobs.rs:26-40, 50-84）；`mark_pending_failed`/`mark_timed_out` 给未完成账号补 `task_error`/`timeout` 失败结果（:199-271）。

### 第 6 层：前端接收与转换

- 事件 → `mapCheckinJobLogEntry`（checkinWafRecovery.ts:56-66）→ `CheckinLogEntry`（camelCase）。
- 失败详情拼装 `getFailedDetail`：`message`（缺省「未知原因」）+ 按 error_code 查 i18n hint 括号注释 + WAF 补救失败附加说明（useCheckinState.ts:366-376）；`getErrorLabel`/`getErrorHint` 映射表覆盖 10 个码（:332-364）。
- 批量聚合展示：summary 后端算好（checkin_jobs.rs:176-189）；CheckinView 结果面板按 status 分三组渲染（useCheckinState.ts:168-181 + CheckinView.vue:102-260 区域），失败时自动 `scrollIntoView`（checkinJobRuntime.ts:114-117）；进度弹窗逐账号日志（CheckinProgressModal）。

### ★ 「未知错误」的真正来源（关键发现）

前端到处使用的错误提取函数：

```ts
// useCheckinState.ts:44-45（同样模式见 CheckinAccountsTab.vue:732-733、CheckinRecordsTab.vue:325-326、CheckinProvidersTab.vue:420-421）
const getErrorMessage = (error: unknown, fallback: string) =>
  error instanceof Error ? error.message : fallback
```

Tauri v2 的 `invoke()` 对 `Result<_, String>` 的 Err **以普通字符串 reject，不是 `Error` 实例**。因此 `error instanceof Error === false`，后端辛苦拼出来的 `"Checkin failed: Network error: …"` 被直接丢弃，UI 显示兜底文案 `t('checkin.errors.unknown')` /「未知错误」。这是「前端只能显示未知错误」的主犯——压扁发生在 Tauri 边界（String），**丢失发生在前端 getErrorMessage**。

其它吞错点：

- `refreshAllBalances` 用 `Promise.allSettled` 后**只处理 fulfilled，rejected 全部静默丢弃**（useCheckinState.ts:221-228），批量刷余额单账号失败用户完全不可见（仅 `logger.error` 整体 catch）。
- 错误展示通道不统一：`alert()`（checkinJobRuntime.ts:194、useCheckinState.ts:252/265）vs `uiStore.showError` toast（CheckinAccountsTab.vue:912-985、CheckinProvidersTab）混用。
- 记录页失败记录 `message` 为空时显示「未知原因」（CheckinRecordsTab.vue:378-390）。

---

## 性能与体验热点

### 并发与节流现状

| 操作 | 模式 | 位置 |
|---|---|---|
| 批量签到（Job） | 并发，`Semaphore(5)`，单账号 90s 超时，Job 全局 600s | commands/checkin.rs:207, 233, 304 |
| service 层 batch_checkin | 并发，`Semaphore(5)`（join_all） | checkin_service.rs:1353-1392 |
| 批量余额刷新 | 前端 `Promise.allSettled` **无上限并发**，N 账号即 N 个并发 invoke + N 路 HTTPS，后端无信号量 | useCheckinState.ts:214-239 |
| WAF/CF cookie 刷新 | 全局 Mutex 防并发开浏览器（但刷新本身未实现） | checkin_service.rs:132-136 |
| WAF cookie 缓存 | SQLite 24h TTL，读取时顺带清理过期 | waf_cookie_manager.rs:11, 57-73 |
| 前端缓存 | Pinia store 30s TTL 存在但主视图不用（死代码路径）；useCheckinState 每次挂载全量重拉 | stores/checkin.ts:11,61-68 vs useCheckinState.ts:382-384 |

### 进度通知：事件推送为主，遗留一处轮询

- 主路径是事件推送（delta 增量，O(变化量)），并有一次性 `get_checkin_job_status` 对账（checkinJobRuntime.ts:164-187）——设计良好。
- **WAF 补救后的重试却退化为 500ms 轮询**（最长 120s，checkinWafRecovery.ts:210-223），没有复用事件监听。

### 数据加载瀑布

- 页面打开 = 5 路并发 invoke：`list_providers` + `list_accounts` + `get_checkin_records(limit=100)` + `export_checkin_stats` + `list_builtin_providers`（checkinDataState.ts:54-92）。无串行瀑布，但**无缓存**，每次进入页面全量重发。
- 每次签到 Job 结束 / 刷余额后 `refreshCheckinData` 再发 2-3 路（accounts+records+stats）（checkinDataState.ts:108-156）；WAF 补救每组成功后又各刷一轮（checkinWafRecovery.ts:334-338）。
- `list_accounts` 后端在单个 blocking task 里做 enrich（provider 名 + 最新余额 map），无 N+1（commands/checkin.rs:437-482）；但 `AccountManager::list` **对每个账号解密一次 cookies 仅为了生成掩码**（account_manager.rs:52-66, 90-114），账号多时是无谓的解密开销。
- 单账号签到至少 2 个 HTTP 请求（远程预查 GET + 签到 POST），批量 N 账号 ≥ 2N 请求（checkin_service.rs:573-614）。
- `BalanceManager::add` 每次插入都执行一次 90 天过期清理扫描（balance_manager.rs:35-49），批量刷 N 账号即 N 次清理。
- `list_waf_cookies` 遍历全部 provider 逐个查询（commands/checkin.rs:971-996）。

### ★ 记录筛选/分页是断的（功能性 bug，影响体验）

`CheckinRecordsTab.loadFailedHistory` 构造了 `status: 'failed'` + `provider_id` + `keyword` + `page`/`page_size`（CheckinRecordsTab.vue:405-427），但：

1. api 层 `listCheckinRecords` 只透传 `accountId/limit/page`，**status/provider_id/keyword 被丢弃**（api/domains/checkin.ts:156-169）；
2. Tauri 命令 `get_checkin_records` 签名只有 `account_id + limit`，**page 也被忽略**（commands/checkin.rs:695-716）；
3. 后端 `RecordManager::get_paginated_advanced` / `get_filtered_advanced` 支持 SQL 级 status/provider/keyword/分页过滤，**但全仓库无任何调用方**（record_manager.rs:66-107）。

结果：「失败历史」面板实际显示的是最近 `page_size` 条**任意状态**记录，过滤与翻页都是假的。优化任务里这是现成的低垂果实（把命令接到 `get_paginated_advanced` 即可）。

### CheckinAccountsTab.vue 62KB 拆解

总 2062 行 = template 1-590（账号卡片网格 + 搜索过滤 + 浮动操作菜单 + 账号编辑大弹窗）+ script 592-1012（约 420 行）+ **scoped CSS 1014-2062（约 1050 行，体积大头）**。

script 内容：账号 CRUD（create/update/delete + cookies 解密回填）、session↔cookies JSON 互转（:794-831）、CDK 三种凭证字段（fuli/b4u/x666，:660-678）、浮动菜单手写定位算法（:740-791）。

可拆方向：`AccountFormModal`（含 CDK 凭证区）、`AccountActionsMenu`（浮动菜单 + 定位逻辑）、账号卡片组件；CSS 中大量与 ProvidersTab 雷同的卡片/徽章样式可抽公共层。其余大文件：CheckinView.vue 1599 行（其中 style ~900 行）、CheckinProvidersTab.vue 1090 行（style ~500 行）、ProviderTemplateSelector.vue 1261 行。

### 其它体验点

- Tab 用 `v-if` 切换（CheckinView.vue:518-547），切 Tab 即销毁重建（菜单监听器重挂，但数据来自父级 props 不会重拉）。
- 批量签到弹窗 + 结果面板双通道展示同一份结果；失败自动滚动定位。
- 余额刷新成功后用 `applyBalanceSnapshot` 原位更新账号行（checkinDataState.ts:94-106），避免整表重拉——好模式，但随后又 `reloadRecords+reloadStats`。

---

## 两套站点目录字段对比表

### 数据源概况

- **签到内置站**：`crates/ccr-checkin/src/managers/checkin/builtin_providers.rs`，Rust 硬编码 22 个站（8 特殊 + 14 标准 NewAPI 公益站，测试断言见 :551-560），注释标明标准站「同步自 PROVIDERS.json」（:361）。前端有手工镜像 TS 接口 `BuiltinProvider`（ccr-ui/src/types/checkin.ts:23-58）。
- **Provider 模板**：`ccr-ui/src/configs/providerTemplates.ts` 纯前端静态合成 = 35 个 Claude presets（configs/providerPresets/claude.ts）+ 3 个 Codex 专属模板 + 5 个 OpenCode presets（types/opencode.ts:229-256），经 `mergeTemplates` 按 id 合并（:298-328）。类型 `ProviderTemplate`（types/providerTemplates.ts:53-69）。

### 逐字段对比

| 概念 | BuiltinProvider（builtin_providers.rs:10-49） | ProviderTemplate（providerTemplates.ts:53-69） | 备注 |
|---|---|---|---|
| 唯一 ID | `id`（固定 `builtin-` 前缀） | `id`（slug） | 重叠 |
| 名称 | `name` | `name` | 重叠 |
| 描述 | `description` | 无核心字段（仅 `platforms.claude.description`） | 部分重叠 |
| 分类 | `category`: standard / waf_required / cf_required / special / balance_only / cdk（**按签到机制分**） | `category`: official / cn_official / aggregator / third_party / local（**按供应商性质分**） | 同名不同义，打通时需双轴 |
| 域名/地址 | `domain`（展示）+ `base_url`（单个 API 地址） | `baseUrls[]`（多 endpoint）+ `websiteUrl`（官网）+ `apiKeyUrl`（取 key 文档） | 部分重叠；模板支持多 endpoint |
| 图标 | `icon`（emoji） | 无 | 签到独有 |
| **签到独有** | `checkin_path: Option`、`balance_path`、`user_info_path`、`auth_header`、`auth_prefix`、`supports_checkin`、`requires_waf_bypass`、`requires_cf_clearance`、`checkin_bugged`、`cdk_config{cdk_type, cdk_source_url, topup_path, requires_cdk_cookies, requires_access_token}`（:52-65）、`oauth_config{github_client_id, linuxdo_client_id, oauth_state_path}`（:68-78） | — | 签到 API 端点 / 认证 / WAF / CDK / OAuth 全部是模板侧没有的 |
| **模板独有** | — | `aliases[]`（搜索别名）、`tags[]`、`modelCatalog[]`、`isOfficial`、`isPartner`、`source: built_in/custom`、`platforms.{claude,codex,opencode}` 平台 override（types/providerTemplates.ts:12-51）、`createdAt/updatedAt` | 模板核心价值在跨平台 override 与可搜索元数据 |

平台 override 字段（模板独有的第二层）：
- `claude`: baseUrl / provider / providerType / model / smallFastModel / defaultOpus|Sonnet|HaikuModel / subagentModel / description（types/providerTemplates.ts:12-23）
- `codex`: baseUrl / websiteUrl / apiKeyUrl / modelCatalog / model / protocol…（:25-35）
- `opencode`: id / name / npm / baseURL / models / extraOptions / rootExtra（:37-45）

### 实际站点重叠

**零重叠。** 22 个内置签到站全是 NewAPI 公益中转站（anyrouter.top、agentrouter.org、api.codemirror.codes、runanytime.hxi.me、elysiver.h-e.top、hotaruapi.com、b4u.qzz.io、x666.me、codex.cab、clove.cc.cd…）；模板侧 40+ 条目是模型厂商/商业聚合商（DeepSeek、Kimi、OpenRouter、SiliconFlow、智谱、PackyCode、AICodeMirror…）。最接近的一对是模板 `aicodemirror`（www.aicodemirror.com，claude.ts:327-333）与内置 `builtin-coderouter`（api.codemirror.codes），名字相似但是不同站点。两套目录服务两类用户场景（白嫖签到 vs 付费 API 配置），打通的价值在**机制统一**而非数据合并去重。

---

## 打通集成点与可复用机制

### 现有集成点盘点

1. **前端获取内置签到站**：`invoke('list_builtin_providers')`（api/domains/checkin.ts:254-256 → commands/checkin.rs:1059-1071，serde 全字段序列化）→ `checkinDataState.loadAllData`（:64, :80）→ `builtinProviders` ref → props 下发到 CheckinProvidersTab / CheckinAccountsTab / OAuthWizardModal。
2. **CheckinProvidersTab 展示/添加**：`availableBuiltinProviders` = builtin 列表过滤掉已添加（**按 name 匹配**，useCheckinState.ts:136-139）→ 卡片网格（CheckinProvidersTab.vue:5-100）→ `emit('add-builtin', bp.id)` → `invoke('add_builtin_provider')`（commands/checkin.rs:1074-1102）→ `BuiltinProvider::to_checkin_provider()` 落库（builtin_providers.rs:83-100）。
3. **★ 元数据降级与 name-join 缝隙**：`to_checkin_provider` 只保留 base_url + 4 个路径/认证字段，**icon / category / waf 标记 / cdk_config / oauth_config 全部丢弃**。之后运行期到处用「名字反查 builtin」补回元数据：
   - 前端 WAF 标记：`builtinProviderMap.get(provider.name)`（CheckinProvidersTab.vue:436-448）
   - 前端 CDK 表单字段：`builtinProviders.find(bp => bp.name === provider.name)`（CheckinAccountsTab.vue:672-685）
   - 后端 CDK 充值：`bp.name == provider.name || bp.id == format!("builtin-{}", name.to_lowercase())`（commands/checkin.rs:840-847）
   - WAF 策略：`policy_for_provider_parts` 按 id/name/base_url 包含 anyrouter 硬编码（waf_cookie_manager.rs:79-103）
   用户改名 provider 即断链——这是统一目录方案必须吃掉的最大债务。
4. **模板选择器数据流**：静态 `BUILT_IN_PROVIDER_TEMPLATES`（构建期 TS 合成）+ 自定义模板（localStorage key `ccr.providerTemplates.custom.v1`，utils/providerTemplates.ts:18, 142-168）→ `useProviderTemplates().templates` 合并（composables/useProviderTemplates.ts:18-23）→ `buildProviderTemplateOptions(templates, platform)` 按平台 override 过滤并按 endpoint 展开成选项（utils:262-292）→ `ProviderTemplateSelector.vue`（搜索/键盘/自定义编辑器）→ 选中后走平台 mapper 出 patch（`mapTemplateToClaudeProfilePatch` 等 6 个，utils:380-493，保证不写密钥字段）。
5. **模板消费方**：ClaudeCodeProfilesView.vue:325、AddConfigModal.vue:55（ConfigsView 的新增配置弹窗）、CodexAuthView.vue:614/1544、CodexProfileEditorModal.vue:88、OpenCodeProvidersView.vue:157/179、ProviderPresetSelector.vue（兼容壳）。契约 spec：`.trellis/spec/ccr-ui/frontend/provider-template-contracts.md`（非敏感约束、平台可见性、mapper 白名单、测试要求）。

### 单一事实来源的可复用机制

1. **「Tauri command 返回目录」模式已成熟**：`list_builtin_providers` 即此模式，且 `BuiltinProvider` 是 `Serialize/Deserialize` 全字段。最小改动方案：模板目录同样后端化（新增 `list_provider_templates` 类命令），或扩 `list_builtin_providers` 返回合并目录；前端 `useProviderTemplates` 数据源从静态 import 换成 invoke（`checkinDataState` 已示范运行期拉目录 + ref 缓存的写法）。
2. **`platforms` override 模式天然可扩签到维度**：给 `ProviderTemplate.platforms` 增加 `checkin` override（checkin_path / balance_path / auth_header / requires_waf_bypass / cdk / oauth），即可把 BuiltinProvider 整体降维成一个平台 override，分类双轴问题用 `tags` 或独立 `checkinCategory` 解决。contracts spec 的「平台可见性 = 有 override 才出现」规则直接复用。
3. **共享 JSON + 生成的先例**：builtin_providers.rs:361 注释「同步自 PROVIDERS.json」表明上游本就有 JSON 目录源（`.trellis/tasks/06-08.../research/cockpit-provider-templates.md` 与本任务 `research/newapi-checkin.md` 也引用过）。仓库已有「单一源传播」基建：`just version-sync` 脚本把 root Cargo.toml 版本撒到全部 crate/package——同样思路可做 `providers.json → build.rs 生成 Rust const + 脚本生成 TS`，消除 types/checkin.ts:23-58 对 Rust struct 的手工镜像（当前已是双端漂移风险点）。
4. **自定义模板持久化差异**：模板自定义存 localStorage（仅前端、不随 WebDAV 同步、Web 预览可用）；签到 provider 存 SQLite。统一时若自定义模板也入 SQLite，需走 Tauri 命令 CRUD（参考 ui_state.rs / checkin provider CRUD），并迁移 localStorage 旧数据。
5. **id 约定可对齐**：builtin 用 `builtin-` 前缀标识不可删除，模板用 `source: built_in/custom` + 重名 id 改写 `${id}-built-in`（utils/providerTemplates.ts:128-140）——统一目录需要先定一套 id/来源语义。

---

## 测试现状

### ccr-checkin（Rust，共 50 个单测，全部内嵌 `#[cfg(test)]`，无 tests/ 集成目录）

| 文件 | 数量 | 覆盖点 |
|---|---|---|
| services/checkin_service.rs | 10 | 响应元数据辅助、reward 解析、daily_checkins/streak/calendar 聚合（:2078-2297） |
| services/cdk_service.rs | 6 | CDK 解析/配置 |
| core/crypto.rs | 5 | 加解密 |
| managers/checkin/builtin_providers.rs | 6 | 22 站完整性、OAuth 元数据、唯一 id、URL 规范（:545-658） |
| managers/checkin/waf_cookie_manager.rs | 6 | 缓存读写、AnyRouter 策略、cookie 解析/必选筛选（:209-345） |
| managers/checkin/account_manager.rs | 5 | CRUD |
| managers/checkin/provider_manager.rs | 4 | CRUD（:176-290） |
| managers/checkin/balance_manager.rs | 4 | 快照/历史 |
| managers/checkin/record_manager.rs | 4 | 记录 |

运行方式：`cargo test -p ccr-checkin -- --test-threads=1`（spec 要求，backend-guidelines.md:115-121）。**core/error.rs 的 error_code() 字符串分类无任何测试。**

### Tauri 层（src-tauri）

`commands/checkin.rs`、`commands/waf.rs`、`checkin_jobs.rs` 均**无 `mod tests`**（src-tauri 有 33 个文件带测试，签到三件套不在其中）。Job 状态机（mark_processing/apply_result/mark_pending_failed/mark_timed_out）与 delta 生成完全无测试。

### 前端（vitest smoke）

| 文件 | 用例 | 覆盖点 |
|---|---|---|
| tests/checkin-state.smoke.test.ts | 3 | WAF 重试日志合并、补救失败标记、缺失 cookie 文案（针对 checkinWafRecovery 导出的纯函数） |
| tests/checkin-accounts-tab.smoke.test.ts | 8 | i18n 渲染、菜单 teleport、api_user 回填、enabled 提交、cookies JSON 保留 |
| tests/checkin-progress-modal.smoke.test.ts | 2 | recovering 阶段禁关 / finished 可关 |
| tests/provider-templates.smoke.test.ts | 8 | 平台过滤与搜索索引、mapper 非敏感字段、自定义模板持久化与密钥剥离、键盘选择、override JSON 校验 |

### 缺口清单（优化任务可直接对照）

- error_code 分类（error.rs）与「消息关键字 → 分类」契约无单测。
- Tauri job 状态机、delta 合并（checkinJobRuntime.applyCheckinJobDelta）、checkinDataState 加载分支均无测试。
- 记录筛选参数链路（前端 query → api 丢参 → 命令缺参）无回归测试，bug 长期潜伏。
- getErrorMessage 对 Tauri 字符串 rejection 的行为无测试（「未知错误」问题不会被现有测试抓到）。

## Caveats / Not Found

- 「Tauri invoke 对 `Result<_,String>` 以普通字符串 reject」基于 Tauri v2 API 行为与代码模式推断（仓库内无把 rejection 包装成 Error 的全局封装，api/domains 直接调 `invoke`），建议实现修复时用一个失败命令实测确认。
- `PROVIDERS.json` 原始文件不在仓库内（仅注释与历史 research 提及），同步流程是手工的。
- 未深入 OAuthWizardModal 的完整 OAuth 流程（仅确认其消费 `builtin_providers.oauth_config`，OAuthWizardModal.vue:89-122, 451-455）与 CheckinAccountDashboardView 渲染细节，与本任务关联较弱。
