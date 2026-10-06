# Codex Auth 参考加固审计报告

基线：分支 `dev`，提交 `7f6f7164`（子任务 1/2 已合并归档）。参考：父任务 `research/codex-auth-reference.md`（Loongphy/codex-auth，2026-10-06 读取）。

方法：静态代码阅读 + `rg` 检索 + 合成测试（临时目录、`TestCodexEnv`、合成凭据）。不涉及真实账户。

锚点约定：「HEAD」指修复前 `7f6f7164` 的行号；「现」指本任务修复后的行号。

## 1. 锚点复核

prd/design 引用的锚点在子任务 1/2 合并后发生偏移。复核结果如下。

| 原引用                           | HEAD 实际位置                           | 说明                                                                             |
| -------------------------------- | --------------------------------------- | -------------------------------------------------------------------------------- |
| codex_config.rs:41               | codex_config.rs:41                      | `MAX_BACKUPS = 10`，未变                                                         |
| codex_config.rs:271-299          | codex_config.rs:271-310                 | `cleanup_old_backups`，函数尾部延长                                              |
| codex_auth_service.rs:908        | codex_auth_service.rs:923               | `save_current` 中的 `ensure_private_permissions`                                 |
| codex_auth_service.rs:1509       | codex_auth_service.rs:1598              | `rename_account` 中的 `ensure_private_permissions`                               |
| codex_auth_service.rs:1964       | codex_auth_service.rs:2053              | `import_accounts` 中的 `ensure_private_permissions`                              |
| codex_auth_service.rs:714        | codex_auth_service.rs:684-692、:729-737 | `matched_saved_account_name` / `sync_current_auth_registry` 按 `account_id` 匹配 |
| codex_oauth_token_service.rs:333 | codex_oauth_token_service.rs:507-544    | `select_sync_target` / `tokens_account_id`                                       |
| codex_auth_service.rs:1223       | codex_auth_service.rs:1310-1337         | `backup_current_auth`（`#[allow(dead_code)]`）                                   |
| codex_auth_service.rs:1700       | codex_auth_service.rs:1789              | `export_accounts` 起始                                                           |
| backend-guidelines:93            | backend-guidelines.md:93                | 「Never log access tokens …」，未变                                              |

## 2. 维度表

| 维度         | CCR 现状（HEAD）                                                                                                                                     | codex-auth 做法                         | 结论                                 | 类别         | Finding                  |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------- | ------------------------------------ | ------------ | ------------------------ |
| 备份策略     | 每次 `commit_plan` 写前无条件备份；秒级文件名，同秒覆盖；`auth`/`config` 前缀各保留 10 个                                                            | 仅内容变化时备份；同秒加 `.N`；保留 5   | 去重缺失、同秒覆盖                   | A（已修）    | F1、F2、F3               |
| 权限加固     | 运行时与快照同步路径在写入前设置私有权限；`save_current`、导入、账号快照备份、注册表写入存在「写入后再加固」或「继承目录 ACL」窗口；无变化路径不加固 | 每次接触执行 `hardenSensitiveFile`      | 写路径有缺口；无变化路径不加固       | A（已修）/ B | F4、F5、F6、F7、F20、F24 |
| 原子写       | 运行时 `auth.json`/`config.toml` 走 `write_guarded`；`save_current` 用 `fs::copy`，导入用 `fs::write`，回滚用 `fs::copy`                             | `replaceFilePreservingPermissions`      | 三处非原子写                         | A（已修）/ B | F4、F5、F8、F23          |
| 身份模型     | 仅 `account_id`                                                                                                                                      | `chatgpt_user_id::chatgpt_account_id`   | 同一 workspace 多用户无法区分        | B            | F12                      |
| 注册表健壮性 | 无 `version` 门；未知字段在重写时丢弃；注册表备份不清理、同秒覆盖                                                                                    | `schema_version` 迁移门，新版本拒绝加载 | 前向兼容缺失                         | B            | F10、F11、F14            |
| 导入/导出    | 明文导出含完整 `auth_data`；导入的 `account_id` 取自导出元数据                                                                                       | import/export，密钥只入快照，展示用指纹 | 明文导出由调用方选择；导入身份未校验 | B            | F15、F16                 |
| 日志脱敏     | `tracing` 未输出 token 值；`CodexAuthJson`/`CodexAuthTokens` 派生明文 `Debug`；配额错误文本含响应体预览，且预览切片可能 panic                        | —                                       | 无现存泄漏；预览切片 panic           | A（已修）/ B | F17、F18、F19            |
| 清理/保留    | `backup_current_auth` 与 `cleanup_old_backups` 无调用方；`auth/backups` 下注册表与账号备份从不清理                                                   | `clean` 白名单                          | 死代码；无界增长                     | A（已修）/ B | F9、F10                  |
| 文件命名     | 账号名直接作为快照文件名；`validate_account_name` 限定 `[A-Za-z0-9_-]`、≤32                                                                          | record_key 派生 + base64url             | 校验阻止路径穿越；名称与身份解耦     | B            | F13                      |

## 3. Findings

分级：高 = 可导致凭据泄露或不可恢复丢失；中 = 特定流程下丢失恢复来源、权限窗口或进程崩溃；低 = 边缘条件或纵深防御。

### F1 备份无内容去重（中，A，已修复）

- 证据：HEAD `managers/codex_config.rs:237-268`
  ```rust
  let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
  let filename = format!("{}.{}.{}.{}.bak", prefix, label, timestamp, ext);
  ...
  AtomicWriter::new(&backup_path).secret(true).write(&content)?;
  ```
  `switch_account` 每次都传 `config: Some(..)`（HEAD `codex_auth_service.rs:1220-1228`），所以每次切换都会创建一份 `config` 备份，内容通常不变。
- codex-auth：仅内容变化时创建备份。
- 风险：相同内容的备份占用 10 个保留位，挤出较早的不同版本。
- 修复：现 `codex_config.rs:255-264`。与同前缀最新备份字节一致时，返回该既有路径并刷新其 mtime，不创建新文件、不触发清理。返回路径的字节与源文件一致，`commit_plan` 回滚（`restore_optional_backup`）仍从该路径恢复。刷新 mtime 使该文件保持「最新备份」位置，`resolve_latest_oauth_doc`（`codex_oauth_token_service.rs:242-310`）按 mtime 排序的结果与修复前一致；mtime 刷新失败时回退为创建新备份。
- 测试：`managers::codex_config::tests::backup_reuses_latest_identical_backup_path`、`services::codex_runtime_service::tests::commit_plan_rollback_restores_config_from_deduplicated_backup`。

### F2 同秒备份同名覆盖（中，A，已修复）

- 证据：同 F1，文件名只到秒，`AtomicWriter` 对已存在目标执行替换。`switch_account` 先执行 `sync_runtime_with_saved_account_best_effort`（可能经 `commit_plan` 写回 runtime，HEAD `codex_auth_service.rs:1188`、`:1138-1143`），随后执行切换的 `commit_plan`（`:1221`）。两次 `backup_auth("runtime_switch")` 常落在同一秒，第二次覆盖第一次。
- codex-auth：同秒冲突追加 `.N`。
- 风险：同秒内第一份不同内容的备份丢失；并发进程的回滚可能读取到被替换的内容。
- 修复：现 `codex_config.rs:280-297` `unique_backup_path`，冲突时生成 `{prefix}.{label}.{ts}_{N}.{ext}.bak`。前缀与 `.json.bak` 后缀不变，清理过滤与 `resolve_latest_oauth_doc` 过滤（`starts_with("auth.") && ends_with(".json.bak")`）仍匹配。
- 测试：`managers::codex_config::tests::backup_creates_distinct_file_when_content_changes`、`managers::codex_config::tests::unique_backup_path_appends_counter_on_collision`。

### F3 保留池数量（低，信息项）

- 证据：`codex_config.rs:41` `MAX_BACKUPS = 10`；清理按 `starts_with(prefix)` 匹配，`auth.*` 各标签共用一个 10 个文件的池。
- codex-auth：保留 5。
- 处置：不缩减。修复后排序增加文件名倒序作为同 mtime 的次序（现 `codex_config.rs:299-323`），数量不变。
- 测试：`managers::codex_config::tests::backup_retention_pool_keeps_ten_distinct_versions`（12 个不同版本后保留 10 个）。

### F4 `save_current` 非原子复制且写入后再加固权限（中，A，已修复）

- 证据：HEAD `codex_auth_service.rs:916-923`
  ```rust
  fs::copy(&src, &dst)...;
  crate::utils::ensure_private_permissions(&dst);
  ```
- codex-auth：替换写入并保留权限。
- 风险：快照含 refresh token。`fs::copy` 非原子，中断时快照被截断；在 Windows 上新文件先继承目录 ACL，随后才由 `icacls` 收紧。
- 修复：现 `codex_auth_service.rs:914-925`，读源字节后 `AtomicWriter::secret(true)` 写入，保留原 `ensure_private_permissions` 调用与原错误前缀「复制 auth.json 失败: 」。
- 测试：`services::codex_auth_service::tests::save_current_snapshot_matches_runtime_bytes`（字节一致、force 覆盖；Unix 上断言 `0o600`）。

### F5 导入写快照用 `fs::write`（中，A，已修复）

- 证据：HEAD `codex_auth_service.rs:2048-2053` `fs::write(&auth_path, auth_content)` 后 `ensure_private_permissions`。
- 风险：同 F4（截断 + 权限窗口）。
- 修复：现 `codex_auth_service.rs:1990-1998`，`AtomicWriter::secret(true).write_string`，错误文本保持「写入 auth 文件失败 (账号: …): …」。只读检查（`:1977-1986`）不变。
- 验证：既有 `test_import_accounts_*` 6 个测试全部通过（见第 6 节）。

### F6 账号快照备份用 `fs::copy`（低，A，已修复）

- 证据：HEAD `codex_auth_service.rs:1359-1377` `fs::copy(&auth_path, &backup_path)`。Unix 上复制源文件模式（源为 `0o644` 时备份同为 `0o644`）；Windows 上新文件继承目录 ACL。
- 修复：现 `codex_auth_service.rs:1332-1356`，`AtomicWriter::secret(true)`，错误文本「备份 auth 文件失败: 」不变。
- 测试：`services::codex_auth_service::tests::account_snapshot_backup_is_owner_only`（`#[cfg(unix)]`，本机 Windows 未编译运行：NOT_RUN）。

### F7 注册表写入后再加固权限（低，A，已修复）

- 证据：HEAD `codex_registry_store.rs:74-78` `AtomicWriter::new(..).write_string(..)` 后 `ensure_private_permissions`。注册表含 `account_id`、脱敏邮箱与 usage 账本。
- 修复：现 `codex_registry_store.rs:74-79` 增加 `.secret(true)`；保留原后置加固调用。
- 测试：`services::codex_registry_store::tests::save_writes_owner_only_registry`（`#[cfg(unix)]`：NOT_RUN）。

### F8 回滚用 `fs::copy` 覆盖运行时文件（中，A，已修复）

- 证据：HEAD `codex_runtime_service.rs:370-375`
  ```rust
  fs::copy(backup, target).map_err(|e| CcrError::ConfigError(format!("回滚文件失败 {:?} <- {:?}: {}", ..)))?;
  ```
- 风险：回滚目标是 `~/.codex/auth.json` / `config.toml`。回滚中断会截断正在保护的文件。
- 修复：现 `codex_runtime_service.rs:371-382`，读备份字节后 `AtomicWriter::secret(true)` 替换；不加锁（与原 `fs::copy` 一致，避免锁超时改变失败语义）；错误文本格式不变。`write_journal::contains` 早返回逻辑不变。
- 测试：`services::codex_runtime_service::tests::commit_plan_rollback_restores_config_from_deduplicated_backup`（config 写入成功、auth 写入失败后，config 从去重复用的既有备份路径恢复，且未产生新的 config 备份）。

### F9 死代码 `backup_current_auth`（低，A，已移除）

- 证据：HEAD `codex_auth_service.rs:36-38`（`MAX_BACKUPS`）、`:1310-1337`（`backup_current_auth`）、`:1379-1413`（`cleanup_old_backups`），均带 `#[allow(dead_code)]`。`rg` 全仓库仅有两个单元测试调用。该函数的清理过滤 `starts_with("auth_") && ends_with(".json")` 会同时删除 `auth_account_*` 账号备份。
- 处置：移除三者及其测试 `test_backup_current_auth`、`test_backup_rotation`。生产路径无调用方，运行行为不变。`ccr-codex` 为工作区内部 crate，`rg` 未发现 `ccr-ui/src-tauri` 或其他 crate 调用。
- 验证：`just lint-strict` 与 `just test` 通过。

### F10 `auth/backups` 下注册表与账号备份无界增长（低，B）

- 证据：`codex_registry_store.rs:72` 每次 `save` 前 `let _ = self.backup();`；`:83-101` 写 `auth_registry_{ts}.toml`；账号备份见现 `codex_auth_service.rs:1332-1356`。修复前唯一清理函数是 F9 的死代码。
- 风险：长期运行累积大量文件。
- 处置：增加清理会把保留数从「无界」降为有限值，属于保留池缩减，列为提案 P3。

### F11 注册表备份同秒覆盖（低，B）

- 证据：`codex_registry_store.rs:93-98`、现 `codex_auth_service.rs:1312-1330`，秒级文件名 + `fs::copy` 覆盖。`fs::copy` 也使注册表备份的权限跟随源文件模式（Unix）或继承目录 ACL（Windows），未在写入前设置私有权限；注册表不含 token，风险低。
- 处置：与 F10 一起设计注册表备份策略，列为提案 P3。

### F12 身份仅 `account_id`（中，B）

- 证据：`codex_auth_service.rs:682-692`、`:729-737`；`codex_oauth_token_service.rs:507-544`。
- codex-auth：`record_key = chatgpt_user_id::chatgpt_account_id`。
- 风险：同一 ChatGPT workspace（Team/Enterprise）下的不同用户共享 `account_id`，同步可能把用户 A 的 runtime tokens 写入用户 B 的快照。子任务 1 已用「current_auth 优先、其次 last_used」规则限定目标，但不能区分用户。
- 处置：改变匹配语义，列为提案 P1。

### F13 账号名直接作为文件名（低，B）

- 证据：`codex_auth_service.rs:186-188` `format!("{}.json", name)`；`:1582-1610` 校验字符集与长度。
- 结论：当前校验阻止路径穿越与非法字符，无安全缺口。文件名与身份解耦属于设计变更，列为提案 P5。

### F14 注册表无版本门且丢弃未知字段（中，B）

- 证据：`models/codex_auth.rs:291-308` `CodexAuthRegistry` 无 `#[serde(flatten)]` 扩展字段；`codex_registry_store.rs:45-55` 加载不检查 `version`。
- codex-auth：`schema_version` 门，新版本拒绝加载。
- 风险：较新版本 CCR 写入的新字段被较旧版本加载后重写时丢失。
- 处置：拒绝加载或保留未知字段都会改变行为与结构定义，列为提案 P2。

### F15 明文导出包含完整 `auth_data`（低，B）

- 证据：HEAD `codex_auth_service.rs:1801-1818`（现 `:1743-1760`）`include_secrets` 为真时读取完整快照。加密导出 `export_accounts_encrypted` 已存在。
- 处置：由调用方显式选择。是否默认强制加密属于用户可见行为，列为提案 P6。

### F16 导入不校验 `account_id` 与 tokens 一致（低，B）

- 证据：HEAD `codex_auth_service.rs:2059-2071`（现 `:2001-2013`）`account_id: import_account.account_id` 直接取自导出元数据。
- 风险：被篡改或手工编辑的导出文件可让快照与注册表身份不一致，影响 F12 的同步目标选择。
- 处置：增加校验会拒绝部分既有输入，列为提案 P6。

### F17 认证模型派生明文 `Debug`（低，B）

- 证据：`models/codex_auth.rs:572-589` `CodexAuthJson`、`CodexAuthTokens` `#[derive(Debug, ..)]`，token 字段为 `String`。`rg` 检索 `ccr-codex/src` 中日志宏与 `{:?}` 用法，未发现对这两个类型或 `ResolvedOAuthDoc` 的 Debug 输出。
- 结论：无现存泄漏，属于纵深防御。改为脱敏 `Debug` 会改变公开类型的 Debug 输出，列为提案 P4。

### F18 配额与刷新错误文本包含响应体预览（低，B）

- 证据：`openai_quota_core.rs:446-454`、`:486-494`，错误消息拼接前 200/300 字节响应体，经 quota error 显示在 TUI。
- 风险：响应体可能含账号描述信息。规范对桌面 OAuth 控制器要求「report status or a fixed diagnostic, never response bodies」，配额路径尚未对齐。
- 处置：修改会改变 TUI 可见的 quota error 文本，列为提案 P4。

### F19 响应体预览切片可能 panic（中，A，已修复）

- 证据：HEAD `openai_quota_core.rs:443-447`、`:487-491`
  ```rust
  let body_preview = if body.len() > 200 { &body[..200] } else { &body };
  ```
  第 200/300 字节落在多字节 UTF-8 字符内部时，切片 panic。
- 修复：现 `openai_quota_core.rs:156-159` `body_preview` 使用 `floor_char_boundary`。ASCII 与较短响应体的输出与修复前逐字节一致；仅原本 panic 的输入改为在字符边界截断。
- 测试：`services::openai_quota_core::tests::body_preview_keeps_ascii_prefix_and_never_splits_multibyte_chars`。

### F20 无变化路径不加固权限（低，B）

- 证据：`codex_oauth_token_service.rs:455-457` tokens 相同时返回 `Unchanged`，不接触文件。
- codex-auth：内容相同时执行 `hardenSensitiveFile`。
- 处置：在读路径增加元数据副作用；Windows 上 `ensure_private_permissions` 每次启动 `icacls` 进程。规范要求 `Unchanged` 时「bytes and mtime unchanged」。列为提案 P4。

### F21 `delete_account` 删除快照前不备份（中，B）

- 证据：现 `codex_auth_service.rs:1288-1293` 直接 `fs::remove_file(&auth_path)`。规范要求「backup-before-destructive-change」。
- 处置：增加备份会额外保留用户要求删除的凭据，涉及保留语义，列为提案 P3。

### F22 `rename_account --force` 忽略备份失败后删除目标（中，B）

- 证据：现 `codex_auth_service.rs:1507` `let _ = self.backup_account_auth(new_name);` 随后 `:1509-1513` 删除冲突快照；`:1519-1520` 同样忽略源快照与注册表备份失败。
- 处置：改为备份失败即中止会改变命令结果，列为提案 P3。

### F23 `rename_account` 回退分支 `fs::copy` 后再加固（低，A，未实施）

- 证据：现 `codex_auth_service.rs:1529-1540`，同目录 `fs::rename` 失败时回退 `fs::copy` + `remove_file` + `ensure_private_permissions`。
- 未实施原因：同目录 rename 失败的条件无法在测试中稳定构造，无法提供回归测试。纳入提案 P4。

### F24 Windows 权限加固依赖 `icacls` 与 `USERNAME`（低，B）

- 证据：`utils.rs:87-118`，Windows 分支读取 `USERNAME` 环境变量后执行 `icacls /inheritance:r /grant:r`，忽略失败。
- 风险：`USERNAME` 为空或执行失败时静默跳过。`ccr-core` 已提供基于进程令牌用户 SID 的 DACL 写入（`AtomicWriter::secret`），本任务的修复已在写入前使用该机制。
- 处置：替换 `ensure_private_permissions` 实现会改变全部调用点行为，列为提案 P4。

### 合规项（无缺口）

- 运行时 `auth.json`/`config.toml`：`codex_config.rs:207-216` `write_guarded` + `secret: true`。
- 快照同步：`codex_oauth_token_service.rs:344-347` `AtomicWriter::secret(true)`。
- 配额刷新回写：`codex_quota_service.rs:611-617` `AsyncAtomicWriter::secret(true).preserve_mode(true)`。
- 日志：`rg` 检索日志宏，未发现 access/refresh token、API key 或原始 auth JSON 输出。
- 机制 1（登录隔离）：CCR 无法拦截外部 `codex login`，由子任务 1 以观测点回写与文档覆盖；本任务不涉及。

## 4. A 类修复映射

| Finding                  | 修复位置（现）                              | 回归测试 / 验证                                                                                               |
| ------------------------ | ------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| F1 备份去重              | `managers/codex_config.rs:255-264`          | `backup_reuses_latest_identical_backup_path`；`commit_plan_rollback_restores_config_from_deduplicated_backup` |
| F2 同秒冲突              | `managers/codex_config.rs:280-297`          | `backup_creates_distinct_file_when_content_changes`；`unique_backup_path_appends_counter_on_collision`        |
| F3 保留池不缩减          | `managers/codex_config.rs:299-337`          | `backup_retention_pool_keeps_ten_distinct_versions`                                                           |
| F4 `save_current` 原子写 | `services/codex_auth_service.rs:914-925`    | `save_current_snapshot_matches_runtime_bytes`                                                                 |
| F5 导入原子写            | `services/codex_auth_service.rs:1990-1998`  | 既有 `test_import_accounts_*`（6 个）                                                                         |
| F6 账号备份私有权限      | `services/codex_auth_service.rs:1332-1356`  | `account_snapshot_backup_is_owner_only`（Unix，NOT_RUN）                                                      |
| F7 注册表写入前加固      | `services/codex_registry_store.rs:74-79`    | `save_writes_owner_only_registry`（Unix，NOT_RUN）                                                            |
| F8 回滚原子写            | `services/codex_runtime_service.rs:371-382` | `commit_plan_rollback_restores_config_from_deduplicated_backup`                                               |
| F9 死代码移除            | `services/codex_auth_service.rs`（删除）    | `just lint-strict`、`just test`                                                                               |
| F19 切片 panic           | `services/openai_quota_core.rs:156-159`     | `body_preview_keeps_ascii_prefix_and_never_splits_multibyte_chars`                                            |

行为说明：

- 去重命中时不新建备份、不触发清理，只刷新既有备份的 mtime。`commit_plan` 回滚契约保持：返回路径存在且字节等于写入前的源文件。
- 失败路径的错误文本保留原前缀；内层原因从 `io::Error` 文本变为 `CcrError` 文本（例如 `save_current` 复制失败）。这些文本只在 I/O 失败时出现。`save_current` 的错误会进入 TUI toast（`ccr-tui/src/tui/codex_auth/app.rs:1056-1060`「保存失败：{e}」）和桌面端命令结果，所以 I/O 失败时 toast 的内层原因文本会变化；toast 前缀与「复制 auth.json 失败: 」前缀不变。quota error 文本不受影响。`rg` 未发现对这些内层原因文本的测试断言。
- `save_current` 在 Unix 上覆盖只读（`0o400`）旧快照：`fs::copy` 原本以权限错误失败；原子替换可以成功，随后 `ensure_private_permissions` 设为 `0o600`。该差异只在手工把快照设为只读时出现。
- 测试移除：`test_backup_current_auth`、`test_backup_rotation` 随死代码一起移除。其他既有测试未修改。

## 5. 后续任务提案（B 类，本任务不实施）

| 编号                      | 位置                                                                                                                                                                     | 证据                    | 建议                                                                                                                                                                                                                                                       |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| P1 身份模型               | `codex_auth_service.rs:682-692`、`:729-737`；`codex_oauth_token_service.rs:507-544`                                                                                      | F12                     | 引入 `chatgpt_user_id::chatgpt_account_id` 复合键（从 id_token/access_token claims 读取 user_id），注册表保存该键，旧记录在首次观测时补全；匹配规则需与子任务 1 的同步合同一起评审。                                                                       |
| P2 注册表版本门与未知字段 | `models/codex_auth.rs:291-308`；`codex_registry_store.rs:45-55`                                                                                                          | F14                     | 增加 `schema_version` 门：未知较新版本拒绝写入并给出提示；或增加 `#[serde(flatten)] extra` 保留未知字段。需评估 `CodexAuthRegistry` 结构字面量构造点的影响。                                                                                               |
| P3 备份与破坏性操作       | `codex_registry_store.rs:72`、`:83-101`；`codex_auth_service.rs:1288-1293`、`:1507-1520`                                                                                 | F10、F11、F21、F22      | 注册表备份采用去重 + 同秒序号 + 有界保留；`delete_account` 删除前备份快照（需决定该备份的保留期）；`rename_account --force` 备份失败时中止。三项均改变保留数量或命令结果，需用户确认。                                                                     |
| P4 权限与日志纵深防御     | `utils.rs:87-118`；`codex_oauth_token_service.rs:455-457`；`codex_auth_service.rs:1529-1540`；`models/codex_auth.rs:572-589`；`openai_quota_core.rs:446-454`、`:486-494` | F17、F18、F20、F23、F24 | 用 `ccr-core` 的 SID DACL 帮助函数替代 `icacls` + `USERNAME`；`Unchanged` 路径按 `enforce_secret_permissions_versioned` 做元数据加固；rename 回退分支改为原子私有写；为认证模型实现脱敏 `Debug`；配额与刷新错误只保留状态码与错误码（改变 TUI 可见文本）。 |
| P5 快照文件命名           | `codex_auth_service.rs:186-188`、`:1582-1610`                                                                                                                            | F13                     | 评估以身份键派生文件名（base64url），账号名只作展示与别名；需要迁移现有快照。                                                                                                                                                                              |
| P6 导入/导出              | `codex_auth_service.rs:1743-1760`、`:2001-2013`                                                                                                                          | F15、F16                | 导入时从 tokens 推导 `account_id` 并与元数据比对，不一致时拒绝或告警；评估默认使用加密导出。                                                                                                                                                               |

## 6. 验证记录

运行环境：Windows 11，`rustc 1.98.0`，本机已安装目标仅 `x86_64-pc-windows-msvc`。

| 命令                                                               | 结果                                                                        |
| ------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| `cargo test -p ccr-codex --all-features -- --skip export_bindings` | `test result: ok. 291 passed; 0 failed; 2 ignored`                          |
| 新增与相关测试按名过滤运行                                         | `test result: ok. 11 passed; 0 failed`                                      |
| `just lint-strict`                                                 | `Sensitive persistence policy check passed`；`✅ 严格 Clippy 检查通过`      |
| `just test`                                                        | 全部 `test result` 行为 `ok`，`0 failed`（`ccr-codex` 单元测试 291 passed） |
| `just fmt-check`                                                   | `✅ 代码格式符合规范`                                                       |

NOT_RUN：

- `#[cfg(unix)]` 测试（`account_snapshot_backup_is_owner_only`、`save_writes_owner_only_registry`、既有 `secret_runtime_and_backup_files_are_owner_only`）：本机为 Windows 且无 Linux 目标，未编译、未运行。需在 Linux/macOS CI 上确认。
- Windows DACL 实测（备份与快照的 ACE 内容）：未运行专门检查；依赖 `ccr-core` 既有 Windows DACL 测试。
- `just ci` 全量门：未运行。
- 真实账户行为：不在范围内。
