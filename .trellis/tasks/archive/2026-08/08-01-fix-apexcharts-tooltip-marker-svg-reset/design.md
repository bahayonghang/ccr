# Design: ApexCharts 主样式双路径交付

## 1. Boundary

改动限定在 ApexCharts 的唯一模块化装配入口及其 contract test：

- `ccr-ui/src/utils/apexChartsCore.ts`
- `ccr-ui/tests/apexcharts-style-contract.smoke.test.ts`

不修改 Usage 组件、图表 options、全局 reset、Vite 配置、依赖版本或 Tauri 代码。

## 2. Selected Design

在 `apexChartsCore.ts` 中静态导入上游发布的完整 CSS：

```ts
import 'apexcharts/dist/apexcharts.css'
```

保留 ApexCharts 默认 `chart.injectStyleSheet: true`。运行时渲染仍可创建
`#apexcharts-css`，Vite 管理的 CSS 则作为独立路径存在。

```text
异步加载 apexChartsCore
  -> Vite 先加载该 chunk 的 apexcharts.css
  -> 模块注册 area/line/bar/donut/heatmap/legend
  -> ApexCharts render() 尝试注入 #apexcharts-css
  -> 任一 CSS 路径正常即可满足完整 tooltip/layout 契约
```

导入与图表模块共置，因此仍遵守现有懒加载和单一装配入口约束。

## 3. Why Full CSS

缺失的不只是 marker 尺寸规则。同一上游样式还拥有：

- tooltip 的 `position:absolute`、opacity、display 和主题表面；
- tooltip series group 的初始隐藏与激活布局；
- marker host 的 12px 尺寸及 SVG 内部缩放；
- 其他 ApexCharts DOM 的交互和辅助布局。

真实 WebView2 差分证明，局部 marker/group fallback 在 hover 后仍留下 836px 宽的静态
tooltip。复制更多 selector 会逐步形成一份不完整且随上游漂移的 CSS fork，边界错误。

## 4. CSS Cascade And Loading

- 项目 SVG reset 位于 `@layer base`；正常的未分层 ApexCharts CSS 优先于该 layer。
- 故障并非 selector 特异性冲突，而是完整上游规则源不存在或不生效。
- `apexcharts` 5.16.0 的 package exports 公开 `./dist/*`，`sideEffects` 保留 `*.css`，
  因此 `apexcharts/dist/apexcharts.css` 是受支持且不会被 tree-shake 的构建输入。
- 不写盘的 Vite 完整应用构建通过 transform 临时模拟所选 import，生成独立的
  `apexChartsCore` CSS asset：17,914 字节、gzip 3,660 字节；关闭 minify 的探测约
  24.5KB。输出包含关键 marker/group 规则，且 `index.html` 不直接引用它。
- Usage、Claude Observer/Insights 和 Platform Usage 三个异步调用方的
  `__vite__mapDeps` 都同时列出图表 JS、共享 vendor 和该 CSS asset；
  `__vitePreload` 为 CSS 创建 `rel="stylesheet"`，等待其 `load` Promise 后才执行动态
  module import。因此该方案保留按需加载，同时避免图表模块在关联 CSS 就绪前执行。
- 该 transform 探测只验证实施前 bundler 行为，没有修改产品源码；实施后的真实
  production build 仍是验收门。
- 保留运行时注入会让相同规则在正常态出现两次，但两份内容相同，cascade 结果一致；
  增量成本约 24.5KB 未压缩、17.9KB minified、3.6KB gzip CSS。关闭运行时注入不会从
  ApexCharts JS 中移除内置 CSS 字符串，反而失去第二条容错路径，因此不采用。

## 5. Alternatives

| Alternative | Decision | Reason |
| --- | --- | --- |
| 排除 `.apexcharts-canvas` 内的全局 SVG reset | Reject | 实测 marker 仍扩展为 836px，且影响所有图表 SVG。 |
| 只设置 marker `width/height` | Reject | 大圆缩小，但 tooltip 仍静态占位且宽 836px。 |
| marker + group visibility fallback | Reject | 初始隐藏有效，hover 后完整 tooltip 定位/主题契约仍错误。 |
| 只静态导入并关闭运行时注入 | Reject | 没有 bundle 节省，减少容错路径。 |
| 降级 ApexCharts | Reject | 回退依赖治理，且当前 wrapper peer 需要 ApexCharts >=5.10.0。 |
| patch marker SVG 内联尺寸 | Reject | 修改上游产物，只修单个症状。 |

## 6. Verification Design

### Static contract

新增 smoke test：

1. 断言 `apexChartsCore.ts` 显式导入 `apexcharts/dist/apexcharts.css`；
2. 读取已安装的发布 CSS，断言存在 tooltip overlay、series group 初始隐藏、marker host
   12px 和 marker SVG 100% 规则；
3. 现有模块化 chart type/legend imports 不得丢失。

该测试把依赖升级时的 CSS 契约漂移变成显式 review gate；production build 负责证明 Vite
实际打包 CSS，而不是只满足源码检查。

### Runtime contract

`research/apexcharts-tooltip-marker-probe.mjs` 连接真实 WebView2 CDP：

- normal：运行时 style 完整，预期绿色；
- `--block-runtime-css`：首次 mount 前拦截 `#apexcharts-css` append。未修复基线为红色，
  实现后必须由 Vite CSS 路径转绿；
- 两种模式均检查 marker 数量/几何、host 尺寸、group 初始 display、tooltip position 和
  关键 rule source；fault 模式在退出前恢复正常页面。

### Visual scope

检查 Usage donut 的首次加载和 hover，并抽查至少一个 axis chart 路由。视觉证据只用于
确认布局与交互未回归，不能替代 probe 的 DOM/CSS 数值断言。

## 7. Rollout And Rollback

这是本地静态资源加载调整，无数据迁移、配置迁移或后端 rollout。若 production build、
图表路由或 bundle 行为异常，回滚单个 CSS import 和对应 contract test 即可；上游运行时
注入仍维持变更前正常路径。

## 8. Deferred Risk

用户现场使 runtime style 缺失/失效的自然触发源仍未知。双路径方案消除已确认的单点
脆弱性，并绕开上游只按 ID 信任且 update 不自愈的恢复缺口，但不等同于找到了删除者。
若修复后仍有报告，应基于探针的 ruleSources 和 runtimeStyle 字段继续采集，而不是回到
marker 身份猜测。

仓库当前未改产品代码的 `bun run build:with-budget` 基线因入口 JS
`237.64 KiB > 110 KiB` 而失败；普通 production build 成功。该既有红线与本任务的异步
CSS 无关，也不应在实施后被误归因。本任务仍检查实际 CSS/chunk 归属；修复 bundle budget
基线需要独立范围。
