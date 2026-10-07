# 设计：私有权限与脱敏

## Core 权限边界

现有 `AtomicWriter::secret` 对已存在 Windows 文件保留原 DACL，现有 `enforce_secret_permissions_versioned` 在 Windows 不收紧 DACL。新增独立 owner-only 元数据帮助函数，复用当前进程令牌 SID 构造，使用已验证文件句柄设置 protected DACL。保留既有 AtomicWriter 与 versioned helper 的 preserve-DACL 合同。

新增带预期内容版本的 owner-only 权限入口，叶锁内验证内容，不替换文件或写 payload。缺失/版本冲突返回 false，权限失败返回既有 CcrError。Unix 清除 group/other 权限，保留更严格 owner mode。`ccr-codex::utils::ensure_private_permissions` 改为可失败的内部调用，必要生产调用点传播或明确处理错误；删除 USERNAME/icacls 依赖。

接口固定为 `ccr_core::core::guarded_write::enforce_owner_only_permissions_versioned(path: &Path, expected_token: &str, lock_timeout: Duration) -> Result<bool>`。Core 实施者只拥有 atomic_writer、guarded_write 与必要导出；Codex/TUI 实施者只拥有 implement.md 列出的产品文件。Core Cargo 检查结束后才运行 Codex/TUI Cargo 检查。

## Unchanged 与移动

plan_runtime_sync 保持只读。执行 Unchanged 时对已观测 runtime/snapshot 做版本校验的元数据加固；版本由原始文件字节生成。内容变化时安全跳过或重试，不改 bytes/mtime/file identity。rename 回退读取源字节、私有原子写目标成功后才删除源；提取窄私有函数，测试 closure 固定注入 rename 失败，不加公开 trait 或生产故障开关。P3 提供破坏前备份。

## 敏感信息

认证 JSON、tokens、完整身份、包含原始 auth map 的 sync plan 使用脱敏 Debug，磁盘 serde 格式保持。同类来源还包括 CodexRuntimeCommitPlan/CodexAuthCacheAction 的原始 auth map 和可能含 provider secret 的 config，以及既有 CodexProfileSecret.secret；同一脱敏边界覆盖这些字段，不改变其运行或序列化语义。HTTP 错误只输出状态和允许的固定错误码；未知 code/message/body 不回显。保留现有永久 refresh 错误码和需重新登录前缀；将内部识别的 token-invalidated 固定短语映射为固定码以保留刷新重试。移除无用 body_preview，UTF-8不崩溃由新脱敏测试覆盖。

同文件的私有 TokenRefreshRequest/TokenRefreshResponse 也包含原始 token，纳入 Debug 脱敏。OpenAiQuotaSnapshot 已有脱敏 Debug，保持原样；CodexAuthExportAccount.auth_data 使用 CodexAuthJson，其 Debug 随模型修复生效，无需修改导出 DTO。

P2 保留的 registry/account `extra` 是原始 TOML map。Debug 隐藏这些未知值，磁盘读写继续保留；合成 marker 回归覆盖两处，避免未知字段绕过凭据脱敏。

## 平台与回滚

Windows DACL 使用本机临时文件读取 ACE 实测，清除 USERNAME 不影响 SID；Unix 未具备环境时标记 NOT_RUN。本任务不修改错误变体、公共 DTO、真实账户或已批准保留策略。
