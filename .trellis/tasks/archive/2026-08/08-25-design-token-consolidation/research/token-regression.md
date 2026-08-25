# 令牌层视觉回归（AC11 / 阶段 E3）

时间：2026-08-25。Web 预览：`ccr-ui` 下 `bun run dev:web -- --host 127.0.0.1 --strictPort`，`http://127.0.0.1:5173/`。
方法：Playwright 对 8 页 × 4 组合读取计算令牌与横向溢出；Cursor 浏览器 MCP 当时不可用。度量写入 `walkthrough-artifacts/metrics.json`。

## 组合结果

| 组合 | `data-theme` / `data-flavor` | `--color-border-default` | `--color-bg-elevated` | `--radius-sm` | 横向溢出 |
|---|---|---|---|---|---|
| light × neutral | light / neutral | `#c9cacd` | `#f2f3f5` | 6px | 0 |
| light × clay | light / clay | `#d4cbbe` | `#f5eee1` | 6px | 0 |
| dark × neutral | dark / neutral | `#48494e` | `#1a1b1f` | 6px | 0 |
| dark × clay | dark / clay | `#3a302a` | `#221b18` | 6px | 0 |

八页（Dashboard `/`、Profiles `/configs`、MCP `/mcp-manager`、Commands `/commands`、Sync `/sync`、Check-ins `/checkin`、Usage `/usage`、Settings `/settings`）在同一组合下令牌一致。`--radius-lg` 8px、`--radius-2xl` 12px、`--radius-full` 9999px 全组合相同。`--color-platform-opencode` 均为 `#735f52`。

## 判定

- 深底（dark × neutral / dark × clay）：实色边框相对 surface/elevated 可辨；clay 暗 shell `#221b18` 与卡片 surface 可分层。
- 浅底：边框可见、不刺眼；clay 亮边框偏暖，不串成中性灰。
- 圆角同屏一致。原 4px chip → 6px、原 16px 容器 → 12px 为预期收敛，不判缺陷。
- 自定义强调色抽查（dark × clay Settings）：`--color-accent-primary` 从 `#e8835b` 变为注入的 `#3b82f6`（选项 A，accent tint 随 `-rgb` 走）。

未发现需进入本任务 3 轮视觉返工的令牌层缺陷。feature CSS 里 `rgb(var(--color-border-*-rgb) / α%)` 二次叠色会变淡，属 B5 已记录的调用点债务，不改选择器结构。
