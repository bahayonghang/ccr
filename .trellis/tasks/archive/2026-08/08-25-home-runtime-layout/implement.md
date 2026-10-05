# 执行计划：首页 1b 栅格与平台卡

前置：`08-25-design-token-consolidation` 已合入。
开工前确认 `--color-platform-opencode` 与四档圆角取值已可解析，否则先等前置。

## 阶段 A：核查（结论决定后续范围）

- [x] A1. chrome 层核查（`design.md` §3）：

      ```bash
      rg -n 'sidebar-glass|topbar-glass' ccr-ui/src/shell/ ccr-ui/src/styles/
      rg -n 'surface-shell|material-glass-chrome' ccr-ui/src/shell/ ccr-ui/src/styles/
      ```

      得出结论 A（零改动）或结论 B（接回语义别名），写入 `design.md` §3。
      两种结论都不改 `--material-glass-chrome-bg` 的定义。

- [x] A2. 跟踪信号核查（`design.md` §7.1）：读 `ccr-ui/src-tauri/src/services/usage.rs` 759–807 行附近
      `UsageSourceHealth` 的构造点，确认 `source` 字段的实际取值来源，判断能否对应 `usageKey`。
      结论写入 `design.md` §7.1。

- [x] A3. ledger 引用面：`rg -n 'DashboardReadinessLedger' ccr-ui/src ccr-ui/tests`，
      确认除首页外是否还有引用，决定删文件还是只解引用。
      结论：仅 `DashboardView.tsx` 引用组件本身；删除 `DashboardReadinessLedger.tsx` 与 `dashboard-readiness-ledger.css`。

- [x] A4. 填写 `design.md` §7.2 的落位表六行，每行给出「迁移到 X」或「删除，理由 Y」。
      本步骤是 AC3 的直接产物，不得留「待定」。

- [x] A5. 组件测试落点：`rg -l 'DashboardPlatformMatrix' ccr-ui/tests`，
      有则扩展，无则确定新建文件名。落点写进本文件的 change list。
      结论：无既有文件，新建 `ccr-ui/tests/dashboard-platform-matrix.smoke.test.tsx`。

## 阶段 B：数据层

- [x] B1. `dashboardPresentation.ts`：`DashboardPlatformRow` 加 `sparkline?: number[]`，
      按 `design.md` §6 的字段对照在 `buildDashboardPresentation` 中派生。
- [x] B2. `dashboard-presentation.smoke.test.ts` 补四条断言（`design.md` §8 表第一行）。
- [x] B3. 跑一次 `dashboard-presentation.smoke.test.ts`，确认既有断言未被打断。

## 阶段 C：平台卡

- [x] C1. `DashboardPlatformMatrix.tsx` 原地改写为四张卡，props 契约不动（`design.md` §5）。
- [x] C2. `dashboard-platform-matrix.css` 重写卡片样式，全部走令牌，圆角用 `--radius-md` / `--radius-2xl` / `--radius-full`。
- [x] C3. 占位分支按 A2 结论实现：`source_health` 可对应则做占位态；不可对应则按 `design.md` §7.1 的替代呈现，不显示 0。
- [x] C4. 平台卡组件测试：`state: "missing"` 触发占位；全零 `series` 不触发占位。

## 阶段 D：栅格与 shell

- [x] D1. `DashboardView.tsx` 栅格重排（`design.md` §2），就绪 pill 与主行动按钮放区块标题行。
- [x] D2. 按 A4 的落位表迁移 readiness 信息，然后移除 `DashboardReadinessLedger` 引用。
      按 A3 结论决定是否删除文件。
- [x] D3. `dashboard-view.css` 新栅格 + 三档响应式，媒体查询用 px 字面量区间语法。
- [x] D4. `MainLayoutNav.tsx`：分组标题 mono 标签样式 + 平台色识别块。
- [x] D5. 若 A1 为结论 B，改 `shell.css` + `MainLayoutChrome.tsx` 接回 `--surface-shell-*`；
      若为结论 A，跳过并把两文件移出 change list。本任务为结论 A，已跳过。
- [x] D6. 新增文案的中英文键补齐。

## 阶段 E：验证

- [x] E1. `just frontend-check-quick`。
- [x] E2. 主题契约测试未被打断（见验证命令）。
- [x] E3. 视觉验证（见下）。

## 验证命令

```bash
just frontend-check-quick
```

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/dashboard-presentation.smoke.test.ts tests/apple-glass-surface-contract.smoke.test.ts
```

用于 AC5 与 §3 约束：presentation 断言全绿，且 chrome 相关断言未被改动。

```bash
rg -n '#[0-9a-fA-F]{3,8}|[0-9]+px' ccr-ui/src/features/usage/styles/dashboard-platform-matrix.css ccr-ui/src/features/usage/styles/dashboard-view.css
```

用于 AC9：颜色应无命中；px 命中只允许出现在 `@media (width ...)` 条件中，
其余位置（圆角、间距、字号）必须走令牌。

```bash
rg -n -- '--breakpoint-' ccr-ui/src/features/usage/styles/dashboard-view.css
```

用于 AC8：应无命中。

```bash
rg -n 'DashboardReadinessLedger' ccr-ui/src/features/usage/dashboard/DashboardView.tsx
```

用于 AC7：应无命中。

## 视觉验证

`cd ccr-ui && npm run dev`，浏览器打开首页。

- [x] 暗色 clay：侧栏 / 顶栏 / 内容区 / 卡片四层可辨（AC1）。
- [x] 顶栏只有面包屑与环境切换，无新增元素；就绪 pill 与主行动在首页区块标题行可见，pill 数字与 `readiness` 一致（AC2）。
- [x] 四张卡各有平台色条、版本、状态 chip、sparkline、请求与 TOKEN（AC4）。
- [ ] sparkline 柱数 = 当前 `activeDays`（AC4）。Web 预览无 series，柱数由 presentation 测试覆盖，桌面 IPC 下未目视。
- [x] 1440px / 1280px / 1024px 三档无横向滚动、无重叠、无截断；1024px 下平台卡降 2 列（AC8）。
- [x] Web 预览（无 IPC）下平台卡不显示误导性的 0（AC6）。
- [x] A4 落位表中标为「迁移到 X」的每一项，都能在新界面上找到（AC7）。
- [x] 四组主题×flavor 组合各扫一遍。

## 回滚

```bash
git checkout -- ccr-ui/src/shell/ ccr-ui/src/features/usage/dashboard/ ccr-ui/src/features/usage/styles/ ccr-ui/src/views/dashboard/dashboardPresentation.ts ccr-ui/tests/
```

若删除了 ledger 的两个文件，同一条命令会一并恢复。

## 提交

`feat(ui): ✨ 首页改为运行时卡布局`

change list 必须包含测试文件（父任务 XC5）。
提交前执行父任务 XC4 的三条检查，确认 `ccr-ui/src-tauri/Cargo.toml` 不在暂存区、不在提交中。
</content>
