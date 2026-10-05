# T04 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 共享诊断用例返回 typed report；终端层渲染。保留既有错误退出码映射，warning-only 为零，错误非零；不让可嵌入服务调用 process::exit。 | assert_cmd 在隔离 HOME/CCR_ROOT 执行实际 binary，断言 exit code 与分类，而非只匹配输出文字。 |
| R2 / AC2 | 调用平台 validator，拆开 Option 未配置与 Result 读取失败，删除 .ok()/None 吞错支路；应用入口保留 T02 enabled policy。 | 平台 × auth-mode × 6 类状态表驱动测试；读取错误不可进入未配置分支。 |
| R3 / AC3 | 使用 T01 纯查询。能力矩阵从共享平台类型派生；Gemini/Droid legacy writer 仅盘点和标记兼容风险，禁止新增调用者或扩张支持范围，不在无可达证据时删除公共 API。 | 文件清单/字节测试和 capability matrix guard；核对 help、docs 与生成入口，旧受支持行为不被静默移除。 |

## 依赖和变更顺序

T01 / 09-28-config-repository-consistency；T02 / 09-28-profile-application-usecases

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

共享 report 与 terminal adapter 同批回退；不回退 T01 的纯读取保护。公开退出码变化以 bugfix 记录，保持错误码数值的既有映射。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
