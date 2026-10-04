# 新增公告补丁方案独立复核

复核时间：2026-09-30 03:41–03:45 UTC。角色：`trellis-check`，本轮只读复核。方案来源：[new-advisories-proposal.md](new-advisories-proposal.md)。

结论：六节点方案的公告范围、父依赖 semver、最低兼容补丁、tarball integrity 和最小锁变更均有证据支持。UI 四节点属于 T08 已批准的动态安全修复范围。扩展两个节点继续等待追加批准。实施中的 UI 锁存在额外 LF→CRLF 变化，已交实施者恢复原换行并重验；本报告不关闭该项或最终门禁。

## 失败事实与时间边界

- [final-ci.json](final-ci.json) / [final-ci.log](final-ci.log)：真实 `just ci` 于 2026-09-30 03:17:38 UTC 启动，745.46 秒后退出 1。前 13 项通过，第 14 项 frontend-audit 收到 7 条命中，后两项未执行。2509 个交付源码文件的前后 SHA 一致。复核已重新核对日志 SHA。
- UI 的 7 条记录对应 4 个唯一 GHSA；扩展 npm audit 报告 2 个受影响包，包含相同 4 个 GHSA；docs 返回空报告。不能把记录数、GHSA 数、受影响包数和锁节点数互换。
- 四份 GitHub API 原文和两份 npm registry 原文的 SHA 已与 [capture](new-advisories-capture.json) 核对。发布时间均早于此次完整 CI。相同锁 SHA 的先前成功与当前失败必须并存；上游数据传播、缓存等具体变化原因未查明。不能写成“CI 执行期间新发布四项公告”。

| GHSA | published_at（UTC） | updated_at（UTC） | 本次涉及的受影响范围 |
| --- | --- | --- | --- |
| GHSA-6j4f-fj2g-mc7p | 2026-09-29 23:44:58 | 2026-09-29 23:44:59 | brace `<1.1.19`、`>=4.0.0 <5.0.10` |
| GHSA-qhr7-859c-m2p7 | 2026-09-29 23:45:17 | 2026-09-29 23:45:19 | brace `<1.1.20`、`>=4.0.0 <5.0.11` |
| GHSA-q2hr-2g5m-vwhr | 2026-09-29 23:45:39 | 同左 | brace `<1.1.21`、`>=4.0.0 <5.0.12` |
| GHSA-hrr3-gc8f-f4qj | 2026-09-29 23:54:25 | 2026-09-29 23:54:26 | fast-uri `>=3.0.0 <3.1.8` |

其他主版本范围也进入验证；表格仅展示当前解析主版本。原始文件为本目录对应 `new-advisories-GHSA-*.json`。

## 节点、兼容性与授权

| 交付文件 / 节点 | 增量版本 | 解析父节点 / 声明范围 | 授权状态 |
| --- | --- | --- | --- |
| UI `brace-expansion` | 1.1.18 → 1.1.21 | minimatch 3.1.5 / `^1.1.7` | T08 既有范围 |
| UI `fast-uri` | 3.1.7 → 3.1.8 | table/ajv 8.20.0 / `^3.0.1` | T08 既有范围 |
| UI `@typescript-eslint/typescript-estree/minimatch/brace-expansion` | 5.0.9 → 5.0.12 | 对应 minimatch 10.2.5 / `^5.0.5` | T08 既有范围 |
| UI `glob/minimatch/brace-expansion` | 5.0.9 → 5.0.12 | glob/minimatch 10.2.5 / `^5.0.5` | T08 既有范围 |
| 扩展 `node_modules/brace-expansion` | 5.0.9 → 5.0.12 | minimatch 10.2.6 / `^5.0.8` | 新节点，追加批准待定 |
| 扩展 `node_modules/fast-uri` | 3.1.7 → 3.1.8 | ajv 8.18.0 / `^3.0.1` | 原批准精确目标为 3.1.7；新目标需同次确认 |

“UI”指 `ccr-ui/bun.lock`，“扩展”指 `ccr-vscode/package-lock.json`。UI 四节点已由实施者按既有授权应用。2026-09-30 03:45:29 UTC 的只读快照中，扩展锁仍保持原批准版本，未应用本轮两个节点。主会话已明确向用户询问两个新目标。未收到答复不能视为批准。扩展 `undici` 保持 7.29.1。

独立扫描三个解析图中的所有直接父依赖声明，与方案父节点列表一致。使用本仓库安装的 semver，重新从 registry 完整版本集选择当前 major 内满足全部父范围且避开全部对应公告的最低稳定版本，得到 1.1.21、5.0.12、3.1.8。新旧 dependencies、optionalDependencies、peerDependencies、engines、main、exports、type 均相同。详情见 [metadata check](new-advisories-independent-metadata-check.json) 和 [parent check](new-advisories-independent-parent-check.json)。

## 完整性与累计差异

独立第二次从官方 npm registry 下载三个唯一 tarball。全部返回 HTTP 200；分别为 7924、12620、44269 字节。计算 SHA-512 与方案和 registry integrity 相同。仅在内存读取 tarball 的 `package/package.json`，核对包名、版本及七项依赖/运行入口字段；没有解压到项目或执行包代码。详情见 [tarball check](new-advisories-independent-tarball-check.json)。

原 T08 为四个包、18 个唯一 GHSA；追加为 4 个唯一 GHSA。合并共 **22 个 GHSA、57 个声明范围**。独立检查拟最终 UI 的 81 个相关“节点×范围”组合和扩展的 56 个组合，均无匹配。该结论针对保存的公告快照，不能代替对实际交付锁重新运行审计。扩展候选只在内存应用后检查。

| 口径 | UI | 扩展 | docs |
| --- | --- | --- | --- |
| 本轮增量 | 4 个节点 | 2 个节点，待批准 | 0 |
| 相对 HEAD 的拟最终累计 | 7 个节点 | 3 个节点，包含既有 undici | 0 |
| 解析条目数 | 743 | 327，含根条目 | 179 |

UI 累计为 brace-expansion 三个解析节点、colord、fast-uri、js-yaml、undici。扩展累计为 brace-expansion、fast-uri、undici；fast-uri 的两次维护只计一个节点。UI 只允许版本和 integrity 字段改变；扩展只允许 version/resolved/integrity 改变。没有新增或删除解析节点，没有变更依赖结构、父范围或 manifest 依赖声明。docs 锁未改。完整逐节点差异见 [cumulative check](new-advisories-independent-cumulative-check.json)。

## Findings (fixed)

本复核未写产品文件。主会话已在批准的 `dependency-governance.md` 中完成三行相关修正，独立读取后确认：

- 移除把 brace 1.1.18/2.1.4/5.0.9 写成恒定安全值的示例，改为保持原 major、满足父范围并核对当前公告。
- 检查要求明确 frozen install 不改变锁字节，交付差异限于批准节点。
- 新增审计快照、保留失败、不能从发布时间推断响应变化原因，以及窄检查完成后重跑完整失败聚合的规则。

已核验规范 SHA-256：`a30515eb1d2e75b97fd3feb2f76650bd62385fb4bd053b3f9f2cf93ca275dafe`。这些规则与五工具共享合同一致，没有降低门槛或增加 ignore。

## Findings (not fixed)

- **UI 附带换行变化，待实施者关闭。** 2026-09-30 03:45:29 UTC，`bun.lock` 有 1568 个 CRLF。反向替换四个补丁节点后，只有恢复 LF 才匹配诊断前 SHA `87a80e33cdbdaf8ed72516419ece312de1134d94a3e0495d92139580ca65ae07`。实施者确认 `git apply` 受现有 `core.autocrlf` 影响；已接受保持四节点并恢复原 LF，保留首次结果。复核者未操作该文件，也没有要求更改 Git 配置。
- **扩展授权待定。** 前次许可仅含 fast-uri 3.1.7 和 undici 7.29.1。本轮 brace 5.0.12 与 fast-uri 3.1.8 必须等两项目标的追加确认。
- **正式结果尚未关闭。** 本轮不重复实施者正在运行的 frontend-check/coverage；扩展审计/CI/coverage 和完整 `just ci` 仍需针对最终实际锁运行。此前首次完整 CI 保留 FAIL。
- **风险边界保留。** 依赖链属于 UI 开发工具及扩展 VSCE 打包工具；没有证明发行产品存在可利用路径。九条 Tauri warning、原生客户端、macOS 和 hosted 边界不受本补丁结论影响。

## Verification

- PASS：四份 GHSA 与两份 registry 原文哈希、原完整 CI 日志哈希、全部六父节点范围、候选最低版本与新旧元数据。
- PASS：三个 tarball 二次下载 SHA-512、包内元数据、拟最终 22 个 GHSA 联合范围、累计 7/3/0 节点差异、manifest 依赖范围不变。
- PASS：当前 `maxActiveExceptions=0`、docs 锁及三个 manifest 与诊断前快照一致；扩展锁尚未变化。只读范围回执见 [scope check](new-advisories-independent-scope-check.json)。
- Lint / TypeCheck / 全套 Tests：本次方案复核未重跑；由各实施检查及最终聚合提供实际结果。
- 批准后检查保持方案原命令：UI frozen install、audit、policy tests、frontend-check、70% coverage；扩展 npm ci、npm audit、vscode-ci、70% coverage、实际最终 VSIX；随后重跑完整 `just ci` 并比较交付源哈希。

本轮写入仅限父任务独立研究报告及回执。没有修改产品源、安装依赖、提高例外额度、降低阈值、提交、归档、远端触发、推送或发布。
