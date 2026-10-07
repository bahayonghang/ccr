# CLI 提示一致性迁移与验收

## 目标

覆盖 Auth 之外的同类 CLI 消息及 Doctor，完成父 R1–R6、R8、R9 的剩余部分，并验证任务的全局展示一致性。

## 范围

父 research/scope-inventory.md 的非 Auth CLI 引用，以及 doctor_cmd 的独立状态展示、共享入口外 Codex/Sync 影响。覆盖 Platform、Profile、Data、Lifecycle、Project、Update、Provider、Sessions、Temp、Sync、UI 服务启动的状态/字段/建议。

已批准的范围包含消息展示和父 windows-fixture-isolation-scope.md 的最小路径/夹具修复；表格、交互输入、Clap 帮助目录、TUI、desktop 和 VS Code 不重设计。Doctor 的报告、JSON、诊断只读性及退出码保持原样。

## 验收标准

- [x] 所有调查引用有源码处置记录；活动旧标签扫描无未分类项。保留 API 字面量/历史内容有明确理由；最终独立复查见 checks/。
- [x] handler 使用字段、计数、说明、取消与成功的对应语义；过程步骤、警告与错误仍可识别；相关成功建议最多两项。终端可读性验收仍单独开放。
- [x] Doctor 使用统一标记，保持报告字段、stdout、JSON 和只读行为；Fail 与 Skip 不混为通过。
- [x] 活动 clean/doctor 文档及英文镜像与新显示一致，docs build/audit 已通过，历史归档不改。
- [x] 非 Auth binary 回归验证 JSON/数据、流、退出码与必要警告不变。
- [x] Windows 40/80/120 列、明暗背景、NO_COLOR、TERM=dumb 和分别重定向有实际证据；长字段/命令不截断。
- [x] 最终 just ci 有当前源码证据；未运行的 OS/hosted/终端项明确记录，不把编译当展示验收。

## 依赖与状态

用户于 2026-10-07 要求继续现有任务。C1 共享主体已可用，C3 迁移及独立局部修正已实施；独立源码/规范复核及最终 version/fmt/lint/type 安全门槛 PASS。已完成有界机器接口与 handler 终端验收；完整 just ci 16/16通过，任务保持 in_progress。提交、推送、发布与归档未授权。

历史首次暂停：2026-10-07 发现 Windows 路径隔离缺口：既有 logger 与 ConflictChecker 直接使用 `dirs::home_dir()`，Windows Known Folder 忽略测试 HOME/USERPROFILE。首轮 Doctor 为 `FAILED_FIXTURE_ISOLATION`；非 Doctor 4 项显示断言通过，但日志副作用隔离为 `UNVERIFIED`。已停止 CLI binary/native 复测，新任务测试在 Windows 明确 ignored；不将 ignored 当作 PASS，不修改业务路径或降低验收标准。完整边界见 [checks/verification.md](checks/verification.md)。

2026-10-07 更新：最小路径扩展已获批准并通过独立安全门槛，17+5 个临时 Windows ignores 已解除，展示 22/22 和完整 workspace Test 通过。最终 native 18 格、162 case、36 次建议复制及混合流 12 格通过；just ci 16/16已通过，首失败保留。上述停止状态属于历史记录；当前证据见 checks/verification.md。
