# 最终 UI 锁文件补充验收

复核日期：2026-09-30 UTC。角色：`trellis-check`。本轮只读，没有修改交付源码。

结论：UI 换行 finding 已关闭。实际最终锁文件保持原 LF 格式，从修改前字节只替换四条批准节点行即可精确重建。最终 LF 的冻结安装、审计和空白检查回执全部通过，输入哈希稳定。扩展追加授权与完整聚合仍未关闭。

本报告补充 [方案独立复核](new-advisories-independent-review.md)，保留原方案报告的时间快照。机器化验证见 [new-advisories-final-ui-verification.json](new-advisories-final-ui-verification.json)。

## Findings (fixed)

- 文件：`ccr-ui/bun.lock`。
- 问题：首次四节点补丁附带 1568 行 LF→CRLF 转换。实施者保留原始失败/检查凭证并恢复原 LF 字节，没有更改 Git 配置。
- 独立复核：实际文件 SHA-256 为 `c4301a8d1909ac7db2f92930e5d91172195d3ad51c84b4e163c16909a2528025`；CRLF 为 0，LF 为 1568，解析节点为 743。
- 从保存的修改前文件 `new-advisories-bun-before.lock`（SHA `87a80e33cdbdaf8ed72516419ece312de1134d94a3e0495d92139580ca65ae07`）逐条应用四个批准的版本/integrity 替换，所得 Buffer 与当前文件逐字节相等。不存在其他字节变化。
- 把当前 LF 文件仅转换为 CRLF 后，得到首次完整前端检查所用 SHA `f4d25c2659543ffbc3c439a92813f0d67d9c44c2bc3d881d7e3710bb52adbf1e`。两版本只有换行差异。

实施者修正记录：[new-advisories-lf-correction.json](../../09-29-harness-frontend-security/research/new-advisories-lf-correction.json)。

## 节点与安全范围

| 精确节点 | 本轮增量 | 相对 HEAD 的最终值 |
| --- | --- | --- |
| brace-expansion | 1.1.18 → 1.1.21 | 1.1.18 → 1.1.21 |
| fast-uri | 3.1.7 → 3.1.8 | 3.1.5 → 3.1.8 |
| @typescript-eslint/typescript-estree/minimatch/brace-expansion | 5.0.9 → 5.0.12 | 5.0.9 → 5.0.12 |
| glob/minimatch/brace-expansion | 5.0.9 → 5.0.12 | 5.0.9 → 5.0.12 |

本轮增量为 **4 个节点**。相对 HEAD 累计为 **7 个节点**，包括保留的 colord 2.9.3→2.9.4、js-yaml 4.3.1→4.3.2、undici 8.10.0→8.10.2。fast-uri 只计一个累计节点。743 个解析节点的名称和顺序不变；每个差异仅涉及版本/integrity，依赖 metadata 不变。

重新对实际最终 UI 解析图检查原 18 个 GHSA 加新 4 个 GHSA：22 个唯一 ID、81 个相关节点×范围组合，匹配数为 0。公告完整原文与 tarball integrity 仍引用方案复核保存的快照。最终 LF 审计另有实际退出 0 的当前回执。

2026-09-30 03:51:31 UTC 的独立快照确认：UI manifest、docs manifest/锁、扩展 manifest/锁均与本轮诊断前哈希相同。扩展锁 SHA 为 `846b84a475e48c9e7f335a87121a1894dc64548f41449d2dd54a5b932174fdd4`；brace-expansion 仍为 5.0.9、fast-uri 仍为 3.1.7，新增目标待批准。

## Verification

以下六份回执的 stdout/stderr SHA 均已重新核对，执行前后输入哈希相等。完整命令、回执文件 SHA 和原始日志摘要记录在独立验证 JSON 中。

| 检查 | 输入锁版本 | 结果 |
| --- | --- | --- |
| `bun install --frozen-lockfile` | 最终 LF，`c4301a8d…` | PASS；654 installs / 723 packages，no changes；锁字节稳定 |
| `just frontend-audit` | 最终 LF，`c4301a8d…` | PASS；0 advisory，0/0 active exceptions |
| 定向 `git diff --check` | 最终 LF，`c4301a8d…` | PASS；退出 0；保留 Git 未来自动换行提示 |
| `just frontend-check` | 此前 CRLF，`f4d25c26…` | PASS；类型、lint、i18n、169 文件/904 测试、构建及 docs 检查 |
| `just frontend-coverage` | 此前 CRLF，`f4d25c26…` | PASS；169 文件/904 测试；Lines 75.21% |
| audit policy smoke | 此前 CRLF，`f4d25c26…` | PASS；3/3 |

最终 LF 三份原始回执位于 T08 的 `research/evidence/new-advisories-lf-{frozen-install,frontend-audit,diff-check}.json`。其中审计于 2026-09-30 03:48:31 UTC 开始，03:48:36 UTC 完成。

独立读取本次 `ccr-ui/coverage/coverage-final.json`，通过 Istanbul 汇总重新得到 Lines **13083/17395，75.21%**；Statements **14655/20092，72.93%**；Functions 4159/5785，71.89%；Branches 8027/13500，59.45%。当前门槛仍为 Lines 70%。覆盖率文件 SHA 保存在独立验证回执中。

这些完整前端与覆盖率结果绑定此前 CRLF 输入。最终 LF 的完整前端执行仍需由主会话的最终聚合提供回执，不能把换行等价证明写成已经重新执行。

## Findings (not fixed)

- **完整聚合 retry1 失败。** [final-ci-retry1.json](final-ci-retry1.json) 于 2026-09-30 03:49:38 UTC 启动，73.79 秒后退出 1。前 7 项通过，第 8 项 Test 在 `legacy_registry::legacy_registry_fields_load_without_becoming_clean_write_routing_truth` 失败；managers 套件为 16 pass / 1 fail。2509 个交付源文件无漂移。日志 SHA 已核对。
- retry1 尚未执行 frontend-check、frontend-audit 或 frontend-coverage，不能提供最终 LF 的完整前端门禁成功结果。该 Rust 失败的根因不在本次只读 UI 复核范围内，本报告不推断原因、不更改源码。
- 扩展 brace-expansion 5.0.12 和 fast-uri 3.1.8 的新增目标仍待批准；当前扩展独立审计的失败不能被 UI audit 0 关闭。
- 九条 Tauri warning、原生客户端、macOS、hosted 与固定运行时证据继续独立保留。UI 换行 finding 的关闭不改变这些项目的状态。

本轮没有重跑完整测试，没有安装依赖，没有更改产品文件、阈值、ignore、Git 或客户端设置。独立写入限于本补充报告和研究回执。
