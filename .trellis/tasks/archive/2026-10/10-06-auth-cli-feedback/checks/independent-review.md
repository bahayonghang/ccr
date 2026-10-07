# C2 独立复查

日期：2026-10-07。HEAD：`528d4bae1b145fd74d7bbc97454caac249a4b896`，加工作区 Auth 展示改动。任务保持 `in_progress`，验收未完成。

## 当前路径隔离复查补充

用户已批准最小路径修复。独立复查确认非空 CCR_ROOT、Claude/Codex 既有路径 resolver 与 Gemini home 优先级生效；core logging 13、ConflictChecker 5、双 logger 合成子进程 2 项通过。过滤、writer、14 天期限、权限、redaction、bridge 和冲突算法均保留。此前 Findings (not fixed) 第 1/2 项已由该扩展解决；旧真实日志实际变更范围仍未审计。

已解除 output_presentation.rs 的 17 个和 rollout 文件的 5 个 Windows 临时 ignore，同时移除同名 warning 子探针的 `--ignored` 参数，保留其他 intentional ignored probes。展示首轮 21/22：Doctor fixture 的 DirEntry::metadata 缓存目录 mtime。无 CCR 调用的 20 轮合成探针复现该差异；fixture 改为 fresh fs::metadata，完整 inventory 等式保留。展示复测 22/22、0 ignored，运行中警告探针实际就绪。Grok/Claude/Codex 8/10/14 回归均 exit 0。

当前 fmt、三个 affected packages strict clippy/type-check 均 exit 0。结果与源身份见 `2026-10-07-path-safe-source-manifest.json`；完整回执索引见 verification.md。Unknown native producer、完整加密密码交互、其他 OS、hosted 与真实账号缺口保留。父原生矩阵与最终 full CI 由主会话继续。

补充读取父 immutable binary identity 与 native acceptance JSON，核对 40/80/120 × dark/light × normal/NO_COLOR/dumb 共 18 格。逐格检查 save screen：首行符合对应模式，描述/邮箱缩进、掩码邮箱、下一步和完整 list 命令均存在；逐格 missing-fields screen 不含描述/邮箱空字段。原生 AC1 局部证据通过，C2 第一条已勾。全 native matrix 与最终 full CI 仍以父独立汇总为准。

以下为授权前的独立复查历史，保留首次失败、未隔离状态和原始发现。

## Findings (fixed)

- 文件：`crates/ccr-cli/src/commands/codex/auth/import.rs`。全零 ImportResult 先输出中性结果，再追加重复空状态 warning。纯 renderer 新增 empty case，首测退出 101；删除重复 warning 后 import 3 项单测通过。账号服务、DTO、返回值和导入模式保持原样。
- 文件：`crates/ccr/tests/commands/output_presentation.rs`。既有平台列表测试将已配置平台数当作注册表总数，并使用错误的 `[platforms.claude]` 夹具。改用注册表数量与 `[claude]`，保留 JSON 数量、Claude enabled、输出中性断言。平台与 Doctor 测试拆分。修正后 binary 复测 `NOT_RUN`。
- 文件：同上。新增受控 Windows `codex.exe` 子进程，编排已批准的运行中 warning 分支，正常结束及 panic 都只回收该子进程。首轮已到达 warning、PID、成功结果和建议断言，随后错误地要求 byte 不变；当前断言仅允许既有 JSON 重排及 `OPENAI_API_KEY:null` 省略，其他值必须相同。账号快照仍须 byte 不变。修正后 binary 复测 `NOT_RUN`。
- 文件：同上。子进程 cwd 改为夹具目录，并移除继承的 `ANTHROPIC_BASE_URL`。发现 Windows Known Folder 绕过 HOME 后，按主会话指示为全部 17 项新文件进程测试添加 Windows ignore，保留跨平台编译与具体原因。ignore 不算通过。

## Findings (not fixed)

1. `crates/ccr-core/src/core/logging.rs` 的 `get_log_dir` 直接解析 `dirs::home_dir()/.ccr/logs`；初始化 file writer 包含创建目录、14 天清理和建立日文件的路径。Windows dirs 6 调用 Known Folder，不接受 HOME/USERPROFILE。`CCR_LOG_LEVEL=off` 只控制 filter。Windows 权限 helper 为 no-op，不声称权限改变；日志实际变更范围未审计。现有进程证据只能证明夹具 credential/runtime 断言，不能证明全部副作用隔离。改变 logger 路径契约超出 C2 展示范围；主会话已停止后续 binary/native 运行。
2. `crates/ccr-cli/src/managers/conflict_checker.rs::collect_env_vars` 使用相同 Known Folder home 读取平台设置。Doctor 首回执含夹具之外的模型配置。保留首回执，标为 `UNVERIFIED`；不改 service，交 C3/主会话处理。只对已保存回执做 `global.conflicts.detail` 布尔审计，没有发现 apiKey/api_key/token/password/secret 关键字值；不读取真实配置或打印值。审计不代表所有凭据通道均已验证，见 receipt-secret-audit.json。
3. Codex/Claude save、switch、delete 等部分失败后返回 Ok 的既有行为未修复。恢复建议、stderr 错误和已执行的基线 zero-exit 断言保持原语义；错误传播属于范围外。
4. `LoginState::Unknown` 是 forward-compatible 类型分支；当前 CodexAuthService 的 native snapshot 只产出已定义状态。Unknown 分支已源码审查：保留 warning，不新增成功结果或操作建议。实际进程 `NOT_RUN`，不声称合成 binary 覆盖。
5. 完整 binary import/export、密码交互、未检验的 repair/sync 分支、TTY、40/80/120 列、明暗背景、其他 OS、真实账号及 hosted 均未完成。Windows Downloads 也来自 Known Folder，禁止以环境覆盖已设为由操作真实 Downloads。

## 源码复查

21 个 Auth 文件逐项复核。当前 production 与当前 HEAD 的 service 调用、DTO 声明、JSON serializers 和提前返回数量的有界审计一致；详见 `2026-10-07-independent-boundary-audit.json`。该 regex 审计不替代控制流审查或进程证据。

手工复核了 save/list/current/switch/delete/rename/import/export/sync/repair/update/off 的展示变化、JSON 提前返回、取消与确认、加密说明、诊断置信度、mask helpers 和流归属。已提交 daemon restart 是当前 HEAD 的已有功能，保留其参数、warning、stderr、结果与退出码处理。C2 未修改业务服务或参数。

原始处置 203 项：迁移 162、共享覆盖 34、保留 7。全零导入的原 warning 处置理由已修正；其他源清单不撤销。当前 source SHA256 清单见 `2026-10-07-independent-source-manifest.json`。

## Verification

| 检查 | 结果 | 边界 |
| --- | --- | --- |
| `just fmt-check` | PASS，exit 0 | 当前源码；不启动 CCR |
| `just lint-strict` 首轮 | PASS，exit 0 | 当前 C2 稳定源码；workspace/all-targets/all-features；最终 C3 集成复测仍待主会话 |
| `just check-workspace` 首轮 | PASS，exit 0 | 当前 C2 稳定源码；不启动 CCR；最终 C3 集成复测仍待主会话 |
| Codex Auth unit filter | 6 PASS | 含已提交 daemon 决策 1 项；不初始化 CLI logger |
| Claude Auth unit filter | 1 PASS | shared off JSON DTO |
| import unit 修复复测 | 3 PASS | renderer 空/跳过/部分/覆盖，service fixture 合并和失败传播 |
| 根展示首个有效运行 | 14 PASS / 1 FAIL，exit 101 | 12 Auth 均通过；旧平台数量断言失败；logger 未隔离 |
| 根展示第二运行 | 17 PASS / 4 FAIL，exit 101 | 新 warning 测试与三个非 Auth 断言失败；全部首失败保留 |
| Grok / Claude / Codex profile 回归 | 8 / 10 / 14 PASS | 夹具数据断言；logger 未隔离；均非全流程合成验收 |
| binary 修正后的复测 | NOT_RUN | Windows logger 未隔离，主会话停止运行 |
| native / Linux / macOS / hosted / real account | NOT_RUN | 不将源码或 ignore 当通过 |

lint/type-check 主会话回执位于父 checks/2026-10-07-continuation-{lint-strict,check-workspace}-first.txt；两个退出码与 static-results.json 一致。C2 scoped git diff --check 退出 0，回执为 2026-10-07-independent-diff-check.txt。最后确认 C2 source manifest 无漂移，17 项 binary 测试均有 Windows ignore。

最早 presentation-first.txt 记录的是 PowerShell receipt 路径表达式错误，不能判断产品。presentation-retest-1.txt 才是首个完整产品失败回执；两个文件均保留。所有历史 first/retest 文件保持原样。

回退只恢复 C2 展示行和新测试/任务记录；不恢复真实账号，不操作日志、认证文件或外部服务。源码与回执均未提交，未归档。
