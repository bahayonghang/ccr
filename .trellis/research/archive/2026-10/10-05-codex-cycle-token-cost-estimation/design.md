# 技术设计：Codex Auth Token、API 等值费用与经验容量

## D1. 责任边界与数据流

```text
Codex usage metadata → ccr-codex 解析/去重/质量 → 本地滚动统计
CCR 激活 ledger → ccr-codex 归属（inferred） ────┘
ccr-types ModelRateCatalog → 逐请求基础价 → 已估值小计/覆盖率
既有 quota query → 网络时刻/窗口元数据 → 有限观测文件
观测 + 对齐的本地增量 → 纯估算 → ccr-tui typed state → EN/ZH
```

Auth 不改用 provider 级 llmusage 汇总，因为当前 `QueryFilter` 没有 Auth account 或精确采样时间边界（`crates/ccr-usage/src/db.rs:31`）。已有 profile usage / desktop SQL 仍由 ccr-usage 独占，本任务不添加 SQL、不链接 upstream llmusage crate，也不将两套统计相加。

ccr-codex 负责归属、时间范围、定价输入、覆盖和估算。ccr-types 只补纯价格数据/规则。ccr-tui 负责 typed load state、刷新、选择和显示，不保存观测文件、不解析私有配置、不定义计价公式。归属算法由 `CodexAuthApp::records_for_account` 下移为可注入时间线的领域函数，保留既有结果与兼容包装；不顺带重构其他 auth 页面。

## D2. Usage 归一化、身份与覆盖

扩展现有 CodexUsageRecord / Stats，保留原字段兼容。新增的缓存、推理、tier、单请求长度与来源字段使用 Option/显式状态，缺少字段保留 unknown。原始 input/output 为 inclusive；另提供普通输入、缓存读、缓存写、可见输出和推理输出的可选互斥投影。规范为 `R+W<=I`、`Q<=O`、`T=I+O`；不满足时保留原值和 `inconsistent_usage`，不能静默 clamp 后标为完整。

解析支持已核实的 flat Codex token 字段与嵌套 Responses usage；字段优先级保留显式 0。以原始实际 model 为价格键；quota normal_model_slug 不代替请求 model。model_provider 仅为可变线索。记录新增 `time_basis`、`attribution_basis`、`measurement_basis` 和 scan diagnostics，避免将缺失时间补为 now 后参与周期估算。

事件策略：

1. 有 response/turn 身份时，每个身份选择一次权威计量，允许更新同一记录的晚到分类。completed 与 cumulative/last 不叠加；不能仅因文件含一个 completed 就丢弃全部其他 turns。
2. 旧累计格式只对已验证同一计数 epoch 的单调变化差分。相同累计值或纯 quota 重报没有新用量。模型/tier 切换、计数回退或 compaction 的 epoch 边界若无法证明，不将恢复旧值当新用量；相应区间标 partial，重新建立基线。
3. 同 session 的源文件副本以确定性 session/event identity 去重；有 ordinal 优先使用。仅数值相同不足以删除合法新请求。子代理复制父前缀只在 fork/parent 身份与边界可证时排除；来源不明则标 `possible_duplicate`，不用于容量校准。
4. 缺少 request identity 或只能得到 turn aggregate 时，保留本地 Token，但标记计量粒度；次数显示用量记录数。单次请求长度未知不能用 turn/周期总量判断 272K 档位。
5. unreadable file、invalid JSON、missing timestamp、未来记录和歧义计数产生计数式诊断，不记录行正文。没有证据时不宣布扫描覆盖所有账号活动。

按固定 `as_of` 计算滚动窗口，要求 `since<=timestamp<=as_of`。校准查询统一为 `[sample0.network_acquired_at, sample1.network_acquired_at)`；每个观测端点都绑定该观测之后完成的 scan_result，水位至少覆盖该端点。晚到记录必须让相关区间重新计算或失效；不要求扫描开始早于网络观测。扫描结束不能证明远端消费已全部写入日志；显示该不可见限制。

用量缓存版本升级，旧缓存可丢弃后读取原日志重建，不能将 old missing fields 变成 0。不改原始 JSONL 或 usage_ledger。对已有 legacy 归属，`activation_interval_inferred` 持续可见；首激活前和未知来源保持 unattributed。全局回退仅显示全局本地统计，绝不进入选中账户校准。

## D3. 逐请求定价与版本

复用 `ModelRateCatalog::calculate` 的互斥输入接口，先将 I/R/W 转为 U/R/W，再传全部输出 O。`API_equivalent=(U*P_input+R*P_read+W*P_write+O*P_output)/1e6`。Q 仅展示一次，不追加费用。

补充 canonical `gpt-6.1-sol` 及经官方明确确认的别名，保留原 model id；没有明确别名时只支持 canonical，不做家族 fallback。参考价来源与日期见 `research/official-and-open-source.md`。新增短上下文 rate 为 input/read/write/output=$2/$0.10/$2.50/$10 per MTok；单请求 inclusive input >272000 时输入、缓存读、缓存写 ×2，输出 ×1.5；Fast ×2。检查当前 long-context helper 对 cache-write 与新模型的能力，必要时仅修正相关模型规则，不重价其他型号。

每个价格计算带 AuthPricingBasis：catalog version/fingerprint、来源 URL、核实日期（内置旧价可为 unknown）、currency=USD、模型匹配、context basis、tier basis。元数据归 Auth 领域，避免为本任务重构整个公共价格 API。已有其他 catalog 模型可以显示 `catalog_estimate`，不能伪造最新官方核实日期。override 可显示 override estimate，与官方来源不同。

默认显示 `Standard API equivalent`。已核实 actual mode 时可显示对应费用；OAuth log 的 `service_tier=default` 不足以证明实际 Standard。缺少 mode 或请求长度时只给带明确假设的基础情景小计，状态为 assumed，不能称实际费用。已知 Fast 样本使用 Fast；未知模式的容量 USD 情景统一按 Standard，不和 known Fast 混合。价格状态 unpriced 的 numeric 0 转为 N/A。

费用状态区分 complete_priced、assumed_standard、partial、unpriced。部分记录显示已估值小计，同时提供 priced_records/all_records 和 priced_tokens/all_valid_tokens 两个覆盖分母。缺失 W 的新模型可以给注明 W=0 假设的基础价，但不能标 complete；USD 容量校准只接受分类完整、可逐请求定价且同价版本的增量。Token 校准不依赖价格完整度。

金额内部使用已确定的统一精度（优先现有计算类型，聚合前不按分舍入），界面最终舍入至小数 2 位；很小非零值显示 `<$0.01`。工具费用、税费、credits、订阅月费和代理倍率均不计入。

## D4. Quota 观测、真实时间和存储

扩展 OpenAiQuotaCore 内部 outcome/cache metadata，区分 request_started_at、network_acquired_at、returned_at、cache_hit。真实响应读取完成时固定 acquisition time；cache-hit 保留原值。现有 fetched_at 展示若改变为真实获取时间，作为明确语义修正并测试所有调用者；不通过伪造 fetched_at 维持假新鲜状态。结构兼容用 additive 字段/适配函数完成。

观察对象至少包含 schema_version、stable account_id、plan、source、quota generation、limit bucket、primary/secondary 原始角色、duration、raw used_percent、resets_at、acquisition/returned 时间、local scan generation、水位及质量。仅记录成功的新 network observation；cache-hit 不增加权重。失败事件保留为状态诊断，不生成数字为 0 的成功样本。TUI-selected account 和被查询账号使用稳定身份匹配，账号 rename 不切割历史。

wham 主 rate_limit 可命名为 `wham_main` 来源桶；若响应提供 limit_id，保留该原始 ID。两种来源不能混为同一系列。code_review 和 additional/model buckets 不和主桶混合。duration=300 或10080 分钟才标 5h/7d，不按 primary/secondary 位置猜测；其他时长保留窗口名称，容量状态 unsupported_window。duration/reset 缺失时不估算。main bucket 是否覆盖本地所见模型仅能给 inferred 质量；显式其他模型 bucket 或 API key/custom route 时不进入本任务的主桶估算。

新存储为 `CodexPaths::ccr_codex_dir/quota_observations.json`，manager 使用现有 leaf lock、受保护原子写和 `secret(true)` 私有文件策略。只保存白名单元数据，不保存 raw_data、email、tokens、headers、正文或完整路径。保留最多35天、4096条、8MiB；写前裁剪最旧观测，读取超限或损坏文件返回 history_error，不静默清空。观测持久化失败保留成功 quota 与旧 history，显示 history warning，不能连带触发 auth 重试或覆盖用户历史。该文件与 usage cache 可重建；旧 registry 保持原样。

沿用既有 quota action/preview 网络路径，不新增定时网络请求或特殊 auth 探测。每次获得新网络结果时保存一次；本地样本在使用者下一次刷新后自然积累。采样/落盘在已有后台任务执行，错误与显示加载分离。

## D5. 经验容量算法和降级

估算对象按 `(account_id, plan, bucket_source+id, duration, reset_generation, model_set, pricing_basis, tier_basis)` 分段。相同窗口 reset 标识改变、percent 下降、plan/模型集合/速度/价格版本改变时新建基线，不能把跨边界差值当一次消耗。模型比例与缓存比例仍可能变化；通过经验范围和有限状态展示，不把该范围称统计置信区间。

采样段必须满足：两端为真实 network 观测、同一分段、raw used percent 在0..100且有限、dp至少5个百分点、有正的可用去重 Token 增量、没有已发现的扫描/时间/归属歧义或期间 activation change、稳定 runtime route 与账号匹配。对于 wham_main 的主桶与激活区间推断，结果始终是 local_estimate，不能升级为官方准确账户消耗。已发现其他来源或 bucket 错配时 invalid_scope；尚不可排除其他设备时保留低置信本地估算标签与不可见限制。

每段：`cap_T=dT/(dp/100)`；有完整逐请求费用时 `cap_USD=dC/(dp/100)`。首个快照只有基线，无容量。至少3个独立、不重叠有效区间（每段≥5pp）才出数字；不足为 insufficient_samples。只使用当前分段最近最多20段；相同 acquisition+identity 只计一次。

对 cap_T 和 cap_USD 分别取 median，min/max 为观察到的经验范围；max/min>2 记 unstable，隐藏该指标容量数字。dp 阈值控制小变化敏感度，不能宣称已消除官方量化误差。观测保留 raw f64，不能使用 TUI 整数百分比作为分母。

当前 remaining=`clamp(1-used_percent/100,0,1)`，每个容量指标及经验范围均乘该值。已过 reset、当前网络观测超过5分钟、缓存来源无法追溯、服务端窗口缺失时 remaining 为 stale/N/A，保留历史经验容量供诊断但不呈现有效当前可用量。5分钟是本任务的 UI freshness 规则，不是官方服务端 TTL。

5h 与7d 分别估算。同 bucket、同模型集合/费用基准且两端均 fresh 有效时，联合 Token 中值取两窗口剩余中值 min，区间端点分别 min。联合 USD 同样处理，但仅有两边完整、同基准的费用估算时输出；标签为同负载假设下最多约可用。任何一个实际存在的约束未知时，联合值 N/A。不能对不同 bucket 相加或取 min 后宣称全账户额度。

状态包括 insufficient_samples、partial_usage、invalid_scope、unsupported_window、reset_changed、stale、unpriced、unstable、history_error、local_estimate。原因未查明的服务端消耗差异不归因于特定设备；dp>0但dT=0记录 unexplained_quota_change 并抑制该段。价格不完整只抑制 USD，Token 条件独立判断。

## D6. TUI 呈现与交互

保留 Account & quota 原有百分比/Reset/真实 Cached 时间。Local usage 继续提供最近5h/最近7d/All time，次数改为语义准确的用量记录数；追加 input、cache-read、cache-write、output、reasoning（included in output）和 API 等值小计。范围行说明本地激活区间归属和 excluded 记录；缺失项用 N/A。

宽屏追加一张简洁估算表：Window / total estimated / remaining estimated，Token 与USD各一行，显示样本数与local_estimate状态；关键文案为“按近期本地负载估算”，并说明其他设备和云端用量不可见。价格模式/覆盖率靠近金额。中等屏优先 quota、滚动总量/费用、质量和每窗口剩余估算；细分可按现有面板空间折叠，不能隐藏 partial/unpriced。最低测试尺寸保留可见错误、范围和footer，不添加重复快捷键行。

只在 domain 后台加载完成后替换同一账号/请求 generation 的快照，旧结果带 stale 状态保留。切换账号后迟到旧结果不得覆盖新选择。扫描、计算、持久化不在 render/on_key 主线程执行。Ctrl+L 和resize 仍即时响应，u/r 沿用原路由。

## D7. 兼容、责任文件与回退

| 候选路径 | 改动目的 |
| --- | --- |
| `crates/ccr-codex/src/services/codex_usage_service.rs` | 保留新字段、确定计量来源、去重、诊断、时间水位和滚动投影 |
| `crates/ccr-codex/src/services/openai_quota_core.rs` | 真实获取时间、cache来源与窗口/bucket白名单元数据 |
| `crates/ccr-codex/src/services/codex_quota_service.rs` | 稳定账号query envelope、新观测保存入口；不扩大 auth 写入范围 |
| `crates/ccr-codex/src/models/codex_auth.rs` | additive 观测/归属/结果 DTO 与旧格式兼容 |
| `crates/ccr-codex/src/services/codex_usage_estimation.rs`（新） | 领域归属、纯价格/估算组合与质量状态 |
| `crates/ccr-codex/src/managers/codex_quota_observation.rs`（新） | 有限元数据文件、锁和原子存储；不创建通用账单框架 |
| `crates/ccr-types/src/model_rate_catalog.rs` | 明确的新模型价格与输入侧cache-write长上下文倍率 |
| `crates/ccr-tui/src/tui/codex_auth/app.rs` / `ui.rs` | 后台数据协调、局部 typed state 与双语呈现 |
| 对应 `mod.rs` / `lib.rs` 与现有测试模块 | 只导出实际需要的接口并验证兼容 |
| `.trellis/spec/ccr-codex/backend/backend-guidelines.md`、`.trellis/spec/ccr-tui/backend/backend-guidelines.md` | 实施后写回新可执行合同和验证边界 |

文件列表是授权边界，其他文件需要在实现前确认直接依赖理由。共享 OpenAiQuotaCore 的所有当前调用者、Root CLI quota JSON 输出和已有 catalog consumers 必须审查 additive DTO 影响；不更改业务页面或无关价格。删除 cache/观测文件只能由用户后续明确动作执行；实现回退为撤销代码与停用读取新缓存，保留用户原日志、registry、激活时间线和观测文件。

## D8. 验证与需求映射

| AC | 设计机制 | 验证 |
| --- | --- | --- |
| AC1 | D2 inclusive/互斥分类，D3单次计价 | flat/nested、显式0、缺失、异常桶、reasoning子集 |
| AC2 | D2 identity/epoch/coverage | mixed、copies、fork、rollback、time、重复刷新 |
| AC3 | D3 rate与逐请求阈值 | estimate-examples + 272000/272001 + cache写倍数 |
| AC4 | D3 provenance/partial/assumed | canonical、unknown、coverage、tier/context缺失 |
| AC5 | D1/D2归属边界 | A/B、外部检测、并发、globalfallback隔离 |
| AC6 | D4真实时间/有限原子文件 | TTL重复、rename、读写失败、权限、无敏感字段 |
| AC7 | D5 ratios/median/envelope | 3段合成算术及全显示链 |
| AC8 | D4/D5状态条件 | 对每个失效条件用独立fixture给确定结果 |
| AC9 | D2 UTC范围，D5联合约束 | 时区、未来值、same/different bucket |
| AC10 | D6 render/async generation | EN/ZH矩阵、slow loader、旧结果迟到 |
| AC11 | D2/D4/D7 additive compatibility | oldregistry/cache、original bytes不变 |
| AC12 | D8 + implement.md正式门槛 | 子系统、workspace、CI、native边界分别记状态 |

本次只核对规划算术与文件。实现测试结果不得预先填写 PASS。
