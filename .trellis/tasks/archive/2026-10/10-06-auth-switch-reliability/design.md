# 设计：切换可靠性修复

## 边界与所有权

- 领域服务：crates/ccr-codex（codex_auth_service、codex_oauth_token_service、codex_quota_service、codex_runtime_service、openai_quota_core）。
- TUI 接入：crates/ccr-tui/src/tui/codex_auth（reload/refresh/repair 调用点）。
- CLI 接入：crates/ccr-cli（apply/profile_lifecycle）仅在必要时最小改动。
- 不新增 crate、不新增依赖、不新增错误变体。

## D1 统一同步入口（观测点回写）

把「runtime auth.json ↔ 匹配的已保存账号快照」的新鲜度定向同步定义为幂等操作（主体逻辑已存在于 sync_runtime_tokens_to_saved_account，codex_oauth_token_service.rs:333；本任务加入新鲜度方向与 current_auth 回写）。

同步算法（新鲜度定向）：
1. 读 runtime auth.json；不存在或非 OpenAI 登录态 → NoOp；
2. 解析 account_id（tokens.account_id → JWT 回退）；缺失 → NoOp + debug 日志；
3. 按 account_id 匹配注册表；0 匹配 → NoOp；多匹配 → 按 D4 规则；
4. 比较内容（tokens + last_refresh）与新鲜度（effective ts = last_refresh，缺省文件 mtime，与 resolve_latest_oauth_doc 一致）：
   - 相同 → NoOp 不写文件；
   - runtime 较新 → 原子写快照 + 更新注册表 last_refresh（沿用 sync_account_auth_file :282；既有方向加新鲜度门槛）；
   - 快照较新且账号为 registry.current_auth → 经 CodexRuntimeService::commit_plan（CodexAuthCacheAction::Write(快照原始 auth map)，config 不写；备份 + 原子 + 失败回滚）写回 runtime，恢复「CCR 配额刷新消费快照 token、runtime 仍持被消费值」的分裂；非 file 凭据存储时 commit_plan 拒绝 Write → warn 跳过；
   - 快照较新且非 current_auth → 跳过 + debug 日志。

调用点（显式调用，无守护/轮询；save_current 本身复制 runtime，无需新增）：

| 调用点 | 位置 | 目的 |
| --- | --- | --- |
| TUI 加载 / r 键刷新 | app.rs reload_accounts | 观测点最大化 |
| 切换前（换出账号） | switch_account 开头 | 消除换出丢失（D2） |
| 切换后（写后校验） | switch_account 末尾 | 读回比较，不一致 warn |
| 活动账号配额路由 | codex_quota_service fetch_account_quota_inner | account_name == current_auth 时以 runtime 为源；刷新落盘 runtime 后经 D1 同步快照（见下） |
| apply/清理前 | platforms/codex.rs:1289（既有） | 保持 |

失败语义：同步失败记录 warn 日志，不阻断主操作。

配额路由（TPR-01 路由 3）：fetch_account_quota_inner 当 account_name == registry.current_auth 时，读 runtime auth.json 为凭据源（替代快照源）；persist_tokens 写回 runtime（update_auth_file，现有机制）后调用 D1 同步把新 tokens 落快照（双落盘）。未保存登录的虚拟 default 路径（fetch_current_quota_inner，app.rs:35）不变；批量/预览路径（fetch_quotas_for_accounts_inner）按名分派，自动继承该路由。

## D2 切换序列重排（switch_account :1106）

新顺序：
1. 既有 ensure 校验不变；
2. 同步换出账号（D1 新鲜度定向；runtime 属于已保存账号时）；
3. 读目标快照 → normalize → commit_plan（保留 runtime_switch 备份与失败回滚，codex_runtime_service.rs:230-300）；
4. 注册表更新 + sync_current_auth_registry 不变；
5. 写后校验：读回 runtime 与目标快照 tokens 比较，不一致 warn（不报错）。

不变项：forced_login_method、apply_account_route_config、usage_ledger、错误类型与文案。

## D3 修复触发与恢复

- should_repair_tokens（openai_quota_core.rs:351）扩展：加入 `refresh_token_invalidated`；保持与 should_force_refresh 的分工（access token 失效先 force refresh 重试，refresh 失败才 repair）。
- 修复流程不变：repair_saved_account → 重试一次（codex_quota_service.rs:249）；仍失败时 quota error 文案标注「需重新登录」结论（现有 CcrError 变体 + 状态字符串，不新增变体）。
- 恢复来源选择（修正表述并补测试）：所有候选（runtime 与 auth.*.bak）按 effective ts（last_refresh，缺省 mtime）取最新，新鲜度决定胜者；runtime_switch 标签仅影响扫描顺序（120 条上限内优先扫描，codex_oauth_token_service.rs:256-262），不改变新旧判定；备份保留为 auth 前缀共享池 MAX_BACKUPS=10（跨标签共享，codex_config.rs:41、:271-299）写入边界文档。

## D4 身份匹配规则

- 匹配键 account_id；缺失时 JWT 回退提取，仍要求与注册表精确相等；无匹配 → NoOp。
- 多账号同 account_id：视为同一登录身份；同步目标 = current_auth 指向者，其次 last_used 最新者；其余不写。行为由测试固定。
- 文档记录与 codex-auth（user_id::account_id）的差异与多 workspace 边界。

## D5 切换不可用分析（先行）

修复前执行复现矩阵，覆盖 research/current-auth-switching-state.md 第 4 节候选 1-6：
- 候选 1 进程占用：codex 运行中切换后的 runtime 是否被回写覆盖（文件 mtime/内容观察）；结论决定 TUI 是否需要二次确认文案（不得默认阻断切换）。
- 候选 2 凭据存储：非 file 模式报错路径走查与文案可执行性。
- 候选 3 normalize：对照本机 codex CLI 实际 auth.json 结构逐字段 diff。
- 候选 4 config/profile：runtime auth only + 无绑定 profile 实测。
- 候选 5：D1-D3 完成后重测。
- 候选 6：should_quit 行为确认；改变退出行为属行为变更，需用户确认后才做。
- 候选 7 CCR 自身轮换：活动账号的 TUI 配额刷新走快照路径消费 refresh_token（codex_quota_service.rs:172、:255-258），runtime 旧 token 随后被 codex 使用 → refresh_token_invalidated；D1 新鲜度定向 + 配额路由后重测。

产出：research/switch-failure-analysis.md（每条候选：方法、结果、结论）。

## D6 窗口边界文档

外部 login 覆盖 + CCR 未运行期间的理论丢失窗口；缓解 = 观测点最大化（D1）；codex 自身备份可用性未核实，标注。落点：research 或 3.3 spec 写回。

## 测试策略

- 合成 fixture（隔离 CCR_CODEX_DIR / CCR_DATA_DIR / CCR_LOCK_DIR，见 test-fixtures spec）：
  - 轮换模拟：改 runtime tokens → 同步 → 快照断言；二次同步不写（字节/mtime 断言）；
  - CCR 自身轮换：配额刷新（stub）消费快照 token 得新值 → 快照较新 → 观察点将新值写回 runtime（断言 runtime refresh_token = 新值）；旧值被 stub 拒绝后不再出现在任一文件；
  - 切换矩阵：save A → rotate A → switch B → A 快照新值；switch 回 A 后 quota（stub 网络）成功；
  - repair 谓词：refresh_token_invalidated 触发 repair；无来源 → 需重新登录状态；
  - 身份：缺失/重复 account_id 无跨写。
- 既有测试保持；新增测试遵循默认并行。

## 兼容与回滚

- registry/快照格式不变，无新字段。
- 各修复独立提交，可单独 revert。
