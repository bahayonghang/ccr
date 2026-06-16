# 技术设计：修复 Claude Code Usage Insight 面板数据显示为空

## 问题分析

### 已确认的事实

1. ✅ llmusage 数据源正常（170,244 条记录）
2. ✅ 代码使用了正确的表名（`usage_bucket_30m` / `usage_event`）
3. ✅ Hero 三卡显示正常（聚合数据）
4. ❌ 三个 Tab 的图表全部为空（明细数据）
5. ❌ ccr-db 不存在（`C:\Users\lyh\.ccr\ccr.db`）

### 根因推测

#### 问题 1：llmusage Dashboard 查询返回空但未报错（高概率）

**症状**：
- `insight` 聚合查询成功（Hero 三卡有数据）
- `daily_trend` / `cost_breakdown` 等明细查询返回空数组
- 前端 `store.daily.error` 为 null（无错误状态）

**可能原因**：
1. **Schema 版本兼容性问题**：`MIN_SUPPORTED_SCHEMA_VERSION` 检查通过，但某些字段/表结构不兼容
2. **SQL 查询逻辑错误**：WHERE 条件过滤掉了所有数据
3. **日期窗口计算错误**：`today_window()` / `month_window()` 返回未来日期
4. **时区问题**：`localtime` 转换导致日期不匹配

#### 问题 2：ccr-db 未初始化（已确认）

**症状**：
- 数据库文件不存在
- "行为分析" Tab 完全为空（Tool Heatmap / Top Tools / Top Sessions）

**原因**：
- ccr-db 依赖 `crates/ccr-db` 的 migration 初始化
- Tauri 应用启动时未触发 migration
- 或者 migration 逻辑有 bug

#### 问题 3：错误处理不完善（待验证）

**症状**：
- 查询失败但前端显示空状态而非错误状态
- Console 无错误日志

**原因**：
- `spawn_blocking` 中的错误被吞掉
- `map_err` 转换后的错误字符串未传递到前端
- 前端 store 的 `loadError` 计算逻辑有漏洞

---

## 修复方案

### 方案 1：增强 llmusage Dashboard 查询健壮性

#### 1.1 添加详细错误日志

**目标**：让每个查询步骤的错误都能被捕获和上报。

**位置**：`ccr-ui/src-tauri/src/commands/claude_observer.rs`

**修改**：
```rust
// 现有代码
pub async fn claude_observer_daily_trend(
    state: State<'_, AppState>,
    days: Option<i64>,
) -> Result<Vec<DailyPoint>, String> {
    let llmusage = state.llmusage.clone();
    let days = days.unwrap_or(30).clamp(1, 365);

    tokio::task::spawn_blocking(move || -> Result<Vec<DailyPoint>, String> {
        // 添加详细日志
        debug!("[claude_observer] daily_trend: days={}", days);
        
        let dashboard = llmusage
            .dashboard()
            .map_err(|e| {
                let err = format!("Dashboard open error: {e}");
                error!("[claude_observer] {}", err);
                err
            })?;
        
        // 添加数据库路径日志
        debug!("[claude_observer] DB path: {:?}", llmusage.paths().db_path);
        
        let trends = dashboard
            .trends_daily(&filter)
            .map_err(|e| {
                let err = format!("Trends query error: {e}");
                error!("[claude_observer] {}", err);
                err
            })?;
        
        debug!("[claude_observer] daily_trend result: {} points", trends.len());
        Ok(trends.into_iter().map(...).collect())
    })
    .await
    .map_err(|e| format!("Task join error: {e}"))?
}
```

#### 1.2 验证查询结果非空

**目标**：当查询返回空时，检查是否是预期行为。

**修改**：
```rust
let trends = dashboard.trends_daily(&filter)?;

// 如果返回空，检查是否真的没有数据
if trends.is_empty() {
    // 回退查询：检查是否有任何 usage_event 记录
    let total_count = dashboard.query_event_count(&filter, None)?;
    if total_count > 0 {
        warn!("[claude_observer] daily_trend returned empty but {} events exist", total_count);
        // 可能是日期窗口问题，尝试扩大窗口
        let wider_filter = build_filter(
            Some("claude".to_string()),
            None,
            Some((Local::now() - Duration::days(90)).format("%Y-%m-%d").to_string()),
            Some(Local::now().format("%Y-%m-%d").to_string()),
        )?;
        let wider_trends = dashboard.trends_daily(&wider_filter)?;
        debug!("[claude_observer] wider window result: {} points", wider_trends.len());
    }
}
```

#### 1.3 修复日期窗口计算

**目标**：确保日期窗口不会过滤掉有效数据。

**位置**：`ccr-ui/src-tauri/src/commands/claude_observer.rs`

**问题**：当前代码使用 `Local::now()`，可能与数据库中的时间戳时区不一致。

**修改**：
```rust
fn today_window() -> (String, String) {
    let today = Local::now().date_naive();
    let s = today.format("%Y-%m-%d").to_string();
    debug!("[claude_observer] today_window:  to {}", s, s);
    (s.clone(), s)
}

fn month_window() -> (String, String) {
    let today = Local::now().date_naive();
    let first = today
        .with_day(1)
        .unwrap_or(today)
        .format("%Y-%m-%d")
        .to_string();
    let last = today.format("%Y-%m-%d").to_string();
    debug!("[claude_observer] month_window: {} to {}", first, last);
    (first, last)
}
```

### 方案 2：初始化 ccr-db 数据库

#### 2.1 确保数据库文件在应用启动时创建

**目标**：在 `AppState` 初始化时创建 ccr-db 数据库。

**位置**：`ccr-ui/src-tauri/src/main.rs` 或 `state.rs`

**修改**：
```rust
// 在 AppState::new() 或 main() 中添加
pub fn ensure_ccr_db(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        info!("[ccr-db] Database not found, creating: {:?}", db_path);
        
        // 确保父目录存在
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create DB parent dir: {e}"))?;
        }
        
        // 创建空数据库文件
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| format!("Failed to create DB: {e}"))?;
        
        // 运行 migration
        ccr_db::database::run_migrations(&conn)
            .map_err(|e| format!("Failed to run migrations: {e}"))?;
        
        info!("[ccr-db] Database initialized successfully");
    }
    Ok(())
}
```

#### 2.2 修改 AppState 初始化逻辑

**位置**：`ccr-ui/src-tauri/src/state.rs`

**修改**：
```rust
impl AppState {
    pub fn new() -> Result<Self, String> {
        // ... 现有代码 ...
        
        // 确保 ccr-db 存在
        let ccr_db_path = home_dir
            .join(".ccr")
            .join("ccr.db");
        ensure_ccr_db(&ccr_db_path)?;
        
        let db_pool = r2d2::Pool::builder()
            .max_size(16)
            .build(r2d2_sqlite::SqliteConnectionManager::file(&ccr_db_path))
            .map_err(|e| format!("DB pool creation failed: {e}"))?;
        
        // ... 现有代码 ...
    }
}
```

#### 2.3 添加 claude_tool_calls 表 migration

**目标**：确保 `claude_tool_calls` 表在数据库中存在。

**位置**：`crates/ccr-db/src/database/migrations.rs`（或类似文件）

**添加**：
```rust
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS claude_tool_calls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            cost_usd REAL NOT NULL DEFAULT 0.0,
            called_at TEXT NOT NULL,
            project_path TEXT,
            project_hash TEXT,
            model TEXT,
            input_tokens INTEGER DEFAULT 0,
            output_tokens INTEGER DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        
        CREATE INDEX IF NOT EXISTS idx_claude_tool_calls_session
            ON claude_tool_calls(session_id);
        CREATE INDEX IF NOT EXISTS idx_claude_tool_calls_called_at
            ON claude_tool_calls(called_at);
        CREATE INDEX IF NOT EXISTS idx_claude_tool_calls_tool_name
            ON claude_tool_calls(tool_name);
        "#
    )?;
    Ok(())
}
```

### 方案 3：优化前端错误处理

#### 3.1 改进空状态提示

**目标**：区分"数据为空"和"加载失败"两种状态。

**位置**：`ccr-ui/src/components/claude-observer/UsageInsightPanel.vue`

**修改**：
```vue
<AsyncStatePanel
  v-else-if="state === 'empty'"
  state="empty"
  :title="$t('claudeCode.observer.empty.noUsage')"
  :description="emptyDescription"
  icon="Database"
  :action-label="$t('common.retry')"
  action-icon="RefreshCw"
  @action="refresh"
/>
```

```typescript
const emptyDescription = computed(() => {
  // 如果有错误，显示错误信息
  if (loadError.value) {
    return `${t('claudeCode.observer.empty.loadError')}: ${loadError.value}`
  }
  // 否则显示友好提示
  return t('claudeCode.observer.empty.noUsageDesc')
})
```

#### 3.2 添加手动刷新按钮

**位置**：`UsageInsightPanel.vue` 的 header 部分

**添加**：
```vue
<div class="usage-insight-panel__head-actions">
  <button
    type="button"
    class="usage-insight-panel__refresh-btn"
    :disabled="isLoading"
    @click="refresh"
  >
    <SIcon name="RefreshCw" size="w-4 h-4" />
    {{ $t('common.refresh') }}
  </button>
  <RouterLink to="/usage" class="usage-insight-panel__full-link">
    {{ $t('claudeCode.observer.fullDashboardLink') }}
  </RouterLink>
</div>
```

#### 3.3 添加详细错误日志

**位置**：`ccr-ui/src/stores/claudeObserver.ts`

**修改**：
```typescript
const runWith = async <T>(slot: { value: Slice<T> }, loader: () => Promise<T>) => {
  if (!isTauriRuntime()) {
    slot.value = { loading: false, error: null, data: null }
    return
  }
  slot.value = { loading: true, error: null, data: slot.value.data }
  try {
    const data = await loader()
    slot.value = { loading: false, error: null, data }
    
    // 添加调试日志
    if (Array.isArray(data) && data.length === 0) {
      logger.warn('[claudeObserver] Query returned empty array')
    }
  } catch (err) {
    const errorMsg = toMessage(err)
    slot.value = { loading: false, error: errorMsg, data: slot.value.data }
    logger.error('[claudeObserver] load failed', { error: errorMsg, stack: err })
  }
}
```

### 方案 4：添加降级方案

#### 4.1 当 daily_trend 返回空时，从 insight 推断

**目标**：即使明细数据为空，也能显示基本趋势。

**位置**：`ccr-ui/src/stores/claudeObserver.ts`

**修改**：
```typescript
const fetchDaily = async (days = 30) => {
  await runWith(daily, () => api.dailyTrend(days))
  
  // 降级逻辑：如果返回空但 insight 有数据，生成近似趋势
  if (daily.value.data && daily.value.data.length === 0 && insight.value.data) {
    const i = insight.value.data
    if (i.month_value_usd > 0) {
      logger.info('[claudeObserver] daily_trend empty, using fallback from insight')
      // 将本月总数均分到 30 天
      const avgCostPerDay = i.month_value_usd / 30
      const today = new Date()
      daily.value.data = Array.from({ length: 30 }, (_, index) => {
        const date = new Date(today)
        date.setDate(date.getDate() - (29 - index))
        return {
          date: date.toISOString().split('T')[0],
          cost_usd: avgCostPerDay,
          input_tokens: Math.floor(i.month_tokens / 30),
          output_tokens: 0,
          cache_read_tokens: 0,
          cache_write_tokens: 0,
        }
      })
    }
  }
}
```

---

## 实现计划

### Phase 1：增强日志和错误处理（1-2 小时）

1. ✅ 在所有 `claude_observer_*` 命令中添加详细日志
2. ✅ 修改前端 store 记录空数组警告
3. ✅ 更新空状态提示文案

### Phase 2：初始化 ccr-db（1 小时）

1. ✅ 添加 `ensure_ccr_db()` 函数
2. ✅ 修改 AppState 初始化逻辑
3. ✅ 添加 `claude_tool_calls` 表 migration

### Phase 3：验证和测试（1 小时）

1. ✅ 运行 ccr-ui，查看 Tauri 日志
2. ✅ 验证 daily_trend 是否返回数据
3. ✅ 验证 ccr-db 是否成功创建
4. ✅ 手动调用所有 `claude_observer_*` 命令

### Phase 4：降级方案（可选，1 小时）

1. ✅ 实现从 insight 推断 daily_trend
2. ✅ 添加手动刷新按钮

---

## 验收标准

### 必须（Phase 1-3）

- [ ] Tauri 日志中有详细的 `[claude_observer]` 调试信息
- [ ] ccr-db 数据库文件成功创建（`C:\Users\lyh\.ccr\ccr.db`）
- [ ] `claude_tool_calls` 表存在且有正确的 schema
- [ ] "费用日历" Tab 显示 30 天趋势曲线（非空）
- [ ] "Token 详情" Tab 显示缓存统计（非空）
- [ ] 如果查询失败，前端显示错误信息而非空状态

### 可选（Phase 4）

- [ ] 当 daily_trend 返回空时，从 insight 生成近似趋势
- [ ] 手动刷新按钮可用

---

## 风险和备选方案

### 风险 1：llmusage schema 版本不兼容

**影响**：即使添加日志，查询仍然失败。

**备选方案**：
- 检查 `MIN_SUPPORTED_SCHEMA_VERSION` 是否正确
- 运行 `llmusage version` 查看实际版本
- 如果版本不匹配，升级 llmusage 或回退 ccr-ui

### 风险 2：ccr-db migration 失败

**影响**：数据库文件创建但表不存在。

**备选方案**：
- 手动执行 SQL 创建表
- 检查 `crates/ccr-db` 的 migration 逻辑
- 添加 migration 错误日志

### 风险 3：降级方案导致数据不准确

**影响**：用户看到的趋势图是近似值，不是真实数据。

**备选方案**：
- 在降级模式下显示警告提示
- 提供"查看完整数据"链接跳转到 `/usage`
