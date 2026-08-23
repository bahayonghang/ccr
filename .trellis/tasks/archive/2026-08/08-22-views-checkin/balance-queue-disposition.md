# 余额查询队列状态归属

判据（design.md §7）：队列状态跨组件共享则进 Zustand；只在一个视图内则用 `useRef` 持有队列 + `useState` 持有可见状态。

结论：队列实现为纯函数 `runPerKeySequential` / `shouldSkipBalanceRefresh`（`src/features/checkin/lib/balanceRefreshQueue.ts`），无跨组件共享状态。调用点仅 `useCheckinState.refreshAllBalances`。可见态 `balanceRefreshing` 放在 CheckIn 运行时 box（组件本地）。并发上限 5、节流 30s 不变。
