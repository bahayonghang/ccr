# P4 源码研究

来源：父任务 research/followup-design-research.md；2026-10-06。仅为只读研究，行号在实施前复核。未运行产品验收。

 SID 权限、元数据版本校验与错误脱敏

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

