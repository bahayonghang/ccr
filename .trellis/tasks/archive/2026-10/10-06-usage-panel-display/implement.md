# 实施计划：用量面板重构

## 步骤（按序）

1. 对照 research/panel-inventory.md 复核 ui.rs:1477-1636 与 app.rs:559-631 行/字段映射（有出入先更新 research）。
2. 重构 local_usage_lines：标题徽标（状态单次出现）、表格行内状态词去除、次区按压缩规则表裁剪。
3. 重构行构建函数：capacity_lines 折叠逻辑（保留联合剩余行）、token_classification_lines 有值字段渲染、price_basis_line 与来源行合并（去 tier 重复）、URL 内宽条件（不截断）。
4. 更新 UsagePresentationCase 测试期望；新增断言：状态词出现次数 ≤1、无全 N/A 行、URL 内宽条件（无截断 URL）、样本不足容量区 ≤1 行、联合剩余行存在。
5. EN/ZH 文案同步；矩阵尺寸（80×24、100×22、100×30、120×22、140×40、180×50）逐一核对 10-05 AC10 可见性清单。
6. 组合检查：draw_embedded Wide/Compact、draw_combined_panel 不回归。

## 验证命令

```bash
cargo test -p ccr-tui --all-features
just lint-strict
just test
```

## 评审门

- 对照本任务 AC1-AC6 与 10-05 AC10 逐项确认。
- 确认 diff 仅限 ccr-tui（必要时含 ccr-types 纯展示辅助）。
