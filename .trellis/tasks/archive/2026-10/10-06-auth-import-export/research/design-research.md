# P6 源码研究

来源：父任务 research/followup-design-research.md；2026-10-06。仅为只读研究，行号在实施前复核。未运行产品验收。

 导入身份校验、原子边界与现有加密入口

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

