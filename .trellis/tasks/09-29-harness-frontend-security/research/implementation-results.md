# T08 实施与验证记录

审查日期：2026-09-29，America/Chicago。命令回执使用 UTC，跨入 2026-09-30。
基线：`fe9d65972376dbb4c332fde9860c1e42ff2f2005`。执行环境见 [environment.json](evidence/environment.json)。

## 当前结果

`ccr-ui/bun.lock` 仅变更四个包的版本和 integrity。`ccr-ui/package.json` 未修改。
`just frontend-audit` 从 18 条未批准公告变为 0 条；`maxActiveExceptions=0` 和空 exceptions 保持不变。
冻结安装通过且 lockfile SHA-256 未变化。审计策略回归 3/3 通过。完整前端检查通过：24 项 i18n、169 文件/904 项 smoke、类型、lint、循环依赖、架构边界、UI 构建和文档检查。覆盖率检查再次通过 904 项测试，行覆盖率为 75.19%（13081/17395），门槛保持 70%。

## 版本与依赖边界

| 包 | 旧版本 | 新版本 | 已确认依赖路径 | 上级声明范围 | 边界 |
| --- | --- | --- | --- | --- | --- |
| colord | 2.9.3 | 2.9.4 | dev stylelint 17.14.1 → colord | `^2.9.3` | CSS lint 工具 |
| fast-uri | 3.1.5 | 3.1.7 | dev stylelint 17.14.1 → table 6.9.0 → ajv 8.20.0 → fast-uri | `^3.0.1` | CSS lint 工具 |
| js-yaml | 4.3.1 | 4.3.2 | dev eslint 9.39.5 → @eslint/eslintrc 3.3.6；dev stylelint 17.14.1 → cosmiconfig 9.0.2 | `^4.3.0`、`^4.1.0` | lint 配置解析 |
| undici | 8.10.0 | 8.10.2 | dev jsdom 30.0.1 → undici；Vitest 使用 jsdom 环境 | `^8.9.0` | 测试工具 HTTP 实现 |

路径通过 `cd ccr-ui; bun pm why <package>` 与 lockfile 交叉核对。上述证据没有证明发行版存在可利用攻击路径。没有变更运行时直接依赖、例外政策、worker 设置或覆盖率阈值。

四项均为现有 major/minor 内的补丁更新。全部上级 semver 范围匹配；各包依赖元数据保持不变。undici 的 Node engine 在更新前后均为 `>=22.19.0`，涵盖声明的 hosted Node 24.20.0；本地成功不能代替 hosted Node 运行证据。

fast-uri 3.1.6 仍落入 `GHSA-qw65-cvwx-89v3` 的 `<3.1.7` 范围，故选 3.1.7。没有以先前抽样公告的最低补丁代替全部公告核对。

## 校验和与官方来源

- [dependency-changes.json](evidence/dependency-changes.json)：每个包的旧/新版本、旧/新完整 SHA-512 integrity、tarball URL、下载字节数、实际计算值、发布时间、父依赖范围。
- 四份 `*-selected-releases.json`：从 `https://registry.npmjs.org/<package>` 取得的旧/新原始版本元数据。旧 integrity 与原锁定值逐项匹配；四个新 tarball 均下载后重新计算 SHA-512，与 registry 及新 lockfile 一致。
- [official-advisories.json](evidence/official-advisories.json)：依据本轮 Bun 报告逐条请求 `https://api.github.com/advisories/<GHSA-ID>`；18/18 取得成功。
- [lock-diff-summary.json](evidence/lock-diff-summary.json)：743 个包条目保持 743；仅四个指定条目发生变化；18 条公告共 42 个版本范围均不包含所选新版本。
- 本轮原始失败报告与命令退出码保存在 `before-audit.*`；原父任务失败证据未覆盖。

## 检查表

每个命令的 JSON 回执记录命令、工作目录、开始/结束时间、退出码、输入文件 SHA-256、日志 SHA-256。原始 stdout/stderr 分别保存在同名 `.stdout.log`、`.stderr.log`。

| 检查 | 退出码 | 结果 | 回执 |
| --- | --- | --- | --- |
| `cd ccr-ui && bun audit --json --audit-level=high`（修改前） | 1 | 18 条：9 high、6 moderate、3 low | `evidence/before-audit.json` |
| 四项 registry 及 tarball 核验 | 0 | SHA-512 匹配；18/18 官方公告取得成功 | `evidence/registry-verification.json` |
| `cd ccr-ui && bun install --frozen-lockfile` | 0 | 安装四包；未修改 lockfile | `evidence/frozen-install.json` |
| `just frontend-audit` | 0 | 0 公告，0/0 例外 | `evidence/frontend-audit.json` |
| `cd ccr-ui && bun run test:smoke -- tests/quality/frontend-dependency-audit.smoke.test.ts` | 0 | 1 文件、3 测试通过 | `evidence/policy-regression.json` |
| lockfile 范围/完整性/政策检查 | 0 | 743 条目；四项变更；42 个受影响范围检查 | `evidence/lock-verification-final.json` |
| `CCR_SKIP_ICON_GENERATION=1 just frontend-check` | 0 | 24 i18n、169 文件/904 smoke、类型、lint、738 文件循环依赖、边界、UI/文档构建通过 | `evidence/frontend-check.json` |
| `just frontend-coverage` | 0 | 169 文件/904 测试；lines 75.19%，13081/17395；门槛 70% | `evidence/frontend-coverage.json` |
| `just version-check` | 0 | 版本及漂移检查通过 | `evidence/version-check.json` |
| `git diff --check` | 0 | 无空白错误；只有既有 autocrlf 转换提示 | `evidence/diff-check.json` |

完整前端命令的子进程退出码为 0，原始日志与回执已完整写入。随后任务内 Python 记录器打印含 `✓` 的日志末尾时遇到 GBK 编码错误，导致外层记录器退出 1。记录器已改为 UTF-8；覆盖率与版本检查随后通过。此错误不修改或覆盖门禁结果，详情见 [capture-print-diagnostic.json](evidence/capture-print-diagnostic.json)。

构建使用已提交图标，日志明确记录 `CCR_SKIP_ICON_GENERATION=1`；未运行品牌生成。仍出现 638.13 kB chunk 警告和 jsdom 导航未实现提示，检查未因此失败。相对父审查覆盖率 13083/17395，本轮少覆盖 2 行，总行数不变；原因未查明，未修改阈值或为平滑结果重复运行覆盖率。

## 未验证范围与集成事项

- 本地 Bun 1.4.2、Node 26.7.0；仓库声明 Bun 1.4.0、hosted Node 24.20.0。未安装全局工具；固定工具链的 hosted 运行仍为 UNVERIFIED。
- 本轮是 Windows shell / jsdom /构建证据；没有 GUI、真实浏览器、Linux、macOS、原生桌面或五套客户端 fresh-session 验证。
- 完整仓库聚合门禁由父任务协调；本子任务不声明 `just ci` 已通过。
- 主会话已回写 `.trellis/spec/ccr/backend/dependency-governance.md` 第 374–375 行：lockfile 负责解析版本、先在既有范围内修复、记录公告/依赖链/开发或运行时边界/integrity；注明 Claude Code、Codex、Grok Build、Kimi Code、OMP 共用规则。本 agent 已复核该段。旧 overrides 示例已由主会话移除。
- 本任务未提交、归档、推送、发布，也未修改全局配置或其他子任务文件。
