# 父任务执行与验收顺序

## 规划收敛

- [x] 用户已确认停止后切换、新会话使用；保存无停止前置。用户授权实施，要求不测试、不影响当前运行的 Grok Build。
- [x] 复核父研究中固定官方 scope/credential 证据与已确认多账号范围；保留三个子任务的授权和验收边界。
- [x] 已呈现收敛后的最终方案；用户明确回复“审核完毕，批准开始实施”，最终审阅完成，无需重复审批。
- [x] 本会话用户明确要求开始实施现有父子任务；已启动父任务及三个子任务，TUI 在服务 DTO/API 契约冻结后开始接入，最终集成等待服务实现与清理结果。

## 子任务顺序与所有权

1. 09-08-opencode-auth-residual-cleanup：配置旧标识、theme 孤立配色、OpenCodePaths、对应规范/README。独立可验收。
2. 09-08-grok-auth-account-service：core 锁/secret 必需补齐、Grok 账号服务、共享 auth_off 协调、服务规范/测试。可独立于清理进行，但不得动 TUI/theme。
3. 09-08-grok-auth-multi-account-tui：等待服务 API 和清理后的 theme；负责 grok_auth、主 app 路由/生命周期、退出总结、TUI 规范及帧/交互测试。

只有文件互不重叠时可并行；实现/check 遵循项目 Trellis subagent 工作流，注入真实 JSONL。各 worker 明确独占文件，不回滚其他人的改动。

## 父级集成验收

- [ ] 从主入口和 ccr grok auth 进入同一多账号页面，六页顺序、其他 Auth 与 Profile 回归。
- [ ] 临时 GROK_HOME/CCR_ROOT 完成 A 保存、B 保存、A 更新 token、切 B、切回 A、删除保存项、独立登出全链路。
- [ ] 保存独立验证：副本落 CCR Grok 目录，官方锁被占用仍能保存，无源文件/官方锁修改，无退出/登录/登出/refresh 动作；保存界面和文案不混入切换的生效前提。
- [ ] 覆盖外部换号、同名覆盖、未知身份、损坏 JSON、scope 不匹配、多 scope 保留、锁/CAS/磁盘失败和敏感输出。
- [ ] 覆盖别名选择、弹窗、Busy、语言、resize、换页及退出总结。
- [x] 同步 spec/README 和现有中英文 Grok/TUI 文档，不重写历史或扩到 ccr-ui/VS Code。
- [x] 完成三个子任务独立静态审查及父级集成核对，发现已修正；证据见 research/implementation-review.md。测试与 just ci 为 SKIPPED（用户要求），未验证行为不标 PASS。

## 验证

用户最新要求优先：本轮不执行任何测试、just ci、运行时验证、构建或安装；不启动/停止/探测当前 Grok Build，不读取或改动真实认证/配置文件。只维护代码和必要文档/测试源码，做静态审查。下列命令及子任务列出的命令仅保留为后续验证参考，本轮全部不执行：

```powershell
rtk proxy just version-check
rtk proxy just fmt-check
rtk proxy just ci
```

本轮不恢复或安装依赖。后续若另行执行上述验证，应记录真实退出码并区分未执行项；core 验证参考服务子任务。本轮不以静态审查替代动态证据。

结构校验：task.py validate 对每个成员；全局 plan_precheck.py 对父任务 --include-descendants。结构通过不是实现批准或 native 行为证据。

## 交付边界

最终汇报源码变更、静态审查、跳过的测试及旧配置整份回退影响；区分实现交付和未验证行为。本会话已按明确授权进入实施。安装、UI、真实账号、当前 Grok Build 进程、提交/push/发布均不操作。
