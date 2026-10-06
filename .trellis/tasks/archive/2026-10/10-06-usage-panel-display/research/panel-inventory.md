# 本地用量面板行清单与冗余分析

基线：dev @ 797c1877。锚点为 2026-10-06 实际读取位置。

## 渲染入口

- draw_local_usage_panel（crates/ccr-tui/src/tui/codex_auth/ui.rs:1638）→ local_usage_lines（:1477）按 budget 分支：full = budget>=22；inline_capacity = budget<6；其余 compact。
- Wide 布局右列（draw_embedded :1823/:1848）：上为 Account & quota，下为本面板；Combined/Compact 走 compact 行。

## full 模式行序列（budget>=22）

1. usage_scope_line（:1067）"partial · Local: account X · inferred"
2. usage_table_lines（:1277）表头 + 3 行："5h/7d/All time | tokens | records | API $xx partial/Std assumed"
3. price_basis_line（:1329）"Std assumed · records n/m; tokens a/b priced"
4. token_classification_lines（:1037）4 行：说明行 + "5h In .. Read N/A Write N/A Out .. Reason N/A" ×3
5. capacity_lines（:1345）9 行：标题 + 每窗口 3 行（Token total/rem、USD total/rem、Samples/span）+ Joint remaining + 免责声明
6. 价格行："Price codex-api-equivalent-... · context assumed"
7. 来源行："gpt-6.1-sol verified ..."（pricing_sources 含 official_verified 时）
8. URL 行："gpt-6.1-sol source: https://..."（cost.source_url）
9. top_model_line "Top model: ..."
10. usage_note_line（:1425）fallback_reason（如 "Excludes other accounts and unattributed records: 702 / 0"）

## 冗余与无效信息（用户截图 2026-10-06 证据）

- 每行重复 "partial/Std assumed"（表格 3 次 + 基础行 1 次）。
- Read/Write/Reason 全 N/A 仍占 3 行 + 1 行说明；说明行与 N/A 列互相矛盾观感。
- 容量区样本 0/0 时仍渲染 9 行全 N/A（两张截图均如此）。
- "Price ... · context assumed"、"gpt-6.1-sol verified"、"gpt-6.1-sol source: URL"、"Top model" 四行元数据；URL 被截断且终端不可点击。
- "Samples Token/USD 0/0 · span N/A"、"scope" 后缀等占位词。

## 数据来源（app.rs）

- CodexAuthUsagePanelData（crates/ccr-tui/src/tui/codex_auth/app.rs:64）：scope / attribution_state / rolling / top_model / fallback_reason / estimate。
- usage_panel_data（:559）组装；rolling 含 5h/7d/all_time（CodexRollingUsage）；cost 含 status / tier_basis / subtotal_usd / priced_records / total_records / price_version / context_assumption / pricing_sources / verified_date / source_url。
- estimate 含 5h/7d/joint（CodexCapacityEstimate：status / usd_status / token_total / token_remaining / usd_total / usd_remaining / sample_count / usd_sample_count / span_start / span_end）与 diagnostics / history_warning。
- 语义状态：CodexEstimateStatus（LocalEstimate/InsufficientSamples/PartialUsage/InvalidScope/UnsupportedWindow/ResetChanged/Stale/Unpriced/Unstable/HistoryError/UnexplainedQuotaChange）与 CodexCostStatus（Unpriced/Partial/AssumedStandard/CompletePriced）。

## 现有测试与约束

- ui.rs 测试（:2205 起）构造 UsagePresentationCase 断言呈现字符串。
- 10-05 任务 AC10 约束：quota/错误/范围/操作在 80×24、100×22、100×30、120×22、140×40、180×50 矩阵可见；统计降级有标签；Ctrl+L 不重置选择或后台状态。
- 截断：clipped_line（:799）按宽度截断并保留样式。
