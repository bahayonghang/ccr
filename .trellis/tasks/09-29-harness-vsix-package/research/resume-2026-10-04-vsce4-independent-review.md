# T05 VSCE 4 主会话复核

日期：2026-10-04。复核人：主会话。本复核只核对已保存回执与当前工作区字节，没有重跑 `just ci`，也没有第二名审查者。

## 文件范围

当前 `ccr-vscode/package.json` SHA-256 为 `2772ba01e054ad6b98c02338c1c6426b5ae2ff15af56221b3b2a8dbc876d34ec`。当前 `ccr-vscode/package-lock.json` SHA-256 为 `eac58599d926c08ca203ae96541955111f2e5202032d2641a1805916b8fc016d`。两者与 `resume-2026-10-04-vsix-candidate-diff.json` 的 `candidate_sha256`，以及 `resume-2026-10-04-vsix-major-apply.json` 的应用后哈希相同。`package.json` 的 `@vscode/vsce` 为 `^4.0.0`。两文件仍未提交。

候选记录 `approved_targets_retained`：`brace-expansion` 为 `5.0.12`，`fast-uri` 与 `undici` 为 null。早先“只改两个锁节点”的续作没有按该范围实施，相关勾选保持未完成。

## 固定 Node 24.20.0

回执目录为 `research/`。下列命令退出码与 `source_bytes_preserved` 来自对应 JSON。

| 回执 | 退出码 | 源字节 |
| --- | --- | --- |
| `resume-2026-10-04-vsix-node24-gate-npm-ci.json` | 0 | true |
| `resume-2026-10-04-vsix-node24-gate-audit.json` | 0 | true |
| `resume-2026-10-04-vsix-node24-gate-ci.json` | 0 | true |
| `resume-2026-10-04-vsix-node24-gate-coverage.json` | 0 | true |
| `resume-2026-10-04-vsix-node24-gate-final-vsix.json` | 0 | true |
| `resume-2026-10-04-vsix-scanner-clean.json` | 1 | 不适用 |
| `resume-2026-10-04-vsix-scanner-clean-activation.json` | 0 | 不适用 |
| `resume-2026-10-04-vsix-scanner-secret.json` | 1 | 不适用 |
| `resume-2026-10-04-vsix-scanner-dotenv.json` | 1 | 不适用 |

首次合成包失败原文保留：`Manifest needs the 'activationEvents' property, given it has a 'main' property.` 产品 `package.json` 有 `main`，没有 `activationEvents`。已安装 VSCE 4 `out/package.js` 的 `validateManifestForPackaging` 在 `engines.vscode` 为 `*` 或 `>=1.74` 且存在 `contributes.languages`、`commands`、`authentication`、`customEditors` 或 `views` 时，把这些贡献视为隐式激活事件。产品清单满足该条件，所以固定 Node 上的 `just vscode-ci` 仍能打包。合成夹具没有这些贡献，因此需要 `activationEvents`。不要把空 `activationEvents` 写进产品清单。

npm 12 阻止了未列入 `allowScripts` 的 `@vscode/vsce-sign@2.1.0` 与 `esbuild@0.28.2` 安装脚本。扩展 CI 仍完成打包。本复核不新增 `allowScripts`。没有 publish、login 或 PAT。

覆盖率门槛仍为行 70%、函数 70%。该次实测为行 91.86%、函数 91.50%。源码清单 13 个文件，最终 VSIX 15 个条目。

## 聚合

`../09-29-evergreen-harness-audit/research/resume-2026-10-04-vsce4-full-ci.json`：`outer_exit_code` 0，`aggregate_exit_code` 0，HEAD `c50716b5ed1bcd1c5b4950a746a8621258d0e0b5`，交付源文件前后均为 2509，`changed_source_paths` 为空，`source_bytes_preserved` 为 true。stdout 的 TOTAL 为 15:06.415，16 步 OK，末行 `CI passed - all steps green!`。这次聚合使用本机默认 Node，不替代上面的 Node 24.20.0 门禁。

Frontend Audit 原文：`frontend dependency audit passed: 1 reported advisories, 1/1 active exceptions`。该行不关闭 T08 原 `maxActiveExceptions=0`。

## 未关闭

hosted CI、Marketplace 接受、原生扩展激活、客户端 trust 仍为 UNVERIFIED。本机退出 0 不代替这些边界。T05 因此保持 `in_progress`，不归档。
