# ts-rs 绑定 diff 判定（dep-upgrade R7 / AC7）

对象：commit `1176a416` 中 `ccr-ui/src/types/generated` 下全部 14 个变更文件（16 insertions / 16 deletions）。
方法：逐 hunk 阅读 `git show 1176a416 -- ccr-ui/src/types/generated`，逐文件 grep `ccr-ui/src` 消费方后判定。

## 背景：ts-rs 12 mapped-type 输出变化

ts-rs 12 对 Rust `HashMap<String, T>` 生成的 TS 映射类型从 `{ [key in string]?: T }` 改为 `{ [key in string]: T }`（可选标记移除）。该差异在本仓 tsconfig（`noUncheckedIndexedAccess: false`、`exactOptionalPropertyTypes: false`）下的实际语义：

- **读取侧**：旧类型读任意键得 `T | undefined`；新类型得 `T`。编译器对缺键读取的告警精度下降，但不会产生新错误。
- **写入侧**：新类型要求每个值可赋给 `T`（不接受 `undefined` 值）。这是收紧方向，可能引发编译错误。

## 判定表

| 文件 | 变化分类 | 具体差异 | 前端消费方 | 影响 | 判定 |
|---|---|---|---|---|---|
| codex_auth/CodexJsonValue.ts | 类型 | `{ [key in string]?: CodexJsonValue }` → 去 `?` | 无（仅生成内部 `CodexLoginState.raw` 引用，`src` 无直接 import） | 无消费方 | 接受 |
| common/OpenJsonValueDto.ts | 类型 | 同上模式，去 `?` | 大量：api/domains/{claude,codex,gemini,grok,systemPrompts,unifiedMcp}.ts 及 generated clients | 编译期无影响（见下方证据） | 接受 |
| events/JsonValueDto.ts | 类型 | 同上模式，去 `?` | utils/logger.ts（`toJsonFields`） | 编译期无影响 | 接受 |
| grok/GrokSettingsPatchDto.ts | 类型 | `set: { [key in string]?: OpenJsonValueDto }` → 去 `?` | api/domains/grok.ts、utils/grokSettings.ts、views/grok/GrokSettingsView.vue | 编译期无影响 | 接受 |
| system/CliVersionsResponse.ts | 类型 | `versions: { [key in string]?: string }` → 去 `?` | api/domains/system.ts、views/DashboardView.vue | 无消费方（该字段） | 接受 |
| usage/CapabilityReport.ts | 类型 | `features: { [key in string]?: FeatureCapability }` → 去 `?` | stores/usage.ts、stores/homeUsageOverview.ts、views/MonitoringView.vue、composables/usage/state/useUsageMeta.ts | 编译期无影响 | 接受 |
| usage/HeatmapResponseDto.ts | 类型 | `data: { [key in string]?: number }` → 去 `?` | stores/usage.ts（heatmap 状态） | 编译期无影响 | 接受 |
| usage/HomeUsageOverviewResponse.ts | 类型 | `by_platform: { [key in string]?: HomeOverviewPlatformStats }` → 去 `?` | views/dashboard/dashboardPresentation.ts、components/dashboard/DashboardUsageMovement.vue | 编译期无影响 | 接受 |
| claude_observer/HeatmapCell.ts | 格式 | `{` 后与 `,` 后新增行尾空格（2 处） | components/claude-observer/*（经 store 间接使用） | 编译期无影响（空白不进 AST） | 接受 |
| claude_observer/InsightDto.ts | 格式 | 同上，行尾空格（2 处） | 经 claude-observer 相关视图间接使用 | 编译期无影响 | 接受 |
| sync/SyncStatusInfo.ts | 格式 | 同上，行尾空格（1 处） | sync 相关组件/store | 编译期无影响 | 接受 |
| sync/WebDavConfigInput.ts | 格式 | 同上，行尾空格（1 处） | sync 表单提交路径 | 编译期无影响 | 接受 |
| usage/DailyTrendDto.ts | 格式 | 同上，行尾空格（1 处） | stores/usage.ts（trends 数组元素） | 编译期无影响 | 接受 |
| usage/UsageImportResultV2.ts | 格式 | 同上，行尾空格（1 处） | 导入结果展示路径 | 编译期无影响 | 接受 |

## 汇总

- 变化分类计数：**类型变化 8 项 / 格式变化 6 项**，共 14 项，与变更文件数一致。
- 类型变化的 8 个文件均为同一模式（mapped-type 可选标记移除）；格式变化的 6 个文件均为纯行尾空白（ts-rs 12 在 doc comment 与单行字段交错输出时产生的尾部空格），无 AST 层面差异。
- 验证：`cd ccr-ui && bun run type-check` exit **0**（写入侧收紧未触发任何编译错误）。

## 类型变化逐项消费方证据

- **CodexJsonValue**：`grep '\bCodexJsonValue\b' ccr-ui/src` 仅命中生成目录自身及 `CodexLoginState.ts`；`CodexLoginState` 也无 src 直接 import（codexAuthAccounts.ts 中的同名标识符是本地独立定义）。→ 无消费方。
- **OpenJsonValueDto**：所有构造路径都经 `unknown` 中转——`toOpenJson(value: unknown)`（codex.ts）、`convertOpenJsonValue`/`asOpenJson` 入参为 unknown（api/_shared.ts）、logger 之外无裸字面量直赋；读取路径统一走 `asRecord` 断言或显式 `Record<string, OpenJsonValueDto | undefined>` 注记（systemPrompts.ts objectValue）、`Object.entries(...).filter(typeof entry[1] === 'string')`（gemini.ts stringMap），对值是否含 `undefined` 均有运行时防护。→ 编译期无影响。
- **events/JsonValueDto**：唯一消费者 logger.ts 的 `toJsonFields(data: unknown): JsonValueDto | undefined` 用 `data as JsonValueDto` 直转，fields 上不做键级类型读取。→ 编译期无影响。
- **GrokSettingsPatchDto.set**：utils/grokSettings.ts:85 `const set: GrokSettingsPatchDto['set'] = {}` 后仅写入 `Number(value)` 与 `string`，均在 `OpenJsonValueDto` 联合内；空对象对索引签名仍合法。→ 编译期无影响。
- **CliVersionsResponse.versions**：全仓唯一响应消费点 DashboardView.vue `applyCliVersions(versions.entries)` 只读结构化 `entries` 字段，兼容字段 `versions` 无任何读取方。→ 该字段无消费方。
- **CapabilityReport.features**：4 处已知键读取（`features.overview`、`features.provider_breakdown`、`features.home_overview`、`features.sync_json_events`）全部保留判空守卫（`?.features.overview ?? null`、`if (overviewCap && ...)`、`if (cap && !cap.supported)`）。读取类型由 `FeatureCapability | undefined` 收窄为 `FeatureCapability` 后这些守卫仍合法且必要。→ 编译期无影响。
- **HeatmapResponseDto.data**：唯一读取点 stores/usage.ts `return heatmap.value?.data ?? null` 为整体透传（轮询刷新信号），无按键访问。claude-observer 的 heatmap 是另一套类型（`HeatmapCell[]`），不受本变更影响。→ 编译期无影响。
- **HomeUsageOverviewResponse.by_platform**：dashboardPresentation.ts:246 `input.overview?.by_platform[platform.usageKey]` 读可能缺失的键，结果传入 `getPlatformMetric(stats: HomeOverviewPlatformStats | undefined)` 且函数内有 `if (!stats)` 守卫；读取类型收窄不影响该签名（宽化赋值合法），运行时缺键防护保留。→ 编译期无影响。

## 结论

14 项变更全部判定**接受**：8 项类型变化在当前 tsconfig 与现有调用点写法下编译期与运行时均无影响，6 项为纯空白格式差异；**无需开立 view-subtask 跟进工单**。

一条非阻塞备忘（不构成工单）：`features`/`by_platform` 等 map 的缺键读取不再被编译器标注为可能 `undefined`，若未来开启 `noUncheckedIndexedAccess` 或新增不经判空的键读取，需要重新评估这批绑定。
