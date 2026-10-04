# T05 新 braces 公告候选方案

状态：候选副本已解析及审计，实际产品未应用。major 变更需新增批准。日期：2026-10-04（America/Chicago）。

## 原因与官方修复路径

当前实际锁为 `@vscode/vsce 3.9.2 / secretlint 10.2.2`。`secretlint → globby 14.1.0 → fast-glob 3.3.3 → micromatch 4.0.8 → braces 3.0.3` 导致 6 个 high 受影响包。官方公告 `GHSA-vfj7-8cjw-p6xm` 声明 `braces <=3.0.3`，`first_patched_version=null`。当前官方 registry 未提供更高 braces 稳定版本，不能构造兼容传递补丁。

官方 registry 中高于或等于 VSCE 3.9.2 的稳定版本只有 3.9.2 和 4.0.0。VSCE 4.0.0 移除了 `secretlint` CLI 和相应 globby 链，直接使用 `@secretlint/core/types/source-creator`，并把文件扫描改为 `tinyglobby`。VSCE 4.0.0 的 engines 为 Node `>=22`；当前固定 Node 24.20.0 满足该范围。VSCE 4 还更换 HTML/Markdown 解析库和密钥存储实现，属于 major 工具迁移，不能视为两节点补丁。

来源：[npm VSCE registry](https://registry.npmjs.org/@vscode/vsce)、[npm secretlint registry](https://registry.npmjs.org/secretlint)、[npm braces registry](https://registry.npmjs.org/braces)、[官方公告](https://api.github.com/advisories/GHSA-vfj7-8cjw-p6xm)。请求状态、时间及原响应 SHA 与提取元数据见 [官方证据](resume-2026-10-04-vsix-braces-official-metadata.json)。[VSCE 4 tarball](resume-2026-10-04-vsix-candidate-tarball.json)重新下载并匹配官方 SHA-512；仅在内存读取 package.json，没有运行代码或解压到文件系统。

## 副本演示结果

在本 research 自有 `resume-2026-10-04-vsix-candidate/` 目录复制实际 manifest/lock，仅把 VSCE 声明 `^3.9.2` 改为 `^4.0.0`，执行 `npm install --package-lock-only --ignore-scripts`。未运行全量升级或 install scripts。候选副本和真实扩展目录分别记录输入哈希。

- 解析 PASS，审计 PASS：0 vulnerabilities。见[解析回执](resume-2026-10-04-vsix-candidate-resolve.json)和[审计回执](resume-2026-10-04-vsix-candidate-audit.json)。该 zero 只属于副本候选。
- 解析条目含根条目由 327 减至 169：新增 19、删除 177、改变 7。完整逐节点清单见[差异](resume-2026-10-04-vsix-candidate-diff.json)。没有隐去大范围依赖变化。
- 7 个改变条目为根条目、`@vscode/vsce`、`@vscode/vsce-sign`、`entities`、`parse5`、`picomatch`、`semver`。根条目还从原锁版本 7.2.0 规范到 manifest 的 7.4.0；该行为由 npm 副本解析产生，实际锁未改。
- 删除 secretlint CLI/globby/fast-glob/micromatch/braces、旧 keytar 和关联依赖；新增 tinyglobby/fdir、marked、proper-lockfile、`@napi-rs/keyring` 及各平台可选包等。`fast-uri` 与 `undici` 随原上游链删除，`brace-expansion 5.0.12` 保留。该变更只属于候选。
- 使用现有 semver 对全部候选节点 engines 逐项验证 Node 24.20.0，0 不兼容。该静态检查不等于在 Node 24.20.0 实际运行。实际 CI/coverage/VSIX 仍使用 VSCE 3.9.2，并未被候选替代。

## 具体新增范围

拟批准文件为 `ccr-vscode/package.json` 与 `ccr-vscode/package-lock.json`。manifest 仅更新 VSCE 声明。lock 更新 VSCE 4 及其必要依赖闭包，逐节点复核上表和完整差异；保留其他直接依赖、配置 contributions、覆盖率和发布边界。不能按原六行锁补丁的权限直接应用。

批准后的必要检查：官方来源及 tarball 完整性、候选差异的独立范围复核、固定 Node 24.20.0 clean install、`npm audit --json`、`just vscode-ci`（8 项清单测试、51 项扩展测试和完整打包）、`just vscode-coverage`（原行/函数 70%）、源码 13 文件和最终 15 条目允许范围、`git diff --check`、父任务最终完整 `just ci`。需核对 `require.resolve('@vscode/vsce/package.json')` 和 VSCE CLI/ZIP-reader 入口兼容性，以及本地 npm 安装脚本阻止策略下的正常打包。只有实际验证暴露必要兼容改动时才提出具体新文件范围。

候选 clean install、类型检查、测试、覆盖率、打包均为 **PREPARED_NOT_RUN**。Node engines 兼容性为静态 PASS，固定 Node 原生运行、native activation、Marketplace 和 hosted 均为 UNVERIFIED。没有在真实目录应用候选、生成发布操作或关闭当前审计 FAIL。
