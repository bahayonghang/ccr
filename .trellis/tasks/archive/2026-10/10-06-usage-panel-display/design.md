# 设计：用量面板信息层级重构

## 目标布局（Wide 右列下卡）

主区（恒定，budget>=22 与 compact 均保留核心语义）：

```
 Local usage · API equivalent USD · partial
 Window   Tokens    Records   API USD
 5h       47.5M     107       $12.56
 7d       897.9M    6K        $39.96
 All      2.2B      14.9K     $39.96
```

- 状态徽标（partial / Std assumed / unpriced）仅在标题行出现一次，取各窗口最严重状态（unpriced > partial > Std assumed > priced）；表格行内不再重复状态词；无颜色终端由徽标文本单独表达状态。
- unpriced/partial 行保留 warning 行色（颜色语义不裁剪）。

次区（按数据可用性与空间裁剪，优先序 1→4）：

1. 容量区：任一窗口 LocalEstimate → 每窗口一行 `5h est 10M–12M tok / $4–$6 · n=3`；样本不足且两窗口同状态 → 合并单行 `Capacity: samples<3`；保留联合剩余一行 `Joint remaining ≈ … / …（状态）`（10-05 R6）；LocalEstimate 时保留免责声明一行。
2. 分类区：仅渲染有值字段（如 `In 47.3M · cache R 1.2M / W 0.5M · Out 158.3K · reason 90K`）；全 N/A → 整区省略（含说明行）。
3. 价格与来源合并一行：`codex-api-equivalent-2026-10-06-v1 · verified 2026-10-06`（tier 状态只在标题徽标出现，不重复）；URL 行仅当面板内宽 >= 行完整宽度（EN 77 列，实现按行实际宽度判断；180×50 内宽约 88 满足）时单独一行，任何尺寸不出现截断 URL。
4. Top model 一行；fallback_reason 一行（保留原文案语义）。

压缩规则表：

| 条件 | 处理 |
| --- | --- |
| 字段全 N/A（Read/Write/Reason） | 字段不出现 |
| 5h/7d 容量状态相同且非 LocalEstimate | 合并一行状态 |
| 样本=0 | 不显示 Samples/span |
| URL 行完整宽度 > 面板内宽 | 不渲染 URL 行（不截断） |
| 联合剩余（joint） | 始终保留一行（10-05 R6） |
| 表格行内状态词 | 提升为标题徽标，行内去除 |
| budget<22（compact 路径） | 保持 compact 语义，同步去除重复状态词 |

## 语义保留（不可裁剪）

- usage_error / history_warning / stale-refreshing 前缀（错误优先，含在标题行前）。
- scope 行（AccountAttributed / Global + inferred / not selected）。
- partial / unpriced 可见（徽标或行色）。
- 容量区显示时的免责声明；折叠时并入状态行。
- 「空间不足」提示语义不变；clipped_line 截断行为不变。

## 实现边界

- 改动集中在 ui.rs：local_usage_lines（:1477）及行构建函数（usage_table_lines :1277、price_basis_line :1329、token_classification_lines :1037、capacity_lines :1345、来源行、top_model_line、usage_note_line :1425）；app.rs 默认不动（确需展示辅助字段时仅增可选字段）。
- 测试：更新 UsagePresentationCase 期望；新增冗余断言（状态词计数、N/A 行数、URL 宽度条件、矩阵尺寸）。
- EN/ZH 字符串同步（tui_text!/tui_format!）。

## 取舍

- 分类区不再固定 4 行：大终端信息密度下降，换取可读性；180×50 仍可完整展开。
- 样本不足时容量区 9 行 → 2 行：逐窗口明细折叠为状态行，联合剩余保留（10-05 R6），消除噪声；LocalEstimate 时保留逐窗口行。
