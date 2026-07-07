# 执行计划:使用统计页性能与排版优化

## Checklist

1. [ ] 基线性能录制:tab 切换 ×3、窗口切换 ×2 的 Performance trace + 耗时记录,存 research/。
2. [ ] composable 拆分(纯搬移,签名不变)。
   - 验证:`bun run type-check`;页面行为无差异;主文件 ≤400 行。
3. [ ] chartOptions.ts 工厂 + options 记忆化;series 独立 computed。
   - 验证:窗口切换时图表不闪烁重挂载(录屏对比);theme/locale 切换正常重建。
4. [ ] KeepAlive + 动态组件 tab;provide/inject usage 上下文。
   - 验证:二次进入 tab 无重建(对比第 1 步基线,记录数据);内存无异常增长(切 20 次)。
5. [ ] Sparkline 三合一,删两份旧实现。
   - 验证:`rg -l "usage/SparkLine|UsageSparkline"` 零引用;涉及卡片截图正常。
6. [ ] 第一屏重排(design.md §5):指标卡上移首行、cockpit 拆为 StaleBanner + 诊断抽屉、L/M/D 人话化、degraded 解释与动作、空告警隐藏、meta popover、去 ambient。
   - 验证:1920/1280/900px 三档截图;stale 与健康两种状态截图;i18n 齐全。
6b. [ ] formatTokens/formatCost 格式化升级(≥1B 用 B、千分位)。
   - 验证:12527.4M → 12.53B;$26,114.04;既有单测/快照更新。
7. [ ] logs 骨架行 + sticky 表头;图表动画接 prefers-reduced-motion。
   - 验证:reduced-motion 模拟下无入场动画。
8. [ ] `bun run type-check && bun run lint` + `just frontend-check-quick`。
9. [ ] 前后性能数据对比写入 research/,截图入 research/;review gate。

## Rollback

按 design.md §7 commit 划分独立 revert;性能不达标时保留 ①② 仅回滚 ③(KeepAlive 内存不可控时)。
