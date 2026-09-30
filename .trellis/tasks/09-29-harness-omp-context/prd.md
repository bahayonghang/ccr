# T01 OMP 上下文完整性与可复现交付

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 F02、F03，完成 OMP 上下文完整性与可复现交付。优先级 P1。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：无前置子任务。

## 要求与验收
- T01-AC1：复杂任务的 main / implement / check 按约定收到 PRD、设计和实施标记；research 的公开任务文档及隔离策略与父契约一致。
- T01-AC2：轻量任务、缺失文件、非法路径、符号链接越界、超额文件及多字节截断均有有效断言。
- T01-AC3：仅版本库内容构成的测试目录能解析 OMP 导入并运行契约测试；无需个人目录或已安装的 Trellis 状态。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
Codex 或 Claude Code 强模型负责注入边界和信任根审查；低成本模型只执行批准后的文档枚举、固定夹具和标题断言调整。OMP 原生验证单独记录。

## 完成后的知识回写
T02 回写 docs/agents/harnesses.md 与英文镜像，明确 OMP 适用；保持本地定制与 Trellis 更新保留策略。
