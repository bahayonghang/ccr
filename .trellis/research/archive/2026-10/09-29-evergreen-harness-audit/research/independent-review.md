# T01 / T04 / T05 / T08 独立实施审查

- 日期：2026-09-29 America/Chicago；执行记录跨入 2026-09-30 UTC。
- 基线：dev，HEAD fe9d65972376dbb4c332fde9860c1e42ff2f2005；审查对象为当前未提交交付候选。
- 授权：父任务八项实施及扩展 fast-uri 3.1.7、undici 7.29.1 补丁均已获用户批准。
- 结论：四项范围内未发现阻断验收的代码缺陷。本轮未改生产代码、测试、共享规则或其他 agent 的文件；仅新增本报告。后续 T02/T03/T06 的集成检查另行记录。

## Findings (fixed)

无。本轮没有需要局部自修的已确认问题。

## Findings (not fixed)

没有新增的本范围代码缺陷。以下验收边界仍保留：

- OMP 的原生 ExtensionAPI 类型检查、客户端加载、信任及新会话注入未验证。Bun 契约测试与独立构建不替代原生 API 类型检查。该边界已在 T01 证据中明确。
- 现有 release.yml 直接调用 VSCE；vscode:prepublish 检查源清单，但该发布路径尚未调用最终 ZIP 检查。release.yml 不在 T05 批准文件范围；本轮不改发布流程。需要后续任务接入最终 ZIP 检查，且发布前单独验证。
- 本机 Bun 1.4.2、Node 26.7.0 与声明 Bun 1.4.0、hosted Node 24.20.0 不同。Linux/macOS、固定托管工具链、原生客户端、当前提交 hosted CI、Marketplace 和已提交 clean clone 均未验证。
- 本次审查只消费已保存的 registry/tarball 核验回执，并重新核对 lock integrity 与节点范围；没有重新下载 tarball 或查询新公告。安全审计结论的时间范围由原回执确定。

## 代码与契约复核

| 子任务 | 复核结果 |
| --- | --- |
| T01 | TASK_CONTEXT_ARTIFACTS 同时驱动注入与缓存签名；四类角色收到 PRD/design/implement/info；research 不加载 implement/check JSONL。新增文档通过现有 resolveProjectFile、trustedRoots、materialize 和 ContextBudget。非法路径、越界文件符号链接、明确可信根、UTF-8、总预算和缓存更新均有行为测试。.gitignore 只放行根 .omp/extensions/trellis/index.ts；运行时 import 仅为 Node 内建模块。 |
| T04 | 只在解析边界规范 LF/CRLF 与一个开头 BOM；资产字节保持原样。缺字段、空值、重复键、未闭合头部、错误引号及术语禁用项继续失败。共享规则检查使用 Git 已跟踪的 AGENTS.md 与 .codex/skills；不依赖本机 .claude。双语说明与实际入口一致。 |
| T05 | 13 个源文件允许清单与 15 个最终 ZIP 条目对应；必需 manifest/runtime/图标/许可证/说明均存在。多余、缺失、重复和非规范路径失败；合成工具状态负例与损坏 ZIP 实际执行。检查器只枚举 ZIP 条目名，未读取真实本地配置内容。npm package 与 just build 在复制产物前执行最终清单检查。 |
| T08 | UI lock 保持 743 个包条目，只改 colord、fast-uri、js-yaml、undici 的版本和 integrity；依赖元数据、manifest、审计 policy 和 0 例外上限不变。扩展 lock 保持 327 个条目，只改 fast-uri、undici 两节点的 version/resolved/integrity；package.json 依赖范围不变。新完整性值逐项匹配保存的 registry/tarball 核验记录。 |

六个目标版本：UI colord 2.9.4、fast-uri 3.1.7、js-yaml 4.3.2、undici 8.10.2；扩展 fast-uri 3.1.7、undici 7.29.1。未发现批量解析变更、审计例外、覆盖率阈值降低或生产可利用性推断。

## Verification

### 本轮独立运行

| 检查 | 结果 |
| --- | --- |
| bun test scripts/trellis/omp-context.test.ts | PASS：14/14，99 expect，无跳过。 |
| node --test scripts/quality/check-copilot-assets.test.mjs | PASS：34/34，无跳过。 |
| node --test ccr-vscode/scripts/check-package-files.test.mjs | PASS：8/8，无跳过。 |
| cd ccr-vscode; npm run lint | PASS：runtime 与 test 两个 TypeScript 配置检查。 |
| node scripts/quality/check-copilot-assets.mjs | PASS：12 个资产、4807 个已跟踪文件。 |
| 四个新增或修改的 MJS 文件 node --check | PASS：Copilot 与包清单检查器及测试。 |
| 最终 VSIX 检查 | PASS：15 条目；63,983 bytes；SHA-256 60f8a9656c0cad03c46b08187cc1184ed1215d2d2e561d3912a4ec91a042c9e2，与补丁验收记录一致。 |
| 批准范围 git diff --check | PASS；仅有既有 core.autocrlf 转换提示。 |
| 锁文件结构比较 | PASS：UI 4 个节点、扩展 2 个节点；包总数、父依赖范围、其余字段与 HEAD 一致。 |

### 复用并校验的完整回执

- T08 frontend-check、frontend-coverage、policy-regression、frontend-audit、frozen-install、lock-verification-final：退出码均为 0；重新计算 stdout/stderr SHA-256，与各回执一致；各回执的 UI lock 输入哈希均与当前锁文件一致。
- 前端 lint、typecheck、24 项 i18n、169 文件/904 项 smoke、3 项 audit policy 测试及构建通过；line coverage 75.19%，原门槛 70%。完整 suite 未重复运行。
- T05 修复后回执：npm ci、npm audit、vscode-ci 和 vscode-coverage 均为 0；8 项包检查与 51 项既有测试通过；line 91.86%、function 91.50%，原门槛均为 70%。本轮独立重跑类型检查与包检查，未重复完整扩展 suite。
- T01/T04 当前源码 SHA-256 与各自 verification.json 一致；selected-source 夹具记录明确包含未提交的可交付文件，没有声称已提交完整 clean clone。
- 已核对 extension-surface-contracts.md 的打包与两节点补丁契约，以及 dependency-governance.md 的安全更新条款。两份源摘录 SHA-256 与当前 dependency-governance.md、workflow.md 匹配。

证据入口：[T01](../../archive/2026-09/09-29-harness-omp-context/research/implementation-results.md)、[T04](../../archive/2026-09/09-29-harness-copilot-crlf/research/implementation-results.md)、[T05](../../09-29-harness-vsix-package/research/dependency-patch-results.md)、[T08](../../09-29-harness-frontend-security/research/implementation-results.md)。

Lint：扩展独立 PASS，前端回执 PASS，脚本语法 PASS。TypeCheck：扩展独立 PASS，前端回执 PASS；OMP 原生 API 类型检查 UNVERIFIED。Tests：重点 56/56 PASS，完整前端/扩展既有回执 PASS。
