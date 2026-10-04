# T07 历史 CI 失败复验与诊断证据

状态：最终计划已获用户批准；任务启动和验收状态以 task.json 与实施记录为准。

## 目标
解决父报告 H01、H02、H03、O01，完成 历史 CI 失败复验与诊断证据。优先级 P2。

## 依据
- 父报告：../09-29-evergreen-harness-audit/research/audit-report.md
- 原始记录与退出码：../09-29-evergreen-harness-audit/research/check-results.md
- 前置依赖：T06。

## 要求与验收
- T07-AC1：每个历史 run 的原 SHA、失败步骤和当前复验状态可追溯，未查明的底层根因明确保留。
- T07-AC2：Linux Rust coverage 总体 70%、gateway 85%；Tauri gateway 85%；前端和 VS Code 70% 按现有定义验证，不替换成局部测试。
- T07-AC3：Windows/macOS 进程测试、MSRV 1.95、全新会话 hook/trust 证据有独立状态；缺少环境时任务保持未完成。
- T07-AC4：安全警告与旧失败不通过隐藏、降阈值或普通本地 PASS 关闭。

## 范围边界
仅实施 design.md 的文件白名单和 implement.md 的检查。不得修改用户全局配置、降低门禁、提交、推送或发布。Trellis check 的可写自修角色仅限批准后的实现范围。

## 模型与工具
Claude Code / Codex 强模型追踪并发、子进程和安全边界；Grok/Kimi plan 可做只读复核。低成本模型仅收集已定义日志和运行固定命令。

## 完成后的知识回写
测试夹具规范和依赖治理说明记录事实与未验证边界；跨工具通用，OS 边界单列。
