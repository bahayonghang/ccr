# T06 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 建立同一 registry record 持有 snapshot、token 和执行所有权，admission 原子化；runner 在 spawn 前检查 token；cancel 仅切换 cancel-requested 并发信号。 | 受控 admission/start barrier 和 fake spawner，覆盖重复 start、早期 cancel、running cancel。 |
| R2 / AC2 | 在 lifecycle service 内封闭状态转换；runner 以 typed execution result 唯一提交终态，保留 cancelled/timed_out/cleanup_failed 分类。事件只投影已提交状态，前端同步完整枚举。 | 状态转移表、乱序事件、重复终态和 cleanup failure 测试，核验 active 释放时点。 |
| R3 / AC3 | 在现有 ProcessGateway/stream owner 中统一 deadline/cancel/terminate/reap 和 bounded reader joins；测试可注入毫秒期限，生产保留现有一小时策略；清理失败优先于伪造 cancelled。 | 可控子进程 fixture 配合单调时钟，允许调度容差并记录期限；Windows/Linux/macOS tree cleanup matrix，不用前端 Promise timeout 代替。 |
| R4 / AC4 | 复用现有 bounded readers、ManagedProcess 和 DTO mapper，仅修生命周期；不链接上游 llmusage Rust crate、不重建 usage parser。 | 现有 bounded stream 测试与 SQL ownership guard，加后端和 React 终态消费测试。 |

## 依赖和变更顺序

无子任务前置；仍须用户批准规划后才可实施。

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

lifecycle result、状态 DTO 和前端映射同批回退；升级/回退不在有活动子进程时替换 registry。保留终态诊断，禁止将 cleanup_failed 降级为成功取消。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
