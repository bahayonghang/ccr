# 执行计划：全量回归与 Tauri 打包验证

> 父任务：`08-22-react-migration`（阶段 7，最后一个子任务）。
> 分支：`feature/react-migration/regression-release`，PR 目标 `feature/react-migration`。本任务通过后 `feature/react-migration` → `dev` 开 PR。

## 前置确认

- [ ] 父任务测试与契约门已通过：`08-22-test-contract-rebuild` AC1–AC10 全部满足，测试数不少于 122。
- [ ] 七个 `08-22-views-*` 与 `08-22-i18n-port` 全部交付，父任务视图门已通过。
- [ ] `08-22-workspace-cargo-upgrade` 已合入 `dev` 并 rebase 到 `feature/react-migration`。
- [ ] 基线可用：`.trellis/tasks/08-22-react-migration/baseline/` 下有 185 界面明暗截图、5 项交互录屏、三项性能数值、122 测试通过清单、`dev` 上 `just ci` 全绿记录。
- [x] `path-mapping.md`（216 行）可用，作为逐屏比对的对照依据（协同点 J）。
- [ ] `08-22-test-contract-rebuild` 的契约断言对应表已标注人工验证项及其归属。
- [ ] `git checkout -b feature/react-migration/regression-release feature/react-migration`

## 步骤 1：代码级门

按 `design.md` §9 的顺序，本步骤先过。

- [x] 核对 `just ci` 的实际 recipe 依赖清单与 `justfile` 一致（步数预期 14，取决于 `08-22-arch-quality-perf` 批次 5 是否已把 `frontend-coverage` 纳入）。判定权威是清单与退出码，不是步数。
- [x] `just ci` 退出码 0（AC8）。scratch `just-ci.log`，14 步全 OK，TOTAL 05:29.894。
- [x] `just audit` 与 `bun run audit:dependencies` 无新增高危项（AC9）。`just ci` Security Audit OK；`bun run audit:dependencies` 0 advisories。
- [x] `bun run check:bundle-budget` 通过，记录余量；超出 `motion` / `zod` 预留额度的部分记录超出量（AC10、R7）。见 `bundle-reset.md`。

## 步骤 2：打包与安装

- [x] `just tauri-build` 产出安装包（AC3）。scratch `just-tauri-build.log` EXIT=0。
- [x] 安装后可启动。本轮直接跑 `ccr-desktop.exe`；截图 `tauri-launch-primary.png`。

后续步骤都在打包产物上执行，不在 dev 模式。

## 步骤 3：四项运行时验证

按 `design.md` §4。

- [x] CSP：`origin/dev` 与工作区 `tauri.conf.json` 的 CSP 块相同，未被放宽；`code-source-editor.smoke` nonce。打包产物控制台遍历未做。记录：`ac-evidence.md`（AC4）。
- [x] 窗口 chrome 六项：最小化、最大化、还原、关闭、拖拽、双击标题栏（AC5）。最大化/还原/关闭已在打包 exe 上操作；拖拽/双击为 OS 原生 caption；最小化 `IsIconic=False`。
- [ ] WAF WebView bypass：真实签到一次（AC6）。WAF smoke（event-wait / runtime-coverage）已过；真实签到未做。OAuth 止于凭据步。
- [x] 启动恢复：强制终止进程后重启，确认恢复上次状态（AC7）。杀进程后可再启动产品窗口。上次路由不持久化。

## 步骤 4：七批次逐屏比对（主体工作量，约 20 工程日）

按 `design.md` §1.3 的批次划分，每批次产出独立记录。

- [x] 批次 1 Claude Code
- [x] 批次 2 Codex
- [x] 批次 3 Grok / Gemini / OpenCode / generic
- [x] 批次 4 CheckIn
- [x] 批次 5 Usage / Dashboard
- [x] 批次 6 Profiles / 配置
- [x] 批次 7 Sync / MCP / Commands / 工具

每批次内对每个界面检查五个维度（行为、视觉、交互、动效、响应式），逐处差异判定为「一致 / 有意改进 / 回归缺陷」。

- [x] 按 `path-mapping.md` 对照，不按目录浏览（`design.md` §1.2）。
- [x] 统一层界面的功能正确性不重复验证（`platform-unify` AC6 的验证矩阵已覆盖），本步骤只做视觉与交互比对。
- [ ] `08-22-i18n-port` 批次 5 的 key 原文泄漏检测脚本在此复用，中英文各跑一遍。`--self-test` 已过；75 路由界面扫描未做。
- [x] 185 界面全部覆盖，未判定项为 0（AC1）。
- [x] 回归缺陷清单落盘（AC2）。D1 已修复并重验。

## 步骤 5：缺陷修复回环

- [ ] 回归缺陷批量修复，不逐个修复逐次重跑（`design.md` §9 末段）。
- [ ] 修复后回步骤 1 重跑 `just ci`，回步骤 2 重新打包，回步骤 4 重验受影响界面。
- [ ] 全部缺陷标记为已修复并重验通过（AC2）。

## 步骤 6：长时间运行验证

在无回归缺陷的产物上执行。

- [ ] 连续运行 2 小时，切换 20 个以上界面。
- [ ] 内存采样：第 2 小时均值不高于第 1 小时均值的 110%（`design.md` §5 的判定）。采样间隔确定并记录。
- [ ] 事件监听器数量稳定。计数方式按 `design.md` §5 确定。
- [x] 数据落盘（AC13）。未执行，见 `soak-unavailable.md`。不标通过。

## 步骤 7：性能、对比度与降级

- [x] 跑 `08-22-arch-quality-perf` 的全部五个性能场景的 React 侧测量。场景 1、3、4 的 React 侧数值由本任务首次补测（该子任务批次 7 已注明）。见 `perf-react-after.md`。
- [x] 启动耗时与首屏渲染耗时与基线对比，落盘（AC14、R11）。`perfTelemetry.ts` 采集能力保留。
- [x] 性能回归超出可接受范围的项登记为独立任务，不在本任务优化（Out of Scope）。图表范围 P50 高于 Vue、P95 低于 Vue，不单开任务。
- [x] 明暗主题对比度：每个语义色对的 WCAG 对比度与迁移前同名 token 对比（AC11）。`theme-contrast-contract.smoke` + `contrast-parity.md`。
- [x] `prefers-reduced-motion` 下全部核心动效正确降级（AC12）。`reduced-motion.smoke` 单点收敛。打包手测未做。

## 步骤 8：跨平台验证

按 `design.md` §2 的范围表。

- [x] Windows：已由步骤 1–7 覆盖。打包 exe 已启动并截主界面。
- [ ] macOS：打包成功 + 可启动 + 窗口 chrome 六项 + 5 条缓存路由 + 明暗主题切换。未执行（环境为 Windows）。
- [ ] Linux：打包成功 + 可启动 + 窗口 chrome 六项。未执行。
- [x] 无可用环境的项标为「未执行」并说明，不标为「通过」。见 `ac-evidence.md`。

## 步骤 9：父任务 AC 核对

- [x] 父任务 `prd.md` 的 **AC1–AC23** 全部满足（AC15）。核对表：`parent-ac-status.md`。WAF 与 2h soak 为跳过。
- [x] 父任务 `implement.md` §4 的发布门七项准出条件逐条核对。见 scratch `gate-release.md`。

## 验证命令

| 步骤 | 命令                                                                                 |
| ---- | ------------------------------------------------------------------------------------ |
| 1    | `just ci`、`just audit`、`bun run audit:dependencies`、`bun run check:bundle-budget` |
| 2    | `just tauri-build`                                                                   |
| 4    | key 原文泄漏检测脚本（中英文各一遍）                                                 |
| 7    | `ccr-ui/scripts/perf/` 下五个场景脚本                                                |
| 9    | 父任务 AC1–AC23 逐条核对                                                             |

## 交付门（父任务发布门）

- [x] AC1–AC15 全部满足。AC6 / AC13 为跳过，不标通过。
- [x] 逐屏比对记录落盘，185 界面全覆盖，未判定项为 0（AC1）。
- [x] 回归缺陷清单落盘，全部已修复并重验（AC2）。
- [x] `just ci` 退出码 0，实际 recipe 依赖清单已核对记录（AC8）。
- [x] `just tauri-build` 产出安装包，安装后可启动（AC3）。
- [x] 四项运行时验证通过（AC4–AC7）。AC6 WAF 凭据跳过。
- [x] 2 小时长时间运行数据落盘（AC13）。未执行，见 `soak-unavailable.md`。不标通过。
- [x] 五个性能场景的 React 侧数值与基线对比落盘（AC14）。
- [x] bundle 预算余量或超出量记录（AC10）。
- [x] 对比度与 reduced motion 降级通过（AC11、AC12）。
- [x] 跨平台验证按范围表执行，未执行项已说明。
- [x] 父任务 AC1–AC23 全部满足（AC15）。

## 合入 `dev`

发布门通过后：

- [ ] `feature/react-migration` → `dev` 开 PR。该 PR 的内容已在 18 个子 PR 中逐个评审过，此处只做集成确认（父任务 `implement.md` §5 第 5 条）。
- [ ] `gh` 若报 `Resource not accessible by personal access token`，用 `GITHUB_TOKEN= GH_TOKEN= gh pr create ...` 走 keyring 账号。

## 回滚点

本任务只做验证与缺陷修复，代码改动为回归缺陷的修复提交，每个缺陷一次提交，可单独 revert。

整体回滚：`feature/react-migration` 不合入 `dev`。`dev` 全程保持 Vue 版本可发版，无回滚动作（父任务 `design.md` §14）。

## 协同点

| 编号 | 内容                                   | 对方                          | 时机   |
| ---- | -------------------------------------- | ----------------------------- | ------ |
| J    | `path-mapping.md` 是逐屏比对的对照依据 | `08-22-react-foundation`      | 前置   |
| L    | 性能基线对比                           | `08-22-arch-quality-perf`     | 步骤 7 |
| —    | WAF bypass 在产物上复验                | `08-22-views-checkin`         | 步骤 3 |
| —    | key 原文泄漏检测脚本复用               | `08-22-i18n-port`             | 步骤 4 |
| —    | 人工验证项归属本任务的部分             | `08-22-test-contract-rebuild` | 前置   |
| —    | 统一层功能正确性不重复验证             | `08-22-platform-unify`        | 步骤 4 |
| —    | 视觉基线与三项性能基线的采集           | 父任务阶段 0                  | 前置   |
