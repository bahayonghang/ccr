# T01 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 在 ccr-config 提供按规范化平台配置路径定位的 mutation 入口，锁在读取之前获取；接收变更闭包或有类型 patch，不接受先前读出的整份 sections。规定 operation lock → resource lock → guarded leaf lock 顺序，避免重复获取非重入锁。 | 双进程 barrier fixture 覆盖 platform/service 与 desktop/service 组合，重复运行并断言磁盘最终值；不得靠测试串行化通过。 |
| R2 / AC2 | 分离 open/read 与 ensure_initialized/reconcile。保留现有各平台 current 优先级，用纯 resolver 返回值、冲突诊断和 repair suggestion；修改只在显式 reconcile 用例中执行。未指定平台的旧 adapter 不可猜测首个 enabled 平台。 | 空目录、缺文件、损坏文件、只读目录和不同 registry 顺序 fixture；比较前后文件路径、字节和修改时间。 |
| R3 / AC3 | 领域 patch 显式区分保留、设置、删除；在锁内检查存在性、重名、字段类型及平台 auth-mode 校验。仍使用既有 guarded secret writer，不改变文件格式。 | CRUD table tests、CAS stale token、unknown field 和 secret sentinel；回归原 current/default 行为。 |

## 依赖和变更顺序

无子任务前置；仍须用户批准规划后才可实施。

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

不迁移磁盘 schema；每个调用者迁移前后使用同一 fixture 对比。回退统一 API 和调用者为同一批次，保留旧文件读取与备份恢复能力。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
