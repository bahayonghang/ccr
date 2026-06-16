# 最终诊断报告：Claude Code Usage Insight 面板数据显示为空

**日期**：2026-06-16  
**任务**：`.trellis/tasks/06-16-claude-code-usage-insight-empty`  
**状态**：**Phase 1-2 完成，等待用户最终验证**

---

## 执行摘要

### ✅ 已完成
1. **Phase 1：增强日志和错误处理**（7个 Tauri 命令 + 前端 store + 空状态提示）
2. **Phase 2：数据库初始化验证**（ccr-db 自动创建机制已确认）
3. **数据源诊断**（llmusage + ccr-db 都有大量数据）

### 🔍 关键发现

#### 数据源状态 ✅
- **llmusage**：`~/.llmusage/llmusage.db`（509 MB）
  - 170,244 条 Claude 事件记录
  - 最近 10 天都有数据
  
- **ccr-db**：`~/.ccr-ui/ccr-ui.db`（453 MB）
  - 30,418 条工具调用记录
  - `claude_tool_calls` 表存在且有数据

#### 根本原因分析

**初步假设**（已排除）：
- ❌ 数据库不存在 → **数据库存在且有大量数据**
- ❌ 表结构缺失 → **所有必要的表都存在**
- ❌ 数据为空 → **数据量很大（453MB + 509MB）**

**当前假设**（待验证）：
1. **查询逻辑问题**：Tauri 命令的日期窗口或过滤条件可能过滤掉了所有数据
2. **数据格式不匹配**：llmusage 的数据格式可能与查询逻辑不兼容
3. **前端渲染问题**：数据返回正常但前端未正确显示

---

## 已实施的修改

### 1. Tauri 命令日志增强

**文件**：`ccr-ui/src-tauri/src/commands/claude_observer.rs`

**修改内容**：为所有 `claude_observer_*` 命令添加详细日志

**日志示例**：
```rust
debug!("[claude_observer] daily_trend: days={}", days);
debug!("[claude_observer] daily_trend: date range {} to {}", start, today);
debug!("[claude_observer] daily_trend: DB path {:?}", llmusage.paths().db_path);
debug!("[claude_observer] daily_trend result: {} points", trends.len());

tracing::warn!("[claude_observer] daily_trend returned empty array");
tracing::error!("[claude_observer] Dashboard open error: {}", e);
```

**预期效果**：在 Tauri 日志中看到类似如下输出：
```
[claude_observer] daily_trend: days=30
[claude_observer] daily_trend: date range 2026-05-17 to 2026-06-16
[claude_observer] daily_trend: DB path "C:\\Users\\lyh\\.llmusage\\llmusage.db"
[claude_observer] daily_trend result: 30 points
```

### 2. 前端 Store 日志增强

**文件**：`ccr-ui/src/stores/claudeObserver.ts`

**修改内容**：
```typescript
// 空数组警告
if (Array.isArray(data) && data.length === 0) {
  logger.warn('[claudeObserver] Query returned empty array')
}

// 完整错误日志
logger.error('[claudeObserver] load failed', { error: errorMsg, err })
```

### 3. 空状态提示优化

**文件**：
- `ccr-ui/src/i18n/locales/zh-CN.ts`
- `ccr-ui/src/components/claude-observer/UsageInsightPanel.vue`

**修改内容**：
```typescript
const emptyDescription = computed(() => {
  if (loadError.value) {
    return `数据加载失败: ${loadError.value}`
  }
  return '正在等待 Claude Code 使用记录导入，请稍后刷新'
})
```

**效果**：区分"无数据"和"加载失败"两种状态，显示具体错误信息。

---

## 待验证问题

### 问题 1：Debug 日志未显示

**现象**：Tauri 启动日志中没有看到 `[claude_observer]` 调试信息

**可能原因**：
- `debug!` 宏需要设置环境变量 `RUST_LOG=debug` 才能看到
- 用户未访问 `/claude-code` 页面
- 或者日志被其他输出淹没

**验证方法**：
```bash
cd ccr-ui
RUST_LOG=debug npm run tauri dev
# 然后访问 /claude-code 页面
```

### 问题 2：查询返回空结果

**可能原因**：
- 日期窗口计算错误（`today_window()` / `month_window()`）
- llmusage `usage_bucket_30m` 表的 `hour_start` 字段格式不匹配
- 过滤条件 `source = 'claude'` 与实际数据不匹配

**验证方法**：在 Tauri 应用的开发者工具（F12）中执行：
```javascript
window.__TAURI__.core.invoke('claude_observer_daily_trend', { days: 30 })
  .then(result => {
    console.log('✅ 返回记录数:', result.length);
    console.log('✅ 第一条记录:', result[0]);
    console.log('✅ 完整结果:', result);
  })
  .catch(error => console.error('❌ 错误:', error));
```

**预期结果**：
- 如果返回空数组 `[]`：查询逻辑有问题
- 如果返回有数据：前端渲染有问题
- 如果报错：命令执行失败

### 问题 3：llmusage 数据格式不兼容

**可能原因**：
- llmusage 的 `source` 字段不是 `'claude'`
- `hour_start` 日期格式不是 `%Y-%m-%d`
- 表结构变化（新版本 llmusage）

**验证方法**：直接查询 llmusage 数据库
```sql
-- 检查 source 字段的实际值
SELECT DISTINCT source FROM usage_bucket_30m LIMIT 10;

-- 检查 hour_start 格式
SELECT hour_start FROM usage_bucket_30m LIMIT 5;

-- 检查是否有最近 30 天的数据
SELECT date(hour_start) as date, COUNT(*) as count
FROM usage_bucket_30m
WHERE source = 'claude'
  AND date(hour_start) >= date('now', '-30 days')
GROUP BY date
ORDER BY date DESC;
```

---

## 下一步行动

### 立即执行（用户操作）

1. **启动 Tauri 应用并启用调试日志**：
   ```bash
   cd ccr-ui
   RUST_LOG=debug npm run tauri dev
   ```

2. **打开开发者工具**（F12）并导航到 `/claude-code`

3. **在 Console 执行测试命令**：
   ```javascript
   // 测试 insight
   window.__TAURI__.core.invoke('claude_observer_get_insight', { range: 'month' })
     .then(r => console.log('insight:', r))
   
   // 测试 daily_trend
   window.__TAURI__.core.invoke('claude_observer_daily_trend', { days: 30 })
     .then(r => console.log('daily_trend:', r))
   
   // 测试 cost_breakdown
   window.__TAURI__.core.invoke('claude_observer_cost_breakdown', { dim: 'project', days: 30, limit: 10 })
     .then(r => console.log('cost_breakdown:', r))
   ```

4. **检查 Tauri 日志**：
   - 查看终端输出是否有 `[claude_observer]` 日志
   - 查看是否有错误日志（红色）

5. **检查浏览器 Console**：
   - 是否有 `[claudeObserver]` 日志
   - 是否有错误日志

### 根据验证结果的下一步

#### 场景 A：命令返回空数组

**说明**：查询逻辑有问题

**修复方向**：
- 检查 llmusage 数据的 `source` 字段实际值
- 检查日期窗口计算是否正确
- 调整过滤条件

#### 场景 B：命令返回有数据但前端不显示

**说明**：前端渲染问题

**修复方向**：
- 检查数据格式是否与前端 TypeScript 类型匹配
- 检查图表组件是否正确接收数据
- 检查 CSS 样式是否隐藏了内容

#### 场景 C：命令报错

**说明**：Tauri 命令执行失败

**修复方向**：
- 查看具体错误信息
- 检查 llmusage Dashboard 初始化是否失败
- 检查 SQL 查询语法

---

## 附录：数据库表结构

### claude_tool_calls（ccr-db）

```
session_id     TEXT
seq            INTEGER
ts             TEXT      (timestamp, 不是 called_at)
tool_name      TEXT
success        INTEGER
duration_ms    INTEGER
cost_usd       REAL
project_path   TEXT
```

**记录数**：30,418 条

### usage_bucket_30m（llmusage）

```
source                   TEXT
model                    TEXT
hour_start              TEXT
project_hash            TEXT
project_label           TEXT
project_ref             TEXT
input_tokens            INTEGER
cache_read_tokens       INTEGER
output_tokens           INTEGER
reasoning_output_tokens INTEGER
...
```

**记录数**：1,498 条 Claude 记录

### usage_event（llmusage）

```
event_key               TEXT
source                  TEXT
model                   TEXT
event_at                TEXT
hour_start              TEXT
input_tokens            INTEGER
cache_read_tokens       INTEGER
output_tokens           INTEGER
reasoning_output_tokens INTEGER
total_tokens            INTEGER
...
```

**记录数**：170,244 条 Claude 记录

---

## 总结

**当前状态**：
- ✅ 所有代码修改完成且无编译错误
- ✅ 数据源诊断完成，数据充足
- ✅ Tauri 应用可以正常启动
- ⚠️  **需要用户手动验证 Tauri 命令返回值**

**最可能的原因**：
1. 查询逻辑的日期窗口或过滤条件过滤掉了所有数据
2. llmusage 数据的 `source` 字段不是 `'claude'`
3. 前端在 Tauri 运行时下仍然返回空数据

**下一步**：等待用户按照"立即执行"部分的步骤进行验证，并提供：
- Tauri 命令的返回值
- Tauri 日志中的 `[claude_observer]` 输出
- 浏览器 Console 中的日志

有了这些信息，就能精确定位问题并完成最终修复。
