# 路径隔离独立复查

状态：PASS_SCOPED。2026-10-07。用户已批准父 research/windows-fixture-isolation-scope.md 的最小路径扩展。路径安全门槛通过后恢复 `ccr.exe`，已解除 22 个临时 Windows ignore。父原生矩阵与完整 just ci 由主会话继续；任务仍为 in_progress。

## 复查边界

- Logger 非空 `CCR_ROOT` 指向 `<CCR_ROOT>/logs`；未设置或空值保留系统 home 回退。只对合成目录初始化 writer 或执行清理。
- 既有 filter、writer、14 天期限、权限 helper、redaction、bridge 保持原样。跨 root 旧日志哨兵与非托管文件不得删除。
- Claude 读取 `ClaudeRuntimePaths::from_env()?.settings_file`；Codex 读取 `CodexConfigManager::resolve_codex_dir()?.join("settings.json")`。不新增配置读取或认证探测。
- Gemini home 优先级为非空 `HOME`、非空 `USERPROFILE`、系统 home。回退测试只解析路径，不读取真实平台设置。
- 保留 ConflictChecker 原 key 提取、冲突分级、建议、warning 和 DTO。
- 项目初始化进程夹具补临时 `CCR_ROOT`。其他夹具只修复已确认的同类缺口。
- 独立安全门槛通过后，删除 output_presentation.rs 的 17 个、output_presentation_rollout.rs 的 5 个临时 Windows 隔离 ignore。保留测试探针的有意 ignore。
- 路径测试、展示 binary、既有 Auth/profile binary、native 终端和 full CI 分别记录。

## 当前检查

已读完整 hook 输出、C3 check.jsonl 所列上下文、PRD、design、implement 和批准方案。已核对旧实现的 `get_log_dir`、`collect_env_vars`、Claude/Codex resolver、ProjectInitFixture，以及三个既有 binary fixture 的 CCR root 覆盖。

当前根分支为 `dev`，ahead 6；已有展示修改及 daemon restart 提交均保留。无提交、推送、归档或真实用户数据读取。

## Findings (fixed)

- output_presentation.rs / output_presentation_rollout.rs：删除已失去原因的 17+5 个临时 Windows ignore。core log_path_probe 与 Doctor renderer child 的 intentional ignore 保留。
- output_presentation.rs：解除同名 warning 测试 ignore 后，删除子进程的 `--ignored` 参数，避免零匹配。最终 warning/PID/成功/退出码/账号快照与运行中子进程断言通过；Drop 只回收受控探针。
- output_presentation_rollout.rs：`DirEntry::metadata` 在 Windows 返回缓存目录 mtime，导致 Doctor 完整 inventory 首测失败。改为 `fs::metadata(&path)`；保留文件字节、文件/目录 mtime 和目录项等式。没有修改 Doctor 生产代码。

## 路径与业务边界

`2026-10-07-path-safety-independent-boundaries.json` 的有界对照通过：logger 仅新增 CCR_ROOT 路径分支；filter、writer、14 天清理、redaction、bridge 保留。ConflictChecker 仅调整 imports/path match；DTO、键提取、冲突分级、建议与读取操作保留。生产与测试模块分界的尾部空白单独归一化；首个 audit assertion 失败的原因记录在 JSON，未当作产品失败。

日志子进程只操作合成目录。两种入口均在 off filter 下创建选定根的 UTC daily 文件，删除 15 天 managed 文件、保留 13 天 managed 和 15 天 unmanaged 文件；另外两个合成根的字节、mtime 和目录项数不变。系统 home fallback 单测只比较路径，没有初始化或读取真实 fallback 目录。

ConflictChecker 单测使用既有 TestHome lock，验证 Claude 显式 settings/config dir、Codex CCR dir/CODEX_HOME、Gemini HOME/USERPROFILE 的优先级，以及合成 model conflict/缺失设置的只读结果。Known Folder fallback 仍由既有 resolver 负责。Windows 权限 helper 仍为 no-op。

独立读取六个同步规范，路径优先级、off filter、14 天期限、默认并行、显式平台夹具与 intentional ignored child 说明均与源码一致。

## 首失败与修正证据

展示 binary 首测 `2026-10-07-path-safe-presentation-first.txt` 为 21 passed / 1 failed / 0 ignored、exit 101。失败只有三个合成目录 mtime；全部文件字节/mtime 与目录项相同。

无 CCR 调用的 `doctor_inventory_probe.rs` 在 20/20 轮复现 Windows cached DirEntry mtime。复测 20 轮 fresh inventory 全等，67 个缓存时间戳差异中 fresh_before/fresh_after 相同。原始 first/retest 输出与 `2026-10-07-doctor-inventory-probe-result.json` 保留。展示修复复测 22/22、0 ignored、exit 0。

## Verification

| 检查 | 结果 | 回执 |
| --- | --- | --- |
| core logging lib | 13 passed、exit 0 | `2026-10-07-path-safety-independent-core.txt` |
| ConflictChecker lib | 5 passed、exit 0 | `2026-10-07-path-safety-independent-conflict.txt` |
| 双 logger test subprocess | 2 passed、exit 0；1 个 intentional child ignore，实际调用 2 次 | `2026-10-07-path-safety-independent-log-process.txt` |
| 展示 binary 修复复测 | 22 passed、0 ignored、exit 0 | C2 `2026-10-07-path-safe-presentation-retest.txt` |
| Grok / Claude / Codex 回归 | 8 / 10 / 14 passed，均 exit 0 | C2 `2026-10-07-path-safe-*-regression-first.txt` |
| fmt | PASS、exit 0 | `2026-10-07-path-safety-independent-fmt.txt` |
| Strict clippy | PASS、exit 0；core/cli/root all-targets/all-features，-D warnings/-D clippy::unwrap_used | `2026-10-07-path-safety-independent-clippy.txt` |
| TypeCheck | PASS、exit 0；core/cli/root all-targets/all-features | `2026-10-07-path-safety-independent-type-check.txt` |
| Scoped diff check | PASS、exit 0；批准路径/夹具及 live specs，raw receipts 保留 | `2026-10-07-path-safety-independent-diff.txt` |

## full CI 首失败的局部复查

主会话首次 just ci 在 root commands 为 138 passed / 1 failed；help::initialized_ccr_init_output_does_not_recommend_retired_platform_init 仍要求旧 profile init 建议。Implementer 只将该断言改为当前完整下一步/查看平台列表/ccr platform list 分组，保留 retired ccr platform init absence、状态成功、临时 CCR_ROOT 与已有 config 夹具。与 init_command 已初始化且 !force 的提前返回分支一致；不进入目录创建或覆盖确认。exact all-features 回归 1 passed、exit 0，fmt/diff 各 exit 0，见 `2026-10-07-gate-help-init-*`。局部回归未记为 full CI PASS。

## Findings (not fixed)

- Unknown 当前没有 native producer；C2 分支仅源码复查，runtime NOT_RUN。不新增业务 producer。
- 完整加密密码交互、其他 OS、hosted 和真实账号均 NOT_RUN。父原生 handler 矩阵及最终 full CI 以主会话回执为准。
- 早期未隔离执行对真实日志的实际变更范围仍未审计；没有读取真实日志或账号补证。
- Codex/Claude 部分 save 等既有失败返回 Ok 仍保留。退出码修复不在已批准展示/路径范围。

当前七个路径/测试源的身份与 Auth 生产源无漂移结果见 C2 `2026-10-07-path-safe-source-manifest.json`。无进一步源码编辑计划。回退只恢复批准的源码、测试和规范；不恢复、删除或修改真实用户数据。提交、推送、归档未授权。
