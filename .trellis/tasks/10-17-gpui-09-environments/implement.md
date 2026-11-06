# GPUI 09：执行环境与桌面集成：执行计划

## Preconditions

04、05、06、07、08 完成。 用户须审阅父任务最新方案。若批准后发现需要改变范围/兼容行为，重新评审变化部分。

## Ordered Work

1. 读取 curated specs、父任务设计和当前代码；记录修改边界及基础 fixture 状态。
2. 接入现有 Local/WSL/SSH 环境，保持 capability、路径解析、SSH trust 与 SFTP 行为。
3. 迁移托盘、隐藏/恢复、退出确认、窗口状态、系统打开与通知等现有 shell 能力。 区分 OS 支持与环境能力，完善用户数据与新 UI 偏好隔离、并存及关闭清理。
4. 先验证 service/adapter 的行为，再验证 native UI；失败/取消/冲突测试使用隔离 fixture。
5. 更新父任务功能台账，写 evidence.md 的实际命令、结果、截图、未运行项。
6. 通过本任务验收再交由 check 角色独立核对，遵守项目角色写入权限。

## Validation

- SSH/WSL path/trust/transport fixture 与 Tauri 回归
- just desktop-test；just desktop-test-ui；native process cleanup checks
- 三平台窗口/托盘/恢复/关闭/权限检查；Windows WSL 实机、受控 SSH 实机
- just secret-write-check；受影响 shared crates gates

上列新 desktop 命令以 01 的交付为前提，perf/package 以 10 的交付为前提。若命令或 fixture 尚不存在，先完成相应实现，不写“检查通过”。Rust 行为测试保持默认并行并 skip export_bindings。

## Completion

逐项对应 prd.md 验收，保存失败修复后的最终结果；按用户 2026-11-05 的续作授权，独立 check 后本地提交当前任务产品改动与规划产物，不发布。不得 archive 未完成工作。父任务最终验收负责全量整合。

## 2026-11-05 非 WSL 续作边界

- 具体缺口：`boundary_before_connect(Again)` 在 `process_trusted=true` 且当前没有任何指纹时返回 `Ok`。旧进程信任不能替代当前 host key。
- 最小修改：共享决策拒绝该组合；共享夹具覆盖缺失、空白、单边指纹，GPUI 环境消费路径覆盖远程状态下的同一拒绝。已有匹配 key + 进程信任路径保持。
- `Open` 的当前生产调用仅来自 Local 设置刷新空 facts，用于本地检查；其 `Ok` 不激活 SSH。没有新增连接、环境切换器、托盘或窗口栈，不改变 Tauri 命令或载荷。
- 先复现失败，再跑共享环境夹具及根 `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`。跨模块改动由 check 运行 `just ci`，Git Bash 的目录置于 PATH 最前；不得运行任何实机 WSL 探针。
- 9.1–9.6 保持开放；本轮新增证据不能代替未运行的实机验收。

## 2026-11-05 退出生命周期验证切片

- 在 `ccr-desktop/tests/runtime_shutdown.rs` 复用现有 `RuntimeOwner`、`CommandJobs::run_fixture_blocking` 和原生二进制闭合 `--native-command-fixture`，不新增产品 API 或窗口栈。
- Windows 合成父进程启动合成后代；收到后代实际输出并持有两个存活进程句柄后，再 detach 视图订阅并关闭运行时。验证 retained command 为 Cancelled、任务数归零、拒绝新工作和两个句柄 signaled；从 job 入队前到最终确认总计必须小于 5 秒，排除合成后代自然运行 10 秒后退出的假通过；失败路径也清理夹具。
- 用已进入的阻塞读和显式释放通道模拟超过 GPUI 200ms quit observer 预算的工作。丢弃超时 observer 后，最终 owner join 必须等待释放，之后保留 Cancelled 终态与零任务。
- 证据只覆盖 Windows OS 合成进程与共享运行时，不覆盖真实 native window close、watcher、托盘、其他 OS 或远程执行。9.4 整项仍开放。
- 先运行窄集成测试，再运行根 `just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`；默认并行和 `--skip export_bindings` 保持。实现与独立 check 收据分别保存。
- 独立 check 期间用户明确改为当前工作区的新 Codex 默认 `File` 标准，不使用新 worktree 验证。GPUI 原 `auto` 夹具显式写 `auto`，另补缺 config/缺 store 设置且无 auth 文件时 `NotDetected` 的消费回归；不回退共享标准、不修改生产映射。本消费测试依赖另一任务的共享默认值改动，不能称在旧基线独立通过。

## 2026-11-05 主窗口关闭确认切片

- `Shell::new` 安装既有 GPUI close callback，复用 Kit AlertDialog 栈。每次关闭主窗口均先确认，包括闲置时；尚未实现可配置退出偏好或跨页面精确 dirty 聚合。
- 默认取消；Escape、取消按钮与其 Enter 保留草稿、原域确认、焦点和运行任务。重复关闭不叠框，首帧 root Enter 安全取消；旧 opening 的延迟回调不能重抢焦点。明确激活退出后移除窗口，继续既有 quit observer 与事件循环外最终 owner join。
- 先失败复现，再运行 rendered/headless 窄回归与根三门禁；具体结果见 [关闭确认收据](research/close-confirmation-20261105.md)。主会话同步 shared-desktop-services 规范。
- 9.4 仍开放：本轮不是原生 OS 用户关闭实机收据；其他 9.1–9.6、WSL/SSH、托盘、其他平台、glass 与延后的尺寸/DPI 状态保持。
