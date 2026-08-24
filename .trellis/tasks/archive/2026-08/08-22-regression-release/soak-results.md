# 2 小时浸泡测量（AC13）

打包产物 `ccr-desktop.exe`。方法：每 60s SPA 切 29 条路由；采主机 WorkingSet；另起 bun tick 读 CDP `JSHeapUsedSize` / `JSEventListeners`（超时 25s）。

判定：第 2 小时均值 ≤ 第 1 小时均值 × 1.10；唯一路由 ≥ 20。

## 修复前（`914565c3` 之前，PID 75572，7203s）

| 项 | 第 1 小时 | 第 2 小时 | 比值 |
| --- | ---: | ---: | ---: |
| WorkingSet 均值 | 85.35 MB | 80.90 MB | 0.948 |
| JS 堆均值 | 1112 MB | 3699 MB | 3.327 |
| JSEventListeners 均值 | 1064 | 4959 | 4.662 |
| `/grok/settings` 监听器 | 4001 → 27306 → 48066 | — | — |
| 唯一路由 | 29 | 29 | — |
| 第 2 小时 CDP 有效样本 | — | 11 / 46 | 其余 tick 超时 |

`SOAK_PASS=False`。

## 修复后（`914565c3`：图表 destroy、resize cancel、Query `gcTime` 120s、Grok/Gemini copy timer）

命令：`just tauri-build` EXIT=0（`just-tauri-build-leakfix.log`），再 `pwsh -File soak-run.ps1`。墙钟 7203.24s。PID 600。`SOAK_PASS=False`。

| 项 | 第 1 小时 | 第 2 小时 | 比值 |
| --- | ---: | ---: | ---: |
| 样本 | 59 | 46 | — |
| 其中含 CDP | 59 | 11 | 第 2 小时后段 35 次 tick 超时 |
| WorkingSet 均值 | 79.59 MB | 73.14 MB | 0.919 |
| JS 堆均值 | 1095 MB | 3648 MB | 3.332 |
| JSEventListeners 均值 | 3404 | 3865 | 1.135 |
| `/grok/settings` 监听器 | 1891 → 167072 → 36016 | 末次 tick 超时 | — |
| 唯一路由 | 29 | 29 | — |

主机 WorkingSet 仍 ≤1.10。JS 堆比值仍约 3.33。监听器均值 1.135，略超 1.10；`/grok/settings` 单次仍出现五位数尖峰。第 2 小时 CDP 仍在 ~70 min `/antigravity/mcp` 后连续超时。

新鲜进程上仅循环 `/settings` ↔ `/grok/settings` 三次：监听器约 324，堆约 8 MB。尖峰只在长循环、堆已很大时出现。

## 结论

AC13 不勾选。修复降低了监听器均值比值（4.66 → 1.14），未改变 JS 堆小时比。

原始样本：`soak-packaged.jsonl`（修复前）与 `soak-packaged-postfix.jsonl`（修复后）。
