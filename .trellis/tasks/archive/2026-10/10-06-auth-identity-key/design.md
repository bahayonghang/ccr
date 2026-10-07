# 设计：完整 OAuth 身份关联

## 身份规则

共享身份解析在 `ccr-codex` 内实现。从 id_token/access_token JWT 的顶层与 `https://api.openai.com/auth` claims 读取 `chatgpt_user_id` 与账号上下文；账号上下文沿用 `tokens.account_id` 优先规则。有效用户 claims 相互冲突时返回未知身份。空值、坏 JWT、仅 email 或裸 `sub` 不构成完整身份。JWT 解析只用于本地关联，不验证远端授权。身份键为 `user_id::account_id`，不增加 CLI/TUI/Tauri DTO 字段，不输出键到错误与日志；身份对象 Debug 脱敏。

注册表 `CodexAuthAccount` 添加可选身份键，缺失字段默认 None，保存 None 时不输出。P2 的未知字段保留提供兼容保护。API key/provider 继续使用原凭据指纹分支。

## 旧记录与只读边界

旧账号只能从该账号自己的快照推导身份；不得用当前 runtime 身份补全其他账号。读取/计划方法保持只读，可临时解析快照用于匹配。执行观测点持久化安全补全 delta，保持账号名与 current_auth；P2 只读注册表跳过元数据保存。无法取得完整 OAuth 身份时同步为 NoOp，且不改变凭据文件。

## 路径覆盖

- runtime 与已保存账号匹配、current_auth 对账、切换前写回按完整身份筛选。
- 同一完整身份有多个别名时，优先 current_auth，其次 last_used 最新者；平手保持有序集合顺序。
- 选中快照后再次确认快照身份等于 runtime 身份，随后执行已有 Unchanged/时间戳定向规则。
- OAuth repair 的 runtime/备份候选必须匹配完整身份，避免另一用户的较新令牌覆盖目标。
- 配额 runtime 路由在同步后重新读取身份。内存配额缓存按完整身份隔离；身份缺失时保留凭据指纹缓存隔离，不使用 account_id 单键。
- HTTP `ChatGPT-Account-Id` 与既有 usage ledger 继续使用 account_id，保持公共协议和用量语义。

## 执行时写回保护

独立检查确认，请求发起时的身份路由不足以保护异步配额刷新结果。刷新写回须比较请求源凭据/完整身份与当前文件，同时校验刷新结果身份，使用带预期内容版本的私有写入拒绝读后变化。身份未知时保留已有同文件quota刷新能力，以凭据/版本限制目标，不借account_id跨文件关联。

WriteRuntime不能直接提交旧计划；执行点复核registry、runtime、snapshot完整身份与新鲜度。RuntimeService增加crate内auth同步CAS入口，仅处理该分支实际auth写入，沿用备份、现有gw路径叶锁与secret原子写。CAS冲突不恢复旧auth备份；公开计划形状与其他commit行为保持。该保护约束CCR合作写入，不承诺外部codex/login的任意并发文件写入完全原子封闭。

## 旧账号手动查询

手动查询的 `force_refresh` 表示取得新配额观测并绕过缓存。共享 `OpenAiQuotaCore::fetch_quota` 当前将该标志同时用于强制 OAuth 刷新，会在 access token 仍有效时因失效 refresh token 提前失败。最小修复在共享核心移除该提前刷新条件，保留本地到期检查与额度端点认证拒绝后的刷新。调用接口与 TUI 查询确认保持。使用 loopback stub 证明 GET、刷新 POST 与持久化调用的次数；真实凭据诊断只记录脱敏状态，不将凭据写入任务证据。

## 参考实现补充：凭据操作互斥

用户要求参考 `ref/repo/cockpit-tools` 深入完善账号管理。参考的账号刷新锁在网络请求前持有，锁内重读 generation/token；CCR 的刷新后 CAS 只能拒绝错误写回，不能防止同一旧 refresh token 被两个请求先后或同时消费。

新增 crate-private 窄凭据操作锁，复用 `ccr-core` FileLock/LockManager 与内容版本哈希；完整身份只作为哈希输入，不进入锁名、错误或日志。async 获取走 spawn_blocking，锁覆盖锁内来源重读、网络刷新、凭据落盘与必要快照写回。未知身份只能以实际来源文件的稳定哈希隔离，不能据 workspace 建立跨文件身份关联。

切换在换出同步前取得换出及目标身份的锁，去重后稳定排序，持有至 runtime 提交；锁后验证身份未变。锁顺序为凭据操作锁 → 注册表锁/guarded path 叶锁，不反向取得操作锁。仅在必要内部调用点分出持锁入口，避免重入死锁，不改变公开 API/DTO。相同完整身份的多个别名需在锁内采用已落盘且足够新的合法来源；仅串行旧快照仍会重复提交被消费 token。优先复用现有 OAuth 候选与新鲜度合同，必要时纳入自有已登记同身份快照。

`save_current` 从一次读取的文档生成快照、email/account_id/identity_key；不再次读取 runtime 获取另一份元数据。该修复不据未验证 JWT 推导远端授权。外部 Codex/login 不遵守 CCR 操作锁的竞争仍依赖版本保护，不能承诺任意外部写入完全封闭。

## 来源校验与时间精度

独立预审补充：每个来源始终加入稳定规范化路径资源，再加入可得的完整身份资源；unknown 在同文件刷新后首次取得完整 claims 时也不能绕开原来源锁。对每个路径保留初始身份并逐路径验证，不以「当前身份属于整个双锁集合」代替原来源不变检查。unknown 请求若未持完整身份资源，不进行跨别名同步。

unknown 刷新首次落盘完整身份时，core 通过私有控制结果立即退出，早于额度 GET、第二次刷新及跨文件同步。外层释放旧锁，携带刚落盘的完整身份作为预期，最多一次重新取锁和读取最新候选。等待期间来源改为另一身份则安全拒绝，不将原请求改查另一账号。控制结果不进入公开错误、DTO 或日志。

新鲜度精度修复保持 last_refresh 优先、否则 mtime 的依据，比较完整 DateTime/SystemTime 精度，不截断到秒。同秒内先后轮换必须按实际时间区分。公开时间字段和 DTO 保持；不引入 AT-exp 优先级或 generation 字段。公共 sync outcome 保留，配额内部候选准备不改变公开同步报告。

## 兼容与回滚

完整身份的新字段为追加字段，不升注册表主版本。现有 OAuth 成功测试夹具补合成用户 claims，保留缺失身份 NoOp 专项断言。P2 只读和原子写合同保持。回滚产品改动不会迁移快照路径；回滚后的旧匹配规则仍有原始跨用户风险。
