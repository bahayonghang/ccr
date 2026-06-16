# 诊断报告：Claude Code Usage Insight 面板数据显示为空

**日期**：2026-06-16
**任务**：`.trellis/tasks/06-16-claude-code-usage-insight-empty`

---

## 执行摘要

**根本原因**：llmusage 数据源正常，代码使用了正确的表名（`usage_bucket_30m` / `usage_event`），但前端仍然收到空数据。需要进一步检查 Tauri 命令执行和前端调用链路。

**影响范围**：
- ✅ Hero 三卡（今日/本月/全部）显示正常
- ❌ "费用日历" Tab 显示为空
- ❌ "Token 详情" Tab 显示为空
- ❌ "行为分析" Tab 显示为空

---

## 数据源诊断结果

### 1. llmusage 数据源 ✅

**数据库路径**：`C:\Users\lyh\.llmusage\llmusage.db`（509 MB）

**表结构**（15 个表）：
- ✅ `usage_bucket_30m`（聚合数据）：1,498 条 Claude 记录
- ✅ `usage_event`（明细数据）：170,244 条 Claude 记录
- ✅ `usage_tool_call`（工具调用）：待验证
- ✅ `project_dim`（项目维度）：1 个项目

**时间范围**：最近 10 天都有数据
```
2026-06-14: 1,301 条事件, 282,755,784 tokens
2026-06-13: 5,529 条事件, 1,143,268,791 tokens
2026-06-12: 4,291 条事件, 672,183,309 tokens
...
```

**结论**：✅ llmusage 数据完整，表名已更新为新版

### 2. ccr-db 数据源 ❌

**数据库路径**：`C:\Users\lyh\.ccr\ccr.db`

**状态**：❌ 文件不存在

**影响**：
- `claude_observer_top_sessions`（Top Sessions）返回空
- `claude_observer_tool_heatmap`（工具热力图）返回空
- `claude_observer_top_tools`（Top 工具）返回空

**结论**：ccr-db 未初始化，导致"行为分析" Tab 为空

---

## 代码审查结果

### 1. llmusage_adapter 表名映射 ✅

**文件**：`ccr-ui/src-tauri/src/llmusage_adapter/db.rs`

**查询语句**：
```rust
// ✅ 已更新为新表名
SELECT ... FROM usage_bucket_30m  // 旧版: buckets
SELECT ... FROM usage_event       // 旧版: events
```

**结论**：代码已正确使用新表名

### 2. Tauri 命令实现 ✅

**文件**：`ccr-ui/src-tauri/src/commands/claude_observer.rs`

**9 个命令**：
1. `claude_observer_get_insight` - ✅ 调用 `overview_in_window()`
2. `claude_observer_daily_trend` - ✅ 调用 `dashboard.trends_daily()`
3. `claude_observer_cost_breakdown` - ✅ 调用 `dashboard.project_breakdown()` / `model_breakdown()`
4. `claude_observer_cache_stats` - ✅ 调用 `dashboard.overview()` + `trends_daily()`
5. `claude_observer_top_sessions` - ❌ 依赖 `ccr-db`
6. `claude_observer_tool_heatmap` - ❌ 依赖 `ccr-db`
7. `claude_observer_top_tools` - ❌ 依赖 `ccr-db`
8. `claude_observer_subscription_get` - ✅ 读取 `user_settings`
9. `claude_observer_subscription_set` - ✅ 写入 `user_settings`

**结论**：命令实现正确，但需验证运行时执行

---

## 疑似问题点

### 问题 1：Tauri 命令执行失败但未上报错误

**猜测**：
- `claude_observer_daily_trend` 等命令在 `spawn_blocking` 中抛错
- 错误被 `map_err` 捕获但前端未正确处理
- 前端 `store.daily.error` 为 null，显示空状态而非错误状态

**验证方法**：
1. 检查浏览器 Console 是否有 `[claudeObserver] load failed` 日志
2. 检查 Tauri 日志（`tauri.log`）是否有 `Dashboard open error` / `Trends query error`
3. 手动调用 `invoke('claude_observer_daily_trend')` 查看返回值

### 问题 2：llmusage Dashboard 初始化失败

**猜测**：
- `open_dashboard()` 连接数据库失败
- `MIN_SUPPORTED_SCHEMA_VERSION` 不匹配
- 表权限问题（只读？）

**验证方法**：
```rust
// ccr-ui/src-tauri/src/llmusage_adapter/db.rs
pub fn open_dashboard(paths: AppPaths) -> Result<Dashboard, LlmusageAdapterError> {
    let conn = Connection::open_with_flags(&paths.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let schema_version = read_schema_version(&conn)?;
    ensure_feature_by_version(schema_version, MIN_SUPPORTED_SCHEMA_VERSION)?;
    // ...
}
```

检查 `MIN_SUPPORTED_SCHEMA_VERSION` 是否与实际 schema_version 匹配。

### 问题 3：ccr-db 未初始化

**确认**：ccr-db 数据库文件不存在

**影响**：
- "行为分析" Tab 完全为空（Tool Heatmap / Top Tools / Top Sessions）

**修复**：
1. 检查 `crates/ccr-db` 的初始化逻辑
2. 确认 `claude_tool_calls` 表的创建时机
3. 触发表初始化（migration / 首次启动）

---

## 下一步行动

### 立即执行（P0）

1. **验证 Tauri 命令执行**：
   ```javascript
   // 浏览器 Console 执行
   window.__TAURI__.core.invoke('claude_observer_daily_trend', { days: 30 })
     .then(console.log)
     .catch(console.error)
   ```

2. **检查前端错误日志**：
   - 打开 `http://localhost:1420`（Tauri dev 模式）
   - 查看 Console 中的 `[claudeObserver]` 日志
   - 检查 Network 标签页是否有 IPC 请求失败

3. **检查 Tauri 日志**：
   ```powershell
   # Windows
   type %LOCALAPPDATA%\com.ccr.dev\logs\*.log | Select-String -Pattern "claude_observer"
   ```

### 修复方案（待验证后确定）

#### 方案 A：Tauri 命令执行失败
- 修复 `spawn_blocking` 中的错误处理
- 确保错误信息正确传递到前端
- 优化错误提示文案

#### 方案 B：llmusage Dashboard 初始化问题
- 检查 schema_version 兼容性
- 更新 `MIN_SUPPORTED_SCHEMA_VERSION`
- 处理数据库权限问题

#### 方案 C：ccr-db 初始化缺失
- 实现 `claude_tool_calls` 表的自动初始化
- 添加 migration 逻辑
- 从 llmusage `usage_tool_call` 表导入数据

---

## 附录：诊断脚本输出

```
============================================================
Claude Code Usage Insight 数据源诊断
============================================================

============================================================
诊断 llmusage 数据源
============================================================
数据库路径: C:\Users\lyh\.llmusage\llmusage.db
✅ 数据库文件存在 (大小: 509636.0 KB)

表列表 (15 个):
  - integration_install
  - meta
  - project_dim
  - run_log
  - source_cursor
  - source_file
  - source_sync_status
  - sqlite_sequence
  - trigger_state
  - usage_bucket_30m
  - usage_event
  - usage_event_raw
  - usage_tool_call
  - usage_turn
  - worker_lock

✅ usage_bucket_30m 表: 1498 条 Claude 记录
  表字段: source, model, hour_start, project_hash, project_label, project_ref, input_tokens, cache_read_tokens, output_tokens, reasoning_output_tokens...
  最近 5 天的记录分布:
    2026-06-14: 15 条
    2026-06-13: 29 条
    2026-06-12: 37 条
    2026-06-11: 31 条
    2026-06-10: 13 条

✅ usage_event 表: 170244 条 Claude 记录
  表字段: event_key, source, model, event_at, hour_start, input_tokens, cache_read_tokens, output_tokens, reasoning_output_tokens, total_tokens...
  最近 30 天的每日记录 (10 天有数据):
    2026-06-14: 1301 条事件, 282,755,784 tokens
    2026-06-13: 5529 条事件, 1,143,268,791 tokens
    2026-06-12: 4291 条事件, 672,183,309 tokens
    2026-06-11: 2731 条事件, 390,805,747 tokens
    2026-06-10: 961 条事件, 138,685,830 tokens
    2026-06-09: 2107 条事件, 375,237,341 tokens
    2026-06-08: 826 条事件, 136,424,534 tokens
    2026-06-07: 654 条事件, 146,072,599 tokens
    2026-06-06: 207 条事件, 28,743,847 tokens
    2026-06-05: 1487 条事件, 252,725,981 tokens

✅ 项目数: 1 个不同项目
  Top 5 项目:
    ... (None): 1498 条记录

============================================================
诊断 ccr-db 数据源
============================================================
数据库路径: C:\Users\lyh\.ccr\ccr.db
❌ 数据库文件不存在！

============================================================
诊断总结
============================================================
⚠️  llmusage 有数据，但 ccr-db 为空

建议：
  1. 检查 claude_tool_calls 表的初始化逻辑
  2. 触发工具调用记录导入
```
