# P1 R5 账号凭据生命周期实施记录

2026-10-06。状态：IMPLEMENTED，LOCAL_GATES_PASS，等待独立检查完成。产品冻结后不再运行 Cargo 或修改产品。前序 P1 完整身份、P2 注册表保护与手动配额查询修复均保留。

## 结果与范围

CCR 同一完整 OAuth 身份的配额刷新在 HTTP 请求前取得操作锁，等待者在锁内重读凭据。换出同步至切换提交持有来源及目标锁。相同身份的合法已登记别名采用已落盘的新鲜候选。`save_current` 从同一份 runtime 字节生成快照和元数据。公开 API、DTO、注册表版本、同步计划和 `RuntimeSyncOutcome` 形状保持。

本轮修改 `crates/ccr-codex/src/services/codex_auth_refresh_lock.rs`（新增）、`mod.rs`、`codex_auth_service.rs`、`codex_oauth_token_service.rs`、`codex_quota_service.rs`。生命周期回归放在对应文件的测试模块。`codex_runtime_service.rs` 沿用前序独立检查增加的 crate-private CAS 入口，本轮未修改。先前 `openai_quota_core.rs` 的 force 配额语义修复与缓存夹具修复也未撤销。

参考依据为 [cockpit-account-lifecycle.md](cockpit-account-lifecycle.md)。参考的账号操作互斥、跨进程锁及锁内重读用于确定修复边界；本轮未增加 generation 字段、AT-exp 优先级、watcher、后台刷新或通用事务框架。

## 操作锁与写回

新模块的 `CredentialLocks::acquire_sources` 为每个实际来源保留规范化路径资源，并加入可得的完整 `user_id::account_id` 身份资源。资源名只包含现有 `content_version_token` 的 BLAKE3 哈希。未知身份仅使用路径资源，无法按 workspace 与其他文件关联。路径规范化解析父目录或最近存在祖先，Windows 路径大小写一致；缺失文件及目录创建后资源名保持一致。

全部资源排序、去重后通过现有 `LockManager::with_default_path` 与 `FileLock` 获取。异步入口使用 `spawn_blocking`。操作锁覆盖来源重读、刷新 POST、凭据持久化与必要快照同步。锁顺序为操作资源的稳定排序 → 注册表锁或 guarded-write 路径叶锁。持锁方法调用 crate-private `_locked` 方法，避免重复获取同一资源。`verify_path` 按每个路径的初始资源分别比较，目标变成双锁集合中的另一身份仍会拒绝。

配额当前来源与已保存来源均在锁内验证初始身份。未知身份还验证同文件 token 内容。运行路由及 repair 后的最终重读均再验证来源，然后执行 HTTP。响应写回保留已有完整身份检查和版本 CAS。外部替换来源导致 CAS 拒绝时，不继续以旧身份锁同步另一身份。成功刷新后只向同完整身份的合法已登记别名同步。

`switch_account` 在只读注册表和命令前置检查通过后，获取 runtime 与目标路径/身份资源，锁内重读注册表、来源和目标。操作锁从换出同步持有至 runtime 提交。`save_current` 同样在只读注册表拒绝点之后获取锁；文件内容、`AuthState`、email、account_id、identity_key、plan 与 last_refresh 来自同一次读取。P2 六项只读命令的文件树不变检查保留并在完整包中通过。

## 已复现边界与修复

首次新增的同来源/别名并发查询和刷新交叉切换回归均失败：两个请求重复消费旧 refresh token；切换替换 runtime 后刷新结果被 CAS 拒绝。最初两项修复后通过。

独立预审指出 unknown 来源首次补全身份后，原调用只持路径锁，而已知身份别名可读取刚落盘的新链并提交新 refresh token。补充回归复现新 `RT1` 被 POST 两次。现在持久化成功且首次形成完整身份时，回调以固定私有控制值结束核心请求；控制值只在本 wrapper 捕获升级 snapshot 后按严格相等识别。核心不会继续 GET 或认证拒绝后的第二次刷新，也不会在旧锁内执行 runtime/别名同步。外层释放旧锁，再按首次写出身份获取路径与完整身份锁，最多内部重试一次。重锁后的来源换成另一用户会拒绝。控制值和私有 enum 不进入公开 DTO、日志或重新登录映射。

同秒精度回归也复现 `refresh_token_reused`。新鲜度仍按 last_refresh 优先、否则 mtime 比较，保留完整纳秒精度。备份候选排序保留完整 `SystemTime`、原 preferred 优先级和 120 项上限。快照 last_refresh 写回使用 `SecondsFormat::AutoSi` 保留小数。回归先查询非 current 别名，再验证目标快照的 `.900` 时间戳，最后查询 current；`.100` 的旧 runtime 不再覆盖新链。

为保持公共同步报告，公共 `sync_runtime_with_saved_account` 不预先 repair 后报告 `Unchanged`。只有配额内部准备按同身份现有候选与新鲜度规则选择较新凭据；必要 runtime 更新调用已有 `commit_synced_auth_versioned`。`SnapshotUpdated`、`RuntimeUpdated`、`SkippedStaleRuntime` 和只读注册表后台 token 同步回归在完整包中通过。

## 首败与检查收据

日志位于本 `research/` 目录。首败原始字节保留，没有将旧结果改为通过。全部运行使用默认测试并行和 `--skip export_bindings`，未更改忽略规则、阈值或扫描上限。

| 收据 | 命令/范围 | 结果 |
| --- | --- | --- |
| `account-lifecycle-first-failure.log` | `cargo test -p ccr-codex --all-features lifecycle_ -- --skip export_bindings`，最初两个并发/切换回归 | 0 通过，2 失败 |
| `account-lifecycle-first-retest.log` | 同一初始过滤范围 | 2 通过 |
| `account-lifecycle-auth-validation.log` | AuthService 定向中间验证 | 81 通过 |
| `account-lifecycle-targeted-validation.log` | 当时 lifecycle 定向 | 4 通过 |
| `account-lifecycle-codex-validation.log` | `cargo test -p ccr-codex --all-features -- --skip export_bindings`，首次包门 | 330 通过，6 失败，2 忽略 |
| `account-lifecycle-same-second-first-failure.log` | 新同秒测试首次编译 | 缺少 `DateTime` 引入；编译失败，无行为结论 |
| `account-lifecycle-same-second-behavior-first-failure.log` | `cargo test -p ccr-codex --all-features lifecycle_same_second -- --skip export_bindings` | 0 通过，1 失败；恢复已消费 token |
| `account-lifecycle-identity-upgrade-first-failure.log` | `cargo test -p ccr-codex --all-features lifecycle_unknown_identity_upgrade -- --skip export_bindings` | 0 通过，1 失败；新 RT 提交 2 次 |
| `account-lifecycle-boundary-final.log` | `cargo test -p ccr-codex --all-features lifecycle_ -- --skip export_bindings` | 8 通过，0 失败 |
| `account-lifecycle-lock-final.log` | `cargo test -p ccr-codex --all-features services::codex_auth_refresh_lock -- --skip export_bindings` | 4 通过，0 失败 |
| `account-lifecycle-codex-final.log` | `cargo test -p ccr-codex --all-features -- --skip export_bindings` | 340 通过，0 失败，2 忽略；46.61 秒；doc-tests 0 |
| `account-lifecycle-clippy-first-failure.log` | `cargo clippy -p ccr-codex --all-targets --all-features -- -D warnings` | 两处 `collapsible_if` |
| `account-lifecycle-clippy-final.log` | 同一 strict Clippy 命令 | 通过 |
| `account-lifecycle-fmt-final.log` | `just fmt-check` | 通过；含 root/Tauri Rust、JSON、5 项 formatter 测试 |

首次包门的 6 项失败分别为：4 项公共同步结果因预先 repair 错报 `Unchanged`，1 项新测试在 runtime 外构造 tokio timeout，1 项旧路由夹具要求查询不匹配用户时仍同步另一用户。前两类已修复代码或夹具。路由夹具现验证不匹配查询不改另一账号快照，并另行调用该账号的主动观测同步确认更新。未删除原行为覆盖。

完整 340 项测试运行后，仅为 Clippy 将两处嵌套 if 折叠为等价 let-chain，并运行最终 Clippy/fmt。独立检查代理另跑冻结源码的完整门，收据归其 [account-lifecycle-check.md](account-lifecycle-check.md) 管理。实施代理不重复占用 Cargo。一次误在测试仍运行时启动 Clippy，两项命令都正常结束；没有把未完成状态计为通过。

`git diff --check --` 后跟本轮五个产品文件通过；Git 仅输出 LF/CRLF 工作区提示。没有提交、归档、替换安装二进制或改写真实凭据。

## 回归证据边界

| 行为 | 证据 |
| --- | --- |
| 同一来源的两个查询、同身份两个别名 | 同进程 loopback HTTP；两个结果成功，旧 RT 提交一次，新链保存在来源和别名 |
| 刷新期间切换 A→B | 同进程 loopback HTTP 和独立切换线程；A 新凭据保存，runtime 为 B |
| 等待来源由 A 换成同 workspace 的 B | 合成文件竞争；锁内拒绝，HTTP 刷新 0 次，B 内容保持 |
| 同 workspace 不同用户并行 | 持有 B 身份资源时 A 查询成功；B 释放后查询成功，无刷新、快照内容保持 |
| unknown 升级与 known alias 同时遇到 401 | 同进程 loopback HTTP；RT0 和 RT1 各提交一次，两个配额结果成功，最终链一致 |
| 升级释放后来源换成 B | 合成 callback 升级状态及重锁；第一次成功落盘后 GET 为 0；替换后拒绝，B 内容保持 |
| save 内容与元数据同源 | 合成 A 内容已捕获、runtime 后为 B；保存文件与元数据均为 A |
| 同秒来源选择与写回 | 非 current alias → 快照 repair → current runtime 路由；完整 `.900` 精度保留，没有重复 POST |
| 跨进程 OS 锁与锁后重读 | 真实测试子进程先零超时取得 `LockTimeout` 后通知 ready；父进程写同身份新版本、释放锁，子进程取得锁并读新 RT |
| 稳定路径与逐来源身份 | Windows 合成缺失路径创建后资源相同；双源 A/B 集合中 B 改成 A 后，B 验证仍拒绝 |

跨进程测试只证明 OS 文件锁互斥和锁后重读；完整 OAuth HTTP 流程的跨进程测试为 NOT_RUN。HTTP 请求全部通过 tokio task-local loopback endpoints，不使用真实 OpenAI 端点。缺失身份只关联同文件；不会借 account_id 为未知来源选择别名。

本轮锁接入范围为 quota、save、switch、已有同步/repair 入口。delete、rename、import 的写入接入属于后续 P3/P6，本轮未声称覆盖全部 CCR 写入。外部 Codex/login 不使用 CCR 操作锁；现有 CAS 仍只能拒绝版本冲突，无法封闭任意外部写入或已到达远端的刷新消费。

实施代理对真实 k12 的读取、OAuth 刷新、登录和修复后真实额度查询均为 NOT_RUN。主会话的只读 HTTP401 证据在独立诊断报告中；本轮合成缺陷不能证明 k12 历史失效原因，也不能证明账号已恢复。用户已选择稍后重新登录。

## 源码与日志哈希

固定清单为 [account-lifecycle-source-hashes.json](account-lifecycle-source-hashes.json)。清单记录最终五个源码文件 SHA256 和已结束的实施收据 SHA256，不纳入独立检查仍在运行的日志。

| 最终文件（services/ 下） | SHA256 |
| --- | --- |
| `codex_auth_refresh_lock.rs` | `F96EDD87B5D5486CF40D530D122F54F38806DF0DEB9E22D42F823842C0EE30FF` |
| `codex_auth_service.rs` | `B5306B70F41C42D4CBB4E8643A27203006BA7BE72CB910048044BB713B488703` |
| `codex_oauth_token_service.rs` | `D5D09966D4B2155C9B14D8D7C1F0485F77D8371F8739551082AEDB5168D64CD2` |
| `codex_quota_service.rs` | `AFF965924E7F5A92991F5BD23F08E31450988826F7E83944E280DDD028957B25` |
| `mod.rs` | `33A488CD50E588918D9F74999377C79425F5B9983BBBC36A81C8B0A082197DDE` |

首败源码哈希为 NOT_CAPTURED。实施时没有保存对应源码副本或哈希；无法从日志恢复精确源码字节，最终哈希不可追填为首败哈希。首败日志完整保留，其当前字节哈希已固定，证据限制明确保留。

父任务 workspace `just lint-strict`、`just test`、`just ci` 在全部子任务冻结后由主会话协调。本轮未执行，状态 NOT_RUN。
