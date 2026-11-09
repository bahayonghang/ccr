# 2026-11-06 独立续作检查

## 检查范围与当前结论

当前检查目标为 `dev@e8329cd8` 之后的中断工作区及续作增量，保留既有变更。已读父 PRD、design、implement、curated check context 与受影响 ccr-app/checkin/db/sync/codex/core 规范。当前先完成稳定部分源码审查；平台认证、用量导入及后续核心功能仍在其他 implement agent 内实施，**本文件目前不是最终完整变更通过收据**。

稳定部分未发现需要立即修复的确定性缺陷：

- Check-in：实际管理入口、批量操作、记录筛选分页、月历/趋势、WAF 手工管理/验证、CDK、配置/统计导入导出；保留捕获 DbAccess/crypto，fixture 真实网络拒绝。账户删除历史使用同一事务。配置导入只保非秘密预览，apply 重读/hash 校验并重新验证当前密钥；未知字段拒绝、已有 extra_config/运行时间保留。预导入已验证快照不是多 manager 原子事务，界面/规范未冒领。
- OpenCode/Antigravity 文件资源：实际全局/项目目录、原格式、未知字段、路径祖先/链接限制、上限、原版本、备份与锁目录检查、retained 写入、原生控件冲突保留草稿；插件代码不读不执行、扩展 manifest 只读。
- SSH fixture 锁：scope 在真正 spawn_blocking 写线程内生效；known_hosts 确认/连接与上传 staging 均消费捕获锁目录，未把 thread-local scope 放在 await 外冒充传播。
- Windows debug render：Remote、CodexAgents、Checkin 自然区块拆分保留顺序/控件，产品栈不增加；低栈合成与已有实际原生收据边界明确。
- WebDAV：清除确认绑定 canonical/legacy 两个版本，写前冲突拒绝，保留文件夹与未知配置，partial legacy failure 明确；批量复用单文件夹传输、逐项结果及 backup，不宣称整批回滚。
- Tauri：使用源码提取脚本逐一比较当前变更中的命令签名（checkin 32、codex_agent_sources 9、gemini 11、opencode 16、settings_raw 6、sync 17，共 91 个），未发现签名/参数/返回类型变化。共享资源 records 的 serde 名称与旧载荷字段保持。

## 仍缺实际产品入口

以下经当前 owning source 核对，不能只归类为原生收据不足：

1. Codex session export/clone/delete：Tauri `commands/codex_sessions.rs` 调用既有 `CodexSessionService`，GPUI 仅有 archive list/detail/index，缺实际操作。
2. Claude observer 的工具统计/订阅读写：已有 Tauri state-free service/subscription，GPUI 当前无对应消费者。
3. Codex quota：已有 `CodexQuotaService` 与 Tauri 单账户/全账户读取，GPUI 当前无入口。产品接通可实施，真实账户/外部服务验证仍遵守本轮限制。

已将这些具体缺口报告主会话，主会话将继续实施；不重开归档任务做纯审计，不修改已延期的 WSL/DPI。

## 本检查轮实际验证

- 稳定部分 14 个关键 Rust 文件 `rustfmt --check --config skip_children=true`：exit 0。
- 稳定部分领域/adapter/UI/Tauri 命令路径 `git diff --check`：exit 0。
- 91 个已变更 Tauri 命令签名源码比较：差异 0。
- 复用同源码已有关键 fixture/headless 收据；本轮未再次运行全部旧测试。未使用 computer use、WSL/doctor、真实 SSH/WebDAV/认证或用户数据。
- 最终 lint/typecheck/test/完整 `just ci` 由主会话集中执行，当前状态 **pending**；新实施部分待角色恢复后完整复查，不能据本轮局部检查认领最终通过。

## 最终当前源码复查

已恢复角色并复查所有后续实现：Claude/Codex/Grok auth-off 与捕获根、Codex 账户导入/API key/重命名、模型目录和供应商库 CAS/秘密保留、配额投影/生命周期、共享 llmusage CLI/NDJSON/清理、Observer 独立 main DB 与 archive 分离/只读无创建/首次保存/订阅 CAS/实际 JSONL 工具事实导入、opaque Codex session 导出/clone/recoverable trash/归档标记和离页清明文。

此前报告的 session、observer、quota 三类真实代码缺口均已由 implement 角色接通；不是把缺入口改写成“等待外部验收”。新共享 owner 没有引入第二份用量 SQL，工具事实 SQL 仍在 `ccr-db::claude_tool_calls_repo`，llmusage projections 仍由 `ccr-usage` 持有。Tauri scanner/service/subscription/CLI adapter 消费同一 owner。

### Findings（已修复）

1. **捕获的 auth-off 删除有竞态**：native 的预检查后，旧 helper 会无条件删除运行凭据；期间发生轮换时可能删除新凭据，旧 rollback 又可能覆盖另一写入者。经主会话批准，在现有 owner 增加 captured-only `auth_off_captured_versioned`，将原 runtime token 绑定到 guarded compare-delete。捕获备份使用精确原字节和独立 UUID 目录，删除前失败不恢复外部改动，删除后失败只 guarded-create 回原本缺失的目标；若已有新凭据则保留。原 CLI/Tauri helper 签名与 legacy rollback 语义不变，Grok 保持自身 revision/official locks，macOS 官方 logout 保留其原生语义、不冒领文件 CAS。
2. **Clone 的版本检查后又重读来源**：报告主会话后由 sessions implement 修复；versioned clone 的 token、session ID 和转换均消费同一有界捕获字节，实际源在捕获后变化也不会混入 clone。原 clone 入口行为保持。独立控件 1 MiB 实测还找到了新 session renderer 栈溢出，implement 按自然区块拆分后原测试通过，未加大产品栈。
3. **JSONL 文件增长可绕过初始大小检查**：scanner 只检查打开时 metadata 的 64 MiB，持续 append 时读循环原来没有累计上限。本角色在每行解析前增加累计读取上限；超限不提交该文件 rows/cursor。
4. **Quota 网络缺少 deadline**：既有 production HTTP client 没有 timeout，新 native retained quota job 的最终 join 可能无限等待。经主会话批准，只给共享 owner 的 usage GET 与 token refresh POST 显式设置 30 秒请求 deadline，不丢弃整个凭据 Future、不伪造取消；保留测试 no_proxy 客户端。

### 最后实际窄检查

- `cargo test -p ccr-cli --locked --lib captured_logout_preserves_rotations_and_restores_only_a_missing_target -- --skip export_bindings`：exit 0，1 passed；覆盖 snapshot→delete 轮换拒绝、删除后失败时保留新 winner、无人写入时精确恢复。初次新增 fixture 的 byte-array 长度类型错误已改为 slices 后重跑，未弱化断言。
- Native `codex_account_management_keeps_runtime_and_saved_accounts_distinct`：exit 0，1 passed；消费新版本化 auth-off seam。
- `cargo test -p ccr-app --locked --lib a_growing_file_cannot_bypass_the_total_read_limit -- --skip export_bindings`：exit 0，1 passed；metadata 后合成 append 超限，rows/cursor 都不写。
- `cargo test -p ccr-codex --locked --lib openai_quota_core::tests -- --skip export_bindings`：exit 0，20 passed；现有 loopback 请求/refresh/错误脱敏/缓存/事实投影回归，没有访问真实服务。
- 最后一次变更的 Tauri 命令签名源码比较仍为 91 个、变化 0；全工作区 `git diff --check` exit 0。负责文件已显式 rustfmt，验证和修复命令分开。
- 所有测试保持默认并行。root target sandbox lock 的访问拒绝经自动批准后对同一隔离测试重跑；没有调整目录权限、HOME/USERPROFILE 或全局工具配置。

### 通过边界与剩余事项

当前源码的上述实际缺口及已识别确定性缺陷已修复，可以进入主会话集中 gates。最终 lint/typecheck/full tests/root `just ci` 和产物验证结果由主会话按实际退出码补充；本独立检查不提前声称这些命令成功。

Quota 整批通过既有并发 5 执行多个账户，单个账户还可能先 refresh 再 query；**整批/最终退出可超过五秒**。运行时 owner 保留工作并最终 join，不能把进程树 cleanup 的五秒预算宣传为所有网络业务的退出预算。新请求级 30 秒 deadline 不代表总批次 30 秒，也没有增加虚假取消。

新功能只有 fixture/headless/低栈收据，不由旧二进制原生收据覆盖。真实 SSH/WebDAV/认证/CDK/quota、macOS/其他 OS、IME、性能、glass、托盘公开 API 限制和用户延期 WSL/DPI 保持各自状态；父任务归档硬门不变。未发现需要额外扩大当前核心修复范围的确定性缺陷，不宣称 Tauri 全命令逐项原生验收完成。

## 主会话最终整合收据

产品文件释放后，主会话集中 `desktop-fmt-check` / `desktop-lint` / `desktop-test` 最终均 exit=0（库161、Shell8、输入2、退出2；Python15、AccessKit3）。永久低栈守卫新增三个现有核心控件测试名称，在1 MiB和默认锁poison下五个子测试均通过。完整 CI 首次暴露本文件下文记录的 Sync fixture 锁隔离，修复并通过 poison 下 Sync17/17 后，最终完整 `just ci` exit=0（14:22.234，16阶段全部通过）。新增文件纳入暂存后补跑的 Copilot/术语治理检查 exit=0，实际覆盖6689 tracked files。

`desktop-package` exit=0，同版本双二进制未签名 ZIP 的 CRC/manifest SHA256复核通过；GUI未启动，不扩张原生验收。完整日志、初始失败、最终退出码与产物哈希统一见 `outputs/desktop/resume-20261106/` 和 `resume-summary-20261106.md`。这些是主会话实际命令结果，不覆盖本文件早期 pending/失败收据。

## 最终 CI 的 Sync fixture 隔离恢复

主会话完整 `just ci` 在 Tauri 测试 `update_sync_folder_upserts_enabled_paths_and_description` 遇到默认锁目录消失（os error 3），此前 root preflight/clippy/check/tests/release/audit/governance 与 Tauri check/lint 已通过，不能认领整次 CI 成功。

该测试单独重跑 exit 0；随后仅对该子进程把 `CCR_LOCK_DIR` 指向 outputs 下合成普通文件，确定性复现失败（无法创建锁目录，os error 183）。原因是测试虽然传了显式 `sync_folders.toml`，嵌套 guarded writer 仍查询 ambient 默认锁；并行环境夹具可以改变/清理这个路径。不是生产 WebDAV 行为或备份故障。

仅修改 `ccr-ui/src-tauri/src/commands/sync.rs` 的 test module：五个实际写入夹具在完整同步 helper 操作期间绑定自己的 thread-local locks；失败用例另断言 fixture-local 锁目录确实创建。保持生产路径、备份、原断言、默认测试并行不变。初次 helper 缺少 `Path` test import，补齐后重跑。

实际验证：`cargo --config .cargo/tauri-ci.toml test --manifest-path ccr-ui/src-tauri/Cargo.toml --locked --all-features commands::sync::tests -- --skip export_bindings` 在仍然 poisoned 的 child-only `CCR_LOCK_DIR` 下 **exit 0，17 passed**；负责文件 rustfmt check 与 diff check 均 exit 0。没有修改 HOME/USERPROFILE 或全局环境，没有串行化 suite、弱化备份或重打产品 ZIP。最终完整 `just ci` 由主会话重跑并记录实际结果。
