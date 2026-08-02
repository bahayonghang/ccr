# ApexCharts tooltip marker 大圆：根因与修复边界

## 结论

用户截图中的浅色大圆是 ApexCharts 5.x 默认 tooltip 里的 marker SVG，不是 donut
图表本身、legend、主题色或应用自定义 SVG。已经确认的故障机制是：

```text
ApexCharts 主样式缺失或未生效
-> tooltip series group 不再 display:none
-> marker span 不再拥有 12x12 尺寸
-> 项目全局 svg 响应式 reset 将 marker SVG 撑满卡片内容宽度
-> viewBox 里的 r=5 圆按 10/12 比例放大
-> 首次 donut tooltip 交互写入内联 display:none
-> 大圆自行消失
```

尚未定位到用户现场中 `#apexcharts-css` 缺失或失效的自然触发源。正常 Web、生产
preview 和真实 Tauri WebView2 均未自然复现样式丢失；因此本任务不能声称已经找到完整
触发源。修复应针对已证实的脆弱边界：不能让图表的完整布局契约只依赖一个运行时
`<style id="apexcharts-css">` 节点。

## 证据等级

### 已确认

1. ApexCharts 5.16.0 的 marker 是 `viewBox="0 0 12 12"` 且不带 `width` / `height`
   的内联 SVG；默认圆为 `<circle cx="6" cy="6" r="5">`：
   `ccr-ui/node_modules/apexcharts/src/modules/tooltip/Marker.js:25`。
2. 上游主样式负责把 marker host 限制为 `12x12`，把 marker SVG 设为 `100% x 100%`，
   并在首次交互前隐藏 `.apexcharts-tooltip-series-group`：
   `ccr-ui/node_modules/apexcharts/src/assets/apexcharts.css:344`。
3. 项目 reset 对所有 SVG 设置 `display:block; max-width:100%; height:auto`：
   `ccr-ui/src/styles/base.css:67`。当上游主样式缺失时，marker host 没有固定尺寸，SVG
   会采用可用内容宽度。
4. 真实 WebView2 中移除 `#apexcharts-css` 后，836px 宽的 donut 宿主产生
   `836x836` marker SVG 和约 `696.67x696.67` 的圆，恰为 `836 * 10/12`。
   用户截图与 475px 复现宿主对应约 396px 圆，比例相同。
5. 首次对 donut slice 触发 `mousemove` 后，ApexCharts 将一个 tooltip group 写成
   `display:flex`、其余 group 写成 `display:none`，并把所有 marker host 写成
   `display:none`。这完整解释了“大圆一会儿自行消失”，不需要假设样式后来恢复。
6. ApexCharts 在 `render()` 中先注入主样式，再创建和挂载图表 DOM，正常路径不存在
   “marker 已挂载但样式稍后才注入”的内建 FOUC：
   `ccr-ui/node_modules/apexcharts/src/apexcharts.js:109`。
7. `destroy()`、应用源码和 Vite 的 `data-vite-dev-id` 样式管理都没有删除
   `#apexcharts-css` 的代码路径；Tauri CSP 允许 `'unsafe-inline'` 样式。

### 现场触发源仍未知

- 首页进入 Usage 时，真实 WebView2 创建了一个正常样式节点：文本 24,436 字符、
  134 条 CSS rule、`disabled=false`。
- Usage 标签往返、时间范围切换、Claude Code 往返和多次主题切换期间，该节点保持
  同一对象，未观察到删除、禁用、清空或重复注入。
- 手工删除节点可立即稳定复现；reload 后恢复为一个完整节点和两个图表。
- 依赖/HMR 状态残留是可能性，但没有观测证据，不能升级为根因结论。

### 上游恢复缺口

ApexCharts 5.16.0 的生命周期进一步缩小了边界：

- `render()` 只按固定 ID 查找 `apexcharts-css`。只要同 ID 元素存在，上游不会验证它
  是否为 style、文本是否完整、CSSStyleSheet 是否启用或版本是否匹配。
- `updateOptions()` 与 donut 的 `updateSeries()` 走 `clear -> create -> mount`，不会重新
  进入 `render()`，因此在外部删除/损坏之后不能自愈；只有新的 chart instance 执行
  `render()` 才会补回一个真正缺失的节点。
- `destroy()` 不删除共享主样式。这排除了“正常 destroy 导致丢失”，但也证明现有更新
  路径没有完整性守护。

上游 issue/PR 历史没有找到与本故障链相同的报告。相关 Shadow DOM 放置和重复注入缺陷
均已在 5.16.0 前修复，不能解释现场节点失效。完整源码、版本与 issue 证据见
`upstream-apexcharts.md`。

## 版本时间线

- `bb46226b` 重算 `ccr-ui/bun.lock` 后，`apexcharts` 从 5.6.0 更新到 5.16.0，
  `vue3-apexcharts` 从 1.10.0 更新到 1.11.1；`package.json` 仍使用范围
  `^5.3.6` / `^1.10.0`。
- `1bf0fcff` 将 UI 改为 `vue3-apexcharts/core` + `apexcharts/<type>` 的模块化入口。
- 上游 marker SVG 引入提交：
  <https://github.com/apexcharts/apexcharts.js/commit/4b2ee4ce2cb364ebf48acfb72c5296f1cea3dce8>
- 配套 tooltip CSS 提交：
  <https://github.com/apexcharts/apexcharts.js/commit/6f3a32119125b7a2080f752cf1ef275fee61f906>
- 5.x 最新锁定版本 5.16.0 与后续 6.7.0 都只在 marker SVG 上提供 `viewBox`，
  没有内联 `width` / `height`；尺寸仍属于 CSS 契约。

## 候选方案差分

以下数据来自真实 Tauri WebView2，在同一 Usage 页挂载完成后移除
`#apexcharts-css`。当前 donut 图表内容宽度为 836px。

| 候选方案 | 首次交互前 | 首次交互后 | 判定 |
| --- | --- | --- | --- |
| 无防护 | marker `836x836`；tooltip `836x5852`、`position:static` | marker 被内联隐藏，但 tooltip 仍是 836px 文档流元素 | 稳定复现 |
| 仅排除全局 SVG reset | marker 仍为 `836x836` | 同上 | 失败；SVG 无内联尺寸时仍会占满可用宽度 |
| 仅固定 marker 为 12px | marker `12x12` | tooltip `836x18.71`、`position:static` | 只掩盖大圆，主样式契约仍破坏 |
| marker + group visibility 兜底 | 初始 group 为 0 高 | hover 后 tooltip `836x18.71`、`position:static` | 初始画面可隐藏，但交互语义仍错误 |
| 完整 `apexcharts.css` | group/marker 初始为 0，tooltip 绝对定位且透明 | tooltip 约 `181.78x37.71`、`position:absolute` | 唯一完整方案 |

局部 marker 规则不是完整修复。缺失的同一主样式还负责 tooltip 的绝对定位、透明度、
布局、边框和交互状态；只修一个 SVG 会把更隐蔽的错误留在原处。

## 推荐方案

在唯一的按需装配入口 `ccr-ui/src/utils/apexChartsCore.ts` 静态导入：

```ts
import 'apexcharts/dist/apexcharts.css'
```

同时保留 ApexCharts 默认的运行时注入，不设置 `injectStyleSheet:false`。这样形成两个独立
交付路径：Vite 管理的依赖 CSS 与上游运行时 style 节点任一正常即可。导入放在懒加载的
图表入口，而不是 `main.ts`，不会把图表 CSS 提升到无图表页面的首屏关键样式。

可行性已用不写盘的 Vite 完整应用构建验证，未修改产品源码：transform 仅在构建内临时
向 `apexChartsCore.ts` 模拟上述 CSS import。默认 minify 生成独立的
`apexChartsCore` CSS asset（17,914 字节、gzip 3,660 字节），关闭 minify 的探测约
24.5KB；输出包含 `.apexcharts-tooltip-marker svg` 和
`.apexcharts-tooltip-series-group` 规则。安装包的 `src/assets/apexcharts.css` 与
`dist/apexcharts.css` 文本相同，SHA-256 都为
`54B61EC43EBE92812ACCFDBC2E5E32D3F2CE294AD905BC163A59AF91D42D2A6C`。

同一完整构建的 `index.html` 没有直接引用该 CSS。Usage、Claude Observer/Insights 和
Platform Usage 三个异步调用方都把 lazy module JS、`charts-vendor`、`vue-vendor` 和
该 CSS asset 同时列入 `__vite__mapDeps`。生成的 `__vitePreload` 会创建
`rel="stylesheet"` link，为 CSS 返回 `load` / `error` Promise，并只在该 Promise
settle 且无未处理错误后调用 `baseModule()` 执行动态 import。这证明所选 import 在当前
Vite 7.3.6 配置下不会提升为入口直链 CSS，也不会形成“图表模块先执行、关联 CSS 后到达”
的 production 时序窗口。这是实施前的 bundler 行为证明；产品源码加入 import 后仍须以
真实 production build 复核最终 chunk 图。

不采用以下方案：

- 不修改全局 SVG reset：实测不能单独修复，且扩大所有 ApexCharts SVG 的回归面。
- 不添加 marker/group 私有 CSS 副本：会复制上游内部契约且仍无法恢复完整 tooltip。
- 不降级或钉死旧 ApexCharts：会回退依赖升级并与 `vue3-apexcharts >=5.10.0` 的 peer
  约束冲突。
- 不把主题检测或 legend 注入纳入本修复：二者不能产生当前几何，已被源码和实测排除。

## 可复现探针

真实 WebView2 已启用 CDP 时，在仓库根目录运行：

```powershell
node .trellis/tasks/08-01-fix-apexcharts-tooltip-marker-svg-reset/research/apexcharts-tooltip-marker-probe.mjs
node .trellis/tasks/08-01-fix-apexcharts-tooltip-marker-svg-reset/research/apexcharts-tooltip-marker-probe.mjs --block-runtime-css
```

当前未修复基线预期：第一条 `[PASS]`、第二条稳定检测大圆并 `[FAIL]`。实现静态导入后，
第二条也必须 `[PASS]`，证明即使首次挂载前阻止运行时 style 节点，完整 CSS 契约仍在。
探针在 fault 模式结束前会清除 session flag 并 reload，避免把 WebView 留在故障态。

2026-08-02 收敛复核依次得到：normal exit 0；fault exit 1，2 次 runtime style append
被阻止、7/7 marker 为 `836x836`、tooltip 为 `836x5852` 且 `position:static`；清理后
再次 normal exit 0。最后一步证明探针没有把 WebView 留在故障态。

最佳视觉证据为 `../no-apex-css-repro.png`，其大圆位置、裁剪方式和尺寸比例与用户截图一致。

## 独立问题

生产 Chrome 中曾出现 Usage 离开再返回后 Vue 图表宿主仍在但
`.apexcharts-canvas` 数量为 0；真实 WebView2 在大量主题切换后也曾出现空宿主。该现象
在 `#apexcharts-css` 完整存在时发生，属于 KeepAlive / 图表实例生命周期缺陷，应另立
任务，不能混入本任务根因或验收。

上游研究还发现既有 `usage-chart-stability-contracts.md` 把所有 `updateSeries()` 都描述为
canvas 全量重建；5.16.0 已允许符合条件的 axis chart 使用 fast update，但 donut 仍明确
走 full update。本任务只依赖 donut 结论，不顺带修改该独立 spec drift。

当前未改产品代码的 `bun run build:with-budget` 基线在 production build 成功后仍因入口
JS `237.64 KiB > 110 KiB` 失败。该既有 budget 红线与本任务的异步 CSS 无关，不能在实施
后误归因为新增 CSS；修复预算基线应使用独立范围。
