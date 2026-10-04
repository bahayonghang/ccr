# T05 实施与验证记录

当前状态：用户后续批准的 fast-uri 3.1.7、undici 7.29.1 已写入两个锁文件节点；npm audit 为 0 vulnerabilities；扩展 CI 通过；行覆盖率 91.86%、函数覆盖率 91.50%；最终 VSIX 15 条目通过。当前完整证据见 [依赖补丁验收](dependency-patch-results.md)。

以下内容均为后续授权前的初轮历史记录，包括当时未修改锁文件、未升级、审计失败和未运行 coverage 的状态。原始失败 JSON 保持不变。

## 初轮历史记录

- 本地日期：2026-09-29 America/Chicago；检查时间：2026-09-30 UTC。
- 输入 HEAD：`fe9d65972376dbb4c332fde9860c1e42ff2f2005`；Windows 11；Node v26.7.0；npm 12.1.0；just 1.58.0；VSCE 3.9.2。
- 初轮状态：批准范围实现及本地验收通过，等待主会话最终审查；未提交、归档或发布。
- 适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。

## 修改与边界

- `.vscodeignore` 从遗漏式排除改为 13 个明确允许文件。
- 新检查器运行 `vsce ls --no-dependencies`，验证缺失、额外和重复文件；ZIP 检查只读取条目名，要求规范路径并显式拒绝读取错误。
- ZIP reader 从已有 `@vscode/vsce` 的依赖解析；没有新增依赖，没有修改 `package-lock.json`。
- `npm run package` 构建、打包并验证最终 VSIX；`just build` 复用该入口，成功后才复制 artifact。
- `vscode:prepublish` 无递归调用清单检查，已通过实际打包验证。现有 release.yml 的直接 VSCE 入口也调用此前置检查；release.yml 不在批准白名单，保持未改，尚无最终 ZIP 门禁。
- 使用合成 `.serena`、`.abcoder`、五工具状态、`*.local.*`、source map 与内部说明夹具；未读取真实本地配置内容。
- 五工具通用打包契约已回写 `.trellis/spec/ccr-vscode/frontend/extension-surface-contracts.md`。

## 检查结果

| 命令 | 退出码 | 结果 |
|---|---:|---|
| `node --test ccr-vscode/scripts/check-package-files.test.mjs` | 0 | 8/8 新增测试通过 |
| `just vscode-ci` | 0 | npm ci、lint、8/8 打包测试、51/51 既有测试、构建、预检查、最终 ZIP 检查和产物复制通过 |
| `cd ccr-vscode && npx --no-install vsce ls` | 0 | 13 个明确允许文件 |
| `cd ccr-vscode && node scripts/check-package-files.mjs --vsix ccr-vscode.vsix` | 0 | 最终 15 条目通过 |
| `git diff --check` | 0 | 本次检查无空白错误 |
| `git diff --exit-code -- ccr-vscode/package-lock.json` | 0 | 扩展锁文件没有变更 |
| `cd ccr-vscode && npm audit --json` | 1 | 独立审计发现 2 个 high 依赖包，见后续项 |
| `cd ccr-vscode && npm ls fast-uri undici --all --json` | 0 | 已保存两包路径与已安装版本 |
| `npm view fast-uri@3.1.7 version engines dist.integrity --json` | 0 | 已确认兼容修复版本可从 registry 获取 |
| `npm view undici@7.29.1 version engines dist.integrity --json` | 0 | 已确认兼容修复版本；Node >=20.18.1 |

首次新增测试迭代为 6/8：合成 manifest 缺少 contributed command，另一个断言误用了小写错误文本。已补全夹具并修正大小写；后续 8/8 通过。未削弱包边界或门禁。

原始命令输出与时间见 [checks.json](evidence/checks.json)。原审查的 F07 和旧失败记录仍保存在父任务，未改写。

## 最终包内容

- 源文件：`package.json`、`README.md`、`CHANGELOG.md`、`LICENSE`、`icon.png`、`dist/extension.js`，以及 ccr/claude/codex/droid/gemini/iflow/qwen 七个 SVG 图标。
- VSCE 使用 `readme.md`、`changelog.md`、`LICENSE.txt` 三个发行路径，并加入两项 VSIX 元数据，共 15 条目、63,983 bytes。
- 不含 `.serena`、`.abcoder`、`*.local.*`、`dist/extension.js.map`、AGENTS.md、CLAUDE.md、code_map.md。
- VSIX SHA-256：`87185e45da40a9dcd37009465a6f0f85f108c4bb166a80ebb0847499ce363741`。完整清单见 [package-inventory.json](evidence/package-inventory.json)。

## 初轮后续项：扩展开发依赖审计（现已批准修复）

该发现来自未变更的扩展锁文件，超出 T05 文件白名单。主会话要求只形成可审阅方案，本次未升级。`npm audit` 的 2 表示高危依赖包数量，原始报告共含 12 条公告。打包验收通过与依赖审计失败分别保留。

| 依赖 | 已安装 / lock 版本 | 当前父依赖范围 | 兼容修复候选 | audit fixAvailable |
|---|---|---|---|---|
| fast-uri | 3.1.6 | ajv 8.18.0 要求 `^3.0.1` | 3.1.7 | true；当前报告受影响范围 `3.0.0 - 3.1.6` |
| undici | 7.29.0 | cheerio 1.2.0 要求 `^7.19.0` | 7.29.1 | true；当前报告受影响范围 `7.0.0 - 7.29.0` |

- fast-uri 路径：`@vscode/vsce@3.9.2 -> @secretlint/node@10.2.2 -> @secretlint/config-loader@10.2.2 -> ajv@8.18.0 -> fast-uri@3.1.6`。
- undici 路径：`@vscode/vsce@3.9.2 -> cheerio@1.2.0 -> undici@7.29.0`。
- 两个 lock 节点都标记 dev。没有证据证明扩展发行 bundle 存在可利用路径。
- 最小后续范围：批准后仅更新 `ccr-vscode/package-lock.json` 的两个传递节点到已确认修复版本；保持现有父包范围，不新增顶层依赖或 override。解析器引入额外变化时应先复核范围。
- 必过检查：`npm ci`、`npm audit --json`、`just vscode-ci`、`just vscode-coverage`、最终 VSIX 清单和 `git diff --check`。
- [npm audit 原始 JSON](evidence/npm-audit.json)；[已安装依赖树](evidence/dependency-tree.json)。registry 查询返回版本和 integrity 已保存在 checks.json。

## 初轮未验证范围与警告（当前范围见补丁验收）

- 本次未运行托管 CI、Linux/macOS、原生 VS Code 激活、Marketplace 发布或五工具原生会话。
- 本次未运行 VS Code coverage；修改没有调整既有 70% line/function 阈值。
- npm 12 的现有安装策略提示三个依赖脚本受阻：vsce-sign、esbuild、keytar；未批准这些脚本或修改全局策略。现有实际构建、打包结果为通过。
- 当前 Node 26.7.0 与托管固定 Node 24.20.0 不同；本地结果不等于托管结果。
- Git 可能显示 LF/CRLF 转换提示；未修改 Git 换行配置，实际 diff check 退出 0。
