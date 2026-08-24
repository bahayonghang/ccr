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

## 第 2 轮（测量污染 + 复访堆增长）

postfix 每 60s 新建 Playwright `connectOverCDP`。对照 12 tick（`soak-control-12tick.jsonl`，采样前 GC）：

| 方法 | 每 tick 堆增量 | `/grok/settings` 监听器 |
| --- | ---: | ---: |
| raw CDP | 0.74 MB | 324 |
| Playwright 持久连接 | 2.34 MB | 332 |
| Playwright 每 tick 重连 | 5.00 MB | 332 |

3 周期 raw CDP 复访同一 29 条路由时堆仍加速上涨（周期均值 16.7 → 53.8 → 129.7 MB）。RR-safe nav 不改变该斜率。已去掉全页 `AnimatePresence` 并把 Iconify 注册进 `@iconify/react`。vite preview 生产包上周期 3 / 周期 2 堆均值仍为 1.66。

`just tauri-build` 之后打包 3 周期（`soak-raw-cycles-packaged-after.jsonl`）堆均值 16.8 → 48.6 → 101.4 MB，周期 3 / 周期 2 = 2.09。监听器稳定。

## persist-raw-cdp 2 小时（打包产物，第 2 轮修复后）

脚本 `ccr-ui/scripts/perf/soak-persist.mjs`。墙钟 7262s。PID 33252。`SOAK_PASS=False`。样本 `soak-packaged-round2.jsonl`。

| 项 | 第 1 小时 | 第 2 小时 | 比值 |
| --- | ---: | ---: | ---: |
| 样本 | 58 | 35 | — |
| 其中含 CDP 堆 | 58 | 16 | 约 78 min 后 Runtime.evaluate 超时 |
| 主机 WorkingSet 均值 | 78.23 MB | 77.35 MB | 0.989 |
| 渲染进程 WorkingSet 均值 | 1910 MB | 2739 MB | 1.434 |
| JS 堆均值 | 1021 MB | 3521 MB | 3.448 |
| JSEventListeners 均值 | 377 | 357 | 0.949 |
| `/grok/settings` 监听器 | 321 → 341 → 341 | — | — |
| 唯一路由 | 29 | 29 | — |

主机 WorkingSet 与监听器 ≤1.10。JS 堆与渲染进程 WorkingSet 超过 1.10。`/grok/settings` 五位数尖峰消失。

AC13 仍不勾选。详见 `soak-round2.md`。

## 第 3 轮

`/codex/mcp` 的 40 MB 跳变是上一页 60s 驻留后的采样点。访问 Codex 仪表盘后，Iconify API 回调与 `refresh` 身份循环在驻留期间往堆上堆对象。

vite preview 3 周期（本轮修改后）：堆均值 8.16 → 10.20 → 11.12 MB，比值 1.09。`/codex/mcp` 相对 settings 约 +0.07 MB。详见 `soak-round3.md`。

`just tauri-build` 之后 persist-raw-cdp 2 小时（墙钟 7206s，`soak-packaged-round3.jsonl`）：样本 117，CDP 全程有效。主机 WorkingSet 1.037、渲染进程 WorkingSet 1.006、监听器 1.073 通过。JS 堆均值 10.73 → 13.89 MB，比值 1.295，按 `ac13-residual.md` 作为残余接受。`/grok/settings` 监听器 321 / 341 / 341 / 380。末次堆 14.9 MB。详见 `soak-round3.md`。
