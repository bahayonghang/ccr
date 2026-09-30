# T02 五套 harness 共享说明与角色权限对齐

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F03、F04，完成 五套 harness 共享说明与角色权限对齐。优先级 P1。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：T01。

## 要求与验收
- T02-AC1：五工具表不再写本机 Kimi 没有项目 agents；对新检出明确所需生成步骤及未验证状态。
- T02-AC2：AGENTS、CLAUDE、双语 harness 页和 gate skill 的并行、审批和副作用规则一致；生成绑定归属保持不变。
- T02-AC3：一致性测试能拒绝失效导入、错误 Kimi 分派说明、串行默认命令和将可写 check 用于审批前的角色漂移。
- T02-AC4：只在仓库说明和已跟踪 skill 回写经批准规则；不复制全局配置、凭据或个人模型设置。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
Claude Code / Codex 强模型维护授权和加载契约；Grok plan 或 Kimi plan 可在明确委派后做只读交叉审查。低成本模型适合批准后的双语镜像和固定规则测试。

## 完成后的知识回写
共享规则写入 AGENTS.md、CLAUDE.md、双语 harness 页和 ccr-gate-recovery skill；每项注明五工具通用或工具专属。四份 ignored platform-map 仅作本地生成漂移线索，不作为团队唯一知识来源；上游 Trellis 改动另行授权。
