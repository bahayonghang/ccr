# T02 实施报告

日期：2026-09-28。基线 commit：34d8a85e0e48b793733835e0304c8ed33940fcee。环境：Windows / PowerShell；Linux 补验由 root 在 WSL 执行。用户已批准父任务与 T02。本代理不操作任务状态、提交、推送或归档。

## 结果与当前检查点

共享 apply/update 生命周期、CLI/TUI/Tauri 适配、限定文件范围的 CAS 补偿，以及前端提交结果消费已实现。T01 仓储与 T05 备份策略保留。三个平台共用合成 fixture 和 fault harness。最终独立验收由 checker 执行，不能依据本报告勾选全部 AC。

最终 CLI 9/9、TUI 8/8、desktop 10/10 已通过。desktop 包含 T02 合同 9 项及现有 ACL 测试 1 项。strict lint 修复一处 T02 collapsible_if 后已通过。会话 20300 与 27065 均结束。T02 application/journal 与三端生产源码写入停止，现释放给独立 checker。T11 当前拥有第二轮 inventory/bindings 生成窗口；生成后的 frontend type-check 作为补充记录，不阻塞 checker 开始验收。

## 责任与数据流

- CLI：async switch → 一个 spawn_blocking → switch_profile_sync → apply_profile。终端 command 只呈现；提交后的详情读取失败只警告，不返回完全未生效。
- TUI：apply_selected → profile_backend::apply → apply_profile。删除了独立 off 和本地忽略错误的计数保存。toast 按结构化结果区分成功、警告、恢复，并使用英中本地化。
- Tauri：一个 spawn_blocking → apply_profile_payload → apply_profile。Claude/Codex update 的同步 helper → update_profile。新 mutation response 只返回标识与 outcome，不返回原始凭据，也不在提交后再次读取 profile 造成伪失败。
- 应用层：校验目标 → 平台 operation lock → 绑定 repository read token → interrupted record → journal → runtime/指针写入 → verify → committed/pending record → journal commit → count/history/auth registry/provider analytics → 最终 record。
- 重放：相同 operation ID 直接返回持久结果，不重复激活或部分完成的附属动作；已提交后源 profile 被删除仍可重放。中断记录保持 recovery_required，无后台自动重试。
- rename：保留精确 TOML section 作为目标初值，应用 JSON projection delta，更新 current/default，删除旧名，必要时激活。未修改的 TOML datetime、扩展字段和平台秘密存储均由仓储/平台 owner 保留。

## 写入与恢复边界

journal 只覆盖显式列出的 profiles、registry、runtime、平台 auth/entry/secret 文件和 operation record。前镜像只在内存中保存，无 Debug/Serialize。journal 不跨线程、不允许嵌套、不允许 guarded async dispatch。CLI/Tauri 将完整同步调用放入同一 worker；同步生命周期内没有 await/spawn。

guarded writer 在叶锁内取得字节和权限前镜像。AtomicWriter 在物理替换成功后、可能失败的目录 sync 前调用内部 publication callback，保证已发布但报错的写入仍登记。回滚逆序执行 CAS；遇到外部版本保留该版本并返回 recovery_paths。原 Unix mode/Windows DACL 在恢复临时文件写入秘密字节前设置，Windows read-only 在 publish 后恢复；失败保持 recovery_required。

应用锁串行化合作调用者。journal 不是 OS 多文件事务。备份和轮换沿用 T05，不在补偿范围内；已产生备份可在失败后保留。掉电/进程强制结束只能依靠中断记录提示恢复，不宣称自动恢复成功。

## AC 证据

| AC | 实现与反例 | 当前证据边界 |
|---|---|---|
| AC1 | 无效/禁用/不存在目标预检；prepare 后删除；每一 declared write 故障；post-publish 故障；外部新版本保留 | core Windows 11/11、Linux 9/9；CLI/TUI/Tauri adapter 共同 harness 最终均通过 |
| AC2 | 三 adapter 共用 fixture；提交后才 count/history；同 ID 不重复；源删除后的 replay | 最终 CLI 9/9、TUI 8/8、desktop 10/10；各命令细项见下表 |
| AC3 | activation_committed、warning code、safe DTO/Debug；捕获 tracing sentinel；真实 React hooks 警告、恢复、refetch 与不重复 activation；TUI 双语 | frontend 专项 8/8、Profile/API 171/171；TUI 英中 presentation 用例通过。CLI 终端呈现已 source review，未新增 stdout capture |
| AC4 | 同步 application owner；CLI/TUI/desktop 显式适配；无终端输入/输出/exit | source guard 限于 production，最终 CLI 合同重跑通过 |
| AC5 | 实际 Claude/Codex desktop update helper 使用同一 rename harness；逐写失败、current/default、秘密存储、未知 TOML datetime、保留名、inactive edit warning | CLI rename/inactive-edit 合同与实际 Claude/Codex desktop adapter matrix 最终通过 |

## 验证记录

零匹配的 integration target 不计入通过测试数。前一版通过与最终版通过分开记录。

| 命令或范围 | 结果 | 证据 |
|---|---|---|
| cargo test -p ccr-core --features test-support journal_restores_ -- --test-threads=1 | 红例：1 passed / 2 failed；证实 post-publish 漏登记与删除后 DACL 丢失 | journal-red.log |
| cargo test -p ccr-core --features test-support write_journal -- --test-threads=1 | 11 passed；Windows DACL、read-only、别名、首次 token、线程/异常/重复写 | journal-green-final.log |
| root Linux cargo test --offline --locked -p ccr-core --features test-support write_journal -- --test-threads=1 | 9 passed；Unix 0400 与 post-publish 故障 | root-linux-write-journal-faults.json |
| root Linux guarded_write -- --test-threads=1 | 24 passed；本次 atomic 修改后的 T05 不回归 | root-linux-guarded-write.json |
| cargo test --offline --locked -p ccr-cli profile_contract -- --test-threads=1 | 最终 9 passed；包含 rename、inactive edit 与 production boundary guard | cli-contract-completed.log |
| cargo test --offline --locked -p ccr-tui profile_backend -- --test-threads=1 | 最终 8 passed；7 个共同合同与 1 个英中呈现测试 | tui-contract-completed.log |
| CCR_SKIP_ICON_GENERATION=1 cargo --config .cargo/tauri-ci.toml test --offline --locked --manifest-path ccr-ui/src-tauri/Cargo.toml application_ -- --test-threads=1 | 最终 10 passed；7 个共同合同、2 个实际 rename adapter、1 个现有 ACL 测试 | desktop-contract-completed.log |
| checker CLI platforms / Codex profile | 独立 checkpoint：CLI 67 passed；Codex 18 passed | check-platform-validators.json / check-codex-profile-validators.json |
| cargo test -p ccr-cli --features ts profile_lifecycle::export_bindings | 3 passed；ProfileWarning 含 analytics_failed | profile-bindings-final.log |
| bun run test:smoke -- tests/profiles/profile-outcome.smoke.test.tsx | 8 passed | frontend-outcome.log |
| bun run test:smoke -- tests/profiles tests/api | 33 files / 171 passed | frontend-profiles-api.log |
| bun run type-check | 当前 exit 0；先修复新 test 的 hook union 推断错误。T11 第二轮生成后的最终补验待完成 | frontend-types-verified.log |
| bun run test:i18n | 24 passed；key detector 4506 leaves | frontend-i18n-final.log |
| bun run lint:ci | 失败：仅用户保护的两个 .tmp 文件 5 条 no-console | frontend-lint-final.log |
| python scripts/quality/check_secret_writes.py | 通过 | secret-write-guard.log |
| scoped rustfmt / git diff --check | 已格式化 T02 源码。生成前 diff check exit 0；T11 生成中发现临时 generated/usage 尾空格，待窗口结束核对 | 终端记录 |
| CARGO_NET_OFFLINE=true just lint-strict | 最终 exit 0；workspace/all-targets/all-features，-D warnings -D clippy::unwrap_used，含 secret write guard | lint-strict-verified.log |

测试过程修正：新增 inactive-edit 断言原先假设 usage_count=Some(0)，实际初值为 None；改为与编辑前值逐项相等。边界 guard 原先扫描整个 adapter 文件，匹配旧测试 fixture 的直接 low-level apply；改为 production 范围。两项是 fixture/guard 修正，没有修改业务期望。最终 strict lint 检出 T02 provider analytics 的嵌套 if；按相同短路规则合并后同一 gate 通过（lint-strict-completed.log → lint-strict-verified.log）。补丁曾在另一 ancillary fixture 误插入未声明的 instance，导致 E0425；已定点移除。该错误属于 T02 测试，不能归因于 T04。网络阶段一次 --locked 命令因 crates.io config.json 的 schannel SSL 握手失败退出 101，未执行测试；随后采用 --offline --locked。T04 自己撤回其未下载的新增 dev-dependency，T02 不改 Cargo.lock。

## 文件变更

- ccr-core：Cargo.toml 的 test-support feature；core/mod.rs；新增 core/write_journal.rs；guarded_write.rs 的 scoped hook/delete/restore；atomic_writer.rs 的内部 publication callback 与 metadata 恢复。
- ccr-cli：Cargo.toml 与 lib.rs 的 test-support 可见性；application/mod.rs、profile_lifecycle.rs、profile_contract.rs、profile_switch.rs、types.rs、profile_off.rs；commands/profile/switch.rs；platforms/gemini.rs、droid.rs 的 guarded runtime 写入。
- ccr-config：managers/provider_activation.rs，将应用期 analytics 延迟到提交后并返回可见失败。
- ccr-codex：managers/codex_config.rs；services/codex_runtime_service.rs；platforms/codex.rs，接入 runtime/secret/legacy restore 的 journal 边界与提交后 registry。
- ccr-tui：Cargo.toml；tui/profile_backend.rs、app.rs、mod.rs。
- Tauri：src-tauri/Cargo.toml；commands/profile_lifecycle.rs、claude_profiles.rs、codex_profiles.rs、grok.rs 的 apply/outcome。handler registry、批量 inventory 与 OAuth 文件仍归 T11。
- React：api/domains/claude.ts、codex.ts；configs/profileEditorAdapter.ts、profiles.ts；features/platform/profiles/useProfileEditor.ts；Claude/Codex editor adapters 与 page hooks；Grok page hook；新增 utils/profileOutcome.ts 与 tests/profiles/profile-outcome.smoke.test.tsx；两语言 profilesSurface 三个 key 与 i18n leaf expectation。
- DTO：generated/profiles 的三个类型；GrokProfileActionResponse 的定向生成待 T11 窗口完成。
- 规范：新增 .trellis/spec/ccr-cli/backend/profile-application-lifecycle.md 与 index 引用；atomic-writer.md 追加 scoped compensation 约束。

## T03 / T04 交接契约

- T03：适配层只调用 apply_profile(ApplyProfileRequest) 或 update_profile(platform, old, target, patch)。patch 仅修改内存 ProfileConfig。保持 command ID 与输入确认策略；不要在 adapter 再组合 off/save/delete/apply，不要在激活提交后用详情读取错误覆盖 outcome。
- T04：继续使用各平台 validate_profile/auth-mode 作为只读验证权威。应用状态与诊断发现分开；applied_with_warning 不能映射为完全未激活。不要改变或新增 CcrError 变体。恢复提示不得包含 Secret 内容。validator 独立 checkpoint 不等于 T02 整体验收通过。
- 三方均应读取新 profile-application-lifecycle spec。只有 root/checker 可以解除最终依赖 gate。

## 限制与后续

- 尚无 native WebView、交互 TUI、浏览器截图或真实用户配置测试。React tests 覆盖真实 hooks 和消费路径，IPC 使用合成 mock。
- 没有执行完整 just ci。正式 frontend lint 保持失败，不排除用户临时脚本替代正式 gate。
- apply 合同集中于 Claude/Codex/Grok；Gemini/Droid 的 guarded writer 接线保留，其他平台的穷举并发/故障矩阵不在本轮证据内。Qwen 仍使用原未实现规则。
- 跨文件外部写入与进程中断不能提供全局原子性；明确恢复结果是契约的一部分。备份轮换不回滚。
- 此报告不授权归档或提交。Cargo.lock SHA256 当前仍为 CA1F03E85B5E0D888109AC8AF7E7943FC1CEECD20A0F6EEB6A60052C5155DB79。
