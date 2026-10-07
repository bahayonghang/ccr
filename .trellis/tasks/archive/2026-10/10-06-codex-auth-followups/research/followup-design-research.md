# Research: Codex Auth P1/P3/P4/P5/P6 实施设计

- Query: 补齐父任务 10-06-codex-auth-followups 的 P1、P3、P4、P5、P6 技术方案、文件白名单、验证入口与未决风险。
- Scope: internal；外部参考仅采用已归档审计，未联网复核。
- Date: 2026-10-06
- 写入边界: 本研究的原生 Active task 为 10-06-auth-registry-schema。主会话已确认先将报告写入本任务 research/，再由主会话整合至父任务。研究未修改产品、规范或规划文件，未执行 Git 操作。
- 证据边界: P2 正在并行修改账号模型、注册表和账号服务。本报告行号来自研究时读取的工作树；函数名为稳定定位依据，进入后续实施前重新核对行号。未读取真实凭据，未运行编译或测试。

## Findings

### 需求状态

- P1: 用户已确认完整 user_id::account_id 无法取得时跳过同步。旧记录先从各自已保存 snapshot 补全；相同完整身份保留 current_auth 优先、last_used 次优。
- P3: 保留 10 份或暂不清理仍等待用户选择。数量缩减和删除前留存凭据不由研究代替用户决定。
- P4: 按既有目标设计 SID 权限、Unchanged 元数据加固、私有原子 rename 回退、Debug 脱敏及错误响应体消除。必要的 ccr-core 权限 API 应进入明确白名单。
- P5: 本轮交付评估，不实施快照布局迁移。
- P6: 身份冲突处理等待用户选择。主会话已确认保留 CLI 现有含凭据导出的强制加密规则，不新增 TUI/Tauri 导出 UI；只处理选定的导入身份策略和必要 CLI 错误传播。

### Files found

| 路径 | 用途 |
| --- | --- |
| .trellis/tasks/10-06-codex-auth-followups/{prd,design,implement}.md | 父任务依赖、实施授权及集成门；顺序为 P2→P1→P3→P4→P5→P6。 |
| .trellis/tasks/10-06-auth-{identity-key,destructive-backup,defense-in-depth,snapshot-naming,import-export}/prd.md | P1/P3/P4/P5/P6 初始要求与 AC；未决项仍需收敛。 |
| .trellis/tasks/archive/2026-10/10-06-reference-hardening/research/codex-auth-audit.md | F10–F24 原始审计及已修复范围。 |
| crates/ccr-codex/src/models/codex_auth.rs | 账号注册表、当前身份、认证 tokens、导入结果和加密格式。 |
| crates/ccr-codex/src/services/codex_auth_service.rs | 保存、显示匹配、current_auth 对账、同步执行、删除、重命名、备份与导入导出。 |
| crates/ccr-codex/src/services/codex_oauth_token_service.rs | runtime/snapshot 新鲜度规划、目标选择、runtime/backups 修复与合成测试。 |
| crates/ccr-codex/src/services/codex_quota_service.rs | 活动账号凭据路由、refresh 回写、repair、loopback OAuthStub。 |
| crates/ccr-codex/src/services/openai_quota_core.rs | JWT 解码、共享 quota cache、HTTP 错误、刷新/修复分类。 |
| crates/ccr-codex/src/services/codex_registry_store.rs | P2 兼容门、注册表文件锁、自动备份和私有原子写。 |
| crates/ccr-codex/src/managers/codex_config.rs | 已验证的最新内容去重、同秒序号与 10 份保留合同。 |
| crates/ccr-codex/src/utils.rs | ensure_private_permissions；当前 Windows 分支依赖 USERNAME 和 icacls。 |
| crates/ccr-core/src/core/{atomic_writer,guarded_write,write_journal}.rs | 私有写、SID DACL 构造、版本校验元数据函数与测试故障注入。 |
| crates/ccr-cli/src/commands/codex/auth/{export,import,current}.rs | CLI 加密默认、明文导入错误传播及不含完整身份键的 JSON DTO。 |
| crates/ccr-tui/src/tui/codex_auth/{app,ui}.rs | repair 调用、quota 错误本地化、EN/ZH 与尺寸测试。 |
| ccr-ui/src-tauri/src/commands/codex_auth.rs | 明文 bundle/单对象/数组导入及显式 DTO 映射。 |

### P1: 完整身份、旧记录与所有匹配边界

#### 现有模式与缺口

1. codex_auth_service.rs:370 的 extract_jwt_claim 只读 id_token 顶层字段；:388 的 decode_jwt_claims 可解码任一 JWT。:432 的 resolve_account_id_from_auth 先取 tokens.account_id，再处理 API/provider 指纹，随后回退 id_token.sub 或 access token 指纹。该显示兼容逻辑不能用作完整 OAuth 身份推导。
2. codex_auth_service.rs:683 的 matched_saved_account_name 与 :728 的 sync_current_auth_registry 只比较 account_id，取 IndexMap 中第一个匹配项。登录态显示和 current_auth 对账也必须修复，否则同步选择修复后仍可能显示或激活另一个用户。
3. codex_oauth_token_service.rs:409 的 plan_runtime_sync 只读规划；:516 的 select_sync_target 以 account_id 筛选，current_auth 优先、last_used 次优，平手保留插入顺序。:540 的 tokens_account_id 优先 tokens.account_id，否则访问 JWT。:669 的 JWT 提取器支持顶层 chatgpt_account_id/account_id 和 auth 命名空间下 account_id，目前不支持命名空间下 chatgpt_account_id。
4. 修复同样存在跨用户风险：codex_oauth_token_service.rs:243 的 resolve_latest_oauth_doc 与 :570 的 repair_saved_account 只按 account_id 过滤 runtime/backups。即使 plan_runtime_sync 修复，repair 仍可能选中共享 workspace 的另一个用户。
5. codex_quota_service.rs:499 的 route_active_account_to_runtime 在 best-effort 同步前读注册表，之后仍只比较 runtime_account_id 与旧 account_id。懒补全后需重新取得有效身份，避免使用同步前的旧字段。
6. openai_quota_core.rs:654 的 cache_key 优先 account_id，缓存结果含 email（:234–237）。只修复 HTTP 凭据路由仍可能复用另一用户的缓存结果。建议在 P1 同时按完整身份隔离内存 quota cache；身份不完整时使用凭据指纹。HTTP ChatGPT-Account-Id、既有 usage ledger 和 quota observation 的 account_id 不在本轮迁移。

#### 建议合同

- 新增专用内部 OAuthIdentity，包含非空 user_id/account_id；完整键只用于内存匹配和注册表，Debug 隐藏键内容。向 CodexAuthAccount 增加 identity_key: Option<String>，serde default + skip None，接上 P2 的未知字段保留。保持原 account_id 兼容字段，不把完整键写入公开 DTO、日志或错误。
- 在 ccr-codex 新增窄共享解析 helper（建议 services/codex_auth_identity.rs）。读取 id_token/access_token 的顶层和 https://api.openai.com/auth 下 chatgpt_user_id、chatgpt_account_id；account context 保留 tokens.account_id 优先的既有顺序。只接受字符串并 trim；空值、非 JWT、坏 base64/JSON、互相矛盾的有效 user claims 返回 None。未找到已核实的 user_id 生产 parser，不应静默把任意 sub/email 当 user_id。
- JWT 解码是本地一致性观测，不能描述为签名认证。合成测试可使用无签名 JWT；不联网验证，也不读取真实账户。
- API key/provider 继续沿用现有指纹分支，identity_key=None。缺少 OAuth 完整身份只限制 OAuth 同步/repair/runtime quota 路由，不取消已有 API key 保存与切换能力。
- 旧记录仅从该记录自己的 snapshot 推导完整身份。不能借 runtime.user_id 或同 workspace 其他记录补全。坏/缺 snapshot 保持 None；已保存 identity_key 与 snapshot 完整身份冲突时跳过自动写回并记录固定诊断，不输出键。
- read_auth_snapshot 和 plan_runtime_sync 保持只读。将 identity backfill 放到允许写入的执行观测点入口；先遵守 P2 的只读版本门，再保存完整 snapshot 推导出的 delta。只读展示可临时在内存推导身份。不要在只读 planner 中增加注册表写入。
- 从完整匹配集合选择 current_auth，否则最大 last_used，平手保留 IndexMap 顺序。选中后再次验证 snapshot 的完整身份与 runtime 相同，才比较 tokens 和新鲜度。完整身份缺失/不匹配返回 NoOp；此路径不得覆盖 token 文件。
- 保存/允许的导入按同一 helper 写入注册表身份键。repair 的所有 runtime/backups 候选和 sync_account_auth_file 入口需要完整身份约束。读取旧 snapshot 的缺失字段可内存补全；不要把过期 token、文件 mtime 或邮箱当作身份。
- 保留现有 freshness、runtime 其他字段、非 file store NoOp、switch-out 回写、auth off 同步和重新登录提示合同。

#### 文件白名单

- crates/ccr-codex/src/models/codex_auth.rs
- crates/ccr-codex/src/services/{mod.rs,codex_auth_identity.rs,codex_auth_service.rs,codex_oauth_token_service.rs,codex_quota_service.rs,openai_quota_core.rs}
- 仅账户结构字面量/合成 fixture 补字段：crates/ccr-codex/src/services/{codex_registry_store.rs,codex_usage_estimation.rs}、crates/ccr-codex/src/platforms/codex.rs、crates/ccr-tui/src/tui/codex_auth/ui.rs。
- 若 CurrentAuthInfo 增加内部 identity 字段，补所有对应 Codex 结构字面量；同名 Claude 类型不在范围内。crates/ccr-cli/src/commands/codex/auth/current.rs:57 的 DTO 映射保持原字段，ccr-ui 显式 DTO 同理。
- 不修改公开导出格式、usage ledger、observation schema、生成 TypeScript、快照文件名或真实凭据。

#### 验证入口

- codex_auth_service.rs 的 create_test_auth_json（:2081）只有 sub，oauth_auth_json（:3907）只有 synthetic-id 和 account_id。成功同步用例需要完整合成 user claims；原缺身份样例保留为独立 NoOp 测试。不能通过恢复仅 account_id 匹配使旧 fixture 通过。
- OAuth token 服务既有 resolve_latest/repair 测试入口 :723、:767、:841。新增同 account_id 不同 user_id 的较新 runtime/backup，证明另一用户 token 不被修复或同步。
- quota_service.rs:941 的 live_access_token、:949 的 expired_access_token、:975–981 的 stub refresh 响应、:1107 的 E2eFixture::write_runtime 都要携带合成 user identity，并在 refresh 后保持身份；可将 stub state 扩为 user/account pair。
- 新增矩阵：顶层/命名空间 claims；id/access fallback；空/坏/冲突 claims；相同 workspace 不同用户双向同步；旧条目补全不改 name/current_auth；重复完整键优先级；Unchanged bytes/mtime；P2 只读注册表不补全、不写 token；quota 路由和缓存用户隔离；API key 兼容。
- 相关回归入口：sync_does_not_write_when_tokens_match、sync_identity_rules_never_write_another_account、sync_with_duplicate_account_id_targets_current_then_latest_used、switch 写回/失败一致性、quota e2e_rotation_*。

### P3: 去重备份、保留池与破坏前置条件

#### 可复用实现

- managers/codex_config.rs:237–277：读取源字节；仅与该池最新备份比较；相同时刷新既有备份 mtime 并返回路径；命中不清理。新备份通过 AtomicWriter.secret(true) 写入。
- :281–297：同秒文件名已存在时追加 _N；:300–320：按 mtime/name 排序；:326–337：保留池 10 份。该 helper 为私有且绑定 runtime backup_dir，不能直接把 CCR auth/backups 送入同一 runtime 保留池。
- codex_registry_store.rs:113–157 的 save/backup 仍忽略自动备份失败、秒级名称+fs::copy。codex_auth_service.rs:1324 的 backup_registry 另有重复实现；:1344 的账号备份已私有原子写，但仍同秒覆盖。
- delete_account（auth_service.rs:1290）直接删除 snapshot；rename_account（:1480）先备份/删除冲突目标，再忽略源 snapshot/registry 备份失败。必须把全部必需备份移到任意删除或移动之前。

#### 建议合同与顺序

- 新增 ccr-codex 私有共享 backup helper（例如 services/codex_auth_backup.rs），供 registry_store 与 auth_service 使用；auth_service.backup_registry 委托 registry_store.backup。保持现有备份根目录与可恢复原始字节。
- 池分为注册表 1 池、每个账号各 1 池。不能用 auth_ 或 auth_account_ 宽前缀旋转，否则用户 A 的历史可删除用户 B 的备份。现有名字可保留；账号名 foo/foo_bar 的池过滤必须匹配完整文件名结构，不能仅 starts_with(auth_account_foo_)。旧文件只在准确归属后参与该池。
- 采用最新相同内容去重、同秒序号、私有原子写。去重返回真实存在且字节相等的路径；读备份或更新时间失败时创建新备份。身份键和凭据不进入备份名称；备份名称仍按安全账号别名。
- 同秒序号选取和创建需在该池的操作锁内完成。现有 exists→AtomicWriter 是 TOCTOU；只序号扫描无法在两个进程并发时保证不覆盖。registry save 已持注册表锁；确定操作锁→backup 池锁→leaf writer 顺序，独立 backup 同样使用池锁，避免递归同一锁。
- 删除：加载/验证可写注册表→验证账号→备份注册表与存在的 snapshot→全部成功→删除 snapshot→移除账号及必要 current_auth→保存。源 snapshot 确实不存在时可返回 None；存在但读/写失败必须中止删除。
- force rename：验证源/目标/版本门→备份存在的目标 snapshot、源 snapshot、原 registry→全部成功→才删除目标、移动源、修改内存 registry 和保存。任何一个备份失败时源/目标 snapshot 和 registry 保持原字节；已成功创建的备份可以保留。
- 自动 registry save 是否将 self.backup() 错误改为传播，需要在设计明确。建议可靠备份为前置条件；不要把失败继续写入描述为已满足 backup-before-destructive-change。
- P3 的 fail-closed 保证范围为备份失败前不破坏。文件移动或最终 registry 保存失败仍可能部分完成，不能宣称多文件事务。若要求补偿，应单独列出最小回滚机制和验证，不自动扩大为通用事务层。
- 保留数量等待用户选择。选择 10 时每池保留 10 份，并明确删除账号的 backup 也属于凭据留存。选择暂不清理时实现去重/防覆盖/私有写和失败中止，保留清理不运行，PRD 不再声称本轮有界保留。

#### 白名单与验证

- services/{mod.rs,codex_auth_backup.rs,codex_registry_store.rs,codex_auth_service.rs}；必要时对应 CLI/TUI EN/ZH 固定失败提示和断言。无需改 runtime CodexConfigManager 的既有保留合同。
- 测试覆盖：连续相同内容路径复用；固定同秒不同内容；多线程/独立进程不覆盖；不同账号及 foo/foo_bar 池隔离；注册表/账号 10 份或不清理的选择；删除前可恢复相同字节；源/目标/registry 逐一备份失败均不删除任何 snapshot；目录变文件的确定性 I/O 失败。
- 采用 tempfile/TestCodexEnv；失败注入限测试 closure/局部函数，不新增全局环境开关。Unix owner-only 与 Windows 新备份 DACL 分平台验证。

### P4: SID 权限、元数据版本校验与错误脱敏

#### 真实 API 边界

- utils.rs:87–118 的 ensure_private_permissions 返回 unit，Windows 依赖 USERNAME + icacls 并忽略失败。全仓主要生产调用点位于 registry_store、oauth_token、quota、auth_service 和 model_provider_store；更改函数签名时必须完整调整调用者。
- atomic_writer.rs:622 的 secret_windows_dacl 是私有函数；:699/:737 的 capture_windows_dacl/apply_windows_dacl 仅 pub(super)。没有供 ccr-codex 直接调用的公开 SID DACL 加固 API。
- atomic_writer.rs:623–625 对已存在目标捕获原 DACL。:632–655 才为新目标从进程令牌用户 SID 构造 D:P(A;;FA;;;SID)。不能把 secret(true) 描述为已有宽权限 DACL 的自动收紧。
- guarded_write.rs:154 的 enforce_secret_permissions_versioned 在 leaf lock 下读取句柄并验证 content_version_token；:172–186 仅 cfg(unix) 改权限。Windows 成功返回保留原 DACL。现有 spec 也明确要求保留 Windows DACL（atomic-writer.md:146）。

#### 建议最小设计

- 新增明确的 owner-only 权限能力，复用当前进程 SID 构造代码；不把 AtomicWriter 的既有 preserve-DACL 合同整体改成删除其它 ACE。可在 ccr-core 提供窄 owner-only metadata helper，以及带预期 content token 的版本校验入口；原 preserve-policy helper 保持原合同。
- 权限操作使用经验证的文件句柄。Windows 只设置 protected DACL，不替换文件、不写 payload、不启动外部进程。缺失/内容版本变化返回 false；权限失败返回现有 CcrError。Unix 保留更严格 owner-only mode（如 0400），避免原 utils 直接 chmod 0600 放宽只读。
- 现有 ensure_private_permissions 的 unit 签名可保持兼容，内部失败至少记录固定权限诊断；P4 的关键安全执行路径使用 fallible helper 并传播失败。若统一改为 Result，则把 model_provider_store 等所有受影响调用点纳入白名单。主会话需要选定一种方式，不能默默吞掉失败。
- plan_runtime_sync 保持只读；Unchanged 的执行分支在 auth_service 和 sync_runtime_tokens_to_saved_account 都加固观测到的 runtime/snapshot。以已读原始文件字节生成内容版本，不用重新序列化 JSON 生成 hash。内容改变时返回安全 NoOp/重试，不加固错误文件；registry 只在 P1 backfill 有 delta 时保存。
- rename 回退在 fs::rename 失败后读取源字节→私有 AtomicWriter 写目标→写入成功后才 remove source。备份前置由 P3 保证。Windows 新目标有私有 DACL；已有目标需要明确强制 owner-only policy，不能依赖 preserve-DACL 替换。
- 无需构造随机跨卷/锁定情况。将 snapshot move 提取为窄内部函数，rename 操作为可注入 closure；生产传 fs::rename，测试传确定 Err。验证原子回退成功、写入失败源仍存在、目标原字节不被截断；源删除失败报告错误并保留已备份恢复路径。无需公开 trait 或生产故障开关。
- AtomicWriter 的 publish 后 sync 失败可能返回 Err 且新目标完整字节已可见（atomic_writer.rs:307–318）。此时源保留满足可恢复性，不能断言所有 Err 都代表目标未替换。只有 publication 前失败才能断言目标旧字节不变。

#### Debug 与 HTTP 错误

- models/codex_auth.rs:582 的 CodexAuthJson、:598 的 CodexAuthTokens 自定义 Debug，所有 API key/id/access/refresh 字段只输出有无或固定 [REDACTED]。Serialize 磁盘格式保持原样；本轮不把 token 字段改为 Secret 后破坏明文磁盘 DTO。
- OAuthIdentity/CodexAuthAccount 的新增键需相同 Debug 防护。ResolvedOAuthDoc 可沿用已脱敏 tokens Debug；RuntimeSyncPlan::WriteRuntime 的原始 JSON Map 不会通过 token 模型 Debug，因此需要独立隐藏 auth map（oauth_token_service.rs:55–69）。
- openai_quota_core.rs:447–455 与 :487–495 移除 body_preview 文本，仅状态码和固定允许的错误码。extract_error_code（:611）当前接受任意 code 字符串；只做字符过滤仍可泄漏恶意 code，需要 allowlist：保留现有 token_invalidated、refresh_token_reused、refresh_token_invalidated、refresh_token_expired、invalid_grant 等已参与分类的值，未知值不回显。
- decoder 可继续内部读取 body 以取得 allowlisted code；兼容 detail.code、error.code、顶层 code，以及 OAuth error 为固定字符串的形状。不回显 message/body/JSON 值。
- should_force_refresh（:389–393）识别 token_invalidated、401 及 authentication token has been invalidated 文本。移除响应体可能改变原 403+该短语的重试行为；先内部把已知固定诊断映射成 token_invalidated，或明确审阅行为变化。should_repair_tokens（:397–402）的四种永久 refresh code 与 RELOGIN_REQUIRED_PREFIX（quota_service.rs:22）必须保留。
- TUI quota_status_line 消费重新登录 marker，ui.rs:3431 的 relogin_quota_error_shows_local_logout_hint_in_both_languages 覆盖 EN/ZH，并保持错误码可见。旧 UTF-8 body_preview 专用测试随删除无用 helper 更新为 response-body 不泄漏测试，保留原 unicode/no panic 覆盖意图。

#### 白名单与验证

- crates/ccr-core/src/core/{atomic_writer,guarded_write}.rs；必要导出所在 core/mod.rs；不修改 CcrError 变体。
- crates/ccr-codex/src/{utils.rs,models/codex_auth.rs,services/codex_auth_service.rs,services/codex_oauth_token_service.rs,services/codex_quota_service.rs,services/openai_quota_core.rs}；若签名传播涉及 services/{codex_registry_store,codex_model_provider_store}.rs，限定为必要调用点。
- crates/ccr-tui/src/tui/codex_auth/ui.rs 及现有 i18n.rs 中真正受影响的固定文本和断言。
- Windows native: TestCodexEnv 持锁后清除 USERNAME，在临时文件设置宽 DACL，调用新函数并读取 ACE，确认 protected、仅当前进程 SID；bytes/mtime/file identity 不变；重复调用无外部进程；权限失败与 content-token 冲突路径。
- Unix native: 0644→0600、0400 保持、bytes/mtime 不变。native 平台缺失标 NOT_RUN；静态或 Windows 测试不能替代 Unix 权限门。
- HTTP synthetic: marker token/邮箱放入普通 message、unknown code、unicode/超长 body；返回错误中均不出现 marker，仍含 HTTP 状态与已知 code；验证 force-refresh、repair、relogin marker。fixture 只用 loopback，不访问服务端。
- Debug synthetic: 两个认证模型、ResolvedOAuthDoc 和 WriteRuntime plan 均无 token/key/identity marker。
- TUI 正式 EN/ZH 矩阵为 80x24、100x22、100x30、120x22、140x40、180x50，并补紧凑尺寸退化；断言 buffer cells/colors。现有 codex_auth::ui 模块另有 70x12、80x12、90x18 等 TestBackend。完整矩阵运行，不以新增单元断言替代尺寸门。

### P5: 快照文件命名评估

- auth_service.rs:187、oauth_token_service.rs:122、quota_service.rs:539 都由账号别名拼接 name.json。validate_account_name（auth_service.rs:1597）限制 ASCII 字母/数字/_/- 和长度、保留 default；审计未发现路径安全缺口。
- 仅用 identity_key.base64url 作为唯一文件名会让相同完整身份的多个别名共享同一快照，而当前别名可各自保存不同 token 新鲜度、描述、api_base_url、provider 配置。需要先决定去重记录、共享快照引用计数和删除语义；不能把 P1 的 deterministic target 直接当作“一身份一文件”的批准。
- API key/provider 没有 OAuth user/account pair；缺身份 OAuth 旧文件也无法命名。不能用密钥明文派生文件名；base64url 可逆，不提供身份隐私。若将来需要与显示别名解耦，随机 opaque snapshot ID 往往比完整身份编码更易支持多个别名，但属于另一个设计选择。
- 迁移涉及三个路径 resolver、rename/delete/force overwrite、导入导出、backup、repair 和外部用户手工恢复流程。需要 manifest、旧/新布局双读、迁移前备份、每项中断恢复、回滚映射、版本门和 unknown-field 保留；文件移动成功不等于 registry 提交成功。
- 建议本轮结论：暂不实施。P1 已修复同步身份混淆，别名文件名在现有校验下无已证实安全缺口；重复别名、API key 和缺身份的迁移成本超过当前证据支持的收益。
- 交付白名单仅 P5 任务评估文件；评估 AC 通过不要求无代码变化时重复 cargo/just 门。报告明确布局迁移、回滚和 native 行为 NOT_RUN。

### P6: 导入身份校验、原子边界与现有加密入口

#### 导入语义

- auth_service.rs:1901 的 import_accounts 顺序写各 snapshot，最后统一 save_registry。现有 ImportResult（models/codex_auth.rs:699）只有 added/updated/skipped/overwritten；没有每项 error/warning。当前实现不是整包事务，也不是已经提交的逐项结果协议。
- :1922–1969 原相邻范围的 force 覆盖会备份后 remove 旧 snapshot；后续单文件 AtomicWriter 不能弥补该 pre-delete 窗口。P6 若改此边界，应明确最小修改并记录为本任务原子安全必要条件。
- Replace 的模型注释为“覆盖同名账号”（models/codex_auth.rs:693），既有 test_import_accounts_replace_mode 保留不在包内的其他账号；不要把 Replace 改成删除所有账号。
- 在创建目录、备份、删除和写文件之前先对全部拟处理条目做身份 preflight。沿用 Merge/no-force 跳过已存在账号的语义；对真正待写项校验名称和有效数据。身份拒绝/坏 JSON 的整个输入 preflight 失败时，磁盘 snapshot、registry 和备份集合均不变。
- OAuth 身份来自 P1 同一 helper；比较导出 account_id 元数据与 tokens 所表达的账户上下文，不相信导出元数据。完整键从 tokens 推导并保存，不添加到公开导出 DTO。缺 user/account 的旧 auth 需明确兼容规则，不能从名称/email 猜测。
- API key 不适用 OAuth 比较；使用既有 API fingerprint 推导并核对模式。metadata-only 输入（auth_data=None）保持既有 metadata 语义，不宣称 token 校验已执行。force 导入无 auth_data 时是否移除已有凭据是现有危险兼容点，应明确保留或另行决定。
- 若用户选“拒绝”：建议全包 preflight 拒绝已知身份不一致，固定错误只含安全账号名和原因；valid-first + invalid-second 测试证明未部分写入。实际文件 I/O 失败的多文件补偿仍另述，不宣称整包原子事务。
- 若选“以 tokens 为准并告警”：先推导标准元数据再写。现有 ImportResult 无警告字段，需要限定 CLI 或新内部报告的警告传递方式；不能只写日志却声称用户已经获知，也不能未审阅就改 Tauri DTO。结果统计应区分跳过和修正，所有错误/警告不得带身份键。
- crates/ccr-cli/src/commands/codex/auth/import.rs:125–137 在 service Err 后打印失败却返回 Ok。选择拒绝策略时，必要修改为返回错误，CLI 的 exit status 才能表达拒绝；既有 encrypted 分支在非密码错误上直接返回 Err（:95–110）。

#### 当前入口评估

- CLI: auth/export.rs:164–175 的 include_secrets 默认走 export_accounts_encrypted；--no-secrets 只走 export_accounts(false)。cli/subcommands/codex.rs:429–432 无明文含凭据开关。已存在强制加密默认，不必新增明文 flag。
- 加密核心: codex_auth_crypto.rs:34–42 采用 Argon2id；:117–121 信封使用 aes-256-gcm/argon2id。Cargo.lock 锁定 aes-gcm 0.11.1（:33–34）、argon2 0.6.0（:136–137）。crypto tests 已有 roundtrip、wrong password、随机 salt/nonce、篡改 metadata、旧版本 fixture；保持格式和 KDF 合同。
- encrypted import 解密后复用 import_accounts（auth_service.rs:1866），新身份 preflight 应同时覆盖明文和加密输入。
- Codex Auth TUI: 源码检索未找到 export_accounts/import_accounts 或 import/export 菜单；repair 调用位于 app.rs:913。没有待切换的默认导出模式。
- Tauri: codex_auth.rs:1073–1079 的 bundle 分支直接调用明文 import_accounts；:1444–1463 的对象/数组逐项提交，每项独立 save，若后项失败前项仍已提交；没有逐项错误 DTO。没有 encrypted-export/decrypt API。仅含 encrypted_payload 的信封目前不能按既有 auth 对象导入。不要把 profile export 接口误认作 Auth 导出。
- 本轮建议维持以上入口形状，交付加密入口评估。若未来要求 Tauri 加密导入或 TUI 导出，需要独立密码生命周期、Secret DTO、取消、错误和 typed IPC 设计。

#### 白名单与验证

- crates/ccr-codex/src/services/codex_auth_service.rs；P1 shared identity helper 的必要校验接口；models/codex_auth.rs 仅在已确认的内部 warning/结果语义需要时修改。
- crates/ccr-cli/src/commands/codex/auth/import.rs 的错误传播和对应测试。export.rs、codex_auth_crypto.rs 保持现有加密行为，仅做必要回归。
- Tauri 无 DTO/API 变更；如果确需逐项 preflight 保证，应先明确 scoped whitelist 为 codex_auth.rs + 合成测试，保持原 Result response shape，不新增加密 UI。
- 合成 JWT: 元数据一致；account_id 不一致；相同 workspace user mismatch（有可核对的已保存键时）；坏/缺 claims；tokens.account_id 与 claims 的冲突；API key；metadata-only；Merge skipped/force、Replace 同名覆盖；两条输入第二条不合法；旧文件和 registry 字节不变；加密输入执行同一校验；CLI Err/exit status。
- 既有 six test_import_accounts_*（merge no-force/force、backup、replace、invalid-name/JSON）保持语义；不要用修改 fixture 元数据隐藏校验缺口。

### Related specs and validation commands

- .trellis/spec/ccr-codex/backend/backend-guidelines.md:245–296：runtime ↔ snapshot 的只读 planner、freshness、current_auth 优先、quota 路由、NoOp、备份命中及 EN/ZH 要求。P1 应写回完整身份；P4 应写回 Unchanged 执行侧元数据加固，保留 bytes/mtime。
- .trellis/spec/ccr-codex/backend/test-fixtures.md：TestCodexEnv 持 process env lock；真实用户目录不可访问；默认测试并行度不降低。
- .trellis/spec/ccr-core/backend/atomic-writer.md:124–146：叶锁、CAS、私有 payload-before-permissions、DACL 保留和 no-op 的现有合同。P4 新 owner-only API 与既有 preserve-policy 必须区分。
- .trellis/spec/ccr-core/backend/ccr-error-freeze.md：CcrError 变体集冻结；使用现有 ValidationError/ConfigError/FileIoError 或所属 crate 内域错误。
- .trellis/spec/ccr-core/backend/logging-contracts.md：secret/身份原值不进入日志；日志最终 redaction 不能替代调用点安全格式化。
- .trellis/spec/ccr-cli/backend/backend-guidelines.md 与 .trellis/spec/ccr-tui/backend/backend-guidelines.md：CLI 输出/exit、TUI i18n 和六尺寸 EN/ZH 布局门；已核对路径与合同。
- 轻量子系统验证：cargo test -p ccr-codex --all-features -- --skip export_bindings；必要 core/CLI/TUI 受影响模块过滤。权限基础修改须运行 core guarded_write/atomic_writer 相关测试；随后正式 subsystem gate。
- 正式门：just version-check；just fmt-check；just lint-strict；just test。跨 crate 最终集成使用 just ci，按现有默认并行度与 --skip export_bindings。未改源码且已经通过的正式门不重复；后续源码变更重新确认受影响门。
- 本研究没有运行上述命令。所有实现行为与 native 权限证据状态为 NOT_RUN。研究只提供实现入口，不提供 PASS。

### External references

- 已归档参考 .trellis/tasks/archive/2026-10/10-06-codex-auth-switch-panel-hardening/research/codex-auth-reference.md:38–40 记录 Loongphy/codex-auth 的完整键与文件名机制；来源 https://github.com/Loongphy/codex-auth，2026-10-06 旧研究读取。此次未访问网络，不能确认上游当前状态。
- 本轮版本和 API 判断以本地 Cargo.lock、源码与 Trellis specs 为依据；没有引入新依赖，也没有使用真实 OpenAI 响应或登录凭据。

## Caveats / Not Found

1. 当前源码没有生产 chatgpt_user_id parser 或包含真实 user claim 的固定 fixture。命名空间/顶层兼容方案为实施建议，必须以合成合同测试固定；不扩展任意 sub/user_id 别名。
2. P1 内存 quota cache 的 account_id-only key 是必须审阅的相关边界；若排除，报告必须保留“同 workspace 不同 user 可能复用缓存 email/quota”的限制。
3. P2 在本报告期间持续修改共享文件，行号会变；后续代理用函数名+最新行号定位，不复用审计旧数值。
4. P3 保留数量、删除凭据备份留存和 P6 身份冲突/缺失兼容仍未全部确定。依赖这些选项的清理或导入行为不能按研究建议自行批准。
5. 现有 Windows versioned metadata helper 不收紧 DACL；P4 需要真实新 API/策略与 native DACL 验证。Unix gate 在 Windows 未运行必须写 NOT_RUN。
6. 当前导入不是整包事务；preflight 可保证身份校验失败无写入，不能覆盖后续 I/O 失败。Tauri 对象数组逐项提交与服务 bundle 批量结果也不同，不能在文档中合并为相同原子保证。
7. P5 评估结论为暂不迁移；P6 CLI 默认加密已经存在。两项都不支持未经确认的新快照布局或 UI 导出入口。
