# T04 实施与验证记录

执行时间：2026-09-30 UTC；精确时间戳、OS、Node 版本和源文件 SHA-256 见 evidence/verification.json。

## 结果

T04 实现与本地验收完成，保持 in_progress，等待主会话独立复核。共享说明已回写两个 Copilot 工作区页面，注明 Claude Code、Codex、Grok Build、Kimi Code、OMP 均可运行相同检查。

## 文件与责任

- scripts/quality/check-copilot-assets.mjs：仅在解析时处理 LF、CRLF 和单个起始 BOM；识别独立的分隔符行，检查必填非空值，拒绝未闭合或格式错误的头部。
- 解析使用当前资产的单行标量格式。支持普通值、单引号值及 JSON 转义兼容的双引号字符串；没有引入通用 YAML 库或多行 YAML 语义。
- 删除 ignored .claude/skills 的强制目录要求。改为检查 AGENTS.md 和已跟踪 .codex/skills/*/SKILL.md 的可用性；全部既有术语禁用项保持有效。
- scripts/quality/check-copilot-assets.test.mjs：新增 34 个真实文件与 CLI 测试，使用各自独立的临时 Git 目录。
- docs/guide/github-copilot-workspace.md 与 docs/en/guide/github-copilot-workspace.md：同步共享规则来源、可移植检查入口、换行/BOM 和失败行为。

没有修改 AGENTS.md、CLAUDE.md、harness 总说明、根 justfile、治理脚本、审批元数据或全局配置。没有改写工作区资产的换行。临时夹具的 git add 只修改自有临时仓库索引；实际仓库未暂存、提交或归档。

## AC 与检查

| 验收项 | 结果 |
| --- | --- |
| T04-AC1 | PASS：LF / CRLF 各自搭配有/无 BOM 均成功；前后字节比较一致。 |
| T04-AC2 | PASS：缺失、普通/引号/空白/注释/null 空值、缺/伪分隔符、未闭合头部、字段格式、重复键、引号格式和结构化必填值均失败。applyTo、agent、name 分别有缺失负例，5 个术语禁用项分别验证非零退出。 |
| T04-AC3 | PASS（本地交付候选）：本机 core.autocrlf=true 的实际资产检查通过。18 个 Git 可交付文件构成的临时 Git 目录通过，目录内没有 .claude。 |

| 命令 | 结果 |
| --- | --- |
| node --test scripts/quality/check-copilot-assets.test.mjs | 工作区 34 pass / 0 fail / 0 skip。 |
| just copilot-check | PASS，校验 12 个资产并扫描 4807 个已跟踪文件；原基线 18 个缺字段误报不再出现。 |
| just docs-check | PASS：frozen install 无依赖变更；docs audit、VitePress 构建及 ccr-ui docs audit 通过。 |
| 独立目录 node scripts/quality/check-copilot-assets.mjs | PASS；18 个明确选取的源码文件中包含规则、技能、资产、检查器及测试。 |
| 独立目录 node --test scripts/quality/check-copilot-assets.test.mjs | 34 pass / 0 fail / 0 skip。 |
| node --check 检查器和测试 | 均 PASS。 |
| git diff --check 与新文件 no-index 检查 | 无空白错误；no-index 的退出码 1 表示新文件差异，额外要求无任何诊断。 |
| task.py validate T04 | PASS。 |

可复现入口：python .trellis/tasks/09-29-harness-copilot-crlf/research/verify_selected_source.py。结果、命令、退出码、哈希与日志见 evidence/verification.json。先前工作区测试和 docs-check 以原工具执行记录列于 earlier_tool_receipts，未伪造完整日志。

第一轮独立验收中，行为测试全部通过，新增测试文件末尾多了一个空行。该空白错误已修复，原结果保留在 evidence/verification-attempt-1.json。随后独立验收再次通过。

## 证据边界

临时目录通过 git ls-files --cached --others --exclude-standard 及明确路径选择源码，包含未提交、非忽略的新测试文件。只在临时仓库中将所选文件加入索引，未创建提交。该检查证明交付候选独立性，未证明已提交完整 clean clone 或原生客户端加载。

Linux/macOS、托管 CI、GitHub Copilot 原生界面和五套 harness 的新会话加载：UNVERIFIED。任务本地检查不替代这些环境证据。
