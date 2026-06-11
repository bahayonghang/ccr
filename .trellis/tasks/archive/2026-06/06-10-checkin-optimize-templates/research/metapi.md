# Research: metapi 如何建模和管理 API 站点目录

- **Query**: ref/repo/metapi(TypeScript Web 应用)如何建模/管理 API 站点目录;签到、健康监测、多端共享数据模型
- **Scope**: internal(只读参考仓库 `ref/repo/metapi`)
- **Date**: 2026-06-10
- **上游项目**: cita-777/metapi(MIT,Fastify + React + Drizzle + Electron)

## 项目概览

**定位:「中转站的中转站」(Meta-Aggregation Layer)。** 它管理的不是公共站点黄页,而是**用户自己注册的中转站账号资产**:把分散在 New API / One API / OneHub / DoneHub / Veloera / AnyRouter / Sub2API 等站点的账号、余额、模型、API Key 聚合成一个统一代理入口(`/v1/*`),下游工具(Cursor / Claude Code / Codex 等)用一个 Key 访问全部模型。

它管理的对象层级(README.md + `src/server/db/schema.ts`):

```
sites(上游站点,1 站可多端点)
  └─ accounts(1 站多账号,含余额/签到/凭证)
       └─ account_tokens(1 账号多 API Token)
tokenRoutes / routeChannels(模型路由 → 具体账号/Token 通道)
downstream_api_keys(发给下游工具的 Key)
```

核心能力:统一代理网关、智能路由(成本/余额/使用率加权)、自动故障转移与冷却、**自动签到**(cron,奖励追踪)、余额定时刷新、模型自动发现、五渠道告警通知。技术栈:Fastify + React 18 + Vite + TypeScript + Drizzle ORM(SQLite/MySQL/PG)+ node-cron + Electron 桌面壳 + Docker。

**对我们的研究问题而言最重要的一点:metapi 没有「公益站/商业站」分类,也没有内置 22 个站那样的公共站点目录。** 它的「内置数据」只有两类:首次启动 seed 的 4 个官方站,和 13 个官方厂商「初始化预设」(见下文)。

## 站点实体与 schema

Schema 单文件定义:`ref/repo/metapi/src/server/db/schema.ts`(SQLite 方言,迁移在 `drizzle/*.sql`,27 个迁移)。

### sites 表(schema.ts:4-27)

| 字段                                                     | 说明                                                                                                                                                                                                                      |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name` / `url`                                           | 展示名 + 主 URL;`uniqueIndex(platform, url)` 防重                                                                                                                                                                         |
| `platform`                                               | **核心分类字段**。注释枚举:`'new-api' \| 'one-api' \| 'veloera' \| 'one-hub' \| 'done-hub' \| 'sub2api' \| 'openai' \| 'claude' \| 'gemini' \| 'codex' \| 'gemini-cli' \| 'antigravity'`(另有 `cliproxyapi`、`anyrouter`) |
| `externalCheckinUrl`                                     | 站点签到页在另一个域名时的外链(签到能力相关的唯一站点级字段)                                                                                                                                                              |
| `proxyUrl` / `useSystemProxy` / `customHeaders`          | 每站点出站代理与自定义请求头                                                                                                                                                                                              |
| `status`                                                 | `'active' \| 'disabled'`;禁用站点级联使账号不参与签到/路由                                                                                                                                                                |
| `isPinned` / `sortOrder` / `globalWeight`                | UI 排序 + 路由全局权重                                                                                                                                                                                                    |
| `apiKey`                                                 | 站点级 API Key(纯 API 兼容站用)                                                                                                                                                                                           |
| `postRefreshProbeEnabled/Model/Scope/LatencyThresholdMs` | 模型刷新后自动拨测配置                                                                                                                                                                                                    |

### 站点分类方式:platform 适配器,而非站点类型标签

区分「官方站 / 聚合面板站 / OAuth 站」靠 `platform` 字段映射到适配器,**而不是站点上的类型枚举**:

- 聚合面板类(可登录、可签到、可查余额):`new-api`、`one-api`、`one-hub`、`done-hub`、`veloera`、`anyrouter`、`sub2api`
- 通用兼容/官方 API 类(只代理):`openai`、`claude`、`gemini`、`cliproxyapi`
- OAuth 连接类:`codex`、`gemini-cli`、`antigravity`

### 能力标记:推导而非存储(关键设计)

**没有任何 `supportsCheckin` 之类的静态布尔字段。** 能力分两层推导:

1. **平台层**:`PlatformAdapter` 接口(`src/server/services/platforms/base.ts:93-108`)规定能力面:`detect / login / verifyToken / checkin / getBalance / getModels / getApiTokens / createApiToken / getSiteAnnouncements / getUserGroups`。每个平台一个适配器类(`newApi.ts` 52KB 最完整;`openai.ts` 560B 只实现最小集,`checkin` 返回不支持)。
2. **账号层**:`AccountCapabilities { canCheckin, canRefreshBalance, proxyOnly }` 由**凭证模式**推导(`src/server/services/accountsOverviewService.ts:60-89`):
   - OAuth 账号 → 全 false,`proxyOnly: true`
   - 有 session token(网页登录态)→ `canCheckin = canRefreshBalance = true`
   - 仅 API Key → `proxyOnly: true`
   - 服务端算好后随 accounts snapshot 下发,前端 `Accounts.tsx` 只按 `capabilities.canCheckin` 渲染签到按钮,不自己判断。

运行期遇到「平台声称支持但站点实际 404」时,签到服务把它归为 `skipped` 并降级健康状态(见下文),即**能力以运行时反馈纠偏,不靠静态目录维护**。

### 配套表

- `site_api_endpoints`(schema.ts:29-45):一站多 API 端点池,带 `enabled/sortOrder/cooldownUntil/lastFailedAt/lastFailureReason` —— 端点级故障转移。
- `accounts`(schema.ts:57-86):`balance/balanceUsed/quota/unitCost/valueScore`、`status('active'|'disabled'|'expired')`、`checkinEnabled`、`lastCheckinAt`、OAuth 三元组、**`extraConfig`(JSON 杂物袋:runtimeHealth、自动重登录密码密文、platformUserId、账号级代理等)**。
- `account_tokens`:`source: 'manual' | 'sync' | 'legacy'` —— Token 可从上游站点同步。
- `checkin_logs`:`status('success'|'failed'|'skipped') + message + reward`。
- `model_availability` / `token_model_availability`:`available + latencyMs + checkedAt + isManual`(人工覆盖)。
- `route_channels`:`consecutiveFailCount / cooldownLevel / cooldownUntil` —— 通道熔断状态机。
- `site_announcements`、`events`、`site_day_usage / site_hour_usage / model_day_usage`(用量投影)。

## 站点目录数据来源与维护

`data/` 目录只是运行时数据目录(SQLite/日志,仓库里仅 `.gitkeep`),**没有 seed 数据文件**。站点数据有三个来源:

### 1. 首次启动 seed(默认站点)

`src/server/services/defaultSiteSeedService.ts`:硬编码 4 行 `sites` insert(OpenAI 官方 / Claude 官方 / Gemini 官方 / 本机 CLIProxyAPI),在一个事务里:

- settings 表存在 marker `default_site_seed_v1` → 跳过;
- 库里已有任何站点 → 只写 marker 不播种(老用户升级不被打扰);
- 否则插入 4 行 + 写 marker。**一次性、幂等、可被用户随意改删。**

### 2. 内置初始化预设(最接近「站点目录」的东西)

`src/shared/siteInitializationPresets.js`(纯 JS,316 行)+ 手写 `.d.ts`:**13 个官方厂商预设**,覆盖阿里云 CodingPlan、智谱 Coding Plan、DeepSeek、Moonshot(Kimi)、MiniMax、ModelScope、豆包 Coding Plan,各有 OpenAI 兼容/Claude 兼容两个入口。预设结构(`siteInitializationPresets.d.ts:15-26`):

```ts
type SiteInitializationPreset = {
  id: SiteInitializationPresetId; // 'deepseek-claude' 等字面量联合
  label: string; // '智谱 Coding Plan / Claude'
  providerLabel: string; // '智谱 Coding Plan'
  description: string; // 接入说明(一句话)
  platform: string; // 映射到哪个适配器
  defaultUrl?: string; // 默认 base URL
  initialSegment: "session" | "apikey"; // 初始凭证类型
  recommendedSkipModelFetch: boolean; // 不可枚举模型的站跳过拉取
  recommendedModels: string[]; // 推荐模型清单(替代模型发现)
  docsUrl?: string; // 官方文档链接
};
// + matches(url): 按 hostname+path 自动识别用户填的 URL 命中哪个预设
```

实现细节:`Object.freeze` 全部冻结,读取走 `clonePreset` 防外部篡改;`detectSiteInitializationPreset(url, platform)` 先精确 host+path 匹配,再退化到归一化 URL 等值比较。**预设随版本发布(代码即数据),不落库**;用户按预设创建的站点才落库。

### 3. 用户自建 + 平台自动探测

`POST /api/sites`,入参用 zod 契约校验(`src/server/contracts/siteRoutePayloads.ts`,可带 `initializationPresetId`)。`platform` 可省略,自动探测链(`src/server/services/platforms/index.ts:54-74`):

1. **URL hint**:`src/shared/platformIdentity.js` 按 hostname 硬规则(`api.openai.com → openai`、host 含 `anyrouter` → anyrouter 等);
2. **页面 title hint**:抓站点首页标题猜平台(`titleHint.ts`),anyrouter/done-hub 等分叉优先信 title;
3. **逐适配器 `detect(url)`** 真实探测 API 特征。

另有 `PLATFORM_ALIASES`(platformIdentity.js:1-34)把分叉站别名归一化:`'wong-gongyi' / 'vo-api' / 'super-api' / 'rix-api' / 'neo-api' → 'new-api'`,`'anthropic' → 'claude'` 等 —— **大量中转站其实是同一套面板的分叉,归一到适配器而不是逐站硬编码**。

**没有远程目录同步机制**;数据迁移靠 `backupService` 的全量导入导出(WebDAV 备份)。站点公告(`siteAnnouncements`)是从上游站点拉的元数据,按内容 sha1 作 `sourceKey` 去重。

## 健康监测与自动任务

### 健康模型(三层)

1. **账号 runtimeHealth 五态机**(`src/server/services/accountHealthService.ts:9`):`healthy / unhealthy / degraded / unknown / disabled`,带 `reason / source / checkedAt`,**存在 `accounts.extraConfig` JSON 里而非独立表**。`buildRuntimeHealthForAccount` 推导优先级:站点或账号 disabled > 凭证 expired(source=auth,按凭证模式给不同中文提示)> extraConfig 里存的最近事件 > 「仅 API Key 且模型探测成功」推断 healthy > unknown。写入方包括签到(`source: 'checkin'`)、余额刷新、模型发现等。
2. **通道熔断**:`route_channels.consecutiveFailCount / cooldownLevel / cooldownUntil`(代理请求失败递增冷却,默认 10 分钟)。
3. **端点池**:`site_api_endpoints.cooldownUntil / lastFailureReason`,请求时绕开冷却端点。

### 主动拨测

- **通道恢复探测** `channelRecoveryProbeService.ts`:每 30s 扫一遍冷却中的通道,用真实小请求(`probeRuntimeModel`)验证恢复,成功则提前解除冷却。刻意保守:**并发 = 1、每批最多 4 个、单次超时 12s、同 key 30s 内不重复**,注释明说「避免被上游当成批量健康检查」(channelRecoveryProbeService.ts:23);provider 主动下发的冷却(无失败计数)不探测。
- **模型可用性探测** `modelAvailabilityProbeService.ts`:按账号/Token 定时探测模型,结果四态 `supported/unsupported/inconclusive/skipped`,只有确定结果才写 `model_availability`(inconclusive 不动旧数据),支持人工覆盖 `isManual`。
- **post-refresh probe**:站点级配置(sites 表 4 个字段),模型列表刷新后立即拨测验证 + 延迟阈值。
- **「可用性监控」页面**(`routes/api/monitor.ts`)是个反例:它其实是 **iframe 反代第三方 LDOH 监控站**(`ldoh.105117.xyz`,社区维护的中转站测活服务),用户填 cookie,服务端代理并重写 HTML —— 公共站点测活外包给社区服务,自己只做账号资产层的监测。

### 签到与定时任务

**调度**(`src/server/services/checkinScheduler.ts`):

- 双模式:`cron`(node-cron,默认 `0 8 * * *`)或 `interval`(每 60s 轮询一次,按 `intervalHours`(1-24h)挑出到期账号)。
- 配置三级覆盖:settings 表(UI 可改)> 环境变量 > 默认值;改配置即热重启任务。
- interval 模式用内存 `Map<accountId, lastAttemptMs>` 记录尝试时间,失败也占用间隔,防重试风暴(`selectDueIntervalCheckinAccountIds`,scheduler.ts:78-102)。
- 同一调度器还管:余额刷新(默认每小时,刷完顺带重建路由)、每日摘要(23:58)、日志清理(06:00)。

**执行与错误处理**(`src/server/services/checkinService.ts`)—— 这是全仓库最值得抄的部分:

- `checkinAll`:按站点分组,**站内串行、站间并行**(checkinService.ts:347-362),避免打爆单站。
- 手动触发走 `backgroundTaskService` 后台任务,`dedupeKey: 'checkin-all'` 实现 README 说的「并发锁防重复签到」(`routes/api/checkin.ts:63-107`,返回 202 + reused 标记)。
- 结果归一为 `success / failed / skipped` 三态,消息分类器一大排:
  - **已签到判定** `isAlreadyCheckedInMessage`:匹配「今日已签到/已经签到/重复签到/签到过/already checked in」等中英文变体 → 视为成功(但 interval 模式不推进 `lastCheckinAt`);
  - **不支持签到** `isUnsupportedCheckinMessage`:404 on `/api/user/checkin` 等 → `skipped` + 健康降级 `degraded`(理由「站点不支持签到接口」),**不报错不告警**;
  - **Turnstile 人机校验** → `skipped` + 提示「需要人工签到」;
  - **Cloudflare challenge** → 失败 + 专门告警;
  - **token 过期** `shouldAttemptAutoRelogin` → 用 extraConfig 里加密存储的密码自动重登录(`tryAutoRelogin`)拿新 accessToken 后**重试一次**,顺带把 `expired` 账号复活为 `active`。
- **奖励解析**:`checkinRewardParser` 从返回消息提取金额;解析不到时签到后刷新余额、用**前后余额差推断奖励**(`inferRewardFromBalanceDelta`,精确到 6 位小数)。
- 每次签到写 `checkin_logs` + `events`,失败发通知(通知层有 300s 节流);成功顺带刷新余额、补写猜出的 platformUserId。

## 多端共享数据模型的做法

metapi 是 web / server / desktop 三个 tsconfig,但**共享数据模型的答案不是「三端共享类型包」,而是「单服务 + 多壳」+ 一个零构建共享层**:

### 1. 服务端是唯一数据所有者

- Drizzle `schema.ts` 是实体唯一定义;服务层类型直接 `typeof schema.sites.$inferSelect` 推导(如 `accountsOverviewService.ts:27`),没有第二份手写实体类型。
- Web(React)是纯 HTTP 客户端(`src/web/api.ts`),从不 import schema;能力、健康、统计都由服务端算好放进 snapshot 接口(15s TTL 缓存 + admin_snapshots 持久化),前端只渲染。
- API 边界用 zod 契约(`src/server/contracts/*RoutePayloads.ts`)校验,契约文件与路由解耦。

### 2. 桌面端 = Electron 壳进程,不复制任何业务

`src/desktop/main.ts:201-240`:Electron 主进程用 `spawn(process.execPath, [dist/server/index.js])`(`ELECTRON_RUN_AS_NODE=1`)把**同一个编译产物的 Fastify server** 起成子进程,轮询 health 端点 ready 后 `BrowserWindow.loadURL(serverUrl)` 加载**同一个 web UI**。还支持 `METAPI_DESKTOP_EXTERNAL_SERVER_URL` 直连远程后端。桌面端自己只有 ~6 个文件(托盘/自动更新/导航守卫/崩溃重启)。**所以根本不存在「桌面端如何同步站点模型」的问题 —— 桌面端没有模型。**

### 3. 真正双端共用的代码:`src/shared/` 纯 JS + 手写 .d.ts

需要同时在服务端(Node ESM 运行时)和前端(Vite 打包)使用的逻辑放 `src/shared/`,**全部是 `.js` 源码 + 手写 `.d.ts` + 同目录测试**:

| 文件                                              | 内容                       | 服务端消费者                             | 前端消费者                                          |
| ------------------------------------------------- | -------------------------- | ---------------------------------------- | --------------------------------------------------- |
| `siteInitializationPresets.js`                    | 13 个官方站预设 + URL 识别 | `routes/api/sites.ts`、`siteDetector.ts` | `Sites.tsx`、`Accounts.tsx`、`SiteCreatedModal.tsx` |
| `platformIdentity.js`                             | 平台别名归一 + URL hint    | `platforms/index.ts`(适配器注册表)       | 站点表单                                            |
| `sitePrimaryUrl.js`                               | URL 归一化/持久化形态      | 站点 CRUD                                | 同左                                                |
| `tokenRoutePatterns.js` / `tokenRouteContract.js` | 路由模式匹配规则           | tokenRouter                              | 路由页                                              |

为什么是 `.js`:`tsconfig.server.json` 的 `rootDir` 是 `src/server`、web 是 `noEmit` —— 共享层**不进任何编译管线**,Node 直接按相对路径 import 运行,Vite 直接打包,`.d.ts` 同时给两端类型。代价是这层不能写 TS,所以只放小而稳的纯函数与常量。

### 4. tsconfig 拆分只是检查/产物边界

根 `tsconfig.json`(bundler resolution,含全 src,paths 别名)给 IDE 和 vitest;`tsconfig.server.json`(NodeNext,只编 server)、`tsconfig.web.json`(noEmit 只查 web)、`tsconfig.desktop.json`(只编 desktop)。**tsconfig 不承担共享职责,共享靠运行时架构(单服务)和 `src/shared` 约定。**

仓库还有成文的工程纪律(`docs/plans/2026-03-23-single-source-consolidation-mega-plan.md`):「schema 元数据与兼容规则不得保留第二份手写事实源」,并配 architecture test 强制(如 `schemaMetadata.architecture.test.ts`)。

## 对 ccr 的启示

ccr 现状:签到站(Rust 硬编码 22 个)+ Provider 模板(前端 TS)两套独立目录。对照 metapi:

1. **站点目录应该是「预设 + 实例」两层,且预设是数据不是代码。** metapi 的 preset(冻结常量 + `matches(url)` 识别 + `recommendedModels`)与 site 实例(DB,用户资产)严格分离,首次启动 seed + marker 幂等。ccr 的 22 个签到站和 Provider 模板本质都是 preset 层 —— 应合并为一份目录,用户实际配置的账号/Key 是实例层。
2. **跨语言单一事实源的等价物是「纯数据文件」,不是共享代码。** metapi 能用 `.js + .d.ts` 是因为两端都是 JS;ccr 是 Rust + TS,等价做法是把统一站点目录做成一份 JSON/TOML(随仓库版本管理),Rust 端 `include_str!` + serde 编译期内嵌(替代 22 个硬编码),前端直接 import 同一份 JSON 或经 Tauri command 读取。ccr 的架构(Tauri,Rust 进程内)其实比 metapi 更接近理想形态:**让 Rust 核心做目录唯一所有者,前端模板清单退化为展示层**,正如 metapi 的 web 不持有任何站点数据。
3. **能力字段按「平台协议」推导,不逐站硬编码。** metapi 用 `platform` 枚举 + 适配器 + 别名表(`PLATFORM_ALIASES`)把几十种分叉站归到 ~8 个协议;账号能不能签到由凭证模式推导(有 session 才能签),运行时 404 自动降级为 skipped/degraded。ccr 的 22 个签到站大多是 new-api 系分叉,可归类为少数「签到协议」+ 站点目录里只存差异字段(URL、externalCheckinUrl、WAF 特征等)。
4. **统一实体可以参考的字段集**:preset 层 `id / label / providerLabel / description / platform / defaultUrl / initialSegment('apikey'|'session') / recommendedModels / docsUrl / matches`;站点层 `url / externalCheckinUrl / status / proxyUrl / customHeaders / sortOrder`;签到能力不放站点表,放协议(platform)定义。
5. **签到工程细节清单**(可直接对照 ccr 的签到模块):已签到消息中文变体判定、「不支持签到」降级为 skipped 不告警、Turnstile/Cloudflare 分类处理、凭证过期自动重登录再重试一次、奖励解析失败用余额差兜底、按站分组站内串行、`dedupeKey` 防并发重复触发、cron/interval 双调度模式 + 失败也占用间隔。
6. **健康状态用五态 + reason + source + checkedAt**,存在现有实体的 JSON 扩展字段里(metapi 放 `accounts.extraConfig.runtimeHealth`),不必新建表;恢复探测要刻意限流(并发 1、小批量),避免被上游 WAF 识别为扫描 —— 这与 ccr 最近的 WAF Cookie 工作直接相关。

## Caveats / Not Found

- metapi **没有**「公益站/商业站/官方站」分类字段,没有公共站点目录、没有目录远程同步 —— 想要的「内置 22 站目录」在 metapi 中最接近的对应物是 `siteInitializationPresets`(只覆盖官方厂商)+ 社区 LDOH 测活站(外部服务,iframe 反代接入)。
- 签到「并发锁」是进程内 background task dedupe(`dedupeKey: 'checkin-all'`),非跨进程锁;多实例部署下不防重。
- `tokenRouter.ts`(142KB)与 `newApi.ts`(52KB)未通读,本文对路由打分与 New API 适配器细节只取了 schema 与 README 层面的结论。
- metapi 的 `src/shared` 有少量例外(`updateCenterReminder.ts` 是 TS),说明该目录约定以「双端是否消费」为准,非绝对 `.js`。
