# T05 已批准的扩展依赖补丁验收

- 验证日期：2026-09-30 UTC。输入 HEAD：`fe9d65972376dbb4c332fde9860c1e42ff2f2005`。
- 用户后续批准 fast-uri 3.1.6 → 3.1.7、undici 7.29.0 → 7.29.1；T05-AC4 和 design.md 白名单已记录。
- 结果：两个补丁已应用，npm audit 为 0 vulnerabilities，扩展 CI、覆盖率和最终 VSIX 检查通过。等待主会话最终审查；未提交、归档或发布。

## 变更边界

`ccr-vscode/package-lock.json` 恰好修改 2 个传递节点，每个节点只改 `version`、`resolved`、`integrity`。没有新增或删除节点，没有修改其他 lock 字段；package.json 的 dependencies、devDependencies、overrides 与 HEAD 相同。

| 节点 | 旧版本 | 新版本 | 保持的父依赖范围 |
|---|---|---|---|
| node_modules/fast-uri | 3.1.6 | 3.1.7 | ajv 8.18.0：`^3.0.1` |
| node_modules/undici | 7.29.0 | 7.29.1 | cheerio 1.2.0：`^7.19.0` |

新增规范适用于 Claude Code、Codex、Grok Build、Kimi Code、OMP：核验公告和 tarball 完整性、限制 lock 节点范围、保留原失败证据、分别验证打包与安全审计。没有调整任何审计或覆盖率门槛。

## 官方来源与完整性

- 使用 `gh api advisories/<GHSA>` 读取初轮报告对应的全部 12 条官方公告记录。fast-uri 3.x 两项公告的首个修复版本为 3.1.7；undici 7.x 十项公告的首个修复版本为 7.29.1。
- 使用 VSCE 既有 semver 解析器验证两个目标版本不属于全部已记录公告的受影响范围，并满足原父依赖范围。
- 从官方 npm registry 读取指定版本元数据，下载元数据中的 tarball；实算 SHA-512 integrity 和 SHA-1 shasum 均与发布元数据一致。
- fast-uri tarball：43,760 bytes；undici tarball：400,243 bytes。`npm ci` 后再次核对已安装版本与锁文件。
- [来源、公告范围与哈希记录](evidence/dependency-patch-sources.json)。

## 必须通过的检查

| 命令或检查 | 退出码 | 结果 |
|---|---:|---|
| `npm ci`（由 `just vscode-ci` 执行） | 0 | 安装 292 包，审计显示 0 vulnerabilities |
| `npm audit --json` | 0 | 全部严重级别均为 0 |
| `just vscode-ci` | 0 | lint、8/8 打包测试、51/51 既有扩展测试、预检查、打包、15 条目最终检查和复制通过 |
| `just vscode-coverage` | 0 | 51/51 测试；line 91.86%、branch 81.08%、function 91.50%；line/function 门槛均保持 70% |
| `npm ls fast-uri undici --all --json` | 0 | 实际安装版本为 3.1.7 和 7.29.1，原有依赖路径保持 |
| 官方公告、父范围及 lock integrity 断言 | 0 | 12 条公告、两个 registry 元数据及已安装版本一致 |
| lock 节点和 package.json 范围断言 | 0 | 2 个节点，每节点 3 个字段；其他变更 0 |
| `node ccr-vscode/scripts/check-package-files.mjs --vsix ccr-vscode/ccr-vscode.vsix` | 0 | 15 条目，允许范围不变 |
| `git diff --check` | 0 | 无空白错误；Git 换行提示单独保留 |

Node 原生覆盖率输出的总计为百分比；测试数量为 51。没有修改统计范围或根据百分比推测行数。

最终包：63,983 bytes，SHA-256 `60f8a9656c0cad03c46b08187cc1184ed1215d2d2e561d3912a4ec91a042c9e2`。条目名称与初轮允许清单相同；重新打包后 ZIP 哈希更新。

- [本轮原始命令输出](evidence/checks-after-dependency-patch.json)
- [修复后 audit JSON](evidence/npm-audit-after-patch.json)
- [节点差异及最终包清单](evidence/dependency-patch-diff-and-package.json)
- [初轮失败 audit JSON，原样保留](evidence/npm-audit.json)

证据写入脚本首次因工具字符串换行编码报 SyntaxError；修正后作用域断言、清单保存及最终检查均通过。该错误没有修改依赖节点或测试门禁。

## 仍未验证的边界

- Windows 本地 Node 26.7.0、npm 12.1.0；托管固定 Node 24.20.0、本次托管运行、Linux/macOS 和原生 VS Code 激活未验证。
- 现有 release.yml 仅经 vscode:prepublish 增加前置文件清单验证；本轮未修改该 workflow，也未运行发布。
- npm 的现有三个 install-script 阻止提示保留；没有改变全局策略。实际扩展构建、打包与检查通过。
