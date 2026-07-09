# TUI profile 页面体验修复（CJK 截断、快捷键冗余、标签语义）

## Goal

修复 Codex/Claude profile 页面经审查确认的四个体验问题：CJK 显示宽度截断 bug、
快捷键提示三重冗余、`usage_count` 标签语义混淆、Focus 面板与 Context 信息重复。
问题清单与审查依据见父任务
`.trellis/tasks/07-06-tui-profile-optimization/prd.md`。

## Confirmed Facts

- **CJK 截断 bug**（`crates/ccr-tui/src/tui/ui.rs`）：`truncate_text`、
  `pad_text`、`column_widths` 全部按 `chars().count()` 计数；CJK 字符终端宽度
  为 2，含中文的 name/description 单元格实际渲染宽度超出列宽，被 ratatui 在
  面板边缘硬裁剪——省略号丢失、右侧描述列错位（用户截图中 anyrouter4 描述断在
  半个单词处）。`usage/ui.rs` 的 `truncate` 同病。ratatui 依赖树已含
  `unicode-width`。
- **快捷键三重冗余**（Wide 视口）：同屏出现三份近似快捷键提示——
  1) Selection 面板（`profile_meta_strings`）的 Legend + `Enter apply · r
     reload · Tab/Shift+Tab switch` 行；
  2) 右下 Status strip（`render_profile_status_strip` → `footer_text`）；
  3) 底部全局 Keys footer（`render_footer`）。
- **`usage_count` 语义**：Activity 分组的 `usage_count` 实为 profile 切换次数
  （`increment_usage` 在 apply 时 +1），子任务 A 落地后与真实用量统计并存，
  易误读。
- **Focus/Context 重复**：Focus 块（`profile_summary_strings`）的
  Name/Status/Description/Model/Base URL 在 Context 的 Overview/Engine 分组
  全部重复出现，Wide 视口右栏 7 行固定高度花在重复信息上。

## Requirements

- **R1 显示宽度截断**：`truncate_text`/`pad_text`/`column_widths` 改按
  `unicode-width` 的显示宽度计算；截断永远以 `…` 结尾且单元格不溢出列宽；
  `usage/ui.rs::truncate`（若子任务 A 已下沉则在其新位置）同步修复。
- **R2 提示去冗余**：快捷键提示只保留底部全局 Keys footer 一处；
  - Selection 面板去掉 keys 行，保留 Selected/Profiles/Legend 选择状态；
  - Status strip 只显示最近一次 apply 结果（无结果时显示 toast 或留空），不再
    重复完整快捷键列表。
- **R3 标签语义**：详情 Activity 分组 `usage_count` 显示标签改为
  `switch_count`（仅显示层改名，`ProfileConfig` 字段与存储不动）。
- **R4 Focus 去重**：Focus 块收敛为不与 Context 重复的行——保留
  `Name + Status`（选中态/当前态一眼可见）与最近 apply 结果，移除
  Description/Model/Base URL 三行；Wide 视口 Focus 高度相应收缩，让出的行给
  Context 详情。
- 全程遵守 per-tab selection 与分页助手红线（backend-guidelines），不动
  选择/分页逻辑。

## Acceptance Criteria

- [x] 截断单测：混合 CJK/ASCII 样本在给定宽度下产出的字符串显示宽度
      ≤ 列宽且以 `…` 结尾；纯 ASCII 行为与现状一致（回归）。
- [x] `column_widths` 单测更新后通过；含中文描述的列表行手工冒烟不再溢出。
- [x] Wide 视口渲染测试/单测断言：快捷键文案（如 "Enter apply"）在
      Selection 面板与 Status strip 的输出行中不再出现，仅存在于 footer。
- [x] 详情行单测：Activity 分组显示 `switch_count`，不再出现 `usage_count`
      标签。
- [x] Focus 块单测：不含 Description/Model/Base URL 行；Name/Status/最近
      apply 结果保留。
- [x] `cargo test -p ccr-tui -- --test-threads=1`、`just fmt-check`、
      `just lint-strict` 全绿。

## Notes（实现期修正与说明）

- `column_widths` 经核对从未按字符计数（纯数值列宽分配），Confirmed Facts
  该点表述不准确；本次未改动该函数，原单测原样通过。实际按字符计数的是
  `truncate_text`/`pad_text`（ui.rs）与 `truncate`（usage/ui.rs），均已改为
  unicode-width 显示宽度。
- AC2 的"手工冒烟"以 TestBackend 渲染断言自动化
  （`profile_list_row_with_cjk_description_stays_within_column_budget` +
  `wide_profile_draw_shows_shortcuts_only_in_global_footer`）；真机终端下的
  眼验留给用户在 wrap-up 前顺手确认。
- Focus 高度未采用 implement.md 的固定 7→4，而是随内容自适应
  （行数 + 边框 = 4~5，三种视口统一）：有 apply 结果时为 5，避免第三行被
  固定高度裁掉；Compact 视口 Context 面板行数不少于改动前。
- usage `truncate` 在 width=0 时返回 `…`（宽 1）为存量行为，生产调用只传
  固定 18/24，按 surgical-change 原则未动。
- trellis-check 复查结论 approve（零修复），红线核对详见其报告。

## Out of Scope

- 列表搜索/过滤、Tab 改名、时间窗（父任务延后候选）。
- auth 子应用（Claude/Codex/OpenCode Auth tab）与 Usage 相关渲染（子任务 A
  范围）。
- 鼠标 hit-testing 与分页行为变更。
