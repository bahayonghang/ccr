# 拆解 CcrError 上帝枚举

## Goal

`crates/ccr-core/src/core/error.rs` 约 26 个 variant 中，History/Sync/Platform/Profile/Database/Ui/Update/Settings/ConfigSectionNotFound 等上层领域词汇下漏进 primitives crate——底层 crate 被迫认识全应用概念，所有 crate 耦合到一个枚举。core 仅留 Io/Lock/Cache/Format 原语，领域错误上移至各自 crate 并以 `#[from]` 包裹。**Speculative——改动面全仓，先出小成本评估结论再定**。审查候选 8。

## Requirements

### 前置评估（必须先做，结论可否决或缩水本任务）

1. 盘点 `rg 'CcrError::' --type rust -c` 按 crate 统计构造点分布，估算迁移体量。
2. 评估两种落法：A) 领域 variant 整体上移（各 crate 自有 error + core 原语 `#[from]`）；B) 保守版——仅新增领域 crate 自有错误用于新代码，存量 CcrError 冻结不再扩张。
3. 与 07-03-arch-ccr-facade（prelude 形状含 CcrError）、07-03-arch-sqlite-seam（CcrError vs DbError 取舍）的时序关系：本任务评估结论应先于它们动手。
4. 若结论是"收益不抵全仓churn"，以 ADR 记录进 spec 后关闭（trellis-update-spec），并给出"CcrError 不再新增领域 variant"的守卫建议。

### 现状（探索报告定位）

- ccr-core 不依赖 ccr-types，却定义 HistoryError、SyncError、PlatformNotFound、ProfileNotFound、DatabaseError、UiError、UpdateError、SettingsError、ConfigSectionNotFound 等 ~26 variant。
- locality 倒置：新增一个领域错误要改依赖图最底部的 crate；每个 crate 都耦合该类型。
- `public-api-boundary.md` 将 `CcrError`/`Result` 列入稳定 prelude——公开面变化受快照守卫约束。

### 要做的（若评估通过，按选定落法）

1. ccr-core 错误收缩为原语集（Io/Lock/Cache/Format 及同级）。
2. 领域错误迁至归属 crate（SyncError→ccr-sync、Platform/Profile→ccr-config、Database→ccr-db/store、Ui→ccr-tui、Update/Settings→对应 crate），经 `#[from]` 包裹 core 原语。
3. 顶层（ccr-cli/ccr）聚合错误的形状在设计阶段定（薄 app error 或直接透传各域错误）。
4. 迁移全部构造点与匹配点；错误信息文案不回归（用户可见输出不变）。

### 约束

- `public-api-boundary.md`：`ccr::prelude::CcrError` 与 legacy 根路径在 6.x 必须继续可用——存量公开类型可保留为兼容别名/聚合，快照有意更新。
- `-D warnings` 下游：不得引入 deprecation 告警（spec 明文）。
- 分 crate 渐进迁移，每步 `just check-workspace` 可编译。

## Acceptance Criteria

- [ ] 前置评估结论已记录（构造点分布数据 + 选定落法 + 与候选 6/7 的时序建议）。
- [ ] （若实施）ccr-core error.rs 不再含领域词汇 variant（History/Sync/Platform/Profile/Database/Ui/Update/Settings 清零）。
- [ ] （若实施）各领域 crate 自有错误类型有测试；用户可见错误文案不回归（对照集成测试输出）。
- [ ] （若实施）`cargo test -p ccr --test public_api_compat` 通过，快照变化均有说明；全仓 `just lint-strict`、`just test` 通过。
- [ ] （若否决/缩水）ADR + 守卫建议记入 spec（trellis-update-spec）。

## Notes

- 复杂任务：评估通过后、`task.py start` 前需补 design.md 与 implement.md。
- 时序：本任务评估结论建议先于 07-03-arch-ccr-facade 与 07-03-arch-sqlite-seam 实施。
