# CCR-UI 性能审计报告

> 日期: 2026-03-01 | 审计工具: Ultrawork (Dual Opus Architect Agents)

---

## 一、总览

| 维度 | 后端 (Rust/Axum) | 前端 (Vue 3/Vite) |
|------|------------------|-------------------|
| **核心问题** | 同步 I/O 阻塞异步线程 | 无统一轮询/缓存基础设施 |
| **严重级别** | P0: 异步线程饥饿风险 | P0: 网络资源浪费 |
| **影响范围** | ~30 个 handler 直接阻塞 | 4+ 独立轮询 timer |
| **估算改善** | P99 延迟从"无界"降至可控 | 网络请求减少 40-60% |

---

## 二、后端性能问题 (Rust/Axum)

### P0-1: 异步线程上直接执行同步 I/O [CRITICAL]

**问题**: ~30 个 handler 在 async 上下文中直接调用 `std::fs::read_to_string`、`fs::read_dir` 等阻塞操作，无 `spawn_blocking` 保护。Tokio worker 线程被阻塞后，整个服务器吞吐量急剧下降。

**受影响文件**:
- `handlers/mcp.rs` — 5 个 handler 直接调 `ClaudeConfigManager::read()`
- `handlers/agents.rs` — 6 个 handler 调 `MarkdownManager::list_files_with_folders()`
- `handlers/plugins.rs` — 5 个 handler 调 `GLOBAL_SETTINGS_CACHE.load()`
- `handlers/hooks.rs` — 5 个 handler 调 `GLOBAL_SETTINGS_CACHE.load()`
- `handlers/slash_commands.rs` — 5 个 handler 调 `MarkdownManager` + 缓存
- `handlers/claude_settings.rs` — `get_settings()` 一次调用含 2 次同步文件读取
- `handlers/usage.rs` — 8 个 handler 直接调 `database::with_connection()`

**修复方案**: 所有涉及同步 I/O 的 handler 必须用 `spawn_blocking` 包装。

```rust
// Before (mcp.rs):
pub async fn list_mcp_servers() -> ApiResult<...> {
    let manager = ClaudeConfigManager::default()?;
    let servers = manager.get_mcp_servers()?; // ❌ 阻塞 async 线程
    // ...
}

// After:
pub async fn list_mcp_servers() -> ApiResult<...> {
    let servers = tokio::task::spawn_blocking(|| {
        let manager = ClaudeConfigManager::default()?;
        manager.get_mcp_servers()
    })
    .await
    .map_err(|e| ApiError::internal(format!("Task join error: {}", e)))?
    .map_err(|e| ApiError::internal(format!("Read MCP servers failed: {}", e)))?;
    // ... ✅ 纯计算在 async 线程上
}
```

### P0-2: `import_usage` handler 严重阻塞 [CRITICAL]

**位置**: `handlers/usage.rs:351-375`

`import_usage()` 在 async 线程上直接运行完整的 `UsageImportService::import_platform()` 管道 (WalkDir + 文件读写 + 数据库批量插入)，无任何 `spawn_blocking`。

**修复**: 包裹为 `spawn_blocking`（< 1 小时工作量）。

### P1-1: 数据库双轨架构冗余

**位置**: `database/mod.rs:32-35`

同时维护 `Arc<Mutex<Connection>>` (legacy) 和 `r2d2 Pool` (new)。`with_connection()` 先尝试 pool 再 fallback 到全局 Mutex。

**修复方案**:
1. 删除 `DB_CONNECTION` 静态变量
2. 简化 `with_connection()` 为 pool-only 路径
3. 重写 `transaction()` 使用 pooled connection
4. 估计涉及 81 个调用点修改

```rust
// After (简化):
pub fn with_connection<F, T>(f: F) -> Result<T, DatabaseError>
where F: FnOnce(&Connection) -> Result<T, rusqlite::Error> {
    let pool = DB_POOL.get().ok_or(DatabaseError::NotInitialized)?;
    let conn = pool.get().map_err(|e| DatabaseError::PoolGet(e.to_string()))?;
    f(&conn).map_err(DatabaseError::Query)
}
```

### P1-2: 通用 async 缓存层

**问题**: 当前缓存仅覆盖 `ClaudeSettings`，其他大量配置文件读取无缓存（每次请求都读磁盘）。

**设计方案**: 通用 TTL 缓存，内置 `spawn_blocking` + `tokio::sync::RwLock`：

```rust
pub struct TtlCache<T: Clone + Send + Sync> {
    data: tokio::sync::RwLock<Option<(Arc<T>, Instant)>>,
    ttl: Duration,
}

impl<T: Clone + Send + Sync + 'static> TtlCache<T> {
    pub async fn get_or_load<F, E>(&self, loader: F) -> Result<Arc<T>, E>
    where F: FnOnce() -> Result<T, E> + Send + 'static, E: Send + 'static {
        // Fast path: read lock (cache hit)
        { let guard = self.data.read().await; /* ... */ }
        // Slow path: spawn_blocking(loader) + write lock
    }
}
```

**估算**: I/O 减少 ~80%，P50 延迟降低 1-5ms/请求。

### P1-3: 缓存返回 `Arc<T>` 避免深拷贝

当前 `cache/mod.rs:66` 返回 `cached.clone()` 做完整深拷贝。改为返回 `Arc<ClaudeSettings>` 后，cache hit 从深拷贝降为原子计数器递增。

### P2-1: 后台任务独立线程池

**问题**: usage import 和 session indexer 每 60s 运行一次，占用共享的 blocking 线程池资源，与请求 handler 竞争。

**方案**: 创建专用 runtime 隔离后台任务。

### P2-2: 考虑 async 数据库驱动 (长期)

替换 `rusqlite + r2d2` 为 `tokio-rusqlite` 或 `sqlx`，消除数据库操作的 `spawn_blocking` 需求。涉及 81 个调用点，建议在上述快速修复落地后再推进。

---

## 三、前端性能问题 (Vue 3/Vite)

### P0-1: 统一轮询调度器

**问题**: 4+ 独立轮询 timer 同时运行，无协调：

| 组件 | 间隔 | 目标 |
|------|------|------|
| `StatusHeader.vue:254` | 5s | 系统信息（折叠时也在轮询！） |
| `useBackendHealth.ts:40` | 15s | 健康检查 |
| `stores/usage.ts:329` | 30s | 仪表盘数据 |
| `stores/usage.ts:332` | 10min | 热力图 |
| `useWebSocket.ts:103` | 30s | WS 心跳 |

**修复**: 创建 `usePolledData` composable 统一管理：

```typescript
const { data: systemInfo } = usePolledData('system-info', getSystemInfo, {
  intervalMs: 5000,
  pauseWhenHidden: true,
  pauseWhen: () => isCollapsed.value, // 折叠时暂停
})
```

### P0-2: API 超时 + 请求取消

**问题**: `api/core.ts:46` 全局 600s 超时，无 AbortController 支持。用户导航离开后请求继续运行最长 10 分钟。

**修复**:
```typescript
// 默认超时降至 15s
timeout: 15000,

// 长耗时操作单独覆盖
export const pushSync = async (req, signal?: AbortSignal) => {
    return api.post('/sync/push', req, { timeout: 600000, signal })
}
```

### P1-1: 拆分巨型组件

| 组件 | 行数 | 建议 |
|------|------|------|
| `CheckinView.vue` | 2572+ | 拆为 4 个 Tab 子组件 + 1 个 shared composable |
| `SyncView.vue` | 1191+ | 拆为 config/folder/operation 三个子组件 |

```
CheckinView.vue (编排器, ~200 行)
  ├── CheckinAccountsTab.vue (~500 行)
  ├── CheckinProvidersTab.vue (~400 行)
  ├── CheckinRecordsTab.vue (~300 行)
  ├── CheckinImportExportTab.vue (~300 行)
  └── useCheckinState.ts (共享状态, ~200 行)
```

**估算**: re-render 开销降低 40-60%。

### P1-2: StatusHeader 折叠时停止轮询

`StatusHeader.vue:254` 每 5s 轮询系统信息，即使面板已折叠。

```typescript
// 监听折叠状态动态控制 timer
watch(isCollapsed, (collapsed) => {
  if (collapsed && refreshInterval) {
    clearInterval(refreshInterval)
    refreshInterval = null
  } else if (!collapsed) {
    loadSystemInfo()
    refreshInterval = setInterval(loadSystemInfo, 5000)
  }
})
```

### P1-3: Barrel Import 优化

`api/client.ts` 通过 `export *` 重导出 18 个模块的 ~150 个函数。任何组件 import 都触发全量解析。

```typescript
// Before:
import { listCheckinProviders, ... } from '@/api/client'

// After (直接从模块导入):
import { listCheckinProviders, ... } from '@/api/modules/checkin'
```

### P1-4: 统一缓存策略

| Store | 当前策略 | 建议 |
|-------|----------|------|
| `configs.ts` | 5min TTL | ✅ 保留 |
| `commands.ts` | 无缓存 | 添加 2min TTL |
| `usage.ts` | 30s 自动刷新 | ✅ 保留 |
| `skills.ts` | 无 TTL | 添加 5min TTL |

提取通用 `useCachedFetch<T>` composable 统一管理。

### P2-1: Backdrop-filter 性能优化

30+ 处 `backdrop-filter: blur()` 在滚动时触发 GPU 重渲染。对 always-visible 元素（sidebar、top bar）添加降级策略：

```css
@media (prefers-reduced-motion: reduce) {
  .sidebar-glass {
    backdrop-filter: none;
    background: rgb(26 10 32 / 95%);
  }
}
```

### P2-2: CheckinView 从 KeepAlive 移除

`MainLayout.vue:331` 将 2500+ 行的 CheckinView 纳入 KeepAlive 缓存，内存开销过大。建议移除，改为每次挂载时重新加载（5 个并行 API 调用延迟可接受）。

---

## 四、优先级路线图

```
Phase 1 (1-3 天) — 紧急修复
  [P0] 后端: 30 个 handler 添加 spawn_blocking
  [P0] 后端: import_usage handler 修复
  [P0] 前端: 默认超时降至 15s + AbortController
  [P0] 前端: StatusHeader 折叠时停止轮询

Phase 2 (1-2 周) — 架构改善
  [P1] 后端: 淘汰 legacy DB_CONNECTION
  [P1] 后端: 通用 async TtlCache 实现
  [P1] 后端: 缓存层返回 Arc<T>
  [P1] 前端: 统一轮询调度器
  [P1] 前端: 拆分 CheckinView/SyncView
  [P1] 前端: 统一缓存层 composable
  [P1] 前端: Barrel import → Direct module import

Phase 3 (长期) — 深度优化
  [P2] 后端: 后台任务独立线程池
  [P2] 后端: 评估 async DB 驱动 (tokio-rusqlite/sqlx)
  [P2] 前端: Backdrop-filter 性能降级
  [P2] 前端: CheckinView 移出 KeepAlive
  [P2] 前端: 添加 Suspense 边界
```

---

## 五、性能提升估算

| 修复项 | 预期改善 |
|--------|---------|
| P0 后端: spawn_blocking 全覆盖 | P99 延迟从"潜在无界"降至"受 blocking pool 约束" |
| P1 后端: DB 统一 | ~2-5% 吞吐提升 + 消除 Mutex 风险 |
| P1 后端: 通用缓存层 | 文件 I/O 减少 ~80%，P50 延迟降低 1-5ms/请求 |
| P1 后端: Arc 缓存 | 热路径内存分配减少 30-50% |
| P0+P1 前端: 统一轮询 | 网络请求减少 40-60%，消除折叠状态下无效请求 |
| P1 前端: 组件拆分 | CheckinView re-render 开销降低 40-60% |
| P1 前端: 超时+取消 | 消除导航后的"僵尸请求"，资源释放及时 |
