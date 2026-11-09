# 当前 Codex Auth 切换与 token 生命周期现状

基线：dev @ 797c1877，工作区干净。锚点为 2026-10-06 实际读取位置。

## 1. 存储与职责

| 文件 | 位置（锚点） | 写入者 |
| --- | --- | --- |
| 活动凭据 | `~/.codex/auth.json`（codex_auth_service.rs:153 auth_json_path） | codex CLI 自身 + CCR commit_plan |
| 账号快照 | `~/.ccr/platforms/codex/auth/<name>.json`（:158 auth_storage_dir、:173 account_auth_path） | 仅 save_current / sync / repair |
| 注册表 | `~/.ccr/platforms/codex/auth_registry.toml`（:163；v1.0 字段 version/current_auth/accounts/usage_ledger，models/codex_auth.rs:292） | CodexRegistryStore（锁+备份+原子写，codex_registry_store.rs） |
| codex 备份 | `~/.codex/backups/auth.<label>.<ts>.json.bak` | commit_plan 写前（runtime_switch 标签，codex_runtime_service.rs:245）；保留为 auth 前缀共享池 MAX_BACKUPS=10——所有 auth.*.bak 标签共用（codex_config.rs:41、:271-299） |
| CCR 备份 | `~/.ccr/platforms/codex/auth/backups/`（:168 backup_dir） | 注册表保存前（CodexRegistryStore.save）；backup_current_auth（:1223，dead_code） |

## 2. 关键流程

### save_current（codex_auth_service.rs:862）
活动 auth.json 整文件复制为快照 + 注册表登记（account_id、脱敏 email、plan、last_refresh、usage_ledger 激活记录）。仅此刻快照与 runtime 保证一致。

### switch_account（:1106）
1. 校验：ensure_managed_auth_supported（:253）、ensure_current_runtime_supports_openai_switch（:763）；
2. 读快照 → infer_auth_intent → normalize_auth_map_for_intent（models/codex_auth.rs，按认证模式仅保留所需字段）；
3. config.toml 写 forced_login_method + apply_account_route_config；
4. commit_plan（codex_runtime_service.rs:230-300）：备份当前 auth.json/config 到 ~/.codex/backups（runtime_switch）→ 原子写 config → 原子写 auth（失败回滚备份）；非 file 凭据存储时 Write 被拒绝（校验错误）；
5. 注册表 current_auth/last_used/usage_ledger 更新 + sync_current_auth_registry 对账。

**缺口：不把换出账号的 runtime tokens 回写其快照。** 换出账号自上次 save/sync 后的轮换 tokens 仅存在于 runtime_switch 备份中。

### TUI 切换（app.rs:1264 switch_selected_account）
- detect_codex_process（codex_auth_service.rs:1369）仅提示运行中进程，不阻断；
- profile_off_for_platform(Codex) → switch_account → 成功后 should_quit = true（退出 TUI）。

### 配额刷新（codex_quota_service.rs）
- 当前账号（未保存虚拟 default，TUI key "default"，app.rs:35、:1462-1466）：读 runtime auth.json（:450 current_auth_path），token 过期即 refresh，persist_tokens 写回 runtime（:214）。
- 已保存账号：读快照，refresh 写回快照（:172 → :249 fetch_saved_snapshot_with_repair）。TUI 对活动已保存账号同样走此路径（selected_quota_key 返回 account.name，app.rs:1400-1402；预览预取刷新全部已保存账号含活动账号，:1374-1379）→ 配额刷新会消费快照中的 refresh_token（见 §3 第二链路）。
- 失败且 should_repair_tokens 命中 → repair_saved_account 后重试一次（:249-300）。

### 自动/手动回写与修复
- sync_runtime_tokens_to_saved_account（codex_oauth_token_service.rs:333）：runtime → 按 account_id 匹配快照 → sync_account_auth_file（:282，原子写+私有权限）+ update_registry_metadata（:321）。
- 调用点：仅 platforms/codex.rs:1289（清理 OpenAI tokens 前）；CLI `ccr codex auth sync`（ccr-cli/src/commands/codex/auth/sync.rs）；TUI 加载/刷新/切换均不调用。
- repair_saved_account（:390）：扫描 runtime auth.json + ~/.codex/backups（优先 runtime_switch 前缀，按 mtime 倒序，上限 120 条）中匹配 account_id 的最新 OAuth doc 回写快照。
- 调用点：CLI repair（ccr-cli/src/commands/codex/auth/repair.rs:15）、TUI R 键（app.rs:910）、配额失败自动修复（codex_quota_service.rs:265）。

### 触发谓词（openai_quota_core.rs）
- should_force_refresh:343：`token_invalidated` / `authentication token has been invalidated` / `401`。
- should_repair_tokens:351：仅 `refresh_token_reused` / `invalid_grant`。
- 截图错误来自 token 端点（`Token 刷新失败 (401 Unauthorized) [refresh_token_invalidated]`，openai_quota_core.rs:448），经 `?` 直接返回（:174、:211），不经过 usage API 错误分支（:207-208）→ 不触发 force-refresh 重试；也不命中 repair 谓词 → 自动修复不触发，账号保持 ERR。

## 3. 失效链条（用户复现：保存 → 退出 → 登录新账号 → 原账号失效）

OpenAI refresh token 为一次性轮换（消费旧值换新值，旧值作废）：
1. A 活动期间 codex CLI 轮换 A 的 refresh_token（runtime auth.json 得到新值 R_n，CCR 快照仍为旧值）；
2. 用户退出，外部 `codex login` B 覆盖 runtime auth.json；R_n 未落入任何 CCR 快照；
3. 切回 A：codex 用快照中的已消费 refresh_token 刷新 → 401 refresh_token_invalidated；
4. 自动 repair 不触发（谓词不匹配）；手动 R 修复需要 runtime/backups 中存在更新的 A tokens——若不存在则永久失效，只能重新登录。

另一条独立链路（CCR 自身轮换分裂）：
1. 活动已保存账号的 TUI 配额刷新走快照路径：读快照（codex_quota_service.rs:172）、消费快照 refresh_token、写回快照并置 last_refresh=now（:255-258、update_auth_file :522）；
2. runtime auth.json 仍持被消费的旧值；codex CLI 下次用旧值刷新 → 401 refresh_token_invalidated；
3. 下一个 CCR 观察点若无新鲜度比较，会把被消费的 runtime 值覆盖较新的快照（sync_runtime_tokens_to_saved_account :333 无新鲜度检查；repair 亦可能以旧换新，:466 `refresh_changed || latest_ts > current_ts`）→ 两侧全部失效。

经 CCR 切换 A→B 时，runtime_switch 备份会捕获 A 当时的 tokens（repair 可恢复），但依赖 auth 前缀共享保留池（10 个，跨标签共享）与手动或谓词命中触发。codex CLI 自身在 login/logout 时是否备份旧 auth.json：未核实，需实验确认。

## 4. 待验证候选（"切换本身不可用"）

用户报告切换功能本身不可用，尚未复现归档。候选原因（实施阶段逐一验证）：
1. codex 进程/App 运行中持有旧 tokens，退出时回写覆盖切换结果（TUI 仅提示不阻断）；
2. cli_auth_credentials_store 非 file 时 commit_plan 拒绝 Write 的报错呈现与恢复指引；
3. normalize_auth_map_for_intent 字段裁剪导致 codex 不认可快照（需对照 codex 期望字段）；
4. forced_login_method / profile 绑定 / runtime auth only 模式交互（截图 "Profile: Not bound · runtime auth only"）；
5. 快照 token 已失效（第 3 节链条）表现为"切换不生效"；
6. TUI 切换后 should_quit 与用户预期不符（切换即退出）。
7. CCR 自身轮换分裂（§3 第二链路）：活动账号配额刷新消费快照 refresh_token，runtime 旧值随后被 codex 使用并失败；D1 新鲜度定向 + 活动账号配额路由 runtime 后重测。

## 5. 参考机制对照（codex-auth）

详见父任务 research/codex-auth-reference.md：
- codex-auth 每次前台命令前同步活动账号快照（copy-on-change）；CCR 无常规观测点同步。
- codex-auth 登录隔离在 scratch home；CCR 无法拦截外部登录 → 观测点最大化 + 窗口边界文档。
- codex-auth 身份为 user_id::account_id；CCR 仅 account_id。
