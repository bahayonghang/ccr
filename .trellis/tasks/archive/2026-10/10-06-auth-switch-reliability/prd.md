# 修复 Codex Auth 账号切换失效与 refresh_token 快照失同步

## Goal

修复保存/切换/外部登录新账号场景下，已保存账号因 refresh_token 轮换失同步而失效的问题；定位并修复切换功能本身不可用的原因。切换后目标账号应立即可用；任何观测到的失效都有明确诊断与可执行的恢复路径。

## Authorization and Status

2026-10-06 用户报告并要求深入分析与修复（含"auth 切换也不行，实现上都有问题"）。本版为规划产物，实施需评审后 `task.py start`。

## Background and Confirmed Facts

见 research/current-auth-switching-state.md（完整锚点）。要点：
- 快照仅在 save_current 写入（codex_auth_service.rs:862）；switch_account（:1106）覆盖 runtime 前不回写换出账号 tokens。
- 自动回写仅 platforms/codex.rs:1289 一处；TUI 加载/刷新/切换均不调用 sync_runtime_tokens_to_saved_account。
- 自动修复谓词 should_repair_tokens（openai_quota_core.rs:351）不匹配 refresh_token_invalidated → 截图错误不触发修复。
- repair 来源为 runtime auth.json + ~/.codex/backups（runtime_switch 优先扫描，按新鲜度取最新，codex_oauth_token_service.rs:209-275）；切换时 commit_plan 创建 runtime_switch 备份（codex_runtime_service.rs:245）；备份保留为 auth 前缀共享池 10 个（跨标签共享，codex_config.rs:41、:271-299）。
- 身份匹配仅 account_id（codex_auth_service.rs:714；codex_oauth_token_service.rs:333）。
- TUI 切换仅提示运行中进程、成功后退出 TUI（app.rs:1264）。
- 参考机制见父任务 research/codex-auth-reference.md。

## Requirements

| ID | 要求 |
| --- | --- |
| R1 | 切换前回写：switch_account 覆盖 runtime auth.json 前，若当前 runtime 属于某个已保存账号，按新鲜度回写（runtime 较新 → 写快照；快照较新且该账号为 current_auth → 经备份+原子写回 runtime）；内容相同跳过，不产生半写。 |
| R2 | 观测点同步：TUI 加载/手动刷新/切换调用统一同步入口（保存路径由 save_current 整文件复制天然满足，无需新增调用）；活动已保存账号的配额读取以 runtime 文件为源（服务层路由），刷新落盘 runtime 后同步快照；同步幂等、新鲜度定向、内容比较短路，不引入后台守护或轮询。 |
| R3 | 修复触发：自动修复覆盖 refresh_token_invalidated 等实际错误族；修复后重试使用修复后的快照；修复失败给出可执行状态而非静默 ERR。 |
| R4 | 恢复来源：repair 按新鲜度选择可用来源（runtime 与 auth.*.bak 同池比较，runtime_switch 标签仅影响扫描顺序）；备份保留为 auth 前缀共享池 10 个，跨标签共享；子任务 3 的加固不得缩减该保留池，不得改变 TUI 可见错误文本；不可恢复账号显示"需重新登录"类明确信息；修复失败不清空账号与快照。 |
| R5 | 身份匹配：account_id 缺失/异常时的匹配不得误回写他人快照；多账号同 account_id 场景有确定行为；user_id 维度缺失的边界记录在案。 |
| R6 | 切换可用性：复现并归档"切换不可用"的具体形态（进程占用、凭据存储模式、config/profile 交互、normalize 字段、should_quit 行为等候选）；修复经证据确认的原因；新增防护不得无据阻断正常切换。 |
| R7 | 窗口边界：外部 login 覆盖且 CCR 未运行期间的理论丢失窗口不可完全消除；该窗口内的丢失必须被检测并呈现"需重新登录"；给出最小化措施（观测点最大化）与文档说明。 |
| R8 | 兼容与安全：快照/注册表格式向后兼容；私有权限、原子写、密钥不进日志；错误变体集冻结（ccr-error-freeze）；10-05 用量/配额语义不回归。 |

## Acceptance Criteria

- [x] AC1（R1）：合成测试：A 活动且 runtime tokens 较新（已轮换）→ switch B 后 A 快照包含轮换后的 tokens；快照较新且 A 为 current_auth（CCR 自身配额轮换场景）→ 观察点把快照 tokens 写回 runtime；switch 失败路径不产生半写状态。
- [x] AC2（R2）：TUI reload/切换路径调用同步且内容相同时不写文件（内容断言）；活动已保存账号的配额读取以 runtime 为源且刷新后快照同步（路径与内容断言）；同步失败不阻断主流程并产生可诊断日志。
- [x] AC3（R3、R4）：refresh_token_invalidated 触发 repair 并重试成功（合成）；无可用来源时状态为"需重新登录"且账号与快照保留。
- [x] AC4（R4）：runtime_switch 备份按新鲜度参与恢复；备份保留为 auth 前缀共享池 10 个（跨标签共享），测试不得假设 runtime_switch 独享保留；超限行为有测试或文档边界。
- [x] AC5（R5）：缺失 account_id、同 account_id 双账号、account_id 变化场景无跨账号误写。
- [x] AC6（R6）：切换不可用分析报告含可复现结论（至少一例根因，或全部候选的验证结论）；合成端到端两场景：(a) save A → 模拟轮换 → CCR 观察点（TUI 加载或切换）→ 断言快照持有轮换后 token → login B（写 runtime）→ switch A → quota stub（拒绝被消费的 refresh_token）成功；(b) 模拟轮换后无观察点直接 login B → 观察点检测到快照 token 已失效 → 呈现"需重新登录"，不静默覆盖。
- [x] AC7（R7、R8）：旧 registry/快照可读；日志无 tokens；ccr-codex 与 ccr-tui 检查通过；10-05 既有测试通过。
- [x] AC8：`just lint-strict`、`just test` 通过；原生手工验证（切换 + 配额）另记，未验证项标 NOT_RUN。

## Out of Scope

- 修改 codex CLI 本体或拦截其登录；代理转发；自动轮换。
- TUI 面板显示（10-06-usage-panel-display）。
- 非行为性加固审计（10-06-reference-hardening）。

## Risks and Evidence Boundaries

- 外部 login 覆盖窗口不可完全消除；以观测点最大化 + 失效检测（"需重新登录"）+ 明确边界为准。
- "切换不可用"症状未归档；AC6 允许以"候选全验证 + 至少一例根因"收口；若全部候选不成立需与用户补充复现。
- 真实账户端到端验证受限；以合成 fixture 为主，原生验证另记。

## Decision Log

- 2026-10-06（实施期 D5）：research/switch-failure-analysis.md 证明用户主场景根因为外部 `codex login`/`codex logout` 调用 `/oauth/revoke` 吊销旧账号 refresh_token（codex 0.160.1；上游 PR #17825、#21747），不是轮换失同步。吊销不可由同步或 repair 恢复。用户选择「原计划 + 本地登出缓解」：D1–D4 按计划实施；新增 R9。

| ID | 要求 |
| --- | --- |
| R9 | 吊销缓解：CCR 本地登出（TUI `o` / `ccr codex auth off`，file 存储）删除 runtime auth.json 前先执行 D1 同步；`refresh_token_invalidated` 等永久失效错误呈现为「需重新登录」，并在 TUI 提示登录新账号前先用 `o` 本地登出（本地删除不触发 revoke）。 |

- [x] AC9（R9）：本地登出前同步有测试（runtime 较新 → 快照落盘后再删除 runtime）；TUI EN/ZH 永久失效错误呈现「需重新登录」与 `o` 提示。
