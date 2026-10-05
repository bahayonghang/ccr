# T05 实施记录

日期：2026-09-28。基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`。工作区：Windows，分支 `dev`。

## 修改边界

- 原行为差距：秒级备份名可以覆盖同秒前镜像；profile-off 复用同名目录；OAuth pending 使用普通 writer 后设置权限，并先发布 memory。
- Owner：`ccr-core::guarded_write` 负责备份发布；`profile_off` 负责一次复合操作的目录；新 `ccr-codex::services::CodexOAuthPendingStore` 负责 pending 存储和清理；Tauri 只做存储适配。
- 本任务不改 OAuth listener/controller、不改仓储 RMW 和平台 CRUD、不改前端、不触碰真实账户、配置或全局工具设置。T01 对 profile_off 的缺失文件清理 helper 有单独协调修改。
- 使用 `trellis-before-dev` 加载规范；测试出现异常后使用 `ccr-gate-recovery` 定位最窄失败。

## 已实现

1. 保留旧备份前缀、时间和扩展名，追加 UUID。先以原权限/DACL 准备并刷新临时备份，再 `persist_noclobber` 发布；碰撞最多重试 16 次，失败不覆盖目标或旧备份。轮换仍 keep-10，mtime 相同时按路径稳定排序。
2. profile-off 使用 create-new 临时目录分配后保留为操作目录，同秒操作不会复用快照。快照内容和补偿机制保持原契约。
3. 新 pending store 统一 save/load(now)/clear，操作锁串行保存和取消/过期清理；保存显式使用 `secret: true`、`BackupPolicy::None`。删除错误传播，过期删除失败不伪报状态已清理。
4. pending verifier/state、授权和回调 URL 使用 `Secret`。默认序列化和 Debug 脱敏，只有私有磁盘 DTO 使用 plaintext 注解。JSON 格式错误只报告 line/column。
5. Tauri 改为存储成功后发布 memory；失败保持旧 memory。成功的 login-start 仍显式返回浏览器授权 URL，现有 camelCase 磁盘格式保持不变。桌面仍以原 PlatformPaths/CCR_ROOT 解析路径后调用 Store::with_path，避免 CCR_DATA_DIR 不同时静默迁移 pending。
6. 敏感写入静态守卫增加实际 pending owner 和 Tauri adapter；同步 core、Codex、profile-off 规范。

## 验收与测试证据

| 命令/检查 | 结果 | 说明 |
| --- | --- | --- |
| `cargo test -p ccr-core guarded_write -- --test-threads=1` 首轮 | exit 0；21 项 | 当时尚未加入两项多进程用例。 |
| 同命令第二轮 | exit 1；测试进程 `0xc0000005` | 未输出具体 running/test 名；原因未查明。未改源码后重跑通过。 |
| `cargo test -p ccr-core test_fixed_clock_preserves_each_preimage -- --test-threads=1 --nocapture` | exit 0；1 项 | 缩小测试范围定位，未复现访问冲突。 |
| `cargo test -p ccr-core guarded_write -- --test-threads=1 --nocapture` | exit 0；23 项 | 包含固定时钟、三子进程、碰撞重试、旧新命名轮换和 Windows DACL。 |
| `cargo test -p ccr-core grok_auth_windows_acl_failure -- --test-threads=1 --nocapture` | exit 0；1 项 | 实际临时文件在 ACL failpoint 时仍为空；secret 字节没有先写入。此前默认捕获方式曾同样出现访问冲突。 |
| `cargo test -p ccr-codex oauth_pending -- --test-threads=1` 首轮 | exit 0；3 项 | 当时尚未加入 native ACL 与真实删除拒绝测试。 |
| 同命令 native 首轮 | exit 1；4 通过、1 失败 | Windows PowerShell 无法加载 Get-Acl 所属模块；改用同进程 .NET File.GetAccessControl 读取真实 ACL。 |
| 同命令修正后 | exit 0；5 项 | 私有 DACL 创建/替换、取消/过期删除拒绝、模拟权限拒绝、无副本目录扫描、错误脱敏。 |
| `cargo test -p ccr-cli profile_off -- --test-threads=1` | exit 1；8 通过、2 失败 | 新增同秒快照及现有 reader 的旧新发现/恢复通过；两项现有 off 用例受 T01 纯读构造器变更影响，交 T01 修复。 |
| `python scripts/quality/check_secret_writes.py` | exit 0 | 静态规则是行为测试的补充。 |
| `git diff --check` | exit 0 | 共享工作区有其他任务修改；未改写这些文件。 |

### 最终检查结果

| 命令/检查 | 结果 | 说明 |
| --- | --- | --- |
| `cargo test -p ccr-cli profile -- --test-threads=1`（T01 报告） | exit 0；60 项 | T01 修复 `clear_current_if_present` 后包含全部 10 项 profile-off；缺失文件 no-op，损坏/不可读仍报错。本任务复用主会话转发的 T01 结果，未重复执行。 |
| `cargo test -p ccr-codex oauth -- --test-threads=1` | exit 0；9 项 | 含全部 6 项 pending store 测试及原有 OAuth 测试；旧 plaintext fixture 无损读写，CLI `user_message` 不含 sentinel。 |
| `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml codex_auth -- --test-threads=1` | exit 0；1 项 registry | 原过滤词只命中 registry，不能作为 pending 行为验证。实际文件属于 `commands::codex::auth`。 |
| `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml oauth_pending_storage_failure -- --test-threads=1 --nocapture` | exit 0；1 项行为测试 | 真实 Windows sharing-denial 下保存/取消失败，旧 disk 和 memory 均保留；cancel handler、IPC 错误和 tracing 无 verifier/state。 |
| `cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml commands::codex::auth::tests -- --test-threads=1` 最终 | exit 0；5 项 | 最终路径兼容补丁后，root/data 分离与真实失败传播两项新增测试、三项原有认证模块测试全部通过。 |
| `cargo test -p ccr-core -- --test-threads=1` | exit 1；101 通过、1 失败 | 既有 `grok_auth_lock_preserves_native_holder_metadata` 在持 Windows 文件范围锁后读取路径失败，OS 33，`lock.rs:319`。 |
| `cargo test -p ccr-core core::lock::tests::grok_auth_lock_preserves_native_holder_metadata -- --exact --test-threads=1 --nocapture` | exit 1；0 通过、1 失败 | 独立复现同位置 OS 33。`git diff --exit-code -- crates/ccr-core/src/core/lock.rs` 为 0；T05 未修改该文件。交主会话核查，不改弱断言。 |
| `cargo clippy -p ccr-core -p ccr-codex --all-targets --all-features -- -D warnings` | exit 0 | 两个拥有新增存储逻辑的窄包严格检查通过。 |
| `rustfmt --edition 2024 --config skip_children=true --check`（6 个本任务 Rust 文件） | exit 0 | 包含新文件；没有运行全仓自动格式化。 |
| 再次运行 sensitive-write guard 和 scoped `git diff --check` | exit 0 | 没有忽略现有失败或清理用户文件。 |

T05 实现与本机可执行的范围验收已完成，等待独立检查。全包 core gate 仍未通过，不能以过滤测试通过替代。

### 访问冲突证据

失败命令均没有 `--nocapture`：

```text
cargo test -p ccr-core guarded_write -- --test-threads=1
cargo test -p ccr-core grok_auth_windows_acl_failure -- --test-threads=1
```

两次返回 `0xc0000005, STATUS_ACCESS_VIOLATION`。测试二进制为
`D:\Documents\Code\Github\ccr\target\debug\deps\ccr_core-89885ef5f62ab391.exe`。
输出未包含具体 running/test 名，**发生阶段未确认**，不能仅据此判定为启动阶段。
随后单例、完整 guarded_write 过滤集与 ACL failpoint 使用 `--nocapture` 均通过；期间没有源码修复。
同时出现的构建告警原文如下，尚无证据将告警与访问冲突关联：

```text
mbx[warning]: incremental state was not reused: absolute path has no stable cache mapping: C:\Program Files\Microsoft Visual Studio\18\Enterprise\VC\Tools\MSVC\14.51.36231\atlmfc\lib\x64
mbx[warning]: prediction was not restored: absolute path has no stable cache mapping: C:\Program Files\Microsoft Visual Studio\18\Enterprise\VC\Tools\MSVC\14.51.36231\atlmfc\lib\x64
mbx[warning]: result was not stored: rustc search path kind is not cacheable yet: native
```

## 四项 AC 追溯

- AC1：固定时钟三次前镜像、三独立子进程版本集合、碰撞重试、失败不覆盖、keep-10 与稳定 tie-break 已验证。
- AC2：原 ConfigManager/SettingsManager 的旧新发现和恢复通过；实际 Windows 私有 DACL 保留通过。Unix mode 用例已添加，尚未在 Unix 执行。
- AC3：创建/替换/取消/过期/模拟权限拒绝均做目标与配置备份目录扫描；Windows ACL 实查通过；真实删除拒绝、既有 core ACL pre-payload failpoint 与 Tauri 失败传播均通过。
- AC4：Secret 默认 Debug/Serialize、解析错误、CLI `user_message`、IPC 错误和实际 tracing 捕获无 sentinel。当前 CLI 没有 pending 操作命令，没有声称运行真实 CLI OAuth 流程。

## 修改文件

- `crates/ccr-core/src/core/guarded_write.rs`：唯一发布、稳定轮换、故障和多进程测试。
- `crates/ccr-core/src/core/atomic_writer.rs`：仅将现有两项 Windows DACL helper 改为 `pub(super)`，同层备份复用；物理权限策略不变。
- `crates/ccr-cli/src/application/profile_off.rs`：独立操作目录及 consumer/补偿测试；缺失文件清理 helper 是 T01 的协调修改。
- `crates/ccr-codex/src/services/codex_oauth_pending_store.rs`（新增）与 `services/mod.rs`：pending owner 和导出。
- `ccr-ui/src-tauri/src/commands/codex_auth.rs`：存储适配、Secret 消费点、disk-first memory 更新及回归。
- `scripts/quality/check_secret_writes.py`：扩展扫描范围。
- `.trellis/spec/ccr-core/backend/atomic-writer.md`、`.trellis/spec/ccr-codex/backend/backend-guidelines.md`、`.trellis/spec/ccr-cli/backend/profile-off-login-prep.md`：同步契约。
- 本任务 `implement.md`：修正 Tauri 过滤词；`implementation-report.md`：证据和边界。

## T02/T11 可复用接口

- T02：`backup_guarded` / `write_guarded` / CAS 的调用接口不变；新名称继续被原 ConfigManager/SettingsManager reader 识别。profile-off 操作目录保留旧前缀并追加唯一后缀。
- T11：`ccr_codex::services::{CodexOAuthPendingStore, CodexOAuthPendingState}`；`new()` 使用 CodexPaths（CCR_DATA_DIR/CCR_ROOT），`with_path(PathBuf)` 支持隔离 fixture，`save(&state)`、`load(now)`、`clear()` 返回原 `ccr_core::Result`。
- 兼容：Tauri 使用 `with_path` 保留原 PlatformPaths/CCR_ROOT 目录；新增 root/data 分离 fixture 验证，未自动迁移或复制凭据。
- T11 继续负责 listener 建立、取消、deadline、后台恢复错误呈现和流程身份保护。本任务没有将跨进程内存状态视为全局事务。

## 尚未验收

- Unix mode 测试已实现，当前 Windows 环境不能运行该分支。
- 未运行全仓 `just ci`、全部 Rust/Tauri gates、真实 OAuth 网络或原生桌面 UI。
- 访问冲突未稳定复现，原因未查明；不得归因于同时出现的缓存告警。
- core 原有 Windows 锁读取测试仍失败，后续由主会话处置。
- 不提交、不归档、不改任务状态；由主会话安排独立审查和集成。
