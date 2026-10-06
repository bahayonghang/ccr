# 优化 Codex Auth 本地用量统计面板显示

## Goal

重构 Codex Auth 右下角 "Local usage · API equivalent USD · Tokens / Records" 面板：建立清晰信息层级，消除冗余与全 N/A 噪声，同时保持错误、范围、partial/unpriced 语义与双语、尺寸矩阵约束。仅改呈现层，不改领域数据与加载逻辑。

## Authorization and Status

2026-10-06 用户报告"信息全部堆到一起、有许多冗余无用的信息"并要求优化。本版为规划产物，实施需评审后 `task.py start`。

## Background and Confirmed Facts

见 research/panel-inventory.md。现状（ui.rs local_usage_lines:1477 起）：budget>=22 时全量渲染约 20 行——表格 3 行 + 计价基础 1 行 + 分类 4 行 + 容量 9 行 + 价格来源 2 行 + URL 1 行 + 主要模型 1 行 + 备注 1 行。用户截图显示：状态词逐行重复、Read/Write/Reason 全 N/A 占行、容量区样本 0/0 仍占 9 行、URL 截断无意义。

## Requirements

| ID | 要求 |
| --- | --- |
| R1 | 主区：窗口表（5h/7d/累计 × Tokens/记录/API 等值 USD）为第一优先；状态与计价模式以单次图例/徽标呈现（徽标取各窗口最严重状态，规则见设计），不再逐行重复。 |
| R2 | 裁剪：全 N/A 字段/行不占行（分类缺失列、样本不足的容量明细）；样本不足的容量区折叠为一行明确状态；价格来源合并为一行（版本+日期）；URL 行仅在面板内宽足以完整显示时渲染（阈值按内宽计，见设计），任何尺寸不出现截断 URL。 |
| R3 | 保留语义：用量错误/历史错误优先显示；范围（归属推断/全局）与 partial/unpriced 状态在全部支持尺寸保持可见（10-05 AC10 不回归）；联合剩余量（10-05 R6，取较小值）保留在容量区；大尺寸下完整明细仍可获得且组织有序。 |
| R4 | 双语与尺寸：EN/ZH 同步更新；TestBackend 80×24、100×22、100×30、120×22、140×40、180×50 矩阵通过；不新增快捷键。 |
| R5 | 仅呈现层：不改 ccr-codex 领域服务、加载/刷新逻辑、usage_panel_data 数据契约（确需新增展示字段须最小且向后兼容）。 |

## Acceptance Criteria

- [x] AC1（R1）：表格行不再重复状态词；同状态文本在面板出现 ≤1 处（图例或徽标）。
- [x] AC2（R2）：样本不足场景（截图 k12/khanh 等价 fixture）容量区 ≤1 行；无全 N/A 行；URL 行仅在内宽足够时完整出现（180×50 可出现），任何尺寸无截断 URL。
- [x] AC3（R3）：错误、范围、partial/unpriced 在全部矩阵尺寸可见；联合剩余行存在（容量区，10-05 R6）；180×50 可查看完整明细（分类/容量含联合剩余/来源）。
- [x] AC4（R4）：EN/ZH 矩阵与快照测试通过；`cargo test -p ccr-tui` 与组合检查（composition）通过。
- [x] AC5（R5）：diff 仅限 ccr-tui（及必要的 ccr-types 纯展示辅助）；usage_panel_data 契约未变或仅增可选字段。
- [x] AC6：`just lint-strict`、`just test` 通过。

## Out of Scope

- 数据采集/估算算法（10-05 已完成）；配额卡（上半部分）重构；其他 Tab。

## Risks and Evidence Boundaries

- ui.rs 现有测试断言具体字符串，需同步更新；不得因此降低语义覆盖。
- 窄屏取舍需保留"空间不足"提示语义；截断行为（clipped_line）保持不变。
