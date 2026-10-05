# T01 实施记录

## 最小变更边界

- 基线：`34d8a85e0e48b793733835e0304c8ed33940fcee`；原有两个前端临时文件保持不变。
- 用户已批准父任务与子任务规划；本代理只实施 T01，不操作任务状态、提交或归档。
- 行为缺口：平台 writer 在锁前读取整份 profiles，ConfigService 使用另一组锁；查询隐式 autofix 或修复 current 指针。
- owner：ccr-config 负责按实际配置路径加锁、锁内读取和保存、strict patch 和 CAS；平台 adapter 只提交锁内变更闭包并执行自己的 auth-mode 校验。
- 预计修改：config manager/file handler/service/base 及对应测试；Claude/Grok/Codex/Gemini/Droid profile CRUD 的必要委托和 current 纯读取；config 规范。
- 不修改：guarded writer 备份算法、OAuth/usage/Settings、真实用户配置、其他任务和原有文件。
- 锁顺序：platform operation lock → configuration resource lock → guarded leaf lock。查询不获取会创建文件的锁。

## 实施结果（2026-09-28）

T01 实现及本任务范围内的检查完成，等待父任务安排独立检查。未修改任务状态、提交或归档。测试只使用临时目录和合成凭据。

### 修改文件

| 文件 | 修改目的 |
| --- | --- |
| `crates/ccr-config/src/managers/config/repository.rs`（新增） | 路径资源锁、锁内 RMW、原始字节版本 token、guarded CAS、strict patch、未知字段 delta 保留 |
| `crates/ccr-config/src/managers/config/repository_tests.rs`（新增） | 多进程写入、纯查询、版本冲突、字段保留和显式标记回归 |
| `crates/ccr-config/src/managers/config/mod.rs`、`src/lib.rs` | 导出仓储接口和注册测试模块 |
| `crates/ccr-config/src/managers/config/manager.rs` | 纯构造器、显式平台路径、显式初始化和 autofix 事务 |
| `crates/ccr-config/src/managers/config_file_handler.rs` | 安全解析、缺文件与读取错误区分、统一 profile 写入选项 |
| `crates/ccr-config/src/services/config_service.rs` | CRUD 进入同一资源锁；查询去除初始化、autofix 和锁文件写入；提供 patch 接口 |
| `crates/ccr-config/src/platforms/base.rs` | 共用解析、ProfileConfig 事务适配、TOML 扩展类型保留、current resolver 与显式标记读取 |
| `crates/ccr-cli/src/platforms/{claude,grok,gemini,droid}.rs` | profile writer 委托事务；平台校验仍由 adapter 执行；查询不修复指针 |
| `crates/ccr-codex/src/platforms/codex.rs` | profile writer 委托事务；保留 secret 行为顺序；查询不清除 stale 指针 |
| `crates/ccr-cli/src/application/profile_off.rs` | 仅 `clear_profiles_file_pointer` 委托 `clear_current_if_present`；其余修改归 T05 |
| `.trellis/spec/ccr-config/backend/profile-repository.md`（新增） | 仓储接口、锁顺序、查询、patch、兼容约束和验证要求 |
| `.trellis/spec/ccr-config/backend/{index,backend-guidelines}.md` | 链接新规范并明确 legacy Claude 适配器 |

本任务未修改 `ccr-core` guarded writer 的算法；工作区中的相关修改由 T05 负责。前端、图标和 OAuth 文件的其他代理修改保持原样。

### 验收证据

| 验收 | 实现机制 | 已执行证据与边界 |
| --- | --- | --- |
| AC1 / R1 | `config_resource_name` 按绝对词法规范化路径生成跨进程稳定资源名；持锁后读取和执行变更；提交保留 guarded leaf CAS | `platform_and_service_processes_keep_both_updates`、`desktop_and_service_processes_keep_both_updates` 各运行 3 轮，共启动 12 个独立子进程；两个写入均保留。desktop fixture 使用显式路径的 ConfigService，并覆盖 `.` 路径别名；真实 Tauri handler 接入由 T03 验证。 |
| AC2 / R2 | `for_platform` 不读 registry 排序；构造、list/current/validate/load/export 不初始化、不 autofix、不创建资源锁；current resolver 返回 repair suggestion | 缺文件、字段不完整、损坏及只读 fixture 比较路径、字节、修改时间；两个 registry 顺序保持平台选择；Claude/Grok/Codex 查询回归通过。简化 profile map 不会被视为声明了 current marker。 |
| AC3 / R3 | patch 区分省略、设置、删除；锁内检查目标、重名、字段类型和调用者 validator；基于原始字节 token 检查过期版本 | 非法字段、缺失源、重名、保留名、过期 token、validator 失败均保持目标与备份不变；rename 同步 current/default；未知字段、未知 settings、secret sentinel 和 TOML datetime 保留。 |

`leaf_cas_detects_nonparticipating_replacement` 在仓储读取后直接替换目标，确认 CAS 冲突保留外部字节且不创建备份。

`clear_current_keeps_missing_absent_and_propagates_corrupt_or_unreadable` 同时覆盖显式 deactivation、`update_current_config` 和 current marker 读取。缺文件保持缺失；损坏文件和目录代替文件的读取失败返回错误。

Windows 上的只读属性及临时目录测试已执行。Unix 文件权限测试受 `cfg(unix)` 控制，本机未执行。

## 验证结果

最终记录：`implementation-checks-verified.json`。运行环境为当前 Windows / PowerShell 共享工作区，默认 Cargo test 并行设置；未禁用并发测试、未更换 target-dir、未清理构建缓存。

| 命令 | 结果 | 日志 |
| --- | --- | --- |
| `cargo test -p ccr-config` | 退出 0；95 passed，1 ignored；doc test 1 ignored | `config-final.log` |
| `cargo test -p ccr-cli platforms::` | 退出 0；61 passed，271 filtered；integration target 11 filtered | `cli-platforms-final.log` |
| `cargo test -p ccr-cli profile` | 退出 0；60 passed，272 filtered；integration target 11 filtered | `cli-profile-final.log` |
| `cargo test -p ccr-codex profile -- --nocapture` | 退出 0；18 passed，213 filtered | `codex-profile-final.log` |
| `cargo clippy -p ccr-config -p ccr-cli -p ccr-codex --all-targets --all-features -- -D warnings` | 退出 0 | `clippy-all-final.log` |
| `just fmt-check` | 退出 0 | `fmt-final.log` |
| `git diff --check` | 退出 0；存在仓库既有 LF/CRLF 提示 | `diff-final.log` |

`ccr-config` 的 ignored unit test 是 `subprocess_adapter_worker`，由两个父测试通过 `--ignored --exact` 实际执行 12 次；不是遗漏的功能验收。两个 CLI 过滤器有重叠，不合计为独立用例总数。

### 过程中的失败与处理

1. 初轮 config tests 为 89 passed、2 failed、1 ignored。inventory 曾扫描共享 `.locks`，遇到其他旧测试的 Windows 文件锁；现仅扫描本 fixture 的 CCR root，并单独检查本资源锁未由查询创建。另一个用例仍断言无效 provider type 被静默丢弃，现改为断言安全的明确错误。后续 config tests 均通过。
2. 纯构造器暴露 profile-off 原有的缺文件路径依赖，相关两个测试失败。已在 T01 拥有的 pointer helper 使用显式 `clear_current_if_present`；profile-off 10 项包含在最终 CLI profile 检查中。
3. 初轮 CLI/Codex Clippy 因移除旧 helper 后残留的文档注释失败，已移除失去附着目标的注释；最终 Clippy 通过。
4. 初轮 `just fmt-check` 报告本任务 `ccr-config/src/lib.rs` 导出排序差异，已定向 rustfmt；最终完整 fmt-check 通过。
5. 首次 `cargo test -p ccr-codex profile` 编译后，测试进程 `target/debug/deps/ccr_codex-0e7a5551c41e43e0.exe` 返回 `0xc0000005 STATUS_ACCESS_VIOLATION`，Cargo 进程返回 5。原因未查明；不根据缓冲日志推断崩溃阶段。原始记录保留于 `cargo-codex-profile.log`。该二进制后续 `--list` 可列出 231 tests；新增 current-query 单测通过；完整 profile filter 使用 `--nocapture` 的复跑及最终复跑均为 18 passed。父任务统一记录该原生崩溃，不以复跑成功推断根因已修复。

早期和中间检查保留于 `implementation-checks*.json` 及对应日志。最终状态以本节和 `implementation-checks-verified.json` 为准。

## 下游接口与兼容约束

- `ConfigManager::for_platform(name)`：新调用者必须显式提供平台。
- `ConfigManager::with_default()`：固定 legacy Claude 域。既有证据为 `crates/ccr/tests/managers/legacy_registry.rs::config_manager_default_ignores_legacy_current_platform_routing`，以及 Claude-only lifecycle clear；父任务已确认保留。未迁移所有旧调用者。
- `snapshot() -> Result<ConfigSnapshot>`：返回配置与原始字节 `version`。
- `mutate` / `mutate_or_create` / `mutate_versioned(expected, create, closure)`：变更闭包只执行一次，不自动重放外部效果。
- `ConfigPatch { fields: IndexMap<String, FieldPatch> }`：`Set(toml::Value)` 设置，`Remove` 显式移除，省略保留；patch 不实现 Debug。
- `ConfigManager::patch(name, new_name, patch, expected, validator)` 与 `ConfigService::patch_config`：`expected` 是必须提交的 read token，auth-mode validator 在锁内执行。
- `base::mutate_profiles` / `mutate_profiles_with_current`：平台 adapter 在闭包中执行校验及变更，后者允许保留空 inactive marker。
- `base::load_current_profile_marker`：只读取实际存储标记；`resolve_file_current_profile` 返回 current 和 repair suggestion；Claude 提供显式 `reconcile_current_profile`。
- `save` / `save_config` / `save_profiles_to_toml`：保留显式整体替换兼容接口。新编辑器不得用锁外读取的整份 sections 执行普通更新。当前平台生产 CRUD 已无 `save_profiles_to_toml` 调用。
- 旧 `ConfigService::add_config` / `update_config` 仍保留其 API-key 校验语义；T03/T04 应使用传入平台 validator 的 patch 入口，避免把该规则用于全部 auth mode。

## 剩余边界

- T02 负责应用级跨文件操作恢复。Codex 删除 secret 与写 profiles 保留原先先后顺序，闭包不重放；单文件 CAS 不提供跨文件回滚。
- T03 的真实 Tauri command 迁移、IPC contract 和前后台联调未在 T01 执行。多进程 fixture 只证明共用仓储适配边界。
- 标准路径以词法规范化识别同一资源，未支持将多个 symlink alias 自动识别为同一目标。
- 未运行完整 Rust workspace tests、Tauri 构建、原生 UI、真实账户、网络流程或父任务 `just ci`。本任务 scoped checks 不能替代父任务最终 gate。
- 独立检查及父任务 requirement-to-evidence ledger 由主会话接续；AC 状态未由本代理勾选。
