# Codex Auth P1-P6 最终本地集成验收

状态：`VERIFIED_LOCAL_PENDING_DELIVERY`。P1/P2/P3/P4/P6 的批准实施范围通过本地验收；P5 为 `ASSESSMENT_SCOPE_PASS / PRODUCT_MIGRATION_NOT_IMPLEMENTED`。

日期：2026-10-06 America/Chicago；源码与日志时间保留 UTC。分支 `dev`，HEAD `c524ac07e77e94587399d969546ed75f13cfaf19`。最终产品源码冻结于 `2026-10-07 02:33:43 UTC`。

## 授权与当前状态

指定聊天 `01a11174-8f5a-70b0-8cc7-0e5c5b980007` 已读取。用户授权现有父子任务的规划补齐、顺序实施、检查与相关规范/证据写回。顺序为 P2 → P1 → P3 → P4 → P5 → P6。完整身份缺失时跳过跨文件关联；全部旧备份保留且不清理；导入身份冲突拒绝；现有加密导出保持。用户选择稍后完成 k12 本人登录。

提交、归档、推送、PR、发布、安装与真实凭据操作未授权，也未执行。任务状态保持 `in_progress`；父任务包含归档的验收条款继续未完成。P5 建议暂缓迁移，迁移决定与实施未授权。

## 最终正式门

命令：`just ci`。全部 16 阶段 PASS，native 与 shell 退出码 0，总用时 `22:07.899`。执行会话 `77215` 已退出。原始日志：`checks/ci-final-first-attempt.log`，末尾为 `NATIVE_EXIT_CODE=0`，日志最后写入时间为 `2026-10-07 02:59:55 UTC`。

日志 SHA256：`330a6d113fd2950d7d15b51bd5affb29144bc1c78c38a39c089c517c00144ca5`。

| CI 阶段 | 结果 | 用时 |
| --- | --- | --- |
| Version Check | PASS | 00:02.261 |
| Format Check | PASS | 00:06.144 |
| OMP Check | PASS | 00:02.373 |
| Harness Check | PASS | 00:01.874 |
| Copilot Check | PASS | 00:17.813 |
| Strict Clippy | PASS | 00:21.081 |
| Workspace Check | PASS | 00:15.284 |
| Test | PASS | 05:02.138 |
| Release Build | PASS | 03:13.524 |
| Security Audit | PASS | 00:11.829 |
| CI Governance | PASS | 02:07.976 |
| Tauri CI | PASS | 05:10.375 |
| Frontend Check | PASS | 02:56.350 |
| Frontend Audit | PASS | 00:04.414 |
| Frontend Coverage | PASS | 01:58.544 |
| VSCode CI | PASS | 00:15.888 |

`just ci` 包含最终 `just version-check`、`just fmt-check`、`just lint-strict`、`just test` 及各子系统正式门。Rust workspace 与 Tauri 产品测试保留默认并行度和 `--skip export_bindings`；独立绑定生成门负责导出。未修改阈值、audit allowlist、ignore 或测试并行设置。

| 验证范围 | 最终日志结果 |
| --- | --- |
| Rust workspace | 41 个 suite，2038 passed、0 failed、16 ignored；从 workspace 阶段的 41 条 `test result:` 汇总，排除后续 Tauri/绑定阶段 |
| Codex | 381 passed、0 failed、2 ignored；包含 P1-P4 回归与最终 P6 方法冲突矩阵 |
| CLI | 347 单元、12 集成、1 doctest 通过；1 doctest ignored；主测试门过滤绑定测试，独立门另执行 24 项 CLI 绑定测试 |
| Core | 130 单元与 6 doctest 通过；6 doctest ignored；包含 native Windows 元数据权限测试 |
| TUI | 253 passed；EN/ZH 在 80x24、100x22、100x30、120x22、140x40、180x50 与 60x18 compact 的 composed cells/color 断言通过 |
| Tauri | 407 passed、0 failed、1 ignored；另有 2 项依赖边界测试；独立生成 197 项绑定测试通过 |
| TypeScript bindings | `TypeScript bindings match the worktree baseline`；未产生 tracked 绑定漂移 |
| 前端 | 169 个测试文件、904 passed；常规测试和 coverage 门各执行一次，计数不累加 |
| 前端 coverage | statements 72.92%、branches 59.45%、functions 71.89%、lines 75.19%；门槛通过 |
| VS Code | 51 passed、0 failed；构建与 VSIX 内容检查通过，15 个归档条目；未安装 |
| 文档、审计与治理 | docs 构建/审计、Rust 安全审计、frontend audit 和治理门通过 |

Rust audit 保留 9 项既有允许警告；Tauri 宏构建保留 Windows linker stdout 警告。警告未被删除或新增豁免。正式门的 PASS 不将 ignored 测试计入通过数。

## 子任务合同与证据

| 子任务 | 最终批准范围与证据 |
| --- | --- |
| P2 | 未知 TOML 字段往返、版本只读门、6 个用户命令副作用预检、后台元数据跳过和 EN/ZH 提示。独立报告：`10-06-auth-registry-schema/research/independent-check.md`，较早 workspace 记录为历史证据。 |
| P1 | 完整 user/account 身份隔离、别名新鲜度、锁内来源重读、并发刷新/切换协调、同源保存元数据、有效 access token 先 GET 与手动缓存绕过。独立报告：`10-06-auth-identity-key/research/account-lifecycle-check.md`。 |
| P3 | 最新内容备份去重、同秒序号防覆盖、精确备份池、私有发布、删除/强制重命名前必要备份成功、保留全部旧备份。独立报告：`10-06-auth-destructive-backup/research/independent-check.md`。 |
| P4 | SID DACL、版本校验后元数据加固、bytes/mtime 保持、私有 rename 回退、敏感 Debug 与 HTTP 诊断脱敏、TUI EN/ZH 矩阵。Core 和完整独立报告：`10-06-auth-defense-in-depth/research/core-independent-check.md`、`research/independent-check.md`。 |
| P5 | 命名收益、成本、别名引用、兼容与回滚约束评估；建议暂缓迁移。独立报告：`10-06-auth-snapshot-naming/research/independent-check.md`。现有 `auth/<name>.json` 保持；未声称迁移测试通过。 |
| P6 | typed 整包解析、全部待写条目身份预检、锁后身份/原始内容版本/注册表复核、旧与 incoming 身份锁、私有版本替换、CLI 服务错误传播。Merge/no-force、Replace、metadata-only、现有加密格式与导出规则保持。独立报告：`10-06-auth-import-export/research/independent-check.md`。 |

P6 独立报告还包含 22 个当前产品文件的 P1-P6 静态集成审查。最终 CI 使用审查代理加入两项 auth_method 冲突案例后的源码。较早独立报告及哈希清单保留原有范围、源码身份和 `WORKSPACE_NOT_RUN/PENDING` 状态；父任务报告追加最终验证，不重写历史报告。

规范写回已完成：Codex 的身份、锁、备份、版本门、权限/诊断与导入合同，Core 的版本校验元数据权限接口，CLI 导入错误传播，以及 TUI 本地化/矩阵证据边界。最终审查与 CI 未要求新增依赖、公开 DTO、错误变体或模板/生成配置变更。

## 源码与证据身份

状态写回前，P6 `research/independent-hashes.json` 中 71 项全部匹配：22 项产品源码、8 项合同、8 项较早审查、2 项 owner manifest、23 项 owner 证据与 8 项 reviewer 证据。该清单及其引用保留为历史证据。

| P6 最终文件 | SHA256 |
| --- | --- |
| `crates/ccr-codex/src/services/codex_auth_service.rs` | `d0ba7f0924fdfdb6e714476624a3f69d3d25dd630fe1be95598c08aa5fa3112b` |
| `crates/ccr-codex/src/services/codex_auth_identity.rs` | `22fccbeb10626998637bef7925a4f11bae3f66ff18d881d949e5fe3b22a770aa` |
| `crates/ccr-codex/src/services/codex_auth_refresh_lock.rs` | `fd1fbb64d499a896e2087a848634cd96fa6f15bfec5c0ed1cda3bd76df8cfd3d` |
| `crates/ccr-cli/src/commands/codex/auth/import.rs` | `11197b8cdb56709233defb5bab70340d22cbd5fa9a3d733077fd18dfcc507b42` |

最终父任务 `research/integration-hashes.json` 捕获全部 22 项产品源码、4 项规范、父子任务验收文档、原始 CI 日志和独立报告/清单。清单不包含自身；后续文档变化应追加新身份记录，保留该清单的历史意义。父任务收尾仅修改任务验收文档，不修改产品源码，不重复已通过的 CI。

Post-CI Git 状态未出现批准产品范围外的 tracked 生成变更。已有未跟踪 `skills-lock.json` 保留，未读取或修改。现有修改与全部旧备份保留；未执行 cleanup、stash、reset 或 checkout。

## 保留的失败与未验证边界

- k12 本地注册表和快照存在。先前只读额度 GET 返回 401；原因未查明。用户选择稍后登录。真实 OAuth、账号切换/恢复与远端凭据有效性为 `UNVERIFIED/NOT_RUN`；合成成功查询不证明 k12 已恢复。
- Windows 上 `Foo`/`foo` 可能指向同一物理快照。备份池已按物理大小写别名协调，但账号路径/强制重命名的既有风险仍未修复；P5 原生复现为 `NOT_RUN`。相关保护没有获得扩大范围授权。
- P6 独立审查报告了 namespaced `chatgpt_account_id` 的 identity/quota fallback 差异，以及局部 OAuth/quota serde 解析错误文本。状态为 `SOURCE_REVIEWED / RUNTIME_NOT_RUN`；未证明运行时泄漏，未扩大 P6 修复范围。
- P3 报告的独立 provider-store 备份错误处理仍在范围外。公共 registry load/save 未升级为通用读改写事务。平台 profile/auth-off 的完整运行时替换区间未加入 P1 的全部操作锁。
- Replace/no-force 无新增快照前像保证；metadata-only 沿用既有语义。多文件写入、权限修改与 registry 发布非整体事务，后期 I/O 失败可能保留前期作用。外部 Codex/login/编辑器不遵守 CCR 锁。
- 跨进程证据验证文件锁与备份池；OAuth HTTP 并发、刷新/切换/rename 行为由同进程合成 loopback 验证。跨进程真实 HTTP 生命周期为 `NOT_RUN`。
- Windows native DACL 测试验证合成临时文件。Unix/Linux/macOS、安装后的 CLI/桌面/扩展、交互式原生 TUI、真实账号、hosted CI 与 fresh-client loading/trust 为 `NOT_RUN`。
- Core 最初测试程序启动退出 `0xc0000005`，未进入测试；后续默认和最终 CI 通过。原始失败源码/程序身份与日志保留，原因仍未查明，不归因为增量编译或损坏产物。
- P1/P3/P4/P6 首次失败、修复后复测与对应身份记录继续保留。P6 reviewer 提前启动的测试为 `INTERRUPTED_UNVERIFIED`。首次 whitespace 原始完整日志曾被 wrapper 覆盖，该损失见 `independent-whitespace-first-failure.md`；后续成功不修补缺失证据。
- 父任务最终哈希清单首次生成失败，清单未写入。PowerShell 数组表达式将 execution-status/design/implement/integration-validation 的 4 个路径拼为单一路径，Get-FileHash 的非终止错误导致 JSON 解析失败，工具记录为 `SyntaxError: Unexpected token 'G'`。只读路径探针确认 14 个独立 task/PRD 路径存在、拼接后的路径不存在。已改用逐项构建路径，并启用终止错误；产品源码与 CI 日志未变。工具错误与路径探针保留在聊天记录，首次完整原始 shell 输出为 `NOT_CAPTURED`。
- 收尾空白检查首次退出 1：`execution-status.md: blank line at EOF`。3 个新验收文件中只有该状态文档失败；已移除末尾空行，并更新最终清单的文档哈希。失败输出保留在聊天记录，未改写任何原始 CI 或子任务收据。

## 回滚与交付边界

源码回滚必须按子任务差异恢复批准文件，保留其他子任务与既有用户修改；Core API 回滚需先协调 Codex 调用方。快照恢复使用经过身份核对的 registry/snapshot 前像，并以私有原子写发布。版本冲突不能用旧备份覆盖外部新凭据。元数据权限加固不由空内容 journal 自动放宽。Replace/no-force 无新增快照前像，不能声称完整回滚。未执行回滚或真实数据恢复。

本地技术验收已完成。归档、提交和分发状态保持未执行；任务的归档验收条款继续开放。
