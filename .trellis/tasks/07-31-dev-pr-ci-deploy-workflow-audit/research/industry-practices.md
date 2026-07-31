# Research: 开发、PR、CI、合并与部署流程的行业实践

- Query: 如何在不牺牲正确性与供应链安全的前提下，改善本地开发、Pull Request、CI、合并队列、Rust/Tauri 发布与部署的体验和速度？
- Scope: external；仅研究官方/一手资料，不评价当前仓库实现
- Date: 2026-07-31

## 结论摘要

行业里更有效的做法不是寻找一个“最快的总 CI”，而是建立分层反馈和单向晋级：

```text
本地受影响检查 -> PR 快速必需检查 -> 目标分支集成验证 ->
同一提交的原生平台构建/签名 -> 同一批产物的受控发布 -> 发布后验证与度量
```

这套模型同时优化两件事：开发者尽早得到高信号反馈，发布产物则经过更强但频率更低的检查。DORA 的当前模型用变更前置时间、部署频率、失败部署恢复时间、变更失败率和部署返工率衡量吞吐与不稳定性，并明确指出速度与稳定性通常不是此消彼长；指标应按单个应用或服务理解，不能脱离上下文横向排名（[DORA 软件交付绩效指标](https://dora.dev/guides/dora-metrics-four-keys/)）。

## 1. 度量先于优化

### 行业基线

DORA 当前五项指标可映射到桌面/CLI 项目：

| 指标 | 桌面/CLI 项目的可操作定义 |
|---|---|
| 变更前置时间 | 提交进入版本控制到 GitHub Release/包管理器/更新元数据对用户可用 |
| 部署频率 | 对外发布稳定版、预览版或扩展版本的频率，渠道要分开统计 |
| 失败部署恢复时间 | 发布失败或回归确认后，到修复版、撤回或安全降级完成的时间 |
| 变更失败率 | 需要热修、撤回、回滚或人工干预的发布占比 |
| 部署返工率 | 因线上/发布事故产生的计划外发布占比 |

GitHub Actions 已提供仓库级和组织级性能指标，包括平均运行时间、平均排队时间和失败率，可直接用来定位慢工作流、慢 job 和不稳定 job（[GitHub Actions metrics](https://docs.github.com/en/actions/concepts/metrics)，[查看 metrics](https://docs.github.com/en/actions/how-tos/administer/view-metrics)）。

建议额外记录以下工程体验指标：

- PR 首次可行动反馈时间：从 push 到第一个失败或全部快速检查完成。
- PR 全部门禁时间：从 push 到所有 required checks 完成。
- 排队时间与执行时间分开统计；前者反映 runner 容量，后者反映工作流本身。
- 取消率、缓存命中率、失败后重跑转绿率（flake 代理指标）、按 job 的 P50/P95。
- PR 首次评审响应时间、从 ready-for-review 到 merge 的周期。
- 发布矩阵完成率、签名/公证成功率、发布后安装和 updater smoke 成功率。

这些阈值没有跨项目通用答案。可以先以最近 30 天为基线，再为 CCR 自行设定 P50/P95 SLO；不应把任意“5 分钟/10 分钟”包装成行业标准。

### 不应照搬

- 不用 DORA 指标考核个人或比较技术栈不同的团队。DORA 明确要求按应用/服务上下文使用。
- 不把“总分钟数下降”当作唯一目标；取消有效测试也会让时间下降，却会提高失败率和返工率。
- 不把排队时间算进编译优化问题，也不把 flaky rerun 当作正常成功。

## 2. 本地开发：同一入口，按影响面递进

### 推荐实践

1. 为本地和 CI 使用同一组版本化命令入口，只改变执行层级，不复制检查逻辑。开发者先跑受影响模块，再按风险升级到全仓检查；CI 调用同一入口，减少“本地绿、CI 红”的漂移。
2. Rust 快速反馈优先使用 `cargo check`。Cargo 官方说明它跳过最终代码生成，因此比 `cargo build` 快，但也提醒部分错误只会在代码生成阶段出现，所以它只能是早期检查，不能替代测试和发布构建（[`cargo check`](https://doc.rust-lang.org/cargo/commands/cargo-check.html)）。
3. 把格式、静态检查、类型检查和窄测试放在提交前/PR 快速层；完整测试、跨平台构建、安全审计按影响面或在后续层执行。
4. 用 Cargo profile 的默认语义而非盲目统一 profile：`dev` 默认启用增量编译，`release` 默认关闭；增量信息会占用 `target` 空间，只适合能复用工作目录的开发场景（[Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html#incremental)）。
5. 对慢 Rust 构建周期性生成 `cargo --timings` 报告。Cargo 会输出 crate 编译时间和并发信息，适合找 proc-macro、重复特性组合和串行瓶颈；它是人工分析报告，不是机器指标接口（[`cargo build --timings`](https://doc.rust-lang.org/cargo/commands/cargo-build.html)）。

### 适用条件

- “只跑受影响模块”适合模块边界和依赖关系清晰的 monorepo；共享 crate、构建脚本、锁文件、版本文件或 CI 配置变化必须升级到更广检查。
- 本地 hook 应短、确定、可显式重跑；较慢检查放到 pre-push、PR 或手动命令，避免开发者绕过所有 hook。

### 反模式

- 每次保存或提交都运行完整跨平台/发布构建。
- 本地脚本与 CI YAML 各自维护一套命令和参数。
- 用 `cargo check` 通过声称二进制、链接、测试或 Tauri bundle 已验证。
- 为提升本地速度永久关闭必要的锁文件、版本同步或生成物一致性检查。

## 3. PR：小批量、快响应、稳定门禁

### 推荐实践

1. 一个 PR/CL 尽量只包含一个自洽变更，并带相关测试。Google 工程实践总结，小变更更快、更仔细地被评审，也更容易合并、回滚和发现缺陷；“约 100 行通常合理、1000 行通常过大”只是经验量级，不是机械上限（[Small CLs](https://google.github.io/eng-practices/review/developer/small-cls.html)）。生成文件、机械迁移和依赖锁文件应单独解释，而不是按行数硬拆。
2. 以团队吞吐而非单个开发者忙碌度优化评审。Google 的一手实践把一个工作日定义为首次响应上限，并强调应在专注工作自然断点处理评审，而非持续打断（[Speed of Code Reviews](https://google.github.io/eng-practices/review/reviewer/speed.html)）。
3. 对主分支启用规则集/保护：PR 审批、required status checks、对话解决、禁止强推/删除；高风险目录（`.github/workflows`、发布脚本、签名/更新配置）由 `CODEOWNERS` 审批。GitHub 支持 required reviews、required checks、conversation resolution、merge queue 和禁止绕过等组合（[Protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)，[Rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)）。
4. required check 名称保持稳定，推荐一个始终上报结果的聚合门禁；受影响模块判断放在工作流内部。GitHub 明确说明：如果 required workflow 因路径/分支过滤或 skip 指令没有运行，对应检查会一直处于 `Pending` 并阻塞合并（[Skipping workflow runs](https://docs.github.com/en/actions/managing-workflow-runs-and-deployments/managing-workflow-runs/skipping-workflow-runs)）。
5. 对文档、依赖更新、代码和发布配置采用不同审批强度，但不要允许通过改文件路径绕过共享代码或发布安全检查。

### 严格检查与 merge queue 的选择

GitHub 的 strict required checks 要求 PR 分支与 base branch 保持最新，正确性更强，但每次 base 更新都可能重复构建；loose checks 减少构建，但可能让不兼容变更进入主分支（[Rulesets 的 strict/loose 说明](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/available-rules-for-rulesets)）。选择原则是：

- 低并发、小团队：保护分支 + required checks + auto-merge 通常足够，避免引入队列等待和重复 merge-group 构建。
- 主分支每天有多个并发 PR、经常因 base 更新重跑或合并后冲突：使用 merge queue，让 GitHub 在最新 base 和队列前序 PR 的组合上验证，不再要求作者手工更新分支（[Managing a merge queue](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)）。
- 启用 merge queue 时，所有 required GitHub Actions workflow 必须监听 `merge_group`；否则检查不会触发，队列合并会失败。队列的 build concurrency 和合并批量需要按 runner 容量和发布触发方式调节。

### 反模式

- 仅按 PR 行数自动拒绝，不看生成物、重命名和变更自洽性。
- 把所有 job 合并成一个不可诊断的 required check，或把几十个易改名的 job 全部直接设为 required。
- required workflow 使用顶层 `paths`/`paths-ignore` 后，依赖人工管理员绕过 `Pending`。
- 低 PR 流量仓库为“看起来先进”启用 merge queue，却不量化节省的重跑与新增等待。

## 4. CI：取消过期工作，精确缓存，并行但可诊断

### 推荐实践

1. 对同一 PR/分支的验证使用 `concurrency`，新 push 取消旧的 in-progress run。GitHub 支持每个 concurrency group 至多一个运行实例，并用 `cancel-in-progress: true` 取消旧实例（[Workflow syntax: concurrency](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#concurrency)）。发布、迁移和 production deployment 通常不可取消，应使用单独 group 排队。
2. 并行独立 job，串行真正有数据依赖的 job。矩阵用 `fail-fast` 快速停止非实验组合，用 `continue-on-error` 标记实验组合；必要时用 `max-parallel` 控制 runner、外部 API 或签名服务压力（[Matrix failure handling](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs#handling-failures)）。
3. 把通用安装、检查和发布合同收敛到 reusable workflow。GitHub 的 `workflow_call` 支持类型化输入、显式 secrets 和嵌套调用，而且嵌套链权限只能保持或降低，不能提升（[Reusing workflows](https://docs.github.com/en/actions/sharing-automations/reusing-workflows)）。
4. 缓存依赖下载与可安全再生成的构建中间物，key 至少包含 OS、工具链/目标、锁文件和影响编译输出的特性；缓存 miss 必须仍能正确构建。GitHub cache 支持精确 key 和 `restore-keys`，同时警告具有仓库读取/PR 能力的人可能访问 base branch cache，因此缓存中禁止放 token、证书和凭据（[Dependency caching](https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/caching-dependencies-to-speed-up-workflows)）。
5. Rust 不应无条件缓存整个 `$CARGO_HOME`。Cargo 官方指出整个目录会同时保存压缩包和解压源码，重新压缩/上传可能反而拖慢 CI；官方建议按需缓存 `registry/index`、`registry/cache`、`git/db`，以及确需复用的已安装工具元数据（[Caching Cargo home in CI](https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci)）。
6. CI 和发布构建使用 `--locked`；Cargo 会在缺少 `Cargo.lock` 或解析需要改写锁文件时失败，官方明确把 CI 的确定性构建列为使用场景（[`cargo build --locked`](https://doc.rust-lang.org/cargo/commands/cargo-build.html)）。
7. 失败诊断产物（测试报告、截图、日志、timings）短期保留，发布产物单独保留。GitHub 区分 cache 与 artifact：cache 用于可再生成依赖，artifact 用于 build/test 输出及 job 间传递（[Workflow artifacts](https://docs.github.com/en/actions/concepts/workflows-and-actions/workflow-artifacts)）。

### 建议的分层门禁

| 层级 | 触发 | 目标 | 常见内容 |
|---|---|---|---|
| Fast PR | 每次 PR push | 最快产生可行动结果 | 格式、版本一致性、lint/type-check、受影响测试、依赖策略 |
| Required PR | Fast 后并行或同时 | 合并安全 | Rust workspace tests、前端/扩展检查、必要平台 smoke、聚合状态 |
| Merge group/main | 进入队列或合并后 | 验证组合状态 | 共享模块全量、集成/文档构建；避免与 PR 层无目的全量重复 |
| Scheduled | 夜间/周期 | 找低频风险 | 全平台/全 feature、长测试、审计刷新、flake 与性能趋势 |
| Release | 版本 tag/人工批准 | 可分发产物 | release profile、原生平台 bundle、签名、公证、SBOM、attestation、安装 smoke |

### 反模式

- PR、`merge_group`、main push 和 release 对同一提交无差别重复完整 CI，却没有记录各层防范的风险。
- 所有矩阵都设置 `fail-fast: false`，导致一个确定失败后继续消耗大量 runner；发布矩阵需要收集全部平台结果时才适合关闭。
- 缓存 key 只含分支名，或在不可信 PR 与发布构建间共享可写构建缓存。SLSA Build L3 明确要求构建不能污染后续构建使用的 cache，且有无 cache 的输出应一致（[SLSA v1.2 Build requirements](https://slsa.dev/spec/v1.2/build-requirements)）。
- 未先测量 queue time/runtime 就迁移 self-hosted runner；这会引入补丁、隔离、密钥和容量运维责任。

## 5. Rust/Tauri 发布：原生构建、签名、一次构建多处晋级

### 推荐实践

1. 发布工作流由明确的版本 tag 或受控手动触发，先验证 tag 指向已通过门禁的提交，再构建 draft release；全部矩阵、签名和 smoke 成功后才发布。Tauri 官方 GitHub pipeline 使用原生 OS 矩阵生成 Windows、Linux、macOS 多架构产物，并建议 release draft 作为发布前状态（[Tauri GitHub pipeline](https://v2.tauri.app/distribute/pipelines/github/)）。
2. 每个平台在其可信原生 runner 上构建和签名。Tauri 说明多数平台要求签名；Windows 签名可避免浏览器下载后的 SmartScreen 未信任提示，macOS 对浏览器分发需要签名并通常需要 notarization（[Tauri distribution](https://v2.tauri.app/distribute/)，[Windows signing](https://v2.tauri.app/distribute/sign/windows/)，[macOS signing](https://v2.tauri.app/distribute/sign/macos/)）。
3. 构建 job 上传命名清晰的 artifact，验证 job、签名/发布 job 下载同一份产物，而不是在每个渠道重新编译。GitHub artifact 可在 job 间传递 build/test 输出；artifact attestation 还能把产物关联到 workflow、仓库、commit SHA 和触发事件（[Store and share artifacts](https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts)，[Workflow artifact attestations](https://docs.github.com/en/actions/concepts/workflows-and-actions/workflow-artifacts)）。
4. 若启用 Tauri updater，私钥只存在发布环境，公钥嵌入应用；updater 签名校验不可关闭，生产默认强制 TLS。丢失私钥会失去继续发布更新的能力，因此必须有受控备份、轮换和恢复演练（[Tauri updater](https://v2.tauri.app/plugin/updater/)）。
5. 将稳定、预览/候选和开发渠道的触发、版本约束、更新元数据和保留策略分开。每个渠道都应能从 release/tag 追到 commit、CI run、各平台 artifact、签名和发布状态。
6. 对真实“部署”使用 GitHub Environment：限制允许部署的 branch/tag，要求审批，按环境控制 secrets，并用 concurrency 保证同一渠道同一时刻只有一个发布。Environment job 在保护规则通过前不能访问环境 secrets（[Managing deployment environments](https://docs.github.com/en/actions/deployment/targeting-different-environments/managing-environments-for-deployment)，[Deploying with GitHub Actions](https://docs.github.com/en/actions/concepts/use-cases/deploying-with-github-actions)）。

### 适用条件

- 桌面应用的发布产物通常必须按 OS/架构分别生成；纯 CLI 是否需要全部平台每个 PR 构建，应由平台特定代码和历史故障决定。
- “一次构建多处晋级”是指同一 commit、同一平台、同一配置的已验证产物；平台签名工具要求签名发生在 bundle 阶段时，应把签名纳入该平台唯一的受控 release build，而不是事后跨平台重建。
- 人工批准适合稳定发布、商店提交和签名密钥使用；预览渠道可自动化，但仍要保留完整可追溯性。

### 反模式

- 在 PR workflow 中暴露签名证书、notarization 凭据或 updater 私钥。
- 每个平台成功后立即独立公开 release，造成用户看到不完整或版本不一致的资产集。
- 发布时重新 checkout 浮动分支而不是固定 tag/commit SHA。
- 用非 HTTPS updater endpoint，或把 updater 私钥与公钥一起提交。
- 只验证“文件上传成功”，不做签名验证、安装启动和 updater metadata smoke。

## 6. 供应链安全：最小权限、不可变引用、可验证来源

### 推荐实践

1. 工作流顶层默认 `permissions: contents: read`，仅在具体 job 提升所需权限。GitHub 提醒 action 即使未显式接收 secret，也能通过 `github.token` 访问 `GITHUB_TOKEN`，因此必须限制最小权限（[`GITHUB_TOKEN` authentication](https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication)）。
2. 第三方 action 固定到完整 commit SHA，并由 Dependabot/Renovate 类工具维护更新；GitHub 明确说明完整 SHA 是当前将 action 用作不可变 release 的唯一方式。工作流目录使用 `CODEOWNERS`（[Security hardening for GitHub Actions](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions)）。
3. 访问云签名、制品库或部署平台时优先用 OIDC 换取短期凭据，不保存长期云密钥；同时在云端按 repo、ref、environment 等 claim 限制信任范围（[GitHub OIDC](https://docs.github.com/en/actions/security-for-github-actions/security-hardening-your-deployments/configuring-openid-connect-in-cloud-providers)）。
4. 为 release artifact 生成 provenance attestation，并在可行时生成 SBOM；消费者或发布 job 应实际验证 attestation，而不只是生成。GitHub artifact attestation 支持 provenance 与 SBOM，但私有/内部仓库的可用性受 GitHub plan 限制（[GitHub artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds)）。
5. 以 SLSA 渐进落地：L1 要求 provenance 存在；L2 要求 hosted build platform 和可验证的真实性；L3 进一步要求不可伪造 provenance 与隔离构建。不要只贴“SLSA compliant”标签，应说明 builder、level、验证者和失败策略（[SLSA v1.2](https://slsa.dev/spec/v1.2/)，[Build requirements](https://slsa.dev/spec/v1.2/build-requirements)，[Provenance](https://slsa.dev/spec/v1.2/provenance)）。
6. Rust 依赖治理分层：RustSec `cargo-audit` 检查 `Cargo.lock` 中已知漏洞；`cargo-deny` 可统一检查 advisories、许可证、重复/禁止依赖和来源策略（[RustSec](https://rustsec.org/)，[`cargo-deny`](https://embarkstudios.github.io/cargo-deny/)）。GitHub dependency review 可在 PR 引入依赖前显示漏洞和许可证影响，但功能可用性也受仓库类型/套餐限制（[Dependency review](https://docs.github.com/en/code-security/supply-chain-security/understanding-your-software-supply-chain/about-dependency-review)）。
7. 用 OpenSSF Scorecard 作为发现工具而不是单一合规分数，重点看 `Pinned-Dependencies`、`Token-Permissions`、`Dangerous-Workflow`、`Binary-Artifacts`、`Signed-Releases` 等具体检查及修复建议（[Scorecard](https://scorecard.dev/)，[checks](https://github.com/ossf/scorecard/blob/main/docs/checks.md)）。

### 反模式

- action 仅固定到可变 tag，然后因为“Verified creator”而视为不可变。
- 所有 job 共享 `contents: write`、`id-token: write` 或 release secret。
- 在 `pull_request_target` 上 checkout/执行不可信 PR 代码，或把 PR 标题、分支名直接插入 shell。
- 只上传 `.sig`/attestation，不验证其主体 digest、builder identity、source repo 和 commit。
- 认为 SBOM、漏洞扫描、代码签名和 provenance 可以互相替代；它们覆盖的是不同风险。

## 7. 发布/部署可观测性与恢复

### 推荐实践

1. 每次发布保留结构化记录：版本、commit/tag、触发者、workflow run、各平台 digest、签名/公证、SBOM/attestation、渠道、开始/结束时间和最终状态。
2. 使用 GitHub Deployment/Environment 记录渠道或发布目标，并写入 `queued`、`in_progress`、`success`、`failure` 等状态及日志 URL。GitHub deployment history 能关联 environment、commit、PR/branch、workflow logs 和部署 URL（[Viewing deployment history](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/view-deployment-history)，[Deployment statuses API](https://docs.github.com/en/rest/deployments/statuses)）。
3. 发布完成后执行最低成本的真实 smoke：校验下载 digest/attestation、验证 OS 签名、干净环境安装、启动/版本输出、updater 查询；失败应自动把 deployment 标记为 failure，并给出可执行日志。
4. 预先定义恢复动作：停止推广/撤下更新元数据、恢复上一已验证渠道状态、发布修复版本。保留上一稳定版本及其签名、元数据和验证证据；Tauri updater 的自定义版本比较可支持回退，但这会改变通常的单调升级安全假设，应单独设计和测试（[Tauri updater dynamic server](https://v2.tauri.app/plugin/updater/)）。
5. 每月查看 Actions 性能趋势和 DORA 指标，每次失败发布做轻量复盘；优化应能说明改善了哪个等待、失败或恢复指标。

### 反模式

- GitHub Release 显示为 published 就视为所有平台部署成功。
- 只保留最终绿灯，不保留失败 job、签名、公证和 smoke 的日志链接。
- 回滚时从旧 source tag 重新构建，而不是恢复已验证的旧产物；新构建已经是不同供应链事件。
- 事故恢复依赖某个人电脑上的证书、私钥、缓存或未版本化脚本。

## 8. 推荐落地顺序

以下顺序强调先获得数据和低风险收益，再增加治理复杂度：

| 顺序 | 动作 | 收益 | 启用条件/退出条件 |
|---|---|---|---|
| 1 | 建立 Actions runtime/queue/failure 基线和 PR 反馈指标 | 找到真实瓶颈 | 至少观察 2-4 周或足够样本，不凭单次 run 决策 |
| 2 | PR concurrency 取消过期 run；分层 fast/required gate | 直接减少等待和浪费 | release/main job 使用不同 concurrency 策略 |
| 3 | 精确 cache key、选择性 Cargo cache、`--locked` | 减少下载/编译抖动 | 比较 cold/warm P50/P95；缓存上传时间大于节省时撤回 |
| 4 | required 聚合门禁，消除顶层 path-filter `Pending` 风险 | 合并体验稳定 | 聚合 job 必须能识别被取消/失败的必要 job |
| 5 | 规则集、workflow CODEOWNERS、最小 token 权限、action SHA pin | 降低治理与供应链风险 | 先确认紧急修复和管理员 break-glass 流程 |
| 6 | 按主分支实际流量评估 merge queue | 减少手工 rebase 与合并后破坏 | 只有 strict 重跑/并发冲突已成为可测瓶颈才启用 |
| 7 | Tauri 原生 release matrix、draft 汇聚、签名/公证、安装 smoke | 提升发布完整性 | 所有目标平台和架构有明确支持矩阵 |
| 8 | build-once/publish-same-artifact、attestation、SBOM、Environment | 提升可追溯和恢复能力 | 发布消费者或自动化必须实际验证 provenance |

## Files Found

- `.trellis/workflow.md`：仅用于确认研究结果必须持久化到任务目录；未用于评价产品流程。
- 未读取产品代码、工作流配置、任务规划文件或 package spec；当前文件是供主审计与仓库事实交叉比对的外部行业基线。

## Related Specs

- 本子任务按派发边界未读取 `.trellis/spec/`。任何仓库内落地建议都必须由主审计依据当前 CI、发布配置、套餐权限、PR 流量和历史时长重新判定。

## Caveats / Not Found

- 官方文档于 2026-07-31 在线核验。GitHub Actions、merge queue、Environment、artifact attestation、dependency review 和 ARM runner 的可用性与计费受 public/private、组织类型和 GitHub plan 影响，实施前需核对当前仓库权限。
- DORA 给出指标定义和研究结论，不给单个仓库通用的 CI/PR 时长 SLO；本文没有把建议目标冒充行业基准。
- Tauri 官方 workflow 示例会随 action 主版本和 runner 镜像变化；实际实施时应采用当前受支持版本，同时遵守 GitHub 对第三方 action 完整 SHA 固定的更强安全要求。
- 本报告未测量 CCR 当前 workflow 的耗时、缓存命中、flake、PR 流量、发布失败或套餐能力，因此不能单独证明 merge queue、自托管 runner、更大 runner 或任一缓存方案具有正 ROI。
- 桌面应用没有传统服务器“部署”边界。正式审计需先约定 GitHub Release、包管理器发布、VS Code Marketplace、文档站点和 Tauri updater 各自何时计为 deployment。
