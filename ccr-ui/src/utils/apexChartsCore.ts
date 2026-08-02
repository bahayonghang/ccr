// ApexCharts 按需装配入口：只注册本仓库实际用到的图表类型与特性。
//
// 默认的 `vue3-apexcharts` 会拉入完整 apexcharts（全部 17 种图表 + 全部特性），
// 且其 dist 自带一份 SSR 副本，构建产物里会出现两份图表库。
// 这里改用官方模块化入口 `vue3-apexcharts/core` + `apexcharts/<type>` 子路径：
// 各 type 子模块共享同一个 `apexcharts/core`，只把用到的绘制器挂上去。
//
// 当前用到的类型（新增图表类型时必须在此登记，否则运行时该类型不渲染）：
//   area    — UsageOverviewTab / CostAttributionTab / PlatformUsageTrendChart
//   line    — PlatformUsageTrendChart（metric=requests）
//   bar     — UsageCostTab / UsageTokensTab / TokenDetailTab / PlatformUsageTrendChart
//   donut   — UsageModelDistributionCard
//   heatmap — BehaviorAnalysisTab
//
// 特性：只需 legend（图例）；所有图表的 toolbar 均为 `show: false`，
// 亦未使用 annotations / exports / keyboard / morph / drilldown，故一律不注册。
import VueApexCharts from 'vue3-apexcharts/core'
import 'apexcharts/dist/apexcharts.css'

import 'apexcharts/area'
import 'apexcharts/line'
import 'apexcharts/bar'
import 'apexcharts/donut'
import 'apexcharts/heatmap'
import 'apexcharts/features/legend'

export default VueApexCharts
