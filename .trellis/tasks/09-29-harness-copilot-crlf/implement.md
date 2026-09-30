# T04 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对 git status、关联来源、core.autocrlf=true 与 Node 版本。
- [x] 本项无前置子任务。
- [x] 主会话已通过上下文校验并启动 T04；实施后 task.py validate 再次通过。

## 步骤
- [x] 复核 F06 与设计文件白名单；实现保持在四个批准文件内。
- [x] 按 design.md 实施最小修改，并保留原失败证据。
- [x] 逐项满足 prd.md 的 3 条 AC；独立目录使用未提交交付候选，证据边界已明确。
- [x] 强模型独立复核代码、边界、负例与证据；见[独立审查](../09-29-evergreen-harness-audit/research/independent-review.md)。
- [x] 回写双语 Copilot 工作区说明并注明五套适用工具；未提交或归档。

## 必须通过的检查
- [x] `node --test scripts/quality/check-copilot-assets.test.mjs`：34 pass / 0 fail / 0 skip。
- [x] `just copilot-check`：12 个资产、4807 个已跟踪文件通过。
- [x] `just docs-check`：文档审计、构建及 UI 文档审计通过。
- [x] `git diff --check`；新增测试另用 no-index 检查，无空白错误。

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

详见 [实施记录](research/implementation-results.md) 与 [验证结果](research/evidence/verification.json)。
