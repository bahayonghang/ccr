# AC 证据与未执行项

> 任务：`08-22-regression-release`。本会话未跑 `just ci`、`just tauri-build`。

## 本会话已勾选

| AC | 状态 | 证据 |
| --- | --- | --- |
| AC1 | 通过 | `screen-comparison.md`：185 行，未判定 0，缺陷 0（视觉） |
| AC4 | 配置通过；打包控制台遍历未做 | 见下节 CSP |
| AC11 | 通过（token / 契约测试，非打包复验） | `theme-contrast-contract.smoke.test.ts` 22/22；`08-22-design-system/contrast-parity.md` 四组合 PASS，与迁移前 token 取值差 0 |
| AC12 | 通过（单点收敛测试） | `reduced-motion.smoke.test.tsx` 4/4；`src/styles` 内 `@media (prefers-reduced-motion)` 仅 `shell-critical.css` 一处兜底 |

## 本会话不勾选

| AC | 原因 |
| --- | --- |
| AC2 | `defects.md` D1 未修复 |
| AC3 | 未跑 `just tauri-build` |
| AC5 | 窗口六项未在打包产物上操作；smoke 只锁 chrome 模式 |
| AC6 | 无真实签到；OAuth 止于凭据步 |
| AC7 | `startup-recovery.smoke` 只覆盖致命启动占位，不覆盖杀进程后恢复上次路由 |
| AC8 | 未跑 `just ci` |
| AC9 | 未跑 `just audit` / `bun run audit:dependencies` |
| AC10 | 构建失败，未跑 `bun run check:bundle-budget` |
| AC13 | 见 `soak-unavailable.md` |
| AC14 | React 侧五项场景与启动/FCP 未测；Vue 基线已在 `baseline/startup-timings.md` 与 `perf-baseline.md` |
| AC15 | 父任务 AC1–AC23 未全部满足，见 `parent-ac-status.md` |

## CSP（AC4）

`ccr-ui/src-tauri/tauri.conf.json` `app.security` 与 `origin/dev` 相同：

```json
{
  "csp": {
    "default-src": "'self' customprotocol: asset:",
    "connect-src": "ipc: http://ipc.localhost",
    "img-src": "'self' asset: http://asset.localhost blob: data:",
    "style-src": "'unsafe-inline' 'self'"
  },
  "devCsp": null,
  "dangerousDisableAssetCspModification": false
}
```

- 策略未被放宽（相对 Vue `dev`）。
- `code-source-editor.smoke.test.tsx`：页面 CSP nonce 读取。
- 未做：打包产物上逐界面看控制台 CSP 阻断。无安装包。

## 窗口 chrome（AC5）

`tests/window-chrome.smoke.test.ts` 2/2：

- Tauri 桌面（windows / macos / linux）→ `native`
- 非 Tauri 预览 → `custom`

`Titlebar.tsx`（仅 custom 模式挂载）：最小化、最大化/还原、关闭、`data-tauri-drag-region`。自定义 Titlebar 无双击标题栏处理；打包态为 OS 原生 chrome。

`native-window-appearance.smoke.test.ts` 2/2：macOS 同步主题色；Windows 跳过。

未做：打包应用上六项手测。

## WAF（AC6）

已跑：

- `checkin-waf-event-wait.smoke.test.ts` 4/4：`checkin:job-finished` 事件等待，无轮询
- `checkin-runtime-coverage.smoke.test.ts` 10/10：WAF 补救路径覆盖

未做：真实 WebView bypass 签到。不要求付费凭据。OAuth 向导对照止于凭据录入（`oauth-wizard-branches.md`；基线 `oauth-wizard-desktop.mp4`）。

`08-22-views-checkin` AC4 在其 prd 上仍未勾选。本任务 AC6 不勾选。

## 启动恢复（AC7）

`startup-recovery.smoke.test.ts` 1/1：`renderFatalStartup` 把致命错误写入 `#app`。

未找到跨进程持久化「上次路由」的实现（无 `lastRoute` / `lastPath` 存储）。`cache-route.smoke.test.ts` 6/6 覆盖的是进程内缓存路由 store / 滚动，不是杀进程后恢复。

异常退出后重启恢复上次状态：未执行。

## 对比度（AC11）

- `theme-contrast-contract.smoke.test.ts` 22/22
- `contrast-parity.md`：light/dark × neutral/clay，主/次/弱文案与 accent 对比度全部 PASS；与迁移前同名颜色 token 取值差 0

未做：打包产物上再算一遍像素对比度。token 契约与迁移前一致。

## reduced motion（AC12）

- `reduced-motion.smoke.test.tsx`：根 `data-reduced-motion` 同步；`src/styles` 仅 `shell-critical.css` 保留一处 `@media (prefers-reduced-motion)`
- `overlay-single-implementation.smoke.test.ts` 3/3：弹层焦点/Esc/滚动锁只有 Radix 一处

未做：打包产物上手测全部核心动效降级。

## `just frontend-check`（2026-08-24）

| 步骤 | 结果 |
| --- | --- |
| type-check | 通过 |
| lint:ci（eslint + stylelint + style-lines） | 通过 |
| check:cycles（666 文件） | 通过 |
| check:arch-boundaries | 通过 |
| test:i18n（4164 叶子 key）+ key-leak self-test | 通过 |
| test:smoke 108 文件 / 511 测试 | 通过 |
| frontend-build | **失败** D1 |
| docs-check | 未跑到 |

## 跨平台（implement.md 步骤 8）

| 平台 | 状态 |
| --- | --- |
| Windows | 部分：Web/smoke/类型与 lint。未安装打包应用 |
| macOS | 未执行。本环境为 Windows 11。不标通过 |
| Linux | 未执行。不标通过 |

## 性能（AC14 / R11）

Vue 基线已落盘：

- `baseline/startup-timings.md`：DCL / FCP（`/` `/settings` `/usage` `/configs`）
- `08-22-arch-quality-perf/perf-baseline.md`：五场景 Vue 侧
- `baseline/bundle-budget.txt`：Vue index 243.69 KiB raw

React 侧五项场景与启动/FCP 本会话未测。`perfTelemetry.ts` 仍由 `main.tsx` `initPerfTelemetry()` 接入。无安装包、且生产构建失败，故无 React 对比表。

## i18n 泄漏脚本

`bun ./scripts/detect-i18n-key-leak.mjs --self-test` 通过（4164 叶子 key）。未在运行中的 75 条路由上对中英文界面做泄漏扫描（无稳定 dev 截图会话）。
