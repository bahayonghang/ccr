# T03 Dependabot Bun 生态配置修复

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F05，完成 Dependabot Bun 生态配置修复。优先级 P1。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：无前置子任务。

## 要求与验收
- [x] T03-AC1：docs 与 ccr-ui 的 bun.lock 对应 bun 生态，VS Code 的 package-lock 对应 npm；重复目录和错配会失败。
- [x] T03-AC2：配置修复后现有治理、版本、冻结安装和文档检查通过。
- [ ] T03-AC3：下一次真实 Dependabot 运行的结果单列；本地配置检查通过不能写成托管任务已成功。真实托管运行仍为 UNVERIFIED，未主动触发。

实施与本地证据：[research/implementation-results.md](research/implementation-results.md)。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
强模型确认生态与锁文件契约；Codex / Claude / Kimi coder / Grok general-purpose / OMP 中可用低成本模型执行两个确定配置修改及回归测试。

## 完成后的知识回写
dependency-governance.md：五工具执行同一锁文件治理；harness 页通过 T02 引用。
