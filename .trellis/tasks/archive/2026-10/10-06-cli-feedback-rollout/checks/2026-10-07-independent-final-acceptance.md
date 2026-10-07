# CLI 展示与路径修复：独立最终验收

状态：LOCAL_GATES_PASS_AC7_UNVERIFIED。2026-10-07。父任务及 C1–C3 仍为 `in_progress`。没有提交、推送或归档。

独立审计完成：24 个补充进程 case、8 份 JSON、18 格实际 Windows console、162 个终端 case、36 次从屏幕提取建议后的执行、12 格共享输出混合流、最终源码身份和完整 CI 原始汇总。AC7 保持开放。

## Findings (fixed)

- 文件：父 `checks/native-smoke.ps1`。初始矩阵缺少实际终端缺字段保存及普通/前导连字符建议复制执行。主会话补充这些 case，并在每个 case 前清空窗口。独立 validator 核对 18/18 格、字段缩进、正文颜色和 36 次复制命令退出码，均通过。
- 文件：C3 `checks/native_acceptance_validator.py`。新增只读校验入口，核对实际 Win32 buffer 的字符、palette、颜色属性、宽度及 immutable binary SHA。校验不执行产品。
- 文件：`crates/ccr/tests/commands/help.rs`。完整 CI 首测保留为失败；旧断言仍要求迁移前的 profile-init 建议。实施及路径检查角色将断言对齐已初始化分支的新分组，保留退役命令缺席检查。完整 CI 随后复测通过。该修正仅涉及测试，原生验收对应的产品源码未变。

路径解析、Windows fresh metadata 和受控 warning child 参数的修正由独立路径检查报告记录；该最终验收未改产品、测试或 spec。

## 当前源码与构建身份

- HEAD：`528d4bae1b145fd74d7bbc97454caac249a4b896`。
- 当前 82 个产品/测试/文档/spec 文件的 SHA-256 与父 `2026-10-07-isolation-final-source-manifest.json` 全部一致。
- 原生使用临时固定 `ccr.exe`，SHA-256：`26560b7ae47654a54f12e90cc75f99622439bd21a6af04c15c876a139a7475c6`。24-process、18-native 回执与该副本一致。
- 原生构建 manifest 与最终 manifest 仅相差 `crates/ccr/tests/commands/help.rs` 的测试断言；没有产品源码漂移。后续 all-features/release artifact 没有替换原生副本身份。
- 当前混合流探针 SHA-256：`4a66ee20b082d78f76c4060430250d12250fee07c9770e0d2ab29258bba21b7b`。该可执行文件只调用共享 renderer，不调用账号或 Doctor service。
- C2 原有 21 份产品源码、C3 原有 45 份产品源码与各自独立复查 manifest 一致。解除 ignore 和 fresh metadata 修改后的测试使用新的路径安全回执。

身份审计见 `2026-10-07-independent-final-identity-ci-audit.json`；原生/进程详见 `2026-10-07-independent-process-pty-audit.json` 和 `2026-10-07-independent-native-checker-retest.json`。

## AC1–AC10

| AC | 状态 | 独立核对的证据与边界 |
| --- | --- | --- |
| AC1 | PASS | 18 格 save 实际显示选定结果行、掩码邮箱和查看账号建议；missing-fields 不显示邮箱/描述。18 格字段为 2 空格，显式多行续行为 4 空格。 |
| AC2 | PASS_BOUNDED | 共享五状态、Doctor typed-status binary 回归、计数/取消/空状态断言通过。实际 native current 是 unsaved warning，duplicate 是 error，Doctor fail/skip 保持区别；未触发 Auth 状态见 AC7。 |
| AC3 | PASS | 18 格正文、完整长描述及邮箱值均使用默认 fg7/bg0；no-color/dumb 所有可见非空字符无样式。共享 marker styling、完整迁移处置及重复标记审查支持剩余条款。 |
| AC4 | PASS_BOUNDED | 普通和前导连字符覆盖命令从实际屏幕提取后执行 36 次，均 exit 0；完整命令和覆盖后果文字可见。未知状态只有源码验证。 |
| AC5 | PASS | 18 格 normal/no-color/dumb 与 12 格 normal/force/dumb-force/no-color × 三种重定向通过；捕获的目标流使用普通状态词，只有 non-dumb force 产生 ANSI。no-color native 保留符号，dumb 使用普通词。 |
| AC6 | PASS_BOUNDED | 24 个补充 case 的两流和退出码符合既有分支，8 个 JSON 可解析；Doctor counts/typed report/human stdout 对应。结合 22 项展示及 8/10/14 项既有回归、DTO/return/service 源码边界；未声称全部交互已经运行。 |
| AC7 | UNVERIFIED | 实际 binary 覆盖已产出的成功、缺字段、重复、不支持、未登录、warning、空状态、取消及失败。partial/skip/overwrite/empty 采用 service/renderer 夹具；Unknown 无 native producer，完整 import 密码交互未运行。不能关闭全分支运行验收。 |
| AC8 | PASS | 931 个基线调用处置为迁移 494、共享覆盖 302、保留 135；当前 1091 个引用另存。语义 baseline/current 为 452/961。全部记录有处置、理由、责任人；72 个当前源码 hash 全部一致。 |
| AC9 | PASS | 40/80/120 列 × dark/light × normal/no-color/dumb 的 18 格实际 Windows console 完整；长字段、多行、缺字段、必要建议和复制命令保留。前景与背景不同，必要正文及字段值保留默认颜色。 |
| AC10 | PASS | 读取正式 `just ci` 原始 timing summary：16 个步骤均 OK，exit 0，总耗时 20:36.985；HEAD 与 82 个最终文件 hash 再核对一致。平台和运行边界独立记录。 |

## Verification

- Lint：PASS，正式 CI `Strict Clippy`；没有增加抑制或降低门槛。
- TypeCheck：PASS，正式 CI `Workspace Check`。
- Tests：PASS，正式 root Test 原始区间有 43 个 suite result，2101 passed、0 failed、19 ignored、33 filtered。ignored/filtered 不计作 passed。路径检查记录另含 22/22 展示 binary、Grok/Claude/Codex 8/10/14 回归和 13+5+2 路径安全测试。
- 完整 CI：PASS，16/16、exit 0。复核只读取原始回执和源码 hash，没有重复运行额外 gate。
- Windows 原生：PASS，18 格/162 case；36 次建议复制执行；12 格共享 mixed-stream。独立 JSON 回执保存具体退出码、颜色和来源。

两个 captured streams 均无合成 access/refresh sentinel。Doctor 报告中的路径均在补充进程 fixture 内，summary 为 7 passed、0 warnings、1 failed、4 skipped；typed status 与人类 stdout 前缀对应，JSON 和 human 均 exit 1、stderr 为空。

24-case 补充 fixture 的 Codex current 在 save 前后为 LoggedInUnsaved，Grok current 为 `logged_in:false`。这组补充回执不证明 Saved 或 Grok 已登录 current。其他状态按独立 binary 的实际断言记录。

ConPTY raw 含 harness 清屏、定位、窗口尺寸及主题转换。不能将整份 raw 的 ANSI 归于产品。共享 captured-stream bytes 与实际 console 颜色属性分别验证产品降级行为。

## Findings (not fixed)

- Unknown LoginState 没有 native producer；仅源码验证，不增加业务 producer。
- 完整加密密码交互为 NOT_RUN。`ccr codex auth import` 仍先扫描系统 Downloads；该路径不在已批准扩展范围，未为补证读取真实 Downloads。
- Linux/macOS、hosted、真实账号和在线 Provider/WebDAV 为 NOT_RUN。本地 Rust/Tauri/frontend/VSCode gate 不证明这些运行环境，也不证明安装或发布状态。
- 发现 Windows 隔离缺口前对真实日志的实际影响未审计；未读取真实日志、配置或账号补证。Windows 权限 helper 仍为 no-op。
- 既有部分 save 失败返回 `Ok`、UI service 中 Vue 3 与 React 19 的说明冲突保留；均在批准修复范围外。

首个 CI 失败、Windows cached metadata 失败、旧隔离失败均保留。审计读取曾使用默认 GBK而失败，改用显式 UTF-8 后完成；读取错误未当作产品失败。回退只涉及批准源码、测试、文档和规范，不操作真实账号或日志。交付权限未扩大。
