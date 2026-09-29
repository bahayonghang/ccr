# T09 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | 在现有 BaseAuth 实现明确 view model；先处理 probe error，再处理 load error，旧成功数据标记 stale。off 使用受控 mutation pending 与 catch 反馈；不强制迁移 Claude/Codex 特殊认证页。 | 真实 BaseAuth 配合 mock domain，覆盖初次两类失败、旧缓存刷新失败、重试、unsupported、off 拒绝和重复点击。 |
| R2 / AC2 | 依赖 T08 typed snapshot，引入编辑基线版本与 draft dirty set；query key 含环境身份。dirty 时收到新服务端版本显示冲突/刷新提示。环境变化冻结旧会话写入并保留其草稿，提供返回原环境或明确放弃后重载的动作；敏感文本不进入持久 storage。 | 控制 Query promises 先后顺序，断言输入值、baseline、禁用提交和确认行为；保存失败保留草稿，成功重建基线。 |
| R3 / AC3 | 使用既有 useAppT/useResolvedT 并让 memo 依赖正确的翻译身份；只修本次配置表面，不清理全仓文案或重做视觉。 | 同一页面实例语言切换行为测试；禁止通过 unmount/remount 满足断言，保留 layering/cycles checks。 |

## 依赖和变更顺序

T03 / 09-28-tauri-config-adapter；T08 / 09-28-settings-lossless-capabilities

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

按 domain hook 与其消费者成组回退，query key 变更需清除本次作用域旧缓存。首页 Insights 的既有任务优先协调，禁止同时修改同一 hook/事件矩阵。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。


## 2026-09-28 R2 implementation clarification

Independent source review found that Claude typed settings reads the active ExecutionEnvironment repeatedly during one asynchronous update. Frontend identity checks cannot prevent the check-to-IPC race. The approved R2 requires a backend environment boundary for the affected read/update. This is necessary completion of R2, not a new platform.

- `claude_get_settings` and `claude_update_settings` accept optional `expected_environment_id`. The Settings page always sends the acknowledged session id. A mismatch at entry rejects before file access. A single captured environment Arc owns the rest of the read or update. Existing callers without the argument retain entry-time environment selection. Local updates retain the current atomic SettingsManager owner.
- Capture once in the shared Claude update helper so an environment switch during read cannot redirect the later write. Test the mismatch and the switch-during-read with two controlled environments and a barrier.
- Codex/OpenCode typed Settings use fixed local paths in the current backend. Their descriptors must expose existing local-only capability and a clear unsupported state in remote environments; no remote support is added.
- Frontend owner: Settings context, descriptors, domain wrappers, editor/Query tests and locale. Backend owner: Claude commands/helper, scoped registry schemas/generated files, Rust regressions. Independent check covers both.
- Include actual IPC payload tests. Do not infer backend safety from mocked frontend success or from identity verification before and after an unbound write.
