# Research: Codex Auth 本地用量归因与黄色说明

- Query: 核对截图右下黄色说明的来源、真实语义、归因边界和最小修复方案。
- Scope: internal；只读源代码及规范，写入本研究文件。
- Date: 2026-09-08

## Findings

### 1. 截图说明是正常筛选范围说明，被错误赋予了警告视觉

截图英文 `Only local records attributed to this account are shown; older or unattributed history is excluded` 的精确来源是 `crates/ccr-tui/src/tui/codex_auth/app.rs:634-645`。此分支已经成功得到选中账号的归因记录，状态明确设为 `AccountAttributed`（同文件 `:624-632`），只有全局记录数大于账号记录数时才增加这段说明。

`crates/ccr-tui/src/tui/codex_auth/ui.rs:1074-1077` 把任意 `fallback_reason` 一律以 `theme::warning_style()` 渲染。这同时覆盖了正常账号范围说明和真正的全局回退原因；字段名与视觉都混合了两种语义。截图上绿色 `CCR ledger matched` 与黄色说明并存，正是这个代码组合的预期结果，不能据此认定解析失败、认证失败或数据损坏。

可证实的次要文案问题：全局数大于账号数，也可能仅因为另一账号有正常已归因记录；不能仅凭这个比较判定存在“更早或未归因历史”。`app.rs:1812-1880` 的既有成功归因测试恰好构造两个账号、两个归因窗口和两条记录，可以直接复用来回归这个情况。

### 2. 归因以 CCR 激活时间窗为依据，不能提升为官方账号账单

- `crates/ccr-codex/src/models/codex_auth.rs:272-285` 定义 `CodexUsageActivation { account_name, account_id, started_at }`，模型注释说明账本来自 CCR 账号激活；稳定匹配使用 account ID。
- `crates/ccr-tui/src/tui/codex_auth/app.rs:662-694` 将账本按开始时间排序，找每条记录之前最近一次激活，按 account ID 匹配，并限制在下一次激活开始前；首次激活之前的记录被排除。
- `crates/ccr-codex/src/models/codex_auth.rs:329-348` 避免连续相同账号激活重复追加。
- `crates/ccr-tui/src/tui/codex_auth/app.rs:287-308` 使用 `CodexUsageService::parse_all_logs()` 读取本地记录并计算全局聚合；不是右上配额 API 的账单数据，也不是 profile 页的 `ccr-usage` SQL 投影路径。
- `crates/ccr-codex/src/services/codex_usage_service.rs:468-471` 每条 usage record 的 `total_requests` 加一；`:483-499` 将每条记录计入 all-time 和相应滚动窗口。当前 `global.all_time.total_requests > attributed_records.len()` 比较单位是一致的，没有在这个比较中发现算术单位错误。

显示建议使用“本地用量”“CCR 切换记录归因”等可理解且证据一致的文字。保留账号名和范围，避免把本地时间窗归因显示成官方账号总消耗或官方剩余额度。

### 3. 必须保留的状态区别

| 输入/条件 | 当前数据范围 | 证据 | 建议展示 |
| --- | --- | --- | --- |
| 没有选中账号 | 全局本地用量 | `app.rs:559-565` | 中性全局范围，不制造警告 |
| 当前登录是未保存的虚拟账号 | 全局本地用量 | `app.rs:569-581` | 明确未保存，账号级归因不可用 |
| 选中账号无 registry 元数据 | 全局回退 | `app.rs:585-601` | 保留醒目的“全局”与回退原因 |
| 无该账号可归因记录 | 全局回退 | `app.rs:605-620` | 保留醒目的“全局”与归因不可用原因 |
| 账号归因成功，全局包含其他记录 | 仅该账号的本地归因记录 | `app.rs:624-645` | 中性简短范围说明，不使用警告色 |
| 读取本地日志失败 | 无正常统计 | `app.rs:308`、`ui.rs:1099-1105` | 保留真实错误样式和原因 |
| 没有本地记录 | 无统计 | `app.rs:299-300`、`ui.rs:1092-1095` | 中性空态 |

不要为了消除黄色说明把全局统计混入账号统计，也不要把真正的回退/错误一律降为灰色。登录状态与历史用量是否存在是不同维度，当前未登录仍可有历史本地用量。

### 4. 最小推荐修复

1. 在现有 `CodexUsageAttributionState` 上判断说明样式：成功归因说明用 muted/info，`VirtualAccount` / `UnattributedFallback` 保留 warning；加载错误继续 error。不需要引入新 schema、通用消息引擎或持久化配置。
2. 将展示模型中的 `fallback_reason` 改为能覆盖正常说明的名字，例如 `scope_note`，或保留字段但明确分支语义；优先由实现者选择与整体页面改造最一致的最小方式。
3. 成功归因说明简化为中性 `Only records attributed to this account are included` / `仅统计归因到此账号的本地记录`，不猜测排除记录一定是旧历史或未归因记录。
4. 精简 scope + attribution 的重复信息，但保留“哪个账号 / 全局”“本地统计”与归因依据。统计数字优先可见，次级说明可放到数字之后；真正回退仍须先说明范围，防止把全局数字认作选中账号数字。
5. UI 文案同时使用 `tui_text!` / `tui_format!` 提供中英文，不增加日志扫描、账本回填、认证变更或新的数据查询。

### 5. 现有测试与建议验收

现有测试（本轮只读检查，未执行）：

- `app.rs:1738-1778`：时间窗前、窗内、其他账号窗内的记录筛选。
- `app.rs:1782-1808`：无归因记录时回退全局。
- `app.rs:1812-1880`：选中账号只保留匹配记录，top model 来自对应记录；目前未断言正常说明的性质。
- `ui.rs:2113-2178`：Ratatui TestBackend 下保留配额与全局回退说明；目前仅断言文本，没有区分正常说明与 warning 样式。

验收/回归应覆盖：

1. 两个账号都有记录时，选中账号的聚合和 top model 不变，正常说明为中性样式，且不声称其他记录未归因。
2. 全部记录属于选中账号时，没有多余警告；数据和账号范围一致。
3. registry 缺失、空账本/无匹配记录、虚拟账号三类回退都明确标为全局，保留可见原因，不冒充账号数字。
4. `UsageState::Error` 保留 error；`NoData`、loading 保持可辨认的非错误状态。
5. 中文/英文的窄、标准、宽 TestBackend 布局都保留统计范围与主要数字；按 cell/style 断言正常说明与真实 warning/error 的视觉区别，避免只检测字符串。
6. 原有窗口隔离测试继续通过。若不改窗口算法，无需扩展到账本修复或解析器重构。

## Files Found

- `crates/ccr-tui/src/tui/codex_auth/app.rs`：页面数据模型、账号归因筛选、本地 usage 加载、状态测试。
- `crates/ccr-tui/src/tui/codex_auth/ui.rs`：scope/attribution/note 渲染，加载错误和空态，TestBackend 测试。
- `crates/ccr-codex/src/models/codex_auth.rs`：激活账本定义及追加逻辑。
- `crates/ccr-codex/src/services/codex_usage_service.rs`：本地日志解析及滚动统计单位。

## Related Specs

- `.trellis/spec/ccr-tui/backend/index.md`
- `.trellis/spec/ccr-tui/backend/backend-guidelines.md`：状态与渲染分离、TUI 双语、显示宽度、fixture/temp-dir 测试、真实错误保留。
- `.trellis/workflow.md`：研究落盘和任务规划要求。

最低相关验证命令由主线程整合到执行计划：`cargo test -p ccr-tui -- --test-threads=1`、`cargo test -p ccr-usage`、`just fmt-check`、`just lint-strict`。本次仅研究，不报告这些检查通过。

## External References

无。本结论针对当前仓库实现，不需要对外部 API 或最新 Codex 日志格式作推断。

## Caveats / Not Found

- 未访问用户认证文件、实际 usage 日志或私有数据库，未发起 API 请求；截图数据正确性与真实账号归属属于 UNVERIFIED。
- 从源码能确认黄色说明的产生条件和样式问题，不能确认截图对应二进制与当前工作树完全一致。
- 未发现足以把本任务扩大为账本回填、全局归因算法修复或本地日志解析器修复的证据。CCR 时间窗归因也不能证明用户在其他工具/进程中的账号身份。
- 右上配额窗口、颜色条和全页排版由主线程独立分析，本研究只负责右下归因说明及其数据语义。
