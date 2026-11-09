# Research: sub2api 的 Codex Token、价格和周期统计对照

- Query: 如何借鉴 sub2api，为 CCR Codex Auth 显示分类 Token、API 等值费用和 5h / 7d 周期容量估算？
- Scope: internal；参考仓库只读源码对照，未执行测试、代理请求、账户探测或部署操作。
- Date: 2026-10-05
- Reference root: `D:/Documents/Code/Agents/sub2api`
- Revision anchor: 本地 `.git/HEAD` 指向 `refs/heads/dev`；该引用为 `f234a142877cb9f4d070c3d81c673ad7da47f7c0`。直接读取引用文件，未执行 git 命令。下文证据来自当前工作区文件；没有推定本地源码与运行镜像相同。

## Findings

### 1. 结论与最小借鉴范围

可以借鉴 4 项机制：统一 usage 解析；互斥 Token 分类；按单个请求的模型、服务层级和上下文长度计算费用；把配额快照与本地账本统计分别保存。

sub2api 的配额类型和快照只提供已用百分比、窗口长度和重置时间，没有官方绝对 Token 容量。它的本地窗口统计仅覆盖经 sub2api 记录的请求。源码没有提供可以直接移植的官方周期 Token 总额公式。

CCR 应把金额命名为 `API 等值费用（USD）`，保留价格来源、模型匹配结果和缺失状态。容量属于本地经验估算（local empirical proxy），应标记为 `历史工作负载估算`。首次采样、缺少分类或价格、周期边界不明时显示不可估算，并保留已经取得的 Token 和配额。

### 2. 文件索引

下列路径均相对于 Reference root；`path:line` 为当前文件锚点。

| 文件 | 作用 |
|---|---|
| `backend/internal/service/openai_gateway_service.go:127` | 原始配额快照、5h / 7d 归一化类型和 `OpenAIUsage` |
| `backend/internal/service/openai_gateway_response_handling.go:1223` | HTTP SSE usage 合并、终止事件替换、非流式共用解析 |
| `backend/internal/service/openai_ws_v2/passthrough_relay.go:1188` | WebSocket 每轮结算、缓存写入解析 |
| `backend/internal/service/openai_gateway_usage.go:154` | 互斥分类、模型选择、费用和用量行记录、配额快照更新 |
| `backend/internal/service/pricing_service.go:623` | 目录解析、覆盖层、已识别模型查找和泛化回退 |
| `backend/internal/service/billing_service.go:1502` | 上下文、缓存、priority 和代理倍率计算 |
| `backend/internal/service/model_pricing_resolver.go:48` | Group → Channel → LiteLLM → Fallback 来源链 |
| `backend/internal/service/service_tier_billing.go:18` | 请求层级与上游声明层级的结算规则 |
| `backend/internal/service/openai_quota_service.go:41` | `/wham/usage` 类型；额度百分比和 credit 独立建模 |
| `backend/internal/service/account_usage_service.go:758` | 为官方窗口附加本地 Token、请求和金额 |
| `backend/internal/repository/usage_log_repo_stats.go:308` | 账户窗口 SQL 汇总 |
| `backend/internal/repository/usage_billing_repo.go:22` | 事务内账单去重与重复指纹冲突 |
| `backend/internal/service/usage_billing.go:18` | 计费指纹和统一金额精度 |
| `backend/resources/model-pricing/model_prices_and_context_window.json:13` | 随源码提供的 `gpt-6-sol` 价格示例 |

### 3. Token 解析和分类

事实：`OpenAIUsage` 保留总输入、总输出、缓存创建和缓存读取，没有独立的 reasoning Token 字段（`openai_gateway_service.go:226-234`）。HTTP 解析依次尝试 `usage`、`response.usage`、`data.usage` 和 `data.response.usage`（`openai_gateway_response_handling.go:1345-1368`）。非流式使用同一解析器（同文件 `1644-1652`）。

事实：输入读取 `input_tokens`，兼容 `prompt_tokens`；输出读取 `output_tokens`，兼容 `completion_tokens`（同文件 `1517-1528`）。缓存读取优先 `input_tokens_details.cached_tokens` 和 `prompt_tokens_details.cached_tokens`；嵌套字段存在且为 0 时仍优先，不使用非零顶层别名覆盖（同文件 `1563-1577`）。

事实：缓存写入优先 `input_tokens_details.cache_write_tokens` / `prompt_tokens_details.cache_write_tokens`，其次嵌套 `cache_creation_tokens`，再兼容顶层 `cache_write_tokens` / `cache_creation_input_tokens` / `cache_write_input_tokens` / `cache_creation_tokens`（同文件 `1580-1597`）。WebSocket 使用同样字段优先级（`openai_ws_v2/passthrough_relay.go:1212-1234`）。这些是通用解析能力，不证明任何特定模型会返回该字段。

事实：普通输入按 `max(InputTokens - CacheReadInputTokens - CacheCreationInputTokens, 0)` 计算。三类输入为互斥桶；输出单独计费（`openai_gateway_usage.go:179-200`）。用量行存的是普通输入、缓存读、缓存写和输出（同文件 `404-407`），窗口总 Token 再将这 4 类相加（`usage_log_repo_stats.go:312`）。

事实：OpenAI reasoning 已包含在 output/completion；解析器只在 `total_tokens` 与独立推理数算术一致的兼容场景调用 xAI 合并逻辑（`openai_gateway_response_handling.go:1529-1538`）。独立推理明细没有写入 `OpenAIUsage`。

建议：CCR 需要额外保留 reasoning 明细用于展示，但 reasoning 作为输出子集，不能再次加到总 Token 或输出费用。CCR 不应移植跨供应商 xAI 判断。字段缺失与显式 0 要分别表示；如果缓存数超过总输入，记录分类异常，避免仅用截断掩盖来源不一致。

### 4. 流式终止、重复和不完整记录

事实：非终止 SSE usage 以非零字段更新当前累计对象；终止事件有有效 Token 时整体替换对象，不逐事件相加。终止全零且已存在有效 Token 时保留先前对象（`openai_gateway_response_handling.go:1223-1244`）。WebSocket 在一轮结束时将该轮 Token 加到连接累计值，然后清空本轮对象（`openai_ws_v2/passthrough_relay.go:1188-1199`）。

事实：已有静态测试覆盖终止替换、终止不带 usage、Responses / Chat 形状、嵌套缓存显式 0 和 reasoning 不重复相加（`openai_gateway_service_test.go:3656-3719`）。本轮只读测试源码，未运行测试。

建议：CCR 当前读取会话 usage 时要按自身数据源的累计 / 增量约定归一化。复用原则为：同一逻辑请求或 usage 事件只能进入账本一次；累计快照差分不应把完整快照再次当增量。不能直接用 sub2api 的 APIKey 去重键代替 CCR 会话 / 事件标识。

### 5. 价格目录、模型身份和新模型限制

事实：价格来源允许 Group、Channel、LiteLLM 和 Fallback，Group / Channel 可覆盖官方参考价（`model_pricing_resolver.go:48-125`）。目录是 model name → JSON 条目，解析器还应用本地 override（`pricing_service.go:623-649`）。默认远程目录位于 `Wei-Shaw/model-price-repo`，本地 fallback 为 `./resources/model-pricing/model_prices_and_context_window.json`（`backend/internal/config/config.go:2323-2326`）。这组目录和 override 的存在不等于当前官方价格已验证。

事实：`GetIdentifiedModelPricing` 避免任意模型系列猜测，未识别时返回 nil（`pricing_service.go:1203-1258`）。另一通用入口允许系列与 OpenAI 回退（同文件 `1173-1200`），最终可能返回 `DefaultTestModel` 的目录价格（同文件 `1606-1613`）。

事实：用量行同时保存请求模型、发给上游的模型、上游响应模型，并保存 service tier / reasoning effort（`openai_gateway_usage.go:388-407`）。业务还可选择 requested、channel mapped 或响应模型为计费基准（同文件 `225-307`）。

建议：CCR API 等值费用使用可核查的模型身份和确定性价格匹配。未知模型显示 `UNPRICED`；不能采用默认模型或近似系列价格后标为已验证。只有明确声明的别名可归一化，同时保留原始 model id。

#### gpt-6.1-sol 的本地支持状态

事实：在 `backend/internal/service`、`backend/internal/pkg/openai` 的生产代码，以及 `backend/resources` 内没有找到 `gpt-6.1` / `gpt-6.1-sol` 条目。内置识别函数仅列出 `gpt-6-sol` 和 `gpt-6-luna`，以及明确 effort / compact 后缀（`backend/internal/pkg/openai/constants.go:163-178`）。随源码价格 JSON 列出 `gpt-6-sol`（`13-36`），没有 `gpt-6.1-sol`。

事实：通用 cache write 解析已经存在。`gpt-6-sol` 的本地 fallback 写入价为输入价 1.25 倍；还包含 priority 和长上下文倍率（`pricing_service.go:77-92`，`billing_service.go:1842-1868`）。这些规则仅证明当前源码对所列模型的实现。

结论：不能把 `gpt-6-sol` 的数值、cache write premium 或长上下文阈值自动迁移到截图中的 `gpt-6.1-sol`。动态目录或私有 override 可能补充模型，本轮未读取运行目录与私有配置，运行时实际支持为 `UNVERIFIED`。模型本身的官方价格和 usage 语义需要另行核实。

### 6. 层级和长上下文

事实：账单保留 Requested、Observed 和 Billing tier。公开 OpenAI API 的低价响应声明可降低账单；本地源码对 ChatGPT OAuth-like 的 default 响应声明采取保留出站 tier 的特殊政策，注释说明 Codex 私有端点的 observed default 不足以判断 Fast（`service_tier_billing.go:18-74`）。这是 sub2api 的业务判断，不能作为官方契约直接移植。

事实：长上下文基准为普通输入 + 缓存写 + 缓存读（`billing_service.go:1502-1517`，`1917-1928`），用整笔请求所在区间计价（`billing_context_schedule.go:18-31`）。输入、缓存读、缓存写分别应用输入侧倍率，输出应用输出侧倍率（`billing_service.go:1592-1607`）。Group / Account 可控制长上下文计费开关（同文件 `1505-1509`）。

建议：CCR 按官方模型规则逐请求定价后汇总，不能先汇总周期 Token 再与单请求阈值比较。订阅会话缺少 tier / 单请求上下文长度时，标为基础价格情景估算或不可完整定价。保持 price source、rate version、tier basis 和 coverage，可用数据不能自行补成 Fast 或长上下文。

### 7. 三种金额的区别

事实：`CostBreakdown.TotalCost` 为输入、输出、缓存和多模态费用之和；`ActualCost = TotalCost * rateMultiplier`（`billing_service.go:1648-1668`）。RecordUsage 将分组、用户和账户业务倍率以及高峰因子带入（`openai_gateway_usage.go:203-221`），统一计费还可叠加渠道分时和 reasoning effort 倍率（`billing_service.go:1550-1552`）。因此字段名 `ActualCost` 不能解释成 OpenAI 的真实账单。

事实：窗口 SQL 同时汇总账户成本、标准成本和用户扣款；账户成本使用 `account_stats_cost` 或 `total_cost` 再乘账户倍率，用户成本使用 `actual_cost`（`usage_log_repo_stats.go:308-334`）。APIKey 配额和用户余额扣款使用 ActualCost，账户配额使用 TotalCost × AccountRateMultiplier（`gateway_usage_billing.go:336-346`）。

建议：CCR 仅借鉴分项金额与来源结构。API 等值费用不应用代理分组、账户、峰时、reasoning effort 或销售倍率。ChatGPT 订阅月费、额外 credit 和 API 等值费用需要独立语义。展示金额使用 USD；若未接入实际账单，不能使用 `实际费用` 标签。

### 8. 5h / 7d 快照和本地周期

事实：`OpenAIRateLimitWindow` 包含 `used_percent`、`limit_window_seconds`、`reset_after_seconds` 和 `reset_at`，没有 token_limit 或美元总额（`openai_quota_service.go:41-56`）。credits 为 nullable decimal 字符串余额，重置信用为独立结构（同文件 `72-106`）。

事实：响应头快照保存 primary / secondary 的原始百分比、窗口和 reset，再保存 canonical 5h / 7d 和采样时间。reset_at 基于快照的 UpdatedAt 推导（`openai_gateway_usage.go:1038-1124`）。归一化优先比较 window minutes；缺少 window 时采用 legacy primary=7d、secondary=5h 假设（`openai_gateway_service.go:149-218`）。后者是本地兼容假设，不是适合新估算器的确定窗口证据。

事实：为窗口附加本地 usage 时，起点为未来 reset_at 减 5h / 7d；失效或未知时退回 now 减固定窗口（`account_usage_service.go:758-769`，`1581-1585`）。统计 SQL 按 account_id 和 created_at >= start 筛选（`usage_log_repo_stats.go:308-334`）。没有按同步水位设明确 end 边界；CCR 的估算查询应使用两个采样之间的半开时间区间和同一水位。

事实：配额过期时该实现将 Utilization 设为 0（`account_usage_service.go:1573-1576`）。CCR 应将旧快照标为过期；未收到新配额证据时，不将过期百分比归零等同于已测量新周期。

结论：sub2api 的 `WindowStats` 证明可以同时展示配额和本地用量。它没有证明本地窗口覆盖全账户消耗，也没有从百分比提供绝对容量。

### 9. 去重和金额一致性

事实：usage_logs 具有 `(request_id, api_key_id)` 唯一索引（`backend/migrations/027_usage_billing_consistency.sql:32-33`）。另有独立 billing dedup 表（`backend/migrations/071_add_usage_billing_dedup.sql:4-13`）。repository 在同一事务内 claim 去重键并执行金额变更；重复相同 key / fingerprint 返回 Applied=false；相同 key 不同 fingerprint 返回冲突；还检查归档 dedup 表（`usage_billing_repo.go:22-109`）。

事实：UsageBillingCommand 包含账户、模型、tier、推理等级、Token 和金额用于指纹，金额统一量化到 NUMERIC(20,8) 对应的 8 位小数（`usage_billing.go:18-101`）。上层只在 Applied=true 后更新缓存与后续动作（`gateway_usage_billing.go:353-389`）。

建议：CCR 统计延续自身账本的事件标识和去重契约；quota sample 另外使用账户 + 窗口身份 + observed_at 去重。重复刷新不能增加 Token 或样本权重。保持原始 Token、定价来源和估算版本，使重算可核查。金额展示可以舍入，计算和存储精度应统一。

### 10. 周期估算建议（推论，非 sub2api 已实现能力）

需要同一账户、同一额度窗口身份的两次有效采样 `s0` / `s1`，以及 `[s0.observed_at, s1.observed_at)` 内可归属、已去重的本地 usage。已用百分比变化为 `dp > 0`，同步对齐后的 Token 变化为 `dT`，已完整定价的 API 等值变化为 `dC`。

候选情景公式为 `Token容量 ≈ dT / (dp / 100)`、`API等值容量 ≈ dC / (dp / 100)`；剩余容量以当前剩余百分比乘情景容量计算。公式只表示采样间工作负载对应的历史消耗强度。采样与账本覆盖对不上时，公式不能产生可信总容量。

失效条件至少覆盖：窗口 reset、百分比下降、plan / model mix / tier 变化、不可归属 usage、同步水位未到采样时刻、缺少价格、百分比量化导致 dp 太小、过期快照、外部设备消耗。外部设备消耗不能由本地账本完全排除，显示限制必须明确。

首次快照不能把全历史 Token 除以当前百分比。不同模型的 quota 权重、缓存比例、输出比例和工具消耗可能不同，应显示工作负载条件、样本数、采样跨度及质量状态。短期和周期限额分别估算，不能把两者相加为一个可用额度。

## External References

- 本轮参考本地源码，未访问远程项目或官方接口。线上官方价格由主会话的官方资料研究负责核实。
- 本地配置声明的目录来源：[Wei-Shaw model-price-repo JSON](https://raw.githubusercontent.com/Wei-Shaw/model-price-repo/main/model_prices_and_context_window.json)。`backend/internal/config/config.go:2323-2326`。未联网验证当前内容、版本或官方准确性。
- 本地 AGENTS.md 声明当前采用 `ghcr.io/ranxi2001/sub2api:2.9.6` 官方镜像，暂停二开。未读取运行容器、数据卷或验证镜像状态；源码结论不代表该镜像行为。

## Related Specs

- CCR `.trellis/spec/ccr-codex/backend/backend-guidelines.md`：Codex auth、quota 和 usage 由领域 crate 负责；使用 CodexPaths，保护凭证，合成测试。
- CCR `.trellis/spec/ccr-tui/backend/backend-guidelines.md`：TUI 仅拥有呈现和交互，不读取私有配置或写数据库；保持中英标签与异步加载边界。
- CCR `.trellis/spec/ccr/backend/llmusage-provider-adapter.md`：`ccr-usage` 为共享 llmusage SQL owner；不链接 upstream Rust crate；费用来源与 pricing state 保留。
- sub2api `.trellis/spec/backend/index.md` 已读；数据库、结构和质量文档仍是模板，不能提供额外已落实契约。

## Formal Implementation Validation Points

后续 CCR 实施应验证：

1. 合成 Responses usage：普通输入、缓存读、缓存写、输出和 reasoning 子集的互斥与总额不重复相加；嵌套显式 0 与字段缺失分离；异常分类有状态。
2. 累计 / 增量和重复事件：同一 usage 重读、文件重复扫描、终止快照替换、部分或缺失 usage 的状态；请求次数不能随 quota refresh 增加。
3. 明确模型价格：已知 model 与声明 alias 命中；`gpt-6.1-sol` 未核实定价时保持 UNPRICED；未知模型不得落到相邻型号；价格与缓存字段缺失不能表现为完整 0 美元。
4. 单请求层级、上下文阈值和缓存价格：阈值前 / 等于 / 超过、priority 缺失、reasoning 包含于 output；按请求计算后聚合；避免移植代理倍率。
5. 配额周期：primary / secondary 交换、缺失 window、过期 reset、reset 后 percent 下降、同次采样重复保存、跨时区显示不改变 UTC 统计边界。
6. 估算状态：首次快照、dp=0 / 小变化、同步延迟、账本覆盖不足、价格部分覆盖、plan 改变、工作负载改变、外部使用限制；缺失数值为 null / 不可估算。
7. 账户隔离：不同账号、未归属记录、切换账号、其他设备影响；只读查询与 snapshot 元数据写入明确分工，个人请求正文和凭证不得进入 research / 估算日志。
8. TUI：窄终端、CJK 显示宽度、缓存 / 错误 / 不完整状态、中英切换和现有 quota refresh / 切换交互。

## Caveats / Not Found

- 没有读取 `local-deploy`、`.env`、数据库、tokens、个人 auth、请求正文或容器运行数据。
- 没有改 sub2api 的任何文件；已存在 AGENTS.md 变更保留。只写当前 CCR 任务 research 文件。
- 本轮静态研究未运行测试。上面的测试源码只证明覆盖意图，运行结果为 NOT_RUN。
- 原因未查明：运行中的 sub2api 是否已通过动态价格目录支持 `gpt-6.1-sol`。本轮未探测该运行环境。
- 没有发现官方绝对 Token 容量字段或可以证明固定周期 Token 总额的本地机制。
- 本地源码快照 SHA256：`openai_gateway_response_handling.go` = `1187827A2C02F1BDB159B335EE1B06625BD4CF59DC7A41E79D0E2727F6E0C097`；`openai_gateway_usage.go` = `5A2CEA318D97DEDBCD029024065BE6A183A508DFFDA845E4E13BB8CD34F96296`；`pricing_service.go` = `17E2259CE18EE082EFB9E66C27E44BB894D324C29D2C6B0AC9E095761477C945`；`billing_service.go` = `587CDA930158FA33CCB681C48189182407E613599928D1F52EC32EF156A15C27`；随源码价格 JSON = `2E96CD43EC42C8217A8348B904C1E5D43531581CDC8497727A1DB8F64CFC0DB9`。相对路径见文件索引，哈希绑定本次读取的工作区内容。
