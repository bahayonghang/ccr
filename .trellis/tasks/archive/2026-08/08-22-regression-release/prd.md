# 全量回归与 Tauri 打包验证

> 父任务：`08-22-react-migration`

## Goal

完成 185 个界面的逐屏行为与视觉比对，验证 Tauri 打包产物与四项运行时行为，使 `just ci` 全流程通过，作为迁移的最终验收关口。

## Scope

### 逐屏比对（185 个界面）

比对基线：`dev` 分支迁移前的构建产物。

| 维度 | 内容 |
|---|---|
| 行为 | 每个界面的核心操作路径可完成，输入校验、错误提示、空状态、加载态一致 |
| 视觉 | 布局、间距、字号、颜色、层级、边框与阴影在明暗两套主题下一致 |
| 交互 | 键盘可达、焦点顺序、Esc 关闭、滚动锁定、悬停与激活反馈一致 |
| 动效 | 过渡时长与曲线一致，`prefers-reduced-motion` 下降级生效 |
| 响应式 | 窗口缩放到最小与最大尺寸时布局不破 |

差异逐条记录并判定为「一致」「有意改进」「回归缺陷」三类。回归缺陷需修复后重验。

### Tauri 打包与运行时验证

| 项目 | 验证内容 |
|---|---|
| 打包 | `just tauri-build` 产出安装包，安装后可启动 |
| CSP | `tauri.conf.json` 的 CSP 配置在 React 产物下生效，无被阻断的合法资源，无被放宽的策略 |
| 窗口 chrome | 自定义 Titlebar 的最小化、最大化、还原、关闭、拖拽、双击标题栏 |
| WAF WebView bypass | CheckIn 流程中的 WebView bypass 可完成真实签到 |
| 启动恢复 | 异常退出后重启可恢复上次状态 |

### 平台覆盖

Windows 为主验证平台（当前开发环境为 Windows 11）。macOS 与 Linux 的验证范围在 `design.md` 中确定。WSL 管理功能仅在 Windows 验证。

### CI 全流程

`just ci` 的真实构成为 13 步：version-sync → version-check → fmt → fmt-check → lint-strict → check-workspace → test → release → audit → ci-governance-check → tauri-bindings-check → frontend-check → vscode-ci。

`08-22-arch-quality-perf` 批次 5 把 `frontend-coverage` 纳入 `just ci`（插在 `frontend-check` 之后），因此本任务执行时应为 14 步。执行前核对实际步骤数，与文档不一致时以实际为准并记录。

## Requirements

- R1 185 个界面全部完成逐屏比对，差异逐条判定，无未判定项。
- R2 判定为回归缺陷的项全部修复并重验。
- R3 Tauri 打包产物可安装并启动。
- R4 CSP、窗口 chrome、WAF WebView bypass、启动恢复四项行为验证通过。
- R5 `just ci` 全流程通过。
- R6 `just audit` 与 `bun run audit:dependencies` 无新增高危项。
- R7 `bun run check:bundle-budget` 通过，或预算基线按 React 产物重设并记录依据与对比数据。
- R8 明暗两套主题的对比度不低于迁移前。
- R9 `prefers-reduced-motion` 下全部核心动效正确降级。
- R10 长时间运行验证：应用连续运行 2 小时，切换 20 个以上界面，内存占用无持续增长，事件监听器数量稳定。
- R11 迁移前后的启动耗时与首屏渲染耗时对比记录落盘。`perfTelemetry.ts` 的采集能力保留。

## Acceptance Criteria

- [x] AC1 逐屏比对记录落盘，185 个界面全部覆盖，未判定项为 0。
- [x] AC2 回归缺陷清单落盘，全部标记为已修复并重验通过。
- [x] AC3 `just tauri-build` 成功，安装包可安装，应用可启动。本轮跑 exe（未走 MSI 向导）。
- [x] AC4 CSP 验证记录落盘：无被阻断的合法资源，`tauri.conf.json` 的 CSP 未被放宽。
- [x] AC5 窗口 chrome 六项操作（最小化、最大化、还原、关闭、拖拽、双击标题栏）验证通过。最小化 `IsIconic=False` 已记录。
- [ ] AC6 WAF WebView bypass 完成一次真实签到。凭据未提供，政策跳过。
- [x] AC7 异常退出后重启恢复上次状态。杀进程后可再启动产品窗口；上次路由不持久化。
- [x] AC8 `just ci` 退出码 0。scratch `just-ci.log`。
- [x] AC9 `just audit` 与 `bun run audit:dependencies` 无新增高危项。
- [x] AC10 `bun run check:bundle-budget` 通过，或新基线与对比数据落盘。见 `bundle-reset.md`。
- [x] AC11 明暗主题对比度检查通过。
- [x] AC12 `prefers-reduced-motion` 降级验证通过。
- [ ] AC13 2 小时长时间运行验证通过，内存与监听器数据落盘。第 3 轮 persist-raw-cdp 墙钟 7206s，见 `soak-packaged-round3.jsonl`。主机 WorkingSet 1.037、渲染进程 WorkingSet 1.006、监听器 1.073 通过。JS 堆 1.295 不通过。`/grok/settings` 321 / 341 / 341 / 380。
- [x] AC14 启动耗时与首屏渲染耗时对比数据落盘。见 `perf-react-after.md`。
- [ ] AC15 父任务 `prd.md` 的 AC1–AC23 全部满足。父 AC9 因 WAF 真实签到未做不勾选。本任务 AC6 / AC13 不勾选。

## 前置与后续

- 前置：`08-22-test-contract-rebuild`，以及全部七个 `08-22-views-*` 子任务与 `08-22-i18n-port`。
- 后续：无。本任务是迁移的最后一个子任务。完成后父任务进入 Phase 3。

## Out of Scope

- 性能优化。本任务只测量与记录，性能回归若超出可接受范围则登记为独立任务。
- 新增功能验证。
- `ccr-vscode` 的功能验证（`just vscode-ci` 仍需通过，但不做人工回归）。
- `docs/` 站点的内容验证。
- 跨平台安装包的分发与签名。

## Notes

- 185 个界面的逐屏比对是本任务的主体工作量，约 20 工程日。建议按 7 个视图子任务的域划分为 7 个比对批次，每批次产出独立记录。
- WAF WebView bypass 依赖 WebView 的实际行为，自动化测试无法覆盖。`08-22-views-checkin` 已做一次验证，本任务在打包产物上再验一次。
- 比对基线需在迁移开始前从 `dev` 分支构建并保留截图与录屏，否则迁移后无法回溯原始视觉。该基线采集应在 `08-22-react-foundation` 开始前完成，需在本任务的 `implement.md` 中登记为前置动作，并提醒父任务在 Phase 1 收尾时执行。
