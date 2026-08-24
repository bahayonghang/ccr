# 2 小时浸泡测量（AC13）

> 任务：`08-22-regression-release` R10 / AC13。打包产物 `ccr-desktop.exe`。主线程命令：`pwsh -File soak-run.ps1`。墙钟 7203.41s。退出码 1（`SOAK_PASS=False`）。

原始 JSONL：scratch `soak-packaged.jsonl`（本目录副本 `soak-packaged-summary.json`）。

## 方法

- 进程：PID 75572，`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222`
- 每 60s 采一次 `ccr-desktop` WorkingSet；另起 bun 进程经 CDP 做 SPA `pushState` 切路由并读 `Performance.getMetrics`（`JSHeapUsedSize` / `JSEventListeners`）。单次 tick 超时 25s，超时杀进程，浸泡继续。
- 判定（design.md §5）：第 2 小时均值 ≤ 第 1 小时均值的 110%。监听器沿用同一比例。路由 ≥20。

## 汇总

| 项 | 第 1 小时 | 第 2 小时 | 比值 | 判定 |
| --- | ---: | ---: | ---: | --- |
| 样本数 | 59 | 46 | — | 两小时均有样本 |
| 其中含 CDP 堆/监听器 | 59 | 11 | — | 第 2 小时 3602–4215s 之后 35 次 tick 超时 |
| WorkingSet 均值 | 85.35 MB | 80.90 MB | 0.948 | ≤1.10 |
| JS 堆均值 | 1111.9 MB | 3699.2 MB | 3.327 | >1.10 |
| JSEventListeners 均值 | 1063.8 | 4959.0 | 4.662 | >1.10 |
| 唯一路由 | 29 | 29 | — | ≥20 |

第 2 小时 CDP 在 `/antigravity/mcp`（elapsed 4214s）之后连续超时。监听器第 2 小时均值被 `/grok/settings` 一次 48066 拉高；去掉该点后其余 10 个 CDP 样本约 390–1495。

JS 堆在第 2 小时仍有 CDP 的 11 个样本上从 3201 MB 升到 4203 MB。

## 结论

浸泡已执行满 2 小时，切换 29 条路由。主机 WorkingSet 第 2 小时低于第 1 小时。JS 堆与监听器不满足 110% 判定。AC13 不勾选。
