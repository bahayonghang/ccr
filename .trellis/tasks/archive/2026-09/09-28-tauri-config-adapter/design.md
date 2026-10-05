# T03 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | typed domain 请求携带明确平台，handler 只校验/映射到 T02 用例。启用持久策略与激活动作分别定义，由用例完成必要组合，避免 enableConfig 仅重命名 switch 调用。 | React 用户动作测试加真实 Rust handler/service fixture；禁止仅 mock switch 为成功来证明链路修复。 |
| R2 / AC2 | 移除 handler 自行持 config 锁的 RMW，采用共享 resource mutation；patch DTO 区分未提供与清空，禁止将非字符串默默转为 None。 | handler 参数反序列化、领域校验及持久化结果联合测试，复用 T01 并发 harness。 |
| R3 / AC3 | 在 registry 定义新输入并同步 generated client、domain wrapper 和调用者；命令 ID 保留兼容 adapter，禁止恢复旧全局隐式 switch。只迁移受影响命令，不扩张为全部 71 个 legacy 命令重写。 | 旧/新 payload contract、辅助窗口 ACL、确认元数据、manifest count 和 bindings drift 测试。 |

## 依赖和变更顺序

T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

handler、registry、生成物和消费者同批回退；旧无平台调用继续明确拒绝，禁止回退到任意平台猜测。T01/T02 的领域修复独立保留。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
