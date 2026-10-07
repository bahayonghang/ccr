# 实施状态

2026-10-06 用户授权：「开始按顺序实施trellis父任务与子任务」。该授权覆盖现有父子任务的规划补齐、启动、产品实施、检查与相关 spec/证据写回。未新增任务。

基线：分支 `dev`，HEAD `c524ac07`。实施前已有 P2 的 `prd.md`、`implement.jsonl`、`check.jsonl` 修改，以及未跟踪 `design.md`、`implement.md`；继续沿用这些规划，不回退。

顺序：P2 → P1 → P3 → P4 → P5 → P6。用户已确认 P1 完整身份缺失跳过同步、P3 保留全部旧备份且不清理、P6 拒绝身份冲突输入并保留现有加密导出。P5 的产品迁移不在评估授权内。用户要求对照 cockpit-tools 深入分析账号生命周期；k12 本人登录稍后处理。

2026-10-07 用户授权拆分提交与归档。推送、PR、发布与安装仍未授权。

## 当前里程碑

| 子任务 | 状态 | 证据/前置 |
| --- | --- | --- |
| P2 | VERIFIED_LOCAL_PENDING_DELIVERY | 独立 AC1–AC7 与最终父任务 CI PASS；未知字段保留、版本门和 TUI 只读提示回归通过 |
| P1 | VERIFIED_LOCAL_PENDING_DELIVERY | 完整身份隔离、手动查询与凭据生命周期独立检查及最终 CI PASS；真实 k12 恢复仍为 UNVERIFIED |
| P3 | VERIFIED_LOCAL_PENDING_DELIVERY | 备份去重、防覆盖、删除/强制重命名前置备份独立检查及最终 CI PASS；全部旧备份保留 |
| P4 | VERIFIED_LOCAL_PENDING_DELIVERY | Windows 权限、Unchanged、私有 rename 回退、Debug/HTTP 脱敏及 EN/ZH 尺寸矩阵独立检查与最终 CI PASS |
| P5 | ASSESSMENT_SCOPE_PASS | 独立评估检查 PASS；建议暂缓迁移，产品布局未改变；迁移决定与实施未授权 |
| P6 | VERIFIED_LOCAL_PENDING_DELIVERY | 整包身份预检、锁后版本校验、私有版本替换、CLI 错误传播独立检查及最终 CI PASS；现有加密导出保持 |

## 最终本地集成验收

最终产品源码冻结于 `2026-10-07 02:33:43 UTC`。`just ci` 的 16 个阶段全部 PASS，native 与 shell 退出码 0，用时 `22:07.899`。Rust workspace 41 个 suite 共 2038 passed、0 failed、16 ignored；Codex 381 passed、2 ignored；TUI 253 passed；Tauri 407 passed、1 ignored，另有 2 项依赖边界测试；前端 904 passed，覆盖率门通过；VS Code 51 passed。绑定生成与工作区基线一致。

验收状态写回前，P6 独立清单中的 71 项哈希全部匹配，其中包含 22 项产品源码。最终报告见 `research/integration-validation.md`，原始日志见 `checks/ci-final-first-attempt.log`，最终源码与验收文档身份见 `research/integration-hashes.json`。较早独立报告继续保留当时的源码与门槛边界。

各任务在本次授权的提交之后归档。真实 OAuth/账号恢复、安装后的运行、交互式 TUI、Unix 与 hosted CI 为 `NOT_RUN`。Windows `Foo`/`foo` 物理路径重叠、局部 serde 错误文本与 quota claims fallback 差异保留为已报告边界；未扩展实施范围。

## 旧账号诊断

k12 注册表与自有快照仍在；账号上下文、完整身份与脱敏邮箱一致。GET-only 额度请求返回401，当前khanh同路径200；未刷新或写入真实凭据。保存不保证远端长期接受token，具体401原因未查明。用户要求对照cockpit-tools继续完善并选择稍后登录；真实恢复为UNVERIFIED。证据见P1 `research/old-account-read-only-diagnosis.md`。
