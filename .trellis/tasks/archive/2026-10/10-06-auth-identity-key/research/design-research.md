# P1 源码研究

来源：父任务 research/followup-design-research.md；2026-10-06。仅为只读研究，行号在实施前复核。未运行产品验收。

 完整身份、旧记录与所有匹配边界

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

