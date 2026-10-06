# Codex Auth 周期 Token、API 等值费用与容量估算

## Goal

在截图所示 CCR TUI Codex Auth 详情中，展示本地 Token 分类、API 等值费用，以及基于实际采样的 5h / 7d 总容量和剩余容量估算。用户能够识别数据范围、价格假设与估算质量，判断近期工作负载大约还能使用多少 Token、相当于多少 API 费用。

## Authorization and Status

2026-10-05 用户授权联网深入分析并创建 Trellis 任务。2026-10-06 用户明确回复“开始按照规划实施”，批准本版规划及必要实现/验证。第五轮完整 `just ci` 停在既有前端依赖的安全公告；用户随后回复“批准，继续”，批准 `checks/dependency-recovery-draft.md` 中两份 JS 锁文件的 3 条目补丁及完整 CI 复测。第六轮完整 `just ci` 退出 0，16 项步骤全部通过，用时 12:30.169；AC1–AC12 全部完成，见 `checks/verification.md`。用户随后要求拆分提交并归档。未推送、未创建 PR、未发布。

## Background and Confirmed Facts

研究基线：CCR `6a2c87eeb4e17c9e317b2cb738e73aeb15999489`；sub2api `f234a142877cb9f4d070c3d81c673ad7da47f7c0`。细节与证据见 `research/ccr-current-state.md`、`research/sub2api-reference.md`、`research/official-and-open-source.md`。

- 当前页面已有 input+output 总量，缺少缓存读写、推理细分和费用。Auth 路径直接读取 Codex JSONL，并未使用 llmusage：`crates/ccr-tui/src/tui/codex_auth/app.rs:287`、`crates/ccr-codex/src/services/codex_usage_service.rs:25`。
- 当前 5h / 7d 是最近时长的滚动统计；Requests 是用量记录数，缺少真实请求标识；解析失败没有覆盖率状态：`crates/ccr-codex/src/services/codex_usage_service.rs:166`、`:468`、`:474`。
- 账户分配依赖本地激活区间，不能证明并发旧会话和外部切换后的真实计费账户：`crates/ccr-tui/src/tui/codex_auth/app.rs:671`。历史 provider 可由可见性修复改写，不能独立证明原始来源：`crates/ccr-codex/src/services/codex_history_sync_service.rs:481`。
- quota 未持久化采样历史；cache-hit 也会得到新的 fetched_at，不能用该时间建立独立观测：`crates/ccr-codex/src/services/openai_quota_core.rs:110`、`:149`、`crates/ccr-codex/src/services/codex_quota_service.rs:106`。
- 共享价格目录可复用，但未收录截图中的 gpt-6.1-sol，未知模型的内部成本是 0 且状态为 unpriced：`crates/ccr-types/src/model_rate_catalog.rs:65`、`:279`。当前目录的长上下文逻辑需要逐请求输入：同文件 `:376`。
- [官方额度说明](https://learn.chatgpt.com/docs/pricing) 未提供固定套餐 Token 容量。sub2api 的 quota 类型同样没有 Token ceiling：参考仓库 `backend/internal/service/openai_quota_service.go:41`。截图的 10.2M 和单次剩余百分比不足以计算可靠容量或费用。

## Requirements

| ID | 要求 |
| --- | --- |
| R1 | 保留最近 5h、最近 7d 和现有全历史统计，补充总输入、缓存读、缓存写、总输出和输出内推理 Token。只统计合法时间范围与去重后的记录；次数按证据标为用量记录数或已识别请求数。 |
| R2 | 费用使用 USD，标为 API 等值费用。按单次可识别请求定价后汇总，保留模型、价格来源、版本、长上下文与速度假设。显式支持经核实的 gpt-6.1-sol；未知模型、缺失分类和部分定价不得显示为完整 $0。 |
| R3 | 本地账本归属继续兼容激活区间，并显示推断性质；未归属、其他账户、真正全局回退和解析覆盖缺口分别呈现。不得把 provider 汇总直接转成 Auth 账户总量。 |
| R4 | 保存不含凭证或请求正文的 quota 观测，使用真实网络获取时间、稳定账户身份、plan、bucket、窗口长度与 reset 标识。缓存重读不增加独立样本，quota 失败保留既有结果与错误状态。 |
| R5 | 分别为有效的 5h / 7d 额度窗口提供经验总容量和剩余容量的 Token / USD 估算、样本数与范围。仅在采样与本地用量满足质量条件时给出数值；标为本地工作负载估算，不宣称官方固定限额或保证可用量。 |
| R6 | 区分本地滚动统计、服务端额度窗口和采样区间。未知窗口、失效或不足样本、reset/plan/bucket 变化、来源不明和同步缺口均有确定状态。受两个相同负载窗口共同约束时，联合剩余量取较小值。 |
| R7 | TUI 只呈现领域结果并协调后台加载；保留原始百分比、重置时间、账户切换、u/r、错误/缓存、中英切换和窄终端操作。新增统计不能让本地扫描或网络读取阻塞输入处理。 |
| R8 | 保持现有 registry、账户激活历史和未知序列化字段。仅新增可重建的用量缓存和有限 quota 观测；遵守现有锁、原子写和私有文件权限。通过合成测试与项目正式检查，并明确 native/真实账户证据边界。 |

## Acceptance Criteria

- [x] AC1（R1、R2）：合成 inclusive usage 的 total 等于 input+output；cached/write 为 input 子集，reasoning 为 output 子集。显式零与缺失有不同状态，异常桶有诊断，不重复加总或收费。
- [x] AC2（R1、R3）：重复累计、last 重报、重复 completed、部分 mixed turns、同 session 文件副本、counter 回退、父子会话复制前缀和无效时间 fixture 不产生未经证明的精确计量；受影响统计显示 partial，校准排除有歧义区间。重读同样文件结果相同。
- [x] AC3（R2）：`research/estimate-examples.json` 的短上下文 Standard / Fast 合成费用为 $5.15 / $10.30；272000 与 272001 阈值按逐请求检验。周期累计超过阈值不会自行触发长上下文费率，cache-write 输入侧倍率与输出推理子集均有回归覆盖。
- [x] AC4（R2）：gpt-6.1-sol 命中本次核实的参考价和显式来源；未知模型显示 UNPRICED/N/A。部分覆盖展示已估值小计与覆盖比例；缺少 tier 或单请求长度显示基础情景假设，不能声称实际账单。
- [x] AC5（R3）：合成 A/B 激活时间线、外部切换检测延迟、并行会话和首激活前记录保持账户范围边界。推断归属、未归属、全局回退可识别，全球回退不进入选中账户容量估算。
- [x] AC6（R4、R8）：30 秒 quota cache 内重复获取不产生第二条独立观测；网络、cache 和失败具有确定时间/状态。history 有容量和保留期上限；写入失败保留旧完整文件且不抹去成功 quota。重命名账户不拆分稳定 ID 历史，文件不包含原始响应、凭证或正文。
- [x] AC7（R5、R6）：同一有效分段的 3 个独立区间、每段使用率增量至少 5 个百分点时可建立经验估算；合成容量中值为 10M Token / $5，经验范围 8M–12M / $4–$6，50% 剩余中值为 5M / $2.50。显示样本数、采样跨度及本地推断限制。
- [x] AC8（R4、R5、R6）：不足 3 段、dp=0/负值/小于阈值、仅 quota 变化而无可用本地 Token、未完整扫描、来源/bucket 不明、过期、窗口/plan/reset 变化时没有有效容量数值。max/min 大于 2 时标为不稳定；缺少价格仅抑制 USD 估算，保留满足条件的 Token 估算。
- [x] AC9（R5、R6）：同 bucket、相同负载基准下，5h 剩余 5M 与 7d 剩余 2M 合成为最多约 2M；两个不同 bucket 或未知窗口不强行合成。UTC 统计边界与本地显示时区无关，未来时间不进入当前滚动窗口。
- [x] AC10（R7）：TestBackend 通过 EN/ZH 的 80×24、100×22、100×30、120×22、140×40、180×50 展示矩阵；原有 quota、错误、范围和操作可见，统计降级有标签，Ctrl+L 不重置选择或后台状态。慢加载 fixture 下输入处理无需等待加载完成。
- [x] AC11（R8）：旧 registry / 用量缓存可读，旧缓存缺少新字段时可重建或显示 legacy 缺失，不改激活历史、不改用户原始 JSONL、不迁移或重价 llmusage.db；参考 sub2api 保持不变。
- [x] AC12（R1–R8）：通过 implement.md 的子系统检查与跨模块 `just ci`，保持默认测试并行、binding 导出独立检查及已有门槛。原生终端检查另记结果；未进行真实账户/其他系统验证时保留 NOT_RUN / UNVERIFIED。

## In Scope and Decisions

范围为 CCR Codex 领域服务、价格目录和 Auth TUI。实现顺序为数据质量与计价、quota 观测、纯估算、TUI 接入。使用现有 JSONL 解析链和 CCR-owned 价格目录；保持 llmusage 只读投影边界。首次打开没有采样历史时显示样本不足。估算仅基于新增、对齐的有效采样，不把历史全量除以当前百分比。上述机制详见 `design.md`。

2026-10-06批准的门槛恢复范围：`ccr-ui/bun.lock`中postcss-selector-parser 7.1.4→7.1.6、source-map-js 1.2.1→1.2.2；`docs/bun.lock`中source-map-js 1.2.1→1.2.2。仅更新这3个现有锁条目，保持清单、审计allowlist和阈值原值。核对frozen install、先通过frontend-audit，再重跑完整`just ci`。

## Out of Scope

- 修改 sub2api、llmusage、Tauri 业务页面、VS Code 扩展或其他供应商功能。
- 代理转发、自动账户轮换、订阅额度销售、实际付款与账单接入。
- 固定套餐容量表、credits 到余额换算、税费/汇率/工具费用、未经核实的模型近似定价。
- 新建或接管 Codex app-server、读取个人会话正文进行研究、真实账户压力/耗尽实验。
- 重写历史账户归属、修改 quota 百分比或扩大现有 auth 操作权限。

## Risks and Evidence Boundaries

本地账本无法覆盖其他设备、云任务与共享额度的全部消耗，经验范围不构成统计置信区间或保证上下界。账户归属继续存在激活时间推断限制。2026-10-06 已重新读取官方 gpt-6.1-sol 价格，规划费率保持不变。产品测试与原生证据由本次实施记录，真实账户端到端行为尚未验证。
