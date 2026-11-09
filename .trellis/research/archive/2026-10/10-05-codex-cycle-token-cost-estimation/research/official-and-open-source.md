# 官方文档与开源资料研究

日期：2026-10-05。状态：RESEARCH_VERIFIED；真实账户端到端能力为 UNVERIFIED。本轮通过公开静态页面与 GitHub 原始源码读取资料，未使用个人认证、会话或数据库。

## 官方事实

| 来源 | 核实结果 | 规划影响 |
| --- | --- | --- |
| [Codex / ChatGPT 价格与额度](https://learn.chatgpt.com/docs/pricing) | Plus 标价 $20/月；包含用量取决于模型、工作量与速度模式。网页给出的消息数范围具有估算性质，周限额可能存在。Codex credits 与 API 定价规则不同。 | 不内置 Plus/Pro 固定 Token 或美元容量；实际返回的窗口优先于套餐推断。订阅价格不作为容量分母。 |
| [GPT-6.1 Sol 模型价格](https://developers.openai.com/api/docs/models/gpt-6.1-sol) | Standard、输入不超过 272K 时，每百万 Token：普通输入 $2、缓存读取 $0.10、缓存写入 $2.50、输出 $10。超过 272K 的整次请求，输入与缓存费率乘 2，输出费率乘 1.5。Fast 为对应 Standard 的 2 倍。 | 必须保留逐请求输入长度与缓存写入。参考价按模型与版本固定，速度未知时只能显示有假设的 Standard API 等值。 |
| [Prompt caching](https://developers.openai.com/api/docs/guides/prompt-caching) | 缓存写入是一种输入费率类别，不能在完整输入费之上再加完整写入费。 | 输入分成普通、缓存读、缓存写三个互斥桶。 |
| [Observability and usage](https://developers.openai.com/api/docs/guides/agents-api/observability) | cached 是 input 的子集，reasoning 是 output 的子集；缺少 usage 不代表零，记录并非最终账单。 | total=input+output，reasoning 仅另行展示，不能再次收费。缺失字段保留未知状态。 |
| [Codex App Server](https://learn.chatgpt.com/docs/app-server) | `thread/tokenUsage/updated` 提供线程用量；`account/rateLimits/read` 提供单桶与按 limitId 分组的窗口。`account/usage/read` 提供账户 Token 汇总和可选每日桶，但没有本文所需的缓存/费用分项或 5h 明细。 | App-server 可作为后续官方账户汇总来源，不能直接替代本地费用账本。本任务不接入或启动第二个 app-server。 |

上表价格于本次读取日核实。提交实现前须重读官方模型价格；模型别名、价格日期和默认 Standard 假设必须可追溯。不得将 Fast 的订阅额度倍率当成 API 费用倍率。

## 上游协议证据

Codex 上游本次读取版本：`822e58cc3d666166c7446c5b1ea2e52f5d09594c`。

- [protocol.rs:2220](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/protocol/src/protocol.rs#L2220)：`TokenUsage` 包含 input、cached input、cache write input、output、reasoning output、total。
- [protocol.rs:2234](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/protocol/src/protocol.rs#L2234)：提供方的 `codex_rollout_budget_units` 标为 `skip_serializing`，本地日志不能据此恢复官方绝对额度。
- [protocol.rs:2328](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/protocol/src/protocol.rs#L2328)：quota 有 limit_id/name、normal_model_slug、plan、primary/secondary 等元数据；配额别名不能替换实际请求模型。
- [protocol.rs:2371](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/protocol/src/protocol.rs#L2371)：window_minutes 注释称滚动窗口长度；resets_at 是下一次重置时间。因此 `reset-duration` 不足以证明官方固定周期起点。

## 开源对照

- [openai/codex #14489](https://github.com/openai/codex/issues/14489) 记录 rate-limit 更新重复携带 last_token_usage 的问题。该问题是历史失败证据，不能单独证明今日所有版本仍如此；解析器需要重复快照夹具，不能无条件累加 last。
- [ccusage #1663](https://github.com/ccusage/ccusage/pull/1663) 在 2026-08-29 合入缓存写入统计。可借鉴直接字段与嵌套 API usage 的归一化、累计变化校验和未知历史字段处理；合入状态不代替 CCR 验证。
- [codex-usage-audit](https://github.com/razzededge/codex-usage-audit) 对本地 Token、缓存与费用分项提供参考，并提示子代理复制父会话前缀的去重问题。应对合成日志验证，不能原样引入第三方聚合公式或价格表。
- sub2api 的本地源码对照见 [sub2api-reference.md](sub2api-reference.md)。版本固定为 `f234a142877cb9f4d070c3d81c673ad7da47f7c0`；代理账单倍率不属于 OpenAI API 官方价格。

## 计算约定与合成示例

记 I=完整输入、R=缓存读取、W=缓存写入、O=完整输出、Q=推理输出。必须满足 `0 <= R+W <= I` 与 `0 <= Q <= O`；缺少字段不自动满足这些条件。

```text
T = I + O
U = I - R - W
API 等值 = (U*P_input + R*P_read + W*P_write + O*P_output) / 1_000_000
```

合成示例采用 GPT-6.1 Sol、Standard、100 次相同请求，每次输入 100,000、缓存读 90,000、缓存写 5,000、输出 2,000、推理 1,000，均不超过 272K。合计 I=10,000,000、R=9,000,000、W=500,000、O=200,000、Q=100,000；T=10,200,000，API 等值=$5.15。Q 已含于 O。W 不能再加到 T 或完整普通输入费用中。

截图只提供 10.2M 总量，缺少上述分类、逐请求模式、长度与周期采样，所以不能据此给出该账户的费用或完整周期容量。若在同一窗口、同一负载组合、同一采价版本的对齐观察区间里，记录 1,000,000 Token、$0.50 等值，额度使用率增加 10 个百分点，则比例外推为完整窗口 10,000,000 Token / $5，50% 剩余对应 5,000,000 Token / $2.50。该外推仍受未观测的其他设备、云端工作、额度规则变化影响，只能作为本地经验估算。

## 实施期间的协议补核

2026-10-06 独立审查补核同一 pinned commit `822e58cc3d666166c7446c5b1ea2e52f5d09594c`：

- [history/src/rollout_payload.rs](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/history/src/rollout_payload.rs) 的 wire enum 使用 type 标记和 snake_case，包含 `Compacted { payload }` 与 `TokenUsageRecord { payload }`。持久化类型名分别为 `compacted` 和 `token_usage_record`。
- [protocol.rs:2243](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/protocol/src/protocol.rs#L2243) 的记录提供 response/turn/session 身份和 usage、turn_token_usage、thread_token_usage。依据响应身份和字段分层，实施将 usage 作为请求计量；该解释属于来源推断。解析不得将三份计量相加。
- `compacted` 进入既有不明确 epoch 的 partial 处理；不解析 compaction 正文。新请求记录复用权威身份去重，混合旧累计记录时保留歧义诊断。以上为公开源码证据，未读取个人 rollout，也不证明本机安装版本已经产生该格式。

## 明确未核实

- 未调用用户真实 quota / usage API；本地安装的 Codex 是否支持新 account/usage/read 为 UNVERIFIED。
- 官方没有提供本任务可使用的 Plus/Pro 固定 Token 容量，百分比到 Token 或美元的固定兑换率为 UNKNOWN。
- 请求工具费、税费、汇率、credits 购买与优惠、代理售价不计入 Token API 等值。
- 覆盖率不能由本地日志完整性证明；实际服务端额度变化原因在缺少证据时应显示原因未查明。
