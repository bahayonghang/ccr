# TUI profile 界面优化（父任务）

## Goal

把 `ccr tui` 的 profile 使用体验修到位：用量统计跟随 profile 而不是独立页面、
详情面板可核对 key（掩码显示前后各若干位）、并修复 Codex/Claude profile 页面
经分析确认的体验问题。

用户价值：切换 profile 时在同一视图内看到该 profile（provider 维度）的用量与
成本；核对 key 无需打开 profiles.toml；列表/详情信息不再被 CJK 截断 bug 和
三重快捷键提示干扰。

## Source Requirements（用户原始三点）

1. TUI 中的统计应跟随 profile，而不是单独一个 Usage 页面。
2. TUI 详情中要显示 key 的前后各若干个字母，方便核对。
3. 结合相关 skills 分析 Codex/Claude 等 profile 页面还存在的问题并修复。

## Task Map

| 子任务 | 交付物 | 复杂度 |
|--------|--------|--------|
| `07-06-tui-usage-follow-profile` | profile 详情面板内嵌 provider 用量区块；下线独立 Usage tab（含 `TuiTabId` 配置兼容） | 复杂（prd+design+implement） |
| `07-06-tui-key-masked-display` | 详情 `token` 行显示 `configured (xxxx...yyyy)` 掩码；Codex `token` 行归位 Routing/Auth 分组 | 轻量（PRD-only） |
| `07-06-tui-profile-page-polish` | CJK 显示宽度截断修复、快捷键提示去冗余、`usage_count` 标签语义澄清、Focus 面板去重 | 中等（prd+implement） |

执行顺序：B（key 掩码）→ C（页面修复）→ A（用量跟随 profile）。B 与 C 都改
`ui.rs` 详情行构造函数，须串行避免冲突；A 改动面最大，建议最后合入。三个子任务
各自独立可验收。

## 分析结论（需求 3 的产出，结合 skills 审查）

审查依据：`.trellis/spec/ccr-tui/backend/backend-guidelines.md`（synthetic tab
契约、异步加载、per-tab selection）、`make-interfaces-feel-better` skill 的可迁移
原则（截断需带省略号、数字对齐、冗余合并、分组一致性）、
`rust-best-practices`（seam 注入、状态机测试）。

已确认问题（进入子任务 scope）：

- **P0 CJK 截断 bug**：`ui.rs` 的 `truncate_text`/`pad_text`/`column_widths` 按
  `chars().count()` 计数，CJK 字符终端显示宽度为 2，含中文描述的行溢出被
  ratatui 硬裁剪，省略号丢失（截图中 anyrouter4 描述断在 "cat"）。→ 子任务 C
- **P1 token 行分组不一致**：Codex 详情把 `token` 放在 Activity 分组末尾
  （`ui.rs` codex_profile_detail_lines），Claude 放在 Routing/Auth；语义上属于
  Routing/Auth。→ 子任务 B（与掩码显示同一处代码）
- **P1 快捷键提示三重冗余**（Wide 模式）：Selection 面板 legend+keys 行、右下
  Status strip、底部 Keys footer 展示几乎相同的快捷键，占用 ~5 行。→ 子任务 C
- **P1 `usage_count` 语义混淆**：Activity 分组的 `usage_count` 是 CCR 切换次数，
  与新增的用量统计（tokens/cost）易混淆。→ 子任务 C（改显示标签）
- **P2 Focus 与 Context 信息重复**：Focus 面板的 Name/Status/Description/Model/
  Base URL 全部在 Context 的 Overview/Engine 分组重复出现。→ 子任务 C
- **P2 Usage tab 数据全部 unattributed**：历史用量无 provider 归因（provider
  激活日志 2026-07 才引入），独立页面价值低，佐证需求 1 的方向。→ 子任务 A

延后候选（本轮不做，需要用户单独决策）：

- profile 列表增量搜索/过滤（`/` 键）：19 个 profile 翻页查找低效，但引入输入
  模式状态机，属新功能。
- Tab 命名一致性："Claude Code" vs "Codex Profile"（一个产品名、一个平台+词缀）；
  改名涉及 `platform.display_name()`、compact 映射与既有用户习惯。
- 用量时间窗（今日/7 天/30 天）切换：子任务 A 的 MVP 为 all-time，窗口切换留待
  归因数据积累后评估。

## Requirements

- 三个子任务的需求详见各自 `prd.md`；本任务不承担直接实现工作。
- 跨子任务约束：
  - 任何界面输出不得出现完整 auth_token/api key；掩码一律复用
    `ccr_core::mask_sensitive`，不新增第二套掩码策略。
  - TUI 渲染循环不得被文件系统/SQLite 阻塞（沿用 `AsyncTaskExecutor::spawn_blocking`
    + 消息通道模式）。
  - 用量 SQL 只允许存在于 `crates/ccr-usage`（llmusage 适配器契约红线）。

## Acceptance Criteria

- [ ] 三个子任务各自的验收标准全部通过并归档。
- [ ] 集成检查：`cargo test -p ccr-tui -- --test-threads=1`、
      `cargo test -p ccr-config -- --test-threads=1`、`just fmt-check`、
      `just lint-strict` 全绿。
- [ ] 手工冒烟：`ccr tui` 打开后 Tab 循环无 Usage 独立页；Claude/Codex profile
      详情含掩码 token 行与 provider 用量区块；含中文描述的列表行以 `…` 截断。
- [ ] 任何界面输出中不出现完整 auth_token/api key。

## Out of Scope

- ccr-ui（Tauri 桌面端）Usage Dashboard 的任何改动。
- llmusage 上游或 `crates/ccr-usage` 查询面的扩展（新 SQL）。
- 历史用量数据回填/重新归因。
- 上表"延后候选"中的三项。
