# T08 技术设计

## 责任与权威来源

本任务只修改上述已证问题的 owner。各领域服务负责业务规则，CLI/TUI/Tauri/React 负责适配和呈现；生成客户端、权限和协议以 registry 为源。

## 机制与逐项追溯

| 需求 / 验收 | 机制与数据流 | 可区分旧缺陷的验证 |
| --- | --- | --- |
| R1 / AC1 | SettingsConfig 保留原始 typed snapshot，按 dirtyKeys 构造 patch；不把未表示的 union 压成 boolean。提供明确 unset 操作，区别未修改、清空和删除；无损 mapper 单独测试。 | 以真实 settings mapper 和 mock transport 捕获请求，后端 fixture 应用 patch 后比对非目标路径；加入 unknown enum/字段 round-trip。 |
| R2 / AC2 | 将 load 结果扩展为 values、managedLocks、layer/token metadata；BaseSettings 消费平台 descriptor 提供的能力，FieldControl 支持 disabled/reason 和 current unknown option，禁止平台名分支。 | 渲染完整 BaseSettings 并触发用户交互，断言 DOM disabled、说明文案、请求 payload；后端拒绝仍有错误反馈。 |
| R3 / AC3 | descriptor 提供 raw callbacks、content token、layer notices 和 environment policy，复用 ConfigSourcePanel；不新增各平台专用重复页面，不更改市场终端视觉方向。 ConfigSourcePanel 的领域中性 composite 移入现有明确共享层后供 Base 复用，避免 features/platform 直接跨域依赖 features/editor，不扩大全域 import 豁免。Grok 保留无备份、policy layer 和 Profiles/off 恢复说明。原始文本仅停留编辑会话，不入全局 store/localStorage/log。 | 路由级 smoke 覆盖入口可见、保存/冲突、local/WSL/SSH 禁用；后续 web 交互与原生 smoke 分别记录，不以截图替代行为测试。 Grok fixture 对 typed/raw 保存和 invalid/stale 等失败路径比较目标与备份目录清单，断言无新增备份，并断言 no-backup notice 可见。 |

## 依赖和变更顺序

无子任务前置；仍须用户批准规划后才可实施。

允许并行研究和编写独立 fixture；同一文件的实现由当前 owner 串行合入。handler_registry、generated IPC、Settings metadata 等共享文件不能由子任务各自覆盖。

## 兼容与取舍

- 使用现有 application/service、guarded writer、ProcessGateway、Query 和 event bridge，先修行为再按责任拆分。
- 操作的固定结果使用具名类型；用户自由配置保留开放 JSON。仅迁移受影响协议，不批量修改 348 个命令。
- 不用延迟、吞错误、无条件重试、伪造默认值或外层 Promise timeout 掩盖不一致。
- 原有配置/认证秘密不得进入日志、DTO、测试输出或全局持久前端状态。

## 回滚边界

mapper、descriptor 与组件同批回退代码，保持原 wire 格式；不以旧表单数据覆盖用户新配置。禁止为 Grok 等无备份域自动保存前镜像或复制敏感原文到发布、研究或历史产物；其他配置仅遵守所属领域已经批准的备份策略，不新增备份动作。

## 未验证边界

本轮只完成审查、隔离前端反例和规划。Rust 故障注入、多进程、OS 权限/清理、原生桌面、真实网络与视觉行为均不能预先标记通过。实施阶段必须执行对应验收；环境阻碍单独报告。
