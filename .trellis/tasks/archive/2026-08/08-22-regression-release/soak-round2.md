# AC13 泄漏修复第 2 轮

判定：**mixed**。AC13 不勾选。

Playwright 每 tick 重连会污染堆。raw CDP 在**同一 29 条路由复访**时堆仍加速上涨。

## 3 周期 raw CDP（采样前 GC）

基线 `pushState(null)`：`soak-raw-cycles-pushstate-null.jsonl`。打包 exe，旧产物。

| 周期 | 堆起点 | 堆终点 | 堆均值 | 监听器均值 | 节点均值 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 6.4 MB | 28.8 MB | 16.7 MB | 373 | 557 |
| 2 | 30.0 MB | 84.7 MB | 53.8 MB | 386 | 610 |
| 3 | 87.0 MB | 180.0 MB | 129.7 MB | 386 | 611 |

周期 3 / 周期 2 堆均值 = 2.41。`/grok/settings` 监听器 324 / 342 / 342。

打包 exe + RR-safe nav（click 或带 `idx` 的 pushState）：`soak-raw-cycles-packaged-rr.jsonl`。仍为旧 JS。

| 周期 | 堆均值 | 监听器均值 | 节点均值 |
| ---: | ---: | ---: | ---: |
| 1 | 16.9 MB | 372 | 545 |
| 2 | 54.9 MB | 384 | 585 |
| 3 | 133.9 MB | 413 | 924 |

与 `pushState(null)` 同量级。导航 API 不是主因。

## 产品修改后（vite preview 生产包，未打 Tauri）

样本：`soak-raw-cycles-preview-after.jsonl`（RR-safe）、`soak-raw-cycles-preview-replace.jsonl`（replaceState，historyLength=1）。

| 周期 | 堆均值 | 监听器均值 | Query 条数 | 活跃 Query |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 9.1 MB | 348 | 28 | 2.3 |
| 2 | 16.0 MB | 360 | 40 | 2.3 |
| 3 | 26.6 MB | 365 | 40 | 2.3 |

周期 3 / 周期 2 = 1.66，未达到 ≤1.10。replaceState 结果相同。Query 键在周期 2 后稳定在 40，观察者约 2。DOM 与监听器稳定。

堆快照（两轮复访、GC 后）：Object +15.9 MB（+58 万个），Array +4.6 MB，闭包 Context +5.8 MB。

## 已落地的产品改动

- `MainLayout` 不再用 `AnimatePresence` 包 `<Outlet />`。进出场改为 `.route-page` CSS。`MotionConfig reducedMotion="user"` 保留。活跃 Query 约 2，说明路由观察者已卸载。
- `iconRegistry` 改为 `addCollection` 到 `@iconify/react`（原先写进 Vue 包缓存），`main.tsx` 启动时注册。3 周期堆斜率未下降。
- `soak-persist.mjs` 导航改为 click `a[href]`，否则 `pushState` 带 `usr/key/idx`。

## 未关闭的堆增长

复访时仍留下脱离 DOM 的 Object / Array / 闭包。不是 Query 键爆炸，不是 history 栈，不是监听器。打包 exe 上的量级更大（含 IPC 数据）。

## 打包产物复测（`just tauri-build` 之后，click / push-idx，采样前 GC）

样本：`soak-raw-cycles-packaged-after.jsonl`。

| 周期 | 堆起点 | 堆终点 | 堆均值 | 监听器均值 | 节点均值 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 6.3 MB | 29.1 MB | 16.8 MB | 370 | 611 |
| 2 | 30.3 MB | 70.4 MB | 48.6 MB | 385 | 680 |
| 3 | 72.1 MB | 135.5 MB | 101.4 MB | 385 | 680 |

周期 3 / 周期 2 = 2.09。相对修复前打包 129.7 MB 周期 3 均值有下降，仍远高于 1.10。`/grok/settings` 监听器 321 / 341 / 341。

## persist-raw-cdp 2 小时结果

`just tauri-build` EXIT=0 后跑 `ccr-ui/scripts/perf/soak-persist.mjs`。墙钟 7262s。`SOAK_PASS=False`。原始样本：`soak-packaged-round2.jsonl`。

| 项 | 第 1 小时 | 第 2 小时 | 比值 |
| --- | ---: | ---: | ---: |
| 主机 WorkingSet | 78.23 MB | 77.35 MB | 0.989 |
| 渲染进程 WorkingSet | 1910 MB | 2739 MB | 1.434 |
| JS 堆 | 1021 MB | 3521 MB | 3.448 |
| JSEventListeners | 377 | 357 | 0.949 |
| `/grok/settings` 监听器 | 321 / 341 / 341 | — | — |

约 78 min、堆约 4.26 GB 后 CDP `Runtime.evaluate` 超时。第 2 小时 35 个样本中 16 个仍有堆。Query 条数全程 1–14，不是键爆炸。

判定仍为 **mixed**：Playwright 重连不是 3.45× 堆比的主因；产品在每次 SPA 导航后留下约 40 MB 不可回收 JS。监听器项已通过。AC13 不勾选。

第 3 轮泄漏修复需要在产品侧继续查脱离 DOM 的 Object / Array / 闭包（每跳约 40 MB），然后再次 `just tauri-build` 并用同一 persist-raw-cdp 脚本复测。
