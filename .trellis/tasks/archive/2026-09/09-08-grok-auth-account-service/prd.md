# Grok 多账号服务与凭据一致性

## 目标

为 TUI 提供可保存、列表、覆盖、切换、删除的 Grok 官方 OAuth 个人/团队账号服务。复用现有 GrokAuthService 和 ccr-core，保证身份不明、并发和持久化失败不会覆盖错误账号。父任务研究记录官方 schema；本会话已按用户明确要求开始实施。

## 需求

- R1：固定单文件 <CCR_ROOT>/platforms/grok/auth/accounts.json（默认 ~/.ccr/platforms/grok/auth/accounts.json），按用户别名保存一个官方 scope 的完整 credential，Secret 包装；派生列表身份/时间，不复制 current 指针或建立注册表/数据库。
- R2：保存像 Codex 一样只读复制当前凭据，允许 Grok 运行且当前账号继续使用；不写 runtime/官方锁、不要求退出、不登录/登出/refresh；同名默认拒绝且明确确认才覆盖；来源 scope 多于一个时由调用者选；外部/企业/API-key/旧 web_login 不进入保存管理。
- R3：切换先保护当前同 scope 会话。内容精确对应一项保存记录时可直接保护；内容改变时只有唯一且明确的身份匹配才自动回存。未保存、变更后身份不明、歧义或冲突要求先明确保存。保留未知字段、原始时间和其他 scope。
- R4：删除仅动保存库；登出仍共享 auth_off 删除写核，已识别账号先回存，其余保存项保持。损坏 runtime 的明确登出仍可清除；其他操作不得把损坏 JSON 当空。
- R5：CCR 操作锁覆盖保存/切换/删除/登出；仅切换/登出安全打开官方同路径锁，保存/删除不占用官方锁；版本冲突不盲写；单文件原子写与权限保护，失败状态准确。
- R6：不自动 profile off，不碰 MCP，不自行登录/refresh，不输出 token；保持现有 CLI/Tauri current 成功 DTO 和 off 入口，新增账号能力只供 TUI，不加 CLI 命令组。

## 验收

- [ ] AC1（R1、R2）：A/B 保存至 CCR Grok 目录与列表；同名拒绝/明确覆盖、无官方 scope/多个候选、未知字段往返；官方锁被占用时仍可保存，缺失时不创建官方锁，CCR 零 runtime 写入、零登录/登出/refresh 调用；快照捕获后原生自行刷新不被回写覆盖；静态夹具 runtime/config/profile/MCP 不变，损坏库拒绝写入。
- [ ] AC2（R3）：A 刷新后切 B 再回 A 使用更新后的 A；原生登录 B 不被当成 A 更新；user_id unknown/enrichment 身份变化/重复候选拒绝错误自动回存；当前缺失可切换。
- [ ] AC3（R3、R6）：只更新目标 scope，enterprise/external/API-key、profile/config/MCP 不变；issuer/client/scope 不一致拒绝，create_time/expires_at 不改。
- [ ] AC4（R4）：删除当前槽不动 runtime；off 保留全部保存槽，已匹配账号回存失败则不删 runtime；损坏/未保存 runtime 的明确 off 保留既有清除语义。
- [ ] AC5（R5）：锁超时、CCR 并发 save/switch/delete/off、官方锁 holder 内容、CAS 冲突、回存写失败、runtime 替换失败、写后 durability 错误均有聚焦用例；无未保存凭据丢失。
- [ ] AC6（R5）：新建/临时 secret 文件 Unix 0600、Windows 当前用户 DACL，已有严格权限保留；权限设置失败发生在 secret 内容落盘前；原目标保留。
- [ ] AC7（R6）：DTO/错误/日志/Debug 不含原始凭据或 sentinel；current/off 既有入口和其他平台 auth_off 回归；无网络或真实凭据测试。
- [ ] AC8（R1–R6）：服务 API/规范交付给 TUI并完成静态审查；用户已确定停止后切换、新会话使用。本轮不运行 ccr-cli/core 测试、strict lint 或 runtime 验证，记录 SKIPPED（用户要求），不声称认证有效性或 native 行为已验证。

## 范围外与依赖

无新 crate/依赖/数据库/用户配置；不改 TUI，不创建全平台账号抽象，不添加 keyring、OAuth 客户端或账号用量。无代码前置依赖；实施授权和切换范围已确认，已进入实施。本轮不执行测试，不触碰当前 Grok Build 进程、真实凭据/配置或已安装二进制。其他 AC 中的行为与用例保留为实现要求，执行证据均不得预先勾选。
