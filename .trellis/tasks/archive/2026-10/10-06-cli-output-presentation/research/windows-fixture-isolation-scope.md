# Windows 进程夹具隔离：已批准的最小范围扩展

状态：IMPLEMENTED_AND_PATH_ACCEPTANCE_PASS。2026-10-07。用户已明确批准“批准最小范围扩展（推荐）”：允许本方案的日志路径、ConflictChecker 路径与对应夹具最小修复，再继续进程、原生终端及 just ci 验收。该批准不含提交、推送或归档。发现缺口与首次回执保留。

## 已确认的前提冲突

- Windows `dirs::home_dir()` 使用 `SHGetKnownFolderPath`，忽略子进程 `HOME`、`USERPROFILE`。C3 首轮 Doctor 回执出现合成夹具未创建的 model conflict，不能作为完整隔离证据。
- `crates/ccr-cli/src/managers/conflict_checker.rs` 的 `collect_env_vars` 直接读取 Known Folder 下的 `.claude/settings.json`、`.codex/settings.json` 和 `.gemini/antigravity-cli/settings.json`，忽略已有平台路径覆盖。
- `crates/ccr-core/src/core/logging.rs` 的 `get_log_dir` 直接选择 Known Folder 下 `.ccr/logs`。`build_file_writer` 在事件过滤之外创建目录、执行 14 天清理并建立 writer。`CCR_LOG_LEVEL=off` 不关闭初始化。
- Windows 的 `set_owner_only_dir`、`set_owner_only_file` 为 no-op，不能声称 Windows 权限发生变化。已执行进程对真实日志的实际变更范围未审计，未读取真实日志或凭据来补证。
- `crates/ccr/src/main.rs` 的 Doctor/Validate 分支在 logger 初始化前返回。Doctor 的问题是读取路径；普通 Auth/CLI 的问题包括日志路径。
- 已生成的 C2/C3 Doctor 失败回执经布尔审计，未发现 apiKey/api_key conflict 明文。保留首失败及原输出边界，未将 model detail 标为合成数据。

## 建议的最小变更

批准对象是路径解析和相关夹具，展示以外的诊断算法、日志过滤、writer、清理期限、权限规则、bridge、redaction、账号业务与 DTO 保持原有实现。

| 文件/责任 | 建议变更 | 可观察结果 |
| --- | --- | --- |
| `crates/ccr-core/src/core/logging.rs` | 已设置非空 `CCR_ROOT` 时将日志定位到该目录的 `logs/`；未设置时保留当前系统目录回退。复用现有变量。 | 子进程日志及清理只落在夹具的 CCR root。 |
| `crates/ccr-cli/src/managers/conflict_checker.rs` | Claude 复用 `ClaudeRuntimePaths` 的 settings 解析；Codex 复用 `CodexConfigManager::resolve_codex_dir()` 并保留既有 `settings.json` 文件名；Gemini 的 home 明确依次使用非空 `HOME`、`USERPROFILE`、系统 Known Folder。 | 已有平台覆盖及临时 home 生效；不增加文件读取或修改冲突计算。 |
| `crates/ccr/tests/commands/project_init.rs` 及回归检查确认的同类进程夹具 | 该夹具已使用临时 cwd/PATH，但未设置 `CCR_ROOT`；补临时 root。其他夹具只在确认同类缺口时补相同覆盖。 | 正式门槛中的进程不清理或创建真实用户日志。 |
| 当前任务测试/证据脚本 | 路径回归通过后解除新增 Windows tests 的隔离 ignore；恢复 Doctor、Auth、混合流、40/80/120 列及明暗背景验收。 | 恢复当前源码的必需进程及终端证据。 |
| 对应 spec / 当前任务 artifacts | 更新路径优先级、夹具边界、回归断言及批准记录。 | 原展示范围的例外可审查；旧失败保留。 |

不新增依赖、CLI 参数或环境变量。设置了 `CCR_ROOT` 的日志目的地，以及 ConflictChecker 的显式路径覆盖行为会变化；用户已批准这两个变化。

## 验证与回退

1. 在纯解析与隔离子进程中断言目标路径均位于夹具 root；不读取真实配置或日志。
2. 用合成旧日志验证清理作用域与原 14 天规则；用合成跨平台设置验证冲突结果、JSON、流与退出码。
3. 重新执行 C1/C2/C3 必需测试、Windows 原生矩阵及 `just ci`；静态检查不能关闭这些验收项。
4. 独立 trellis-check 复核路径所有权、清理作用域、无凭据输出及回退范围。
5. 回退只恢复经批准的源码、测试和规范；不恢复、删除或修改真实账号和真实日志。没有用户数据迁移。

## 当前未运行项

授权时的 Windows 完整 CLI binary 复测、Doctor native、最终 handler 终端矩阵、完整相关命令套件与 `just ci` 为 NOT_RUN 或 UNVERIFIED。该历史状态保留；路径修复及隔离回归通过后按原验收计划复测。

## 2026-10-07 实施证据

最小路径修复已落盘。新增同类夹具修复为 commands/help.rs 的 version 子命令：该命令经过 logger 初始化，Clap --version/--help 提前返回。logging 13 项、ConflictChecker 5 项、日志隔离进程 2 项通过；隔离探针的有意 ignore 由父测试实际调用。包严格 clippy、fmt 和 scoped diff 检查通过。独立路径检查与完整进程/native/CI 尚未完成。原始回执位于 C3 checks/2026-10-07-path-isolation-*.txt。

## 2026-10-07 独立安全门槛

独立 trellis-check 实跑 logging 13、ConflictChecker 5、隔离 logger 2 项，全部通过。源码有界审计确认日志过滤、writer、14 天规则、redaction、bridge、Conflict DTO/算法保持；合成跨根哨兵字节、mtime 和目录项数保持。允许在显式 CCR_ROOT、平台路径及 home 覆盖下恢复 ccr.exe 进程和原生矩阵。回执见 C3 checks/2026-10-07-path-safety-independent-*.txt 与 boundaries.json。完整验收尚未完成。

扩展范围最终结果：独立安全、实际binary/原生矩阵和完整just ci 16/16通过；日志路径、ConflictChecker路径及同类夹具补充均验收。父AC7其他未运行分支单独保留，本扩展不扩大到Downloads或账号业务。最终源码与HEAD无漂移，未提交或归档。
