# Research: Tauri 后端调用边界、状态与持久化审查

- Query: 审查 ccr-ui/src-tauri 对 CLI 能力的调用、复用、IPC、错误、锁、任务生命周期和配置安全，提出可执行子任务。
- Scope: internal；只读源码/规范；仅写本研究文件。
- Date: 2026-09-28
- Active task: .trellis/tasks/09-28-cli-tauri-architecture；保持 planning。
- Evidence labels: confirmed 为当前源码确认的结构/分支；inferred 为明确调度顺序推导的运行后果；untested 为未对真实桌面、外部 CLI、OAuth 或用户配置执行。

## Findings

### 1. 结论和调用分类矩阵

Tauri 已有共享 Rust 库、桌面服务、独立进程和桌面专用适配器。重构重点是用例所有权、事务、取消终态和错误契约。不能把全部 Tauri 能力归类为 ccr 子进程调用，也不应把全部能力改成 CLI 文本协议。

| 代表功能 | 实际调用链 | 边界判断 |
| --- | --- | --- |
| 旧配置列表/增改/删除 | commands/config.rs → ccr_config::{ConfigManager,ConfigService}，部分 handler 自行 RMW | 已复用库；需要唯一事务入口。 |
| 旧配置切换 | config.rs:200 → ccr::commands::switch_command → CLI presentation shim | 有效 UI 调用已退役 API；TA-01。 |
| Claude/Codex profile | claude_profiles.rs:295、codex_profiles.rs:231 → PlatformConfig::apply_profile | 库调用，但没有统一 CLI application 的完整结果/历史契约。 |
| profile/auth off | claude_profiles.rs:275、codex_profiles.rs:212、claude_auth.rs:439、codex_auth.rs:1231 → ccr_cli::application | 已有共享应用用例，可作为迁移参照。 |
| Claude 认证 | claude_auth.rs:383-430,464-520 → ClaudeAuthService；同步部分 spawn_blocking | 已复用领域账号服务。 |
| Codex 认证 | codex_auth.rs:1170-1369 → ccr_codex::CodexAuthService/provider store | 账号服务已共享；OAuth pending/listener/token exchange 仍在 command 层。 |
| usage 同步 | usage.rs:438 → llmusage_adapter::run_sync_stream → ProcessGateway → llmusage sync --json-events | 外部 CLI 是当前强制边界；TA-03/04。 |
| usage 查询 | usage.rs:937 → services/usage.rs → llmusage_adapter/db.rs:18-94 → ccr_usage::Dashboard | ccr-usage 继续持有全部 usage SQL；adapter 映射 DTO/error。 |
| llmusage 安装 | commands/install.rs:14-106 → ccr_cli::services::InstallService | 薄 adapter；返回 attempt ID 并转发事件。 |
| 通用 CCR 命令工作台 | command_exec.rs:1797-1850 → request validator → sidecar version check → ProcessGateway → CCR sidecar | 保留 allowlist、确认、hash、输出上限；高频功能采用 typed use case。 |
| WebDAV | commands/sync.rs:4-6,576-618 → ccr_sync::SyncService/SyncFolderManager | 已复用同步库；桌面有资产编排。 |
| Local/WSL/SSH | platform/mod.rs:80-103 的 ExecutionEnvironment；claude.rs:37-71 使用环境接口 | 环境能力不是所有 profile/auth handler 自动具备的能力。 |
| 原始配置编辑 | settings_raw.rs:29-48,61-82 → local 检查、content token、versioned guarded write | 保留 CAS 冲突保护。 |

ccr-ui/src-tauri/Cargo.toml:22-44 直接依赖多个工作区 crate，ccr 默认 features 已关闭。crates/ccr/src/lib.rs:161-164 的 commands 是弃用 re-export。现阶段可先收敛已有 ccr-cli::application 和领域 service，不必先新增通用业务 crate。

### 2. 应保留的合理设计

- Registry 同源生成 handler/descriptor/客户端声明：handler_registry.rs:381-400,483-492；不恢复第二份命令数组。
- runtime_policy.rs:34-62 让实际 future 持有 permit；CompletionAware 只约束排队，避免 drop spawn_blocking 后伪造已取消。不能给所有命令添加仅影响响应的 timeout。
- ProcessGateway 有受控 executable、输出限制和进程树所有权；CCR sidecar 禁止 PATH fallback。command_exec.rs:1802-1829 返回 timeout/truncation 元数据。
- 通用 command job 取消只发信号，不立即伪造终态：command_exec.rs:1863-1892。可作为 usage 修复参照。
- llmusage 两流并发消费；stdout 单行 1 MiB，stderr 单行 64 KiB、保留 64 行：llmusage_adapter/cli.rs:16-19,137-151,180-205,220-239。不因使用子进程断言 OOM 或性能差。
- llmusage_adapter/db.rs:4-20,24-94 是 ccr-usage 的薄代理；不链接上游 llmusage Rust crate。
- config_file_handler.rs:111-120 已用 guarded secret 写入。叶子写锁保护原子替换，不替代上层 RMW 事务。
- desktop-command-policy.md 明确允许 WSL 同步文件适配器和 SkillPort detached handoff；不能把所有 std_command 命中泛化为新漏洞。

### 3. 高价值问题

#### TA-01 — P1：有效配置切换/启用入口调用永久失败的旧 CLI shim

- confirmed / 触发：features/configs/hooks/useConfigsPage.ts:55 调 switchConfig；api/domains/config.ts:51 → api/generated/config.ts:26 invoke switch_config。启用配置也经 api/tauri.ts:622-624 走该函数。
- 根因：handler_registry.rs:486 仍注册入口；config.rs:200-204 调 ccr::commands::switch_command；crates/ccr-cli/src/commands/profile/switch.rs:24-26 无分支，直接返回 legacy_switch_error(config_name)。
- 影响：上述 UI 动作不能成功完成。结论来自无条件 Err 调用链；untested：未操作真实配置。
- owner/最小修复：配置 application use case 和 Tauri adapter；明确平台输入，以共享 typed service 替换 presentation shim。不要恢复已移除的隐式全局平台切换。
- 行为验收：真实 Rust handler/service + 隔离 fixture 完成切换，验证 runtime、profile 指针、历史、失败状态；前端切换和启用均覆盖，不能只 mock API 成功。
- 关联：与 TA-02 共享用例子任务；CLI agent 的多文件切换一致性使用同一事务设计，避免重复计分。

#### TA-02 — P1：配置 RMW 存在不同的锁和验证入口

- confirmed：config.rs:221-233,291-303,343-354,509-520 持名为 config 的锁，自行 load/mutate/save；crates/ccr-config/src/services/config_service.rs:91-103 持 ccr_config 和 CONFIG_LOCK。两套入口不能互斥整个 RMW。
- supporting evidence：ConfigManager::save 只是委派（managers/config/manager.rs:135-137）；config_file_handler.rs:111-120 明确调用者负责 RMW。底层原子保存不能保护读到旧值的上层组合操作。
- inferred / 影响：桌面与 CLI/service 并发操作同一文件时，最后一次完整写入可能覆盖另一项修改。untested：未在用户目录复现数据丢失。
- confirmed / 验证差异：service add/update 会 section.validate()（config_service.rs:205-209,227-236），桌面 add/update 直接保存；config.rs:528-558 静默忽略未知 key，非字符串已知字段可映射为 None。
- owner/最小修复：ccr-config 的统一事务 service；桌面构造明确 patch DTO，rename/duplicate 也走同源锁。沿用 update_config 时仍需检查原配置存在、目标重名等约束，不能假定现有 service 已完整。
- 行为验收：隔离双进程/barrier 测试同时经桌面与 CLI/service 修改，两项独立改动都保留；非法字段/类型不改变磁盘；保留 current/default、secret 权限、备份及 unknown fields。

#### TA-03 — P1：usage 取消终态漂移，token 登记存在早期取消窗口

- confirmed / 正常取消：usage.rs:1570-1585 取走 token、cancel、立即 mark_cancelled；llmusage_adapter/cli.rs:153-155 返回普通 Err(Cli(cancelled))；usage.rs:488-493 → sync_llmusage_failure_snapshot:731-740 无条件 mark_failed。一次取消可形成 Cancelled → Failed。
- confirmed / 登记顺序：start 在 usage.rs:1539-1546 发布 snapshot 并 spawn；token 在后台 run_usage_import_job:446-449 才创建。cancel 找不到 token 仍终态化（:1576-1585）；state.rs:733-742 随终态释放 active job。
- inferred / 影响：runner 尚未执行时取消，后续 runner 仍可创建未取消 token 并启动进程；snapshot 已 Cancelled，usage.rs:505-507 忽略后续事件，后台执行不可见。untested：未在原生桌面复现调度窗口。
- confirmed / 终态保护：cancel 对 Finished/Failed 也无条件 mark_cancelled；usage_jobs.rs:164-170 不验证状态转换。
- owner/最小修复：usage job lifecycle service；admission 原子发布 snapshot 和 token；cancel 只请求取消，runner 清理完成后确认 terminal；结构化 cancelled/failed/cleanup_failed 结果，终态幂等。
- 行为验收：延迟 runner 的立即取消断言零次 spawn；running cancel 最终保持 Cancelled；重复/完成后取消不改终态；清理失败可见，active 仅真实结束后释放。

#### TA-04 — P1：llmusage stream 未执行 descriptor deadline，丢失 cleanup failure

- confirmed：ProcessDescriptor::llmusage 在 process/gateway.rs:197-206 声明 1 小时 timeout；spawn:271-279 只给 ManagedChild 传 capability/cancel/ports，不执行 deadline。llmusage_adapter/cli.rs:139-159 等 stdout、child、stderr；consume_stdout_events:220-239 仅 select cancel/line，未使用 descriptor timeout。
- confirmed：cli.rs:141-147 保存 terminate_tree 结果，:153-155 的取消/解析错误分支先返回，绕过 :159 的 cleanup error 传播；:151 stderr join 无独立 deadline。
- inferred / 影响：静默或持管道的进程可超出声明期限；清理失败可能只返回 ordinary cancelled/parse error。untested：未运行 OS 进程树故障场景。
- owner/最小修复：现有 ProcessGateway/llmusage_adapter streaming execution；统一 deadline、cancel、tree cleanup、reader join 结果，避免外围 Promise timeout。
- 行为验收：受控假子进程覆盖静默、stdout EOF 但进程未退出、无换行 flood、stderr 持续占用、cleanup failure 和取消；在期限+grace 内完成并保留诚实终态和流上限。

#### TA-05 — P2：风险推导的全局/模块 permit 阻挡控制命令

- confirmed：所有 ProcessExecution 使用同一 Singleton（handler_registry.rs:257-263；runtime_policy.rs:23,69-81）。manifest 中 execute_ccr_command、start_ccr_command_job、cancel_ccr_command_job、llmusage_install_execute、llmusage_install_cancel 均共享该 permit。usage import/cancel 共享 usage 模块 gate；Codex OAuth completed/cancel 共享 codex_auth gate。
- inferred / 触发：一个前台通用 CCR 运行未完成，用户取消另一个已在后台运行的命令或安装任务，取消调用先排队，不能及时到达 owner。同步 usage import 也能阻挡同模块后台取消。
- 根因：risk、资源互斥、控制操作优先级是不同属性。真实完成后释放 permit 的设计合理，但固定 risk→concurrency 映射没有表达资源所有权。
- owner/最小修复：registry/runtime policy 增加显式资源/操作 override；get/cancel 可直接到达既有 job；后台任务由业务 owner 持 admission，不能只由瞬时 start handler 持锁。
- 行为验收：foreground fixture 用 barrier 持续运行，另一 job 的 cancel 在短期限内到达 token；同资源双 start 仍互斥；ACL/确认不变。untested：未测原生取消延迟。

#### TA-06 — P1：OAuth pending 敏感状态偏离 secret guarded write 规范

- confirmed：pending 含 code_verifier/state/callback_url（codex_auth.rs:39-49）；:518-521 使用默认 AtomicWriter 写入后再 ensure_private_permissions。默认 secret=false（ccr-core/src/core/atomic_writer.rs:158-162）；已有 secret 模式在内容写入前设置 Unix mode/Windows DACL（:165-174,208-238）。
- confirmed：ccr-codex/src/utils.rs:87-111 的权限调整返回 ()，chmod/icacls 失败均被忽略；桌面可能报告持久化成功而未确认权限。该路径没有统一 guarded write 文件锁。
- 影响边界：明确的 secret-write 规范偏离与不可观察的权限失败。untested：没有读取真实 pending、检查本机 ACL 或复现泄露；NamedTempFile 默认权限不能直接作为已利用漏洞证据。
- owner/最小修复：OAuth 状态存储使用已有 write_guarded + secret:true + backup:None，权限初始化失败必须传播；避免复制含 verifier 的状态到备份。
- 行为验收：隔离目录下新建/覆盖、权限初始化失败、持久化失败；Unix owner-only 与 Windows current-user DACL 平台检查；不先返回成功再修权限，不记录 verifier/token。

#### TA-07 — P2：OAuth listener/pending 缺少统一完成责任者

- confirmed：codex_auth.rs 同时持有 DTO、全局 pending mutex、磁盘、端口探测、TCP listener、token exchange 和账号保存。问题依据是下述失败边界，不按文件行数单独评分。
- confirmed：:595-599 探测端口后关闭 listener；:1426-1429 保存 pending 后 spawn 再 bind，并用 let _ 丢弃 listener 错误；:1432 仍返回 auth URL。端口竞争可以留下没有 listener 的 pending。
- confirmed：:671-672 只给 accept 1 秒 timeout，:679-680 已接受 socket 的 read 无 cancel/deadline；cancel :1497-1506 只清 pending。silent socket 会阻止 listener 回到 pending/expiry 检查。
- confirmed：:544-551 先更新内存再持久化，磁盘失败后可不一致；token exchange :773-793 没有显式 request/body deadline，而 completed/cancel 共享 gate（TA-05）。
- owner/最小修复：可注入 clock/storage/http/listener 的 OAuth application service；start 返回前持有实际 listener；任务有 cancel handle；读写 socket 有界；定义内存发布/磁盘失败顺序。桌面事件留在 Tauri adapter，不强制把 callback 搬进 CLI presentation。
- 行为验收：端口占用、bind 失败、silent socket、state 不符、timeout/cancel、磁盘失败、重启恢复；每个 login ID 一个终态；端口/pending 可恢复。untested：本次未执行网络或 native fixture。

#### TA-08 — P2：profile 编排重复，typed 计数尚不等于业务契约覆盖

- confirmed：claude_profiles.rs:99-153 与 codex_profiles.rs:130-168 各自 load/current/check/patch/save-new/delete-old/apply。save 后 delete/apply 失败可能留下部分状态；缺少统一 application owner。inferred：故障后具体文件组合需 fixture 验证。
- confirmed：当前 command-manifest.json 共 348 条；277 generated，71 legacy_json；100 条 generated output 为 OpenJsonValueDto。wire.rs:8-18 的开放 union 合理用于用户 JSON，但不能保证操作固定字段/状态。计数仅描述范围，不单独作为质量缺陷。
- confirmed：command-macros/src/lib.rs:20-38,54-63 强制 Result<T,String>；domain error 常经 format 丢失类型，TA-03 的取消也以字符串错误进入失败分支。
- owner/最小修复：profile lifecycle 用例归 application/service；操作输入/结果用 named DTO，错误分域迁移为 versioned envelope。用户自由配置继续 OpenJson；成功/冲突/unsupported/partial failure 用 enum。先 profile/usage/auth，避免一次破坏 348 个命令兼容。
- 行为验收：同 fixture 下 CLI/Tauri 输出和磁盘后果一致；rename 的 save/delete/apply 点故障注入；current/default、旧名/新名结果明确；macro/生成物/旧调用兼容检查同时通过。

### 4. 需要保留为 caveat 的候选

- claude_apply_profile/codex_apply_profile 等没有 state/environment 输入；raw/profile-off 检查 ensure_local_env。源码确认支持范围不一致，但未穷尽前端 SSH/WSL 可达性，不能直接断言用户误写本地配置。父任务应建立逐入口环境矩阵。
- state.rs:676-681 的 usage job map 直接 insert；main.rs:497-504 周期 prune 只见通用 command jobs。长期快照保留纳入 TA-03；没有内存测量，不声明实际 OOM。
- llmusage_adapter/cli.rs:83-93 的 run_sync_collect 保存全部事件；单行有界不等于总事件有界。可测量后改增量 summary，保留 warnings；不声明已证明的性能瓶颈。
- claude_auth.rs:418-427 用 .ok() 把 service error 折成 missing info，可影响 logged_in；未核实全部 error 变体和 UI，作为分域错误迁移检查点。
- 不把 Arc/RwLock、字符串错误、大文件或独立 Cargo workspace 单独定义为漏洞。

### 5. Files found / code patterns

| 路径 | 一行说明 |
| --- | --- |
| code_map.md；ccr-ui/code_map.md；ccr-ui/AGENTS.md | 导航、责任边界和质量要求。 |
| ccr-ui/src-tauri/Cargo.toml | 当前 workspace crate/Tauri/ts-rs 依赖与 features。 |
| ccr-ui/src-tauri/src/main.rs；state.rs | 服务 wiring、状态、后台维护与 job/token。 |
| ccr-ui/src-tauri/src/commands/config.rs | legacy config RMW、service 与旧 CLI shim 混合入口。 |
| ccr-ui/src-tauri/src/commands/claude_profiles.rs；codex_profiles.rs | 平台 profile 编排及开放 JSON wire。 |
| ccr-ui/src-tauri/src/commands/claude_auth.rs；codex_auth.rs | 认证 service adapter 与桌面 OAuth。 |
| ccr-ui/src-tauri/src/commands/handler_registry.rs；runtime_policy.rs | 单源注册、策略与 completion-aware permit。 |
| ccr-ui/src-tauri/command-macros/src/lib.rs | async/String error 强制和真实 future wrapper。 |
| ccr-ui/src-tauri/src/commands/usage.rs；install.rs；command_exec.rs | 三类不同 long-operation owner。 |
| ccr-ui/src-tauri/src/llmusage_adapter/cli.rs；db.rs；queries.rs | CLI/NDJSON、ccr-usage 投影代理、DTO。 |
| ccr-ui/src-tauri/src/services/usage.rs | State-free usage service 示例。 |
| ccr-ui/src-tauri/src/process/gateway.rs | executable、deadline、进程树、owned registry。 |
| crates/ccr-config/src/services/config_service.rs | 配置验证和 ccr_config RMW lock。 |
| crates/ccr-config/src/managers/config_file_handler.rs | secret guarded write 持久化。 |
| crates/ccr-core/src/core/atomic_writer.rs | 原子替换与内容写入前的敏感权限。 |
| ccr-ui/src/api/generated/command-manifest.json | 当前 348 个 Windows 命令的 wire 和运行策略。 |

### 6. Related specs

- .trellis/spec/ccr/backend/desktop-command-policy.md：ProcessGateway、sidecar、job bounds、cleanup_failed、WSL/SkillPort 例外。
- .trellis/spec/ccr/backend/tauri-handler-registry.md：descriptor/ACL/typed registry/completion-aware；历史 336/348 字样需父任务与当前 manifest 核对。
- .trellis/spec/ccr/backend/typed-ipc-bindings.md：generated DTO、开放 JSON、drift guard；String error 当前为规范约束，迁移必须同步 macro/规范。
- .trellis/spec/ccr/backend/llmusage-provider-adapter.md：上游 no-crate、ccr-usage SQL owner、read-only/schema 能力。
- .trellis/spec/ccr-core/backend/atomic-writer.md：预写 secret 权限、统一锁、调用方 RMW、CAS/backup。
- .trellis/spec/ccr-ui/frontend/environment-scoped-dashboard-contracts.md：active environment/local-only 跨层失效语义。
- .trellis/spec/ccr-ui/frontend/layering-contracts.md：依赖方向，应用编排不回流 React feature。
- .trellis/workflow.md：研究持久化、planning 和实现批准边界。

### 7. External references / versions

本报告使用当前源码、manifest 和项目规范，没有外部博客结论。内部声明为 ccr-desktop 7.3.0、Tauri =2.11.5、tauri-build =2.6.3、Rust 1.95、ts-rs 12.0.1。声明不代表本机安装或构建验证。llmusage adapter 注释中的 0.5.3 是历史注释，本次没有运行已安装 CLI 核验。

### 8. 建议子任务边界与依赖

| 子任务责任 | 问题映射 | 前置和验收 |
| --- | --- | --- |
| 共享 profile/config use case + Tauri adapter | TA-01/02/08 profile；合并 CLI 多文件一致性 | 明确平台输入/事务结果；CLI/Tauri 同 fixture；并发与失败后果；恢复可见切换。 |
| usage lifecycle / stream executor | TA-03/04；快照保留 | admission→cancel request→cleanup→terminal 单一 owner；stream deadline 和 cleanup failure。 |
| command workbench concurrency | TA-05 | 先明确 job owner；控制命令及时到达，同资源 start 仍互斥。 |
| safe persistence / OAuth | TA-06，TA-07 存储部分 | secret guarded write、权限失败传播、memory/disk 失败策略。 |
| OAuth lifecycle | TA-07 listener/http 部分 | 协调 concurrency；端口/socket/task 可取消、终态唯一；可作为 auth 子任务或明确后续范围。 |
| frontend query/error + typed boundary | TA-08 error/wire | 先明确 service 结果；分域 named operation DTO/error；保留自由配置内容。 |
| gates / cross-surface integration | 全部与环境 caveat | Local/WSL/SSH 支持矩阵，真实 Rust fixtures，原生权限/进程验收，最终门禁。 |

优先恢复断路和配置事务，并行处理取消/超时与 OAuth 敏感写入；再完成资源并发、typed result/error 与跨层 UI 消费。任务全部保持 planning。父任务负责兼容边界和最终集成，不能用拆文件数量替代行为验收。

## Caveats / Not Found

- 本次仅执行源码/规范读取、搜索和 manifest 统计；没有执行 cargo test、Tauri/浏览器、真实 llmusage、OAuth 网络或用户配置操作；没有修改产品代码、规范、任务状态，没有执行 Git 操作。
- TA-01 无条件失败、TA-02 锁名差异、TA-03 Cancelled→Failed 已由源码确认。竞态、OS 权限、进程树、原生 UI 仍需列出的 fixture/平台验收。
- 初期路径不存在、PowerShell regex、UTF-8 解码错误已改用实际文件列表和 UTF-8 读取；未当作产品故障。
- 主线程负责整体 version/fmt/governance/type/lint 等检查；本报告不预先声称通过。
- 没有找到 switch_command 的成功分支、usage stream 使用 descriptor.timeout 的执行代码。该结论限定在本报告列出的已审查入口，不泛化到其他域。
