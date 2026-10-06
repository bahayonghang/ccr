# M1–M3 领域补齐与验证记录

日期：2026-10-06。范围：`crates/ccr-codex/**`、`crates/ccr-types/src/model_rate_catalog.rs`。

状态：M1–M3 合同补齐完成。历史领域全包子系统测试为 260 PASS、0 FAIL、2 ignored。该次测试之后的 scan-to-cost partial、schema4 协议与同 request metadata 修复仍需主会话完整 `just ci`；最后 parser focused 已通过 26 PASS、0 FAIL、1 ignored 与严格 Clippy。本记录不代替正式门槛。

此前 `domain-recovery-implementation.md` 的 225 PASS 是历史阶段证据。该阶段只有少量 parser、estimator 和 observation 测试，不能代表完整 AC 覆盖。本轮保留历史日志，补齐合同与合成回归。

## 产品改动

本轮恢复修改五个领域文件，并格式化两个导出文件：

- `crates/ccr-codex/src/services/codex_usage_service.rs`
- `crates/ccr-codex/src/services/codex_usage_estimation.rs`
- `crates/ccr-codex/src/services/codex_quota_service.rs`
- `crates/ccr-codex/src/services/openai_quota_core.rs`
- `crates/ccr-codex/src/managers/codex_quota_observation.rs`
- `crates/ccr-codex/src/lib.rs`（仅格式）
- `crates/ccr-codex/src/services/mod.rs`（仅格式）

本轮保留前序实现的共享价格目录、模块导出、registry 兼容字段和 TUI 修改。主会话与 TUI checker 管理各自范围。

M1：usage cache 更新为 schema 4；旧 schema 从原始 JSONL 重建。嵌套分类优先，显式零保留；任一 included record 缺少分类时，聚合分类保持未知。request/response/turn 身份决定权威记录；已知 parent event 证明复制前缀后才排除。counter 回退、恢复、model/speed 切换、last-only 无身份、无法关联的 cumulative/completion 混合均保留 partial 诊断。缺失时间记录保留推断标签，滚动统计排除无有效事件时间的记录；未来时间不进入当前滚动统计。

M1 最后协议补丁：顶层 `token_usage_record` 使用逐响应的 `payload.usage` 和 response ID，忽略 turn/thread 累计值。顶层 `compacted` 与既有 context_compacted 均建立未知累计 epoch，不读取 compaction 正文。显式缺失或非法 usage 生成 scan diagnostics，不制造零值完整记录。同 request metadata conflict/supplement 在 duplicate、counter chunks 和 completed 替换时保留；completed 独占 Token/classification 数值，已有 scope 负证据不能被覆盖。

M1：逐请求价格保留 USD、模型命中、价格来源与版本、context/speed 假设、定价小计和已解析覆盖分母。已估值零与 Unpriced 分开。scan 不完整时，各滚动窗与 by_model 的 typed cost 同步 partial_usage；已有可定价小计转 Partial，Unpriced 保留。原始 JSONL、registry 激活历史与 llmusage 不修改。

M2：quota cache 保留真实 network acquisition，cache-hit 不增加观测。观测仅保存白名单元数据；稳定账户 ID、bucket、duration、role 和 acquisition 决定 identity。35 天、4096 条、8 MiB 限制保留。读改写使用标准化路径锁与现有私有原子写。history 失败保留旧完整字节并将 warning 附加到成功 quota。

M2：可见 5h/7d 窗口按实际 duration 选取，支持 primary/secondary 互换；缺失或无效 used_percent 不显示为满额。reset_generation 只接受稳定 server reset_at；仅 reset_after 时不构造周期身份。

M3：独立 Token/USD 质量、最多 20 个非重叠区间、最少 3 个区间、每段至少 5 个百分点、中值与经验 min/max、max/min 大于 2 的不稳定状态。reset/plan/bucket/window/model/tier 边界重启采样。当前容量受 acquisition+5min 与 reset 到期共同约束。`with_quota` 对齐当前 quota provenance：新获取时间、unknown window、history warning 或账号范围失配时隐藏当前剩余容量，保留本地统计与历史总量。

## 合成合同覆盖

| AC | 领域 fixture 与判定 |
| --- | --- |
| AC1 | inclusive input/output/cache/reasoning；nested/top-level alias；零、缺失、异常桶；optional 聚合完整性与已估值零。 |
| AC2 | mixed request chunks/completed/turn authority；重复 last/cumulative/completed；同 session 副本；相同数值独立请求；有证明及无证明 fork prefix；counter rollback/restoration/model/epoch；last-only 有身份及无身份；无关联 mixed 记录标 partial；重读与 cache 重建；missing/invalid/future timestamp。 |
| AC3 | 100 次短上下文请求 Standard $5.15 / Fast $10.30；272000/272001 inclusive-cache 边界；cache-write 长上下文倍率；reasoning 属于 output；累计大于阈值不触发长上下文价格。 |
| AC4 | verified gpt-6.1-sol 与 catalog_estimate 来源；未知模型 Unpriced；缺分类/请求长度/speed 假设；费用小计及 record/token coverage；scan malformed/unreadable 传到 typed cost partial 的最终补丁等待正式门槛。 |
| AC5 | A/B 激活时间线、首激活前记录、unattributed/global fallback、旧 concurrent session、账号/路由/bucket 负证据；已知 API/custom route 抑制容量，保留本地量。 |
| AC6 | network/cache provenance、cache 不新增、稳定 ID rename、写失败不抹成功 quota；35d/4096/8MiB；8 并发 record 不丢 distinct samples；unknown credential 字段旧文件拒绝且 bytes 保留；Windows 创建/替换 ACL 和真实 replacement failure。 |
| AC7 | 300 个独立实际计价请求、3 段有效 dp10pp，结果 10M/$5 中值、8–12M/$4–6 范围、50% remaining 5M/$2.50；采样数与 span 精确。 |
| AC8 | 样本不足、dp0/负值/小于5、无本地量、scan 不完整、unknown source/bucket/reset/window、过期、状态边界；Token/USD 独立缺价与不稳定；重复 acquisition；20 段上限。 |
| AC9 | freshness/reset 与 joint 独立；相同负载/bucket 取较小剩余量，不同 bucket/价格基准不合成；UTC 半开区间和未来记录；迟到日志重新投影观测区间。 |
| AC11 | schema2 旧 usage cache 重建；原始 JSONL 字节不变；registry 序列化不变；未知观察字段拒绝保留旧文件；未访问或迁移 llmusage/sub2api。 |

关键测试入口为 `inclusive_nested_aliases_zero_missing_and_invalid_classification`、`mixed_request_authority_nested_response_and_partial_turns`、`fork_prefix_requires_parent_event_proof`、`last_only_snapshots_need_identity_and_completed_event_wins`、`uncorrelated_counter_and_request_completion_are_partial`、`scan_gaps_mark_cost_subtotals_partial_without_inventing_coverage`、`independent_synthetic_requests_produce_full_token_usd_envelope`、`invalid_sampling_conditions_have_determined_states`、`quota_write_warning_and_new_acquisition_hide_only_current_capacity`、`late_log_arrival_recomputes_bound_observation_intervals`。

AC7 使用每请求 I=10000、R=8000、W=400、O=0、Q=0，真实目录计算每请求 $0.005。100/120/80 次请求得到 1M/1.2M/0.8M Token 和 $0.50/$0.60/$0.40。每段除以 10% 后得到 10M/12M/8M 与 $5/$6/$4，未注入虚构价格。

## 检查与失败保留

均使用默认测试并行和独立 binding 导出约定，无 `--test-threads=1`。

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | 首次 FAIL：256 PASS、2 FAIL、1 ignored | `domain-completion-first-test.log` |
| 同命令修复后重测 | PASS：258 PASS、0 FAIL、1 ignored | `domain-completion-retest.log` |
| `cargo test -p ccr-types --all-features -- --skip export_bindings` | PASS：47 PASS | `domain-completion-types-test.log` |
| `cargo clippy -p ccr-codex -p ccr-types --all-targets --all-features -- -D warnings` | 首次 FAIL：1 个生产分支重复与 3 个 test default field reassignment | `domain-completion-first-clippy.log` |
| 同 clippy 修复后重测 | PASS | `domain-completion-clippy-retest.log` |
| 领域最终全包 test 同上述命令 | PASS：260 PASS、0 FAIL、2 ignored | `domain-completion-final-tests.log` |
| 最后 parser focused test：`cargo test -p ccr-codex --all-features codex_usage_service::tests -- --skip export_bindings` | FIRST_FAILURE_INTERRUPTED：completed metadata fixture FAIL；随后 nested TestCodexEnv 锁阻塞，中断无完整汇总 | `final-parser-first-tests.log` |
| 同 parser focused 命令修复后重测 | PASS：26 PASS、0 FAIL、1 ignored | `final-parser-retest.log` |
| `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings` | PASS：最后 parser 源码 | `final-parser-clippy.log` |

首次 test 的两个失败原因：正常 config TOML 使用 Value FromStr 被误拒；duplicate acquisition 未先 exact dedup 导致边界样本归零。分别改为 `toml::from_str::<toml::Value>`、先 exact dedup 后分组。失败日志完整保留，未修改阈值或删除失败 fixture。

最终两项 ignored 均为既有 benchmark：session inventory benchmark 与本轮恢复的 `benchmark_compute_rolling_usage_cache`。后者在早期 parser 替换中丢失，本轮恢复合成隔离 fixture 和 ignored 属性。两项未计入功能 PASS。

最后 full test 后新增的 scan-to-cost partial 补丁已 scoped rustfmt。主会话要求直接由正式 `just ci` 覆盖，未重复执行窄 Cargo。该补丁及最后 parser 补丁的最终严格 lint 以主会话 gate 日志为准。

最后检查代理复核同 request 两条分支时新增三组实际 parser/calibration 回归：account/speed/tier/bucket 冲突导致 partial、counter metadata 补充可见、completed 仅接管数值并保留 scope 证据。首次运行另发现 malformed-usage fixture 持有 TestCodexEnv 后调用 scan_lines 重入非重入锁；完成原字节断言后显式释放旧 fixture，默认并行重测通过。未增加串行参数、ignore 或 allow；原始失败与阻塞日志保留。主会话完整 gate 仍 PENDING。

主会话首次正式 `just ci` 停在 fmt-check，指出 `lib.rs` 与 `services/mod.rs` 的新导出/module 顺序未格式化。按主会话要求仅对这两个文件运行 rustfmt，使用 `skip_children=true`；未单独运行检查，重测由主会话记录。

## 证据边界

Windows 原生文件验证已在最终领域测试执行：`windows_sharing_denial_keeps_old_complete_history` 使用 deny-delete sharing handle 触发真正的替换失败，旧文件字节相同；释放句柄后正常追加。`windows_create_replace_uses_private_acl_without_exposing_sid` 通过 pwsh `Get-Acl` 验证新建文件关闭继承且仅有当前用户 ACL，替换前后 Access SDDL SHA256 相同；测试不输出 SID、用户名或个人路径。

配额/会话内容均为 synthetic 临时 fixture。真实账号、真实 endpoint、个人 auth/session、其他设备与其他操作系统：NOT_RUN。原生 TUI、TestBackend 完整矩阵和正式 `just ci` 由主会话/TUI checker 独立记录；本记录不宣称这些门槛完成。未提交、归档或推送。
