# T06 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对 git status、关联来源与当前依赖版本。
- [x] 主会话确认前置项的本地实施和窄门槛后分派 T06；本项集成这些已验证代码。T03 AC3 真实 Dependabot 运行、T01 原生加载/已提交新检出等边界继续单列，不代表前置 AC 全部关闭。
- [x] 主会话完成上下文验证后启动 T06；task.json 为 in_progress。

## 步骤
- [x] 复核 F08 与设计文件白名单；发现新范围先报告。
- [x] 按 design.md 实施最小修改，并保留原失败证据。
- [x] 逐项满足 prd.md 的 4 条 AC。AC1–AC3 证据见 `research/implementation-results.md`。AC4 只覆盖 2026-10-04 迁移后 `just ci` 的实际执行项。T03 AC3、T01 原生加载/新检出、T08 原 0 例外和 hosted CI 继续单列。
- [x] 强模型独立审查完成；T02/T03/T06 见父任务最终集成审查，T01/T04/T05/T08 见首轮审查。
- [x] 主会话已回写项目规范并注明五套工具适用；未提交或归档。

## 必须通过的检查
- [x] `python -m unittest scripts.ci.test_check_workflow_governance scripts.ci.test_architecture_contract_gates` — 48/48
- [x] `just workflow-governance-check` — 48/48，47 个 immutable action 引用
- [x] `actionlint`
- [x] `just ci`：2026-10-04 在已记录的阻塞项处理之后退出 0，15:06.415，16 步全 OK。回执 `../09-29-evergreen-harness-audit/research/resume-2026-10-04-vsce4-full-ci.json`。该结果不关闭 T03 AC3、T01 原生加载或 T08 原 0 例外。历史 FAIL 回执保留。
- [x] 执行前后 2509 个交付源文件哈希一致，`changed_source_paths` 为空，`source_bytes_preserved` 为 true。同一回执。
- [x] Rust/Tauri coverage 与 MSRV 已在 T07 单列实际 OS、版本、完整命令和原阈值；托管边界未替代。

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

实施记录：`research/implementation-results.md`。最终窄检查 SHA：`research/final-scoped-check-results.json`。独立审查、完整 `just ci` 和全仓库源漂移检查由主会话执行，本任务保持 `in_progress`。
