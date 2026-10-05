# T02 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 复用 ccr-cli::application 建立 prepare → execute → outcome 用例，平台 operation lock 覆盖整个复合流程。off 的必要清理规则纳入同一计划，禁止 TUI 先提交独立 off。跨文件失败以版本保护回滚；不能安全回滚时返回 partial/recovery 信息，禁止覆盖外部新版本。 | 在 runtime、profiles、registry 每个写点注入故障；比对旧状态字节、版本和指针；加入并发外部改动使 rollback 必须拒绝覆盖的测试。 |
| R2 / AC2 | 保留现有 ConfigService 禁用项不能激活的规则。runtime/指针确认后才提交成功副作用；用操作标识及可重放记录避免对已提交结果的重试重复计数。返回不含终端输出的结构化 outcome。 | 共用 contract harness 调 CLI application、TUI backend adapter、Tauri service adapter；成功、重复提交、计数写失败、history DB 不可用逐项断言。 |
| R3 / AC3 | 区分 unchanged、applied、applied_with_warning、recovery_required 等结果语义，最终命名与既有 error freeze 保持兼容；不新增 CcrError 公共变体作为捷径。适配器负责 CLI 文本/退出策略、TUI 消息和 IPC DTO。 | 结果序列化和三端呈现测试覆盖成功、附属失败、部分提交；secret sentinel 不进入日志和 DTO。 |
| R4 / AC4 | 先在现有 application 模块形成可测试接口，再迁移调用者；不在本任务新建全能 ccr-app crate。CLI command 只解析、调用、呈现，平台低层 apply 限定为内部机制。 | 依赖/调用边界 guard 加行为适配测试，保留 CLI help、公开路径和生成 IPC 漂移检查。 |
| R5 / AC5 | 将 Claude/Codex handler 中的 load/patch/save/delete/apply 编排迁入同一 application lifecycle；沿用 T01 mutation、操作锁和版本保护补偿；不对自由 JSON 内容强制封闭 schema。 | 三阶段 fault-injection 加 current/default、旧名/新名存在性和 unknown fields fixture。 |

## 依赖和变更顺序

T01 / 09-28-config-repository-consistency；T05 / 09-28-safe-persistence-backups

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

按一个平台的三端适配器为迁移批次，不能长期混用旧 off+apply 与新事务；以受保护前镜像和 operation record 恢复，禁止无条件覆盖用户的新文件。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
