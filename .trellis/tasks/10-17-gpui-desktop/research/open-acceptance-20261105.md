# GPUI 开放验收检查点

日期：2026-11-05（America/New_York）；父切片基线为干净 `dev@bcf355ebfd9c2d7cf16bd293e01f0593f8fea864`。本轮按 09 → 01 → 父任务推进，只剩这三项活跃。同日 [continuation-20261105.md](continuation-20261105.md) 为早期 08 历史记录，不覆写、不重新 capture inventory、不重开其他已归档任务。

## 已提交并独立检查的切片

- **09：`0bf9ba48938fb286a581f4e9eb51e07f3273c629`**。7 文件包含产品修复、回归、SSH 环境契约和当前任务规划/证据；重连不能仅凭旧进程信任放行缺失的当前 key。[实施证据](../../10-17-gpui-09-environments/evidence.md)、[独立检查](../../10-17-gpui-09-environments/research/check-20261105.md) 均已入历史。
- **01：`bcf355ebfd9c2d7cf16bd293e01f0593f8fea864`**。仅 5 份当前任务 Markdown，无产品或历史 baseline 改动；[源码续作收据](../../10-17-gpui-01-foundation/research/continuation-20261105.md)、[独立检查](../../10-17-gpui-01-foundation/research/check-continuation-20261105.md) 已入历史。

两次均为本地 `git commit -F`；未 push、amend 或归档。切片检查通过不等于整项任务完成。

## 验收边界

| 09 验收 | 已有证据 / 仍缺证据 |
| --- | --- |
| 9.1 | 路径规划 fixture 不回退本机；WSL 实机 read/write 与非 Windows 执行 `not_run` |
| 9.2 | 未知/新/变更 key、认证拒绝、取消、重连、未连接断开 fixture；真实 SSH 流程 `not_run` |
| 9.3 | cache/draft/job、本地专用读取与晚到结果 fixture；无真实切换器，真实切换 `not_run`，不标通过 |
| 9.4 | 只读取消 fixture；托盘与 Windows 隐藏/恢复 `unsupported`，无对应控件；退出 process/task/watcher 清理 `not_run` |
| 9.5 | Windows capability 说明；本域可见 Windows 窗口及 macOS/Linux 原生壳 `not_run`，通知投递未确认 |
| 9.6 | 13 入口与实色纸面回归；原生 light/dark glass/solid 截图及系统外观变化 `not_run` |

01 保留 Windows AC1.1–1.4/native 输入历史记录；host 和唯一 installed target 为 Windows，macOS 编译 `not_run`，AC1.5 不关闭。whole-app fixture 隔离、真实 first-present/action-ready、同负载 S0/S1/S2 仍缺；性能 `not_run`，相对指标 `not_comparable`。历史 WSL Linux 编译不转记独立 Linux 桌面通过；尺寸/DPI 继续 `user-deferred`。

09 独立 check 的根 desktop-fmt/lint/test、ccr-app test/clippy 全为 0；完整 `just ci` **退出 0**（16 阶段，**14:13.540**），日志 `outputs/desktop/gpui09-check-20261105/just-ci.log`。该进程前置 `C:\Program Files\Git\bin`，`bash-resolution.txt` 确认为 Git Bash；默认并行与 `--skip export_bindings` 保持，完整 CI 未重试或弱化门禁。01 与本父切片只有文档，未重复产品门禁；其后没有新产品改动，不把前序收据冒称本父切片新执行。

本轮没有运行 WSL/doctor、启动应用、开 SSH 套接字、keyscan 或读取密码/身份文件；没有新窗口栈、Tauri 签名/载荷、主题或产品文案改动。现有 SSH 契约已随 09 入历史，本父同步无新 spec 知识，不额外改规范。

## 父归档硬门

| 必须同时满足的条件 | 当前值 |
| --- | --- |
| 01 独立验收并归档 | false；仍 `in_progress` |
| 09 独立验收并归档 | false；9.1–9.6 开放，仍 `in_progress` |
| 发布门 `just ci` 退出 0 | 最新切片收据为 true；未来产品改动仍需对应门禁 |
| AND 结果 / 允许父归档 | **false** |

父任务保持 `in_progress`、`completedAt=null`。仅三项条件全部满足后才运行 `python ./.trellis/scripts/task.py archive .trellis/tasks/10-17-gpui-desktop`；本轮不运行 archive/finish。产品及规划工作先入历史，归档另成纯归档提交；没有新改动不做空提交。

本父 implement 检查：JSON/本地链接/提交存在性、三项状态/`completedAt`、范围与尾随空白断言、实际子任务路径和 CI 日志计算的归档 AND、`task.py validate`、`git diff --check` 均退出 0。既有依赖治理大文件注入截断警告保留；独立 check 结果另记。
