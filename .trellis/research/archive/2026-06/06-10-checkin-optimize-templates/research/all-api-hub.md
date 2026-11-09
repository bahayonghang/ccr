# Research: all-api-hub 多站点账号管理与刷新/签到机制

- **Query**: ref/repo/all-api-hub 如何管理多站点账号（数据模型、站点目录、余额刷新、签到、批量 UX、接入规范），供 ccr 签到管理页与 Provider 模板统一建模参考
- **Scope**: internal（只读外部参考仓库 `ref/repo/all-api-hub`）
- **Date**: 2026-06-10

> 路径均相对 `D:\Documents\Code\Github\ccr\ref\repo\all-api-hub\`。该仓库为外部镜像，仅作资料引用。

## 项目概览

- **定位**：浏览器扩展（WXT + React 19 + TypeScript + Tailwind），"一站式聚合管理所有 AI 中转站账号的余额、模型和密钥"。版本 3.45.0，支持 Chrome / Firefox / Safari（`package.json`）。
- **关键依赖**：`@plasmohq/storage`（扩展存储封装）、`@tanstack/react-query`、`@dnd-kit`（拖拽排序）、`react-virtuoso`（虚拟列表）、`echarts`（用量图表）、`i18next`（5 语言）。
- **代码组织**：`src/types`（领域类型）、`src/constants`（站点类型注册表等）、`src/services`（按域拆分：accounts / checkin / managedSites / siteDetection / integrations / webdav / history…）、`src/features`（按页面功能拆分 UI）、`src/entrypoints`（background / popup / options / sidepanel）。
- **流程资产**：`openspec/specs/` 下 50+ 个"每能力一个目录"的行为规范（spec-driven）；`docs/docs/` 下 VuePress 用户文档（auto-checkin / auto-refresh / supported-sites / ldoh-site-lookup 等）。
- **核心设计取向**：不维护"站点实例清单"，而是维护**站点软件类型（family）注册表 + 自动识别**；站点实例由用户添加账号时产生。实例级目录通过两个轻量外挂解决：赞助商目录 JSON（远程可更新）与 LDOH 社区站点索引（外部 API）。

## 站点与账号数据模型

### 核心实体 `SiteAccount`（`src/types/index.ts:73-187`）

| 字段 | 说明 |
|---|---|
| `id` | 账号条目 id |
| `site_name` / `site_url` | 站点名称 / URL（站点元数据直接内嵌在账号上，没有独立"站点实体"） |
| `site_type: AccountSiteType` | 站点**软件类型**（new-api / one-api / Veloera / anyrouter / sub2api / AIHubMix / unknown…），见下节 |
| `health: HealthStatus` | 4 态健康：`healthy/warning/error/unknown` + `reason` 文本 + 机器可读 `code`（UI 据此提供可操作跳转，`index.ts:12-32`） |
| `exchange_rate` | 人民币/美元充值比例（CNY per USD），用于双币种余额换算 |
| `account_info: AccountInfo` | `id`（站点内稳定用户标识）、`access_token`、`username`、`quota`、今日 prompt/completion tokens、今日消耗、今日请求数、今日收入（充值+签到）（`index.ts:37-47`） |
| `authType: AuthTypeEnum` | `access_token` / `cookie` / `none`（`index.ts:536-540`） |
| `cookieAuth.sessionCookie` | 账号级 Cookie 串（多账号隔离，请求时与 WAF cookie 合并） |
| `sub2apiAuth` | Sub2API 的 refreshToken + tokenExpiresAt（长效凭据，注释明确声明会进入导出/WebDAV 备份） |
| `checkIn: CheckInConfig` | 签到配置与状态（见下） |
| `notes` / `tagIds` | 备注 + 全局标签 id 引用（标签实体存独立 TagStore，支持全局改名/删除，`index.ts:362-375`） |
| `disabled` | 禁用：不参与后台刷新/自动签到/聚合统计，仅可重新启用 |
| `excludeFromTotalBalance` / `excludeFromTodayIncome` | 仅从聚合数字中剔除，不影响刷新/签到行为 |
| `manualBalanceUsd` | 手动覆盖余额（无法自动抓取的站点） |
| `last_sync_time` / `updated_at` / `user_updated_at` / `created_at` | 同步时间 / 系统改动 / **用户意图改动**（区分用于 WebDAV 合并） |
| `configVersion` | 配置迁移版本号，注释中列出 0→6 的演进史；`src/services/accounts/migrations/` 下每个字段一个迁移脚本 |

### 签到配置 `CheckInConfig`（`src/types/index.ts:189-285`）— 双轨设计

- `enableDetection`：签到能力总开关（检测支持 + 刷新时查状态 + 显示 UI）。
- `autoCheckInEnabled`：账号级自动签到开关（默认 true）。
- `siteStatus`：**站点 API 签到轨**状态：`isCheckedInToday`（true/false/undefined 三态）、`lastCheckInDate`（YYYY-MM-DD，用于每日重置）、`lastDetectedAt`（状态检测时间戳，UI 用于"状态可能过期"提示）。
- `customCheckIn`：**自定义 URL 签到轨**（魔改站点用）：`url`、`redeemUrl`（自定义充值页）、`openRedeemWithCheckIn`（签到时顺带开充值页）、`turnstilePreTrigger`（Cloudflare Turnstile 预触发配置）、独立的 `isCheckedInToday`/`lastCheckInDate`。两轨状态互不干扰。

### 凭据存储

- 全部账号存为单个 `AccountStorageConfig` blob：`accounts + bookmarks + pinnedAccountIds + orderedAccountIds + deletedEntryRecords(删除墓碑，防 WebDAV 合并复活) + last_updated`（`index.ts:332-354`）。
- 存储后端：`@plasmohq/storage`，`area: "local"`（`src/services/accounts/accountStorage.ts:79-80`），**明文**保存 access_token / cookie / refreshToken。
- 并发安全：集中式锁注册表 `STORAGE_LOCKS`（`src/services/core/storageKeys.ts`）+ `withStorageWriteLock` 包裹所有读-改-写序列；所有 storage key 集中注册便于审计。

### 分类字段现状

没有"公益站 vs 商业站"的显式枚举。分类靠三层：`site_type`（软件类型，其中 `wong-gongyi` 这种公益站直接作为独立类型存在）、用户自定义 `tagIds`、`notes`。赞助商目录条目另有 `supportStatus: supported/unsupported`。

### UI 投影 `DisplaySiteData`（`index.ts:419-495`）

持久层与展示层分离：余额/今日消耗/今日收入均投影为 `CurrencyAmount {USD, CNY}` 双币种对象（用 `exchange_rate` 换算），并把 `disabled`/`excludeFrom*`/`checkIn` 等行为标记一并投影，UI 不需要回查存储。

## 站点目录维护方式

**三套互补机制，没有任何一处硬编码"站点实例列表"：**

### 1. 站点软件类型注册表（内置，TS 常量）— `src/constants/siteType.ts`

- `SITE_TYPES` 共 18 种；`ACCOUNT_SITE_TYPES`（可添加账号的 15 种）与 `MANAGED_SITE_TYPES`（可作为自建后台管理的 6 种：new-api / Veloera / done-hub / octopus / axonhub / claude-code-hub）是两个不同子集——**同一注册表服务两套功能**，这正对应 ccr 的"签到站点 vs Provider 模板"双目录问题。
- 每类型的路由配置 `SITE_ROUTE_CONFIGS`（`siteType.ts:147-208`）：`loginPath / usagePath / checkInPath / redeemPath / adminCredentialsPath / siteAnnouncementsPath`，以 `Default` 兜底 + 类型级 override（lodash merge）。**签到入口与充值入口是类型级元数据而非站点实例数据**。
- 类型识别规则同文件内声明：标题正则 `ACCOUNT_SITE_TITLE_RULES`（自动生成 `\bnew[-_ ]?api\b` 这类正则）+ 域名规则 `ACCOUNT_SITE_DOMAIN_RULES`（如 AIHubMix 按域名优先识别）。

### 2. 自动识别（用户自定义站点的加入方式）— `src/services/siteDetection/detectSiteType.ts:160-176`

三级 fallback：域名规则 → 抓站点根路径**原始 HTML title**（优先经 temp-window 真实浏览器上下文穿 WAF，失败再直接 fetch）→ 探测 `/api/user/self` 用错误消息/兼容头指纹判型。识别失败落 `unknown`（仍可用默认路径集工作）。用户添加账号 = 输入 URL + 自动识别 + 自动抓 token/用户信息（`features/AccountManagement/components/AccountDialog/useAccountDialog.ts`，91.5K 的向导逻辑）。

### 3. 可远程更新的实例目录（两个）

- **赞助商目录**：`public/sponsor-catalog.json`（`schemaVersion: 3`）。加载顺序：远程缓存 → bundled 兜底（`features/AccountManagement/sponsors/loader.ts`）；远程源是 GitHub raw URL（`sponsors/constants.ts:5-6`），拉取后**严格 normalize 校验**（`sponsors/catalog.ts`：schemaVersion 不符整体拒绝、逐条校验 enabled/supportStatus/日期窗口/URL 协议安全性、locale 回退链 当前语言→zh-CN→en）后才写缓存。条目结构很值得抄：`id/rank/supportStatus/urls{primaryAffiliate,website,apiKeyCreate}/locales{...}/accountPrefill{siteType,siteUrl,authType}/fallbackHints`——`accountPrefill` 支持"点击推荐站 → 一键预填添加账号表单"。
- **LDOH 社区站点索引**：`src/services/integrations/ldohSiteLookup/`。外部社区维护的中转站聚合 API（`https://ldoh.105117.xyz/api/sites`），本地缓存 12h TTL（`constants.ts`），仅取 `id/name/apiBaseUrl` 三字段（`types.ts` 注释明确"其余字段有意忽略"），按账号 origin 匹配成功后在账号行显示"查看 Linux.do 社区口碑"入口。扩展自身不背站点目录维护成本，外包给社区。

### 接入规范文档

- `openspec/specs/` 每能力一个 `spec.md`，格式：`Purpose / Definitions / Current Implementation Notes (Informative) / ADDED Requirements`，需求用 RFC2119 MUST + `#### Scenario:` 场景块（看过 `ldoh-site-lookup/spec.md`；`auto-checkin/spec.md` 有 31K 的完整行为契约）。**没有"新站点接入"的通用 spec**——因为接入的单位是"站点类型适配器"（apiService + checkin provider + 路由配置），而不是站点实例。
- `docs/docs/supported-sites.md` 人工维护"支持的站点类型"列表（含官方链接、兼容性备注、推荐赞助站）。

## 余额刷新与并发策略

### 单账号刷新 `refreshAccount`（`src/services/accounts/accountStorage.ts:1024-1286`）

流程：禁用检查 → 站点元数据刷新 → **最小间隔节流** → 顺带刷新签到支持状态（`fetchSupportCheckIn`）→ 按 `site_type` 取对应 apiService 刷新账号数据 → 更新健康状态 + 抓每日余额快照（balance history）→ 持久化（保留用户时间戳）。

- **节流**（`accountStorage.ts:1919-1932`）：非 force 时，`Date.now() - last_sync_time < minInterval*1000` 则跳过。`minInterval` 用户可配，最低 30s（`docs/docs/auto-refresh.md`）。force（手动"立即刷新"）绕过。
- **失败处理**：异常时健康状态写 `unknown` + reason，不抛出中断全局（`1265-1285`）；健康状态变化打日志。**刷新本身没有自动重试**，失败靠健康徽章 + 原因展示，等下个周期。
- **特例串行**：Sub2API 账号刷新按账号粒度加写锁串行（refreshToken 轮换不能并发，`1250-1262`）。

### 批量刷新 `refreshAllAccounts`（`1291-1333`）

`Promise.allSettled` 全量并发（**无全局并发上限**），单账号失败隔离，汇总 `{success, failed, latestSyncTime, refreshedCount}`。另有 `refreshDisabledAccounts`（`1339-1388`）：重新探测禁用账号，成功则自动重新启用。

### 后台定时 `AutoRefreshService`（`src/services/accounts/autoRefreshService.ts`）

- 单例 + 单 `setInterval`（防重复定时器），间隔来自用户偏好（UI 拦截 <60s）。
- 后台周期刷新走非 force（受 minInterval 保护）；完成后 runtime message `AUTO_REFRESH_UPDATE` 通知前端，popup 未打开则静默吞掉"无接收者"错误。
- 可选"打开弹窗即刷新"（非 force）——配合 minInterval 实现"打开就新鲜但不轰炸站点"。

### 限流/并发控制的三种局部策略

1. **签到后补刷余额**：固定 `batchSize = 3` 分批 + 批内 `Promise.allSettled`（`src/services/checkin/autoCheckin/scheduler.ts:168-207`），避免大量签到成功后流量尖峰。
2. **per-origin 串行队列** `runPerKeySequential`（`src/services/accounts/accountKeyAutoProvisioning/perOriginQueue.ts`）：同 key（站点 origin）内严格串行、不同 key 并行——同站多账号防限流的标准解法，用于批量密钥操作。
3. **WAF 穿透**：Cloudflare 盾站点经 temp-window（临时真实浏览器窗口）fetch，配合 Turnstile 预触发配置。

## 签到与批量操作 UX

### Provider 体系（判定逻辑）

- 注册表按 `site_type` 选 provider（`src/services/checkin/autoCheckin/providers/index.ts:24-29`）：anyrouter / Veloera / wong-gongyi / new-api 四个内置。
- 统一契约：`canCheckIn(account)`（快速资格判断）+ `checkIn(account) → AutoCheckinProviderResult`，结果为 **4 态**：`success / already_checked / failed / skipped`，附 `messageKey`（i18n 键）+ `rawMessage`（保留后端原话）（`providers/types.ts`、`types/autoCheckin.ts:15-25`）。
- 结果判定示例（`providers/anyrouter.ts`）：POST `/api/user/sign_in`（cookie 认证）→ 看 `response.success`；message 含 "success"/"签到成功" → SUCCESS；**空 message 视为已签到**（AnyRouter 特性）+ 共享启发式 `isAlreadyCheckedMessage`；异常统一走 `resolveProviderErrorResult`。
- **关键原则**（`docs/docs/auto-checkin.md`）：调度器**不信任**本地 `isCheckedInToday` 标记决定是否执行（"该字段不可信"）；以 provider 返回的 `already_checked` 作为"已签到"的真实来源。本地标记只用于 UI 展示，且带 `lastDetectedAt` 过期提醒。

### 自动签到调度（`scheduler.ts`，93.5K）

- `chrome.alarms` **双闹钟**：daily 闹钟（每自然日最多一次；时间窗内**随机时刻**或 deterministic 固定时刻，窗口可跨夜如 22:00→06:00）+ retry 闹钟（只重试当日失败账号，`intervalMinutes` 间隔、`maxAttemptsPerDay` 上限，`types/autoCheckin.ts:303-330`）。重试永不挤占下一次每日计划。
- 闹钟带"目标日"防护（`dailyAlarmTargetDay`），触发时与今天比对，过期闹钟不执行。
- `pretriggerDailyOnUiOpen`：打开 popup/侧栏/设置时，若在窗口内且今日未跑，提前触发当日 run，跑完弹结果汇总对话框。
- 执行：过滤资格（6 种跳过原因枚举：account_disabled / detection_disabled / auto_checkin_disabled / already_checked_today / no_provider / provider_not_ready，均有 i18n）→ **并发 `Promise.all` 跑 providers**（`scheduler.ts:2042-2049`）→ 汇总 `AutoCheckinRunSummary {totalEligible, executed, successCount, failedCount, skippedCount, needsRetry}` → 成功/已签写 `markAccountAsSiteCheckedIn` → 成功账号分批(3)补刷余额 → 持久化 `AutoCheckinStatus`（lastRunAt、perAccount 结果、账号快照、nextDailyScheduledAt、nextRetryScheduledAt）→ 广播 `AutoCheckinRunCompleted` 让打开的 UI 局部刷新。
- 手动 `runNow` 支持传 `accountIds` 子集（单账号/选中账号签到与全量复用同一管线）。
- 专门的「自动签到」状态页：上次结果（成功/部分成功/失败三态）、下次每日计划、下次重试计划、账号维度日志（耗时、失败原因）。

### 账号列表 UX（`src/features/AccountManagement/components/AccountList/`）

- **批量模式**：显式 bulk mode 开关 → 多选 `selectedAccountIds`，区分"可见选中/被过滤隐藏的选中"计数；批量禁用（只作用于已启用账号）、批量删除（确认对话框）；操作完成后把已处理 id 从选中集剔除、剩余保留（`index.tsx:726-852`）。
- **组织手段**：拖拽排序（@dnd-kit）+ 置顶 pinned 列表 + 标签筛选 + 搜索 + 多字段排序（名称/今日消耗/今日收入/余额/创建时间，升降序）。
- **状态展示**：健康徽章 4 态 + reason + 可操作 code（如一键跳设置）；禁用账号灰显但保留"重新启用"；`excludeFrom*` 只影响聚合数字。
- **性能**：账号列表**未用虚拟列表也无分页**（账号量级几十个，靠 memo 化选择器）；`react-virtuoso` 用在模型列表（`features/ModelList/`，数据量大）；加载骨架 `AccountListLoadingState`；`react-countup` 余额数字动画；toast 用 react-hot-toast。
- 批量操作均埋点（成功/失败计数进 product analytics）。

## 对 ccr 的启示

1. **类型与实例分离，统一两套目录**。all-api-hub 用一张"站点软件类型表"（适配器粒度：API 路径、签到端点、充值路径、认证方式）同时支撑"账号管理"和"自建后台管理"两个功能域（`ACCOUNT_SITE_TYPES` vs `MANAGED_SITE_TYPES` 同表两个子集）。ccr 的 22 个硬编码签到站点与 Provider 模板可统一为：**类型层**（Rust 适配器：new-api 系/独立 API 系，含 `checkInPath/redeemPath/usagePath` 等路由元数据）+ **实例层**（数据文件：站点名/域名/类型/分类/签到能力/充值方式/推荐排序），签到页与 Provider 模板各取实例层不同投影。
2. **实例目录用"bundled JSON + 远程拉取 + 本地缓存 + schemaVersion 严格校验"**（sponsor-catalog 模式）：内置兜底保证离线可用；远程 raw URL 更新无需发版；校验失败整体拒绝防脏数据；条目带 `accountPrefill`（一键预填添加账号）与 `supportStatus`。ccr 的 Tauri 端完全可复制此模式（22 站点从 Rust 常量迁到 JSON 资源文件）。
3. **签到结果统一 4 态契约**：`success / already_checked / failed / skipped` + 6 种跳过原因枚举 + i18n messageKey + 保留 rawMessage。判定上**不信任本地"今日已签"缓存**，以服务端返回 `already_checked` 为准；本地状态只做展示并带 `lastDetectedAt` 过期提示、`lastCheckInDate`(YYYY-MM-DD) 每日重置。ccr 的批量签到结果汇总可直接采用 `{totalEligible, executed, success, failed, skipped, needsRetry}` 结构。
4. **刷新策略三件套**：force/非 force 两档 + per-account `minInterval` 节流（最低 30s）+ `allSettled` 失败隔离；失败不重试而是落 4 态健康状态（reason + 可操作 code）驱动 UI。**同 origin 串行、不同 origin 并行**的 per-key 队列（`runPerKeySequential`，30 行实现）正是 ccr"同站多账号批量签到防限流"需要的并发原语；签到成功后小批量（3 个一批）补刷余额避免尖峰。
5. **重试设计**：每日主调度与重试调度分离（重试只针对当日失败子集，有 `maxAttemptsPerDay` 上限，不影响明日计划）；调度带"目标日"防陈旧任务。ccr 桌面端可用 tokio 定时任务对应 chrome.alarms 双闹钟模型。
6. **批量 UX 细节**：显式批量模式 + 可见/隐藏选中计数（有过滤时不误伤）、批量操作后只剔除已处理项、禁用（软删，禁参与后台任务+聚合）与"仅从汇总剔除"是两个独立开关、执行记录页（上次结果/下次计划/账号级日志）。
7. **可演进的持久化**：账号记录带 `configVersion` + 逐字段迁移脚本目录 + 删除墓碑（防同步复活）+ `user_updated_at` 与 `updated_at` 分离，ccr 的签到账号配置如要支持 WebDAV/多端同步，这套是现成蓝本。

## Caveats / Not Found

- `refreshAllAccounts` 全量并发**无全局上限**，是有意取舍（账号量小 + 单账号有 minInterval 节流）；ccr 若站点/账号更多需自己加全局并发上限。
- 凭据为**明文**存扩展 local storage（浏览器扩展环境限制），ccr 桌面端不应照搬，应继续走现有 secret masking/keyring 约定。
- 没有找到"公益站/商业站"显式分类字段，也没有"新站点实例接入"的通用 openspec 规范（接入单位是类型适配器）。
- 账号列表无虚拟化/分页；虚拟列表仅用于模型列表，对 ccr 签到页（22 站点 × 多账号）参考意义：百级以下不必虚拟化。
- `scheduler.ts`（93.5K）与 `useAccountDialog.ts`（91.5K）仅做了重点段落阅读，细节（如 Turnstile 全流程、newApi provider 17.6K 的完整判定分支）未逐行核对。
