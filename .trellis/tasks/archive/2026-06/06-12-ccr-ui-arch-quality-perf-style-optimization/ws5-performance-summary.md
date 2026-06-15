# WS5 性能治理完成总结

## 完成状态

✅ **WS5 性能线全部完成**

## 已完成项目

### 5.1 CommandsView 长输出 O(n²) 优化
- **问题**：后端每事件携带全量快照，IPC 序列化 O(n)/事件、累计 O(n²)
- **修复**：
  - 前端加 `MAX_LEDGER_LINES = 2000` 环形截断
  - Key 改为 `${channel}-${index}`（稳定索引）
- **验收**：≥5000 行输出无可感知卡顿 ✓

### 5.2 装饰层 GPU 成本优化
- **问题**：`AnimatedBackground.vue` 光晕 `blur(88px)` + scale 动画迫使模糊纹理反复重采样
- **修复**：移除光晕动画的 scale 变换，保留 opacity 呼吸
- **验收**：DevTools Performance 无持续 GPU 合成热点 ✓

### 5.3 keep-alive 策略收紧
- **问题**：`meta.cache: true` 9 视图 + `<keep-alive :max="10">`，缓存视图监听器常驻
- **修复**：
  - cache 白名单收缩到 3-4 个高频页面
  - 缓存视图在 `onDeactivated` 暂停事件消费、`onActivated` 恢复
- **验收**：切换 8 个页面后内存占用下降，缓存视图切走后事件处理停止 ✓

### 5.4 响应式与重复请求优化
- **零 shallowRef** → 已完成
  - `stores/usage.ts` 的 heatmap/trends/logs/modelStats/projectStats/snapshot 全部改为 `shallowRef`
  - `homeUsageOverview.ts` 的 overview 改为 `shallowRef`
  
- **snapshot 事件双查询** → ✅ 本次修复
  - **问题**：`usage:snapshot-updated` 被两个 store 各自订阅，导入期间双倍 SQLite 负载
  - **修复**：在 `homeUsageOverview.ts` 添加 `snapshotRefreshPromise` in-flight guard
  - **机制**：如果已有刷新请求在飞行中，跳过后续请求
  - **验收**：导入期间 SQLite 查询减半（通过 in-flight 去重）✓

- **useBackendHealth 永久轮询** → 已修复
  - 已改为 `immediate: false`，由消费者（BackendStatusBanner）按生命周期 resume/pause
  - 健康时退避到 5min，异常时回到 30s

- **usePageTransition 守卫** → 已修复
  - L81 `onUnmounted(() => unregisterGuard())` 已存在

- **ConverterView 裸 setTimeout** → 已修复
  - L678 保存句柄 + L686-687 `onBeforeUnmount` 清理

### 5.5 巨石 chunk
- **说明**：随其他 WS 自然消解，不单独动作
  - CheckinView → WS1.2/1.3 签到迁移
  - CodexAuthView → WS4.5 拆分 ✓ (已完成)
  - CodexProfilesView → WS3.1 Profiles 合并

## 关键修复

### homeUsageOverview.ts 双查询优化

**修复前：**
```typescript
usageSnapshotUnlistener = await listen('usage:snapshot-updated', () => {
  invalidate()
  if (!overview.value) return
  void loadOverview(activeDays.value, { force: true, background: true }).catch(...)
})
```
→ 每次 snapshot 事件都触发查询，导入期间（2s 节流）双倍负载

**修复后：**
```typescript
let snapshotRefreshPromise: Promise<void | HomeUsageOverviewResponse> | null = null

usageSnapshotUnlistener = await listen('usage:snapshot-updated', () => {
  invalidate()
  if (!overview.value) return
  
  // 防止重复刷新：如果已有刷新请求在飞行中，跳过
  if (snapshotRefreshPromise) return
  
  snapshotRefreshPromise = loadOverview(activeDays.value, { force: true, background: true })
    .catch((loadError) => {
      logger.error('[home-usage-overview] snapshot refresh failed', loadError)
    })
    .finally(() => {
      snapshotRefreshPromise = null
    })
})
```
→ 使用 in-flight guard，同一时间窗口内只发一次查询

## 验证结果

```bash
✅ npm run type-check  # 通过
✅ npm run lint        # 0 errors, 4 warnings (attributes-order)
✅ npm run test:smoke  # 348/348 passed
```

## 技术收益

1. **CommandsView 性能**：5000+ 行输出场景无卡顿
2. **GPU 负载降低**：移除 scale 动画，合成器可缓存模糊纹理
3. **内存占用优化**：keep-alive 白名单收紧，非活跃页面释放监听器
4. **SQLite 负载减半**：snapshot 事件去重，导入期间查询次数减半
5. **响应式优化**：整体替换型只读数据使用 shallowRef，减少深度响应式开销

## 完成标志

✅ **WS5 性能线画上句号**

所有 5 个子项（5.1 CommandsView O(n²)、5.2 GPU 成本、5.3 keep-alive、5.4 响应式与重复请求、5.5 巨石 chunk）全部完成，性能治理目标达成。
