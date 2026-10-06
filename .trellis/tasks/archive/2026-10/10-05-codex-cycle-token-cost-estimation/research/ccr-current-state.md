# Research: CCR Codex Auth 用量、配额和 API 等价费用现状

- Query: 核查截图中的 Codex Auth TUI 用量来源、账户归属、去重、token 分类、价格、配额历史，以及最小复用边界。
- Scope: internal；外部资料由主会话另行核实，本文不探测真实账户。
- Date: 2026-10-05
- Repository: `D:/Documents/Code/Github/ccr`
- HEAD: `6a2c87eeb4e17c9e317b2cb738e73aeb15999489`，通过 `.git/HEAD` 和 `.git/refs/heads/dev` 只读取得。
- Status: `READ_ONLY_RESEARCH`。未实施，未执行测试，未读取个人 auth、session 或 SQLite 数据库。

## Findings

### 1. 截图已经显示 token，但数字来自独立的简化解析器

截图的 `Local usage · Tokens / Requests` 通过 `CodexAuthApp::load_usage_data` 调用 `CodexUsageService::parse_all_logs`，再构造全局及按账号筛选的滚动数据。该路径未调用 `ccr-usage` 或 `llmusage`。

- 加载入口：`crates/ccr-tui/src/tui/codex_auth/app.rs:287-305`。
- 本地解析 DTO 只有 `session_id`、`timestamp`、`input_tokens`、`output_tokens`、`model`：`crates/ccr-codex/src/services/codex_usage_service.rs:25-36`。
- 界面 `Tokens` 明确显示 `total_input_tokens + total_output_tokens`：`crates/ccr-tui/src/tui/codex_auth/ui.rs:1045-1075`。
- `CodexUsageStats::total_tokens()` 同样只有 input + output：`crates/ccr-codex/src/services/codex_usage_service.rs:53-57`。
- `Requests` 是每条正增量用量记录加 1：`crates/ccr-codex/src/services/codex_usage_service.rs:468-471`。现有 DTO 没有 request/turn identity，无法证明这些记录各自对应一个真实 API 请求。
- `Top model` 使用全部保留记录的按模型统计，不限定 5h/7d：`crates/ccr-codex/src/services/codex_usage_service.rs:494-500`；`crates/ccr-tui/src/tui/codex_auth/app.rs:659-667`。

当前需求应扩展细分、费用、周期语义和质量说明。界面已有 token 总量，不应将任务描述为从零增加 token 统计。

实施文案建议保留当前数字，将列标题改为 `Usage records / 用量记录`。如上游新格式提供可用 response/request ID，优先利用该 ID 去重；旧格式采用经测试的累计高水位与 ordinal 兼容合同，仍不宣称真实 HTTP 请求数。

### 2. 5h/7d 的定义与官方 reset 彼此独立

当前 5h 为记录时间 `>= now - 5 hours`，7d 为 `>= now - 7 days`；窗口终点 DTO 写入 `now`。解析器不读取官方配额 reset，也不依赖 plan。未来时间的记录没有 `<= now` 上界过滤，当前实现可能将这类记录计入滚动窗口。

- 窗口计算：`crates/ccr-codex/src/services/codex_usage_service.rs:474-511`。
- 对外计算使用实际 `Utc::now()`：`crates/ccr-codex/src/services/codex_usage_service.rs:210-211`。
- 官方 DTO 独立提供百分比、reset 和窗口分钟数：`crates/ccr-codex/src/models/codex_auth.rs:778-806`。

设计建议：保留历史 5h/7d 滚动统计；单独显示服务端窗口及 reset 信息。服务端 `window duration` 与 `reset_at` 不自动证明服务端采用固定周期。只有明确标注为推导区间时，才可将 `[reset_at - duration, reset_at)` 用作本地统计辅助范围。百分比采样间的校准使用真实采样时间范围，遇到 reset/bucket/plan 改变后重新分段。

### 3. 账户归属来自激活时间区间，属于本地推断

`CodexAuthRegistry.usage_ledger` 存储 `account_name`、稳定匹配键 `account_id` 和 `started_at`；没有结束事件、请求 ID、session ID、source、auth mode 或 limit bucket。

- 结构及持久化位置：`crates/ccr-codex/src/models/codex_auth.rs:272-307`。
- 连续相同账号激活去重：`crates/ccr-codex/src/models/codex_auth.rs:329-348`。
- 保存当前登录时追加：`crates/ccr-codex/src/services/codex_auth_service.rs:924-933`。
- 切换账号时追加：`crates/ccr-codex/src/services/codex_auth_service.rs:1163-1176`。
- 检测到当前 runtime 指针变化时追加检测时刻：`crates/ccr-codex/src/services/codex_auth_service.rs:709-734`。外部改变发生时间与 CCR 检测时间可能不同。
- 账号重命名更新 `account_name`：`crates/ccr-codex/src/services/codex_auth_service.rs:1528-1535`。
- 归属算法把记录分配给记录时间之前最近的激活项：`crates/ccr-tui/src/tui/codex_auth/app.rs:671-703`。

该算法可在合成数据中验证时间区间匹配。算法无法证明并行运行的旧会话在切换后已使用新凭据，也无法完整记录 CCR 进程外的账号切换。首条激活之前的记录不归属任何账号；最后一条激活之后全部匹配记录持续使用最后一条账号标签。

现有账号成功分支保留 `Excludes other accounts and unattributed records`，符合当前规格。没有匹配记录时显示全局本地用量并保留原因；不能把全局用量填入选中账号统计。

- 成功与回退：`crates/ccr-tui/src/tui/codex_auth/app.rs:571-655`。
- 对应显示：`crates/ccr-tui/src/tui/codex_auth/ui.rs:1019-1039`。
- 规格明确本地激活归属不构成官方账单或 quota 转换：`.trellis/spec/ccr-tui/backend/backend-guidelines.md:420-432`。

设计建议：兼容保留 legacy 归属，新增 `activation_interval_inferred` 质量标签。若基于该数据做经验比例估算，置信度必须受限，说明其他设备、云任务和外部账号切换不可见。未知 provider/auth mode/bucket 匹配时输出 `N/A`，不要假定等价账户覆盖完整。

### 4. 当前日志解析和 provider 元数据不提供账户正证据

`CodexUsageService` 的 session metadata 仅解析 session ID、model、created_at 和 cwd。当前产品未解析日志 account ID、auth mode、provider、limit bucket 或 request identity。

- metadata DTO：`crates/ccr-codex/src/services/codex_usage_service.rs:85-91`。
- metadata 解析：`crates/ccr-codex/src/services/codex_usage_service.rs:356-392`。
- 用量 DTO 无上述字段：`crates/ccr-codex/src/services/codex_usage_service.rs:25-36`。

以上只能证明 CCR 当前未消费这些字段，不能证明所有上游日志版本都缺少这些字段。本次不读取个人日志来建立此结论。

另一个重要边界：CCR 的历史可见性修复会将 `openai/custom/缺失 provider` 历史桥接为当前 provider，并执行首行 provider 改写。因此 rollout 的 `model_provider` 可能是可变的历史可见性元数据，不应单独作为原始 API 计费来源或账号正证据。

- 默认 bridge 行为：`crates/ccr-codex/src/services/codex_history_sync_service.rs:481-511`。
- 首行改写入口：`crates/ccr-codex/src/services/codex_history_sync_service.rs:2692`。
- 同一功能的 fixture 明确检查 provider 改写：`crates/ccr-codex/src/services/codex_history_sync_service.rs:3377`。

### 5. 现有去重覆盖范围有限，不能直接作为精确计费事件序列

当前 `token_count` 只读取 `info.total_token_usage`，否则读取 payload 的顶层字段。当前实现不读取 `last_token_usage`。input/output 以累积值差分，更新 previous totals，并仅生成正增量记录。

- 累积差分：`crates/ccr-codex/src/services/codex_usage_service.rs:294-316`。
- 字段路径：`crates/ccr-codex/src/services/codex_usage_service.rs:408-423`。
- `turn.completed` 直接生成记录：`crates/ccr-codex/src/services/codex_usage_service.rs:323-343`。
- 只要文件中有任意 `turn.completed`，整份文件只保留 completed 记录，放弃所有 token_count 记录：`crates/ccr-codex/src/services/codex_usage_service.rs:348-352`。
- 对应测试只覆盖 completed 优先的基础例子：`crates/ccr-codex/src/services/codex_usage_service.rs:708`。

静态推论及待验证案例：

1. 累积值重复上报产生零增量，不重复生成记录。
2. 累积计数回退使用 `saturating_sub` 得到零，再降低 previous baseline；后续恢复旧值可能重新计算已有 token。压缩、重置、恢复的 epoch 规则尚未建立。
3. 同一 session 的日志副本会分别解析再全部追加；没有跨文件 `(session, event/turn)` 级去重。`parse_all_logs` 仅按时间排序。
4. 混合日志中只有部分 turns 有 completed 时，全文件优先策略可能丢弃其余 token_count 事件。不能把 completed 和 cumulative/last 全部相加解决该问题。
5. 未携带 timestamp 的记录回退到 session created_at；两者都没有时使用 `Utc::now()`。窗口归属可能不反映调用时刻。
6. 无法读取/解析单文件时静默跳过；读取行失败及无效 JSON 行也静默跳过。没有对外 coverage diagnostics。

对应证据：`crates/ccr-codex/src/services/codex_usage_service.rs:166-206`、`:230-242`、`:270-277`。

设计建议：保留原始来源/计数 epoch/确定性 identity/coverage 状态。累计与 last 信息在同一 turn 上选择一个权威计量来源。增加混合格式、重复事件、日志副本、回退、恢复、缺失时间、model 切换的合成 fixture。费用仍以明确的可计价记录作为输入。

### 6. cached、cache-write、reasoning 和价格目前均未进入 Auth 用量 DTO

目前 Auth 用量 DTO 不保留 `cached_input_tokens`、`cache_write_input_tokens`、`reasoning_output_tokens`、上游 `total_tokens`、service tier、token completeness 或价格来源。现有 `turn.completed` fixture 含 `cached_input_tokens`，但生产读取只取 input/output，测试不验证 cache 分类。

- 缺失字段：`crates/ccr-codex/src/services/codex_usage_service.rs:25-36`、`:94-97`、`:327-334`、`:414-423`。
- 带 cache 的 fixture：`crates/ccr-codex/src/services/codex_usage_service.rs:668`。

当前 `total_tokens=input+output` 没有再额外加 cache/reasoning，现有代码没有因为独立加这两项导致重复总量。扩展必须明确上游是否为 inclusive totals。不能把 inclusive input/output 直接相加到独立 cache/reasoning 桶。

建议内部统一成互斥分项，并另外保留原始 inclusive totals：

- 非缓存输入、缓存读取、缓存写入各为独立输入桶。
- 可见输出、推理输出为独立输出桶。
- total 只合计互斥桶，或校验原始 inclusive total；不再把 cache/reasoning 加到已含这些 token 的总量。
- 先验证上游 `cache_write_input_tokens` 包含关系，再执行非缓存输入扣减。未知字段与 0 必须区分。
- 计算费用时 output 参数使用全部计费输出，reasoning 不额外重复计费。

### 7. 共享 llmusage 投影已经有分类和成本，但归属维度不同

`ccr-usage` 只读已安装 llmusage 的 SQLite 投影；不解析原始 provider 日志，不创建/迁移 DB，不写 CCR state。Auth 页面简化 JSONL 解析与该投影是两条不同数据路径。

- crate 职责：`crates/ccr-usage/src/lib.rs:1-5`。
- `TokenSummary` 含 input、cache_read、output、reasoning、total：`crates/ccr-usage/src/queries.rs:9-28`。
- `ModelBreakdown` 还有 cache_creation、cache-aware/cache-free USD、pricing status/source/rate：`crates/ccr-usage/src/queries.rs:77-91`。
- `ProviderBreakdownDto` 含 cache_creation 和两种成本：`crates/ccr-usage/src/queries.rs:119-137`。
- SQL 直接 sum 上游互斥分项、total 和成本，不在 CCR 重新定价：`crates/ccr-usage/src/db.rs:394-441`。
- provider SQL 按 `provider_label` 分组：`crates/ccr-usage/src/db.rs:447-486`。
- logs API 的 `output_tokens` 已合并 reasoning：`crates/ccr-usage/src/db.rs:703-706`。消费者不得再次加 reasoning。

现有 `QueryFilter` 只有 source/model/provider/日期/project/timezone，没有 Auth account 或精确时分秒范围。按日期过滤转换为本地日的 UTC 起止；bucket API 查询 `usage_bucket_30m`。该 API 不足以精确筛选账户激活片段或官方样本区间。

- 过滤 DTO：`crates/ccr-usage/src/db.rs:31-38`。
- 日期范围：`crates/ccr-usage/src/db.rs:120-132`。
- bucket/event 过滤的时间列：`crates/ccr-usage/src/db.rs:80-96`。

`provider_activation.jsonl` 来自 profile 激活，和 Codex Auth 的 `usage_ledger` 分离。provider event 的 `account` 字段属于 profile/provider 元数据，不等价于 CodexAuthRegistry 的 OpenAI account_id。

- profile 激活日志职责：`crates/ccr-config/src/managers/provider_activation.rs:1-10`。
- provider 激活 DTO：`crates/ccr-config/src/managers/provider_activation.rs:42-58`。
- Auth 激活 DTO：`crates/ccr-codex/src/models/codex_auth.rs:277-285`。

复用建议：Auth 的本地成本扩展由 `ccr-codex` 负责；模型 rate 复用 `ccr-types`。保持现有 llmusage 费用的 pass-through。不要将 provider 分组的汇总费用分配给 Auth 账号，不要把两条路径相加。若未来读取 llmusage 新的精确事件投影，SQL 仍只由 `ccr-usage` 拥有，且应另行定义账户与来源合同。

### 8. 当前 quota 没有持久化历史，也缺少 bucket 身份

HTTP typed DTO 只含 plan_type、rate_limit 和被忽略的 code_review_rate_limit。窗口含 `used_percent`、duration seconds、reset_after、reset_at。未定义 limit_id/name、normal_model_slug 或 additional/model limits。

- HTTP DTO：`crates/ccr-codex/src/services/openai_quota_core.rs:32-59`。
- endpoint 是 `https://chatgpt.com/backend-api/wham/usage`：`crates/ccr-codex/src/services/openai_quota_core.rs:22`。
- 转换保留窗口分钟数，百分比换成 `100-used`：`crates/ccr-codex/src/services/openai_quota_core.rs:432-505`。
- raw_data 保留原始 JSON 于单次 DTO：`crates/ccr-codex/src/services/openai_quota_core.rs:462-478`；该行为不建立历史数据库。
- DTO 只带 account_name/email/quota/error/fetched_at，没有稳定 account_id：`crates/ccr-codex/src/models/codex_auth.rs:811-824`。

官方 quota 只有进程内 30 秒共享 HashMap 和 TUI preview/cache。没有 quota-history repository 或周期样本持久化。

- 进程内共享缓存及 TTL：`crates/ccr-codex/src/services/openai_quota_core.rs:18-30`、`:110-114`、`:519-543`。
- TUI cache 和初始化：`crates/ccr-tui/src/tui/codex_auth/app.rs:170-181`、`:251-256`。
- quota service 写入路径用于更新 OAuth auth 文件：`crates/ccr-codex/src/services/codex_quota_service.rs:386-416`；没有保存 quota 观测。

另有采样质量问题：cache outcome 没有真实 network acquisition timestamp。命中共享缓存直接返回 outcome，但 CodexQuotaService 在调用入口设置新的 `fetched_at=Utc::now()`。新估算不能将 cache-hit 的新 fetched_at 当作独立服务器观测。

- cache hit：`crates/ccr-codex/src/services/openai_quota_core.rs:149-158`。
- fetched_at 入口：`crates/ccr-codex/src/services/codex_quota_service.rs:106`、`:142`。
- response 使用新 fetched_at：`crates/ccr-codex/src/services/codex_quota_service.rs:321-331`。

设计建议：观测 key 至少含稳定账号身份、plan、limit bucket、窗口角色及服务端 reset/窗口数据；保留 `network_acquired_at`、`observed_at`、source=`network/cache`、window presence、error。正常模型 bucket 未映射或 additional limits 无法分类时输出 `N/A`。不要按模型名将共同订阅 bucket 拆成各自独立容量。

保留 legacy `fetched_at` 展示兼容时，应单独新增真实获取时间和 cache-hit 状态。校准只接纳真实 network sample。改变旧字段语义需在设计中明确。

调用者及兼容范围：

- `OpenAiQuotaCore` 和 `OpenAiQuotaFetchOutcome` 属于 `ccr-codex`；两者为 `pub(crate)`，outcome 当前字段只有 `email` 和 `quota`：`crates/ccr-codex/src/services/openai_quota_core.rs:90-93`、`:117`。外层 result 是 `Result<OpenAiQuotaFetchOutcome, String>`：`:121-125`。
- `openai_quota_core` module 本身被声明为 public，但上述核心类型仍受 crate 可见性限制：`crates/ccr-codex/src/services/mod.rs:14`。
- 核心生产 fetch 调用者只有 `CodexQuotaService`：`crates/ccr-codex/src/services/codex_quota_service.rs:230-239`。`CodexAuthService` 只使用其 plan normalization helper：`crates/ccr-codex/src/services/codex_auth_service.rs:14`。历史注释中的 OpenCode 跨页共享不证明当前存在第二个生产 fetch caller。
- 外层 public `CodexAccountQuota` 被 `ccr-cli` re-export：`crates/ccr-cli/src/models/mod.rs:16-17`；`CodexQuotaService` re-export：`crates/ccr-cli/src/services/mod.rs:44`。
- CLI quota command 直接消费 public service，JSON 输出同一 DTO：`crates/ccr-cli/src/commands/codex/quota.rs:11-18`。需保留已有百分比/reset/错误合同。
- TUI `AsyncMessage`、preview 和 selected quota 直接使用 public DTO：`crates/ccr-tui/src/tui/codex_auth/app.rs:89`、`:115-137`、`:368`。
- Tauri `codex_get_all_quotas/codex_get_quota` 将 public DTO 序列化为 `OpenJsonValueDto`：`ccr-ui/src-tauri/src/commands/codex_usage.rs:62-79`。新增 public 字段会进入这些 payload，不能视为只影响 TUI。
- Tauri tray 的 `CodexTrayAccountRow.quota` 嵌套 `CodexQuota`：`ccr-ui/src-tauri/src/commands/codex_tray.rs:6-19`；当前 quota 同样调用 service：`:58-64`。需要编译/合成兼容验证。
- Tauri 专用 `codex_get_usage` 也调用同一简化 usage service：`ccr-ui/src-tauri/src/commands/codex_usage.rs:47-57`。扩展 shared usage DTO/语义后需确认其既有 payload builder 不被意外改义。

最小方案是在 crate-private outcome 增加 acquisition provenance，sample 写入留在 service；public DTO 如增加 bucket 字段，应采用兼容的 optional/default 结构并覆盖所有构造 fixture。若可在 crate-private 独立 snapshot 中完成校准，不必为了样本历史扩大既有公开 JSON shape。

### 9. ModelRateCatalog 可以复用，但需要请求级输入和价格能力补齐

现有 `ccr-types::ModelRateCatalog` 支持非缓存输入、cache read、cache creation 和完整 output 的美元成本计算；允许 override；返回 normalized model、pricing_status/source、rate label 和 rates。

- rate/computation 结构：`crates/ccr-types/src/model_rate_catalog.rs:4-30`。
- 计算接口及成本公式：`crates/ccr-types/src/model_rate_catalog.rs:49-103`。
- 未知模型返回 `unpriced`，内部 numeric costs 为 0：`crates/ccr-types/src/model_rate_catalog.rs:65-75`。新界面必须以 `N/A/unpriced` 呈现，不能显示 `$0` 作为有效估值。
- override 是 normalized ID 精确匹配，source=`override`：`crates/ccr-types/src/model_rate_catalog.rs:114-129`。
- normalization 去已知 provider 前缀、lowercase 和 `_`：`crates/ccr-types/src/model_rate_catalog.rs:182-207`。
- 现有 OpenAI 模型分支只含 gpt-5.4-mini/gpt-5.5/gpt-5.4/gpt-5.3-codex 及 legacy：`crates/ccr-types/src/model_rate_catalog.rs:279-320`；`gpt-6.1-sol` 未收录，最终落入 None：`:355`。
- long-context 规则：输入总 prompt tokens `>272000` 时 input/cache read ×2，output ×1.5：`crates/ccr-types/src/model_rate_catalog.rs:376-385`。
- `basic_rate` 的 cache creation 默认为 input rate：`crates/ccr-types/src/model_rate_catalog.rs:358-363`；现有结构允许专门 cache-write rate，但 gpt-6.1-sol 如有不同 cache-write 价格必须显式配置。
- `rate_summary` 用 prompt=0；不能据此判断请求是否触发长上下文档位：`crates/ccr-types/src/model_rate_catalog.rs:107-111`。

现有 calculate 方法没有服务 tier、价格生效日期、出处 URL、价格快照版本。把整个 5h 累计 token 一次传入 calculate 会错误触发长上下文档位；必须逐请求/计量事件定价，再累计费用。原始 Codex input 如包含 cached，必须先转为互斥桶；reasoning 如含在 output，则仅对 output 计费一次。

最小复用方案保留现有固定 price catalog 的公开 API，增加已核实 canonical model/rates。Auth 估价快照单独保存官方来源 URL、核实日期、快照版本、tier、长上下文状态及估价质量。tier multiplier 与 provenance 包装留在 Auth 估价服务，除非发现现有 rate API 无法表达必须支持的价格关系，不进行通用价格 API 重构。

如果共享 catalog 新增 rate/档位逻辑，需检查其现有 legacy consumers；不得在本任务中重价 llmusage.db 历史 rows。现行规格明确桌面投影保存上游 pricing provenance：`.trellis/spec/ccr/backend/llmusage-provider-adapter.md:273-287`。

### 10. 最小责任文件及分层顺序

| 责任 | 最小候选路径 | 边界 |
| --- | --- | --- |
| Codex 用量标准化、解析质量、去重及费用输入 | `crates/ccr-codex/src/services/codex_usage_service.rs` | 保留原始 inclusive 计数和互斥展示；升级 usage cache version；仅读取允许的用量元数据 |
| quota typed 字段、cache 真实获取时间 | `crates/ccr-codex/src/services/openai_quota_core.rs` | 不增加授权探测，不把 cache hit 计作新服务器观测 |
| quota DTO 及稳定 identity | `crates/ccr-codex/src/models/codex_auth.rs` | 兼容 old registry/cache；保留账号重命名的稳定 ID |
| quota 样本持久化入口 | `crates/ccr-codex/src/services/codex_quota_service.rs` | 只保存非敏感样本；不能保存 OAuth/access tokens 或完整原始响应 |
| 周期/观测估算纯计算与样本存储 | `crates/ccr-codex/src/services/` 中一个经 design 审批的新模块 | 复用 ccr-core atomic writer/locks 和 CodexPaths；质量标签、样本不足、reset变化、bucket错配为一等状态 |
| 模型价格 | `crates/ccr-types/src/model_rate_catalog.rs` | 固定catalog补官方 canonical ID及已核实rate；tier/日期/URL/version留在Auth估价快照；避免隐式猜测模型alias |
| 导出 | `crates/ccr-codex/src/services/mod.rs`、`crates/ccr-codex/src/lib.rs`；按需 `crates/ccr-cli/src/services/mod.rs` | 仅导出实际新增服务/DTO |
| Auth 状态与后台加载 | `crates/ccr-tui/src/tui/codex_auth/app.rs` | 归属/费用在 service；TUI 保留 typed 状态、选中账号、刷新及质量标签 |
| Auth 展示及中英文 | `crates/ccr-tui/src/tui/codex_auth/ui.rs` | 小尺寸仍显示来源/范围/质量；TestBackend 检查真实 composed path |
| llmusage 共享投影 | 无默认修改 | 本任务不向现有 provider SQL 混入 Auth 账号，也不重价上游数据库 |

建议顺序：先确定互斥 token 与计价合同，再补 quota 样本身份/真实时间，随后建立纯估算与质量状态，最后接入 TUI。估算不得改变原始剩余百分比。

### 11. 建议验证命令与测试矩阵

本次 `NOT_RUN`。实施批准后，遵守根 AGENTS 和 justfile 的默认并行及 `--skip export_bindings`，不要沿用部分旧 spec 中的 `--test-threads=1`。

```powershell
just version-check
just fmt-check
cargo test -p ccr-codex --all-features -- --skip export_bindings
cargo test -p ccr-types --all-features -- --skip export_bindings
cargo test -p ccr-tui --all-features -- --skip export_bindings
just lint-strict
just test
just ci
```

若未修改 ccr-usage SQL，无需增加其独立生产数据探测。若修改 ccr-types catalog 且影响既有 default rows/legacy consumers，加入 `ccr-store` 与 `ccr-db` 合成测试。跨模块最终验收由 `just ci` 承担。

- 正式并行策略：`justfile:397-401`。
- 格式验证：`justfile:464-468`。
- secret write/clippy 严格门槛：`justfile:479-484`。
- Codex isolated fixture：`.trellis/spec/ccr-codex/backend/test-fixtures.md`；现有 parser 测试使用 TempDir：`crates/ccr-codex/src/services/codex_usage_service.rs:581-584`。
- UI 最低矩阵 EN/ZH 80×24、100×22、100×30、120×22、140×40、180×50；quota stale/missing/error、成功 inferred、真正 global fallback、unpriced、partial token schema、N/A、估算样本不足：`.trellis/spec/ccr-tui/backend/backend-guidelines.md:434-450`。
- parser 必须涵盖 cumulative/last 双上报、重复 completed、mixed partial turns、日志副本、counter rollback/new epoch、缺失 timestamp、未来 timestamp、cache read/write 及 reasoning 包含关系。
- 定价必须涵盖 canonical/精确 alias/未知模型、272000 边界与逐请求累计、cached/cache-write、reasoning 不重复、tier 与价格版本。
- 样本/估算必须涵盖 cache-hit 不新增独立观测、同账号不同 bucket/plan/reset、百分比量化、无可见 token 增量、外部消耗/多设备不可见、服务端 window 缺失/unknown。

## Files Found

- `crates/ccr-tui/src/tui/codex_auth/app.rs`：Auth usage 加载、选中账号范围、时间区间归属、quota preview 状态。
- `crates/ccr-tui/src/tui/codex_auth/ui.rs`：截图中的 quota 条形图和 token/Requests 表。
- `crates/ccr-codex/src/services/codex_usage_service.rs`：简化 JSONL usage 解析、文件 cache、滚动统计。
- `crates/ccr-codex/src/models/codex_auth.rs`：Auth registry、激活 ledger 和 quota DTO。
- `crates/ccr-codex/src/services/codex_auth_service.rs`：切换、保存、检测指针变化和 rename 时维护 ledger。
- `crates/ccr-codex/src/services/openai_quota_core.rs`：wham DTO、quota 百分比转换、进程内共享 cache。
- `crates/ccr-codex/src/services/codex_quota_service.rs`：quota orchestration、OAuth token 回写和 fetched_at 构造。
- `crates/ccr-codex/src/services/codex_history_sync_service.rs`：历史 provider 可见性改写，形成来源元数据可信边界。
- `crates/ccr-config/src/managers/provider_activation.rs`：独立的 profile/provider 激活时间线。
- `crates/ccr-types/src/model_rate_catalog.rs`：CCR-owned 静态模型定价、override 和长上下文档位。
- `crates/ccr-usage/src/{lib,queries,db}.rs`：read-only llmusage SQL/DTO，provider 维度不等同 Auth account。
- `justfile`：默认并行、binding skip、fmt 和 strict lint 验证锚点。

## External References

本文不执行网络研究。主会话已提供下列后续来源，价格及协议结论应以主会话的外部研究报告为准：

- OpenAI Codex protocol pinned commit `822e58cc3d666166c7446c5b1ea2e52f5d09594c`：主会话报告 `TokenUsage` 包含 `cache_write_input_tokens`，`RateLimitSnapshot` 有 bucket 相关字段。本文只核查 CCR 现状缺失，不代替上游字段包含关系核验。
- `https://learn.chatgpt.com/docs/app-server`：主会话报告 `account/usage/read` 提供 lifetime/每日 tokens，缺少费用和 5h 细分；可作为独立来源后续评估，不能与现有本地样本直接合并。
- gpt-6.1-sol 官方价格由主会话另行记录；本文只指出 CCR catalog 当前未收录该模型及现有计算接口不足。

## Related Specs

- `.trellis/workflow.md`：保持 Phase 1 planning；研究文件持久化；未批准实现前不启动任务。
- `.trellis/spec/ccr-tui/backend/backend-guidelines.md:393-450`：官方 quota 与 local usage 范围、状态、cache、尺寸及 native evidence 边界。
- `.trellis/spec/ccr-codex/backend/backend-guidelines.md`：Codex domain ownership、CodexPaths、秘密写入和服务复用。
- `.trellis/spec/ccr-codex/backend/test-fixtures.md`：临时目录、crate 内环境锁及禁止读取个人数据的测试方案。
- `.trellis/spec/ccr/backend/llmusage-provider-adapter.md:108-128`：SQL sole owner、provider projection、installed CLI 和无上游 crate 依赖边界。
- `.trellis/spec/ccr/backend/llmusage-provider-adapter.md:239-305`：静态 pricing catalog 与 llmusage provenance pass-through 边界。

## Caveats / Not Found

- 根 `CONTEXT.md` 不存在；未创建 context 文件。相关 domain 合同使用 code_map、spec 和源码。
- 本次只验证源码结构与算法，未证明用户截图中的 10.2M/97 与真实账单、实际账号、真实 quota consumption 完全一致。
- 未读取个人 auth、rollouts、usage cache 或 llmusage DB；未联网调用真实 quota API。
- 无现成 quota 历史存储、Auth account-level llmusage query、精确原始 request identity、service tier 字段或 gpt-6.1-sol catalog 条目。
- 上游日志是否包含可用账户正证据、cache-write 与 input 的包含关系、额外模型 bucket 字段如何与 wham HTTP 对应，需要外部协议研究支撑；不能从 CCR 忽略字段推导上游绝对缺失。
- 部分 spec 仍列串行测试示例；本任务实施命令应服从当前根 AGENTS 和 `justfile:397-401` 的默认并行合同。
- TestBackend 只证明 terminal buffer composition。原生终端外观与用户私有 runtime 正确性仍需分别标注证据边界。
