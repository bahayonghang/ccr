# 2 小时长时间运行：未执行（AC13）

> 任务：`08-22-regression-release`。对应 R10 / AC13 / implement.md 步骤 6。

## 结论

未执行。不标为通过。AC13 保持 `[ ]`。

## 当前条件（2026-08-24 发布门补测后）

打包产物已存在：

- `ccr-ui/src-tauri/target/release/ccr-desktop.exe`
- MSI / NSIS 安装包
- 主界面可启动（scratch `tauri-launch-primary.png`）

`defects.md` D1 已修复。`just frontend-check` 与 `just ci` 退出码 0。不再以「无产物」为理由。

## 未采集的数据

| 项 | 状态 |
| --- | --- |
| 连续运行 2 小时 | 未执行 |
| 切换 ≥20 个界面 | 未执行（路由切换性能脚本 29 条 ×5 不是本项 2 小时浸泡） |
| 内存采样间隔 | 未按 design.md §5 做小时均值 |
| 第 2 小时均值 / 第 1 小时均值 | 无数据 |
| 事件监听器数量 | 未执行 |

本项需要 2 小时墙钟。本轮未启动该浸泡。

相关但范围不同的已有证据：`event-bridge-leak.smoke.test.tsx`；日志流 5 分钟 ×3（`perf-react-after.md` 场景 3）。
