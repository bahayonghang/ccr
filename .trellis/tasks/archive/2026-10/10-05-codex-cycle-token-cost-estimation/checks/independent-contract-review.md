# 独立合同检查

日期：2026-10-06。角色：`trellis-check`。任务范围：本任务全部产品 diff、PRD R1–R8 / AC1–AC12、design D1–D7、implement.md、check.jsonl 的 9 项来源及相关 package indexes。

状态：最终源码的全范围静态检查已执行。正式 `just ci` 由主会话独占运行；本报告不会用阶段子系统结果代替正式门槛。以下检查正文保留代理交接时点的证据，最终完整 CI 和合成原生 PASS 见末尾“主会话最终验收补记”。真实账户、Linux/macOS 和 hosted CI 为 `NOT_RUN`。

## 上下文与边界

检查代理读取了保存的 native hook 输出，并补读任务 PRD/design/implement、check.jsonl、9 项 spec/research、相关 package indexes、code_map.md 和 crates/AGENTS.md。最终压缩后的 Codex/TUI 新合同已与源码核对。未修改价格、采样阈值、公共接口、原始 JSONL、registry、激活历史、llmusage SQL 或数据库。

M1–M3 的缺陷先报告主会话和领域实施代理，由领域实施代理修复。主会话明确授权检查代理自修 M4 的局部显示和消息保留缺陷。其他供应商 footer 行为保持原值。未提交或归档。

## 已修复发现

| 文件 / 路径 | 缺陷 | 修复与回归 |
| --- | --- | --- |
| `codex_auth/ui.rs::local_usage_lines`、`compact_usage_lines` | 窄布局可能丢失某窗口 remaining/N/A 原因；宽行超过内容宽度 | 按实际宽高选择紧凑/完整行；窄布局优先 3 个统计窗口和 2 个 remaining；宽布局拆分 Token/USD/样本行；显示折叠标签和显式截断 |
| `codex_auth/app.rs::usage_panel_data` | 成功 quota 的 history warning 和取得时刻未参与当前剩余估算 | 在 `for_display` 后调用领域 `with_quota`，保留 quota/本地统计并抑制不匹配或失败历史的当前 remaining |
| `codex_auth/ui.rs::usage_state_line` | 选择 B 后 B 加载失败，A 旧快照阻止 B 错误显示 | 错误优先呈现；不同账号快照继续不提供 B 统计；实际 cells 回归 |
| `codex_auth/app.rs::drain_task_messages` | 新增 is_ok guard 后 `unreachable!()` 出现在生产消息分支 | 单个 Quota 分支直接 match Result；不新增 panic |
| `codex_auth/ui.rs::local_usage_lines` | 旧快照刷新时没有明确 stale 标记 | 同账号保留值旁标 `stale/refreshing` / `旧快照/刷新中` |
| `codex_auth/ui.rs::compact_capacity` | 紧凑数值缺少估算标签 | 数值旁标 `est` / `估算` |
| `codex_auth/ui.rs::compact_capacity` | 主会话最终复核发现 Token 无效时提前 return，隐藏独立有效的 USD | 两轴独立格式化；仅同一无效原因合并短标签；USD-only 真实 composition 验证 Token Unstable/N/A 与有效 USD 同时可见 |
| `tui/ui.rs` 的 Codex Auth footer 分支 | 80×24 footer 的 q/Ctrl+L 被长操作提示挤出 | 仅 Codex Auth 的窄/短 footer 使用短提示；退出和语言键优先；其他平台路径保持原值 |
| `codex_auth/ui.rs` 的宽面板价格来源 | 混合新官方价和旧目录价时，单一 Verified 日期可能覆盖全部小计 | 日期和 URL 明确限定 gpt-6.1-sol；目录价独立标 `catalog estimate; date N/A`；新增 catalog/mixed 实际 composition fixtures |
| `codex_auth/ui.rs` 的相关面板标题 | API USD 未明确表示等值估价 | 显示 `API equivalent USD` / `API等值USD`；核心短行不加宽；六尺寸双语标题断言 |
| `codex_auth/app.rs::cache_quota_preview` | failed quota envelope 保留 quota/fetched_at，但丢失 observation，导致未过期旧估算误变 stale，history warning 丢失 | 同块保留 previous.observation；真消息路径覆盖 preview、failed quota envelope、outer error × preview/loaded cache × history warning 有/无；旧取得时刻+5分钟仍过期 |
| `codex_usage_service.rs` 的同 request cumulative 合并与 completed 替换 | chunks 未检查 account/speed/tier/bucket 冲突；completed 可抹去已经取得的 scope 负证据与 metadata | 非数值 metadata 单独补充并检查冲突，冲突产生 partial/scan diagnostics；completed 仍独占 Token/classification 数值；三组实际 parser/calibration fixture 通过 |
| `codex_usage_service.rs::tests::malformed_usage_objects_keep_scan_partial_without_fake_zero_records` | 新 schema4 fixture 持有 TestCodexEnv 时调用 scan_lines 再次获取相同非重入锁，默认并行测试阻塞 | 原始字节断言完成后显式 drop(env)，再创建 quota-only 隔离 fixture；保留首次阻塞日志，默认并行复测通过 |
| `codex_auth/ui.rs` 的 draw_usage_panel 旧标题断言 | 合同标题已改为 Quota / Local usage · API equivalent USD，旧断言仍要求 Quota remaining | 只更新标题断言；原 quota、Reset、7d、global scope 与归属原因断言保留 |
| `tui/ui.rs` 的 CatalogPrice 来源断言 | 禁止整个界面出现 2026-10-06，误匹配允许的价格版本 codex-api-equivalent-2026-10-06-v1 | 精确禁止 gpt-6.1-sol verified/官方来源行；仍要求 catalog estimate、date N/A 和全部数值/范围可见；未改产品定价或展示 |

领域实施代理已落实的主要合同发现：缺失分类使聚合值保持未知；fork 去重要求父事件证明；completed/cumulative/last 采用可证身份；真实 duration 映射窗口；reset countdown 不提供稳定 generation；当前 quota/history 与估算对齐；Token/USD 独立有效性及 joint；扫描缺口传入 typed cost Partial。具体首轮失败和领域复测保留于 domain-completion 日志。

## AC 到实际机制与回归

| AC | 源码机制与合成回归 | 静态结论 |
| --- | --- | --- |
| AC1 | `TokenUsage::parse` 验证 inclusive 输入/输出与子集；`add_to_stats`/`price_record` 只计 I+O、exclusive input；nested/alias/zero/missing/invalid fixtures | 符合；显式缺失不等于零 |
| AC2 | `parse_session_file` 请求/turn authority、单调累计 epoch、源副本身份；`scan` fork 父事件证明与 diagnostics；canonical TokenUsageRecord 仅取 request usage；顶层 compacted 切换未知 epoch；同 request chunks/completed metadata 补充与冲突回归 | 最后协议补丁已静态核对，parser 26 PASS、1 ignored；受影响结果保留 partial，校准拒绝歧义；完整门槛仍 PENDING |
| AC3 | canonical catalog + 逐记录 pricing；100 个短请求 $5.15/$10.30、272000/272001、cache-write/输出倍率回归 | 阈值按 inclusive 单请求输入；周期累计不触发阈值 |
| AC4 | cost status、record/Token 分母、context/tier assumption、source set；unknown/known zero/partial 回归；M4 CatalogPrice/MixedPriceSources/Unpriced/TinyCost | 保留小计和来源状态；最终新版展示需正式 gate 复测 |
| AC5 | `records_for_account` 与稳定 ID/activation timeline；GlobalFallback 不进入 load 校准；route/account/provider/session-start/bucket mismatch 验证 | 本地归属标 inferred；未扩大为服务端真实归属 |
| AC6 | network_acquired_at 固定；cache 保留时刻且不录新样本；metadata-only store；normalized operation lock + guarded leaf write，secret/None backup | 有 cache/rename/concurrent8/retention/rows/bytes/corrupt/unknown-field/native Windows ACL 与 deny-delete replacement 回归 |
| AC7 | 半开区间，完成扫描绑定；3 段 >=5pp，最近20段；median/min/max；10M/$5 范围与50%剩余 fixture | 阈值未变；经验范围没有称为置信区间 |
| AC8 | per-window status、分段变化、扫描/价格/范围独立门禁、raw percent、freshness/reset、max/min >2 | 每个无效状态有领域 fixture；Token/USD 独立；history 仅抑制当前 remaining |
| AC9 | fixed UTC rolling；joint 要求相同 bucket/workload，USD 还要求价格基准；较小剩余值；half-open 与 future 回归 | 未把两个不同 bucket 强制合并 |
| AC10 | 真正主 App draw，6尺寸×EN/ZH×18状态=216（含独立 Standard、assumed Standard、USD-only）；旧 scope/quota/error matrix；慢注入 loader、account+generation、latest queued selection | 216 composition focused PASS；断言各窗口数值/原因、费用与等值标题、来源、范围、退出/语言键；完整正式门槛仍 PENDING |
| AC11 | usage cache v4 可重建；旧 JSON 字段 serde default；registry service 无写账本迁移；原始 fixture 字节保留 | 未改原始 JSONL/activation history/llmusage/sub2api |
| AC12 | 默认并行、--skip export_bindings；子系统日志与独立 binding-generation 门槛分离 | 全模块/原生状态见主会话正式 receipt，不能由本静态检查推断 PASS |

## 跨层兼容与规范核对

- `CodexAccountQuota.observation` 使用 serde default 和 skip-none，旧 payload 缺少字段仍可读。成功 `fetched_at` 改为实际网络取得时刻；cache 不延长该时刻。
- Tauri quota 命令通过 `OpenJsonValueDto` 返回 JSON。前端 handwritten `CodexAccountQuota` 的宽松 validator 检查 account_name/fetched_at，允许 additive 字段；没有 CodexAccountQuota 的生成 TS 绑定需要手工绕过。正式 Tauri/独立 binding gate 仍需要执行。
- Tauri 用量 payload builder 手动保留既有 input/output/records/windows/by_model 字段。新分类/费用没有顺带进入 Tauri 业务页面。`codex_tray.rs` 仅 test literal 添加 Default 尾项。
- 新 Codex/TUI spec 与来源、unknown、partial、当前 quota reconciliation、后台 generation、实际大小降级、真实并行测试合同相符。包索引由主会话更新。相关新合同不要求改各客户端模板或工具加载入口。
- 新 history schema 拒绝未知字段，不包含 email、URL、token、response body、headers、prompts 或路径。读取失败和替换失败不删除旧文件。观测仍沿既有 quota 请求保存，没有新增周期联网。

## 本检查代理执行的验证

| 命令 / 日志 | 结果 | 限定 |
| --- | --- | --- |
| `cargo check -p ccr-tui --all-features` / `check-tui-typecheck-01.log` | PASS | M4 首轮修复源码 |
| focused composition / `check-tui-composition-01.log` | FIRST_FAILURE：1 PASS、1 FAIL | 中文 wide-char continuation cell 的 filler space 导致断言错误；保留日志 |
| focused composition / `check-tui-composition-02.log` | RETEST_PASS：2 PASS | 逐行断言先移除 filler space；当时为168 feature fixtures |
| full TUI / `check-tui-full-01.log` | FIRST_FAILURE：246 PASS、1 FAIL | 旧长CJK断言要求小预算显示 Top model；合同允许有标记折叠 |
| full TUI / `check-tui-full-02.log` | RETEST_PASS：247 PASS、0 FAIL | 更新为实际 cells 的核心统计/窗口原因/折叠/截断断言；包含 Result match 修复 |
| scoped rustfmt | PASS | 后续来源、等值标题、provenance 真消息测试和 est/stale 标签均已格式化 |
| `cargo test -p ccr-codex --all-features codex_usage_service::tests -- --skip export_bindings` / `final-parser-first-tests.log` | FIRST_FAILURE_INTERRUPTED：已复现 1 FAIL，随后因 fixture env 锁阻塞中断 | completed metadata 丢失；该运行未产生完整测试汇总，不能填全组 PASS/FAIL 计数 |
| 同 parser 命令 / `final-parser-retest.log` | RETEST_PASS：26 PASS、0 FAIL、1 ignored | 包含 canonical usage/compacted/invalid usage/duplicate metadata 与最后三组 chunks/completed 回归；默认并行 |
| `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings` / `final-parser-clippy.log` | PASS | 最后 parser 修复源码，包含 type-check；未与主会话并发 Cargo |
| `cargo test -p ccr-tui --all-features codex_auth_composed_feature_states_keep_cost_capacity_and_keys_visible -- --skip export_bindings` / `final-tui-matrix-retest.log` | PASS：1 test，内部216 composition fixtures | 最终18状态×6尺寸×2语言，包含 USD-only；原数值/范围/来源标准保留 |
| `cargo test -p ccr-tui --all-features -- --skip export_bindings` / `final-tui-full-retest.log` | PASS：247 PASS、0 FAIL | 最终 TUI 源码、默认并行；包含旧 quota/scope 与真消息保留、后台交互和216矩阵 |
| `cargo clippy -p ccr-tui --all-targets --all-features -- -D warnings` / `final-tui-clippy-retest.log` | PASS | 最终 TUI 源码，包含下游 type-check；未新增 allow 或修改门槛 |

`check-tui-full-02.log` 的 `247 PASS` 早于最后的来源/标题/provenance/est/stale/独立USD 修改；`final-tui-full-retest.log` 的 `247 PASS` 覆盖最终源码。子系统与216-fixture 已通过，跨模块最终验收以主会话完整 `just ci` 为准。本检查代理没有与领域或主会话并发运行 Cargo。

主会话 `just-ci-02.log` 保留正式首轮 Clippy 失败：`collapsible_if` 与新增测试 13 处 `unwrap_used`。检查代理已合并条件、逐项改为含具体 fixture 失败说明的 `expect`，未新增 allow 或放宽门槛，完成 scoped rustfmt。同步补充 required 显式 Standard fixture；复测由主会话下一轮完整 `just ci` 执行。

主会话 `just-ci-04.log` 正式退出1，保留 TUI 245 PASS、2 FAIL：旧标题断言和 CatalogPrice 全局日期断言。实际 cells 中 quota/金额/容量/目录来源可见，失败原因是测试期望语义；检查代理只更新两处断言，未修改产品显示、公式、采样阈值或价格来源。后续完整 `just ci` 仍需主会话重跑。

## 未修复 / 等待验证

- 最后协议补丁已复用既有路径补齐，并由本检查代理静态核对与 focused parser 执行。顶层 `compacted` 在任何 payload usage 前返回；`TokenUsageRecord{payload}` 仅取 request `usage` 与 response ID，忽略 turn/thread 累计量；缺 usage/非法 usage 产生 diagnostics。公开协议证据由主会话取得：[rollout_payload.rs](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/history/src/rollout_payload.rs)、[protocol.rs](https://github.com/openai/codex/blob/822e58cc3d666166c7446c5b1ea2e52f5d09594c/codex-rs/protocol/src/protocol.rs#L2243)。未新增正文持久化。
- 最后 M4 USD-only 实际 composition 静态核对与216 fixture执行完成：Token 与 USD 分别格式化；Token Unstable/N/A 与有效 USD $2.50/$1.00 同时可见；窄布局显示 est/估算，宽布局保留各轴状态。完整正式门槛等待主会话重跑。
- 最终 lint、type-check、workspace/Tauri/binding/full CI：主会话负责，当前 PENDING。
- 合成原生终端：主会话负责，当前 PREPARED_NOT_RUN。
- 真实账户、其他系统、hosted CI：NOT_RUN；未扩大授权进行这些验证。

当前静态复核没有未修复的已确认产品缺陷。216-fixture 已通过；正式跨模块门槛和原生 receipt 未完成前，不宣布最终验收通过。

## 主会话最终验收补记

2026-10-06，主会话执行第六轮完整 `just ci`，退出 0，16 项步骤全部通过，用时 12:30.169。最终源码覆盖领域 272 PASS/2 ignored、类型 47 PASS、TUI 247 PASS、216 composition、Tauri 407 PASS/1 ignored、独立 binding gate、前端 904 tests、覆盖率和 VS Code 检查。前述正文 PENDING 状态为交接时点记录，最终正式结果见 [verification.md](verification.md) 和 [just-ci-06.log](just-ci-06.log)。

主会话的 Windows 合成原生终端检查为 PASS：实际 140×40、100×22，Token/费用显示及 Ctrl+L/r/q，两个进程均退出 0。第六轮 Release Build 的 CLI SHA-256 与原生测试记录一致，见 [native-verification.md](native-verification.md)。AC1–AC12 全部完成；真实账户、Linux/macOS 和 hosted CI 保持 NOT_RUN。检查代理未运行主会话的完整 CI 或原生程序。
