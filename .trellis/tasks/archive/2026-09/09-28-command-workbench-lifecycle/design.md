# T07 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 使用现有 shell event bridge 与 stream store；发起后保存 job ID，页面挂载按 ID 调现有 status API 对账。事件带 seq 的增量幂等合并；不新增第二个页面私有任务真相源。 | MemoryRouter mount/unmount/remount 行为测试，运行中回页与离页期间完成两类场景。 |
| R2 / AC2 | 以 job ID 和终态版本去重，由同一 owner 记历史；snapshot 是后端权威，事件为增量加速。后端过期返回明确已过期，不伪造运行。 start 响应、cancel 响应和事件统一走 reducer，terminal 为同 job 的单调终点；提交 pending 独立于 job snapshot。 | 保留重复事件、序号间隙、snapshot/事件倒序及过期 job 测试，并增加三个受控时序：①延迟 start Promise，先发送同 job 的终态，再 resolve 旧 queued snapshot，终态不回退；②新 job 生效后发送旧 job 的 event 和 start response，新 job 不变；③start Promise 未 resolve 时连续点击两次，仅调用一次 start。每项均断言界面状态、history 写入次数和启动/取消控件可用性，旧终态不重复记录。 |
| R3 / AC3 | 复用 eventBridge 已有 disposed guard，页面不再独立订阅全局 job stream；需要局部监听时采用相同生命周期 helper。 | deferred Promise 控制 listen 完成顺序，断言订阅/反订阅次数和卸载后无 state update。 |

## 依赖和变更顺序

T11 / 09-28-desktop-control-oauth-lifecycle

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

只替换命令页状态 owner，保留后端 command job API 和持久 history schema；store 和页面同批回退。进程重启后的任务恢复不在本任务范围。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
