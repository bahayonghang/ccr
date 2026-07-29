# .claude.json 写入策略重估(C8)

## Goal

独立评估并重定义 ccr 对 Claude state_file(`.claude.json`,位置受 `CLAUDE_CONFIG_DIR` 影响)的写入策略。原任务树把 C8 定为"补文件锁 + 验证第三方并发不丢字段",经二轮审阅证实**该验收标准不可实现**;本任务以诚实的并发模型重新定义修复。父任务序列第 4 号。

## 问题(源码三轮核实)

- onboarding 写入点:`ensure_onboarding_completed`(`crates/ccr-cli/src/platforms/claude.rs:88-119`),ApiKey profile 应用时整读 `.claude.json` → 插入 `hasCompletedOnboarding: true` → 整写回,无任何并发保护。
- **三轮新增事实**:`ccr-ui/src-tauri/src/commands/claude_mcp_config.rs:181-212/:593-613` 也会为 user/local MCP 作用域整读整写 `.claude.json`,并手搓 `NamedTempFile`。因此“唯一写入点”前提错误;MCP 写入是现有用户功能,不能随 onboarding 一起删除。
- 该文件由 Claude Code 进程高频读写(含 `oauthAccount`、`customApiKeyResponses`、`primaryApiKey` 等)。ccr 写入窗口内 Claude Code 的更新会被整体覆盖。
- **为什么锁不可行**:文件锁只约束合作进程,Claude Code 不获取 ccr 的锁;仓库自身 `guarded_write.rs:12-15` 也明确"load→mutate→save 事务性由调用方负责,本模块只保证单次写"。任何"加锁后无丢字段"的承诺都是虚假保证。
- 可用原语:`write_guarded_versioned` / `VersionedWriteOutcome::{Written,Conflict}`(`crates/ccr-core/src/core/guarded_write.rs:31-42`)提供 CAS——能把"覆盖窗口"缩小到 read-version 与 rename 之间,并把冲突显式化,但**不能消除**与外部进程的竞态。

## Requirements

- R1:删除 `ensure_onboarding_completed` 调用与 doctor 的 `hasCompletedOnboarding` 告警。Claude Code 2.1.220 隔离探针已证明无该字段的 `--bare -p` 能进入认证阶段;交互式 onboarding 由 Claude Code 自己管理,ccr 不写私有状态键。
- R2:保留 MCP user/local 作用域写入,但迁移为:共享正确状态路径 + `write_guarded_versioned` CAS + 最多 3 次立即重读重放变更;重试耗尽显式返回冲突,不得把 MCP 操作报告为成功。写入 `secret:true`,未知字段整份往返保留。
- R3:在 spec 中为 state_file 立边界条款:profile/auth 不写;Tauri MCP user/local 只写对应 MCP 子树;记录 CAS 并发模型与**残余风险声明**(外部进程在 CAS 窗口外仍可修改文件,ccr 不承诺跨进程事务性)。
- R4:与 `07-29-claude-credentials-hardening` 的接口固定为零写面:C10 只通过凭据匹配选择快照元数据,不借用 MCP CAS 回写 `oauthAccount`。

## Acceptance Criteria

- [ ] design.md 引用 Claude Code 2.1.220 隔离探针;API-key profile apply 不再创建或修改 `.claude.json`,过时 onboarding doctor 告警移除。
- [ ] MCP user/local 添加、更新、删除继续工作;CAS 冲突可重试,耗尽后显式失败;`oauthAccount`、`primaryApiKey`、未知字段保留。
- [ ] 不出现任何"加锁保证无丢字段"表述(代码注释、文案、spec)。
- [ ] `just lint-strict` + `just test` + `just frontend-check-quick` 通过。

## Notes

- 本任务因发现 Tauri MCP 第二写面,按复杂任务处理,必须有 design.md + implement.md。
- 依赖:实现时依赖 #2 的共享 state_file 路径;#3 已吸收本任务停写结论并选择 C10 零写面方案。
- Planning status:2.1.220 探针、`design.md`、`implement.md` 与 JSONL 已就绪;父任务序列中在 credentials 子任务后 start,C10 已选择不回写 state_file。
