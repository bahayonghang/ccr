# 修复 Claude Code 页面 Usage Insight 面板数据显示为空

## Goal

修复 ccr-ui Claude Code 页面（`/claude-code`）Usage Insight 面板中"费用日历"、"Token 详情"、"行为分析"三个 Tab 显示为空的问题。

## Problem Statement

**症状**：
- 顶部 Hero 卡片（今日/本月/全部费用）**有数据**（$0.00 / $8178 / $16013）
- 下方三个 Tab 的图表和列表**全部为空**，显示"暂无 Top 10 浏览量数据，检 list-price 是否是 USD。"

**截图证据**：
- Hero 三卡正常显示数值
- "费用日历" Tab：30 天趋势图为空
- "Token 详情" Tab：缓存统计、每日趋势为空
- "行为分析" Tab：热力图、Top 工具、Top Sessions 为空

**根因推测**：
- ✅ `claude_observer_get_insight`（聚合数据）正常
- ❌ `claude_observer_daily_trend` / `cost_breakdown` / `cache_stats` / `tool_heatmap` / `top_tools` / `top_sessions` 等详细维度查询返回空数组

数据源边界：
1. **llmusage**（workspace 依赖）：提供 token/cost 维度数据（insight / daily / breakdown / cache）
2. **ccr-db** `claude_tool_calls` 表：提供工具调用维度数据（heatmap / tools / sessions）

## Requirements

### R1. 诊断数据源状态（P0）

**目标**：确认为什么 `daily_trend` 等查询返回空，但 `insight` 聚合有数据。

#### R1.1 llmusage 数据完整性检查
- [ ] 检查 llmusage 数据库是否有 `platform='claude'` 的记录
- [ ] 验证 `trends_daily` 查询是否返回数据
- [ ] 验证 `project_breakdown` / `model_breakdown` 是否返回数据
- [ ] 确认数据时间范围是否覆盖最近 30 天

#### R1.2 ccr-db 数据完整性检查
- [ ] 检查 `claude_tool_calls` 表是否存在
- [ ] 验证表中是否有记录（至少 1 条）
- [ ] 检查记录字段完整性（`session_id` / `tool_name` / `cost_usd` 非空）

#### R1.3 运行时日志检查
- [ ] 浏览器 Console 查看 `[claudeObserver] load failed` 错误
- [ ] Tauri 日志查找 `Dashboard open error` / `Trends query error`
- [ ] 验证所有 `claude_observer_*` 命令的返回值和错误信息

### R2. 修复数据源问题（P0）

**前置条件**：R1 诊断完成，确认根因。

#### 场景 A：llmusage 数据导入缺失
- [ ] 确认 llmusage 数据导入机制（自动/手动触发）
- [ ] 检查 `src-tauri/src/llmusage_adapter/` 导入逻辑
- [ ] 触发数据导入，确保 `trends_daily` 有数据

#### 场景 B：ccr-db 表未初始化
- [ ] 确认 `claude_tool_calls` 表创建时机（migration/首次启动）
- [ ] 检查 `claude_tool_calls_repo.rs` 插入逻辑
- [ ] 触发工具调用记录导入

#### 场景 C：查询逻辑错误
- [ ] 验证 `build_filter()` 是否正确传递 `platform='claude'`
- [ ] 验证日期窗口计算逻辑（`today_window()` / `month_window()`）
- [ ] 检查 SQL 查询是否有语法错误或索引问题

### R3. 用户体验优化（P1）

- [ ] **空状态提示准确化**：
  - 当前："暂无 Top 10 浏览量数据，检 list-price 是否是 USD。"（误导）
  - 改为："暂无数据，请等待后台导入 Claude Code 使用记录"
- [ ] **错误日志上报**：在前端 Console 输出详细错误（`store.daily.error`）
- [ ] **手动刷新按钮**：在面板顶部增加"刷新数据"按钮，调用 `store.fetchAll()`

### R4. 降级处理（P2，可选）

如果 llmusage 明细数据长期无法获取：

- [ ] 考虑从 `insight` 聚合数据推断简单趋势（如均分到 30 天）
- [ ] 或明确告知用户：详细图表需要完整的数据导入

## Acceptance Criteria

### AC1. 数据源完整性恢复

- [ ] llmusage `trends_daily` 返回 ≥30 天的数据点
- [ ] llmusage `project_breakdown` / `model_breakdown` 返回 ≥1 条记录
- [ ] ccr-db `claude_tool_calls` 表有 ≥1 条记录

### AC2. 前端展示正常

- [ ] "费用日历" Tab 显示 30 天趋势曲线（非空白虚线框）
- [ ] "按项目"/"按模型" 横向条形图显示 Top 10 数据
- [ ] "Token 详情" Tab 显示缓存统计（命中率、四个 token 总量）
- [ ] "行为分析" Tab 显示工具调用热力图、Top 工具排行、Top Sessions

### AC3. 错误提示准确

- [ ] 当数据源为空时，显示友好的空状态提示（而非误导性提示）
- [ ] Console 或 Tauri 日志输出详细错误信息（非静默失败）
- [ ] 用户可通过"刷新数据"按钮手动重试

### AC4. 测试场景通过

1. **首次启动**：ccr-ui 首次启动时，Usage Insight 面板显示"数据正在同步"或空状态
2. **数据导入后**：手动触发数据导入后，刷新页面，三个 Tab 均有内容
3. **长期使用**：使用 Claude Code 一段时间后，数据持续更新（验证自动导入机制）

## Constraints

1. **不破坏现有架构**：llmusage 和 ccr-db 的数据源边界保持不变
2. **向后兼容**：修复不应影响其他平台（Codex/Gemini）的数据展示
3. **性能要求**：`store.fetchAll()` 并发查询总耗时 ≤3 秒

## Non-Goals

- 不涉及 Usage Insight 面板的 UI/UX 重构
- 不涉及 `/usage` 完整仪表盘的修复
- 不涉及 llmusage 数据源的性能优化
- 不涉及新数据维度的添加（如按用户、按时段统计）

## Reference

- **前端组件**：`ccr-ui/src/components/claude-observer/UsageInsightPanel.vue`
- **Tab 组件**：`CostAttributionTab.vue` / `TokenDetailTab.vue` / `BehaviorAnalysisTab.vue`
- **Store**：`ccr-ui/src/stores/claudeObserver.ts`
- **API 封装**：`ccr-ui/src/api/tauri.ts`（`claudeObserver.*`）
- **后端命令**：`ccr-ui/src-tauri/src/commands/claude_observer.rs`（9 个 command）
- **数据源适配**：`ccr-ui/src-tauri/src/llmusage_adapter/`
- **ccr-db repo**：`crates/ccr-db/src/database/repositories/claude_tool_calls_repo.rs`
