# CLI / Tauri 架构实施集成报告

本轮验证完成于 2026-09-29 UTC。最终源码冻结为 remaining-platform-tests-source-freeze.json（a31c8730，269 路径）。Windows 完整 just ci 退出 0（1078.351 秒）；Linux workspace quality、MSRV、审计、两类覆盖率、Tauri 和限定原生验收通过。专职全范围复核和最终证据矩阵核对完成，无新增代码 finding。

两份临时脚本的 5 条 lint 已获授权并修复，专职 checker 已恢复并完成本轮复核。历史 exporter 0xc0000005 的原因未查明；后续成功不能证明该历史故障的根因已解决。

Linux 验证期间发现的配置语义 no-op 秘密文件权限缺陷、任务局部 Python 启动入口、权限字面量和 ccr-store 路径测试问题均完成限定修复或环境补齐，原始失败与各源码版本的执行记录保留。旧运行仅归属于原冻结。

基线 commit 为 34d8a85e0e48b793733835e0304c8ed33940fcee，分支 dev。父任务与 11 个子任务保持 in_progress；没有提交、推送或归档。必需 macOS 验收及字面的修复 commit 条款仍未满足。

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

以下记录绑定最终源码冻结 a31c8730（269 路径）。Windows 和 Linux 成功记录均已完成源码与生成物前后核对。

| 验证 | 结果 | 证据（T10 research） |
| --- | --- | --- |
| Linux 根 quality | exit 0，224.101 秒；version/governance/fmt/strict Clippy/并行 workspace tests/check-workspace 完成 | remaining-linux-workspace-quality-after-path.json |
| Linux MSRV | 实际 Rust 1.95.0；workspace all-targets/all-features --locked check 通过 | remaining-linux-workspace-msrv-after-path.json |
| Linux 根覆盖率 | 74554/99931 = 74.61% ≥70%；core Gateway 393/409 = 96.09% ≥85% | remaining-linux-workspace-coverage-after-path.json |
| Linux 安全审计 | 固定 cargo-audit 0.22.2，exit 0 | remaining-linux-workspace-audit-after-path.json |
| Linux Tauri 完整门禁 | exit 0，216.576 秒；397 行为测试、2 guards、24+9+197 exporters、inventory 和治理检查通过 | remaining-linux-tauri-ci-after-path.json |
| Linux Tauri 覆盖率 | Gateway 654/703 = 93.03% ≥85%；整体 17964/34456 = 52.14%，该 recipe 没有整体阈值 | remaining-linux-coverage-tauri-after-path.json |
| Linux 原生构建与验收 | custom-protocol build exit 0；attempt7 六项通过，保存磁盘、二进制和资源链均独立核验 | remaining-native-acceptance-final.json |
| Windows 完整根 CI | exit 0，1078.351 秒；所有步骤通过，包含完整 Tauri、正式前端、docs 与 VS Code | remaining-windows-ci-after-path.json |
| 前端与扩展 | 前端 169 文件/904 测试、i18n 24；Lines 13081/17395 = 75.19% ≥70%；VS Code 51 测试通过 | 同上原始日志 |

成功记录的源码和生成类型前后无漂移。旧失败、旧成功及原冻结分别保留。

## 原生验证发现

Linux WSLg 原生 WebKitGTK 使用 debug 二进制及生产 custom-protocol 资源/CSP。启动环境指向合成 HOME、CCR/Claude/Codex/llmusage 目录，并使用私有 D-Bus；该环境重定向不构成操作系统文件沙箱。

attempt1 的挂载点和 attempt2 的按钮文本选择器已按真实源码修正。attempt3/4 到达真实明文确认和 CodeMirror。attempt4 证明 style-src-elem 违规来自确认弹窗 react-remove-scroll 无 nonce 的滚动锁样式；CodeMirror 的 nonce 和 computed style 正常。新修复由共享 utils 在 React 挂载前传入页面 nonce，未降低 CSP 策略。失败尝试全部保留。attempt5 的 WebDriver 按键及可交互定位失败未计保存通过；attempt6 使用 W3C actions、保存前内容精确断言和原生点击，磁盘仅更新 model，env/未知字段保持不变。保存后未捕获样式 CSP 违规，编辑器 DOM 已关闭，进程组清理完成。未检测动态 style 节点移除或 body 滚动锁还原；见 remaining-native-style-evidence-correction.json。attempt7 已在 a31c 最终源码上重新通过六项检查，root 独立磁盘读取与截图复核通过；证据见 remaining-native-acceptance-final.json。

## 验收边界

- 必需 macOS 进程与工作区矩阵未执行。本轮没有推送或创建 PR 触发远程工作流；本地 WSL 结果不等于 GitHub required status。
- 原生证据仅覆盖 Linux WSLg WebKitGTK、debug 二进制、生产 custom-protocol 资源/CSP、合成本地 Claude raw settings。Windows WebView2、macOS native、Profiles/Configs/Commands 原生交互、真实 SSH/远程 WSL、provider OAuth、交互式 TUI、物理输入和发布安装包未验证。
- 保存后仅确认未捕获 style CSP 违规。style DOM 移除与 body scroll-lock 还原未测量，不计入已通过范围。
- 历史 exporter 0xc0000005 原因未查明，失败前的可执行文件 SHA 缺失；后续通过与其他夹具修复不能补足该故障的根因证据。
- 部分 P1 原始失败分支仅保留源码证据，未分别执行旧基线；逐项边界见 T10 research/requirements-evidence.md/json。9 组 P1 的映射核对不代表全部闭环验收。
- fix_commit=null。本轮没有提交授权，任务状态保持 in_progress。

## 最终复核与留存

最终矩阵包含 39 项 AC、48 个测试组、164 个精确选择器、9 组 P1、86 条完成运行和 393 个仓库证据产物。原 41 条运行的 canonical 指纹、20 个原始反例产物及 7 份源码 manifest 保留。8 条当前门禁记录退出 0；该结果不改变上述未验证范围。

专职复核见 T10 research/remaining-dedicated-check.md/json、remaining-windows-final-independent-review.json 和 remaining-final-matrix-independent-review.json。T01 的既有作者范围明确保留；本轮 DC-02 权限修复由 root 完成非作者审查，DC-03 路径测试修复由 checker 完成非作者审查。未将所有审查称为完全作者独立。

269 个源码路径（267 文件、2 删除标记）与最终冻结一致，230 个生成类型与成功 Windows 生成事务的原始字节一致。4 份原 schema 保持既有字节；再次生成的 Linux schema 已保留独立副本，并仅移除原先不存在的工作区生成文件。最终 git diff --check 退出 0。

12 次任务 context validation 退出 0，24 份 JSONL 未变。4 条 dependency-governance.md 注入警告保留：33513 字节超过 32768 字节上限；checker 已完整读取。两个 Insights 任务分别保持 planning 与 in_progress。

最终交付校验：research/remaining-final-delivery-verification.json。当前状态以本报告、execution-ledger.json 和该交付记录为准。历史完整报告为 research/before-remaining-integration-report.md；上一轮原始报告为 continuation-integration-report.md；最终门禁前快照为 research/before-final-local-gates-integration-report.md。
