# CLI / Tauri 架构实施集成报告

当前检查点：2026-09-29 UTC。基线 commit 为 34d8a85e0e48b793733835e0304c8ed33940fcee，分支 dev。父任务与 11 个子任务保持 in_progress；没有提交、推送或归档。

Windows 完整 just ci 已在 d8ee 冻结通过，Linux Tauri、Gateway 覆盖率及原生 attempt6 同样通过。两份临时脚本的 5 条 lint 已获授权并修复，专用 checker 已恢复。历史 exporter 0xc0000005 的原因未查明；后续成功不改变该结论。

后续 Linux 根覆盖率发现配置语义 no-op 未保障秘密文件权限，已修复并通过两包定向测试与严格检查。完整 Linux quality 依次发现缺少 python 启动入口、权限字面量 lint 和 ccr-store 会话路径测试失败；三项均已完成限定修复并独立复核。最新源码冻结为 remaining-platform-tests-source-freeze.json（a31c8730，269 路径）。该冻结的 Linux 完整 quality、MSRV、两类覆盖率、审计、Tauri 和原生 attempt7 已通过；Windows 完整 CI 正在运行。旧成功仍仅归属于原冻结。

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

## 本轮已完成验证

以下记录绑定最终源码冻结 a31c8730（269 路径）；Windows 当前运行尚未计入通过。

| 验证 | 结果 | 证据（T10 research） |
| --- | --- | --- |
| Linux 根 quality | exit 0，224.101 秒；version/governance/fmt/strict Clippy/并行 workspace tests/check-workspace 完成 | remaining-linux-workspace-quality-after-path.json |
| Linux MSRV | 实际 Rust 1.95.0；workspace all-targets/all-features --locked check 通过 | remaining-linux-workspace-msrv-after-path.json |
| Linux 根覆盖率 | 74554/99931 = 74.61% ≥70%；core Gateway 393/409 = 96.09% ≥85% | remaining-linux-workspace-coverage-after-path.json |
| Linux 安全审计 | 固定 cargo-audit 0.22.2，exit 0 | remaining-linux-workspace-audit-after-path.json |
| Linux Tauri 完整门禁 | exit 0，216.576 秒；397 行为测试、2 guards、24+9+197 exporters、inventory 和治理检查通过 | remaining-linux-tauri-ci-after-path.json |
| Linux Tauri 覆盖率 | Gateway 654/703 = 93.03% ≥85%；整体 17964/34456 = 52.14%，该 recipe 没有整体阈值 | remaining-linux-coverage-tauri-after-path.json |
| Linux 原生构建与验收 | custom-protocol build exit 0；attempt7 六项通过，保存磁盘、二进制和资源链均独立核验 | remaining-native-acceptance-final.json |
| Windows 完整根 CI | 当前运行中；旧 d8ee 版本的完整 CI 已通过，不能替代当前源码 | remaining-windows-ci-after-path.json |

成功记录的源码和生成类型前后无漂移。旧失败、旧成功及原冻结分别保留。

## 原生验证发现

Linux WSLg 原生 WebKitGTK 使用 debug 二进制及生产 custom-protocol 资源/CSP。启动环境指向合成 HOME、CCR/Claude/Codex/llmusage 目录，并使用私有 D-Bus；该环境重定向不构成操作系统文件沙箱。

attempt1 的挂载点和 attempt2 的按钮文本选择器已按真实源码修正。attempt3/4 到达真实明文确认和 CodeMirror。attempt4 证明 style-src-elem 违规来自确认弹窗 react-remove-scroll 无 nonce 的滚动锁样式；CodeMirror 的 nonce 和 computed style 正常。新修复由共享 utils 在 React 挂载前传入页面 nonce，未降低 CSP 策略。失败尝试全部保留。attempt5 的 WebDriver 按键及可交互定位失败未计保存通过；attempt6 使用 W3C actions、保存前内容精确断言和原生点击，磁盘仅更新 model，env/未知字段保持不变。保存后未捕获样式 CSP 违规，编辑器 DOM 已关闭，进程组清理完成。未检测动态 style 节点移除或 body 滚动锁还原；见 remaining-native-style-evidence-correction.json。attempt7 已在 a31c 最终源码上重新通过六项检查，root 独立磁盘读取与截图复核通过；证据见 remaining-native-acceptance-final.json。

## 尚未完成

- Windows 完整 CI 仍在运行；完成后更新最终矩阵、P1 ledger 和专职终审。
- 必需 macOS 进程与工作区矩阵未执行；本轮没有推送或创建 PR 触发远程工作流。
- 原生证据仅覆盖 Linux WSLg WebKitGTK 的本地 Claude raw settings；Windows WebView2、macOS native、Profiles/Commands 原生交互、真实 SSH/OAuth、交互式 TUI 和发布安装包未验证。
- 历史 exporter 0xc0000005 原因未查明，后续通过不改变该结论。
- 12 任务 context validation 返回 0，24 JSONL 与 Insights 状态未变。保留 dependency-governance.md 的 33513 > 32768 字节注入警告；checker 已独立完整读取。

历史完整报告：research/before-remaining-integration-report.md。上一轮原始报告：continuation-integration-report.md。历史失败、41 条旧运行和既有冻结均保留。当前为中间检查点，完成后更新。
