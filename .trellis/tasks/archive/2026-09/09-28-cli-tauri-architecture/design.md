# 父任务技术设计

## 关键决策

1. 共享应用用例拥有完整业务规则和提交结果。先收敛现有 ccr-cli::application 与领域 service，再按实际依赖需求判断拆包；不新建全能 facade。
2. Tauri handler 是 DTO/授权/事件适配器，不调用 CLI 终端 handler。通用工作台继续用受控 CCR sidecar，usage 同步继续用 llmusage CLI，usage SQL 继续由 ccr-usage 拥有。
3. 配置仓储锁覆盖读取到提交，operation lock 与 guarded leaf lock 顺序固定。查询纯读，修复与初始化显式执行。
4. 复合 profile 流程先 prepare，再执行，返回结构化 outcome。跨文件写使用版本保护补偿，部分提交明确显示恢复状态，不宣称 OS 级多文件原子性。
5. job/controller 唯一提交终态。取消先到控制 owner，清理成功后确认 Cancelled；失败/超时/cleanup_failed 保留原义。
6. Settings 原始 typed snapshot、dirty patch、锁/层/token、raw callbacks 由平台 descriptor 提供。Base 不按平台名分支；复用 editor 前修正共享层归属。
7. Query snapshot、编辑草稿、后台任务投影与页面局部偏好分开；路由 mount 不决定后台生命周期，迟到数据不能覆盖新会话或终态。

## 方案比较

| 方案 | 判断与证据 |
| --- | --- |
| Tauri 全部启动 CLI | 不采用。已有服务可复用，CLI presentation 有永久迁移错误和 process::exit；子进程边界仍用于外部程序拥有的能力。 |
| 一次拆出新业务大 crate | 延后。先证明共享用例和测试边界，避免仅移动 imports 而保留重复副作用。 |
| 各页面局部修补状态 | 不采用。A13/A16/A17 需要对齐 state owner 与结果语义；只增加延迟或默认值不能满足反例。 |
| 按现有 owner 分阶段重构 | 采用。保留 registry、ProcessGateway、guarded writer、Query、生成协议和现有平台能力。 |

## 需求到机制、验证的映射

| 父需求/验收 | 子任务机制权威 | 具体检查 |
| --- | --- | --- |
| R1/AC1 | ../09-28-config-repository-consistency/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：CLI-01、CLI-06、TA-02 |
| R2/AC2 | ../09-28-profile-application-usecases/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：CLI-02、CLI-03、CLI-04、CLI-07、TA-08 profile lifecycle |
| R3/AC3 | ../09-28-tauri-config-adapter/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：TA-01、TA-02、CLI-07 |
| R4/AC4 | ../09-28-cli-diagnostics-contract/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：CLI-05、CLI-06、CLI-09、CLI-10 |
| R5/AC5 | ../09-28-safe-persistence-backups/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：CLI-08、TA-06、TA-07 storage |
| R6/AC6 | ../09-28-usage-job-lifecycle/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：TA-03、TA-04 |
| R7/AC7 | ../09-28-command-workbench-lifecycle/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：F01、F02、F03 |
| R8/AC8 | ../09-28-settings-lossless-capabilities/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：F04、F05 |
| R9/AC9 | ../09-28-frontend-query-error-contracts/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：F06、F07、F08 |
| R10/AC10 | ../09-28-architecture-contract-gates/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：G-01、G-02、G-03、CLI-10 |
| R11/AC11 | ../09-28-desktop-control-oauth-lifecycle/design.md 的逐项追溯表 | 对应 prd 的每个 AC、implement 的行为测试与命令；问题映射：TA-05、TA-07、TA-08 typed outcome |

## 显式依赖

| 子任务 | 必要前置 |
| --- | --- |
| T01 / 09-28-config-repository-consistency | 无 |
| T02 / 09-28-profile-application-usecases | T01、T05 |
| T03 / 09-28-tauri-config-adapter | T01、T02 |
| T04 / 09-28-cli-diagnostics-contract | T01、T02 |
| T05 / 09-28-safe-persistence-backups | 无 |
| T06 / 09-28-usage-job-lifecycle | 无 |
| T07 / 09-28-command-workbench-lifecycle | T11 |
| T08 / 09-28-settings-lossless-capabilities | 无 |
| T09 / 09-28-frontend-query-error-contracts | T03、T08 |
| T10 / 09-28-architecture-contract-gates | T01、T02、T03、T04、T05、T06、T07、T08、T09、T11 |
| T11 / 09-28-desktop-control-oauth-lifecycle | T05、T06 |

第一阶段可分别推进 T01/T05/T06/T08；T01+T05 完成后进入 T02；T02 后进入 T03/T04；T05+T06 后进入 T11，继而 T07；T03+T08 后进入 T09；T10 最后。同文件实现不能并行覆盖。

## 跨入口验证矩阵

| 操作 | 入口 | 必须比较 |
| --- | --- | --- |
| profile apply/off/rename | CLI application、TUI adapter、Tauri service | runtime、profiles、registry、enabled、计数、history、结果与错误 |
| config CRUD/读取 | CLI/service、Tauri、React Configs | 两进程 RMW、只读前后文件、显式平台、非法 patch 无写入 |
| 诊断 | validate binary、共享 validator | auth-mode、unconfigured/invalid/unreadable、退出码、无隐式修复 |
| 后台任务 | job service、stream executor、React route | admission、start/terminal 顺序、status 恢复、cancel、deadline、cleanup |
| Settings | typed read、表单编辑、patch、reread | 通知 union、未知字段、托管锁、CAS、环境、dirty 草稿 |
| OAuth | controller/storage/listener、Tauri adapter | secret 权限、bind、silent socket、cancel、HTTP deadline、唯一终态 |
| 环境 | Local/WSL/SSH 按已支持能力 | 不支持操作明确拒绝，禁止向其他环境隐式回退 |

## 兼容与回滚

按单一领域完整调用链为提交批次；生成协议、registry 和消费者成组更新。旧公开命令保留明确兼容 adapter；无法确定平台时拒绝，不恢复隐式全局切换。自由配置保留 OpenJson；仅固定操作结果迁入具名类型。

持久化不迁移用户配置 schema。备份命名变化兼容旧文件，禁止删除已有前镜像。跨文件回滚必须检查版本，不覆盖其他进程新写入。活动进程/登录需结束或受控交接后再替换实现。

## 风险和明确延后

公开 CcrError 变体、String-error command macro、旧配置和备份发现规则是兼容敏感面。规划不授权修改真实凭据、扩大环境支持或删除 legacy adapter。全量性能、原生视觉和外部账户行为没有本轮证据；详见审查 D01-D05。
