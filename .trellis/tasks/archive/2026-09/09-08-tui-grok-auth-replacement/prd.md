# TUI OpenCode Auth 清理与 Grok 多账号管理

## 目标与授权

移除 OpenCode Auth 遗留代码和测试，完善 Grok Auth，使用户可以保存多个官方账号、查看列表并切换。用户已授权实施，并确认先结束当前 Grok、切换后供新会话使用；保存仍可在 Grok 运行时只读复制。用户要求不运行测试，不影响当前正在运行的 Grok Build：不得操作该进程、真实凭据或配置，不全局安装或替换当前使用的二进制。提交/发布不在授权内。最终规划审阅与实施授权均已完成；2026-09-08 本会话用户再次明确要求开始实施现有父子任务，已进入实施，先前会话的 planning-only 限制不再适用。

## 已核实事实

- 当前主体已替换为 Grok Auth：crates/ccr/src/main.rs:38、crates/ccr-tui/src/tui/app.rs:586；本次清理剩余配置兼容、配色和 OpenCodePaths，见 research/codebase.md。
- 现有 Grok 服务只判断 auth.json 存在（crates/ccr-cli/src/services/grok_auth_service.rs:25），没有账号保存库。现有页面的初始化/刷新错误与直接登出问题见同任务研究。
- 官方 auth.json 是 scope -> credential map；个人和团队账号可能共用同一 scope。官方会刷新 token 和修改身份附加信息，不能用邮箱或 CCR current 指针无条件回存凭据。固定官方源码和边界见 research/grok-multi-account.md。
- 当前源码与用户截图不一致；PATH 二进制构建来源、真实热切换与服务端 token 有效性未验证。

## 需求

- R1：删除 OpenCode Auth 专属残留和测试，保留有效 OpenCode 配置/session/usage、Codex quota core、语言/主题/Usage 独立回归及历史归档。
- R2：主 TUI 保持六个有效页签与默认顺序：Codex Profile、Claude Code、Grok Profile、Codex Auth、Claude Auth、Grok Auth；合法自定义排序、快捷入口和其他页行为不退化。
- R3：Grok Auth 提供官方 OAuth 个人/团队账号的保存、列表和同名显式覆盖。保存像 Codex 一样只读复制当前凭据到 <CCR_ROOT>/platforms/grok/auth/accounts.json（默认 ~/.ccr 下）；允许 Grok 运行，当前账号继续使用，不要求退出、不登录/登出/refresh、不改原生文件或官方锁。保存完整的单个 scope 凭据对象，保留未知字段；账号别名不等同 scope。不管理企业 OIDC、external provider、API key 或旧 web_login，这些非目标条目原样保留。
- R4：切换所选保存账号，只替换该账号对应的官方 scope；当前凭据能唯一对应保存项（内容相同，或身份明确）时，先保留其最新凭据，再写目标。未保存、内容已变且身份不明/歧义、文件损坏或回存失败时不得覆盖当前会话。禁止伪造 create_time、主动调用 OAuth refresh 或自动登录。
- R5：删除只移除 CCR 保存项，不登出运行时。登出是独立确认动作，继续走共享 auth_off 写核，保留所有 CCR 保存账号；可识别的已保存当前凭据在删除前回存。登出仍清除整个 runtime auth.json，UI 必须说明其影响，不将其假装成“删除选中账号”。
- R6：界面区分“选中账号”“本地会话匹配账号”和“实际认证未验证”，提供保存/覆盖/切换/删除/登出反馈，中英文、深浅主题及窄终端可读。刷新仅重读；不展示无证据的订阅、配额或账号用量。
- R7：凭据只在服务/持久化边界使用；TUI/日志/错误不输出 token 或原始 JSON。账号库写入使用受保护原子写、CCR 操作级锁和版本冲突处理；切换/登出写运行时才使用官方锁，保存不占用官方锁。损坏数据不默认为空库。失败不得丢失未保存会话或覆盖其他 scope。
- R8：Auth 账号切换不自动 profile off，不更改 Grok custom model、profile 指针、MCP 凭据、第三方配置。页面说明“会话写入”与当前认证路线生效不同；只读复用当前 activation 状态。
- R9：更新当前规范、过时宣传和相关测试，按子任务及父任务整体验证；源码/fixture 通过与真实 Grok 会话/安装证据分开记录。

## 验收标准

- [ ] AC1（R1、R2）：有效源码无 OpencodeAuth/OpenCodePaths/专属 theme helper；六页顺序、合法自定义排序和 Grok 快捷入口通过，其他 Auth 导航保持。
- [ ] AC2（R3）：临时 CCR Grok 目录内保存 A、保存 B、列表显示两项；同名默认拒绝，确认覆盖只改目标槽；完整 scope 对象和未知字段往返。保存不检查 Grok 退出、不触碰 runtime/官方锁/config/profile/MCP、不登录/登出/refresh；官方锁被持有和快照后原生自行刷新时仍满足只读保存契约。
- [ ] AC3（R4、R7）：A 在 runtime 更新 token 后切到 B，再切回 A，恢复的是保存前捕获的最新 A；外部换号不得回存到错误槽；未知/歧义身份、损坏/缺失目标、冲突、回存失败均零 runtime 写入。
- [ ] AC4（R4、R8）：切换只替换目标 scope，其他 scope/API-key 字段、profile/MCP 文件不变；create_time/expires_at 原样保留。用户确认的使用前提为先结束当前 Grok，切换后供新会话使用；成功反馈仅说明本地凭据已写入，不承诺既有进程热切换或服务端认证有效。
- [ ] AC5（R5）：删除当前保存槽后 runtime 字节不变；登出保留账号库且已识别凭据先回存；登出取消零写/零 spawn，失败沿用备份回滚；未知/损坏 runtime 的明确登出保持原清除能力。
- [ ] AC6（R6）：140×40、100×30、80×24 中英文 TestBackend 覆盖列表/详情/弹窗/空态/错误，40×12 至少有选中或空态及必要操作；Mocha/Latte 可读。选择不是当前标记，刷新和排序后按别名保持选择。
- [ ] AC7（R6、R7）：长时间锁等待期间页面可响应，禁止重复 mutation；确认默认取消，换页关闭未提交弹窗，提交后不丢结果或假称取消；刷新失败保留旧内容并标旧状态，操作成功/后续刷新失败分别反馈。
- [ ] AC8（R7）：新建 secret 文件和临时文件具私有权限，既有严格 mode/DACL 被保留；锁竞争不清空官方锁 holder 信息；同一 auth 操作串行，外部版本冲突不盲写；secret sentinel 不出现在 DTO/渲染/日志/错误。
- [ ] AC9（R1、R9）：含旧 opencode_auth 的配置按既有未知枚举路径失败并整体回退默认；初次加载不覆写磁盘。合法配置保持排序/语言/主题；不加迁移器。
- [ ] AC10（R1–R9）：父级完成需求—子任务—源码对应的静态审查。按用户要求，本轮不运行测试、just ci 或运行时验证；对应行为验收保留为未验证，测试标 SKIPPED（用户要求），不得宣称通过。实现交付与行为验证完成分开记录。

## 任务树与顺序

| 子任务 | 独立交付 | 覆盖父需求 | 依赖 |
| --- | --- | --- | --- |
| 09-08-opencode-auth-residual-cleanup | 旧代码/测试/规范清理 | R1、R2、R9 | 无 |
| 09-08-grok-auth-account-service | 账号存储、scope 写回、身份与并发保护 | R3、R4、R5、R7、R8 | 无；可与清理独立研究 |
| 09-08-grok-auth-multi-account-tui | 多账号界面及主壳接入 | R2、R3、R4、R5、R6、R8、R9 | 服务契约冻结；合入清理后的 theme |

父任务拥有总需求、最终设计决策、跨子任务静态审查和验证证据边界；just ci 留待后续授权验证，本轮不执行。父任务不是默认产品实现目标。

## 范围外

企业/外部认证流与 API-key 账号管理、旧 web_login 恢复、token 编辑/人工延期、自动后台刷新全部保存账号、网络配额/用量、导入导出/重命名/跨机同步、ccr-ui/VS Code 新页、新 CLI save/switch 子命令、新 crate/依赖/数据库、自动杀进程或重启 Grok。

旧配置自动迁移、全局安装、真实凭据操作和发布不在本轮授权内。

## 关键取舍与风险

单文件账号库替代“账号文件 + 注册表”双持久化，当前匹配从 runtime 派生，不保存第二份 current 指针。身份不确定时要求明确保存，优先保留用户当前凭据。

移除 OpencodeAuth 后旧 tui.toml 会触发整份默认回退，影响内存排序/语言/主题，后续用户保存可能持久化默认值；代码为 crates/ccr-config/src/managers/tui_config.rs:200、242。独立 Usage 兼容保留。

现有 FileLock 截断官方锁文件、新建 Windows secret 文件权限是本次新存储/共享锁必须补齐的具体缺口，不推广成全仓权限重构。外部不守锁 writer 和服务端 token 撤销无法用本地 CAS 彻底解决。

## 已定决策与状态

切换采用用户已确认的“先结束当前 Grok，切换后供新启动会话使用”；不实现运行中立即换号，不自动结束或重启进程。保存独立采用只读复制到 CCR Grok 目录，当前账号继续使用。

产品待决事项已清空，最终规划审阅与实施授权均已完成。四个任务已在本会话按已批准方案启动。本轮交付约束为只实施源码及必要文档/测试源码维护，不运行测试、构建或 runtime，不触碰当前 Grok Build。未执行的行为验收不得勾选完成；实施交付与动态验证状态分别记录。
