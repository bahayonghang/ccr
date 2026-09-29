# T10 规范与证据实施报告

日期：2026-09-28。实施 owner：`/root/implement_t10_specs`。本切片完成当前架构规范、导航和逐项验收证据。父任务最终状态由主会话汇总。

## 规范变化

- `dependency-governance.md` 对齐三平台本地 aggregate 复用现有 `tauri-ci`、严格 desktop Clippy、完整行为 suite 与绑定检查。普通测试仅排除有生成副作用的 `export_bindings`，保留原有并行。明确根 `just ci` 仍包含会修改文件的 `version-sync` 和 `fmt`。
- `typed-ipc-bindings.md` 记录事务生成规则：直接生成仅成功时保留结果；失败恢复目录存在状态、目录结构和原始字节。check 在成功、漂移与异常后均恢复调用者原字节，比较当前工作区的规范化基线。
- `tauri-handler-registry.md` 及相关规范对齐当前 registry/manifest：340 个基础命令、348 个 Windows 命令、278 个 typed 命令、278 个 exact wire contracts、38 个基础模块。保留有明确历史标识的迁移例子。
- 版本同步源改为实际 `ccr-ui/src/config/appMeta.ts`；消费路径为 `MainLayoutChrome.tsx` 和 `Titlebar.tsx`。保留版本脚本对该 TS 源使用的既有 `vue` parser 标签。
- backend index、根和 UI `code_map.md` 明确 `crates/ccr-usage` 是 usage SQL 与只读投影 owner；`llmusage_adapter` 仅负责已安装 CLI、NDJSON 和 DTO/error mapping。

## 上下文预算与保留范围

原 `llmusage-provider-adapter.md` 为 37,537 字节，超出单文件 32,768 字节上下文限制。本次只提取 T06 新增的 4,810 字节 usage job lifecycle 内容，形成 `usage-job-lifecycle.md`，并加入 backend index 及 T06 implement/check JSONL。原文件剩余 32,727 字节，原有 analytics、pricing、Home Insights 内容的字节保持不变。

提取后的新 lifecycle 规范另行对齐事务式 `just tauri-bindings` / `just tauri-bindings-check` 指令。该新文件已有这项后续修改；字节保留声明仅指原 adapter 中剩余的既有内容。来源与提取阶段 SHA 见 `research/spec-convergence-evidence.json`，当前全部文件 SHA 与尺寸见 `research/spec-validation.json`。

## 验收证据

`research/requirements-evidence.json` 和 `.md` 覆盖 39 个子任务 AC。每个测试组记录真实源文件、选择器、源码行、原始日志、SHA-256、执行层级、平台和 mock 边界。T10 AC2/AC3/AC4 分别绑定 6/4/7 项 artifact 证据，覆盖 P1 与旧红例、规范与上下文校验、正式/隔离门禁和受保护基线。

父任务 `research/p1-evidence-ledger.json` 的 9 组 P1 已更新为旧基线反例与修后测试映射。A14 保存原 mapper/helper 的单字段修改红例和原始输出。所有 `fix_commit` 保持 `null`。本次记录候选验收证据，不自行改变父任务、子任务或原 Insights 任务状态。

旧反例均标注范围：A02 为确定性交错，A03 为实际提交后的 history failure，A09 只缩短原 descriptor deadline，A11 为真实权限失败与原函数组合，A14 为原 mapper/helper，A15 为真实页面与底层 IPC mock。未运行的旧分支、macOS、native WebView、真实远程和真实 OAuth 验收继续明确列出。

## 检查结果

- `research/validate_spec_evidence.py`：8 个规范/导航文件、25 个相对链接、2 组任务上下文、2 个受保护文件全部通过。所有相关单文件均小于等于 32,768 字节。
- `task.py validate`：T06 implement/check 各 8 条；T10 implement/check 各 9 条，全部通过。
- `research/validate_evidence_mapping.py`：检查 39 AC、9 P1、源码与日志 SHA、精确选择器/行、证据组一致性和 Python `Ran 27 tests` / `OK` 解析。结果见 `research/evidence-mapping-validation.json`。
- 本 owner 对门禁代码另行完成非作者交叉审查，独立运行 Python 27 项和前端 25 项检查，均通过；范围与限制见 `cross-check-gates.md`。

两个原 `.tmp` 脚本的 SHA 与基线一致；`09-24-home-insights-redesign` 保持 `planning`，`09-24-home-insights-frontend` 保持 `in_progress`。正式原工作区 frontend gate 的 5 个 `no-console` 错误与隔离授权差异 gate 的成功分别保留，不互相替代。

## 文件与后续范围

修改范围：6 个 backend 规范/索引文件、2 个 code map、T06 两个上下文 JSONL、T10 research 生成/验证脚本与证据、父任务 P1 evidence ledger，以及本报告和门禁交叉审查报告。没有修改业务产品代码、提交、推送、归档或修改父任务执行台账。

T10 专用 checker 的派发因 `agent thread limit reached` 失败；主会话授权采用交叉 owner 非作者审查。最终全部 Rust/Tauri/前端聚合门禁、最新源文件冻结对应关系和父任务验收仍由主会话记录。专用 checker 派发成功与完整原生 UI 验收均未发生。
