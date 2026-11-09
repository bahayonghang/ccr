# CLI / Tauri 架构实施集成报告

基线 commit：34d8a85e0e48b793733835e0304c8ed33940fcee，分支 dev。原始日志使用 UTC 时间；本次续作跨越 2026-09-28 至 2026-09-29 UTC。

11 个子任务的实施已写入工作区。本轮完成依赖审计修复、Tauri all-targets lint 修复、Settings 无损保存与双语回归、Doctor 超时夹具修正。完整工作区测试、Release、安全审计及独立完整 Tauri 门禁通过。最新根 just ci 仍为 exit 1：绑定导出程序启动时返回 0xc0000005，原因未查明。后续单次绑定检查和完整 Tauri 检查通过，不能改写该次 aggregate 的失败。

主工作区正式 frontend-check 仍有两份受保护脚本的 5 条 no-console。必需的 macOS 进程清理矩阵及原生 WebView/CSP 验收未完成。所有架构父子任务保持 in_progress；未提交、推送或归档。

## 实施后的职责

| 任务 | 实施结果与责任边界 |
| --- | --- |
| T01 配置仓储 | ccr-config repository 统一资源身份、锁内最新读取、partial edit、CAS 与未知字段保留；查询不隐式初始化或修复文件。 |
| T02 Profile application | Claude/Codex/Grok apply 经同一 application 用例到 CLI、TUI、Tauri；统一预检、已提交警告和条件补偿。批准的 update/rename 迁移仅覆盖 Claude/Codex；Grok 保留既有具名部分状态。 |
| T03 Tauri 配置适配 | 通用 /configs 明确为 Claude；真实 handler 经 repository/application，保留启用策略补偿和草稿版本检查；registry 生成客户端、ACL 和命令清单。 |
| T04 CLI 诊断 | service 返回具名、只读报告，分别表达运行时与 profile 问题；binary 持有退出码和输出职责。 |
| T05 持久化与备份 | 唯一备份命名、秘密文件发布前权限、叶锁/CAS 与内存 preimage 条件补偿。没有承诺操作系统级多文件事务。 |
| T06 Usage 生命周期 | 真正执行 owner 持有任务 admission；取消请求与终态分离；deadline、流上限和清理失败进入有界状态机。Usage SQL 仍由 ccr-usage 独占。 |
| T07 Commands 前端 | shell 持有任务状态；页面重入恢复快照，事件幂等合并，终态去重。历史为每 job 至多一次写入尝试，失败可见。 |
| T08 Settings 无损编辑 | 保留原快照，仅提交 dirty leaves；托管锁、未知枚举、通知数组及 raw editor 的确认/CAS/草稿能力均有分层回归。 |
| T09 查询与编辑会话 | 区分 pending、失败、unsupported 与 stale 数据；环境身份和世代约束 typed/raw 草稿；后端捕获单一环境目标；Configs 翻译订阅修复。 |
| T10 门禁与规范 | root aggregate 包含独立 Tauri 门禁；生成目录事务保留原字节；实际故障 fixture 验证失败传播；规范导航、owner 和命令计数收敛。 |
| T11 控制命令与 OAuth | 控制入口到达实际执行 owner；permit 随任务存活；listener/bind/pending 成功后发布 URL；取消与超时有界，单一终态不伪造登录成功。 |

Tauri llmusage_adapter 只负责已安装 CLI 同步、NDJSON、DTO 和错误映射，没有链接上游 llmusage Rust crate。没有重写整个 crate 图或所有 legacy IPC。

## 本轮验证

| 验证 | 当前结果 | 原始证据 |
| --- | --- | --- |
| 根 just ci | exit 1；version sync/check、fmt/check、strict Clippy、workspace check、完整 workspace Test、Release、安全审计、CI Governance 均通过。Tauri 407 行为测试和 2 个 guard 通过；CLI exporter 以 0xc0000005 退出，未观察到该失败进程的 running 测试计数 banner。该次 aggregate 未执行 frontend。 | research/root-continuation-final-ci.json / .log |
| 独立 just tauri-ci | exit 0，230.503 秒；407 行为测试、2 个 guard、1 个 ignored helper。专门生成阶段执行 24 CLI、9 usage、197 Tauri exports，另含 inventory。 | ../09-28-architecture-contract-gates/research/continuation-bindings-startup-tauri-ci.json / .log |
| 单次 just tauri-bindings-check | exit 0，60.203 秒；230 个生成类型路径及字节不变。 | ../09-28-architecture-contract-gates/research/continuation-bindings-startup-bindings-check.json / .log |
| Windows 最低 Rust 版本 | rustc 1.95.0；Tauri --locked --all-targets --all-features check，exit 0。不覆盖原生运行时。 | ../09-28-architecture-contract-gates/research/continuation-windows-msrv-tauri.json / .log |
| Linux 最低 Rust 版本 | rustc 1.95.0；workspace --locked --all-targets --all-features check，exit 0。 | ../09-28-architecture-contract-gates/research/continuation-linux-msrv-workspace.json / .log |
| Linux Rust 1.98 定向 | ccr-codex strict all-targets/all-features Clippy、9 项 core process、3 项 Doctor 通过。旧 process metadata 的 source_sha 属于 codex_process_service.rs，不能当作 core 源码命令开始指纹。 | ../09-28-architecture-contract-gates/research/continuation-linux-{codex-clippy-online,process-online,doctor}.json / .log |
| Linux Tauri process smoke | Rust 1.98：Tauri gateway 8 项、core gateway 9 项通过。该次 Tauri 编译有一条 Windows 测试导入告警，随后仅修正 cfg 条件；修正后检查单独记录。完整 Linux lane 尚未运行。 | ../09-28-architecture-contract-gates/research/continuation-linux-process-smoke-summary.json |
| 最后 cfg 修正补验 | Linux Rust 1.98 Tauri strict Clippy 通过；process aggregate 8（完整 selector 5）、OAuth aggregate 20（完整 selector 18）通过。Windows Rust 1.95 all-targets/all-features check、Rust 1.98 OAuth 1 项和 fmt 通过。两次错误选择器 exit 0 / 0 用例排除行为验收。 | ../09-28-architecture-contract-gates/research/continuation-platform-final-checks.json；continuation-oauth-selector-correction.json |
| 两锁审计与 Tauri 定向 | 两锁审计通过，原有 Tauri 9 条告警保留；62 项后端定向测试通过，包含 model-only 临时文件保存与重读。 | ../09-28-architecture-contract-gates/research/continuation-security-check.json；continuation-rust-check.json |
| 前端完整测试 | 168 files / 901 Vitest passed；i18n 24 passed / 4523 leaf keys。 | research/root-continuation-frontend-tests.json / .log |
| 前端覆盖率 | Lines 75.19% = 13079/17393，高于原 70% 阈值；Statements 72.92%，Branches 59.45%，Functions 71.88%。 | research/root-continuation-coverage.json / .log |
| 前端生产构建 | bun run build 实际 exit 0。采集脚本完成 metadata 写入后，向 GBK 控制台输出 Unicode 尾行失败，wrapper exit 1 单列；未重复构建。既有大 chunk 提示保留。 | research/root-continuation-ui-build.json / .log |
| 主工作区 just frontend-check | exit 1；仅两份受保护脚本共 5 条 no-console。未修改、移动、忽略原脚本。 | research/root-continuation-frontend.json / .log |
| 实际 Web 行为 | 官方 Tauri mock + 真实 React Settings：model-only 一次 patch，通知数组及未知枚举保留；确认后一次 raw read，CodeMirror 可见；纯 Web runtime unavailable 显示错误和重试。pageerror 0，未知 mock 命令 0。没有截图证据。 | ../09-28-architecture-contract-gates/research/continuation-web-review.md / .json |

测试计数不累加重叠过滤。零匹配用例不作为通过。Web 检查使用合成 IPC 和路径，不覆盖 native IPC、真实配置或生产 CSP。浏览器页面、Tabbit 任务与 Vite 已关闭。

较早的完整 TUI 246、VS Code、docs 和隔离 frontend-check 成功保留为历史证据，没有将旧源码运行改称本轮重新执行。参见 research/integration-report-before-continuation.md。

## 本轮修正与失败记录

1. 两个 Cargo.lock 仅升级 rustls 0.23.40 → 0.23.45、rustls-webpki 0.103.13 → 0.103.14 的版本和 checksum；各锁包数及依赖列表保持。没有新增 audit ignore 或改变项目 features/MSRV 声明。锁中漏洞包与当前激活依赖图的结论分开记录。
2. Tauri 4 项旧 all-targets Clippy 问题已修正。Codex Settings 通过显式 path 的共享生产持久化 helper 执行 model-only 保存和重读，保留通知数组及所有非 model TOML 语义。测试不覆盖 State、IPC admission、缓存失效、注释顺序或原生端到端。
3. Settings 补齐 Claude subtitle、Codex Save 与四档 reasoning 中英翻译；未知合法值保持，真实 translator 和已挂载 SettingsView 有双语回归。
4. Doctor 树清理测试原先混合夹具启动时间与被测 action timeout。受控红例后，测试先确认父/孙进程 ready 且存活，再执行同一生产等待/清理路径；新增生产入口在夹具未 ready 时仍按 deadline 返回的测试。没有扩大超时或削弱清理断言。Windows CLI 345 单元、12 integration、1 doctest 及 Linux 3 项 Doctor 通过，另 1 doctest ignored。原宿主迟滞具体原因未查明。
5. Linux smoke 编译发现 Mutex 仅用于 Windows 测试却在全部测试目标导入。仅将该 import 的 cfg(test) 收紧为 cfg(all(test, windows))，生产路径及测试逻辑保持；独立逐字节审查和上述平台补验通过。原 8 项通过及 compiler warning 记录保留。
6. 最新完整根 CI 的 CLI exporter 启动访问冲突保留；失败 invocation 没有事先采集 exe SHA。失败后 --list、单次 bindings 和完整 Tauri 成功，后三次观测的 exe SHA 相同。不能据此断言失败时二进制相同或原因已修复。限定时间内没有匹配的 Application Error/1000；事件缺失不否定崩溃。
7. 既有失败快照保留，包括此前 workspace doctor PID、测试 exe 启动 0xc0000005、早期 0xc0000135、OS1224、生成物漂移以及修正前的 Tauri fixture、TUI 帧和 Usage UTC 断言。上一轮审计失败由本轮锁修复和审计通过接续，没有删除旧失败。

非作者审查见 T10 research/continuation-security-check、continuation-rust-check、continuation-frontend-check、continuation-doctor-check。绑定启动调查见 continuation-bindings-startup-report。

完整根 CI、完整 Tauri 与先前 Linux smoke 均绑定原 continuation-final-source-freeze.json。最后单行 cfg 修正绑定 continuation-platform-final-source-freeze.json；旧运行不改归新清单。新清单继承的较早 changes_since_previous 元数据不完整，精确来源链补充见 T10 research/continuation-platform-freeze-provenance.json，冻结清单原字节保持。

## 验收与文件保护

验收矩阵校验通过：39 项 AC、42 组测试、9 组 P1、41 条本轮运行记录及 227 个已核验 SHA 的证据产物。该结果证明映射与来源一致，不授予未通过门禁或未执行平台验收。详见 T10 research/requirements-evidence.json、evidence-mapping-validation.json 与父 research/p1-evidence-ledger.json。fix_commit 保持 null。源码定位指纹不证明历史日志使用当前源码；保留旧指纹、运行时间和来源限制。

T01–T09、T11 有独立定向审查。T10 专用 trellis-check 在已有代理结束后再次派发，仍受 agent thread limit 限制，当前使用明确范围的非作者交叉审查；没有将其称为专用 checker 派发成功。

263 个冻结路径由 261 个现存文件与 2 个明确删除标记组成，null 不作为缺失错误。独立完整 Tauri 复验前后与冻结清单一致，230 个 generated types 同样一致。最后平台测试完成后已核对当前冻结清单，263 路径与 230 类型均无漂移，git diff --check 为 0；详见 research/final-delivery-verification.json。两份保护脚本与 Insights 状态不变；12 组父子 task context validate 已通过。最终非作者产物复核见 T10 research/continuation-evidence-check.md / .json。

## 剩余工作

- 正式主工作区 frontend-check：两份受保护脚本 5 条 lint，最小修改授权问题仍待用户回答。
- T06 必需 macOS 进程树清理矩阵：尚无可用设备或 runner 信息，未验证，未改为可选。
- 原生 WebView/CSP：启动隔离尚不完整，日志、数据库、pending 和 WSL 探测仍可能访问真实 home，未启动桌面程序。
- 完整 pinned Linux 1.98 lane：未执行；最低版本检查和定向进程测试不能代替。
- 根 aggregate 启动访问冲突：原因未查明，后续成功保留为独立复验证据。

真实 provider OAuth、SSH 登录和交互 TUI 是明确未测试扩展，没有要求用户登录。当前状态不授予完整 CI、完整跨平台验收或发布就绪。
