# T04 Copilot 检查的跨平台换行兼容

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F06，完成 Copilot 检查的跨平台换行兼容。优先级 P2。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：无前置子任务。

## 要求与验收
- T04-AC1：等价 LF 与 CRLF 文档产生相同字段结果，BOM 行为明确。
- T04-AC2：缺失必填字段和未闭合 frontmatter 仍非零退出；现有术语禁止项仍有效。
- T04-AC3：本机 core.autocrlf=true 与仅跟踪源文件夹具均通过，不依赖用户的 .claude 目录。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
适合任何有 shell/edit 权限的低成本执行模型；强模型审查换行和缺字段的负例覆盖。Copilot 为附加检查对象，不扩展为第六套执行 harness。

## 完成后的知识回写
双语 Copilot 工作区说明记录可移植入口和检查命令，五套工具均可运行。
