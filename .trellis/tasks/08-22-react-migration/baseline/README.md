# 迁移前基线（Phase 0）资产清单

采集自 `dev` 等价内容（feature/react-migration 分支，`git diff dev -- ccr-ui crates` 为空），Vue 版 v7.2.0。

## 资产

| 文件/目录 | 内容 | 采集方式 |
| --- | --- | --- |
| `screens/light/*.png`（75） | 75 条路由亮色主题截图 | Vite dev :4180 + 无头 Chromium，1800x1125 视口 |
| `screens/dark/*.png`（75） | 75 条路由暗色主题截图 | 同上，localStorage `ccr-theme=dark` |
| `recordings/cached-routes.mp4` | 5 条缓存路由（dashboard/grok/commands/configs/usage）离开与返回 | Web 模式录屏 |
| `recordings/oauth-wizard-desktop.mp4` | OAuth 向导：入口 → 提供商选择 → Session/Cookies 凭据录入步 | 桌面运行时录屏 |
| `recordings/log-stream.mp4` | 监控页实时日志流（刷新/级别筛选/路由切换期间累积） | 桌面运行时录屏 |
| `recordings/chart-time-range.mp4` | 用量图表时间范围切换 | 桌面运行时录屏 |
| `recordings/large-form-input.mp4` | 大表单输入（configs 搜索 + Claude Code 设置页长文本键入） | 桌面运行时录屏 |
| `smoke-test-run.txt` | `just frontend-test` 全量通过输出（123 文件 / 626 测试，其中 smoke 122 项口径见 prd 基准表） | 本地运行 |
| `coverage-run.txt` | `just frontend-coverage` 输出，lines 75.4% | 本地运行 |
| `bundle-budget.txt` | bundle 预算检查输出（index 243.69KiB raw 等） | 构建后运行 |
| `route-timing-settings.json` | measure-vite-route 冷启动测量原始输出 | 本地运行 |
| `startup-timings.md` | 启动/首屏渲染基线数值与口径 | 汇总 |
| `capture.cjs` / `routes.mjs` | 截图脚本与 75 路由清单 | — |

## 口径说明（185 界面 vs 75 条路由）

- 「185 个界面」源自 `.vue` 组件总数；应用路由为 75 条。逐屏比对以 **75 条路由 × 明暗两主题 = 150 张基准截图** 为视觉比对单元。
- 其余组件（弹层、向导、面板等非路由级界面）通过交互录屏覆盖其可见状态；`08-22-regression-release` 的 185 行比对记录按「旧路径 → 新路径映射表」把每个组件归属到其出现的截图或录屏条目，保证 185 项全覆盖、零未判定。

## 已知边界

- OAuth 向导录制止于凭据录入步骤；完成真实授权需要真实凭据，超出本地自动化范围。回归阶段（子任务 15）如需完整授权链路验证，需人工提供测试账号。
- 日志流、图表数据在桌面运行时下为真实 IPC 数据；Web 模式截图中的 IPC 报错横幅为无 Tauri 运行时的预期表现，前后端对比时保持同一模式即可。
