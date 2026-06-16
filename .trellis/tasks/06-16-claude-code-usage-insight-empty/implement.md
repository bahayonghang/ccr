# 实现计划：修复 Claude Code Usage Insight 面板数据显示为空

## 实现顺序

按照风险和依赖关系，分 4 个 Phase 依次实现：

---

## Phase 1：增强日志和错误处理（P0）

**目标**：让所有错误都能被捕获、记录和上报，方便定位问题。

### 1.1 增强 Tauri 命令日志

**文件**：`ccr-ui/src-tauri/src/commands/claude_observer.rs`

**任务**：
- [ ] 在 `claude_observer_daily_trend` 添加 debug/error 日志
- [ ] 在 `claude_observer_cost_breakdown` 添加 debug/error 日志
- [ ] 在 `claude_observer_cache_stats` 添加 debug/error 日志
- [ ] 在 `claude_observer_tool_heatmap` 添加 debug/error 日志
- [ ] 在 `claude_observer_top_tools` 添加 debug/error 日志
- [ ] 在 `claude_observer_top_sessions` 添加 debug/error 日志

**日志格式**：
```rust
debug!("[claude_observer] daily_trend: days={}", days);
debug!("[claude_observer] DB path: {:?}", llmusage.paths().db_path);
debug!("[claude_observer] daily_trend result: {} points", trends.len());
error!("[claude_observer] Trends query error: {}", e);
```

**验证**：
```bash
# 运行 Tauri dev 模式，查看日志
cargo tauri dev
# 或查看日志文件
type %LOCALAPPDATA%\com.ccr.dev\logs\*.log | Select-String -Pattern "claude_observer"
```

### 1.2 增强前端 store 日志

**文件**：`ccr-ui/src/stores/claudeObserver.ts`

**任务**：
- [ ] 在 `runWith()` 中添加空数组警告
- [ ] 在 `fetchAll()` 开始/完成时记录日志
- [ ] 在错误捕获时记录完整错误信息（包括 stack）

**日志格式**：
```typescript
logger.info('[claudeObserver] fetchAll 开始')
logger.warn('[claudeObserver] Query returned empty array')
logger.error('[claudeObserver] load failed', { error: errorMsg, stack: err })
```

**验证**：
```javascript
// 浏览器 Console
localStorage.setItem('debug', 'ccr:*')
// 刷新页面，查看日志
```

### 1.3 优化空状态提示

**文件**：`ccr-ui/src/components/claude-observer/UsageInsightPanel.vue`

**任务**：
- [ ] 区分"数据为空"和"加载失败"两种状态
- [ ] 当有错误时，显示错误信息
- [ ] 更新 i18n 文案

**代码**：
```typescript
const emptyDescription = computed(() => {
  if (loadError.value) {
    return `加载失败: ${loadError.value}`
  }
  return t('claudeCode.observer.empty.noUsageDesc')
})
```

**i18n 更新**：
```json
// ccr-ui/src/i18n/locales/zh-CN.ts
"claudeCode.observer.empty": {
  "noUsage": "暂无数据",
  "noUsageDesc": "正在等待 Claude Code 使用记录导入，请稍后刷新",
  "loadError": "数据加载失败"
}
```

**验证**：
- 模拟错误场景，查看是否显示错误信息
- 模拟空数据场景，查看是否显示友好提示

---

## Phase 2：初始化 ccr-db 数据库（P0）

**目标**：确保 ccr-db 数据库文件和 `claude_tool_calls` 表在应用启动时存在。

### 2.1 添加数据库初始化函数

**文件**：`ccr-ui/src-tauri/src/state.rs`（或新建 `ccr-ui/src-tauri/src/db_init.rs`）

**任务**：
- [ ] 实现 `ensure_ccr_db(db_path: &Path) -> Result<(), String>` 函数
- [ ] 检查数据库文件是否存在
- [ ] 如果不存在，创建空数据库并运行 migration
- [ ] 添加详细日志

**代码**：
```rust
pub fn ensure_ccr_db(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        info!("[ccr-db] Database not found, creating: {:?}", db_path);
        
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create DB parent dir: {e}"))?;
        }
        
        let conn = rusqlite::Connection::open(db_path)
            .map_err(|e| format!("Failed to create DB: {e}"))?;
        
        run_ccr_db_migrations(&conn)
            .map_err(|e| format!("Failed to run migrations: {e}"))?;
        
        info!("[ccr-db] Database initialized successfully");
    } else {
        info!("[ccr-db] Database found: {:?}", db_path);
    }
    Ok(())
}
```

**验证**：
```bash
# 删除现有数据库（如果存在）
del %USERPROFILE%\.ccr\ccr.db
# 启动应用
cargo tauri dev
# 检查数据库是否创建
dir %USERPROFILE%\.ccr\ccr.db
```

### 2.2 添加 claude_tool_calls 表 migration

**文件**：`ccr-ui/src-tauri/src/state.rs`（或 `db_init.rs`）

**任务**：
- [ ] 实现 `run_ccr_db_migrations(conn: &Connection) -> Result<(), String>` 函数
- [ ] 创建 `claude_tool_calls` 表
- [ ] 创建必要的索引
- [ ] 添加其他必要的表（如 `user_settings`）

**代码**：
```rust
fn run_ccr_db_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- claude_tool_calls 表：工具调用记录
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
        
        -- user_settings 表：用户设置（包括订阅信息）
        CREATE TABLE IF NOT EXISTS user_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#
    )?;
    Ok(())
}
```

**验证**：
```bash
# 启动应用后，检查表结构
sqlite3 %USERPROFILE%\.ccr\ccr.db ".schema claude_tool_calls"
sqlite3 %USERPROFILE%\.ccr\ccr.db "SELECT * FROM sqlite_master WHERE type='table';"
```

### 2.3 修改 AppState 初始化逻辑

**文件**：`ccr-ui/src-tauri/src/state.rs`

**任务**：
- [ ] 在 `AppState::new()` 中调用 `ensure_ccr_db()`
- [ ] 确保在创建连接池之前数据库已初始化
- [ ] 处理初始化失败的情况

**代码**：
```rust
impl AppState {
    pub fn new() -> Result<Self, String> {
        // ... 现有代码 ...
        
        // 确保 ccr-db 存在
        let ccr_db_path = dirs::home_dir()
            .ok_or("Cannot resolve home directory")?
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

**验证**：
```bash
# 删除数据库
del %USERPROFILE%\.ccr\ccr.db
# 启动应用
cargo tauri dev
# 检查日志是否有 "[ccr-db] Database initialized successfully"
# 检查数据库是否创建
dir %USERPROFILE%\.ccr\ccr.db
```

---

## Phase 3：验证和测试（P0）

**目标**：确认修复生效，所有数据都能正常显示。

### 3.1 手动测试 Tauri 命令

**工具**：浏览器 DevTools Console

**任务**：
- [ ] 测试 `claude_observer_get_insight`
- [ ] 测试 `claude_observer_daily_trend`
- [ ] 测试 `claude_observer_cost_breakdown` (project)
- [ ] 测试 `claude_observer_cost_breakdown` (model)
- [ ] 测试 `claude_observer_cache_stats`
- [ ] 测试 `claude_observer_tool_heatmap`
- [ ] 测试 `claude_observer_top_tools`
- [ ] 测试 `claude_observer_top_sessions`

**脚本**：
```javascript
// 在 Tauri 应用的 DevTools Console 执行
const test = async () => {
  const commands = [
    { name: 'claude_observer_get_insight', args: { range: 'today' } },
    { name: 'claude_observer_daily_trend', args: { days: 30 } },
    { name: 'claude_observer_cost_breakdown', args: { dim: 'project', days: 30, limit: 10 } },
    { name: 'claude_observer_cost_breakdown', args: { dim: 'model', days: 30, limit: 10 } },
    { name: 'claude_observer_cache_stats', args: {} },
    { name: 'claude_observer_tool_heatmap', args: { days: 30 } },
    { name: 'claude_observer_top_tools', args: { days: 30, limit: 10 } },
    { name: 'claude_observer_top_sessions', args: { limit: 10, by: 'cost' } },
  ]
  
  for (const cmd of commands) {
    try {
      const result = await window.__TAURI__.core.invoke(cmd.name, cmd.args)
      console.log(`✅ ${cmd.name}:`, result)
    } catch (error) {
      console.error(`❌ ${cmd.name}:`, error)
    }
  }
}

test()
```

**预期结果**：
- `daily_trend` 返回 ≥30 个数据点
- `cost_breakdown` 返回 ≥1 条记录
- `cache_stats` 返回有效的统计数据
- `tool_heatmap` / `top_tools` / `top_sessions` 返回空数组（ccr-db 暂无数据）

### 3.2 验证前端显示

**任务**：
- [ ] 刷新 Claude Code 页面（`/claude-code`）
- [ ] 检查 Hero 三卡是否显示
- [ ] 检查"费用日历" Tab 是否显示 30 天趋势曲线
- [ ] 检查"Token 详情" Tab 是否显示缓存统计
- [ ] 检查"行为分析" Tab 是否显示空状态（ccr-db 暂无数据）

**预期结果**：
- ✅ Hero 三卡显示正常
- ✅ "费用日历" Tab 显示趋势曲线（非空白虚线框）
- ✅ "Token 详情" Tab 显示缓存统计和趋势
- ✅ "行为分析" Tab 显示友好的空状态提示（而非错误）

### 3.3 检查日志

**任务**：
- [ ] 检查 Tauri 日志是否有 `[claude_observer]` 调试信息
- [ ] 检查浏览器 Console 是否有 `[claudeObserver]` 日志
- [ ] 确认无错误日志

**日志位置**：
```bash
# Tauri 日志（Windows）
type %LOCALAPPDATA%\com.ccr.dev\logs\*.log
# 或在 cargo tauri dev 的终端输出中查看

# 浏览器 Console
# 打开 DevTools (F12)，切换到 Console 标签页
```

**预期日志**：
```
[claude_observer] daily_trend: days=30
[claude_observer] DB path: "C:\\Users\\lyh\\.llmusage\\llmusage.db"
[claude_observer] daily_trend result: 30 points
[claudeObserver] fetchAll 开始
[claudeObserver] fetchAll 完成
```

---

## Phase 4：降级方案和优化（P1，可选）

**目标**：增强用户体验，即使部分数据缺失也能正常使用。

### 4.1 实现从 insight 推断 daily_trend

**文件**：`ccr-ui/src/stores/claudeObserver.ts`

**任务**：
- [ ] 在 `fetchDaily()` 中检查返回数据是否为空
- [ ] 如果为空且 `insight` 有数据，生成近似趋势
- [ ] 添加警告提示

**代码**：
```typescript
const fetchDaily = async (days = 30) => {
  await runWith(daily, () => api.dailyTrend(days))
  
  if (daily.value.data && daily.value.data.length === 0 && insight.value.data) {
    const i = insight.value.data
    if (i.month_value_usd > 0) {
      logger.warn('[claudeObserver] daily_trend empty, using fallback from insight')
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

**验证**：
- 模拟 `daily_trend` 返回空的场景
- 检查是否显示近似趋势

### 4.2 添加手动刷新按钮

**文件**：`ccr-ui/src/components/claude-observer/UsageInsightPanel.vue`

**任务**：
- [ ] 在 header 添加刷新按钮
- [ ] 绑定 `refresh()` 方法
- [ ] 添加 loading 状态

**代码**：
```vue
<div class="usage-insight-panel__head-actions">
  <button
    type="button"
    class="usage-insight-panel__refresh-btn"
    :disabled="isLoading"
    @click="refresh"
  >
    <SIcon
      name="RefreshCw"
      size="w-4 h-4"
      :class="{ 'animate-spin': isLoading }"
    />
    {{ $t('common.refresh') }}
  </button>
  <RouterLink to="/usage" class="usage-insight-panel__full-link">
    {{ $t('claudeCode.observer.fullDashboardLink') }}
  </RouterLink>
</div>
```

**CSS**：
```css
.usage-insight-panel__refresh-btn {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  border-radius: 0.65rem;
  border: 1px solid var(--color-border-default);
  padding: 0.45rem 0.85rem;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 0.8rem;
  font-weight: 600;
  transition: all var(--motion-subtle-duration);
}

.usage-insight-panel__refresh-btn:hover:not(:disabled) {
  color: var(--color-text-primary);
  border-color: rgb(var(--color-accent-primary-rgb) / 30%);
  background: rgb(var(--color-accent-primary-rgb) / 6%);
}

.usage-insight-panel__refresh-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
```

**验证**：
- 点击刷新按钮
- 检查是否重新拉取数据
- 检查 loading 状态是否生效

---

## 检查清单

### Phase 1 完成标准
- [ ] 所有 Tauri 命令都有详细日志
- [ ] 前端 store 有空数组警告和错误日志
- [ ] 空状态提示能区分"无数据"和"加载失败"
- [ ] i18n 文案已更新

### Phase 2 完成标准
- [ ] ccr-db 数据库文件自动创建
- [ ] `claude_tool_calls` 表存在且结构正确
- [ ] `user_settings` 表存在
- [ ] AppState 初始化逻辑正确

### Phase 3 完成标准
- [ ] 所有 `claude_observer_*` 命令手动测试通过
- [ ] "费用日历" Tab 显示趋势曲线
- [ ] "Token 详情" Tab 显示统计数据
- [ ] Tauri 和浏览器日志无错误

### Phase 4 完成标准（可选）
- [ ] 降级方案实现并测试
- [ ] 手动刷新按钮可用

---

## 回滚计划

如果修复后出现新问题，按以下顺序回滚：

1. **Phase 4 回滚**：删除降级方案和刷新按钮（前端修改）
2. **Phase 2 回滚**：删除 `ensure_ccr_db()` 调用（保留函数代码）
3. **Phase 1 回滚**：删除日志代码（git revert）

---

## 时间估算

- Phase 1：2 小时（日志和错误处理）
- Phase 2：1.5 小时（ccr-db 初始化）
- Phase 3：1 小时（验证和测试）
- Phase 4：1.5 小时（降级方案，可选）

**总计**：4.5-6 小时
