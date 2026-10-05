# T01 独立检查报告

日期：2026-09-28。角色：独立 `trellis-check`。工作区：`D:/Documents/Code/Github/ccr`，Windows / PowerShell。基线 commit：`34d8a85e0e48b793733835e0304c8ed33940fcee`。检查针对共享工作区未提交的 T01 实现与本报告列出的修复。

## 结论

T01 的 AC1、AC2、AC3 在下述本机验证范围内通过。独立检查发现并修复了资源身份、备份策略及 current 意图投影问题；7 个行为反例均先运行失败，再运行通过。另增加 1 个 Windows 双进程回归。没有剩余已确认但未修复的 T01 源码问题。

三包 Clippy、类型检查、完整 `just fmt-check` 与 T01 范围 `git diff --check` 通过。CLI 两个过滤器在最终编译后发生 `0xc0000005`，同一二进制和原 Cargo 命令复跑均通过，原因未查明。全仓 `git diff --check` 的最后一次检查失败于 T06 生成文件尾空格。上述结果不能作为父任务全仓验收或归档依据。

本检查未更改 task.json、PRD 验收勾选或任务生命周期；未提交、归档或修改真实用户配置。T05/T08、用户的两个临时脚本、图标和其他任务文件均保持原样。

## Findings (fixed)

### F1：Windows 普通与 verbatim 路径生成不同资源锁名

- File: `crates/ccr-config/src/managers/config/repository.rs:83`。
- Issue: `canonicalize()` 返回的 verbatim drive 路径与普通 drive 路径指向同一文件，但原路径散列保留不同 prefix。资源锁会分裂。
- Fix: 在同一词法规范化函数中合并普通/verbatim drive 和 UNC prefix；不改变非 Windows 路径规则。继续使用稳定 FNV-1a 和 Windows 大小写折叠。
- Evidence: `resource_identity_matches_windows_verbatim_paths` 在修复前断言两个不同 lock name；修复后 drive/UNC 身份断言通过。新增 `windows_verbatim_and_service_processes_keep_both_updates`，执行 3 轮独立双进程写入。

### F2：Windows 大小写路径别名跳过标准备份

- File: `crates/ccr-config/src/managers/config/repository.rs:115`。
- Issue: `PROFILES.TOML` 与 `PLATFORMS` 的大小写别名未匹配标准路径规则，导致同一资源使用 `BackupPolicy::None` 或不同备份位置。
- Fix: Windows 下按大小写不敏感方式识别标准文件名和目录名；其他平台保留精确匹配。保留原平台备份目录、`profiles` prefix 和 secret writer。
- Evidence: `uppercase_profile_path_keeps_standard_backup_policy` 修复前未创建备份目录；修复后在标准备份目录产生 1 份前镜像。

### F3：已有空 inactive 文档被 add_config 激活

- File: `crates/ccr-config/src/services/config_service.rs:197`。
- Issue: 当 sections 为空时，add_config 无条件设置 current，即使已有文档明确存储 `current_config = ""`。
- Fix: 仍更新 default，但保留已有空 current。显式首次创建仍保留原初始化默认值与旧服务成功路径。
- Evidence: `adding_to_empty_inactive_document_preserves_current_marker` 修复前 current 非空；修复后 default 指向新 profile，current 保持为空。

### F4：兼容 parser 的合成 current 进入查询和普通编辑

- Files: `crates/ccr-config/src/managers/config/repository.rs:61`、`:240`，`crates/ccr-config/src/managers/config_file_handler.rs:62`，`crates/ccr-config/src/platforms/base.rs:491`。
- Issue: 简化 profile map 没有声明 current，但低层兼容 parser 合成首 profile。服务查询将首项标为当前；普通 patch/update 无条件写入合成 marker。Grok 因同一投影问题在编辑时产生激活意图，并将 inactive profile 删除误判为 active。
- Fix: repository 从同一源字节投影声明 current，供 load、snapshot、mutation 和 marker helper 共用。已有简化 map 暴露空 current；list/get-by-name 的 is_current 为 false；get_current 走现有无当前错误。普通编辑保留 marker 缺失，显式 activation 才写 current/default。低层 parser 的格式兼容默认保持不变；没有新增公共错误变体或改变返回签名。
- Grok 使用资源锁内的 owner 投影；没有新增第二次 profile 文件读取来判断 marker。
- Evidence: 以下 4 项在修复前失败，最终通过：`service_queries_do_not_infer_current_from_simplified_profile_order`、`unrelated_service_edits_preserve_absent_current_until_explicit_activation`、`profile_save_preserves_missing_current_marker_in_simplified_file`、`profile_delete_accepts_inactive_simplified_file_without_current_marker`。查询回归比较原始文件清单、字节和修改时间；编辑回归断言普通修改后无 marker、显式激活仍成功。

### F5：Grok 规范仍描述查询清除 registry

- Files: `.trellis/spec/ccr-cli/backend/grok-profile-runtime.md`、`.trellis/spec/ccr-config/backend/profile-repository.md`。
- Issue: Grok 规范的删除示例仍把 current query 描述为会清除 registry，与 T01 的纯读实现不一致。
- Fix: 明确 drift query 返回无当前 profile 并保留存储意图；删除须检查 raw intent 和 runtime。补充统一 current 投影、简化 map 编辑、Windows prefix/大小写备份规则及对应回归。

## Findings (not fixed)

### N1：CLI 原生测试进程访问冲突，原因未查明

- 最终 owner 修复后的 `cargo test -p ccr-cli platforms::` 和 `cargo test -p ccr-cli profile` 两次返回 Cargo exit 5；测试二进制 `target/debug/deps/ccr_cli-442c371c21e1c1ca.exe` 返回 `0xc0000005 STATUS_ACCESS_VIOLATION`。
- 原始日志为 `check-cli-platforms-final.log`、`check-cli-profile-final.log`。不根据缓冲输出判断崩溃发生于哪个阶段。
- 同一二进制 `--list` 成功列出 334 tests；`platforms:: --nocapture` 为 63 passed，`profile --nocapture` 为 62 passed。复跑前后 SHA-256 相同，见 `check-cli-native-retry.json`。随后原 Cargo 命令也分别通过，见 `check-cli-cargo-retry.json`。
- 未禁用测试并行、增加无条件生产重试、清理 target、替换 target-dir 或修改无关锁实现。复跑成功不等于原因已修复。父任务需在最终 Windows gate 中继续保留该不稳定记录。
- 实施阶段另有 Codex `0xc0000005` 原始记录，见 implementation-report.md / cargo-codex-profile.log；本次最终 Codex profile 过滤器通过。父任务报告的 core Grok auth lock `os error 33` 未在本检查复跑，未修改该路径。

### N2：全仓 diff 检查存在其他任务生成文件尾空格

- 最后一次完整 `git diff --check` 返回 2，指出 `ccr-ui/src/types/generated/usage/UsageImportResultV2.ts:3` 尾空格。日志：`check-diff-final.log`。
- 文件属于 T06；本检查未修改。T01 范围 diff 检查通过，不能替代完整命令。
- 早期 `just fmt-check` 曾失败于 `ccr-ui/src-tauri/src/usage_jobs.rs:22,33`。该文件由其他 owner 处理；最后一次完整 fmt-check 已通过。早期日志 `check-fmt.log` 保留。

## AC 逐项证据

| 验收 | 检查与证据 | 结论与限制 |
| --- | --- | --- |
| AC1 / R1 | `mutate_versioned` 在 `repository.rs:203` 获取资源锁后读取；closure 执行一次；之后进入 guarded leaf CAS。平台 helper/service、显式路径 desktop-service/service、Windows canonical-path/service 各 3 轮，每轮 2 个独立子进程，最终两项独立写入均保留。 | 本机仓储边界通过，共 18 个 worker 进程。desktop fixture 使用 ConfigService，不是实际 Tauri handler；真实 IPC 由 T03 验证。没有声称跨文件原子性。 |
| AC2 / R2 | 显式平台路径不读 registry 顺序；legacy with_default 固定 Claude。缺文件、损坏、不完整、只读文件及无 marker fixture 验证 list/current/validate/load 不改路径、字节和 mtime；无 query 资源锁文件。读取、snapshot、marker、mutation 共用声明 current 投影。 | 通过。`ccr` managers 中现有 legacy routing 测试也实际运行 1 项通过。Windows 只读文件属性已执行；未执行目录 DACL 禁写或 Unix 模式验证。 |
| AC3 / R3 | 非法类型、重名、缺失源、保留名称、stale token、validator 失败均保持目标和备份；合法 patch 保留未改字段、secret、未知 settings 和 TOML datetime。rename 同步 current/default。leaf-only replacement 测试验证 CAS 冲突不会覆盖外部字节或新增备份。 | 通过。Windows 标准备份大小写路径回归通过；未知字段和 current/default 均经磁盘重新读取核对。平台 auth validator 在锁内调用；跨文件 secret 补偿由 T02 负责。 |

`subprocess_adapter_worker` 的 ignored 标记仅用于避免直接无参数执行；上述父测试通过 `--ignored --exact` 实际启动 18 个 worker。文档测试另有 1 项 ignored。CLI 两个过滤器存在重叠，不合计为独立用例总数。

## Verification

| 实际命令 | 结果 | 日志 / 记录 |
| --- | --- | --- |
| `cargo test -p ccr-config` | 退出 0；101 passed，1 ignored；doc test 1 ignored | check-config-final.log |
| `cargo test -p ccr-cli platforms::` | 原生失败后原命令复跑退出 0；63 passed | check-cli-platforms-final.log；check-cli-platforms-retry.log |
| `cargo test -p ccr-cli profile` | 原生失败后原命令复跑退出 0；62 passed | check-cli-profile-final.log；check-cli-profile-retry.log |
| `cargo test -p ccr-codex profile -- --nocapture` | 退出 0；18 passed | check-codex-profile-final.log |
| `cargo test -p ccr --test managers config_manager_default_ignores_legacy_current_platform_routing` | 退出 0；1 passed | check-legacy-adapter-final.log |
| `cargo clippy -p ccr-config -p ccr-cli -p ccr-codex --all-targets --all-features -- -D warnings` | 退出 0 | check-clippy-final.log |
| `cargo check -p ccr-config -p ccr-cli -p ccr-codex --all-targets --all-features` | 退出 0 | check-typecheck-final.log |
| `just version-check` | 退出 0 | check-version.log |
| `just fmt-check` | 最终退出 0；早期无关文件失败保留 | check-fmt-final.log；check-fmt.log |
| `git diff --check` | 最后一次退出 2；其他任务生成文件尾空格 | check-diff-final.log |
| `git diff --check -- crates/ccr-config crates/ccr-cli/src/platforms crates/ccr-codex/src/platforms/codex.rs .trellis/spec/ccr-config .trellis/spec/ccr-cli/backend/grok-profile-runtime.md` | 退出 0 | check-diff-scope.log |

修复前反例日志：`check-regression-before-config.log`、`check-regression-before-grok.log`、`check-regression-before-backup.log`、`check-regression-before-marker-owner.log`。最终命令状态以 `check-final-commands.json`、`check-cli-cargo-retry.json`、`check-final-format.json` 及本报告的失败/复跑说明共同解释；不得丢弃第一次失败。

## 未验证边界与交接

- 未执行完整 Rust workspace tests、`just ci`、完整 Tauri workspace gate、原生 UI、真实账户、真实网络或 Unix 权限测试。
- Windows UNC 仅验证普通/verbatim 字符串资源身份；没有访问真实网络 share。路径身份仍为词法规则，多 symlink alias 不在当前契约内。
- 跨文件 runtime/secret/registry/history 一致性与补偿仍由 T02 负责。T01 不将单文件 CAS 描述为外部进程不可覆盖保证。
- T03 需通过真实 handler/IPC 验证接入；T04 需执行诊断/退出码验收。本报告不把仓储 fixture 等同于三端集成。
- T10/root 负责全仓尾空格、Windows 原生测试不稳定及最终全仓验收。本检查未修改其他任务代码。
