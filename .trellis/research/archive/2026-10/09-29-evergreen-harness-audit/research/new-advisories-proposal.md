# 审计结果变化后的六节点补丁方案

证据时间：2026-09-30 03:33–03:38 UTC。方案初稿由只读诊断产生；以下补丁尚未由本方案文件应用。

## 结论与根因边界

完整 `just ci` 在第 14 项 Frontend Audit 失败，前 13 项通过；第 15、16 项未执行。父回执 `final-ci.json` 记录 2509 个源文件 SHA 未变。此次查询复现 UI 7 条公告记录，涉及 4 个唯一 GHSA ID；扩展 npm audit 报告 2 个受影响包；docs Bun audit 返回 `{}`。

即时失败原因是现有拒绝未批准公告的审计策略收到 7 条命中。相同 lock SHA 的早先审计返回 0。官方公告时间早于早先本地成功结果，不能将输出变化直接归因于新发布时间；数据传播、缓存或其他上游原因未查明。保留旧 PASS、完整聚合失败和本次查询作为不同时间的证据。

## 官方公告

各行直接取自对应 `new-advisories-GHSA-*.json` 的 GitHub 官方 API 原文。HTTP 200、抓取时间、Date、ETag 和文件 SHA 位于 `new-advisories-capture.json`。

| GHSA | 严重性 | published_at / github_reviewed_at（UTC） | updated_at（UTC） | 涉及本仓库主版本的补丁下界 |
| --- | --- | --- | --- | --- |
| GHSA-6j4f-fj2g-mc7p | high | 2026-09-29 23:44:58 | 2026-09-29 23:44:59 | brace 1.1.19、5.0.10 |
| GHSA-qhr7-859c-m2p7 | high | 2026-09-29 23:45:17 | 2026-09-29 23:45:19 | brace 1.1.20、5.0.11 |
| GHSA-q2hr-2g5m-vwhr | medium（npm moderate） | 2026-09-29 23:45:39 | 2026-09-29 23:45:39 | brace 1.1.21、5.0.12 |
| GHSA-hrr3-gc8f-f4qj | medium（npm moderate） | 2026-09-29 23:54:25 | 2026-09-29 23:54:26 | fast-uri 3.1.8 |

brace 三项公告分别包含 `<1.1.19`、`<1.1.20`、`<1.1.21` 以及 `>=4.0.0 <5.0.10`、`>=4.0.0 <5.0.11`、`>=4.0.0 <5.0.12`。fast-uri 的相关范围是 `>=3.0.0 <3.1.8`。其他主版本范围保存在 API 原文和方案 JSON；当前三个锁没有 brace 2.x/3.x 节点，不增加这些主版本的更新。

## 最小改动与授权归属

只修改两个 lockfile 的六个解析节点。所有候选均满足当前解析父节点的 semver，且不落入四项官方公告的任何匹配范围。`package.json`、依赖声明、例外策略及生产源码不需要改动。

| 文件 | 精确节点 | 当前 → 候选 | 当前直接父节点 / 范围 | 范围归属 |
| --- | --- | --- | --- | --- |
| ccr-ui/bun.lock | brace-expansion | 1.1.18 → 1.1.21 | minimatch 3.1.5 / `^1.1.7` | T08 既有动态安全修复范围 |
| ccr-ui/bun.lock | fast-uri | 3.1.7 → 3.1.8 | table/ajv 8.20.0 / `^3.0.1` | T08 原节点兼容补丁 |
| ccr-ui/bun.lock | @typescript-eslint/typescript-estree/minimatch/brace-expansion | 5.0.9 → 5.0.12 | 对应 minimatch 10.2.5 / `^5.0.5` | T08 既有动态安全修复范围 |
| ccr-ui/bun.lock | glob/minimatch/brace-expansion | 5.0.9 → 5.0.12 | glob/minimatch 10.2.5 / `^5.0.5` | T08 既有动态安全修复范围 |
| ccr-vscode/package-lock.json | node_modules/brace-expansion | 5.0.9 → 5.0.12 | minimatch 10.2.6 / `^5.0.8` | 新增扩展包范围，需追加批准 |
| ccr-vscode/package-lock.json | node_modules/fast-uri | 3.1.7 → 3.1.8 | ajv 8.18.0 / `^3.0.1` | 同一原节点；原追加记录明确 3.1.7，由主会话纳入本次确认 |

UI 共 743 个解析节点；扩展共 327 个 `packages` 条目（含根条目）。docs 共 179 个解析节点，没有 brace-expansion、fast-uri 或声明两包的直接父节点，当前审计退出 0。因此无需改 `docs/bun.lock`，无需请求新增 docs 文件范围。

UI 的目标包来自 ESLint/TypeScript-ESLint、glob、Stylelint/table/AJV 等开发工具链。扩展两包位于 `@vscode/vsce` 打包工具链，lock 标记 `dev: true`。依赖链原始输出为 `new-advisories-ui-brace-chain.log`、`new-advisories-ui-fast-chain.log` 和 `new-advisories-extension-chain.log`。这些依赖证据没有证明生产发行包存在可利用攻击路径。

## 候选版本完整性

从官方 npm registry 的实际版本全集选择当前主版本内满足全部父范围和公告约束的最低稳定版本。未用未经核实的版本猜测决定候选。完整旧/新元数据、发布时间和父范围位于 `new-advisories-version-proposal.json`；三个唯一 tarball 均已下载后计算 SHA-512，并与 registry integrity 一致。tarball 内 `package.json` 的包名、版本和 dependencies 已核对。新旧 dependencies 与 Node engines 均保持一致。

| 候选 | 发布日期（UTC） | 下载字节 | SHA-512 校验 |
| --- | --- | --- | --- |
| brace-expansion 1.1.21 | 2026-09-14 21:59:28.332 | 7924 | 匹配；完整值见 candidate-verification.json |
| brace-expansion 5.0.12 | 2026-09-14 21:59:00.288 | 12620 | 匹配；完整值见 candidate-verification.json |
| fast-uri 3.1.8 | 2026-09-15 07:36:25.444 | 44269 | 匹配；完整值见 candidate-verification.json |

2026-09-30 03:38:23 UTC，官方 npm bulk advisory 接口对三个候选返回 `{}`，原始响应保存在 `new-advisories-candidate-bulk-response.json`。该时点查询不替代批准后对实际交付锁文件执行的审计。

## 实施顺序与检查

1. 强模型确认公告范围、六个节点及授权边界；执行模型只接受批准的节点列表。五工具 Claude Code、Codex、Grok Build、Kimi Code、OMP 使用同一节点、政策和验收定义。扩展新范围未批准时只处理 UI。
2. 保存修改前 lock SHA 和解析节点快照；只应用 `new-advisories-proposed-locks.patch` 中已批准的文件部分。UI 仅改 4 条目的版本/integrity；扩展仅改两条目的 version/resolved/integrity。保持父 semver、包计数和其他字段，禁止 `audit fix`、ignore、override、阈值变化或全量更新。
3. UI：`bun install --frozen-lockfile`，检查安装前后 lock SHA；`just frontend-audit` 必须为 0 advisory；运行现有 audit policy 3 项回归；`just frontend-check` 覆盖类型、lint、i18n、904 项 smoke、架构、构建与 docs；`just frontend-coverage` 保持 70% 门槛。构建使用已提交图标。
4. 扩展在追加批准后：`npm ci`，检查 lock SHA；`npm audit --json`；`just vscode-ci` 覆盖类型、测试、打包和最终 VSIX 清单；`just vscode-coverage` 保持现有 70% 门槛。更新必须保留已有 undici 7.29.1 和打包允许范围。
5. 定向 `git diff --check`、解析节点精确 diff、全仓库源文件哈希；两表修复及独立复核完成后由主会话重新执行完整 `just ci`。保留首次完整失败记录。此前 Rust/Tauri coverage、MSRV、原生客户端、托管运行证据继续分别报告，不用新前端检查替代。

## 诊断写入边界

本诊断只新增父任务 `research/new-advisories-*` 证据和可审阅 patch。六个 manifest/lock 的诊断前后 SHA 一致，详见 `new-advisories-candidate-verification.json`。没有应用补丁、改锁、安装依赖、改例外、降低检查、提交、归档或推送。方案完成后 UI 的实施授权由主会话按 T08 原批准范围单独分派；扩展继续等待确认。
