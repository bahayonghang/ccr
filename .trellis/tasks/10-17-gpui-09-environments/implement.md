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
