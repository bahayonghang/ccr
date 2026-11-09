# CLI / Tauri 架构实施集成报告

日期：2026-09-28。基线 commit：34d8a85e0e48b793733835e0304c8ed33940fcee，分支 dev。

已实施获批的 11 个子任务。最终根 CI 的完整 workspace 测试与 Release 构建通过；独立完整 Tauri 门禁和前端自动化验证通过。最终根 just ci 在既有 rustls 依赖的 Security Audit 阶段失败；本报告不授予完整 CI、发布就绪或原生交互验收。所有架构任务仍为 in_progress。本轮未提交、推送、归档，也未改变既有 Insights 任务生命周期。

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

## 实际验证

| 验证 | 结果 | 原始证据 |
| --- | --- | --- |
| 完整 just tauri-ci | exit 0；406 个行为测试、2 个 guard 通过，1 个 ignored helper；197 个 export 在专门生成阶段执行。包含 fmt/check/strict bin Clippy、bindings 和 inventory。 | research/root-acceptance-after-tui-tauri.json / .log |
| 最终根 just ci | exit 1；version sync/check、fmt/check、strict Clippy、workspace check、完整 Test、Release Build 全部通过；Security Audit 报既有 rustls 0.23.40 的 RUSTSEC-2026-0285，后续 Tauri/frontend 未在此 aggregate 继续执行。 | research/root-delivery-after-utc-ci.json / .log |
| 完整 workspace 测试 | 最终 aggregate 的 Test 阶段通过，耗时 3分41秒。保留默认并行；仅 ts-rs exports 由专用生成入口负责。包含 CLI 344、TUI 246、ccr-usage 65 项及其他 workspace targets/doc tests，不累计重复过滤计数。 | research/root-delivery-after-utc-ci.log |
| 完整 TUI 库 | 246 passed / 0 failed；包含全部 Grok 双语尺寸帧和共享 application 适配测试。 | ../09-28-architecture-contract-gates/research/root-tui-full-final.json / .log |
| 前端完整测试 | 167 files / 900 Vitest passed；i18n 24 passed，4517 leaf keys。 | research/root-final-frontend-tests.json / .log |
| 前端 coverage | 75.19%，高于原 70% 阈值；未降低阈值。 | research/root-final-coverage.json / .log |
| 主工作区 just frontend-check | exit 1；两份原有受保护脚本共 5 条 no-console。 | research/root-final-frontend.json / .log |
| 隔离授权源码副本的同一 frontend-check | exit 0；129 个现存前端/文档输入文件与主工作区字节一致，2 个预期 CSS 删除路径仍不存在。该结果不覆盖主工作区的正式失败。 | research/isolated-final-frontend.json / .log；research/isolated-frontend-input-parity.json |
| VS Code / docs | just vscode-ci、just docs-check 均 exit 0。 | research/root-final-vscode.json；research/root-final-docs.json |
| 完整 bindings guard | 24 CLI + 9 usage + 197 Tauri exports；调用前后 230 个 types 文件无新增、丢失或字节变化。最终完整 Tauri gate 中再次通过。 | ../09-28-architecture-contract-gates/research/gates-full-bindings-final.json |
| 聚合门禁 / 绑定事务 / 日期回归 | 独立治理 27 passed；绑定事务与日历 25 passed。Windows 实际 just fixture 注入 desktop 失败后 aggregate 非零。 | ../09-28-architecture-contract-gates/cross-check-gates.md |
| 验收映射 | 39 项 AC、33 组测试、9 组 P1；逐项关联旧反例、测试选择器、原始日志、源码与差异指纹。fix_commit 保持 null。 | ../09-28-architecture-contract-gates/research/requirements-evidence.json；research/p1-evidence-ledger.json |

测试数量不累计过滤重叠。0 匹配用例不作为通过证据。Codex Settings 新增后端组包含 1 项临时文件读取/投影和 2 项内存合并测试；没有宣称三项均为文件写回或原生界面端到端。

## 集成修正与保留的失败

1. clean 可选配置读取：仅 ConfigMissing 使用 skip_confirmation=false；损坏和不可读错误在删除前返回。强化后的 13 项真实 binary 测试通过，保留交互确认、dry-run、备份与坏文件字节。
2. list 分派：成功 fixture 显式建立合成 profiles；缺文件精确返回 ConfigMissing，None/Some(bytes) 同时比较目录和文件。12 项通过。
3. Tauri fixture：新增 CLI 合同入口先取既有 desktop 锁，再取 CLI 锁；Codex 四项使用 TestProcessEnv RAII 并隔离 CCR_LOCK_DIR；Grok 隐式锁目录依赖补入已有 guard。未将整个 suite 串行化。401/5 与 404/2 两轮失败均保留，最终完整 suite 为 406/0。
4. Grok TUI 旧断言：英文首字母与中文宽字符占位/换行不符合原始网格字串假设。现在只提取弹窗正文并规范化空白和大小写，保留两条警告及 2×4×6=48 帧矩阵。生产文案和布局未改。
5. Windows doctor 原 fixture 两次出现 grandchild.pid 未写。单项复跑和临时启动诊断下完整 CLI 344/0 均通过，原因未查明。诊断已全部撤除，fix.rs 恢复为 HEAD 原字节；没有修改生产超时或把通过重跑称为原因修复。
6. 较早一次 root aggregate 出现 commands.exe 启动 0xc0000005。单独 workspace 复核前后记录 binary SHA；失败 invocation 没有预采集 SHA，不能声称两次哈希已证明相同。早期 0xc0000135、OS1224、首次 6 文件 binding drift 的原始失败同样保留；原因未查明。

7. Usage UTC 夹具：两个旧聚合测试存储 UTC 时间且期望 UTC 日期，但使用默认 Local，在 America/Chicago 会移入前一天。仅显式设置测试的 ReportTimezone::Utc；SQL、生产默认和全部期望值不变。65 项通过，最终完整 workspace 再次通过。
8. 最新根 CI 的 Security Audit 发现既有 rustls 0.23.40 的 RUSTSEC-2026-0285；cargo audit 报告建议升级至 >=0.23.45。根 Cargo.lock 与基线逐字节一致，证明该锁定版本未由本轮引入。依赖升级超出批准的架构任务文件范围，本轮没有修改依赖或锁文件。证据：research/root-security-audit-baseline.json。完整测试与 Release 构建通过不覆盖该审计失败。

代码位置、原始红例、修正过程和非作者复核见 ../09-28-architecture-contract-gates/root-integration-review.md 与 cross-check-specs.md。

## 审查与验收边界

T01–T09、T11 已有各自独立定向审查。T10 专用 trellis-check 派发因 agent thread limit reached 阻断；两个 owner 交叉审查非本人切片，root 负责集成并将自己的补丁交给非作者复核。该证据不等于专用 checker 派发成功。

以下验收未完成：macOS；Linux Rust 1.98 pinned 全量 CI；原生 WebView/CSP；真实 SSH/WSL 账户；真实 provider OAuth；交互 TUI。已有 Linux Rust 1.95 MSRV 定向结果单独记录。额外 Tauri all-targets Clippy 的旧问题没有被正式 bin Clippy 结果覆盖。

主工作区两份受保护脚本未修改、移动或忽略。Insights 任务 09-24-home-insights-redesign 保持 planning，09-24-home-insights-frontend 保持 in_progress。12 组父子任务 context validate 已通过。最终 248 个源码/规范/产物文件与冻结快照一致，230 个 generated types 与成功 guard 的产物一致，git diff --check 通过。源码、保护文件、任务状态与前端副本对照见 research/final-delivery-verification.json。

## 交付状态

实施结果已写入工作区，验收状态以 execution-ledger.json 和原始日志为准。完整根 CI 因既有依赖安全审计未通过，主工作区正式 frontend-check 的 5 条原脚本 lint 错误也保持开放；发布验收和上述原生边界未授予通过。没有创建提交或归档任务。
