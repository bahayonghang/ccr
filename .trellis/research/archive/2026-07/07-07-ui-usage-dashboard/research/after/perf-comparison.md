# Usage Dashboard 性能前后对比(implement.md 第 9 项)

- **日期**: 2026-07-08
- **基线**: `../baseline/baseline-perf.json`(改造前,commit 55c3775a 之前录制)
- **改造后**: `after-perf.json`(含第 9 项复测中追加的 3 处修复,见下文)
- **口径**: 两轮同用 `../perf-harness/tauri-shim.js` + Playwright + vite dev 15173,
  stale 变体、light/clay/clay、1920×1080、10ms 轮询、data-perf-id 节点标记法。
  差异之处已写入 after-perf.json 的 `method` 字段(KeepAlive 后再进入不再产生新 canvas,
  计时终点放宽为"活动 tab 出现 canvas";窗口切换补充 30s 快照缓存路径的结算规则)。

## 结论:U1 / U2 均已消除

| 指标 | 基线 | 改造后 |
| --- | --- | --- |
| tab 再进入重建图表(U1) | **12/12 全部重建** | **0/12 重建**(KeepAlive 命中) |
| tab 再进入耗时(3 轮 cycle) | 25~62ms(重建) | **10~14ms**(缓存回填) |
| warmup 首次进入 tokens/cost | 80 / 45ms | 55 / 28ms(冷挂载,合理重建) |
| warmup 返回 overview | 45ms(重建) | 11ms(rebuilt=false) |
| 窗口切换图表重挂载(U2) | **2/2 重挂**(61~64ms,发生在 refetch 前) | **0/2 重挂**,2/2 canvas 存活 |
| 窗口切换耗时 | 61 / 64ms(重挂事件) | 本月 319ms(300ms 防抖+refetch 落定);近 30 天命中 store 30s 快照缓存,零 IPC |
| 20 次快速往返内存 | 38.0→22.4MB 无泄漏 | 16.3→16.3MB 无泄漏 |
| console error | 0 | 0 |

> 窗口切换耗时口径说明:基线的 61~64ms 计的是"重挂事件出现"(即 U2 症状本身),
> 改造后该事件不复存在,319ms 计的是"点击→数据 refetch 触达"(含 300ms 防抖),
> 图表全程不拆挂、由 series 引用稳定性保证仅在真实数据变化时走 updateSeries 快路径。

## 第 9 项复测揪出的 3 处残留根因(均已修复)

首轮复测(仅有第 1~7 项改动时)并不达标:toTokens/toCost 六次再进入全部重建(且耗时
168~198ms 比基线更差),窗口切换 37ms 内重挂。用 `../perf-harness/diagnose-after.mjs`
定位到根因后修复:

1. **UsageTokensTab / UsageCostTab 本地 chartOptions 缺 `redrawOnParentResize:false /
   redrawOnWindowResize:false`**(overview 的 TREND/PIE 冻结基座有,两个本地 options 没抄全)。
   ApexCharts 默认 parentResize 触发全量 update(销毁重建 canvas),KeepAlive 重挂正好触发
   parent resize → 每次再进入都重建,KeepAlive 收益被整体抵消。KeepAlive 本身工作正常
   (probe 证实 tokens 根 DOM 跨往返存活,重建只发生在 apexcharts 层)。
2. **`dashboardPresentation` 依赖 `selectedWindowLabel`(纯文案)**,窗口一点 label 同步变 →
   presentation 整体重算 → `trendSeries/pieSeries` 产出**值相同、引用全新**的数组 →
   vue3-apexcharts 对 series 是 deep watch,引用一变就 `updateSeries` → ApexCharts 内部
   走全量 update 重建 canvas(37ms,发生在 300ms 防抖 refetch 之前,数据根本没变)。
   修复:`useUsageCharts.ts` 对 `trendSeries/pieSeries/modelTokenPieSeries` 做按值记忆化
   (`computed(previous)` + join key,与既有 labels 记忆化同一思路),值不变复用旧引用,
   真实数据变化仍走 updateSeries 快路径。
3. **测量脚本层**:store 有 `DASHBOARD_CACHE_TTL_MS = 30s` 的 dashboard 快照缓存,30s 内
   切回同窗口不发 IPC(合法行为),harness 结算规则补充了该路径(`servedFromCache: true`)。

## 已知残留(记录,不在本任务修)

- `UsageTokensTab/UsageCostTab` 本地 chartOptions 仍硬编码 `animations:{enabled:false}`
  (7b 只升级了 TREND/PIE 基座的 reduced-motion 接线);且 cost tab 的 options 直接依赖
  `ctx.trends`(数据刷新即换 options 引用 → 对已缓存的离屏图表触发 updateOptions 重建)。
  离屏重建用户不可见、真实数据刷新本就需要更新,影响限于回到该 tab 时首帧已是新 canvas,
  不影响 U1/U2 验收指标;后续如做图表 options 统一收编到工厂时一并处理。

## 产物清单

- `after-perf.json` — 改造后完整数据(方法、warmup、3 轮 cycle、窗口切换、内存、console)
- `after-tokens-reentry-1920.png` — tokens tab KeepAlive 命中态截图
- `after-window-last30-1920.png` — 窗口切换(近 30 天)落定后截图
- `after-overview-final-1920.png` — 20 次往返后 overview 终态截图
- `../perf-harness/measure-after.mjs` — 复测脚本(可重跑,dev server 15173 需在线)
- `../perf-harness/diagnose-after.mjs` — 根因定位 probe(节点身份 + 宿主链盘点)
