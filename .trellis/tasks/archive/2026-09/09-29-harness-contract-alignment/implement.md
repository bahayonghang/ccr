# T02 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对当前共享文档、关联来源、Git 交付文件和本地 Python/Bun 版本。
- [x] T01 已报告受控注入及交付夹具 14/14 通过；主会话按依赖启动 T02。
- [x] 主会话已校验上下文并运行 task.py start；当前 task.json 为 in_progress。

## 步骤
- [x] 复核 F03、F04 与白名单；包级 cargo 命令缺口已报告并由主会话修正。
- [x] 新增两个 Python 文件实现静态检查与负例；没有覆盖主会话的共享说明。
- [x] 4 条 AC 的本地静态与受控测试通过，证据边界见实施记录。
- [x] 独立强模型完成审查，并补齐 HC005 独立环境变量赋值的负例；见父任务最终集成审查。
- [x] 主会话已回写 AGENTS、CLAUDE、双语 harness 页和 gate skill，均明确五工具适用；未提交归档。

## 必须通过的检查
- [x] `python -m unittest scripts.quality.test_check_harness_contracts`：初轮 20/20，独立自修后 21/21。
- [x] `python scripts/quality/check_harness_contracts.py`：6 个共享文件、5 个工具。
- [x] `just docs-check`：audit、构建、前端文档审计均通过。
- [x] `just version-check`：7.4.0 及现有治理门禁通过。
- [x] `git diff --check`：退出 0；新增 Python 文件补充 no-index 检查无空白错误。

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

结果与 T06 入口见 [实施记录](research/implementation-results.md)；原始命令结果见 [verification.json](research/evidence/verification.json)。
