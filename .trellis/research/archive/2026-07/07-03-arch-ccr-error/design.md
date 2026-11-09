# design：CcrError 评估结论与缩水实施方案

## 决策

**否决落法 A（领域 variant 上移归属 crate），采用落法 B（冻结 CcrError + 守卫 + ADR）。**
依据 `research/ccr-error-inventory.md` 与 `research/ccr-error-options.md`（2026-07-05，全量盘点 1082 处引用 / 104 文件，命令可复现），载重证据已在主会话抽查核实：

1. **A 的前提在依赖图上为假**：UiError 主构造方 ccr-cli（38 次）反被 ccr-tui 依赖（`crates/ccr-tui/Cargo.toml:16`）；DatabaseError 最大构造方 ccr-codex（32 次）不依赖 ccr-store（`crates/ccr-codex/Cargo.toml:12-14`）；SettingsError 归属 crate 就是 ccr-cli 自己。除 HistoryError（ccr-store 11 处，唯一 100% 集中）外，"迁至归属 crate"对构造量前四的领域 variant 不可实施。A 的诚实形状是"每 crate 自建错误枚举 + 顶层聚合"，150-180 文件、1030+ 构造点。
2. **6.x 冻结使 A 的验收项不可达**：`CcrError` 是冻结 prelude 成员（`public-api-boundary.md` §2/§3）且未标 `#[non_exhaustive]`（`error.rs:104-105`），删/改公开 variant 即 breaking；当前 6.4.3。"core 领域词汇清零"只能是 7.0 工作。
3. **收益侧实证接近零**：生产代码 variant 分支全仓仅 1 处且匹配原语（`codex_history_sync_service.rs:2878` FileLockError）；exit_code/user_message/is_fatal 唯一消费方是 `dispatch.rs:738-747`；ccr-core 拆分（2026-03-31）以来新增 variant 0 次——上帝枚举的两项经典代价（分支耦合、locality 倒置）当前都不发生。

中间态 C 无数据支撑（仅 HistoryError 满足集中条件，为 1 个 variant 动冻结枚举不成比例），不做。

## B 的实施形状（缩水后全部范围）

### 1. 代码守卫（crates/ccr-core/src/core/error.rs + 2 处幽灵注释）

- enum `CcrError` 顶部加冻结声明注释（中文）：25 个 variant 冻结，不再新增领域 variant；原语 variant 个案评审；新领域错误建在归属 crate（参照 ccr-db/ccr-usage/ccr-checkin 先例）；指向 ADR spec 路径。
- error.rs 既有 `#[cfg(test)]` 模块内加 variant 清单快照单测：穷尽 match 的 `variant_name()`（新增 variant → missing arm 编译错，删除/更名 → 未知 variant 编译错）+ `FROZEN_VARIANTS: [&str; 25]` 名单断言。测试带冻结横幅注释，把"碰了必须解释"显性化（`exit_code()` 的穷尽 match 只强制"碰 error.rs"，不强制读 ADR）。
- 顺手修 2 处幽灵文档注释：`crates/ccr-cli/src/commands/profile/{enable.rs:22, disable.rs:24}` 引用不存在的 `CcrError::ConfigNotFound`（文档腐化），改为代码实际返回的 variant。

### 2. ADR 记入 spec（trellis-update-spec 流程）

新增 `.trellis/spec/ccr-core/backend/ccr-error-freeze.md`（并登记进该 index.md）：

- 决策与数据摘要（构造点分布、依赖图冲突、6.x 约束、收益实证）。
- 规则：CcrError 冻结；新模块/新子系统自建错误类型（ccr-db `DbError` / ccr-usage `UsageError` / ccr-checkin 为范式）；存量 8 crate 的 CcrError 路径维持不动。
- 给 07-03-arch-sqlite-seam 的错误规则：**共享 seam 代码说 DbError，ccr-store 在自己边界 `map_err` 成 `CcrError::DatabaseError`**（存量 27 处构造不动，不写 `impl From<DbError> for CcrError`）。
- 给 07-03-arch-ccr-facade：prelude 中 `CcrError/Result` 形状不变，无依赖阻塞。
- 7.0 重评估条目：A 登记为 next-major breaking 候选，届时重跑盘点命令核实分布。

### 3. spec 措辞修正（防未来漂移）

盘点 §6 列出 9 处 spec 主动规定"映射到 CcrError 领域 variant"。逐处核对，只改**对新代码有引导性**的措辞（加"存量路径维持；新模块自建错误类型，见 ADR"限定），纯描述现状的不动。已知必改 3 处：

- `.trellis/spec/ccr-sync/backend/backend-guidelines.md:27`
- `.trellis/spec/ccr-store/backend/backend-guidelines.md:31`
- `.trellis/spec/ccr-codex/backend/backend-guidelines.md:34`

## 不做（显式出界）

- exit_codes/user_message 从 error.rs 拆到 dispatch 侧的内部整理（唯一消费方在 dispatch.rs，是比 A 便宜两个数量级的减薄替代，但超出本任务问题域；ADR 中记一笔即可）。
- ConfigError 万能兜底（344 次）的治理——A 也治不了，冻结不恶化它。
- 任何 public_api_compat 快照变更、任何 variant 增删。

## 验收对照（prd AC）

- AC1（评估结论记录）：research/ 两文件 + 本 design.md。
- AC2-4（若实施）：N/A——A 已否决。
- AC5（若否决/缩水：ADR + 守卫记入 spec）：上述 §1-§3。

## 回滚

全部改动为注释、新增测试、文档——`git revert` 单提交粒度即可，无数据/接口风险。
