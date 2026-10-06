# Codex Auth 切换可靠性、用量面板与实现加固

## Goal

修复 Codex Auth 账号切换失效与 refresh_token 轮换失同步；重构本地用量统计面板显示；参考 Loongphy/codex-auth 审计并加固实现，保持既有运行逻辑。

## Authorization and Status

2026-10-06 用户要求创建 Trellis 任务并深入分析、优化修复，并补充说明"不仅是 token 显示失效，auth 切换也不行，实现上都有问题"。本版为规划产物，未开始实施；实施需用户评审后 `task.py start`。规划基线：dev @ 797c1877（工作区干净；10-05 用量估算任务已完成并归档）。

## Background and Confirmed Facts

用户报告（截图 2026-10-06）：
- 账号 khanh 配额显示 `Quota error: Token 刷新失败 (401 Unauthorized) [refresh_token_invalidated]`；k12 正常。
- 复现序列：保存账号后退出，登录新账号，原账号失效。
- 补充：切换功能本身也不可用。
- 右下角 "Local usage · API equivalent USD · Tokens / Records" 面板信息过密、含冗余与无效信息。

代码基线核实（锚点详见子任务 research/current-auth-switching-state.md）：
- 账号快照仅在 `save_current` 写入（crates/ccr-codex/src/services/codex_auth_service.rs:862）；`switch_account`（:1106）覆盖 runtime auth.json 前不回写换出账号的轮换 tokens。
- 自动回写仅一处（清理 OpenAI tokens 前，crates/ccr-codex/src/platforms/codex.rs:1289）；TUI 加载/刷新/切换均不调用。
- 自动修复谓词不匹配实际错误串：`should_repair_tokens` 仅匹配 `refresh_token_reused`/`invalid_grant`，不含 `refresh_token_invalidated`（crates/ccr-codex/src/services/openai_quota_core.rs:351）。
- 身份匹配仅用 account_id（codex_auth_service.rs:714、crates/ccr-codex/src/services/codex_oauth_token_service.rs:333）。
- 备份：commit_plan 写前备份到 ~/.codex/backups（auth.runtime_switch.*.bak，crates/ccr-codex/src/services/codex_runtime_service.rs:245）；保留为 auth 前缀共享池 10 个——所有 auth.*.bak 标签（runtime_switch、reset_to_defaults 等）共用同一池（crates/ccr-codex/src/managers/codex_config.rs:41、:271-299）。
- codex-auth 参考机制：登录用 scratch CODEX_HOME 隔离；每次前台命令前将 runtime auth.json 回写活动账号快照（内容比较后复制）；身份键为 user_id::account_id。详见 research/codex-auth-reference.md。

## Requirements

| ID | 要求 |
| --- | --- |
| R1 | 切换可靠性：保存、切换、外部登录新账号等序列后，在 CCR 观察点（TUI 加载/刷新、切换、保存、配额刷新）之间，任何已保存账号不得因 refresh_token 已轮换而失效；观察点之外无 CCR 参与的外部登录窗口内发生的轮换丢失，必须被检测并呈现为「需重新登录」的明确状态；切换后目标账号可正常使用。 |
| R2 | 面板显示：右下角本地用量面板按信息层级重排，消除冗余与全 N/A 噪声；错误、范围、partial/unpriced 语义保持可见。 |
| R3 | 参考加固：对照 codex-auth 审计存储、身份、备份、权限、恢复实现；保持运行逻辑不变的前提下修复实现问题。 |
| R4 | 约束：EN/ZH 双语；私有权限与原子写；不泄露 secrets；错误变体集冻结；10-05 既有验收语义不回归。 |

## Task Map

| 子任务 | 目录 | 交付 |
| --- | --- | --- |
| 切换可靠性（先启动） | .trellis/tasks/10-06-auth-switch-reliability | 切换失效与 token 失同步修复 + 切换不可用根因分析 |
| 用量面板显示 | .trellis/tasks/10-06-usage-panel-display | 面板信息层级重构（仅呈现层） |
| 参考加固 | .trellis/tasks/10-06-reference-hardening | 对照审计报告 + 无行为变更加固 |

父任务无直接实现工作，负责跨子任务验收与最终集成复核。

## Cross-Child Acceptance Criteria

- [x] 三个子任务各自验收通过并归档。
- [x] 子任务 1 不改变面板数据契约；子任务 2 不改动 ccr-codex 领域服务。
- [x] 子任务 3 无用户可见行为变化（保留池不缩减、TUI 可见错误文本不变）；需行为变更的发现记录为后续任务提案，不转交其他子任务。
- [x] 集成复核：`just lint-strict`、`just test` 通过；Codex Auth TUI EN/ZH 与尺寸矩阵通过。
- [x] 未推送、未创建 PR、未发布（除非用户另行要求）。

集成复核记录：research/integration-review.md（2026-10-06）。

## Out of Scope

- 修改 codex CLI 本体、拦截其登录流程、代理转发、自动账号轮换、额度销售。
- 接管 Codex app-server；读取个人会话正文；真实账号压力/耗尽实验。
- 未经核实的模型定价变更。

## Risks and Evidence Boundaries

- 外部 `codex login` 不可拦截；CCR 只能在其观测点回写。无 CCR 观测点的外部登录覆盖窗口不可完全消除；R1 的保证以观测点为界，该窗口内的丢失必须被检测并呈现「需重新登录」。子任务 1 最小化窗口并记录边界。
- "切换不可用"的具体症状未复现归档；子任务 1 先做可复现分析再修复。
- 备份保留为 auth 前缀共享池 10 个（跨标签共享，codex_config.rs:41、:271-299）；修复来源受该池约束，其他标签备份可驱逐 runtime_switch 备份。
