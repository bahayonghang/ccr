# T11 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | registry 按 control-command-matrix.md 的真实命令 ID 显式区分风险、资源和操作类别；control 不继承执行任务的全局或模块长 permit，直接交给按 job/login/attempt ID 校验的 owner，以短临界区或消息控制。start 的 admission 在发布 ID 前取得并转移给真实 job owner，持到完成与清理；foreground permit 继续绑定实际完成。保留 completion-aware future、ACL/confirmation 与现有幂等规则，不把所有命令改为 Parallel。 | 执行矩阵 C01-C04，分别阻塞 command/install 的共享执行路径、usage 同模块同步导入、OAuth 同模块 token exchange，逐个控制 ID 验证在 barrier 释放前到达 owner；get/status/recent/port probe 逐个保留行为测试。C05 对四类后台 start 验证 handler 返回后 admission 仍占用、第二 start 无并行、终态与清理后释放；C06 验证辅助窗口 ACL、用户确认、错误 ID 和外部进程权限。用 owner acknowledgement 与虚拟时钟/有界看门狗断言，不靠 sleep 或只检查 manifest。 |
| R2 / AC2 | 将 probe-close-rebind 改为持有 listener，先验证/保存再发布内存状态；service 注入 clock/storage/http/listener，Tauri 仅做 DTO/events。使用 T05 secret pending writer 和既有端点白名单。 | fake listener/storage 及本地 loopback 隔离 fixture；覆盖 bind race、磁盘失败、state mismatch 和过期恢复。 |
| R3 / AC3 | 同一 login controller 持 cancel handle，socket read、request/body 共享有界执行机制；取消等待受控清理完成后终态；typed内部结果区分取消、超时和失败，映射现有 IPC，不全量改写 String error macro。 | 可控 socket/server 与时钟 fixture，不调用真实 OAuth；后续原生 smoke 单独证明 OS 资源释放。 |
| R4 / AC4 | 只为变动的 profile/usage/auth操作使用 named result DTO；自由配置保留 OpenJson，旧未知消费者采用兼容 envelope，公共 CcrError freeze 保留。 | manifest/bindings、旧payload、ACL 与 sentinel tests。 |

## 逐命令控制矩阵

R1/AC1 的强制逐 ID 策略、N/A 边界及 C01-C06 行为验收：`.trellis/tasks/09-28-cli-tauri-architecture/research/control-command-matrix.md`。该矩阵是本设计的一部分。

## 依赖和变更顺序

T05 / 09-28-safe-persistence-backups；T06 / 09-28-usage-job-lifecycle

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

registry override 与 job resource admission 同批回退，禁止只撤销 admission；OAuth storage与controller使用同一兼容pending格式，活动登录先结束再切换实现。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
