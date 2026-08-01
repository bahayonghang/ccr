# 适配 llmusage 数据库重构并优化查询

## Goal

让 CCR Usage 分析正确消费 `llmusage 1.1.1` 的当前数据库与 NDJSON 同步契约，恢复 Antigravity/Kimi Code/Pi/Grok 数据的可见性，并在不改变既有统计口径、只读边界和分页语义的前提下降低 SQLite 查询延迟与重复 I/O。

## Confirmed Facts

- 上游事实基线为 `D:/Documents/Code/CLI/llmusage` 的 `d99762bfdd9f920b0d1859fa0a2f7357a9a48d68`；本机 `llmusage.exe` 为 `1.1.1`，与该源码一致。
- `ccr-ui` 不链接上游 crate。同步由已安装 CLI 执行，全部用量 SQL 由 `crates/ccr-usage` 通过只读 SQLite 连接提供，Tauri `llmusage_adapter` 只负责 CLI、事件、错误和表现层映射。
- 上游当前 schema 是 19；本机约 1.16 GB 的代表数据库仍为 schema 18。schema 19 只增加 Activity 覆盖索引，不能作为 CCR 通用最低版本；现有最低兼容线继续为 schema 10，provider 功能继续要求 schema 14。
- schema 13 把所有持久化来源键从 `gemini` 迁移为 `antigravity`。当前来源全集是 `claude`、`codex`、`opencode`、`antigravity`、`kimi_code`、`pi`、`grok`；CCR 仍只识别旧四项并查询 `source='gemini'`。
- 当前真实数据库包含七类来源。CCR 的来源白名单会漏掉 Antigravity/Kimi/Pi/Grok，代表数据库中约漏掉 4,226 个事件和 444.5M tokens。
- 上游同步 NDJSON 已增加 pricing upgrade 和 token-accounting repair 生命周期事件；CCR 当前 `JobEvent` 不认识这些合法事件，且 `SourceKind` 无法反序列化 Kimi/Pi/Grok。
- CCR 日期过滤把 `date(timestamp, 'localtime')` 放在 `WHERE` 左侧，阻断现有时间索引。上游已使用 DST 感知的本地日界转 UTC 半开区间：`timestamp >= ? AND timestamp < ?`。
- `Dashboard::ensure_feature_for_filter` 每个 section 都重新打开连接并重复 schema/表/列探测；`overview` 对同一 bucket 表做 7 次聚合扫描；`home_overview` 做两次可合并的 bucket 聚合。
- `usage_bucket_30m` 从未包含 `project_path`；该列只属于 `usage_event`。CCR 当前 project breakdown 的 bucket 列探测是无效工作。
- 详细源码锚点、schema 矩阵、查询计划和代表数据库测量记录在 [`research/upstream-impact.md`](research/upstream-impact.md)。

## Requirements

### R1 来源契约与兼容

- `SourceKind` 对齐七个当前持久化来源键。
- `gemini`、`gemini-cli` 等既有 CCR 输入继续作为 Antigravity 的兼容别名；新请求和新 wire 值统一使用 `antigravity`。
- schema 10-12 的旧库仍能读取 `source='gemini'`，返回给上层时规范化为 `antigravity`；schema 13+ 查询当前键。
- `source_breakdown` 不再丢弃合法来源，份额分母覆盖数据库中全部七类已知来源。
- Usage 工具栏、视图类型、标签和相关 smoke tests 增加 Antigravity、Kimi Code、Pi / Oh My Pi、Grok Build；旧 `gemini` 前端调用仍能路由到 Antigravity。

### R2 首页投影契约

- 首页 `by_platform` 和 summary 统计全部合法来源；每日 series 保持上游当前四个稳定字段 `claude`、`codex`、`antigravity`、`opencode`，不自行发明 Kimi/Pi/Grok 的新固定 wire 列。
- 现有 `gemini` series 字段切换为 `antigravity`，同步生成 TypeScript 绑定和前端消费。
- `home_overview` 使用一次 date/source bucket 查询派生 totals、active days、by-platform 和 series，保持 `total_sessions = 0` 的当前 CCR 语义。

### R3 同步协议兼容

- `JobEvent` 完整接受当前 pricing upgrade、bucket reconcile 和 token-accounting repair 事件。
- 事件处理器把这些生命周期事件映射到既有 bootstrap/syncing 进度状态，不误报协议错误，也不改变取消、失败或终态处理。
- 已知字段保持强类型；上游结构破坏仍应显式报错，不把任意 `{...}` 行静默吞掉。

### R4 时间范围与时区

- SQL 参数从字符串专用容器改为 typed SQLite values。
- `since`/`until` 按查询时区解析为 UTC 半开区间；`until` 仍是用户语义上的包含日期。
- 本地日期分组和边界使用 IANA/DST 感知时区；UTC 行为保持确定性。
- bucket 与 event 的范围过滤必须让现有 `hour_start`/`event_at` 索引可用，不能在索引列上包 `date()` 后再过滤。

### R5 查询编排与性能

- `overview` 用一次 bucket 条件聚合返回 total、last-24h、事件数、成本、来源数和 bucket 数；run log 时间戳用一次条件聚合读取。
- `home_overview` 合并为一次 bucket 查询；其余 section 复用同一半开区间过滤器，避免重复构造不一致 SQL。
- `Dashboard::open` 在其单个只读连接上生成 capability snapshot；section 查询不再为能力探测重复打开连接。
- project breakdown 直接使用 bucket 的 label/ref/hash 回退链，不再探测不存在的 bucket `project_path`。
- logs 的 keyset pagination、`page_size + 1`、可选 total count、raw JSON 降级行为保持不变。

### R6 兼容与错误边界

- schema 10、13、14、18、19 fixture 都有明确覆盖；未来 schema 只要必需表列仍存在便保持只读可用。
- 缺失数据库、缺失可选表/列、旧 provider schema、空数据库和 malformed schema 的既有 typed error/degrade 行为保持。
- llmusage 继续独占 bootstrap、迁移和写入；CCR 不修改 `D:/Documents/Code/CLI/llmusage`，不引入上游 crate。

### R7 性能与等价证据

- 每项聚合重构必须有 fixture 结果等价测试。
- 关键日期过滤必须有 `EXPLAIN QUERY PLAN` 断言证明命中现有范围索引，且不出现由旧 `date(column, ...)` 过滤造成的可避免全表扫描。
- 在同一代表数据库、同一 filter、预热后多轮中位数口径下，目标至少为：logs 日期过滤降低 80%，overview 降低 50%，home overview 降低 30%，主要 bucket sections 合计降低 20%；若绝对环境噪声使时间阈值不稳定，以查询数和 query-plan 改善为硬门禁并记录实测值。

## Out of Scope

- 修改 `llmusage` 自身的 schema、迁移、索引、同步、CLI 或源解析器。
- 让 CCR bootstrap、迁移或写入 llmusage 数据库，或重新引入上游 Rust crate 依赖。
- 扩展上游尚未定义的 Kimi/Pi/Grok 首页固定 series 字段，或重做 Usage 页面视觉布局。
- 把 schema 18/19 的 Behavior/Activity 新表面引入 CCR。
- 远程 push、PR 或发布操作。

## Acceptance Criteria

- [x] AC1：七类来源在当前 schema 上均能聚合、筛选和展示；Antigravity 不再返回空数据，schema 10-12 的 `gemini` 行规范化为 `antigravity`。
- [x] AC2：overview、daily trend、model、provider、project、source、heatmap、logs 和 home overview 的代表 fixture 结果与重构前语义等价，只有明确列出的来源键升级发生变化。
- [x] AC3：当前 llmusage NDJSON 的全部同步事件和七类 source 都能解析；导入进度、失败、取消和完成状态测试通过。
- [x] AC4：日期范围 SQL 使用 typed UTC 半开区间，DST 跨越用例正确，bucket/event 范围查询的计划命中对应时间索引。
- [x] AC5：overview 只做 1 次 bucket 聚合和 1 次 run-log 聚合；home overview 只做 1 次 bucket 聚合；能力探测不为每个 section 重开连接。
- [x] AC6：schema 10/13/14/18/19、未来兼容、缺表缺列、空库和 provider 降级测试通过；CCR 全程只读且未链接 llmusage crate。
- [x] AC7：代表数据库性能复测满足 R7 阈值或留下查询计划与查询数硬证据，并记录无法满足阈值的具体环境因素；不得把缺失证据报告为通过。
- [x] AC8：`cargo test -p ccr-usage`、Tauri adapter/usage 聚焦测试、frontend usage smoke/type-check/lint、Rust fmt/clippy 及最终 `just ci` 通过。
- [x] AC9：最终 diff 仅包含本任务所需的用量适配、依赖、生成绑定、测试、规范与 Trellis 记录。
