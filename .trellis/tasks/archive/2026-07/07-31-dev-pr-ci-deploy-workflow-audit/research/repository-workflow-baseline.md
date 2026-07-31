# Research: CCR 开发、PR、CI、合并与发布流程现状

- Scope: repository + GitHub read-only evidence
- Captured: 2026-07-31 (Asia/Shanghai)
- Repository: `bahayonghang/ccr`
- Branch: `dev`
- Evidence boundary: 本文件记录命令输出和配置事实，不实施任何改动；建议仍须与 `industry-practices.md` 交叉分析。

## 1. 执行摘要

当前流程已经具备较强的基础质量门：四个稳定 required context、按 surface 判定相关性的聚合门禁、完整 SHA 固定的 Actions、Rust/前端/Tauri/VS Code 覆盖率与安全检查，以及严格的 `main` 保护。主要问题不是缺少检查，而是反馈链路在不同入口之间不一致，并且相当一部分时间消耗在可避免的安装、排队和重复运行上。

已证实的高影响事实：

1. `dev` 允许管理员绕过 required checks，而 Root/Tauri/VS Code workflow 只监听 PR。2026-07-31 最新一次管理员直接 push 到 `dev` 只运行 Frontend CI；同一提交上，本地 `just version-check` 因 `ccr-ui/README.md` 仍写 `version-7.0.0` 而失败。
2. 最近样本的 Tauri CI 中位端到端时长为 14.58 分钟，P90 为 26.85 分钟；Root CI 中位数为 11.78 分钟，P90 为 20.15 分钟。安装 `just`、`cargo-audit`、`cargo-llvm-cov` 和 Linux 系统依赖占据明显比例。
3. PR #42 在 7.75 小时内累积 72 个提交、605 个文件，并触发四个 PR workflow 各 7 轮；仓库所有 16 个历史 merged PR 均无 review 记录。
4. 当前 23 个开放 PR 全部来自 Dependabot、全部目标为 `main`，其中 12 个处于 `BLOCKED`；六个 weekly update 配置没有 grouping、并发上限、错峰或 `target-branch`。
5. Release workflow 在各平台矩阵成功后直接写入公开 Release，没有统一的 build/verify/publish 汇聚点；失败时可能暴露部分平台产物。仓库已有 `release` environment，但没有任何 job 使用它。

## 2. 本地开发与验证

### 已证实事实

- 根 `just ci` 在 `justfile:501-522` 串行执行 12 个阶段：`version-sync`、`fmt`、`fmt-check`、`lint-strict`、`check-workspace`、`test`、`release`、`audit`、`ci-governance-check`、`tauri-bindings-check`、`frontend-check`、`vscode-ci`。
- 该命令不是纯验证：`version-sync` 和 `fmt` 会修改受管版本文件、Rust 源码或 JSON 配置。失败后工作树可能混入校验产生的改动。
- `justfile:702,708,714,720` 的前端 type-check、lint、test、build 分别执行一次 `bun install --frozen-lockfile`；一次 `frontend-check` 因而重复四次同目录安装解析。
- 没有版本化 `CONTRIBUTING.md`、PR template、`CODEOWNERS`、pre-commit 配置或 pre-push 配置。`core.hooksPath` 指向仓库 `.git/hooks`，但其中没有活动的非 sample hook。
- 当前 `just version-check` 失败：代码与包版本均为 `7.1.1`，但 `ccr-ui/README.md:5` 仍是 `version-7.0.0`。
- 当前 `just workflow-governance-check` 通过：43 个 action 引用全部固定到完整 SHA，serial-only test 标注为 0，12 个治理单元测试通过。

### 体验与速度含义

- 完整门禁会主动修复后再验证，开发者难以区分“原始代码通过”与“门禁帮忙改过后通过”。
- 前端重复安装和 12 阶段全串行让本地全量门禁具有稳定但不必要的固定成本。
- 仓库已有 `version-check`、`fmt-check`、`frontend-check-quick`、surface 专用门禁，具备建立分层快速反馈的现成基础。

## 3. PR、分支保护与合并

### 分支与门禁

- `main` 和 `dev` 均为 protected branch，strict required checks 都是：
  - `Root Workspace Required`
  - `Vue and Docs Required`
  - `Tauri Linux Required`
  - `VS Code Required`
- `main` 的 `enforce_admins=true`，`dev` 的 `enforce_admins=false`。两者都禁止 force push 和 deletion；未启用 required review、conversation resolution 或 linear history。
- required context 分别由 `.github/workflows/ci.yml:127`、`frontend-ci.yml:73`、`tauri-rust-ci.yml:101`、`vscode-ci.yml:60` 的 always-run 聚合 job 提供。
- Root、Tauri、VS Code workflow 只监听 `pull_request`；Frontend 同时监听 `push` 和 `pull_request`。四个 workflow 都包含 `develop`，但远端和本地不存在活动 `develop` 分支。
- `origin/dev` 相对 `origin/main` 为 ahead 80、behind 1。当前没有开放的人类 PR。

### 历史 PR 事实

- 当前共有 16 个 merged PR（#2、#4、#5、#8、#11、#14、#18、#20、#22、#37-#43），GitHub Reviews API 返回的 review 数均为 0。
- PR #42 (`dev` -> `main`)：
  - 创建 `2026-07-27T07:43:28Z`，合并 `2026-07-27T15:28:33Z`，历时 7.75 小时。
  - 72 commits、605 changed files、+53,993/-7,274 lines、0 reviews。
  - 四个 workflow 各运行 7 次且没有旧运行取消：Root 83.3、Tauri 99.4、Frontend 30.7、VS Code 16.4 aggregate wall-minutes。
- PR #42 运行证据：
  - Root run IDs: `30278637116, 30278042469, 30277161602, 30252249630, 30250765237, 30247648787, 30247180765`
  - Tauri run IDs: `30278630016, 30278038717, 30277161545, 30252249641, 30250765242, 30247648793, 30247180751`
  - Frontend run IDs: `30278630472, 30278037927, 30277161697, 30252249690, 30250765252, 30247648808, 30247180717`
  - VS Code run IDs: `30278630237, 30278038950, 30277161603, 30252249627, 30250765260, 30247648794, 30247180758`

### Dependabot 事实

- `.github/dependabot.yml` 配置六个 weekly ecosystem：GitHub Actions、根 Cargo、Tauri Cargo、UI npm、VS Code npm、docs npm。
- 配置中没有 `groups`、`open-pull-requests-limit`、`target-branch` 或 schedule time/day 错峰。
- 2026-07-31 有 23 个开放 PR，全部作者为 Dependabot、全部 base 为 `main`；12 个 `mergeStateStatus=BLOCKED`。

## 4. CI 运行时与资源成本

### 样本定义

- 来源：GitHub Actions `gh run list --limit 200` 的只读结果。
- 区间：`createdAt >= 2026-07-27T00:00:00Z`，状态为 completed。
- 数量：136 runs，其中 12 个为 Dependabot Updates，124 个为四个产品 CI workflow。
- 时长：`updatedAt - createdAt`，包含排队和运行；P90 使用 nearest-rank。

| Workflow | Runs | Success / Failure | Median | P90 |
|---|---:|---:|---:|---:|
| Root CI | 29 | 23 / 6 | 11.78m | 20.15m |
| Tauri Rust CI | 29 | 26 / 3 | 14.58m | 26.85m |
| Frontend CI | 37 | 32 / 5 | 4.73m | 15.25m |
| VS Code CI | 29 | 27 / 2 | 6.10m | 14.67m |
| Dependabot Updates | 12 | 12 / 0 | 1.98m | 8.25m |

### 代表性 job/step 证据

Root run [`30277161602`](https://github.com/bahayonghang/ccr/actions/runs/30277161602)：

- Windows tests 505s，其中 tests 352s、安装 `just` 126s。
- macOS tests 416s，其中 tests 286s、安装 `just` 109s。
- coverage 362s，其中 coverage 执行 197s、安装工具 147s。
- security audit 142s，其中安装 `cargo-audit` 121s、实际 audit 5s。
- workspace validation 106s，其中安装工具 90s。

Tauri run [`30280315441`](https://github.com/bahayonghang/ccr/actions/runs/30280315441)：

- Linux validation 排队 733s、运行 791s；系统与仓库工具安装 285s，实际 Tauri gate 486s。
- coverage 排队 746s、运行 599s；工具安装 390s，coverage 189s。
- Windows smoke 排队 337s、运行 792s；安装 `just` 171s，smoke 594s。
- macOS smoke 排队 1,485s、运行 488s；安装 `just` 115s，smoke 355s。
- 该运行发生在 23 个 Dependabot PR 同时创建后的 runner 竞争期，排队时间不能直接归因于工作流代码。

Frontend run [`30277161697`](https://github.com/bahayonghang/ccr/actions/runs/30277161697)：验证 job 247s，其中安装 `just` 87s、本地门禁 98s、coverage 45s。

VS Code run [`30277161603`](https://github.com/bahayonghang/ccr/actions/runs/30277161603)：验证 job 128s，其中安装 `just` 82s、本地门禁 20s。

### 配置事实

- 五个 workflow 中均没有 `concurrency` 或 `timeout-minutes`。
- PR surface relevance 通过 `scripts/ci_surface_policy.py` 在 workflow 内判定，required aggregator 始终创建；这避免了 required workflow 被顶层 path filter 跳过后永久 Pending 的问题。
- Root/Tauri/Frontend/VS Code 多个 job 通过 `cargo install` 现场编译 `just`、`cargo-audit` 或 `cargo-llvm-cov`。
- Release workflow 对 registry、git 和 target 使用 Actions cache；PR workflows 没有等价的 Cargo dependency/build cache。

## 5. 发布、分发与部署

### 版本与历史

- `dev` 代码版本为 `7.1.1`，`main` 为 `7.0.0`。
- 最新 tag/GitHub Release 是 `v6.5.0`，发布于 2026-07-09；该 tag 是 `main` 的 ancestor。
- `CHANGELOG.md` 在 `Unreleased` 后的最新正式 section 是 `4.3.0`，没有为 5.x、6.x 或 7.x 建立正式 release section。

### Release workflow

- `.github/workflows/release.yml:3-6` 对任意 `v*` tag push 触发，没有 tag 格式、tag 与代码版本一致性、tag 是否来自 `main`、或对应 commit 是否已通过 required CI 的 preflight。
- workflow 顶层 `permissions: contents: write`（`:9-10`），因此所有 build job 都继承写权限。
- CLI 三平台 matrix 直接用 `softprops/action-gh-release` 发布，`draft: false`（`:96,154`）。
- Tauri 三平台 matrix 直接用 `tauri-action` 发布，`releaseDraft: false`（`:256,264`）。
- VS Code job 必须等待全部 CLI matrix 完成（`:163`），尽管采样中其核心 build/test/package 约数十秒，二者没有数据依赖。
- 发布矩阵 `fail-fast: false`，但没有一个最终 publish job 在全部 artifacts、checksums、smoke 都成功后再一次性公开 Release。
- 没有 `concurrency`、`timeout-minutes`、artifact attestation、SBOM、OS/VSIX publisher signing、macOS notarization 或安装 smoke。
- Tauri config 的 macOS `signingIdentity=null`、Windows `certificateThumbprint=null`，没有 updater plugin/config；Release 文案明确说明 SHA256 只能证明完整性，不能证明发布者身份，自动更新保持禁用。

### Environment 与 deployment

- GitHub 有一个名为 `release` 的 environment，允许自定义 tag policy `v*`，管理员可 bypass。
- 没有 workflow job 声明 `environment: release`，因此其保护规则和 environment-scoped secrets 没有参与发布。
- GitHub Deployments API 返回空数组；Pages API 返回 404。当前可观察的交付终点是 GitHub Release，未发现 Marketplace、包管理器、文档站点或 updater 的自动发布记录。

## 6. 证据限制

- Actions 时长样本高度受 2026-07-27 同时生成的 Dependabot PR 影响；报告必须把 queue time 与 execution time 分开解释，不能把 P90 全部视为编译性能。
- 远端 branch protection 只能证明当前设置，不能证明过去每次 push 都经过相同规则。
- GitHub 没有可用 deployment history，不能计算真实的 deployment frequency、change failure rate 或 recovery time。
- 没有团队评审 SLA、发布支持矩阵、Marketplace 账号或签名证书证据；相关建议只能标注为条件性路线图。
- 本次不运行完整 `just ci`：它会执行 mutating recipes，违反只读审计约束。仅运行了 `just version-check` 和 `just workflow-governance-check`。

## 7. 后续综合时必须回答的问题

1. 哪些改进可以直接降低首个可行动反馈时间，而不降低 required gate 覆盖？
2. `dev` 的管理员直推是否应保留；若保留，怎样保证 push 后仍获得完整门禁？
3. Dependabot 应以 `dev` 还是 `main` 为目标，如何 grouping/limit 才与当前分支模型一致？
4. 发布流程怎样拆成 build -> verify -> controlled publish，避免部分公开 release？
5. 哪些行业实践当前不具正 ROI，例如 merge queue、自托管 runner、强制 CODEOWNERS 或 updater？

