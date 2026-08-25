# 执行计划：用量与成本图表区

前置：`08-25-design-token-consolidation` 与 `08-25-home-runtime-layout` 已合入。

开工前读 `08-25-design-token-consolidation/research/token-name-delta.md`，
取数据字号档位的实际令牌名（`design.md` §2 说明为何不在设计里写死）。

## 检查清单

- [x] 1. 测试落点：`rg -l 'DashboardUsageMovement' ccr-ui/tests`，确定扩展既有文件还是新建，写进 change list。
- [x] 2. `homeDateWindow(days)` 纯函数（`design.md` §4.1），本地日期格式化，不用 `toISOString()`。
- [x] 3. 堆叠柱派生纯函数（`design.md` §3），含 `maxDailyTotal === 0` 的空态分支。
- [x] 4. 两个纯函数的单元测试先写（`design.md` §8 前两行）。
- [x] 5. `DashboardCostMetric.tsx` 新建：唯一调用 `useUsageSummary(undefined, startDate, endDate)` 的组件，实现三态。
- [x] 6. `DashboardUsageMovement.tsx` 条件挂载成本子组件（`design.md` §4.2），沿用既有 `scheduleWhenIdle`。
- [x] 7. 成本三态与延迟挂载的测试（`design.md` §8 后两行）。
- [x] 8. 指标行：四项 + 平台图例，字号档位按第 0 步取到的令牌名。
- [x] 9. 图表：嵌套 flex 堆叠柱 + 两条虚线网格 + 底边框 + 日期轴。
- [x] 10. 分段控件 7D/30D/90D 接既有 `onChangeDays`，`role="radiogroup"` + 键盘可达。
- [x] 11. loading / error / 空态 / 非 Tauri 四种分支各自可见，无空白卡。
- [x] 12. 可访问性：容器 `role="img"` + `aria-label`；每根柱 `title`。
- [x] 13. `prefers-reduced-motion` 分支关闭入场动画。
- [x] 14. 新增文案的中英文键补齐。
- [ ] 15. `just frontend-check-quick`。（type-check / lint / smoke 通过；唯一失败是 i18n leaf-count：期望 4166，zh/en 实计 4178。本任务新增 6 键，其余为并行子任务；不改 `check-i18n.mjs` / `i18n.test.cjs`。）
- [x] 16. 视觉与交互验证（见下；Web 空态无法看到堆叠柱，AC1/AC9 的分层色留待桌面有数据）。

## 验证命令

```bash
just frontend-check-quick
```

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/dashboard-usage-movement.smoke.test.tsx
```

用于 AC4 / AC5 / AC6 / AC12：区间构造、延迟挂载与成本三态由测试断言，不靠肉眼。
文件名以第 1 步的落点结论为准。

```bash
rg -n '#[0-9a-fA-F]{3,8}|border-radius:\s*[0-9]+px' ccr-ui/src/features/usage/styles/dashboard-usage-movement.css
```

用于 AC11：应无命中。

```bash
rg -n 'invoke\(' ccr-ui/src/features/usage/dashboard/DashboardUsageMovement.tsx ccr-ui/src/features/usage/dashboard/DashboardCostMetric.tsx
```

用于 AC3：应无命中（数据只经既有 hook）。

```bash
git diff --name-only -- ccr-ui/src/features/usage/queries.ts ccr-ui/src/api/
```

用于 AC3：应为空（不改共享 hook 与 API facade）。

```bash
rg -n 'useUsageSummary' ccr-ui/src/features/usage/dashboard/
```

用于 AC5：唯一命中应在 `DashboardCostMetric.tsx`。

## 视觉与交互验证

`cd ccr-ui && npm run dev`。

- [ ] 柱按天渲染、按平台分层；层色与图例、平台卡色条一致（AC1）。Web 预览无 IPC / `maxDailyTotal === 0`，未见堆叠柱。
- [x] 全页只有一个 hero 档数字（AC2）。computed：请求 `--text-2xl` = 26px；TOKEN/成本/会话 `--text-xl` = 21px；同页 26px 节点仅此 hero。
- [x] Web 预览下成本显示 `—`（AC6）。`data-cost-state=unavailable`。
- [x] 切换 7D/30D/90D，图表与指标行（含成本）同步刷新，柱数随之变化（AC7）。radiogroup + 点击切换已验；Web 空态柱数保持 0，成本保持 `—`。
- [ ] 灰度模拟（浏览器渲染设置或 `filter: grayscale(1)`）下仍能判断各层归属（AC9）。空态无层；非颜色手段已落地（图例文字、柱 `title`、`aria-label`）。
- [x] `prefers-reduced-motion: reduce` 下无柱状入场动画（AC10）。根 `data-reduced-motion=true` 时探测柱 `animation-name: none`。
- [x] 四组主题×flavor 组合各扫一遍。DOM 切 `light|dark` × `neutral|clay`，卡片可读；未进设置页。

## 回滚

```bash
git checkout -- ccr-ui/src/features/usage/dashboard/DashboardUsageMovement.tsx ccr-ui/src/features/usage/styles/dashboard-usage-movement.css ccr-ui/tests/
rm -f ccr-ui/src/features/usage/dashboard/DashboardCostMetric.tsx
```

新增文件需显式删除。

## 提交

`feat(ui): ✨ 首页用量区改为堆叠日柱与成本指标`

change list 必须包含测试文件（父任务 XC5）。
提交前执行父任务 XC4 的三条检查，确认 `ccr-ui/src-tauri/Cargo.toml` 不在暂存区、不在提交中。
</content>
