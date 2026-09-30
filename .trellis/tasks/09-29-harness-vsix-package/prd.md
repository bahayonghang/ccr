# T05 VSIX 打包范围与本地文件排除

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F07，完成 VSIX 打包范围与本地文件排除。优先级 P1。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：无前置子任务。

## 要求与验收
- T05-AC1：最终 VSIX 不含本轮已确认的 .serena/project.local.yml、.serena/project.yml、dist/extension.js.map 和内部 agent 说明。
- T05-AC2：使用合成 .serena/.abcoder/本地配置夹具证明拒绝规则；运行必需文件仍全部存在。
- T05-AC3：lint、51 项既有扩展测试和完整打包命令通过；不发布 VSIX。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
Codex / Claude Code 强模型审批发布文件范围；低成本模型可执行排除规则、合成夹具与清单校验。最终包内容需强模型复审。

## 完成后的知识回写
extension-surface-contracts.md 记录五工具通用的打包检查和本地配置保护。

## 已批准扩围：扩展开发依赖

用户批准将本轮新发现的扩展开发依赖公告纳入 T05。

- T05-AC4：仅将 package-lock.json 的 fast-uri 3.1.6 更新到 3.1.7、undici 7.29.0 更新到 7.29.1，保留父依赖兼容范围；核验 registry integrity；npm ci、npm audit、扩展 CI、70% 覆盖率与最终包内容检查通过。新范围或新公告单独报告。
