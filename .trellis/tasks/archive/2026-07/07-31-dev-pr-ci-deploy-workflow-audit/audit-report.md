# CCR 开发、PR、CI、合并与发布流程审计

- 审计范围：本地开发、提交前验证、PR/评审、required CI、分支保护与合并、Release、分发/部署交接
- 仓库：`bahayonghang/ccr`，审计分支：`dev`
- 远端刷新：2026-07-31 20:03（Asia/Shanghai；GitHub 时间以 UTC 输出）
- 证据边界：只读仓库、Git、GitHub API/Actions 历史数据和任务目录研究文件；本报告不实施任何建议

## 阅读口径

报告中的标签含义如下：

| 标签 | 含义 |
| --- | --- |
| **事实** | 可由当前文件、命令输出、GitHub API 或已链接的历史 run/PR 直接证明 |
| **推断** | 由多个事实支持，但没有直接因果事件证明；同时列出替代解释 |
| **证据缺口** | 当前无法取得、不能据此下结论的数据 |
| **建议** | 基于事实、项目规模和官方资料选择的可执行方案，不代表当前行为 |

## 执行摘要

当前 CCR 已有一套质量覆盖较完整的门禁：`main`/`dev` 均使用四个稳定 required context，workflow 内部做 surface relevance 判断，required aggregator 总是创建，Actions 引用固定到完整 SHA，Rust、Tauri、Vue/Docs、VS Code 和依赖安全均有检查。这些是应保留的基础，不应通过减少检查来换取速度。

最优先的改进不是增加更多检查，而是修复交接和等待：

1. **P0 反馈缺口**：`dev` 允许管理员绕过保护，而 Root/Tauri/VS Code 只监听 PR；2026-07-31 最新一次 `dev` push（`a09cf340`）只产生 Frontend CI。管理员直推后的完整状态无法作为一个稳定合同获得。
2. **P0 过期运行浪费**：没有 workflow `concurrency` 或 `timeout-minutes`。PR #42 在 7.75 小时内产生 72 个提交，四个 workflow 各运行 7 次，旧运行未取消。
3. **P0 依赖突发**：当前 23 个开放 PR 全为 Dependabot、全部指向 `main`，12 个为 `BLOCKED`；六个 weekly 配置没有 grouping、limit、错峰或 target branch。
4. **P0 发布完整性**：Release 的各平台 job 直接将资产写入公开 Release（`draft: false`/`releaseDraft: false`），没有“全部产物完成后再公开”的汇聚发布点；现有 `release` environment 没有 job 使用，GitHub deployments 为空。
5. **P1 本地反馈合同**：根 `just ci` 串行执行 12 阶段，且 `version-sync`、`fmt` 会修改工作树；前端四个子检查分别执行 `bun install --frozen-lockfile`。需要显式的 verification-only 入口和一次安装复用。
6. **P1 观测与成本**：136 个 completed run 样本的端到端时间混合了排队和执行；Tauri 示例中 macOS queue 1,485s、Linux queue 733s，而工具安装也占运行时间。没有缓存命中率，不能承诺精确节省百分比。
7. **P1 发布前置**：tag 触发没有 tag/版本/来源提交/required CI preflight；当前代码为 7.1.1、`main` 为 7.0.0、最新公开 Release 为 v6.5.0，CHANGELOG 正式 section 仍到 4.3.0。
8. **P2 规模化事项**：merge queue、自托管 runner、更大 runner、多人强制审批、签名/公证、attestation/SBOM、updater 和多渠道发布均需要新的流量、权限、证书或安全决策，当前不应作为立即前置。

## 1. 端到端流程与交接合同

```text
本地修改
  -> just/模块命令（当前可修复工作树）
  -> push/PR
  -> surface relevance + required aggregator
  -> 分支保护（strict checks，dev 可 admin bypass）
  -> merge 到 main/dev
  -> v* tag
  -> 原生平台 build + checksum（当前各 job 直接公开）
  -> GitHub Release（未形成真实 deployment）
  -> 用户安装/手工验证（无统一 post-release smoke）
```

| 交接 | 当前入口/触发 | 当前门禁与依赖 | 主要断点 |
| --- | --- | --- | --- |
| 本地 -> PR | `just`、根/UI/VS Code 命令；push 后开 PR | 版本、格式、lint、测试、构建、审计 | `just ci` 会 repair；全量阶段串行 |
| PR -> required CI | 四个 workflow 的 `pull_request`；Frontend 另有 `push` | 四个稳定 aggregator；surface relevance 在 workflow 内判断 | `dev` 直推时三个 workflow 不运行 |
| CI -> merge | `main`/`dev` strict required contexts | Root、Vue/Docs、Tauri Linux、VS Code | `dev` admin 可 bypass；无 required review/conversation resolution |
| tag -> Release | `.github/workflows/release.yml` 的任意 `v*` tag | 各平台 build、checksum、VSIX/Tauri | 无 tag preflight；矩阵 job 直接写公开 Release |
| Release -> 分发/部署 | GitHub Release 资产 | 当前只可见 checksum 完整性说明 | `release` environment 未接入；deployments 为空；无统一安装 smoke |

交接合同建议固定为：同一 commit/tag -> 同一组已验证 artifacts -> 受控 publish -> 发布后 smoke；任何阶段失败都应产生可行动的失败状态，而不是依赖管理员绕过或从部分公开资产推断成功。

## 2. 数据样本、口径与限制

### 2.1 实时状态（as of 2026-07-31 20:03 CST）

- `origin/main...origin/dev`：`1 behind / 80 ahead`；本地/远端没有活动 `develop` 分支。
- 开放 PR：23 个，作者全部为 Dependabot，base 全部为 `main`，其中 12 个 `BLOCKED`；当前没有开放的人类 PR。
- `main` 与 `dev` 都是 protected、strict，required contexts 均为四个稳定名称；`main.enforce_admins=true`，`dev.enforce_admins=false`；两者没有 required reviews，conversation resolution 关闭。
- 仓库 owner 类型为 User；merge commit、squash、rebase 三种 merge method 均启用，auto-merge 关闭，`delete_branch_on_merge=false`。当前没有证据表明 merge method 选择本身是时延或失败来源。
- 最新 `dev` push run：Frontend CI `30620306511`，head `a09cf340953724771edd0ac8f2eea9cf63826840`；Root/Tauri/VS Code 没有对应 push run。
- 最新 GitHub Release：v6.5.0（2026-07-09）；`GET /deployments` 返回 `[]`。
- `release` environment 存在，允许管理员 bypass、配置 `v*` branch policy，但没有任何 release job 声明 `environment: release`。

实时查询入口：

- [main protection API](https://api.github.com/repos/bahayonghang/ccr/branches/main/protection)
- [dev protection API](https://api.github.com/repos/bahayonghang/ccr/branches/dev/protection)
- [开放 PR 查询](https://github.com/bahayonghang/ccr/pulls?q=is%3Aopen)
- [Actions runs](https://github.com/bahayonghang/ccr/actions)
- [Release environment](https://github.com/bahayonghang/ccr/deployments/activity_log?environments_filter=release)
- [Deployments API](https://api.github.com/repos/bahayonghang/ccr/deployments)

### 2.2 历史样本

研究基线从 `gh run list --limit 200` 取样：`createdAt >= 2026-07-27T00:00:00Z` 的 136 个 completed runs，其中 124 个产品 CI、12 个 Dependabot Updates。端到端时间定义为 `updatedAt - createdAt`，P90 使用 nearest-rank；它同时包含 queue 和 execution，不能当作纯编译时长。

| Workflow | Runs | Success/Failure | Median | P90 |
| --- | ---: | ---: | ---: | ---: |
| Root CI | 29 | 23 / 6 | 11.78m | 20.15m |
| Tauri Rust CI | 29 | 26 / 3 | 14.58m | 26.85m |
| Frontend CI | 37 | 32 / 5 | 4.73m | 15.25m |
| VS Code CI | 29 | 27 / 2 | 6.10m | 14.67m |
| Dependabot Updates | 12 | 12 / 0 | 1.98m | 8.25m |

代表性拆分：

- Root run [30277161602](https://github.com/bahayonghang/ccr/actions/runs/30277161602)：Windows tests queue/执行以执行为主，安装 `just` 126s；coverage 工具安装 147s、coverage 197s；security audit 安装 `cargo-audit` 121s、实际 audit 5s。
- Tauri run [30280315441](https://github.com/bahayonghang/ccr/actions/runs/30280315441)：Linux queue 733s、执行 791s；coverage queue 746s；macOS queue 1,485s、执行 488s。该 run 处于 Dependabot burst runner 竞争期，不能把 queue 归因给 YAML 或编译。
- PR [#42](https://github.com/bahayonghang/ccr/pull/42)：创建到合并 7.75h，72 commits、605 files、+53,993/-7,274 lines、0 reviews；四个 PR workflow 各运行 7 次，累计 aggregate wall-minutes 为 Root 83.3、Tauri 99.4、Frontend 30.7、VS Code 16.4。

### 2.3 限制

- Dependabot burst 使 queue P90 偏高；应在至少 2-4 周正常流量后重算。
- 没有缓存 hit/miss、flake、取消率、review SLA、真实安装成功率或 deployment history，不能计算节省百分比、change failure rate、MTTR 或 deployment frequency。
- branch protection API 证明当前设置，不证明历史每一次 push 均使用相同设置。
- 没有证书、Marketplace 账号、平台支持矩阵或签名运营证据；签名/公证/updater 只能条件性讨论。

## 3. 现有优点（应保留）

1. 四个稳定 required aggregator 使用 `if: always()`，surface relevance 在 workflow 内部判断，避免顶层 path filter 造成永久 Pending。对应入口为 `.github/workflows/ci.yml:127-129`、`frontend-ci.yml:73-75`、`tauri-rust-ci.yml:101-103`、`vscode-ci.yml:60-62`。
2. workflow 顶层已有 `contents: read`（Release 除外），43 个实际 action 引用固定到完整 SHA，治理检查通过，serial-only test annotation 为 0。
3. Rust/Tauri/Vue/Docs/VS Code 均有专用门禁、覆盖率或审计步骤，且本地命令被 hosted workflow 复用；不建议用“快速”名义删除必要的锁文件、版本同步或安全门。
4. Release 文案已正确声明：产物未使用 publisher identity 签名，SHA-256 只证明完整性，updater 保持禁用（`.github/workflows/release.yml:140-145`）。这与 `.trellis/spec/ccr/backend/dependency-governance.md:208-235` 的 `ACCEPTED_RISK` 合同一致。

## 4. 主要发现与建议

### F-PR-01（P0）`dev` 管理员 bypass 造成不完整 push 反馈

- **证据（事实）**：`main`/`dev` required contexts 相同且 strict；`dev` `enforce_admins=false`。Root/Tauri/VS Code 只有 `pull_request`（`.github/workflows/ci.yml:3-5`、`tauri-rust-ci.yml:3-5`、`vscode-ci.yml:3-5`），Frontend 才监听 `push`（`.github/workflows/frontend-ci.yml:3-12`）。实时最新 `dev` push `a09cf340` 只有 Frontend CI。该提交上 `just version-check` 先通过代码版本 7.1.1，随后因 `ccr-ui/README.md:5` 仍为 `version-7.0.0` 失败。
- **影响**：体验上 push 后没有完整反馈；治理上 required context 与实际执行入口不一致；版本文档漂移可能在管理员直推后才被发现。
- **根因（推断）**：触发器只覆盖 PR，而分支保护又允许管理员跳过，所以“required”在 dev 直推场景退化为事后局部信号。替代解释是该 push 只涉及 frontend，但 Root/Tauri/VS Code 的 aggregator 本应明确报告 irrelevant；当前它们根本没有创建。
- **建议**：短期给 Root/Tauri/VS Code 增加 `dev` push 触发，并让 change detection 在 PR 使用 `base...head`、在 push 使用 `before...after`（空值或解析失败必须 fail closed）；保留 aggregator，确保每个 dev push 都产生四个可追踪 context。同时记录管理员 bypass 为 break-glass 事件。若 2-4 周后确认维护者不需要直推，再将 `dev.enforce_admins` 收紧为 true、统一 PR-to-dev 合同。
- **收益**：每次 dev push 都有完整、可定位的状态，降低漏掉文档/绑定漂移的概率；不减少 required gate 覆盖。
- **成本**：M；需修改三个 workflow、补充 push 场景测试和 branch protection 运行手册。
- **风险/取舍**：保留 bypass 仍是事后反馈，不能等同于 push 前阻断；为每个 dev push 创建四个 context 会增加 runner 使用，应与 F-CI-01 的分支 concurrency 一起落地；立即开启 admin enforcement 会增加单维护者紧急修复等待。
- **前置条件**：确认 dev 是否必须支持紧急直推；为 bypass 指定原因、责任人和复核时限。
- **验证指标**：连续 2-4 周每个 dev push 均有四个 context；bypass 次数/原因；push 到首个 actionable result 和到四个 context 完成的 P50/P90；版本漂移失败是否在完整 gate 暴露。

### F-PR-02（P0）Dependabot burst 与分支策略不匹配

- **证据（事实）**：实时有 23 个开放 Dependabot PR，12 个 `BLOCKED`，全部 base=`main`；`.github/dependabot.yml:3-32` 有六个 weekly ecosystem，但没有 `groups`、`open-pull-requests-limit`、`target-branch` 或时间错峰。
- **影响**：同一时段大量 PR 竞争 runner，queue time 放大；blocked PR 增加维护者清理成本；依赖更新与 `dev -> main` 的分支流不一致。
- **根因（推断）**：每个 ecosystem 独立 weekly 创建 PR，且没有合并批次、并发上限或目标分支合同。替代解释是 blocked 主要来自 strict branch drift，而非单纯 PR 数量；需要按状态历史确认。
- **建议**：先按 ecosystem 建最小 grouping，设置保守 open limit（例如每个 ecosystem 1-2，具体值用 2-4 周数据校准），将六个 schedule 错峰；再决定依赖 PR 目标是 `dev` 还是 `main`，并写入维护手册。不要直接把 target branch 改成 `dev` 而不验证发布分支和 required checks 合同。
- **收益**：减少同时创建的重复验证和 queue 峰值，降低 blocked backlog；可保留每类依赖的独立回滚边界。
- **成本**：S-M；仅需 Dependabot 配置、合并策略说明和观察期。
- **风险/取舍**：grouping 使单个 PR 变大，某一依赖失败会拖住同组；过低 limit 会延迟安全更新。
- **前置条件**：明确安全 advisory 是否允许独立 PR；确认 `dev` 是否为依赖集成分支。
- **验证指标**：开放 Dependabot 数量与 blocked 占比、每周创建峰值、同一 SHA 的重复 workflow 数、queue P50/P90、安全更新从发布到合并的时间。

### F-PR-03（P1）PR 批量和评审可观测性不足

- **证据（事实）**：PR #42 有 72 commits、605 files、7.75h、0 reviews；研究样本中的 16 个历史 merged PR 的 review API 记录均为 0。当前没有版本化 PR template、CODEOWNERS 或评审 SLA 文档。
- **合并现状（事实）**：仓库同时允许 merge commit、squash 和 rebase，未启用 auto-merge，也不自动删除已合并分支；当前样本不能证明这些设置造成瓶颈。
- **影响**：大批量变更降低定位/回滚能力；无 review 事件时无法区分“单维护者批准”与“无人复核”。这是流程治理风险，不等同于代码必然有缺陷。
- **根因（推断）**：当前低人类 PR 流量和单维护者模式没有形成最小评审合同。替代解释是 review 发生在 PR 外部或维护者自审，GitHub API 不可见。
- **建议**：添加轻量 PR template（变更面、风险、测试、是否生成物/依赖更新）和“小批量优先”指导；暂不强制多人 approval。对 `.github/workflows`、发布脚本和签名配置，待有第二维护者或高风险变更频率后再评估 CODEOWNERS/required review。
- **收益**：降低 PR #42 类超大批次，改善首个可行动反馈与回滚边界；不引入低流量下的等待仪式。
- **成本**：S；文档和模板维护成本低。
- **风险/取舍**：过度按行数拆分会把生成文件/锁文件拆碎；模板本身不能证明真实 review。
- **前置条件**：定义哪些生成物可随主变更提交，哪些变更需要人工复核。
- **验证指标**：PR changed files/commits 分布、ready-for-review 到首次响应/merge 的中位数、review 数和 rework 次数、超大 PR 占比。

### F-LOCAL-01（P1）`just ci` 将修复与验证混在一个串行门禁

- **证据（事实）**：根 `justfile:499-523` 的 `ci` 依次执行 `version-sync`、`fmt`、`fmt-check`、lint、测试、release、audit、治理、bindings、frontend、VS Code 共 12 阶段；`version-sync` 与 `fmt` 可能写入版本文件/Rust/JSON。当前 `just version-check` 在版本 7.1.1 与 `ccr-ui/README.md` 7.0.0 漂移处失败。
- **影响**：开发者难以知道“原始工作树是否通过”；失败后可能留下门禁生成的改动；全量串行延长首个 actionable feedback。
- **根因（推断）**：`ci` 同时承担 repair 和 acceptance，且把独立 surface 当作线性脚本。替代解释是自动格式修复被认为是本地体验特性，但 hosted required gate 的可重复性仍受影响。
- **建议**：保留显式 `ci-repair`/`version-sync`/`fmt`，新增 verification-only full gate（只调用 `--check`/不写入 recipe）；将独立 surface 在 required CI 中并行，真正有数据依赖的步骤保持串行。不要改变已有阈值。
- **收益**：通过/失败含义稳定，减少工作树污染；缩短可行动反馈，保留完整覆盖。
- **成本**：M；需维护 recipe 依赖图、Windows/Unix 两套 wrapper 和测试。
- **风险/取舍**：把 repair 命令拆开后，开发者可能忘记运行；应在失败信息中给出明确修复命令。
- **前置条件**：逐步标注所有会写文件的 recipe，确认 hosted workflow 不依赖隐式生成物。
- **验证指标**：verification-only 工作树零变更；失败后未跟踪/修改文件数；首次失败定位时间；全 gate execution P50/P90。

### F-LOCAL-02（P1）前端依赖安装被重复执行

- **证据（事实）**：`justfile:699-720` 的 `frontend-typecheck`、`frontend-lint`、`frontend-test`、`frontend-build` 各执行一次 `cd ccr-ui && bun install --frozen-lockfile`；`frontend-check` 在 `:736-738` 组合四个子检查。docs 另有独立安装（`:723-728`）。
- **影响**：一次 frontend full check 重复解析/校验同一 lockfile，固定消耗时间和网络；并可能造成安装日志噪声。
- **根因（推断）**：recipe 以单命令可独立调用为优先，没有 session-level install boundary。替代解释是 Bun 已命中本地缓存，实际额外成本在不同 runner 上差异很大。
- **建议**：为本地 full check 和 CI job 提供一次安装后的组合 recipe；独立子命令保留自包含语义，但组合命令不重复安装。CI 继续 `--frozen`，不扩大缓存范围。
- **收益**：减少固定安装解析和重复日志；独立命令兼容性保持不变。
- **成本**：S-M；调整 recipe 依赖图并补充 clean checkout 验证。
- **风险/取舍**：共享安装目录可能掩盖 lockfile/平台差异；组合命令必须在同一 job 中明确安装边界。
- **前置条件**：确认 Bun 版本、`node_modules` 生命周期和并发执行不会共享不安全目录。
- **验证指标**：cold/warm 安装耗时、frontend full check 的 install 次数、cache hit/miss、失败后重跑转绿率。

### F-CI-01（P0）过期 PR run 不取消且缺少超时

- **证据（事实）**：四个 PR workflow 中没有 `concurrency` 或 `timeout-minutes`；PR #42 的四个 workflow 各运行 7 次，研究记录没有旧运行取消，累计 aggregate wall-minutes 为 209.8。
- **影响**：连续 push 时旧 SHA 仍占用 runner，queue 和账单时间增加；卡死或外部依赖异常没有明确上限。
- **根因（推断）**：workflow 只定义了 job 依赖和 `fail-fast`，没有按 PR/ref 建立验证取消合同。替代解释是旧运行仍可提供诊断，但对 required context 不应继续占用同等资源。
- **建议**：PR/branch verification 使用 `concurrency.group`（包含 workflow、PR 或 ref）和 `cancel-in-progress: true`；每个 job 设置基于历史 P95 的 timeout。Release 使用独立不可取消或受控排队 group，不套用 PR 策略。
- **收益**：新 push 更快获得 runner，减少无效执行；超时失败可行动且可统计。
- **成本**：S；配置和少量治理测试。
- **风险/取舍**：取消会丢失旧 SHA 的完整日志；必须保留新 run 的 SHA、取消原因和 artifact retention 规则。
- **前置条件**：确认 required aggregator 对 cancelled/skipped 状态 fail closed；为 release/deploy 单独定义 group。
- **验证指标**：取消率、每 PR 无效 run 数、queue P50/P90、超时率、required gate P50/P90、失败后重跑转绿率。

### F-CI-02（P1）queue 与 execution 未分离，工具现场安装放大固定成本

- **证据（事实）**：研究样本的端到端时长是 `updatedAt-createdAt`；Tauri 示例有 733-1,485s queue。Root run 中 `cargo-audit` 安装 121s、实际 audit 5s，coverage 工具安装 147s；多个 workflow 现场 `cargo install just`（例如 `.github/workflows/ci.yml:47,77,106`）。
- **影响**：维护者可能把 runner 容量问题误当成编译慢，也可能为少量检查支付重复工具构建成本。
- **根因（推断）**：没有统一的 queue/execution/step 指标，也没有验证工具预装、精确缓存或 artifact 复用的收益。Tauri queue 受 Dependabot burst 强烈影响。
- **建议**：先按 job 记录 `startedAt-createdAt` 与 `completedAt-startedAt`，再比较选择性 Cargo registry/cache、工具缓存或预装 runner image；独立 job 并行，数据依赖保持串行。缓存 key 至少含 OS、toolchain、lockfile，避免共享凭据和不可信构建缓存。
- **收益**：把容量问题和 workflow 问题分开，减少无效优化；有证据后再选择缓存/预装方案。
- **成本**：M；需指标脚本、缓存实验和回归窗口。
- **风险/取舍**：缓存上传/恢复可能超过节省；错误 key 会产生陈旧或不一致结果，Rust 官方也不建议无条件缓存整个 `CARGO_HOME`。
- **前置条件**：至少 2-4 周样本、可识别的 cold/warm run、确认 GitHub plan/runner 能力。
- **验证指标**：queue/execution P50/P95、工具安装耗时、缓存 hit/miss/restore 时间、失败率和有无缓存输出一致性。

### F-CI-03（P2）治理规范计数与实际 action 引用数漂移

- **证据（事实）**：`.trellis/spec/ccr/backend/dependency-governance.md:337` 仍写“52 immutable action references”；当前只读 `python scripts/check_workflow_governance.py` 输出为“43 immutable action references”，且 serial-only 为 0、检查通过。
- **影响**：维护者无法判断 52 是历史快照、预期阈值还是实际计数；文档漂移会削弱治理审计的信任，但不等于 action 未固定。
- **根因（事实+推断）**：规范中的固定计数未随 workflow 引用变更同步。当前脚本是实际来源；没有证据表明 9 个引用缺失。
- **建议**：让规范说明“由脚本动态报告”，或在发布/治理检查中生成带时间戳的计数；不要把 52 硬编码为门禁阈值。将文档计数修正作为低风险治理维护项。
- **收益**：减少知识漂移和误报；保留完整 SHA 与 relevance/aggregator 合同。
- **成本**：S；更新一处规范和相关测试说明。
- **风险/取舍**：动态计数掩盖了错误 action 新增；仍需保留“所有引用必须完整 SHA”的断言。
- **前置条件**：确认脚本扫描范围与治理报告格式。
- **验证指标**：规范/脚本/实际引用数一致；治理测试对可变 tag、重复 YAML key、缺失 workflow 仍 fail closed。

### F-REL-01（P0）Release 直接公开部分产物

- **证据（事实）**：`.github/workflows/release.yml:3-6` 对任意 `v*` tag push 触发；顶层 `contents: write`（`:8-10`）。CLI matrix 在 `:95-157` 使用 `softprops/action-gh-release` 且 `draft: false`；VSIX 在 `:202-209` 直接上传；Tauri 在 `:255-264` 使用 `tauri-action` 且 `releaseDraft: false`。`post-release` 仅在 `:267-270` 等全部 job 后汇总，晚于公开动作。
- **影响**：一个平台失败时，用户可能先看到部分或不一致的公开资产；Release published 不能证明所有平台 build、安装或 deployment 成功。
- **根因（推断）**：build 与 publish 绑定在每个平台 job；没有统一 artifact/verify/publish 汇聚点。替代解释是作者有意允许逐平台下载，但当前文案未将其定义为 preview/partial release。
- **建议**：拆为 `build`（各平台上传 artifact）-> `verify`（下载同一 artifacts，校验 checksum、版本、文件清单和最低 smoke）-> `publish`（唯一 job，使用 `environment: release`，全部成功后将 draft 转 published）。保留 checksum-only 语义，不自动引入签名。
- **收益**：公开 Release 只包含同一批已验证资产；失败不会留下看似完成的部分发布。
- **成本**：M-L；需重构 workflow、artifact retention、发布环境权限和回滚说明。
- **风险/取舍**：会增加 artifact 存储和最终发布等待；publish job 失败时需可重试且不重复上传。
- **前置条件**：确认 environment 审批/branch policy、tag 与 commit 合同、各平台 artifact 命名和保留期。
- **验证指标**：matrix 全成功后才发布的比例、partial release 次数（目标 0）、verify 失败率、publish queue/执行时间、同一 tag 的文件清单和 digest 一致性。

### F-REL-02（P1）tag/版本/发布后验证合同不完整

- **证据（事实）**：Release 仅按 `v*` tag 触发，没有 tag 与代码版本、来源 `main`、required CI 或 CHANGELOG preflight；CLI release build 在 `.github/workflows/release.yml:74` 也没有使用 `--locked`。当前代码 7.1.1、`main` 7.0.0、最新 Release v6.5.0；`CHANGELOG.md:8` 为 Unreleased，最新正式 section 为 4.3.0。GitHub deployments API 为空。
- **影响**：错误来源或版本可能进入 build；发布后没有可观察的安装/启动/版本输出验证；不能把 Release published 当成部署成功。
- **根因（推断）**：版本、tag、资产和渠道状态没有统一发布记录。替代解释是发布由人工在外部流程核对，但仓库没有可追踪证据。
- **建议**：在 build 前校验 tag 解析版本、目标 commit 已通过 required CI 且来自允许 ref；所有 Cargo release build 使用 `--locked`；生成版本/commit/run/asset digest/校验结果清单；发布后对下载文件做 checksum、干净环境安装、启动与 `--version` smoke，并把失败状态写入环境/发布记录。
- **收益**：降低错 tag 和“上传成功即发布成功”的误判；为恢复和审计提供单一索引。
- **成本**：M；需要跨 OS smoke 设计和少量结构化 artifact。
- **风险/取舍**：跨平台安装 smoke 增加运行时间；没有平台支持矩阵时不能强行覆盖所有环境。
- **前置条件**：确定稳定/预览渠道和每个平台最低支持版本；定义 GitHub Release 何时计为“对用户可用”。
- **验证指标**：tag preflight 拒绝数、发布记录完整率、下载/安装/启动 smoke 成功率、失败发布恢复时间、每版本可追溯到的 run/commit/digest 比例。

### F-REL-03（P2，当前接受风险）签名、attestation、updater 不应直接变成当前发布前置

- **证据（事实）**：`.trellis/spec/ccr/backend/dependency-governance.md:208-235` 明确 unsigned release 是 `ACCEPTED_RISK`，不得要求 signing identity、certificate、notarization、attestation；`.github/workflows/release.yml:140-145` 也明确 SHA256 不是 publisher authentication，updater 保持禁用；Tauri config 的 `signingIdentity`/`certificateThumbprint` 为 null，未发现 updater 配置。
- **影响**：当前主要风险是身份不可认证，而不是 checksum 完整性缺失；将未准备好的签名/attestation 强塞进 P0 会导致发布阻断和新的密钥运维风险。
- **根因（事实）**：用户/项目已有“checksum-only + updater disabled”的安全决策。
- **建议**：当前路线只完成 atomic publish、最小权限和可追溯 checksum。签名/公证、artifact provenance、SBOM、updater 仅列为新安全决策后的 P2；若启用，私钥必须留在受控 environment，消费者实际验证主体、builder、commit 和 digest。
- **收益**：不违反已批准的 accepted-risk 边界，同时保留未来升级路径。
- **成本**：当前 S（保持现状）；未来 M-L，取决于证书、notarization、GitHub plan 和 updater 恢复演练。
- **风险/取舍**：继续 unsigned 仍有 publisher spoofing 风险；但不能用 SHA256 冒充身份认证。
- **前置条件**：新的明确安全决策、平台证书/账号、密钥轮换与恢复流程、消费者验证方案。
- **验证指标**：当前 checksum 文件完整率与下载校验成功率；未来再增加签名/公证/attestation 验证成功率，不能把缺失身份报告为 PASS。

## 5. 行业对照（官方/一手来源）

| 主题 | 官方基线 | CCR 当前 | 适用条件与差距 |
| --- | --- | --- | --- |
| 分层反馈/小批量 | Google 建议小而自洽的 CL；[Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html) | PR #42 72 commits/605 files；本地 full gate 串行 | 单维护者低流量适合轻量 template 和指导，不适合机械行数门禁 |
| 取消过期验证 | GitHub `concurrency` 支持每组一个运行并取消旧 run；[workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#concurrency) | 无 concurrency/timeout | PR verification 适用；release/deploy 应用独立不可取消策略 |
| required aggregator | GitHub 说明被 path/skip 的 required workflow 会永久 Pending；[Skipping workflow runs](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-workflow-runs/skipping-workflow-runs) | 四个 aggregator always-run，属于现有优点 | 应保留；dev push 入口仍需补齐 |
| cache/工具 | Cargo 建议按需缓存 registry/index/git/db，不要无条件缓存整个 `CARGO_HOME`；[Cargo Home](https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci) | 多处现场 `cargo install`，无 PR cache | 先分离 queue/execution，再做 cold/warm 实验；缓存不得带凭据 |
| Rust 快速反馈 | `cargo check` 跳过最终代码生成，不能替代 build/test；[cargo check](https://doc.rust-lang.org/cargo/commands/cargo-check.html) | 已有 `check-workspace`/lint/test/release 分层 | 可把 check 放 fast 层，不能用它宣称发布构建已验证 |
| 发布汇聚 | Tauri 官方 pipeline 支持原生 OS matrix 和 draft release；[Tauri pipeline](https://v2.tauri.app/distribute/pipelines/github/) | matrix job 直接公开 | build -> verify -> controlled publish 适用；签名取决于平台账号和新决策 |
| Environment | GitHub environment 在保护规则通过前不放出 secrets；[environments](https://docs.github.com/en/actions/deployment/targeting-different-environments/managing-environments-for-deployment) | `release` environment 存在但未被 job 使用 | 适合唯一 publish job；当前只需把受控 publish 接上，不等于有 deployment |
| 最小权限/不可变 action | GitHub 建议最小 `GITHUB_TOKEN`；完整 SHA 是不可变 action 引用；[security hardening](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions) | 非 Release workflow 已 `contents: read`，实际 43 refs 全 SHA | Release 仍顶层 `contents: write`，应下沉到 publish job |
| 度量 | DORA 五指标要求按应用上下文使用；[DORA metrics](https://dora.dev/guides/dora-metrics-four-keys/)；Actions 提供 queue/runtime/失败指标 | 无 deployment history/cache/flake 基线 | 先建立 2-4 周基线，不把任意 5/10 分钟当行业 SLO |

## 6. P0/P1/P2 路线图

### P0：0-2 周，修复交接和明显浪费

1. **恢复 dev 完整反馈并写明 bypass 合同（F-PR-01）**：三个 PR-only workflow 增加 dev push 入口，或在确认不需要直推后启用 `enforce_admins`；保留四个 aggregator 和 relevance。验收：每个 dev push 四个 context 均可追踪，bypass 有原因记录。
2. **PR concurrency + job timeout（F-CI-01）**：验证按 PR/ref 取消旧 run，release 独立 group。验收：旧 SHA 的无效 run 数、queue P50/P90、超时率和 required gate P50/P90 可比较。
3. **Dependabot grouping/limit/stagger（F-PR-02）**：先按 ecosystem 分组、设保守上限并错峰；再用数据决定 `main`/`dev` target。验收：开放/blocked PR 峰值下降，安全更新延迟不恶化。
4. **Release build -> verify -> publish（F-REL-01）**：统一 artifacts、verify job 和唯一 `environment: release` publish；发布前为 draft/不可见状态。验收：任何 matrix/verify 失败都不产生公开 Release，partial release 目标为 0。

### P1：2-6 周，稳定本地反馈和可观测性

1. **verification-only full gate + 显式 repair（F-LOCAL-01）**：保留原有阈值和 repair 能力，消除隐式工作树修改。
2. **frontend 安装去重（F-LOCAL-02）**：组合 recipe/job 一次安装，独立 recipe 仍可自包含。
3. **queue/execution/step 指标与选择性 cache（F-CI-02）**：先测再决定 cache、工具预装或矩阵并行；比较 cold/warm 结果，不承诺固定百分比。
4. **轻量 PR template/批量指导（F-PR-03）**：记录风险、测试和生成物；暂不要求多人 approval。
5. **release preflight + post-release smoke（F-REL-02）**：记录 tag、commit、run、asset digest、checksum、安装/启动/版本结果；GitHub Release published 不等同 deployment success。

### P2：有新证据/新决策后

1. **签名、公证、provenance、SBOM（F-REL-03）**：只有在重新批准 accepted-risk、取得平台账号/证书并定义消费者验证后启用；不得作为当前 P0 前置。
2. **merge queue**：只有 strict 重跑和并发冲突成为可测瓶颈，且所有 required workflow 已监听 `merge_group` 时评估。
3. **self-hosted/更大 runner**：只有 queue 基线证明 hosted 容量是主要瓶颈，且组织能承担隔离、补丁、密钥和容量运维时评估。
4. **updater/Marketplace/多渠道部署**：先定义支持矩阵、签名/更新密钥、恢复和真实 deployment 目标；不能因已有 GitHub Release 就假定这些渠道存在。
5. **治理文档计数修正（F-CI-03）**：将 `52` 改成动态计数说明或刷新为有时间戳的事实，不改变完整 SHA 门禁。

## 7. 暂缓或不建议现在做

- **现在不启用 merge queue**：当前没有人类 PR 流量，23 个 PR 是 Dependabot burst；queue 新增的 merge-group 构建和 workflow 触发复杂度尚未证明有正 ROI。
- **现在不迁 self-hosted runner/更大 runner**：缺少按 job 的 queue/execution 基线和容量成本；本次 Tauri queue 明显受 Dependabot burst 影响。
- **现在不强制多人审批或全面 CODEOWNERS**：16 个历史 merged PR 无 GitHub review 记录，且没有第二维护者/SLA 证据；先用 template 和高风险目录评估，不引入低流量阻塞。
- **现在不强制单一 merge method 或启用 auto-merge**：三种 merge method 的现状尚未显示为失败来源；先定义提交历史、回滚和分支清理合同，再决定是否收紧。
- **现在不把 signing、notarization、attestation、SBOM 或 updater 作为 release 前置**：违反 `.trellis/spec/ccr/backend/dependency-governance.md:208-235` 的 accepted-risk 合同；需要新的安全决策和平台前置。
- **现在不做无边界的全 Cargo/构建目录缓存**：Cargo 官方警告整个 `CARGO_HOME` 可能因压缩上传反而变慢；先做选择性、可验证缓存实验。
- **现在不把 GitHub Release published 当 deployment success**：当前 deployment API 为空，真实用户安装/启动/更新状态仍未验证。

## 8. 证据缺口与后续测量计划

| 缺口 | 为什么重要 | 最小补测 |
| --- | --- | --- |
| queue/execution/cache hit/miss | 决定是容量问题、工作流问题还是工具安装问题 | 每个 run/job 记录 created/started/completed、安装 step、cache 命中和 restore 时间，连续 2-4 周 |
| 取消率与 flaky rerun | 判断 concurrency 是否减少浪费而非隐藏失败 | 记录 cancelled、failure 后 rerun、同 SHA 首次绿灯 |
| 人类评审/响应 SLA | 不能从 review=0 推导“无人复核” | 记录 review event、ready-for-review、首次响应、merge 时间；区分维护者自审/外部沟通 |
| bypass 历史 | 当前保护设置不能回溯所有 push | 记录 dev push、admin bypass、对应四个 context 和事后修复状态 |
| 发布成功/失败恢复 | 无 deployment history 无法算 DORA change failure/MTTR | 为每个 tag 记录 preflight、matrix、verify、publish、下载/安装/启动 smoke 和恢复动作 |
| 平台签名/渠道前置 | 决定 P2 是否有正 ROI | 维护支持 OS/架构、证书/账号、Marketplace/updater 和 GitHub plan 清单 |
| action 计数漂移 | 规范 `52` 与脚本 `43` 的来源不明 | 在治理脚本输出中记录扫描范围、时间和 SHA 引用清单，修正规范描述 |

## 9. 证据索引与官方来源

### 仓库与任务证据

- `justfile:499-523`：12 阶段串行 `ci`；`justfile:699-741`：前端重复安装和组合 recipe。
- `.github/workflows/ci.yml:3-8,47,77-78,127-129`：PR-only、现场安装、Root aggregator。
- `.github/workflows/frontend-ci.yml:3-15,56,73-75`：push + PR、前端 required aggregator。
- `.github/workflows/tauri-rust-ci.yml:3-8,47,87-103`：PR-only、工具安装、Tauri aggregator。
- `.github/workflows/vscode-ci.yml:3-8,45,60-62`：PR-only、工具安装、VS Code aggregator。
- `.github/workflows/release.yml:3-10,74,95-157,202-209,255-270`：tag 触发、未锁定的 CLI build、权限、直接公开资产、post-release summary。
- `.github/dependabot.yml:1-32`：六个 weekly ecosystem，无 grouping/limit/target/stagger。
- `ccr-ui/README.md:5`：`version-7.0.0`；`CHANGELOG.md:8,68`：Unreleased 与最新正式 4.3.0。
- `ccr-ui/src-tauri/tauri.conf.json:41,44`：平台 signing identity/certificate thumbprint 为 null。
- `.trellis/tasks/07-31-dev-pr-ci-deploy-workflow-audit/research/repository-workflow-baseline.md`：历史样本、job/step 拆分、分支/Release/Dependabot 事实与限制。
- `.trellis/tasks/07-31-dev-pr-ci-deploy-workflow-audit/research/industry-practices.md`：官方行业对照与适用条件。
- `.trellis/spec/ccr/backend/dependency-governance.md:208-235`：unsigned release accepted-risk；`:337`：52 引用计数旧描述。

### GitHub 只读证据

- PR #42：[https://github.com/bahayonghang/ccr/pull/42](https://github.com/bahayonghang/ccr/pull/42)
- PR #42 代表性 run：[Root 30277161602](https://github.com/bahayonghang/ccr/actions/runs/30277161602)、[Tauri 30280315441](https://github.com/bahayonghang/ccr/actions/runs/30280315441)
- 最新 `dev` push：[Frontend CI 30620306511](https://github.com/bahayonghang/ccr/actions/runs/30620306511)
- [Branches protection REST endpoints](https://docs.github.com/en/rest/branches/branch-protection)
- [Release list](https://github.com/bahayonghang/ccr/releases)、[environment activity](https://github.com/bahayonghang/ccr/deployments/activity_log?environments_filter=release)、[deployments API](https://api.github.com/repos/bahayonghang/ccr/deployments)

### 官方/一方资料

- [DORA 软件交付绩效指标](https://dora.dev/guides/dora-metrics-four-keys/)
- [GitHub Actions metrics](https://docs.github.com/en/actions/concepts/metrics) 与 [workflow concurrency](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#concurrency)
- [GitHub protected branches/rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)
- [GitHub skipping workflow runs](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-workflow-runs/skipping-workflow-runs)
- [GitHub dependency caching](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/caching-dependencies-to-speed-up-workflows)、[artifacts](https://docs.github.com/en/actions/concepts/workflows-and-actions/workflow-artifacts)、[environments](https://docs.github.com/en/actions/deployment/targeting-different-environments/managing-environments-for-deployment)
- [GitHub security hardening](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions)、[artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds)
- [Cargo check](https://doc.rust-lang.org/cargo/commands/cargo-check.html)、[Cargo locked builds](https://doc.rust-lang.org/cargo/commands/cargo-build.html)、[Cargo Home caching](https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci)
- [Tauri GitHub pipeline](https://v2.tauri.app/distribute/pipelines/github/)、[Tauri distribution](https://v2.tauri.app/distribute/)、[Tauri updater](https://v2.tauri.app/plugin/updater/)
- [Google Engineering Practices: Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html)
- [SLSA v1.2 build requirements](https://slsa.dev/spec/v1.2/build-requirements)

## 结论

CCR 当前的核心质量门并不薄弱；真正的高收益路径是把已有门禁接到所有实际入口、取消过期验证、降低依赖突发、将 Release 改成受控的 build/verify/publish，并先建立 queue/execution/release 指标。所有涉及签名、updater、attestation、merge queue 或自托管基础设施的事项都应在补齐前置证据和新的安全/运维决策后再进入 P2，而不是作为本次审计的立即实施项。
