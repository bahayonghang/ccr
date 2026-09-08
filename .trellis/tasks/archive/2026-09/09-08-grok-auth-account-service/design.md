# Grok 账号服务设计

保存已确定为 Codex 式只读复制：允许 Grok 正在运行，仅向 CCR 目录保存副本。切换已由用户确认为“先结束 Grok，写入后供新会话使用”；由用户自行停止，CCR 不探测或终止进程。协议证据见 ../09-08-tui-grok-auth-replacement/research/grok-multi-account.md。

## 文件与类型

继续扩展 crates/ccr-cli/src/services/grok_auth_service.rs；内部 credential/store 细节可拆到相邻 grok_auth/ 模块，不建立新 crate。持久化文件为 <CCR_ROOT>/platforms/grok/auth/accounts.json，原生文件路径复用 application/auth_off.rs:341。

账号库结构仅 accounts map。每项：saved_at、scope、credential: Secret（完整对象的 JSON 文本，持久化显式 expose_plaintext）。不存 current 指针、第二 registry、last-used 日志或冗余身份副本。DTO 与持久化类型分离，凭据集合不派生泄漏内容的 Debug，未知字段只在受控解析/写入边界存在。

read_snapshot() 返回脱敏账号条目、官方 runtime scope 候选、匹配状态/错误、以及内存用观察版本（store/runtime 内容 token，不显示/不记日志）。账号条目含别名、可用邮箱/团队、保存时间和本地到期状态。保存账号允许元数据缺失，但不得由此自动判断身份。

现有 GrokAuthCurrent.logged_in 保持文件存在性字段和成功 DTO；新的完整 snapshot 不复用该 bool 表示认证有效。

## 支持边界与身份

只接受 auth_mode=oidc、生产 issuer https://auth.x.ai 的个人/团队对象。scope 必须等于 credential.oidc_issuer 去尾斜杠 + :: + oidc_client_id；不硬编码 client ID 别名表。key、create_time 与 OAuth 格式按固定官方 schema 检查，refresh_token/expires_at 可缺失。全对象和未知字段保留。API-key/external/企业 OIDC/旧 web_login 留在 map，不写入管理库。

来源选择：恰有一个可保存 scope 时直接使用；多个则 TUI 选择服务返回的来源；没有则提供原生登录指引。读取 auth.json 不代表解析了有效 Grok 配置，因此 UI 标“本地 scope 匹配”，不标“当前请求必定使用”。

自动匹配先看同 scope 完整 credential 的规范化内容相等；否则只有完整且唯一的 scope/user_id/principal_type/principal_id/team_id 保守精确身份一致才匹配。“完整”要求有效 user_id 及无矛盾的可选信息；个人账号 principal/team 在两边均为 None 可以相等，不要求个人账号具备团队字段。一侧缺失一侧出现或字段值变化不自动推断为同账号。user_id 缺失/空/unknown、principal/team 信息变化或多候选都算不确定；email 只展示。没有 current 指针回存路径。同一确定身份重复存为第二别名时要求更新原槽，防制造歧义；身份未知时仍可显式保存，后续仅按确切内容匹配。

## 服务接口形状

- read_snapshot()：纯读；缺失库为空，损坏/权限错误为错误。
- save_current(name, source_scope, observed_revision, replace)：服务校验别名、来源和确认版本后保存；replace 只表示用户确认覆盖同名项。
- switch_account(name, observed_revision)：保护原账号再替换同 scope。
- delete_account(name, observed_revision)：只原子更新账号库；删当前保存项后 runtime 变为未保存状态。
- off_checked(observed_revision)：TUI 确认调用，复用共享 Grok off 内部写核并检查确认对象没变；既有 CLI off 仍是显式清除当前凭据的命令。

实现可以根据现有类型风格调整名称，但语义不得简化为未验证的直接覆盖。新公共导出只给真实 TUI 消费者，遵守 facade 边界。

## 锁与写入顺序

1. 统一 Grok auth 操作资源锁，所有 CCR save/switch/delete/off 都经过它；资源区分真实 runtime/store 目标路径，操作锁文件留在 CCR 自身锁目录。先完整操作锁，再按需获取官方 runtime 锁，最后叶子 guarded-write 路径锁，不能倒序。
2. 仅写 runtime 的 switch/off 打开 GROK_HOME/auth.json.lock 的同一个文件；save/delete 不获取、不创建或修改该锁。先将 ccr-core FileLock::new 的 truncate(true) 改为 truncate(false)，保留 holder 信息，不写 PID、不删除官方锁。fs4 已在 core，无新增依赖。
3. 持 CCR 操作锁后重读操作涉及的 store/runtime，检查服务返回的 observed_revision。冲突返回“已变化，请刷新”，不以旧确认覆盖新身份。save 从同一次完整读取的字节解析来源、身份和待保存 credential；原生刷新后续可以继续，CCR 不把快照写回 runtime。纯读取不创建目录/锁文件。
4. 需要写入时用 write_guarded_versioned + secret:true + BackupPolicy::None。保存库单文件无独立 registry 提交；save 仅写账号库，不要求停止 Grok。官方锁不能保证不遵循同锁的外部写路径，CAS 也不消灭最终检查后的竞争，所以已定切换契约要求用户先停止其他 Grok writer。
5. 锁等待在 TUI spawn_blocking 中完成，已有10秒级有界超时；不让 render 线程阻塞。测试夹具覆盖获取顺序和竞争，不将其描述成真实运行中全保证。

## 保存与切换步骤

save：只读取得完整 runtime 快照并核对观察版本 -> 校验名称/来源/同名确认 -> 更新一项账号库 -> 原子写至 <CCR_ROOT>/platforms/grok/auth/accounts.json（默认 ~/.ccr/platforms/grok/auth/accounts.json）。允许 Grok 运行，不检查进程是否退出，不登录、登出或调用 refresh，不改 runtime/官方锁/config/profile/MCP；当前账号继续使用。快照读取失败或解析失败只报错，不修复源文件。原生在快照读取后的自行刷新不阻止本次保存，后续显式切换仍按 outgoing 保护流程回存最新凭据。别名只作为 map key，不拼成任意路径；沿用现有账号命名约束，不另建通用验证框架。

switch：
- 先校验目标存在/结构/范围，校验观察版本；原 native 文档不可解析时拒绝覆盖。
- 仅检查目标 scope 的 outgoing。缺失允许；已保存且唯一匹配则把当前完整对象更新到对应保存槽。未保存或不确定返回需保存，不提供隐式丢弃。
- 原账号回存必须完成；同账号切换以当前较新 runtime 为准更新保存项后 no-op，不能拿旧快照倒灌。
- 从更新后的库取得目标；只替换目标 scope，其他 entries 保留语义值；原生 credential 时间/隐私/未知字段不改。
- 原子 CAS 写 runtime，随后重新读取状态。无 current 指针，不创建归属不一致的第二阶段提交。
- 原账号回存成功、runtime 写失败时保留最新保存项，不回滚成旧 token；错误反馈区分“原账号已保存”和“切换未完成”。
- writer 在 rename 后返回 durability 错误时回读：如果目标字节已写入，报告“已应用，持久化未确认”，不可说原文件必然没变，也不盲目恢复覆盖外部数据。回读失败则状态未知，重试须刷新。

## 删除和登出

delete 只更新单库，不触及 runtime。最后一项删除后保留合法空库，不递归删目录。

off 协调集中在共享 Grok 分支：锁只获取一次，内部分离“已持锁写核”避免服务/应用递归锁。可解析且唯一匹配的已保存 scope 在删除前更新库，保存失败阻止删除；未知/损坏 runtime 的显式 off 仍按原语义清除，UI 明示未保存数据将删除。CLI/Tauri 既有 off 也经过统一锁和同一写核。

off 保留原全 auth.json 删除、备份与失败回滚；这包含文件中其他 scope/API-key 缓存，不是删除选中 CCR 账号。不改 MCP/config/profile。changed=false 只说未删除凭据，实际缺失/未知由回读确定。

## 新存储文件权限（R5）

现有 AtomicWriter 在 Windows 仅保留已有目标 DACL，新建 secret 文件缺 owner-only 初始化。只在 ccr-core/src/core/atomic_writer.rs 现有 secret 分支补齐新目标：临时文件写入内容之前设置当前用户私有 DACL；sync/async 复用同一内部 WinAPI helper，已有 DACL/Unix 严格 mode 保留，非secret行为不变。无新公开 option、权限策略层或依赖。

账号库和 runtime 缺失后创建均用同一 secret 写路径；不依赖事后 chmod，不创建旁路明文备份。core lock/atomic-writer 的规范同步，只涉及本次真实调用契约。

## 失败和证据

覆盖空库、损坏库、无会话、多 scope、过期目标、名称冲突、外部换号、身份 enrichment、锁/CAS、回存失败、runtime 写失败、写后报错、secret 输出、权限失败。过期快照不等于无效，允许显式切换但显示本地过期，认证和 refresh 由官方决定。

保存的独立回归：官方锁被另一进程持有时仍可只读保存；无官方锁时不会创建它；静态夹具的 runtime/config/profile/MCP 字节不变；读取快照后模拟原生刷新，保存副本来自完整捕获内容且不回写覆盖新 runtime。此并发用例分别检查 CCR 零 runtime 写入和外部 writer 的自然变化，不把“源文件永不变化”作为保存成功条件。

真实服务端轮换规则、跨 fs2/fs4 原生运行竞争与新会话认证有效性均 UNVERIFIED；已运行 Grok 的即时换号不在范围内。本轮按用户要求不运行测试、不操作真实凭据或当前 Grok Build；上述用例仅维护源码，不执行。
