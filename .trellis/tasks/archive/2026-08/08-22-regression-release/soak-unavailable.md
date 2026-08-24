# 2 小时长时间运行：已测量，未通过（AC13）

> 任务：`08-22-regression-release`。对应 R10 / AC13。完整记录见 `soak-results.md`。

## 结论

已在打包 `ccr-desktop.exe` 上连续运行 7203s，切换 29 条路由。AC13 不勾选。

| 项 | 结果 |
| --- | --- |
| 主机 WorkingSet 第2小时/第1小时 | 0.948（≤1.10） |
| JS 堆第2小时/第1小时 | 3.327（>1.10；第2小时仅 11 个 CDP 样本） |
| JSEventListeners 第2小时/第1小时 | 4.662（>1.10；含 `/grok/settings` 48066） |
| 第2小时 3602s 之后 | 35 次 CDP tick 超时 25s，主机 WorkingSet 仍在采样 |

命令：`pwsh -NoProfile -File soak-run.ps1`。退出码 1。`SOAK_PASS=False`。
