# T05 VSCE 4 固定 Node 验证

日期：2026-10-04。HEAD：`c50716b5`。产品差异仍只有已批准的 `ccr-vscode/package.json` 与 `ccr-vscode/package-lock.json`。两文件字节与候选 `resume-2026-10-04-vsix-candidate-diff.json` 的 `candidate_sha256` 一致，见 `resume-2026-10-04-vsix-major-apply.json`。后续门禁回执的 `source_bytes_preserved` 均为 true。

## 运行边界

Node 可执行文件 SHA-256 为 `5c976096e04e5c2c1f091938926234cc9fbebfe9787ddd149351b3b0ecc707b5`，与已保存的官方 `win-x64/node.exe` 条目一致。实测版本为 v24.20.0，随包 npm 为 12.2.0。安装目录是临时 `MISE_DATA_DIR`，mise 报告该版本未写入任何配置。直接下载 nodejs.org zip 时 schannel 握手失败；上述可执行文件哈希来自此前已核对的官方条目，以及本次临时安装中的同一哈希。

`@vscode/vsce@4.0.0` 的 `engines.node` 为 `>= 22`。`require.resolve('@vscode/vsce/package.json')` 指向扩展 `node_modules`，`vsce --version` 为 4.0.0。

npm 报告 `@vscode/vsce-sign@2.1.0` 与 `esbuild@0.28.2` 的安装脚本因未列入 `allowScripts` 而被阻止。扩展 CI 仍完成打包。没有执行 publish、login 或账户操作。

## 命令

| 命令 | 结果 | 回执 |
| --- | --- | --- |
| `npm ci` | PASS，0 vulnerabilities；源字节保持 | `resume-2026-10-04-vsix-node24-gate-npm-ci.json` |
| `npm audit --json` | PASS，info/low/moderate/high/critical/total 均为 0；依赖总计 168 | `resume-2026-10-04-vsix-node24-gate-audit.json` |
| 合成普通包 `vsce package` | PASS，6 个归档条目 | `resume-2026-10-04-vsix-scanner-clean-activation.json` |
| 合成 GitHub token | FAIL，退出 1，规则 `github`，未生成 vsix | `resume-2026-10-04-vsix-scanner-secret.json` |
| 合成 `.env` | FAIL，退出 1，规则 `@secretlint/secretlint-rule-no-dotenv`，未生成 vsix | `resume-2026-10-04-vsix-scanner-dotenv.json` |
| `just vscode-ci` | PASS，15.498 秒；包检查 8/8，扩展测试 51/51，源码 13 文件，VSIX 15 条目 | `resume-2026-10-04-vsix-node24-gate-ci.json` |
| `just vscode-coverage` | PASS；行 91.86%，函数 91.50%，原 70% 门槛保持 | `resume-2026-10-04-vsix-node24-gate-coverage.json` |
| `vsce ls --no-dependencies` | PASS，13 个源文件 | `resume-2026-10-04-vsix-node24-gate-source-list.json` |
| 最终 VSIX 允许范围 | PASS，15 个归档条目 | `resume-2026-10-04-vsix-node24-gate-final-vsix.json` |
| `git diff --check` | PASS | `resume-2026-10-04-vsix-node24-gate-diff-check.json` |

首次合成普通包因缺少 `activationEvents` 被 VSCE 4 拒绝。该失败保留在 `resume-2026-10-04-vsix-scanner-clean.json`。补上空 `activationEvents` 后的重跑单独记录。扫描目录是临时合成夹具，使用 Node 24.20.0 启动 VSCE 4.0.0。

## 完整聚合

迁移后的 `just ci` 于 2026-10-04 退出 0，耗时 15:06.415。16 个阶段全部 OK：Version Check、Format Check、OMP Check、Harness Check、Copilot Check、Strict Clippy、Workspace Check、Test、Release Build、Security Audit、CI Governance、Tauri CI、Frontend Check、Frontend Audit、Frontend Coverage、VSCode CI。交付源文件前后均为 2509，`changed_source_paths` 为空。回执为 `../09-29-evergreen-harness-audit/research/resume-2026-10-04-vsce4-full-ci.json`。

这次聚合使用本机默认 Node。上面的固定 Node 24.20.0 门禁是单独回执。Frontend Audit 输出为 `1 reported advisories, 1/1 active exceptions`，不关闭 T08 原 `maxActiveExceptions=0`。

## 未关闭项

hosted CI、Marketplace、原生扩展激活和客户端 trust 仍为 UNVERIFIED。主会话复核见 `resume-2026-10-04-vsce4-independent-review.md`。该复核不关闭这些边界。
