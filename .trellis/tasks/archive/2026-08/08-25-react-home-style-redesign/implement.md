# 父任务执行计划

父任务本身不写业务代码。它负责排序、集成校验与跨子任务验收（XC1–XC4）。

## 执行顺序

```
阶段 0  arch-drift-docs              （与其余并行，无依赖）
阶段 1  design-token-consolidation   （必须先落地，其余子任务消费其令牌）
阶段 2  home-runtime-layout          （建立首页栅格骨架，其余首页子任务在其骨架内改）
阶段 3  home-usage-chart ‖ home-side-rail ‖ appearance-settings-refresh （可并行）
阶段 4  父任务集成走查与验收
```

阶段 2 是首页栅格的唯一负责人。阶段 3 的三个子任务只改自己的组件内部与自己的 CSS 文件，不改 `DashboardView.tsx` 与 `dashboard-view.css`，避免同文件冲突。
`home-usage-chart` 与 `home-side-rail` 都依赖阶段 2 的栅格，此处与 `prd.md` 的 Task Map 一致。

## 基线

父任务 `task.json` 的 `base_branch` 为 `dev`。`main` 落后 `dev` 197 个提交，用 `main` 做基线会把无关历史混进 diff 审阅。
所有 diff 范围检查以本任务起始时的 `dev` HEAD 为基准。

规划完成时的 `dev` HEAD 为 `a4d3e480`。第一个子任务开工时重新取一次并写回本节，
因为 `dev` 在规划与实施之间可能前进：

```bash
BASELINE=$(git rev-parse HEAD)
echo "$BASELINE"
```

开工基线（2026-08-25）：`a4d3e4806b2f6bbe100b9a8c9467330ce730b437`（与规划时 `dev` HEAD 一致）。后续 `$BASELINE` 一律使用该值。

后续所有出现 `$BASELINE` 的检查命令都用这个值。同一轮集成走查内保持一致，不中途重取。

## 集成检查清单（阶段 4）

- [x] 六个子任务全部 done，各自的验收标准已勾。
      归档目录 `.trellis/tasks/archive/2026-08/08-25-{arch-drift-docs,design-token-consolidation,home-runtime-layout,home-usage-chart,home-side-rail,appearance-settings-refresh}`。
      产品提交：`0ebc75a0` `1147e6ac` `a6c22e40` `00e6704f` `8c71743a` `3d467d44`；各自随后 `chore(task): archive …`。
- [x] XC1：对本任务改动的 CSS 文件清单执行硬编码扫描，无命中。
      文件清单从 `git diff --name-only <baseline> -- '*.css'` 取，不扫整目录，避免命中既有历史代码。

      ```bash
      git diff --name-only "$BASELINE" -- '*.css' | xargs -r rg -n '#[0-9a-fA-F]{3,8}|[0-9]+px'
      ```

      解释：`tokens.css` 的 hex/px 是令牌定义，预期命中。组件 CSS 无 hex；px 只出现在 `@media (width …)`（`dashboard-view.css` 1440/1024，`dashboard-platform-matrix.css` 1024）以及 `shell.css` 既有历史字面量（本任务 diff hunk 未新增）。`dashboard-readiness-ledger.css` 已删除。
      已知误报源：`ccr-ui/src/features/configs/lib/flavorPreview.ts` 是 `.ts`，不在本扫描范围；
      其取值一致性由 `08-25-appearance-settings-refresh` 单独负责。
- [x] XC2：`just version-check` → `just fmt-check` → `just frontend-check` 全通过。
      退出码：version-check=0（`$env:PYTHONUTF8='1'`）；fmt-check=0；frontend-check=0（终端 983936，约 191s）。`just frontend-check-quick`=0（终端 381092）。用户发布门不含 `just ui-check`。
- [x] XC3：全页回归走查完成，截图与结论写入 `research/regression-walkthrough.md`。
      走查页面：Dashboard、Profiles、MCP、Commands、Sync、Check-ins、Usage、Settings。
      走查组合：`light×neutral`、`light×clay`、`dark×neutral`、`dark×clay`。
      走查维度：边框可见性、圆角一致性、mono 误用、对比度、横向溢出。
      32 组合 overflowX=0；`data-theme`/`data-flavor` 与 dock 文案对齐；最低标题对比 11.58；1024px 平台卡 2 列。
- [x] XC4：`ccr-ui/src-tauri/Cargo.toml` 未进入暂存区，也未出现在 `a4d3e480..HEAD`。
      用户目标是该文件保持工作区脏且未暂存。实测：`git diff --name-only` 与 `--cached` 均为空；`git log a4d3e480..HEAD -- ccr-ui/src-tauri/Cargo.toml` 为空。早期 porcelain 的 ` M` 为 CRLF/stat 幽灵（blob 与 HEAD 相同），未写入任何提交。
- [x] XC5：每个 UI 子任务的提交都包含测试文件改动。
      `git log --name-only "$BASELINE"..HEAD -- 'ccr-ui/tests/**'` 非空：
      `1147e6ac` token-consolidation + theme-contrast；`a6c22e40` platform-matrix + presentation；`00e6704f` usage-movement；`8c71743a` next-actions + signal-stream；`3d467d44` flavor-preview-consistency。docs-only `0ebc75a0` 无测试（非 UI 行为）。
- [x] XC6：名称增量登记完成，`theme-token-contracts.md` 已同步（+6：`--color-*-tint` 四个 + `--color-platform-opencode` / `-rgb`）。
- [x] AC2 响应式三档抽查：1440px、1280px、1024px。overflowX=0；列数 4 / 4 / 2。截图 `research/shots/dashboard-dark-clay-{1440,1280,1024}.png`。
- [x] AC4：`prefers-reduced-motion: reduce` 下首页抽样 animationName=0；`theme-contrast-contract.smoke.test.ts` 文本门槛仍为 primary≥12、secondary≥7、muted≥4.5、accent≥3.5、border≥1.2；边框改为实色 `a===1`，未下调阈值。
- [x] AC7：中英文文案键完整，无缺键回退。i18n leaf-count 4178，zh=en；`just frontend-check` 含 i18n 门。
- [x] AC9：readiness 信息逐项落位结论已在 `design.md` §D-2 六行表勾完。

## 阶段 4 命令退出码

| 命令 | 退出码 | 备注 |
|---|---|---|
| `just version-check` | 0 | `PYTHONUTF8=1`，全仓 7.2.0 |
| `just fmt-check` | 0 | JSON + cargo fmt |
| `just frontend-check-quick` | 0 | 终端 381092 |
| `just frontend-check` | 0 | 终端 983936 |
| XC3 Playwright 走查 | 0 | 32 组合 theme 对齐，overflowFails=0 |
| `git status --porcelain -uall` | 0 | 归档前仅父任务 `??`；Cargo.toml 不在列表 |

首页视觉验证走 Web 预览：`ccr-ui` 下 `bun run dev:web -- --host 127.0.0.1 --strictPort`，打开 `http://127.0.0.1:5173/` 与 `/settings`。
Web 预览无 Tauri IPC，native-only 的不可用状态按 AC5 保持诚实展示，不得为了截图伪造数据。

## 回滚点

- 阶段 1 之后：`tokens.css` 单文件还原即可回到旧视觉。
- 阶段 2 之后：`DashboardView.tsx` + `dashboard-view.css` 还原即可回到旧栅格，各组件仍能渲染。
- 阶段 3/4：各子任务改动限定在自己的组件与 CSS 文件，可单独 revert。

## 提交约定

- 每个子任务独立提交，scope 用 `ui`：`feat(ui): ✨ ...` / `refactor(ui): ♻️ ...` / `docs: 📝 ...`。
- 分支 `dev`。子任务不各自开分支，除非用户另行要求。
- 提交前确认 `ccr-ui/src-tauri/Cargo.toml` 未被 `git add`。
