# T10 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 复用 just tauri-ci 并与 root aggregate 对齐，分清只读 check 与 format/regenerate。保留 hosted required lane 和跨平台 tree-cleanup matrix；不降低现有 coverage 阈值。 | recipe/workflow contract tests、故障 fixture 与最终 just ci/just tauri-ci 组合；仅在授权实施阶段、隔离工作树执行有改写的步骤。 |
| R2 / AC2 | 复用 T01-T09 及 T11 fixture，父任务维护 operation × platform × entry 的验证矩阵；行为测试与静态边界检查互补。Rust/原生未执行必须保持未通过，禁止将检查子集改称完整 gate。 | 审阅实际测试断言并运行聚合 contract suite；每个标记通过的目标至少执行一个匹配测试。 |
| R3 / AC3 | 以 registry/manifest 和 code map 为证据更新受影响规范，不批量替换历史案例；每个子任务随实现同步自己的契约，T10 只完成跨域收敛。 | 文档路径检查、inventory/bindings drift、治理脚本，人工核对历史示例与当前契约的区分。 |
| R4 / AC4 | 最终在 clean worktree 或明确保留用户未提交内容的工作区运行正式 gate；不新增忽略规则绕过真实失败、不移动用户文件。归档只在各任务真实完成且符合用户授权后进行。 | 基线/最终 git status 对照、门禁原始日志、任务状态和 requirement coverage 检查。 |

## 依赖和变更顺序

T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases；T03 / 09-28-tauri-config-adapter；T04 / 09-28-cli-diagnostics-contract；T05 / 09-28-safe-persistence-backups；T06 / 09-28-usage-job-lifecycle；T07 / 09-28-command-workbench-lifecycle；T08 / 09-28-settings-lossless-capabilities；T09 / 09-28-frontend-query-error-contracts；T11 / 09-28-desktop-control-oauth-lifecycle

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

门禁变化先以 fixture 证明仍 fail-closed；回退只撤销本任务规则，不降低既有检查。报告保留失败和未验证记录，不以修改基线定义完成验收。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
