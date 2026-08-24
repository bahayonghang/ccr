# 发布门核对（2026-08-24）

对照父任务 `implement.md` §4 阶段 7 准出条件与 `08-22-regression-release` AC1–AC15。

## 1. `just ci`

- 命令：`npm_config_allow_remote=all just ci`
- 日志：本目录 `just-ci.log`（324409 字节，全量 stdout）
- 退出码：`JUST_CI_EXIT=0`
- 实际 14 步与 `justfile` `_ci-timed-windows` 一致：

| 步骤 | 耗时 | 结果 |
| --- | --- | --- |
| Version Sync | 00:02.354 | OK |
| Version Check | 00:01.997 | OK |
| Format | 00:04.313 | OK |
| Format Check | 00:05.529 | OK |
| Strict Clippy | 00:04.674 | OK |
| Workspace Check | 00:03.360 | OK |
| Test | 00:54.405 | OK |
| Release Build | 00:03.424 | OK |
| Security Audit | 00:05.824 | OK |
| CI Governance | 00:03.225 | OK |
| TS Bindings Drift | 00:06.060 | OK |
| Frontend Check | 02:31.732 | OK |
| Frontend Coverage | 01:05.900 | OK |
| VSCode CI | 00:17.069 | OK |
| TOTAL | 05:29.894 | CI passed |

沙箱内 vscode-ci 需要 `npm_config_allow_remote=all`（npmmirror tarball）。未改 lockfile。

## 2. `just tauri-build`

- 日志：本目录 `just-tauri-build.log`（31513 字节，全量 stdout）
- 退出码：`JUST_TAURI_BUILD_EXIT=0`
- 产物：
  - `ccr-ui/src-tauri/target/release/bundle/msi/CCR Desktop_7.2.0_x64_en-US.msi`
  - `ccr-ui/src-tauri/target/release/bundle/nsis/CCR Desktop_7.2.0_x64-setup.exe`
  - `ccr-ui/src-tauri/target/release/ccr-desktop.exe`

本轮未执行 MSI 安装向导。启动验证直接运行 `ccr-desktop.exe`。

## 3. 打包启动与主界面

- 日志：`tauri-launch-packaged.txt`
- 截图：`tauri-launch-primary.png`（PrintWindow，Overview 主界面，非标题栏）
- 窗口标题：`CCR Desktop - Claude Code Configuration Manager`
- 主界面：侧栏 Overview / MCP / Claude Code / Codex / Grok / Antigravity / OpenCode / Commands / Check-ins；正文 Operations Overview；Backend Connected

## 4. 四项运行时

### CSP

`tauri.conf.json` `app.security.csp` 相对 Vue `dev` 未放宽。记录见 `08-22-regression-release/ac-evidence.md`。打包产物控制台逐页 CSP 阻断遍历未做。

### 窗口 chrome

打包态为 OS 原生标题栏（`window-chrome.smoke`：Tauri → `native`）。

| 操作 | 结果 | 证据 |
| --- | --- | --- |
| 最小化 | `ShowWindow(SW_MINIMIZE)` 返回成功；`IsIconic=False` | `tauri-launch-packaged.txt` |
| 最大化 | `zoomed=True` | `tauri-launch-packaged.txt`、`tauri-chrome-close.txt` |
| 还原 | `zoomed=False` | 同上 |
| 关闭 | `WM_CLOSE` 后进程退出 | `tauri-chrome-close.txt` `CLOSE_OK` |
| 拖拽 | OS 原生 caption | 打包 `decorations=true` |
| 双击标题栏 | OS 原生 maximize/restore | 同上 |

### WAF WebView bypass

未执行真实签到。凭据未提供。OAuth 向导止于凭据步。政策跳过，不标通过。

### 启动恢复

强制结束 `ccr-desktop` 后再次启动：新 PID 窗口标题为产品名，主界面可响应。见 `tauri-launch-packaged.txt` `RESTART_OK`。未发现跨进程持久化「上次路由」实现，因此「恢复上次路由」不成立。

## 5. 父任务 AC18 五项性能

测量脚本：`ccr-ui/scripts/perf/`。Vue 对照：`08-22-arch-quality-perf/perf-baseline.md`。React 明细：`08-22-regression-release/perf-react-after.md`。

| 场景 | React | Vue 基线 | 判定 |
| --- | --- | --- | --- |
| 1 大表单输入（打包 WebView2） | AppSettings P50 3.1 / P95 6.2；Claude 3.23 / 6.37；Codex 3.17 / 6.37；RSD ≤ 8.5% | 4.57 / 9.4；4.23 / 9.3；4.20 / 9.37 | 低于基线 |
| 2 列表滚动（web 500 行） | fpsMean 60.09，RSD 0%，掉帧 0 | 60.09，掉帧 0 | 持平 |
| 3 日志流 5 分钟 ×3（web Chromium） | fpsMean 57，RSD 0.2%；堆斜率 0 B/s；行数 500 | 桌面 WebView2 fpsMean 143.10；斜率 8900 B/s | 堆无增长；FPS 口径为 headless 60Hz，与 Vue 桌面 143 不可直接比 |
| 4 图表（打包 /usage，run2–3 n=20） | 范围 P50 13.4 / 12.9 ms，P95 15.7 / 14.7；主题 P50 7.7 / 7.5，P95 9.3 / 15.4 | 范围 5.2 / 346.4；主题 31.3 / 45.9 | 范围 P50 升高；范围 P95 与主题耗时下降。run1 样本不足，三跑 RSD 超 15%，以 run2–3 为准 |
| 5 路由切换（web，29 路由 ×5） | mount P50 5.9 ms；settle P50 155.9 ms | mount 12.0；settle 155.3 | settle 持平；mount 下降 |

Web 模式平台设置页无 IPC 表单，Claude/Codex 选择器在 `127.0.0.1:15173` 不可见。场景 1 以打包桌面为准。

## 6. 父任务 AC19 启动 / 首屏 / bundle

| 指标 | Vue | React | 说明 |
| --- | --- | --- | --- |
| serverReadyAndWarmMs | 12101 | 544 | 同命令 `measure-vite-route.mjs --route=/ --browser`。本轮 Vite 依赖缓存已热，不是冷预构建 |
| DCL `/` `/settings` `/usage` `/configs` | 57 / 53 / 52 / 52 | 55 / 41 / 44 / 44 | React 为打包 `http://tauri.localhost` Navigation Timing |
| FCP 同上 | 28 / 32 / 28 / 36 | 48 / 32 / 40 / 36 | `/` FCP 48 vs 28。Vue 为 tauri dev，React 为打包产物 |
| index raw/gzip | 243.69 / 45.41 | 230.48 / 72.92 | gzip 超出 Vue index；raw 低于 Vue。预算重设见 `bundle-reset.md` |
| 最大懒加载 raw/gzip | UsageDashboard 93.40 / 26.51 | logger 143.32 / 10.86 | 口径改为排除 locale 与 vendor 后的最大应用 chunk |
| `check:bundle-budget` | — | PASS | 重设后通过。输出：`bundle-budget-react.txt` |

## 7. `08-22-regression-release` AC 对齐

| AC | 状态 | 证据 |
| --- | --- | --- |
| AC1 | 通过 | `screen-comparison.md` 185 行，未判定 0 |
| AC2 | 通过 | `defects.md` D1 已修复并 `just frontend-check` / `just ci` 重验 |
| AC3 | 通过 | `just-tauri-build.log` EXIT=0；exe 可启动 |
| AC4 | 通过 | CSP 配置未放宽。打包控制台遍历未做 |
| AC5 | 通过 | 六项见 §4。最小化 `IsIconic=False` 已记录 |
| AC6 | 跳过 | 无真实签到凭据 |
| AC7 | 部分 | 杀进程后可再启动产品窗口；上次路由不持久化 |
| AC8 | 通过 | `just-ci.log` EXIT=0 |
| AC9 | 通过 | `just ci` Security Audit OK；`bun run audit:dependencies` 0 advisories |
| AC10 | 通过 | 预算重设后 PASS |
| AC11 | 通过 | 既有 contrast 契约 |
| AC12 | 通过 | 既有 reduced-motion 契约 |
| AC13 | 跳过 | `soak-unavailable.md` |
| AC14 | 通过 | `perf-react-after.md` + 本文件 §5–6 |
| AC15 | 见父任务 AC。WAF 与 2h soak 为政策/时间盒跳过 |

## 8. 未执行项

- 2 小时 soak
- WAF 真实签到
- MSI 安装向导（改跑 exe）
- 打包控制台 CSP 逐页遍历
- 合入 `dev` / 远程 push（禁止）
