# T05 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 备份名保留既有时间/前缀并增加唯一后缀；通过 create-new 语义保证不覆盖。排序采用稳定 tie-break；复合 off 备份目录采用同样操作身份。 | 注入固定时钟与不同内容；碰撞重试、备份失败、轮换和同秒多进程测试。 |
| R2 / AC2 | 复用统一 backup parser/selector 或现有兼容规则；明确新的唯一后缀 contract，不通过批量重命名用户备份迁移。 | 旧命名 fixture、恢复后内容、Windows DACL 与 Unix mode 的平台测试。 |
| R3 / AC3 | 将普通 AtomicWriter 后 chmod/icacls 的顺序迁入现有 secret/guarded writer；权限在临时文件发布前建立。CLI/Tauri 复用 pending 存储/清理 owner，桌面仅负责 listener 和 UI 事件。 OAuth pending 显式使用 secret:true 和 BackupPolicy::None，禁止备份 code_verifier/state，旧权限失败不得吞没。 | 以合成 verifier/state sentinel 分别覆盖创建、替换、取消、过期清理、权限拒绝；比较目标目录和配置的备份目录前后清单并扫描所有非当前目标文件，断言无新增凭据副本。fake permission adapter 注入拒绝，断言首次写入无新目标、替换失败旧字节不变；原生 Windows/Unix 分别验 ACL，不以源码检查替代。 |
| R4 / AC4 | 保持 Secret 类型与 redacted DTO，清理错误只携带分类和非敏感路径；扩展相关持久化守卫覆盖实际 owner。 | 捕获 tracing、IPC 和 CLI 错误进行 sentinel 断言；不使用真实账户数据。 |

## 依赖和变更顺序

无子任务前置；仍须用户批准规划后才可实施。

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

新备份保留旧前缀和可识别后缀，回退前验证旧 reader 的恢复路径；任何回退均保留已生成备份。OAuth writer 与调用者同批发布，禁止恢复权限错误吞没。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
