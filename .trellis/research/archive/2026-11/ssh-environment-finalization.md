# SSH 执行环境本地实施记录归档

日期：2026-11-16（执行时本地日期）。范围：[实施规格 #205](https://github.com/bahayonghang/ccr/issues/205) 的本轮本地验收分类与归档；GitHub Issues 继续作为任务源，mini-trellis 不驱动任务。

本轮没有新增产品、测试、依赖、执行脚本或门禁改动，只整理归档和 journal。S1–S4 的交付与协调者核销已完成；**父任务验收未完成，#205 保持 OPEN，#206–#214 保持 CLOSED**。最终提交、工作区、GitHub 状态及归档终态由协调者在本地提交后核验，以 `outputs/ssh-checks/205/finalization/final-integration-receipt.json` 和 `archive-receipt.json` 为准；本文不提前声明这些收据已生成。

## 代码交付与验证基线

源码与既有测试/CI HEAD：`173c9f0b6d75cff5179a3942c90d3b4f586ef0fa`。以下是本轮开始前已经完成的 A/B/C，不是本轮新增提交：

| 既有提交 | 交付 |
| --- | --- |
| `de174ad8` | 上传祖先链接检查纳入原取消与 10 秒预算；生产入口内存 SSH/SFTP 回归。 |
| `d776bad8` | 安装取消清理期间保留 Busy/终态；状态观察不授予新执行权限，清理中的旧 owner 不被重连替换。 |
| `173c9f0b` | 仅新增在途 helper 失权重连组合测试，验证不借用或撤销新会话、不下载/发布缓存；没有新增产品修复。 |

协调者和 S4 已核对源码 manifest、原始日志哈希及退出标记。本轮明确**复用**同源码 HEAD 的以下结果，没有重新运行整套门禁或 CI；文档提交 HEAD 另见最终收据：

- 根 6 项：`just version-check`、`just fmt-check`、`just lint-strict`、`just test`、`just audit`、`just ui-check`，全部 exit 0。
- desktop 5 项：`just desktop-fmt-check`、`just desktop-lint`、`just desktop-test`、`just desktop-test-ui`、`just desktop-audit`，全部 exit 0。
- 完整 `just ci`：exit 0，1245.8 秒，17 阶段通过。来源为 [旧 CI 结果](../../../../outputs/ssh-checks/205/next-session/final-ci/results.json)；复用依据及原始日志路径见 [本轮复用收据](../../../../outputs/ssh-checks/205/finalization/reuse-receipt.json)。

旧 B 首次 audit 因 RustSec 拉取 TLS EOF 失败、同 HEAD 窄重试通过的事实和原日志均保留，不改写为首次全绿。本轮最终 CI 修复轮为 0/1；与更早轮次的修复预算分开记账。默认并行、独立 binding 导出、预算/容量/阈值及既有审计忽略未改变。

## 本轮逐组结论与证据层级

| 阶段 | 本轮实际结果 | 证据 |
| --- | --- | --- |
| S1 原生窗口 | 9 项 `blocked_tool`；其中同主机重连、远端安装、每主机连接另有 `blocked_fixture`。初始化后首次 `list_apps` 报 native pipe `os error 2`。应用启动、窗口输入、新截图和截图查看均为 0。 | [S1 验收](../../../../outputs/ssh-checks/205/finalization/S1/run-20261116-222816/acceptance.md) |
| S2 SSH/凭据/文件/进程 | 7 项及子项全部 `blocked_resource`：三类认证/keyring、信任、HOME、SFTP、两主机、断线重连、真实进程树。没有授权资产，未扫描或连接。 | [S2 验收](../../../../outputs/ssh-checks/205/finalization/S2/run-20261116-223151/acceptance.md) |
| S3 用量/快照/WebDAV | 4 组、31 子项全部 `blocked_resource`：远端 CLI/Python/schema/流生命周期、WAL/平台/日期、WebDAV、私有快照。未运行本机 CLI 冒充远端、未新建 WAL/cache。 | [S3 验收](../../../../outputs/ssh-checks/205/finalization/S3/run-20261116-223525/acceptance.md) |
| S4 单一只读审查 | clean review，零产品发现、零 tracked 修改、零新门禁；覆盖与复用依据成立。 | [S4 审查](../../../../outputs/ssh-checks/205/finalization/S4/run-20261116-224008/review.md) |

已有 fixture/mock、本机 SQLite 和 fake DAV 仅为本地回归证据；A 的内存加密 SSH/SFTP 使用 `tokio::io::duplex`，不是 TCP、外部主机、真实文件系统或系统凭据库验收。旧截图不绑定当前源码，本轮没有复用为新截图。

S1 只做静态隔离路径审计，没有完成动态准备或启动。普通 fixture 不自动发现本机 Insights，但两个 catalog 参数不足以证明所有页面已经完成动态隔离；普通启动也没有可连接 SSH/远端安装注入入口。不把这些验收设施缺口写成产品失败或静态审计通过即原生通过。

## 自主决策、失败事实与恢复

- 原生管道失败后不重复同类探测、不修改系统设置、不自建 UIA/管道替代控制器，也不扩产品 fixture 只为截成功图。
- 缺资源逐项阻断，不读取保存主机、agent、keyring、个人配置或真实私钥推定授权。复用充分覆盖的旧协议/fixture 证据，不重复 mock，不为制造提交寻找缺陷。
- [执行环境契约](../../../spec/ccr-app/backend/execution-environment.md) 保持准确：SQL/backup/校验由 `ccr-usage` 持有，协议/加密/冲突由 `ccr-sync` 持有；session authority、状态保留、失败终态和既有范围限制不扩展。
- 本轮外部连接、读写、WebDAV 请求、凭据变更、远端进程、快照路径、缓存及外部对象均为 0/空清单。前像和恢复验证为不适用，**不是恢复通过**；没有本轮未确认恢复对象，历史外部清理未验证项继续保留。
- 初次恢复 wrapper 的 GBK/UTF-8 解码失败及后续独立重跑、S4 日志匹配器大小写误判及窄更正均保留原记录，属于证据工具问题，不是产品/门禁失败或 CI 修复轮。
- T3 重启取消原 S5；协调者先确认无主题/journal/提交/Git 锁/相关在途命令，再恢复未写入的 S5，未重做 S1–S4。见 [恢复检查](../../../../outputs/ssh-checks/205/finalization/restart-20261116-2243.json)。

## 未通过与后续入口

原生交互/浅深截图、真实 SSH/OS agent/keyring、SFTP 权限与原子替换、两主机顺序切换、远端 CLI/Python/schema/WAL/日期对照、外部 WebDAV、真实 nonce 进程树退出、精确私有快照清理及跨 OS 验收均未完成。获得明确授权的独立资产后，沿用 [真实资源清单](../../../../outputs/ssh-checks/205/next-session/real-host-checklist.md)；先记录前像和精确对象，逐对象验证恢复。

真实远端树需要本轮 nonce 绑定的父子进程身份与退出证据；取消标志、本机进程数或 channel 关闭不能证明退出。旧会话私有对象仅可由授权管理通道依据精确路径、原 session UUID/所有权处理，不借新 session 自动补删、不通配清理；无法确认继续保留 CleanupFailed/遗留。

历史 `303c898e`、`12e741a2`、`5d908cb6` 的个人 author/committer 元数据发布阻断仍在，只记录合规布尔，不复制真实值、不改写历史。因此不能宣称全部公开发布条件满足。

本轮归档定位是“本轮本地实施记录”，不是父任务完成。未推送、未建 PR/合并、未移动 dev/main、未改历史或全局 Git/主机设置。`outputs/` 和截图保持原忽略策略，原始证据只在本机保存；新 checkout 不包含这些忽略输出。协调者的最终收据负责落定“本轮本地实施已归档，父任务验收未完成”或有失败时的实际状态。
