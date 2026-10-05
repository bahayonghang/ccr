# 执行计划：首页右侧栏

前置：`08-25-design-token-consolidation` 与 `08-25-home-runtime-layout` 已合入。

开工前读 `08-25-design-token-consolidation/research/token-name-delta.md`，
取 accent / warning / danger tint 的实际取值形式（`design.md` §4 说明为何不在设计里写死）。

## 检查清单

- [x] 1. 测试落点：`rg -l 'DashboardSignalStream' ccr-ui/tests`，确定扩展还是新建，写进 change list。
- [x] 2. 先写测试固定现状（`design.md` §6 四条），在改代码之前跑通——这些断言描述的是既有行为。
- [x] 3. `DashboardSignalStream.tsx` 行版式改为四列栅格（`design.md` §2），保留筛选、`channel`、聚合 `×N`、空态 CTA、页脚链接。
- [x] 4. `dashboard-signal-stream.css` 版式重写；错误/警告行 tint 底色，信息行无底色。
- [x] 5. 三档的图标与可读文本（`design.md` §5），保证灰度下可辨。
- [x] 6. 重跑第 2 步的测试，确认改版没有改变筛选、计数与聚合语义。
- [x] 7. `DashboardNextActions.tsx` 首条强调态，其余静默态。
- [x] 8. `dashboard-next-actions.css` 版式重写。
- [x] 9. 两块的空态与超长文本省略。
- [x] 10. 新增文案的中英文键补齐。
- [x] 11. `just frontend-check-quick`。父任务集成 `QUICK_EXIT=0`（终端 381092）。
- [x] 12. 视觉与交互验证（见下）。父任务 XC3 四组合截图确认首条强调态与四列事件行。

第 2 步先于第 3 步是有意的：先把既有语义锁进测试，再改版式，改坏了立即可见。

## 验证命令

```bash
just frontend-check-quick
```

```bash
cd ccr-ui && bunx vitest run --config vitest.smoke.config.ts tests/dashboard-signal-stream.smoke.test.tsx
```

用于 AC3 / AC4 / AC5 / AC6 / AC10。文件名以第 1 步的落点结论为准。

```bash
rg -n 'PillToggleGroup|dashboard-signal__channel|entry.count' ccr-ui/src/features/usage/dashboard/DashboardSignalStream.tsx
```

用于 AC2 / AC3 / AC5：三者都必须仍有命中，证明能力未被删除。

```bash
rg -n '#[0-9a-fA-F]{3,8}|border-radius:\s*[0-9]+px' ccr-ui/src/features/usage/styles/dashboard-signal-stream.css ccr-ui/src/features/usage/styles/dashboard-next-actions.css
```

用于 AC9：应无命中。

## 视觉与交互验证

`cd ccr-ui && npm run dev`。

- [x] 「下一步」首条强调态明显区别于其余（AC1）。
- [x] 事件流行含时间戳、状态点、channel、文本四列（AC2）。
- [x] 三档筛选可切换，`warn` 档结果含 error 级条目（AC3）。
- [x] 相邻重复条目显示 `×N`（AC5）。
- [x] 清空数据后两块各自显示空态与 CTA（AC6、AC8）。
- [x] 灰度模拟下仍能区分错误、警告、信息三档（AC7）。
- [x] 超长文本省略，无横向滚动（AC8）。
- [x] 四组主题×flavor 组合各扫一遍。

## 回滚

```bash
git checkout -- ccr-ui/src/features/usage/dashboard/DashboardNextActions.tsx ccr-ui/src/features/usage/dashboard/DashboardSignalStream.tsx ccr-ui/src/features/usage/styles/dashboard-next-actions.css ccr-ui/src/features/usage/styles/dashboard-signal-stream.css ccr-ui/tests/
```

## 提交

`feat(ui): ✨ 首页右侧栏改为紧凑动作与事件行`

change list 必须包含测试文件（父任务 XC5）。
提交前执行父任务 XC4 的三条检查，确认 `ccr-ui/src-tauri/Cargo.toml` 不在暂存区、不在提交中。
