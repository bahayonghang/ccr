# Research: CLI、TUI 与共享业务边界架构审查

- Query: 审查 CLI/TUI、共享 crates，以及 Tauri 对相同业务的调用边界；提出可独立验收的重构子任务。
- Scope: internal；规划阶段，只读产品代码。允许写入范围为本 research 文件。
- Date: 2026-09-28
- 方法: 使用 `trellis-start` 加载流程，使用 `su-architecture-first` 的 ownership / source-of-truth / regression-design 方法和 `research` 的一手证据规则。
- 证据分级: **confirmed** 表示当前源码可直接建立事实或确定性控制流；**inferred** 表示由控制流推导的并发交错、故障后果或架构影响；**untested** 表示本轮未执行真实二进制、Rust 测试或故障注入。源码 confirmed 不代表运行验收通过。

## Findings

### 1. 审查边界与当前结构

工作区已经按领域拆分。当前重点缺口是共享业务入口不统一，以及文件写入安全与业务操作一致性之间的边界不完整。不能以 crate 数量、文件行数或 `unwrap` 命中数量认定架构失败。

| 层 / owner | 已核对文件 | 当前职责与边界 |
|---|---|---|
| 二进制与兼容 facade | `crates/ccr/src/main.rs:16`、`crates/ccr/Cargo.toml:25`、`crates/ccr/src/lib.rs:160` | 解析、日志、注入 TUI launchers、兼容导出；`ccr` 已不是所有业务的实际 owner。 |
| CLI/application | `crates/ccr-cli/Cargo.toml:9`、`crates/ccr-cli/src/cli/dispatch.rs:33`、`crates/ccr-cli/src/application/profile_switch.rs:24` | Clap、输出、业务编排、Claude/Grok 平台实现仍在同一 crate。 |
| TUI | `crates/ccr-tui/Cargo.toml:17`、`crates/ccr-tui/src/tui/app.rs:934` | 依赖 CLI application/domain；部分写入编排仍直接在 UI 状态处理函数中。 |
| 配置领域 | `crates/ccr-config/src/services/config_service.rs:90`、`crates/ccr-config/src/platforms/base.rs:476` | Profile 模型、转换、TOML、注册表、CRUD；存在多组不同层级的 writer。 |
| Codex 领域 | `crates/ccr-codex/src/platforms/codex.rs:2254` | 独立领域 crate；运行态/auth 业务与 profile 标记更新组合。 |
| 基础持久化 | `crates/ccr-core/src/core/guarded_write.rs:182` | 路径锁、备份、原子替换；没有承诺上层完整 RMW 事务。 |
| 历史与查询 | `crates/ccr-store/src/history.rs:225`、`crates/ccr-usage/src/lib.rs:1` | 前者写 SQLite 操作历史；后者拥有只读 llmusage SQL projection，明确不运行外部 CLI、不写状态。 |
| Tauri 适配 | `ccr-ui/src-tauri/Cargo.toml:19`、`ccr-ui/src-tauri/src/commands/claude_profiles.rs:295` | 同时依赖 `ccr`、`ccr-cli` 和领域 crate；有直接 trait 调用及 legacy CLI command 调用。 |

Cargo 清单的本地依赖方向未发现循环。`ccr-core`、`ccr-types`、`ccr-usage` 不依赖 CLI。需要保持这些边界。不能将 Tauri 所有功能都改成启动 `ccr.exe`：文件/领域业务已有可复用 Rust API，进程边界只适合真正由外部程序拥有的操作。

### 2. 五条代表调用链

1. **CLI Claude profile 切换**：`main.rs:16` → `cli/dispatch.rs:33` → `commands/claude/profile.rs:154` → `commands/profile/switch.rs:32` → `application/profile_switch.rs:24` → `ClaudePlatform::save_profile/apply_profile` (`platforms/claude.rs:254,279`) → settings CAS (`managers/settings.rs:124`) → profiles current (`claude.rs:328`) → registry (`claude.rs:329`) → history SQLite (`profile_switch.rs:73-92`)。应用层返回结构化结果，但仍存在提前计数及提交后历史失败。
2. **TUI Enter/Space 切换**：`tui/app.rs:859` → `apply_selected:934` → `profile_off_for_platform:960` → `PlatformConfig::apply_profile:979` → 本地递增 usage 并忽略保存错误 (`988-993`)。没有经过上述共享 switch 用例。
3. **Tauri profile apply**：`claude_profiles.rs:295` / `codex_profiles.rs:231` → `spawn_blocking` → 各平台 `apply_profile` → DTO；Codex handler 另行使 dashboard cache 失效 (`245`)。没有执行 CLI application 中的 usage/history 编排。通用 IPC 审计不能替代这些领域副作用。
4. **CLI validate**：`dispatch.rs:48` → `commands/lifecycle/validate.rs:30` → `ConfigService::with_default/validate_all` (`39-41`) → `ConfigValidator::validate_section` (`config_validator.rs:95`) → API-key 通用校验 (`config/types.rs:160`)；随后另行判断 Claude/Codex auth，最后 `generate_report:279`。错误聚合没有传回非零退出。
5. **配置 CRUD / 落盘**：Tauri `config.rs:add_config/rename_config` (`209,291`) 或 shared `ConfigService` (`config_service.rs:205`) → `ConfigManager` → `ConfigFileHandler::save:112` → fileio → guarded path lock → AtomicWriter。平台 profile CRUD 另经 `base::save_profiles_to_toml:476`。最终单次替换安全，但调用方 RMW 不同锁名及锁范围仍可丢更新。

### 3. 优先问题

以下 10 项是证据条目，不应作为 10 个互不相关的根因计数。CLI-02/03/04 归入共享切换用例与事务；CLI-01 中的 Tauri generic config 重复写入应与后台报告合并；CLI-05/06 归入诊断与查询边界；CLI-07 属于适配器迁移。CLI-08/09/10 可作为对应子任务的验收或兼容工作，不必各建独立子任务。

#### CLI-01 — P1：profile RMW 锁范围不覆盖实际读取，且同一文件有不同业务锁

- **事实 / confirmed**：Claude `save_profile` 在 `crates/ccr-cli/src/platforms/claude.rs:258-262` 先读取完整 profiles、替换一项，再调用共享保存器。共享保存器直到 `crates/ccr-config/src/platforms/base.rs:482-484` 才加 `platform_profiles_{name}` 锁；锁内只从最新文件保留 current/default/settings，`sections` 来自传入的旧快照 (`489-504,533-539`)。
- **第二入口 / confirmed**：`ConfigService` 的 RMW 使用 `ccr_config` (`crates/ccr-config/src/services/config_service.rs:98`)，Tauri add/rename 使用 `config` (`ccr-ui/src-tauri/src/commands/config.rs:225-227,295-297`)。这些路径可经 `ConfigManager::with_default` 写同一平台的 `profiles.toml`。
- **触发 / inferred**：两个调用者都读取 S0；A 修改 profile A 并保存 S1；B 在随后取得 writer 锁后仍写入从 S0 构造的 sections，抹除 A 的修改。无须同时进行字节写入即可发生。
- **影响**：新增、编辑、删除、usage_count 更新可能互相覆盖；备份与原子替换不能补足 RMW 边界。
- **根因 / owner**：`ccr-config` 没有唯一、包含读取和变更函数的 profile repository mutation API；底层写锁被误当作完整业务锁。
- **最小重构**：在 `ccr-config` 建立按实际配置资源定位的 mutation API，将读、变更、校验、写入置于同一 RMW 锁内，或用版本令牌比较后重放确定性 patch；CLI/TUI/Tauri 均委托。保持 guarded path lock 为 leaf lock，不能嵌套重复获取同一非重入锁。
- **验收**：使用 barrier 控制两个独立进程读写同一个 fixture；分别修改不同 profile，最终两项均存在；删除与编辑冲突返回明确 conflict 或确定策略；CLI-service / Tauri-handler / platform adapter 两两组合覆盖；检查锁超时及 secret/backup 不回退。
- **状态**：锁与读写次序 confirmed；丢更新交错 inferred；并发运行 untested。

#### CLI-02 — P1：TUI 在验证目标前独立提交 off，失败的 apply 不能恢复旧配置

- **事实 / confirmed**：`crates/ccr-tui/src/tui/app.rs:934-979` 只检查选中项和实例存在，然后先 off，再 apply。目标语义验证在平台 `apply_profile` 内才发生，例如 `crates/ccr-cli/src/platforms/claude.rs:279-302`。
- **提交点 / confirmed**：Claude off 在 `crates/ccr-cli/src/application/profile_off.rs:256-261` 清 managed env、registry 指针及 profiles 指针，随后 `backup.commit()`。off 返回后 apply 失败，不在这个 rollback guard 的作用域。
- **触发**：当前配置可用，选中无效目标，或目标在选择后被改坏/删除，或 apply 的后续 I/O 失败。
- **影响 / inferred**：界面报告“切换失败”，旧 runtime/指针已经清除；Codex off 还可能清 auth.json，须按其登录准备规则验证恢复边界。
- **根因 / owner**：TUI 状态层自行拼装两个独立提交的领域操作。
- **最小重构**：共享应用用例先解析并验证目标、建立变更计划，再执行受一致性边界约束的必要 cleanup 和 apply。保留 `.trellis/spec/ccr-cli/backend/profile-off-login-prep.md:22-36` 的清理所有权与 off 错误中止规则；不要直接删掉 cleanup 来掩盖缺陷。
- **验收**：TUI 以注入 backend 测试 invalid target、target removed、runtime write failure、marker failure；前置失败后旧文件字节及旧指针不变；需要 cleanup 的成功切换仍满足 auth/profile-off contract。
- **状态**：控制流 confirmed；文件后果 inferred；本轮未启动 TUI。

#### CLI-03 — P1：多文件切换缺少统一完成语义，失败可能发生在实际生效之后

- **事实 / confirmed**：`application/profile_switch.rs:60-71` 在 apply 前持久化 usage_count；`71-92` 在 apply 后初始化历史数据库并写 history，任一 `?` 失败都返回整体 Err。`crates/ccr-store/src/history.rs:180-182,243-261` 表明历史依赖单独 SQLite 初始化/插入。
- **多文件边界 / confirmed**：Claude 先 settings CAS，再 profiles marker，再 registry (`platforms/claude.rs:317-329`)；该方法没有覆盖全链的 operation lock 或 rollback。Codex 对应步骤在 `crates/ccr-codex/src/platforms/codex.rs:2261-2279`，还忽略 auth-registry 同步错误 (`2281-2286`)。Grok 已在 `platforms/grok.rs:1033` 使用 operation lock，应作为可保留的局部机制，不能声称所有平台都没有锁。
- **触发 / inferred**：runtime 写成功后 profiles/registry 不可写，或者 SQLite 打不开；也包括 Claude 并发切换 A/B，runtime 与 marker 最后写入者不同。
- **影响**：返回失败而新 runtime 已生效；重试可重复计数；旧指针/新 runtime 组合误导界面与后续 off；单个 `Result<()>` 无法表达 applied-with-warning / partial / unchanged。
- **根因 / owner**：跨运行态、元数据和审计的成功定义不明确。应用层必须拥有操作结果，平台层必须拥有 runtime/指针一致性。
- **最小重构**：定义 `SwitchOutcome` 与明确提交点；失败前验证全量条件；序列化同一平台的 CCR 写操作；写后发生审计故障时按明确策略返回可识别警告或持久化待补偿记录。跨文件回滚必须检查版本，避免覆盖其他进程的新写入。不要把 blanket ignore / fallback success 当作修复。
- **验收**：对 settings、profiles、registry、history 每个阶段注入失败；断言结果类别、所有文件版本、计数、history 条数；两进程相反顺序交错后 runtime/current 一致；重试不会重复记录成功。
- **状态**：提交次序 confirmed；故障后果 inferred；故障注入 untested。

#### CLI-04 — P2：禁用策略、usage_count 和 history 在入口间不一致

- **事实 / confirmed**：`commands/platform/profile.rs:491-519` 持久化 `enabled=false`；旧 `ConfigService::set_current` 拒绝禁用项 (`config_service.rs:283-290`)。新 switch 用例没有 `is_enabled` 检查 (`application/profile_switch.rs:33-71`)；Claude 的 `validate_profile` 只检查 auth/字段 (`platforms/claude.rs:339-346`)，通用字段 validator 不检查 enabled (`config/types.rs:160-194`)。因此禁用的有效 Claude profile 仍可通过平台 scoped CLI/Tauri apply。
- **入口差异 / confirmed**：CLI 在 apply 前计数并写 history；TUI 在 apply 后计数并吞掉保存失败 (`tui/app.rs:988-993`)；Tauri Claude/Codex apply 直接调 trait (`claude_profiles.rs:295-309`、`codex_profiles.rs:231-246`)，不会执行 application 层这些副作用。
- **影响**：用户的禁用选择不构成可靠操作限制；同一切换的统计、历史和失败报告由客户端决定。
- **根因 / owner**：共享 trait 混合低层 apply 和用户操作入口，客户端自行选择副作用。
- **最小重构**：共享命令用例承载 enabled policy、计数和领域历史；底层 adapter API 明确只供内部 use case 使用。若禁用仅用于列表过滤，必须先明确产品契约并移除现有互相矛盾的行为描述，不可静默改变含义。
- **验收**：同一个 fixture 用 CLI、TUI backend、Tauri handler 执行相同请求，状态文件、计数和 history 等价；禁用目标失败且无写入；计数保存失败在 outcome 中可见。
- **状态**：Claude 路径 confirmed；Codex/Grok 的全部 auth 模式等价测试 untested。

#### CLI-05 — P2：validate 同时存在错误退出码和校验策略分裂

- **事实 / confirmed**：`commands/lifecycle/validate.rs:279-307` 在 `has_errors=true` 打印错误后仍返回 `Ok(())`。`crates/ccr/src/main.rs:47-48` 只在 dispatcher Err 时走错误退出，因此被该函数汇总的错误不会产生非零退出。
- **错误吞没 / confirmed**：`validate.rs:255-276` 将读取平台/当前 profile 的所有错误转成 `None`，随后将其作为未配置而跳过 runtime/auth (`197-200,245`)。读取失败与没有 profile 没有区分。
- **策略分裂 / confirmed**：通用 `ConfigValidator` 调用强制 `base_url/auth_token` 的 `ConfigSection::validate` (`config_validator.rs:95-108`、`config/types.rs:160-185`)；Claude platform 明确允许 subscription 的可选字段 (`platforms/claude.rs:339-345`)。因此合法 subscription profile 可在 apply 验证通过而在通用 validate 报错。
- **影响**：脚本门禁假成功；合法配置被报错；损坏/不可读状态被降级成 skip。
- **根因 / owner**：CLI handler、`ValidateService`、平台 validator 分别拥有部分规则。已有 `ValidateService` 没被该 CLI handler 采用，其 settings 校验也未统一 auth-mode 语义 (`services/validate_service.rs:88-114`)。
- **最小重构**：建立按平台/认证模式输出 typed report 的诊断用例；区分 unconfigured / invalid / unreadable；输出层只负责 human/JSON；在二进制边界将 report severity 映射退出码。
- **验收**：API-key、subscription、missing、corrupt、unreadable、warning-only 六类 fixture；错误非零，warning 按文档策略；同一 profile validate/apply 的规则一致；只读诊断不创建/修改文件。
- **测试缺口 / confirmed**：`crates/ccr/tests/commands/validate.rs:125-161` 两个集成用例只断言成功与文字；本轮查看的该文件没有无效配置退出码断言。
- **状态**：源码行为 confirmed；二进制执行 untested。

#### CLI-06 — P2：查询调用隐式修复文件，默认平台依赖注册表顺序

- **事实 / confirmed**：`ConfigManager::with_default` 注释宣称当前平台，但实际取 `list_enabled_platforms().next()` (`crates/ccr-config/src/managers/config/manager.rs:31-46`)；该列表来自 IndexMap 迭代次序 (`platform_config.rs:222-227`)。`build_for_platform` 在文件缺失时直接创建默认 profiles (`manager.rs:89-108`)。
- **查询副作用 / confirmed**：`ConfigService::list_configs/get_current/validate_all` 走 autofix，必要时保存 (`config_service.rs:108-110,310-315`；`config_file_handler.rs:84-100`)。Claude `get_current_profile` 的 stable read 会修复另一份 marker 或清理 registry (`platforms/claude.rs:138-158`)。
- **影响 / inferred**：查看列表/诊断可能需要写权限、产生备份、改变旧状态；不同注册表顺序可令 generic config 读写到不同平台；只读 IPC 分类难以保证只读。
- **根因 / owner**：发现路径、bootstrap、repair 与 query 没有分离；同一个 current fact 同时存于 profiles 和 registry。
- **最小重构**：新业务必须显式平台；纯查询返回状态和 repair suggestion；bootstrap / reconcile 是显式 mutation。为兼容旧 API 保留明确 deprecated adapter；制定 current-profile authority 与冲突规则。
- **验收**：更改 registry 顺序不改变平台 scoped 结果；对整个 fixture 目录做前后内容/文件清单比较，list/current/validate 只读；只读文件权限仍允许查询；显式 repair 才改 marker。
- **状态**：隐式写入与排序事实 confirmed；具体页面触发频率由 Tauri 审查补充。

#### CLI-07 — P2：应用服务和 CLI 呈现仍共包，legacy Tauri 入口已调用必失败 handler

- **事实 / confirmed**：`ccr-cli` 同时暴露 `cli/commands/application/services/platforms`，包含 Clap、dialoguer、colored、comfy-table (`crates/ccr-cli/Cargo.toml:20-43`)；Tauri 同时依赖 facade 与 CLI (`ccr-ui/src-tauri/Cargo.toml:20,28`)。依赖方向本身不证明性能问题，本轮没有测量包体或启动时间。
- **实际错误 / confirmed**：Tauri `switch_config` 调 `ccr::commands::switch_command` (`commands/config.rs:200-204`)；后者无条件返回 migration error (`crates/ccr-cli/src/commands/profile/switch.rs:24-26`)。生成 API 仍导出 `switchConfigTyped` (`ccr-ui/src/api/generated/config.ts:26`)。该 endpoint 当前不能成功切换。汇总复核已确认 Configs 页面仍调用该 endpoint（`ccr-ui/src/features/configs/hooks/useConfigsPage.ts:55`）；结论限于配置页这条切换/启用链，不能扩展为全部 Profile 页面不可用。
- **边界风险 / confirmed**：`doctor_command` 在库里的命令处理层直接 `process::exit(1)` (`commands/doctor_cmd.rs:49-51`)；目前未发现 Tauri 直接调用该 handler，因此不能声称已造成桌面退出。该模式说明输出层不适合作为嵌入式业务 API。
- **最小重构 / owner**：先建立稳定的无终端副作用应用 API，迁移相同功能调用者，再按依赖需要将 application / 平台服务逐步移到独立共享包。无需一次拆完全部 crate；不要用 facade/prelude 重导出代替 owner 迁移。legacy endpoint 按兼容策略明确删除或映射到带平台的新版 API。
- **验收**：桌面业务路径不调用 `ccr::commands::*`；共享 API 无 stdin/print/process-exit；Cargo 依赖检查确认领域包不依赖 presentation；IPC manifest、生成 API、前端消费者一起更新；保留 binary help/exit compatibility tests。
- **状态**：endpoint 控制流 confirmed；当前页面调用与性能影响 untested。

#### CLI-08 — P2：同秒备份使用相同文件名并覆盖，无法保留每次变更的前镜像

- **事实 / confirmed**：guarded writer 的时间戳精度为秒 (`crates/ccr-core/src/core/guarded_write.rs:264`)，SameDir 与 Dir 命名不带序列或唯一标识 (`277-299`)，`copy_backup` 用 `fs::copy` 到计算出的路径 (`306-312`)。同一个资源、同一个 tag/prefix、同一秒的第二次备份目标名相同。
- **触发 / inferred**：一次复合切换或快速连续保存产生两次以上备份；后者覆盖前者。写锁保证次序，但不保证备份唯一性。
- **影响**：最新改动的前镜像可能存在，更早的原始状态已经被覆盖；keep-10 只保留不同文件名，不能保证十次修改历史。
- **根因 / owner**：`ccr-core` 的备份身份等于秒级时间，而不是一次备份操作。`ProfileOffBackup` 同样以 label+秒级 UTC 建目录 (`application/profile_off.rs:82-86`)，需一起评估。
- **最小重构**：在保持旧文件可发现的前提下增加唯一序列/随机后缀与 create-new 语义；更新命名 contract 和旧/新文件的排序、恢复、轮换逻辑。现 spec 明确冻结旧命名（`.trellis/spec/ccr-core/backend/atomic-writer.md`），因此必须是计划中的兼容变更。
- **验收**：注入固定时钟，在同一秒连续保存三个不同内容，三份前镜像不被覆盖；旧格式备份仍列出/可恢复；轮换和 DACL/权限测试通过。
- **状态**：路径生成与覆盖调用 confirmed；真实快速写回归 untested。

#### CLI-09 — P3：保留的 Gemini/Droid adapter 不满足当前安全写入基线

- **事实 / confirmed**：Gemini `save_settings` 直接 `fs::write` (`platforms/gemini.rs:112-123`)，apply 构造全新仅含 google 的 settings (`248-259`)；Droid writer 同样直接写 (`platforms/droid.rs:115-127`)，registry 更新也直接 manager RMW (`250-257`)。这些路径没有共用 secret/backup/CAS policy。
- **限制 / confirmed**：`Platform::auth_profile_supported` 现在只包含 Claude/Codex/Grok (`crates/ccr-config/src/models/platform.rs:115-120`)，而工厂仍可创建 Gemini/Droid (`crates/ccr-cli/src/platforms/mod.rs:59-69`)。因此这是保留公共 API/旧适配器风险，不能直接归为当前主 UI 数据丢失。
- **影响 / inferred**：若旧 API 或未来入口继续启用这些 adapter，可能覆盖未知 settings 字段、发生非原子写入及权限基线缺失。
- **最小重构**：先完成能力矩阵与调用者清单；明确 deprecated/disabled 边界。仍需支持的 writer 使用现有 guarded/versioned writer 并保留未知字段；不支持的路径不得被工厂误表述为完整可用。
- **验收**：能力枚举、CLI/IPC 暴露、测试一致；可用路径的 fixture 未知字段保留、写失败旧文件不变、secret 权限受控。
- **状态**：代码 confirmed；当前用户可达面未全证实。

#### CLI-10 — P3：规范与回归重点仍偏单入口，尚未约束跨入口结果等价

- **事实 / confirmed**：`.trellis/spec/ccr-config/backend/backend-guidelines.md:29` 仍将 `with_default` 称为 current-platform caller；实际实现改为首个 enabled。`commands/doctor_cmd.rs:29` 手写平台集合缺 Grok，而 auth/profile 支持集合已含 Grok。这些具体差异说明共享能力模型未成为所有入口的唯一来源。
- **测试证据**：TUI 的 apply regression 在 `tui/app.rs:1750-1754` 只覆盖无实例；本轮检索未发现 off 成功后 apply 失败的组合回归。`validate` 集成测试仅有两条成功路径。shared settings CAS、Grok operation lock、secret writer 则已有针对性测试，应保留。
- **根因 / owner**：规则在多个 spec/handler/adapter 中独立描述，单入口单元测试不能约束副作用与结果分类。
- **最小重构**：建立操作 × 平台 × 入口的行为契约；每个后续修复同步所属 spec 与失败路径测试；能力列表从共享类型/registry 派生。保持生成物漂移检查，不把快照“更新通过”当作语义验证。
- **验收**：新增 cross-entry contract suite，至少覆盖 apply/off/CRUD/validate；规范中的路径、平台集合、owner 与代码一致；运行相关 package gates，最终跨模块交付再运行 `just ci`。
- **状态**：已列具体差异 confirmed；没有宣称全仓测试不足或当前 gates 失败。

### 4. 应保留的设计

1. **TUI launcher 注入**：`crates/ccr/src/main.rs:35-52` 在 binary 构造函数指针，`ccr-cli/src/cli/dispatch.rs:11-26` 定义注入点；避免 CLI↔TUI 循环依赖。
2. **共享 Secret 与 managed-env registry**：`ccr-config/src/models/platform.rs:167-173`、`ccr-cli/src/platforms/claude.rs:353-357`，secret 类型与键集合可作为跨端边界。不要再在前端或 handler 手写清理键集合。
3. **Claude settings 乐观并发更新**：`crates/ccr-cli/src/managers/settings.rs:124-148` 有版本比较、有限次重放及明确冲突错误；失败不伪造成功。
4. **guarded writer 的 leaf lock 与 Windows 原子替换策略**：`ccr-core` 已集中安全文件写入、secret 权限和备份策略；需要补上调用者 RMW，不应另造一套 Tauri writer。
5. **Grok operation lock、Codex 独立领域 crate、只读 usage crate**：分别提供操作串行、明确领域 owner、只读 SQL 投影边界。重构应复用现有边界并补齐覆盖范围。
6. **profile-off 共享用例及本地快照**：已避免三端复制 clear 列表；需要完善复合 apply 的一致性，不能退回各客户端自行清理文件。

### 5. 建议子任务与依赖

以下是父任务规划建议，本研究没有创建或启动任何任务。

| 子任务建议 | 主 owner / 对应问题 | 依赖与独立验收 |
|---|---|---|
| A. 统一 profile repository mutation 与备份身份 | `ccr-config` / `ccr-core`；CLI-01、CLI-08 | 先明确资源锁与旧备份兼容；用并发/固定时钟 fixture 独立验收。 |
| B. 统一 apply/off 应用用例和结果类型 | shared application + 各 platform；CLI-02、CLI-03、CLI-04 | 依赖 A 的写入边界；定义 prepare/commit/outcome、enabled、usage/history；先建立可调用 API。 |
| C. CLI、TUI、Tauri 适配器迁移 | `ccr-cli` / `ccr-tui` / Tauri commands；CLI-04、CLI-07 | 依赖 B；移除客户端拼装业务流程；比较跨入口同 fixture 的状态与副作用。 |
| D. 纯查询与统一诊断 | `ccr-config` / application diagnostics；CLI-05、CLI-06 | 可先设计，与 A 对齐纯读取/repair API；覆盖 exit code、auth 模式与未配置/损坏区别。 |
| E. 能力/规范/兼容面收敛 | 各 owner；CLI-09、CLI-10 | 依赖 B/C 的新边界；旧 adapter 保留/禁用需显式策略；不是顺手扩大平台支持。 |

共同验收至少包含：同资源多进程并发、每个写阶段故障注入、只读前后文件清单、secret sentinel 不出现在 DTO/log、旧 profile 格式迁移、旧备份恢复，以及真实三入口的同一操作结果。新的用例 API 必须证明行为等价后，才考虑物理拆包与大规模 imports 迁移。

### 6. 已读规范与版本依据

- `code_map.md`、`crates/AGENTS.md`、`.trellis/workflow.md`。当前发现的 crates 范围没有额外 nested AGENTS/CONTEXT；根 `CONTEXT.md` 不存在，本轮未创建。
- `.trellis/spec/ccr-cli/backend/index.md`、`backend-guidelines.md`、`test-fixtures.md`、`profile-off-login-prep.md`。
- `.trellis/spec/ccr-config/backend/index.md`、`backend-guidelines.md`。
- `.trellis/spec/ccr-core/backend/index.md`、`atomic-writer.md`。
- `.trellis/spec/ccr-tui/backend/index.md`、`backend-guidelines.md`。
- `.trellis/spec/ccr-codex/backend/index.md`、`backend-guidelines.md`。
- 清单基线：根 workspace version `7.3.0`、Rust minimum `1.95`；Tauri manifest pinned `2.11.5`。仅按当前本地 Cargo 清单记录，未查远端版本。
- 外部参考：无。本审查针对当前仓库行为，没有引入通用架构文章来替代源码证据。

## Caveats / Not Found

- 本轮没有运行 `cargo test`、`just ci`、桌面/TUI 界面或真实用户配置 mutator，没有证明运行性能问题、Windows 视觉行为或所有平台 runtime 行为。没有产品代码、spec、git 状态或任务生命周期修改。
- 对生产 panic 的定向扫描命中主要为 `#[cfg(test)]` / tests 文件与文档例子。本轮没有据此列“生产大量 panic”缺陷；错误吞没问题均使用上方具体调用链说明。
- `ccr-usage`、`ccr-db`、checkin、sync、skills 的内部算法没有完成逐函数审计；这里确认其依赖/owner 边界并沿配置与历史链抽查。对应领域若单独重构，仍需独立研究。
- Tauri generic `switch_config` 的注册与生成 API 存在已确认，当前 UI 页面是否还有活跃调用尚未证明；主平台 Profile 页面另走各自 apply handler。
- 所有并发与故障后果已区分 source facts 和推断。实施前应把相应交错/故障点固化为失败测试，不应把本报告当作已经完成回归验证。
