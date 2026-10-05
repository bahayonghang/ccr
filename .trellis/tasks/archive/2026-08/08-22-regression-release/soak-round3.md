# AC13 泄漏修复第 3 轮

判定：**product**（预览 3 周期过关）。AC13 不勾选。打包 exe 需 `just tauri-build` 后再跑 2 小时 raw soak。

## `/codex/mcp` 拐点

2 小时 persist-raw-cdp 在 `/codex/mcp` 从 9.3 MB 跳到 43.3 MB，之后每 tick 约 +40 MB。短 settle（2s）直达 `/codex/mcp` 只有 9.7 MB，**不是 MCP 页面本身加载了 40 MB 模块**。

60s 驻留实验（`soak-mcp-dwell.jsonl`）：

| 步骤 | 堆 |
| --- | ---: |
| `/codex` 2s | 6.8 MB |
| `/codex/settings` 2s | 7.9 MB |
| 同一页驻留 60s | **40.9 MB** |
| `/codex/mcp` 2s | 42.0 MB |
| 驻留 60s | **72.0 MB** |

监听器与 DOM 不变。跳变发生在 **60s 驻留**，采样点碰巧落在下一路由（2 小时脚本先采样再等 60s）。

跳过 `/codex` 首页、只开 settings 再驻留 60s（`soak-converter-dwell.jsonl`）堆保持 ~6.5 MB。泄漏在访问 **Codex 仪表盘之后** 才启动，并在卸载后继续分配。

机制：

1. `CodexView` / `GrokView` 在 `useEffect(..., [refresh])` 里调用 `refresh(false)`，而 `refresh` 依赖整个 `useQuery` 返回对象，身份每轮渲染都变，会在 `isStale` 期间重复 `refetch`。
2. `SIcon` 使用 `@iconify/react` API 版。未命中本地集合时 `loadIcons` 回调因陈旧 abort 闭包不能在卸载时取消，图标 JSON 留在堆上。Codex 首页图标更多，API 响应在随后 60s 到达。

## 3 周期（vite preview 生产包，本轮修改之后）

样本：`soak-raw-cycles-round3-preview.jsonl`。raw CDP，GC，1.2s settle，RR-safe nav。

| 周期 | 堆均值 | 监听器均值 | Query 条数 | 活跃 Query | `/codex/settings` → `/codex/mcp` |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 8.16 MB | 348 | 28 | 2.3 | 7.23 → 7.30 MB |
| 2 | 10.20 MB | 366 | 40 | 2.3 | 9.96 → 9.99 MB |
| 3 | 11.12 MB | 366 | 40 | 2.3 | 10.80 → 10.82 MB |

周期 3 / 周期 2 = **1.09**（≤ 1.10）。上一轮 preview 为 1.66。MCP 拐点约 +0.07 MB。

打包 3 周期（`just tauri-build` 之后，1.2s settle，`soak-raw-cycles-packaged-round3.jsonl`）：堆均值 9.4 → 12.2 → 13.5 MB，周期 3 / 周期 2 = 1.107。`/codex/settings` → `/codex/mcp` 为 8.47 → 8.52 MB。

## persist-raw-cdp 2 小时（打包产物，第 3 轮修复后）

墙钟 7206s。PID 58536。样本 117，第 1 小时 59，第 2 小时 58，CDP 全程有效。`SOAK_PASS=False`。原始样本：`soak-packaged-round3.jsonl`。

| 项 | 第 1 小时 | 第 2 小时 | 比值 |
| --- | ---: | ---: | ---: |
| 主机 WorkingSet | 88.26 MB | 91.51 MB | 1.037 |
| 渲染进程 WorkingSet | 555 MB | 558 MB | 1.006 |
| JS 堆 | 10.73 MB | 13.89 MB | 1.295 |
| JSEventListeners | 376 | 403 | 1.073 |
| `/grok/settings` 监听器 | 321 / 341 / 341 / 380 | — | — |
| 唯一路由 | 29 | 29 | — |

主机 WorkingSet、渲染进程 WorkingSet、监听器 ≤1.10。JS 堆 1.295 按 `ac13-residual.md` 作为残余接受。末次堆 14.9 MB。泄漏修复轮次 3/3。

## 代码

- 去掉 Codex/Grok 首页挂载时随 `refresh` 身份变化的重复 refetch；`refresh` 只依赖稳定的 `refetch`。
- `SIcon` 与 `iconRegistry` 改用 `@iconify/react/offline`。
- Codex 概览 / 用量 IPC 只拷贝仪表盘用到的字段。
- `CodexMcpView` 直接引用 `BaseMcp`，不经过 platform 桶文件。
- `__CCR_SOAK_STATS` 增加 `queryDataChars`（长度，不含 payload）。
