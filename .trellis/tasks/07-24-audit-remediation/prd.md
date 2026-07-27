# GPT Pro 极限审计整改总控（2026-07-24）

## Goal

跟踪 `ccr_extreme_code_audit_2026-07-24.md`（GPT Pro 审计，基线 `main@9958787`）的逐条核实结果与整改任务映射。父任务持有需求集、任务地图与跨子任务验收标准；实现工作全部在子任务中进行。

## 逐条核实结论（2026-07-24，dev 分支）

审计报告 35 条发现已在当前 `dev` 分支逐条核对，**全部成立**（个别有细微差别，见备注）：

### P1（11 条，全部确认）

| ID | 结论 | 当前 dev 证据 |
|---|---|---|
| P1-01 | ✅ 确认 | `ccr-ui/src-tauri/src/commands/install.rs:44-49` 接收完整 `InstallPlan`；`install_types.rs:177-187` 含 `command/args/envs`；`install_service.rs:72-97` 无 provenance 校验；`install_exec.rs:51-57` 直接 `Command::new(&plan.command)` |
| P1-02 | ✅ 确认 | `platform/ssh.rs:91-96` `shell_escape_single` 只替换 `"` `` ` `` `$`，不处理反斜杠；`remote_home`（:73-78）无校验拼入 double-quoted 命令（:160, :201-203）；`\$(cmd)` 逃逸成立 |
| P1-03 | ✅ 确认 | `ccr-sync/src/sync/service.rs:580-586` `extract_filename` 取 href 末段可返回 `..`；`should_exclude_from_sync`（:638）显式放行 `.`/`..`；`:312` `local_dir.join(&file_name)` 无 containment 检查；Windows `\` 未拒绝 |
| P1-04 | ✅ 确认 | `commands/sync.rs:451-460` rename 备份；`:543-589` `pull_asset_config` 先备份后 pull，pull 失败无回滚 |
| P1-05 | ✅ 确认 | `atomic_writer.rs:288-321` `AsyncAtomicWriter` 用 `async_fs::write` 新建 temp（mode 由 umask 决定），rename 覆盖目标不保留原 mode/ACL；`managers/settings.rs:201-215` `save_atomic_async` 用于含 `ANTHROPIC_AUTH_TOKEN` 的 Claude settings |
| P1-06 | ✅ 确认 | `command_exec.rs:1698` 前台 `cmd.output().await`，无 timeout、无输出上限 |
| P1-07 | ✅ 确认 | `command_exec.rs:1594` `mpsc::unbounded_channel`；`:1611-1613` 每行 `update_and_emit` 发完整 snapshot |
| P1-08 | ✅ 确认 | `ccr-db/src/database/migrations.rs:589,619` `.filter_map(|r| r.ok())` 吞行错误；`:663` `let _ = update_stmt.execute(...)` 吞 UPDATE 错误；`:629` 坏 JSON 静默跳过；`:567-571` 之后仍写 v3 marker |
| P1-09 | ✅ 确认 | `commands/ssh.rs:206-249` `connect_internal` 不握手即 `connected: true`；`:466-487` confirm 直接接受前端指纹写 app DB；`ssh/connection.rs:119` 连通测试用 `accept-new`；`platform/ssh.rs:98-117` 实际读写未绑定 app known_hosts |
| P1-10 | ✅ 确认 | `commands/sync.rs:334-345` 只查非空；`ccr-sync/src/sync/service.rs:37-48` 任意 URL + Basic Auth，无 HTTPS 策略 |
| P1-11 | ✅ 确认 | `.github/workflows/` 仅 3 个 workflow；`ci.yml` paths 不含 `ccr-ui/src-tauri/**`；Tauri Rust 与 VSCode 无 PR 门禁 |

### P2（19 条，全部确认）

| ID | 结论 | 当前 dev 证据 |
|---|---|---|
| P2-01 | ✅ | `codex_auth.rs:653-678` `kill_port_processes` 对端口所有 PID `kill -9`/`taskkill /F`，无归属校验 |
| P2-02 | ✅ | `codex_auth.rs:577-608` `open_external_url` 只查非空，无 scheme/host allowlist |
| P2-03 | ✅ | `release.yml:155-197` VSIX 仅 tag 时 build，未跑 lint/test；无 PR workflow 覆盖 `ccr-vscode/**` |
| P2-04 | ✅ | `frontend-ci.yml:5` PR branches 仅 `[main, develop]`，root `ci.yml:5` 已含 dev |
| P2-05 | ✅ | `commands/sync.rs:610-613` (true,true)+!force 调 `push_asset_config(..., false)`，后者 `:527-535` 见 remote exists 必报错 |
| P2-06 | ✅ | `sync/service.rs:253-256` `.bytes()` 整体缓冲；`pull_directory` 无 depth/entries/bytes 上限 |
| P2-07 | ✅ | `commands/sync.rs:635-658` 先写 legacy manager 再写 folder manager，第二写失败无补偿 |
| P2-08 | ✅（细节修正） | `command_exec.rs:1630-1638` cancel 仅 `child.kill()`（tokio kill 会 reap 直接子进程，但孙进程/进程树不处理，job 立即标记 Cancelled） |
| P2-09 | ✅ | `command_exec.rs:1338` 候选缺失回退 PATH `"ccr"`；`:1365-1373` 身份仅靠 `--version` 自报 |
| P2-10 | ✅ | migrations v3/v4/v5 直接在 `conn` 上多语句执行，无显式事务包裹 marker |
| P2-11 | ✅ | `handler_registry.rs:6-11` `CommandModule` 仅 key/title/commands，无 risk/timeout/audit 元数据 |
| P2-12 | ✅ | typed IPC 仅 Usage V2 + Claude Observer 试点（约 26/315≈8%），其余手写 TS |
| P2-13 | ✅ | Rust `AttemptId` 为 transparent UUID string；`install.ts:23-26` 却声明为 `{[key: string]: string}` object |
| P2-14 | ✅ | `tauri.conf.json` `signingIdentity`/`certificateThumbprint` 为 null；release 仅 SHA256，无签名/notarization/provenance |
| P2-15 | ✅ | `ci.yml:130` `cargo test ... -- --test-threads=1`；无 coverage job |
| P2-16 | ✅ | `dtolnay/rust-toolchain@stable`、Bun `latest`、actions mutable major tags |
| P2-17 | ✅ | MSRV 分裂：多数 crate 1.90，`ccr-db`/`ccr-types`/`src-tauri` 1.88；drift 脚本有 allowlist 但未进 hosted CI |
| P2-18 | ✅ | `atomic_writer.rs:358-361` Unix rename 后无父目录 fsync |
| P2-19 | ✅ | `commands/sync.rs:117-161` 三类 sensitive 资产（ccr-platforms/claude-settings/codex-config）明文 PUT，无客户端加密 |

### P3（5 条，全部确认）

| ID | 结论 | 备注 |
|---|---|---|
| P3-01 | ✅ | `crates/ccr/src/lib.rs` 兼容 facade 大量 re-export |
| P3-02 | ✅ | `command_exec.rs`（1700+ 行）、`migrations.rs`（1800+ 行）、`codex_auth.rs`、`sync.rs` 均超大 |
| P3-03 | ✅ | spec `typed-ipc-bindings.md:56` 写 312/320；`handler_registry.rs:502,505` 测试断言 323/315 |
| P3-04 | ✅ | `command_exec.rs:1244` `lines.remove(0)` O(n) 左移 |
| P3-05 | ✅ | `migrations.rs:562-654` 注释 mojibake（GBK 乱码）；`tauri.conf.json` 单行压缩 |

## 任务地图（9 个子任务）

| 子任务 | 覆盖发现 | 优先级 | 对应报告 Epic | 状态 / 证据 |
|---|---|---|---|---|
| `07-24-audit-install-plan-handle` | P1-01, P2-13 | P1 | A1 | 完成；`b444b459`；install/bindings/frontend/lint/workspace tests |
| `07-24-audit-ssh-hardening` | P1-02, P1-09 | P1 | B1-B5 | 完成；`19cef4b2`；SSH/Tauri/frontend/lint/workspace tests |
| `07-24-audit-webdav-sync` | P1-03, P1-04, P1-10, P2-05, P2-06, P2-07, P2-19 | P1 | C1-C7 | 完成；`0e58e9e9`；sync/Tauri/Vitest/lint/frontend/workspace tests |
| `07-24-audit-persistence-migration` | P1-05, P1-08, P2-10, P2-18 | P1 | D1-D5 | 完成；`3a3c9c55`；Windows/WSL2 secret writer、migration、ccr-db、CLI/Codex、lint/workspace tests |
| `07-24-audit-process-gateway` | P1-06, P1-07, P2-01, P2-02, P2-08, P2-09, P3-04 | P1 | A2, A3, A5 | 完成；`e5892e04`；gateway/Windows process tree/Tauri/frontend/lint/workspace tests；PR #42 Linux/Windows/macOS hosted 通过 |
| `07-24-audit-ci-governance` | P1-11, P2-03, P2-04, P2-15, P2-16, P2-17, P3-03 | P1 | E1-E3, E5-E7 | 完成；`691fd0d5`、`bb46226b`、`7e7c4514`、`158b007c`、`6951839f`、`09acd6f2`、`133842b3`；PR #42 四个稳定 contexts 与跨平台矩阵通过；`main`/`dev` strict required protection 已配置并回读 |
| `07-24-audit-typed-ipc` | P2-11, P2-12 | P2 | A4, E4 | 完成；实现 `3de89558`、证据 `b381e1ad`、归档 `f8201d42`、journal `de6deaf1`；metadata 315/315、typed 252/315 (80.00%)、精确单一声明 252/252，runtime policy/ACL/confirmation/timeout ownership 已闭环 |
| `07-24-audit-release-signing` | P2-14 | P2 | E8 | 仓库侧完成；实现 `d2cabc6a`、证据 `07f8b12f`、journal `94eda6d0`；fail-closed workflow、SBOM/provenance wiring、updater freeze、验证文档和 PR #43 托管回归通过；release environment 的 secrets/variables 均为空，真实 Apple/Windows/VSIX 身份与签名产物仍 `UNVERIFIED`，保持未归档 |
| `07-24-audit-p3-cleanup` | P3-01, P3-02, P3-05 | P3 | - | 完成；`a4e9dd3f`；public API/doctest、dependency/JSON、migration、fmt、lint、workspace tests；`version-check` 仅被并行 README 版本事实阻塞 |

### 建议执行顺序

1. **发版阻断组（先行）**：install-plan-handle → ssh-hardening → webdav-sync → persistence-migration（对应报告"阶段 0 Release Blockers"）
2. **稳定性组**：process-gateway、ci-governance（ci-governance 中 frontend-ci 加 dev、工具链 pin 等 quick win 可最先做）
3. **中期组**：typed-ipc、release-signing
4. **收尾**：p3-cleanup

子任务间无硬依赖，可独立验收；typed-ipc 建议在 install/sync/ssh 整改后进行以避免迁移返工（顺序约束已写入其 prd.md）。

## Requirements

- 每个子任务的修复必须附带审计报告第 9 节对应的 security regression tests
- 修复不得破坏现有行为契约：secret masking、backup-before-destructive-change、文件锁、原子写（CLAUDE.md 既有规则）
- 所有子任务遵循 Conventional Commits，scope 对应所改 crate/模块

## 35 条发现整改证据矩阵（2026-07-27 interim）

`PASS` 只表示对应审计发现已有代码与本地回归证据；需要 hosted、跨平台或真实身份的行在权威证据取得前保持 `PARTIAL`/`UNVERIFIED`。完整命令明细由对应子任务 PRD 的 Verification Evidence 持有。

| ID | 工作提交 | 回归 / 量化证据 | 当前状态 |
|---|---|---|---|
| P1-01 | `b444b459` | forged/expired/reused/cross-host plan；renderer 只持 opaque plan ID | PASS |
| P1-02 | `19cef4b2` | hostile remote-home/path、OpenSSH argv 与 SFTP-only fixture | PASS |
| P1-03 | `0e58e9e9` | traversal/encoded traversal/Windows separator 与 containment fixture | PASS |
| P1-04 | `0e58e9e9` | stage→validate→fsync→swap；失败恢复 active/backup | PASS |
| P1-05 | `3a3c9c55` | WSL2 umask/mode 与 Windows real-file DACL preservation | PASS |
| P1-06 | `e5892e04` | ProcessGateway timeout/output cap；PR #42 Tauri Linux gate 与 Windows/macOS process smoke 全部成功 | PASS |
| P1-07 | `e5892e04` | bounded delta、独立 drop 计数、≤20 Hz、producer>consumer soak | PASS |
| P1-08 | `3a3c9c55` | row decode/malformed JSON/UPDATE trigger/postcondition/accounting | PASS |
| P1-09 | `19cef4b2` | new/match/mismatch host key、真实握手与 app-owned known_hosts | PASS |
| P1-10 | `0e58e9e9` | HTTPS/loopback policy、credential scope 与 redirect tests | PASS |
| P1-11 | `691fd0d5`, `bb46226b`, `7e7c4514`, `6951839f`, `133842b3` | PR #42 四个稳定 required contexts 全部成功；`main`/`dev` strict protection 绑定 app `15368` | PASS |
| P2-01 | `e5892e04` | OAuth port ownership registry；unknown PID report-only | PASS |
| P2-02 | `e5892e04` | exact OpenAI authorize endpoint + loopback callback allowlist | PASS |
| P2-03 | `691fd0d5`, `7e7c4514` | VS Code clean CI 50/50 + package build；PR #42 `VS Code Required` 成功且已 required | PASS |
| P2-04 | `691fd0d5`, `7e7c4514` | workflow policy 覆盖 main/develop/dev；dev push 与 PR #42 frontend hosted 均成功 | PASS |
| P2-05 | `0e58e9e9` | sync 四组合 × force truth table | PASS |
| P2-06 | `0e58e9e9` | depth/entry/byte/response/redirect limits 与 adversarial DAV fixtures | PASS |
| P2-07 | `0e58e9e9` | 双状态 staged commit/rollback 与失败补偿 tests | PASS |
| P2-08 | `e5892e04` | Windows descendant termination/reap；PR #42 Linux/Windows/macOS process gates 成功 | PASS |
| P2-09 | `e5892e04` | absolute sidecar + compile-time SHA-256；PATH precedence/spoof fixtures | PASS |
| P2-10 | `3a3c9c55` | v3-v5 transaction + v16 repair；DDL rollback/marker idempotence | PASS |
| P2-11 | `3de89558`, `b381e1ad` | capability metadata 315/315；Tauri permissions 323；confirmation/ACL、module/singleton permit、queue/cooperative/completion-aware/business timeout ownership 全部有 runtime enforcement | PASS |
| P2-12 | `3de89558`, `b381e1ad` | typed 34/315 (10.79%) → 252/315 (80.00%)；typed exact declaration 252/252；typed boundary `Value`=0 | PASS |
| P2-13 | `b444b459` | UUID transparent aliases 与 generated binding drift tests | PASS |
| P2-14 | `d2cabc6a`, `07f8b12f` | repo-side release security 6/6、PR #43 四条 required contexts PASS；release secrets/variables=0，真实签名/attestation artifact 不存在 | UNVERIFIED |
| P2-15 | `691fd0d5`, `09acd6f2` | serial-only 0；Rust 70.10%、gateway 93.20/95.57%、Vue 74.54%、VS Code 91.79%；hosted coverage 成功 | PASS |
| P2-16 | `691fd0d5`, `133842b3` | Rust/Bun/Node/actions pinned；PR #42 同 SHA 四 workflow 成功，Tauri Linux 固定 Bun 1.3.10 | PASS |
| P2-17 | `691fd0d5` | dependency/MSRV drift gate；19 repeated dependencies、1 active exception | PASS |
| P2-18 | `3a3c9c55` | parent fsync + crash/replace failure tests | PASS |
| P2-19 | `0e58e9e9` | encrypted envelope v2、wrong key/tamper/no-plaintext tests | PASS |
| P3-01 | `a4e9dd3f` | seven 7.x compatibility paths deprecated; public API 3/3, doctest 10/10 | PASS |
| P3-02 | `a4e9dd3f` | ownership/decomposition spec + umbrella dependency guard | PASS |
| P3-03 | `691fd0d5`, `b381e1ad` | generated registry/docs 315 base / 323 Windows | PASS |
| P3-04 | `e5892e04` | VecDeque bounded channel storage replaces `remove(0)` | PASS |
| P3-05 | `a4e9dd3f` | UTF-8 migration comments + repository JSON formatting gate | PASS |

## Before → Target 量化证据（2026-07-27 interim）

| 指标 | Before | Target | 当前证据 / 状态 |
|---|---|---|---|
| Renderer executable capability | 完整 `command/args/envs` 可回传执行 | renderer 可执行字段 0 | opaque plan handle；PASS |
| SSH trust closure | 未握手即 connected；`accept-new` | app-owned trust + real handshake 100% | focused SSH/Tauri tests；PASS |
| DAV traversal/resource bounds | containment 与总量上限 0 | hostile href 100% reject；显式 depth/entry/byte caps | adversarial DAV suite；PASS |
| Sensitive writer/migration durability | mode/ACL/parent fsync/事务不完整 | secret writes 与 marker commit 全部 fail-closed | WSL2 + Windows + historical DB fixtures；PASS |
| Process lifecycle | timeout/output/tree ownership 无统一边界 | foreground 60s/1 MiB；events ≤20 Hz；owned tree cleanup | local + PR #42 Linux/Windows/macOS PASS |
| Hosted product checks | 2/4 产品面 | 4/4 且 protected required | PR #42 四 contexts PASS；`main`/`dev` strict protection PASS |
| Coverage | 未测量 | Rust/Vue/VS Code lines ≥70%；gateway ≥85% | 70.14/74.69/91.79%；93.20/95.57%；PASS |
| Command metadata | 0/315 | 315/315 | 315/315；PASS |
| Typed IPC | 34/315 (10.79%) | ≥80% | 252/315 (80.00%)；runtime policy/ACL/confirmation/timeout ownership 全部闭环；PASS |
| Serial-only tests | 全局 `--test-threads=1` | 0 blanket serial annotations | 0；PASS |
| Dependency exceptions | 未结构化/hosted 漂移 | owner/rationale/expiry，active ≤3 | 1；PASS |
| Release identity | checksum-only；真实签名 0 | macOS/Windows/VSIX signed + provenance verified | repo DAG PASS；真实 artifact 0，`UNVERIFIED` |

## Integration validation checkpoint (2026-07-27, local through `94eda6d0`; hosted PR #42/#43)

| Gate | Result |
|---|---|
| Child security regression targets | PASS：install 60；WebDAV 64；atomic writer 9；migration 16；Tauri SSH 26、sync 33、command gateway 33、handler registry 20、runtime policy 3；frontend focused 12 files / 46 tests；usage focused 14/14 |
| `just fmt-check` / `just lint-strict` / `just test` | PASS |
| `just frontend-check` / `just ui-check` | PASS：104 files / 464 tests；type-check、lint、build、docs audit/build |
| `just vscode-ci` | PASS：50/50；development VSIX package generated |
| Coverage | PASS：root 70.14%、root gateway 93.20%、Tauri baseline 41.40%、Tauri gateway 95.57%、Vue 74.69%、VS Code 91.79% |
| `just ci` | PASS：12 stages，03:53.493；含 release build、Rust audit、governance、bindings drift、104/464 frontend、docs 和 VS Code packaging |
| `just version-check` | FAIL：版本 7.0.0 全部一致；排除范围 `ccr-ui/README.md` 缺少 `version-7.0.0` 事实 |
| Hosted PR matrix | PASS：PR #42 head `133842b3` 验证 CI governance；PR #43 head `94eda6d0` 验证最终 Typed IPC 集成，Root `30259859698`、Tauri `30259859694`、Frontend `30259859557`、VS Code `30259859538`；四条 required contexts 与 Tauri Linux/Windows/macOS、gateway coverage 全部成功；按 relevance policy 跳过的 Root/VS Code heavy jobs 未被冒充为执行 |
| GitHub branch protection | PASS：`main`/`dev` protected；strict checks、admin enforcement；四 contexts 绑定 app `15368`；force-push/deletion disabled |
| GitHub `release` environment | PARTIAL：environment 存在且仅允许 `v*` tag；repository/environment secrets 与 variables 均为 0；真实 Apple/Windows/VSIX/OIDC artifact `UNVERIFIED` |
| Git boundary | checkpoint 前 index empty；任务外 tracked/version/Trellis/`CLAUDE.md`/`.gitattributes`、6 个 generated whitespace 文件与独立 Vite 任务均未 stage；父任务 6 files 仅进入本证据 checkpoint |

## Acceptance Criteria

- [x] 全部 11 条 P1 关闭或有等价的可验证阻断性 hotfix（发版门槛，见报告 §13）
- [x] P2 中 process/sync/migration/CI 类问题进入连续整改而非零散 patch
- [ ] 每个子任务归档时在本文件任务地图中标记完成状态
- [x] 最终跑通 `just ci` 且新增回归测试全部通过
- [x] 集成复查：按报告 §6 量化指标表逐项核对 Before → Target

## Out of Scope

- 不处理审计报告之外的功能需求或相邻重构
- 不把缺失的跨平台、hosted CI、branch protection 或真实签名证据推断为通过
- 未单独授权时不 push、不发布 release、不修改生产 secrets/证书或远程仓库设置

## Key Decisions

- 2026-07-26 用户选择严格端到端验收：`release-signing`、required branch protection 和真实签名产物在取得相应远程权限、证书/publisher 身份并完成实际验证前保持未完成；不得拆成仓库侧完成后即归档的两阶段口径
- 2026-07-26 用户同意 WebDAV v2 使用同步时输入的独立口令：口令仅在单次同步操作内存中存在，不落盘到 WebDAV 配置或本机 secret store；跨设备通过输入同一口令解密

## Notes

- 审计报告原文：仓库根 `ccr_extreme_code_audit_2026-07-24.md`（基线 `main@9958787`，逐条核实基于 2026-07-24 的 dev 分支）
- P2-08 细节修正：tokio 的 `child.kill()` 会等待并 reap 直接子进程，报告"未等待退出"表述不准；但进程树/孙进程不处理的核心问题成立
- 报告 §11"未能验证项"（reqwest_dav href 归一化、Windows ACL 等）需在对应子任务中先做验证再定实现
