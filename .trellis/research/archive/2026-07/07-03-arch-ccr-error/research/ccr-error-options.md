# Research: CcrError 拆解落法评估(否决式)

- **Query**: 基于 ccr-error-inventory.md 数据,评估落法 A(领域 variant 上移)/ B(冻结)/ 可能的中间态 C,给出时序影响与否决判据
- **Scope**: internal(分析)
- **Date**: 2026-07-05
- **数据来源**: 同目录 `ccr-error-inventory.md`(下称"盘点§N")

## 结论先行(供主会话决策,非最终决定)

**建议采用落法 B(冻结 CcrError,不实施 A)**。两条最硬的证据:

1. **A 的前提在依赖图上不成立**(盘点§2/§9):UiError 的主构造方是 ccr-cli(38 次)而 ccr-tui 依赖 ccr-cli,方向相反;DatabaseError 的最大构造方是 ccr-codex(32 次)而 codex 不依赖 ccr-store;SettingsError 同理(cli 51 + codex 14,归属 crate 就是 ccr-cli 自己)。"领域错误迁至归属 crate"对 14 个领域 variant 中的大头(UiError/SettingsError/DatabaseError/SyncError,合计构造 196 次)要么无处可迁、要么迁了原构造方够不着。
2. **收益侧接近于零的实证**(盘点§3/§10):1082 处引用中生产代码的 variant 分支只有 1 处且匹配原语 variant;行为消费(exit_code/user_message/is_fatal)全集中在 dispatch.rs 一处;拆分 ccr-core 以来 3 个月新增 variant 次数为 0。"上帝枚举"的两项经典代价——分支耦合与新增错误的 locality 倒置——在本仓当前都不发生。

## 落法 A:领域 variant 整体上移

### A 按 PRD 字面执行的形状修正

PRD 设想的映射(Sync→ccr-sync、Platform/Profile→ccr-config、Database→ccr-db/store、Ui→ccr-tui、Update/Settings→对应 crate)与依赖图冲突点:

| PRD 映射 | 冲突 | 现实出路 |
|---|---|---|
| Ui→ccr-tui | ccr-cli 构造 38 次,但 ccr-tui→ccr-cli(盘点§9),不可反向依赖 | UiError 只能留在 ccr-cli 或更低层(即 core,回到原点) |
| Database→ccr-store | ccr-codex 构造 32 次(> store 的 27),codex 不依赖 store;加边 codex→store 技术上无环但纯为错误类型引入新依赖 | codex 需自有 `CodexError::Database`,即 per-crate 错误 |
| Settings→"对应 crate" | Settings 管理器就在 ccr-cli(managers/settings.rs);codex 另有 14 处构造,codex↛cli(会成环) | 同上,per-crate |
| Sync→ccr-sync | ccr-cli 构造 21 次(> sync 的 10),但 cli→sync 依赖已存在 | 可行,但"归属 crate"只贡献 1/3 构造 |
| Platform/Profile→ccr-config | cli/codex/tui 都构造;cli、codex 已依赖 config,tui 需补直接依赖 | 可行 |
| History→ccr-store | 100% 集中(11 次) | 唯一干净可行项 |

**因此 A 的诚实形状不是"variant 上移",而是"每个 crate 自建错误枚举(ccr-checkin 模式)+ 顶层聚合"**——比 PRD 描述的工程量更大一档。

### A 的成本估算(数据支撑)

- 要动的 crate:8 个(cli/codex/config/skills/store/sync/tui/core)+ ccr 根 + src-tauri = 10 个编译单元。
- 文件量级:104 个含构造的文件 + 142 个含 import 的文件(重叠后估 **150-180 个文件**);构造点 1030+ 处(盘点§2);`-> Result<` 签名约 1142 个——若各 crate 自建 `Result` 别名,签名文本可不变,但 import、`?` 跨界转换、聚合枚举 `#[from]` 臂全要新写。
- 公开面:public_api_compat.rs 两个测试的快照必须有意更新(盘点§6);`ccr::prelude::CcrError` 形状变化。
- spec 面:至少 9 处 spec 文件行需要改写(盘点§6 列表)。
- **6.x 冻结的硬约束**:CcrError 是 6.x 冻结 prelude 的成员且**未标 `#[non_exhaustive]`**(error.rs:104-105)。从公开枚举删除 14 个 variant、或把 `SyncError(String)` 载荷改成 `SyncError(ccr_sync::SyncError)`,对任何下游构造/匹配代码都是 breaking。PRD 允许"保留兼容别名/聚合",但**别名救不了 variant 级兼容**:要么(a)6.x 内保留全部旧 variant + 并行新增域错误 → 枚举先变胖再等 next-major 瘦身,即"付了 A 的钱先拿到 B 的货";要么(b)承认这是 next-major 工作。
- `-D warnings` 无 deprecation 约束(public-api-boundary.md:34)→ 迁移期不能用 `#[deprecated]` 引导,1030+ 构造点全靠人工/机械替换,无编译器渐进护栏。
- 分步可编译性:可行路径是自底向上、CcrError 临时保留 `#[from] 各域错误` 聚合臂,每步 `just check-workspace` 可过;但中间态 = 双轨枚举共存贯穿整个迁移窗口。
- 文案回归风险:25 条 Display + 11 段 user_message 长文案需逐字搬运;测试对文案耦合极薄(盘点§7)——这**降低迁移改测试的成本,同时也意味着回归不会被测试捕获**,需人工对拍。
- 体感工作量:数千行 diff、跨 10 编译单元、多 PR 分步——属"周级"改造,对照收益(见否决判据)严重倒挂。

### A 的收益(诚实清点)

- ccr-core 枚举缩到 11 个原语 variant,底层 crate 不再"认识"应用词汇——**纯架构洁癖收益,当前无行为差异**。
- 未来可对域错误做类型化分支(如 ccr-tui 对 UsageError 那样,盘点§8)——当前需求为 0(生产匹配点 1 处且是原语)。
- 新增域错误不再改 ccr-core——该事件 3 个月发生 0 次(盘点§10)。

## 落法 B:冻结 CcrError

### 内容

1. CcrError 25 个 variant 冻结:不再新增领域 variant;原语 variant(如未来需要 `#[from] reqwest::Error` 之类)个案评审。
2. 新 crate / 新子系统一律自建错误类型——**这已经是事实上的现行制度**:拆分后诞生的 ccr-db、ccr-usage、ccr-checkin、llmusage_adapter 全部如此,零摩擦运行至今(盘点§8)。
3. 守卫建议(低成本、可执行):
   - error.rs 顶部加冻结声明注释(中文,说明新错误应建在归属 crate,指向 ADR);
   - ccr-core 加一个 variant 清单快照单测(枚举 25 个 variant 名的显式断言;`exit_code()` 的穷尽 match 已天然强制"加 variant 必须碰 error.rs",快照测试再把"碰了必须解释"显性化)——比 CI grep 守卫更贴近现有测试习惯;
   - 修正 spec 中主动引导写入 CcrError 领域 variant 的措辞(盘点§6 的 ccr-sync/ccr-store/ccr-codex 三行改为"存量路径维持 CcrError,新模块自建错误类型,参照 ccr-db/ccr-usage 先例");
   - 顺手清掉 2 处幽灵 `CcrError::ConfigNotFound` 文档注释(盘点§2)。
4. ADR 经 trellis-update-spec 记入 spec,任务按"否决/缩水"路径关闭。

### B 的代价(诚实清点)

- 双轨永续:存量 8 个 crate 说 CcrError,新 crate 说自有错误——但盘点§8 证明双轨已稳定共存 3 个月,边界清晰(crate 粒度,拆分时间即分界线);
- CcrError 名字继续误导("统一错误"实际只覆盖老 8 crate);
- ConfigError 的语义漂移(344 次万能兜底)不会被治理——但这本来也不是 A 能治的(A 只是把兜底换个名字);
- 若未来某处真需要对域错误类型化分支,届时需局部引入自有错误——成本按需支付,且有 UsageError 现成范式。

## 中间态 C:只上移构造完全集中的 variant

数据裁决:满足"构造点 100% 集中在归属 crate"的领域 variant **只有 HistoryError(ccr-store,11 处)**(盘点§2)。为 1 个 variant 动公开冻结枚举(删 variant 即 breaking,同 A 的 6.x 约束)完全不成比例。**C 无数据支撑,不建议**。

若主会话仍想要"象征性收缩",唯一零破坏动作是文档级的:在 error.rs 为每个领域 variant 注释标注归属与冻结状态。这实际是 B 的守卫子项。

## 时序影响

- **07-03-arch-ccr-facade**(prelude 形状含 CcrError):结论 B ⇒ prelude 中 `CcrError/Result` 形状不变,facade 任务**立即解除阻塞**,按现状收拢 re-export 墙即可;若选 A,facade 必须等聚合枚举定形,阻塞周级。facade prd.md:42 已声明"建议在本任务结论之后动手,非硬阻塞"——B 让它变成"无依赖"。
- **07-03-arch-sqlite-seam**(CcrError vs DbError 取舍):结论 B ⇒ 明确规则:**共享 seam 代码说 DbError(域自有类型),ccr-store 在自己边界 `map_err` 成 CcrError::DatabaseError**(存量 27 处构造不动,只在 seam 注入点桥接;不需要 orphan-rule 敏感的 `impl From<DbError> for CcrError`,map_err 即可)。若选 A,seam 任务还要再等 codex 侧 32 处 DatabaseError 的归属定论(codex 不依赖 store/db,盘点§9)。B 同样解除其错误维度的阻塞。
- 顺带:ccr-codex 的 29 处 DatabaseError 集中在 codex_history_sync_service.rs 一个文件,若 sqlite-seam 任务未来重构该文件,可趁势局部引入 CodexDbError——按需演化,不必预付。

## 否决判据:收益/成本量化对照

| 维度 | A 的账 | B 的账 |
|---|---|---|
| 触碰文件 | 150-180 个(10 编译单元) | ~4 个(error.rs 注释+1 单测、2 处 spec、幽灵注释) |
| 构造点迁移 | 1030+ 处 | 0 |
| 公开 API | prelude/快照×2 有意变更 + 6.x variant 级 breaking 风险 | 0 变更 |
| 解决的实际痛点 | 分支耦合:现存 1 处(原语);locality 倒置:3 个月 0 次 | 同左(冻结后未来也不会新增) |
| 文案回归风险 | 25 Display + 11 user_message 人工对拍,测试网极薄 | 0 |
| 对候选 6/7 | 阻塞两者数周 | 立即解除两者阻塞 |

PRD 预设的否决触发条件是"领域 variant 构造点若高度集中在归属 crate,说明 locality 问题比声称的轻"。实测结果比这更强:**构造点不是集中在归属 crate,而是集中在消费 crate(cli/codex),归属模型本身不成立**——A 不是"收益打折",是"方案前提为假"。叠加 6.x 冻结让"核心枚举清零领域词汇"这一验收项在本 major 内不可达,结论指向否决 A、以 B + 守卫 + ADR 关闭。

## Caveats

- 本文件是调研建议,最终决策(含 ADR 措辞、守卫具体形态)由主会话做。
- 若项目明确规划 next-major(7.0)breaking 清单,A 可作为 7.0 候选重新评估;届时本盘点的构造分布数据仍有效,但应重新跑计数命令核实。
- 未评估"把 exit_codes/user_message 从 error.rs 拆到 dispatch 侧"这类不动 variant 集的内部整理——超出本任务问题域,但若想减薄 ccr-core 的"应用知识",那是比 A 便宜两个数量级的替代方向(唯一消费方就在 dispatch.rs,盘点§3)。
