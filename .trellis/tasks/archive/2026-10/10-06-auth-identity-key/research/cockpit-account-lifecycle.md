# Research: cockpit-tools Codex account lifecycle and CCR gaps

- Query: 用户已执行 `ccr codex auth save k12`；切换账号后为何仍不能查询 k12？对照 `ref/repo/cockpit-tools`，定位账号保存、切换、令牌轮换回写及查询的最小可修复遗漏。
- Scope: internal。只读本地参考源码与 CCR 当前源码；未读取用户真实凭据，未联网，未运行参考项目或产品测试。
- Date: 2026-10-06
- Active task: `.trellis/tasks/10-06-auth-identity-key`
- Status: RESEARCH_COMPLETE / IMPLEMENTATION_NOT_PERFORMED。源码处于 P1 独立检查并行修改中；以下行号对应本次读取。

## Findings

### 1. 保存存在与远端认证可用分别判断

主会话提供的脱敏诊断：k12 的保存文件存在，完整 OAuth 身份可解析，保存文件与 registry 的 account context 一致，两个用户 claims 一致；email claims 存在，按 CCR `mask_email` 处理后与 registry.email 一致。runtime 与 k12 分属不同用户、不同 workspace；config 明确采用 file store。k12 的 access token 本地到期时间为 2026-10-16；仅执行额度 GET 时 k12 返回 401，当前 khanh 同方法返回 200。发现的 1 份身份匹配备份较旧，未发现更新凭据。没有真实刷新、登录或写入。以上是主会话传入事实，本研究未重新读取用户文件。

`save_current` 将读取到的 runtime `auth.json` 字节以私有权限复制至 `auth/<name>.json`，并记录 registry；切换使用目标保存文件，不删除其他账号快照（`crates/ccr-codex/src/services/codex_auth_service.rs:945`、`:981`、`:1262`、`:1292`）。因此 k12 的保存信息仍在，与现有诊断一致。保存操作不能证明服务端接受该 access/refresh token，也不能冻结服务端认证状态。

k12 的 401 原因未查明。不能从 401 推断执行过 `codex logout`、refresh token 被复用、账号数据丢失或当前 P1 的共享 workspace 覆盖。当前诊断已排除该案例的同 workspace 用户覆盖和非 file store 路由；没有证据证明哪条历史路径令 token 失效。用户已选择稍后本人登录，先修代码。

### 2. Files found

| File | Description |
| --- | --- |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_account.rs:4` | 桌面 Codex 账号模块入口；通过 include! 组合存储、刷新、authority sync、projection、switch 与 import。 |
| `ref/repo/cockpit-tools/crates/cockpit-core/src/modules/codex_account.rs:4` | cockpit-core 的另一套精简入口；不能把桌面完整锁/刷新实现默认归给 core。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_account_lifecycle.rs:366` | 保存/重新授权 upsert，保存 token chain 的代数和更新时间。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_account_storage_locks.rs:253` | 账号锁、跨进程 token 锁、profile 租约及 chain generation。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_account_authority_sync.rs:316` | 按 file/keyring/auto 读取官方 authority，身份与新鲜度过滤，多个 runtime 目录同步。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_account_token_refresh.rs:438` | quota 查询前同步、access token 到期判定、取得刷新锁后重读。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_account_runtime_switch.rs:155` | 锁内刷新、复用其他调用完成的更新、完整 switch 期间持锁。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_quota.rs:1645` | 查询与 token 刷新分层、独立账号 quota 与状态保存。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_quota_refresh_scheduler.rs:115` | 同一账号的并发 quota 请求共享一次执行结果。 |
| `ref/repo/cockpit-tools/src-tauri/src/modules/codex_temp_login.rs:212` | 新建空白独立 profile，登录后导入账号，保留当前默认 profile。 |
| `crates/ccr-codex/src/services/codex_auth_service.rs:905` | CCR 保存、换出同步、目标投影与 registry 更新。 |
| `crates/ccr-codex/src/services/codex_oauth_token_service.rs:229` | CCR runtime/backup repair 候选、双向同步计划、快照 CAS。 |
| `crates/ccr-codex/src/services/codex_quota_service.rs:191` | CCR 独立账号路由、HTTP 后写回与 repair。 |
| `crates/ccr-codex/src/services/openai_quota_core.rs:181` | OpenAI quota 缓存、使用 access token、需要时刷新、错误分类。 |
| `crates/ccr-core/src/core/lock.rs:198` | 可复用 LockManager、跨进程 FileLock 与超时。 |
| `crates/ccr-core/src/core/guarded_write.rs:131` | 稳定 BLAKE3 内容版本及带版本私有原子写。 |

### 3. 可参考的生命周期机制与 P1 当前覆盖

| Mechanism | cockpit-tools evidence | CCR assessment |
| --- | --- | --- |
| 更新已存账号 | upsert 更新 existing account、标记 chain 更新；缺失新 RT 时保留 existing RT（`codex_account_lifecycle.rs:425`）。 | 已有显式 `save --force` 与轮换回写，不需要修改账号名称或新增账号 UI。 |
| 切换前同步 | `switch` 先调用 `sync_active_official_account_before_switch`（`codex_account_runtime_switch.rs:1039`）；从运行中的官方 profile 回收新 tokens（`codex_account_authority_sync.rs:798`）。 | P1 已有换出前同步（`codex_auth_service.rs:1260`），完整身份已隔离。现有步骤是 best-effort，尚缺统一 refresh 操作锁。 |
| 查询前使用最新官方凭据 | 先同步匹配 runtime authority，再判断 AT 到期；进入跨进程 RT 锁后重读、重新同步、再次判定到期（`codex_account_token_refresh.rs:438`、`:500`）。 | 活动账号会先 sync 再从 runtime 查询（`codex_quota_service.rs:520`）；非活动保存文件独立查询，permanent refresh 错误才尝试 repair（`:287`）。 |
| 刷新成功立即持久化 | 新 tokens -> chain generation++ -> `save_account` -> sidecar（`codex_account_runtime_switch.rs:69`、`codex_account_storage_locks.rs:583`）。 | 已有刷新 callback、私有 CAS 写入及 runtime 后回写 snapshot（`codex_quota_service.rs:220`、`:226`、`:599`）。 |
| 并发刷新去重 | 异步账号锁及跨进程文件锁；锁内重读；observed generation 已更新则复用（`codex_account_runtime_switch.rs:208`、`:247`）。 | 仍缺 HTTP 前临界区。P1 的写回 CAS 仅保护结果落盘，不能阻止重复消费同一旧 RT。 |
| 查询与刷新语义 | 有效 AT 直接返回查询准备结果，不因 id_token 到期额外轮换 RT（`codex_account_token_refresh.rs:496`、`:541`）。 | P1 补充已使 `force_refresh` 仅绕过 quota 缓存（`openai_quota_core.rs:180`、`:226`）。401 仍进入刷新分支（`:267`），符合真实远端拒绝需恢复认证的边界。 |
| 同账号 quota 合并 | scheduler 为已有账号 pending 添加 waiter，leader 执行一次（`codex_quota_refresh_scheduler.rs:139`、`:191`）。 | CCR 全部查询只有 semaphore=5 与 join_all；没有按身份合并（`codex_quota_service.rs:410`、`:418`）。最小修复先保证 token 请求串行，不必新增 scheduler。 |
| credentials store | file/keyring/auto/ephemeral 选择、Windows vault adapter（`codex_account_authority_sync.rs:276`、`:323`）。 | CCR 明确仅 file 支持 managed account（`codex_auth_service.rs:256`），实际 k12 config=file；keyring 扩展不能解决本例，不纳入本轮。 |
| 新登录隔离 | 独立空白临时 profile 与 session marker（`codex_temp_login.rs:218`）。 | 用户已选择稍后本人登录。可后续提供隔离 CODEX_HOME 与命令，不自动替换 khanh，也不把登录作为代码测试。 |

### 4. 仍可修复的具体遗漏

#### A. HTTP 前缺少账号级刷新锁

当前 `fetch_account_quota_inner` 先路由并读取 snapshot，直接调用 core（`codex_quota_service.rs:197`、`:204`、`:220`）。core 在 AT 到期或 quota 拒绝后直接 `refresh_access_token`（`openai_quota_core.rs:226`、`:267`）。callback 的 expected token 比较及 guarded CAS 位于 token HTTP 返回后（`codex_quota_service.rs:617`、`:663`）。因此两个 service/process 可读到同一旧 RT，并在任何一个结果写入前同时提交 token POST。后置 CAS 可拒绝过时写入，但旧 RT 已被提交，不能证明整个 chain 仍可恢复。

该判断是源码可复核的风险，尚未对当前 CCR 构造并发复现；不能作为 k12 历史 401 的已证实原因。

#### B. 相同完整身份的别名或 runtime/snapshot 需要共享刷新来源

当前 runtime routing 先要求 `registry.current_auth == account_name`（`codex_quota_service.rs:523`），同完整身份的其他别名可继续读取各自较旧 snapshot。仅以文件路径加锁不能保护同一 RT 的多个副本。即使改为同 identity 的锁，锁后重读同一个未同步的别名文件仍可提交旧 RT。必须在锁内确定可信、最新的匹配 source，并复用另一调用已持久化的 chain。

runtime 可作为同完整身份来源，但必须保留 P1 完整 key 校验、用户选中账号语义及 freshness。其他已保存 snapshot 若纳入 canonical source，需要明确同 identity 的不同有效登录 chain 与同 RT 副本的规则；不要在未知身份时跨文件复制。当前 `resolve_latest_oauth_doc` 只扫描配置 runtime 与符合命名的 runtime backups，未扫描所有别名 snapshot（`codex_oauth_token_service.rs:229`、`:247`）。

#### C. switch 与刷新临界区需要协调

switch 换出同步后读取目标 snapshot，随后 commit config/auth（`codex_auth_service.rs:1260`、`:1263`、`:1292`）。该步骤未持有与 quota 共享的账号 token 操作锁。若当前 A 的 quota 正在刷新，切换到 B 可令 A 的刷新结果被 CAS 拒绝；A 新 chain 未成功回收。只取得目标 B 锁不能覆盖换出 A。

参考在目标准备、关闭旧 runtime、最终投影全过程持有 token 锁（`codex_account_runtime_switch.rs:1054`）。CCR 最小协调应在换出同步前取得 outgoing/incoming 的去重锁集，稳定排序，锁后复核来源身份，持有至 token 回收与 runtime commit 完成。多次并行 switch 还需稳定 runtime 操作锁或现有 CAS 约束；该部分须由实现/检查明确保证范围，不能宣称可封闭外部 `codex login` 的任意写入。

#### D. save 的 metadata 来自第二次 runtime 读取

`save_current` 先读取并复制 content（`codex_auth_service.rs:948`），随后调用 `get_current_auth_info` 再取 runtime（`:959`）；registry 的 account_id/email/last_refresh 取第二次读，identity_key 取第一次 content（`:964`）。读取期间外部切换会使保存字节、identity_key 和 metadata 分属两个账号。

最小修复从同一 content 构造 auth state、metadata 与 identity。保留原名称、`--force`、API key/provider 路由及 P2 只读 gate。当前 k12 的 context 与 snapshot 已确认一致；本缺口不能解释现有 k12 案例。

#### E. 时间戳可能让旧 token 覆盖新 token

参考拒绝 access JWT expiration 明确较旧的 authority snapshot，即使 snapshot last_refresh 更晚（`codex_account_authority_sync.rs:477`、`:502`）；其注释明确 last_refresh 单独不能证明 chain 已更新。CCR 同步和 repair 主要按 last_refresh/mTime 决定（`codex_oauth_token_service.rs:243`、`:287`、`:507`、`:343`）。

可增加最小防回退校验：同完整身份、AT expiration 两边均可解析时，明显更早的候选不能仅凭 mTime/last_refresh 获选。该规则属于另一个新鲜度变更，必须先写入设计与合成回归；本轮优先完成 A–D，不自动改既有阈值或时间合同。

### 5. 最小 CCR 实施方案

1. 新增 crate-private `services/codex_auth_refresh_lock.rs` 并在 `services/mod.rs` 注册；复用 `ccr_core::core::lock::{FileLock, LockManager}`，无需修改 core 或公开 DTO。
2. 资源标识使用 `codex_auth_refresh_` 加完整 OAuth key 的稳定 BLAKE3 摘要；复用 `content_version_token`（`guarded_write.rs:130`）。身份未知时使用现有 refresh/access 凭据 fingerprint 语义形成私有锁 key。锁文件名、日志和错误不得含 token 或完整 identity。不要用账号别名或文件路径作为已知 identity 的锁 key。
3. 复用 `LockManager::with_default_path` 和 `CCR_LOCK_DIR`（`lock.rs:218`），使同一用户的多个 CCR process 共享资源。`FileLock::new` 含 blocking retry/sleep（`:151`），async 调用用 `spawn_blocking` 获取 RAII FileLock 后带回 async 作用域；不能阻塞 Tokio executor。锁超时是明确失败，不标为账号注销。
4. 修改 `codex_quota_service.rs` 的 source wrapper：初读只用于确定锁 key；取得锁后重新确认 registry、runtime routing 与实际 snapshot。与初读身份不同则释放后重算或安全返回冲突，不能把 k12 查询改查 khanh。锁内复用已落盘的新 chain，再调用现有 core，并持锁覆盖刷新结果写入和匹配 snapshot 回收。修复后 callback CAS 继续保留。
5. 已知 identity 的 runtime/saved/别名共享锁；在锁内采用安全最新匹配 source，避免两个旧副本先后提交同一 RT。core `fetch_quota` 及公开 `CodexAccountQuota` 无需改变；合成测试证明实际来源和请求次数。
6. 修改 `codex_auth_service.rs`：switch 参与 incoming/outgoing 账号锁；save 从同一字节构造 metadata。若扩展到其它 snapshot 写入入口，为 import/delete/rename 与 OAuth sync 明确与此锁的关系，避免下轮引入反向锁序。
7. 固定锁顺序：refresh 操作锁集 -> registry save 锁 / guarded path 叶锁。多个身份操作锁去重并按稳定 key 排序。禁止持有 registry/path 叶锁后回取 refresh 操作锁。持锁路径不得递归获取同 identity 锁；需要内部 already-locked 执行 helper。

该方案借鉴机制，不复制参考代码或新建后台守护。外部 Codex 不使用 CCR 锁，仍依靠身份/版本校验防止错写；未落盘的新 RT 遇外部强制替换仍属于显式限制。

### 6. Meaningful validation

全部使用 TestCodexEnv、临时私有文件和 loopback stub。不得使用真实 OAuth endpoint 或真实账号 token。

| Case | Required receipt |
| --- | --- |
| 两个 service 同时查询同一过期 source | 1 次旧 RT POST；两调用读取新 chain，结果不误报需重登；最终文件包含新 RT。 |
| 跨进程查询同一 identity | 使用真实 FileLock 的子进程隔离测试证明第二进程锁内重读；不能只用 mock Mutex 代替。 |
| current + 相同 identity saved alias | 共同锁与最新 source；旧 RT 总提交一次，aliases 不造成 token chain 分裂。 |
| 同 workspace 的不同 user | 不共享身份锁，不交叉写入/cache；可在 loopback barrier 证明独立请求可并行。 |
| 未知 identity | fingerprint 锁隔离且无跨文件关联，保留已批准 NoOp 合同。 |
| source 在等待锁期间切换为另一 user | 固定冲突或重新路由；不请求另一个账号额度，不写另一个账号文件。 |
| A 查询期间切换 B | switch 等待/冲突处理；A 的新 RT 回收完成后 B commit，A saved 仍保留。 |
| 新 token 到达后第二请求的旧 401 | 第二请求采用已落盘的新 AT，不重复消费旧 RT。 |
| 锁超时/读取失败 | 无 token POST、无 credentials 改动，错误不含 secrets；RAII 正确释放。 |
| save 读取后 runtime 改为 B | metadata 与保存 A 的 bytes 一致，或安全拒绝；不能生成混合账号记录。 |
| 有效 AT + 失效 RT | force query 只发 GET、绕过缓存；无 POST/写入。此项 P1 已实现，仍保留回归。 |
| 真实 k12 恢复 | 用户本人新登录后另行验证；本地合成 PASS 不表示真实账号可查询。 |

建议在上述定向回归后运行现有 Codex/TUI/CLI 相关门槛、fmt 与 strict lint；最终全 workspace 门槛由主会话执行。保持默认测试并行及 `--skip export_bindings`。

## Related specs

- `.trellis/spec/ccr-codex/backend/backend-guidelines.md`：Filesystem And Security、Runtime And Snapshot Sync、Complete OAuth Identity Association、Registry Compatibility And Read-Only Mode。新操作锁必须保留私有写、已有 CAS、缺完整身份 NoOp 和 P2 只读规则。
- `.trellis/spec/ccr-codex/backend/test-fixtures.md`：合成 TestCodexEnv 与 CCR_LOCK_DIR 隔离。
- `.trellis/tasks/10-06-auth-identity-key/prd.md`、`design.md`、`implement.md`：完整 identity 与 force-query 修复已在当前任务；账号级 HTTP 前锁与 save 同源 metadata 需要主会话补充范围及独立检查。
- 父任务的 P3 已获用户决定：保留所有旧备份，只修复去重、防覆盖和可靠备份。当前研究不新增 retention 清理。
- P6 已获用户决定：拒绝 token 与 metadata 身份冲突输入；保留 CLI 凭据导出强制加密。当前研究不扩展导出 UI。

## External references (local checkout only)

- 本地参考入口 `ref/repo/cockpit-tools/CONTEXT.md`：官方 provider/custom provider 术语，本研究按该 glossary 使用。
- `ref/repo/cockpit-tools/src-tauri/Cargo.toml:3`：参考桌面包版本 `1.3.65`；`crates/cockpit-core/Cargo.toml:3` 为 `0.1.0`。这是本地 checkout 声明，未验证 upstream latest。
- 未使用联网文档，不对 OpenAI 服务端 revocation 机制、OAuth 是否撤销其它 session 或当前账户安全策略作外推。

## Caveats / Not Found

- 未发现 Codex `auth.json` 的 `notify::` / `RecommendedWatcher` watcher。参考主要在查询、刷新、切换、投影执行的观测点同步，还支持多运行目录来源。不能把 quota scheduler 的 `notify_waiters` 当文件监测。无需因对照而新增 CCR watcher、后台守护、自动切换或新 UI。
- 在所检查 Codex OAuth/account/temp-login Rust 模块中没有找到主动发起 logout/revoke 的实现；部分符号是错误分类的 revoked 文本。该结果不能说明外部官方 Codex 不撤销 token，也不能解释历史 k12 401。
- 参考 identity 方案并非全面优于 P1：存储 ID 是 email/account/organization 的 MD5（`codex_account_token_refresh.rs:878`），部分 authority 匹配不要求 user_id（`codex_account_authority_sync.rs:382`），file lock 名部分用 account_id/email（`codex_account_storage_locks.rs:308`）。CCR 已批准完整 user_id::account_id，不应退回参考的较弱关联。
- 参考将 raw 错误拼入 user-facing refresh 信息（`codex_account_token_refresh.rs:95`）。CCR 应继续 P4 secrets/output boundary，不能照搬错误回显。
- registry.email 由 `mask_email(current_info.email)` 生成（`codex_auth_service.rs:972`）；原始 snapshot email 与 registry 直接比较不等属于预期，不能作为错配证据。主会话已确认 k12 在相同 masking 规则下匹配。email 不是 P1 OAuth identity source，不输出用户邮箱，也不据此判断 token 归属错误。
- 未运行产品测试、参考测试、真实登录、刷新或 server revoke；本报告全部实施建议尚待 implement/check 回归证明。报告写入是本代理唯一文件变更。
