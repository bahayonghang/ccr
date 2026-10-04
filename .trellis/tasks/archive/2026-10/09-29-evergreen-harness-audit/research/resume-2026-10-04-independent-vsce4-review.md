# VSCE 4 候选：审批前独立源码复核

日期：2026-10-04（America/Chicago）。状态：**PREPARED_NOT_RUN**。实际产品保持 VSCE 3.9.2；未安装候选依赖、执行候选代码、读取凭据、运行 login/publish 或修改产品。

后续授权快照：主会话确认用户已明确批准major迁移。上述PREPARED_NOT_RUN是本报告源码审查时的运行状态；实际实施需等待主会话源快照结束，不以批准替代安装、扫描负例或正式打包回执。

结论：官方 VSCE 4.0.0 源码保留打包时的 secret 和 `.env` 检查。删除的是 Secretlint CLI 与原 globby 依赖链，调用方式改为 Secretlint core API。源码证据支持候选进入审批；运行兼容性、安全负例、固定 Node clean install 和打包验收仍未执行。

## 来源和完整性

- 独立下载 [官方 npm tarball](https://registry.npmjs.org/@vscode/vsce/-/vsce-4.0.0.tgz)，56,482 bytes，SHA-256 `de32bbd17a76ff3471a36e0feff4e07510b3783065b77b1c094685b0accb1d7f`，SHA-512 与已有 registry/候选回执一致。
- 本次读取 tarball 中选定文件，逐文件保存字节、SHA 和来源；没有执行包代码或使用通用 archive 解压。证据位于 T07 `research/resume-2026-10-04-independent-vsce4-source.json` 和同前缀源码副本。
- curl 首次 TLS 握手失败退出 35；改用 Python urllib 默认 TLS 验证下载成功。首失败与成功来源分别保存于 `resume-2026-10-04-independent-vsce4-fetch.json`。没有关闭 TLS 验证。
- `web-access` 前置检查提示浏览器偏好未配置；本次仅访问公开 registry，不启动浏览器、代理或修改偏好。

## secret 扫描路径

官方发布包 `out/package.js:1676` 仍调用 `scanFilesForSecrets`。`:1684–1696` 默认开启 secret 和 `.env` 检查，分别检查磁盘文本文件与内存文件。依赖目录和图片的过滤保留在源码中。`:1700–1738` 检测到不被现有 allow 参数放行的 secret 或 `.env` 时退出 1。

`out/secretLint.js:47–84` 保留推荐规则集、basic auth 规则配置、私钥规则配置、npm token 规则配置及 `.env` 规则。与本机 VSCE 3.9.2 的这些规则片段逐项核对，没有看到删除规则的差异。

`:119–162` 用 `@secretlint/core.lintSource`、`@secretlint/source-creator.createRawSource` 和动态导入的规则实现替换 `@secretlint/node.createEngine` / `executeOnFiles`。错误被捕获后退出 1。`:165–181` 将 messages 转为结果，以 severity 为 error 的消息决定失败；SARIF formatter 包删除后，由 VSCE 内部代码映射字段。

因此，候选审计 0 与打包 secret 检查通过属于不同结论。源码保留检查不能证明新 core API 与实际依赖闭包在目标 Node 上的运行行为。审批后的验收应增加自有合成夹具：正常文件通过、`.env` 文件阻断、合成私钥或测试 token 阻断、对应允许参数保持原行为。夹具不得读取真实账号或凭据。

## keyring 的职责和加载边界

`out/store.js:109–137` 的 `KeytarStore` 改用 `@napi-rs/keyring/keytar.js` 提供的兼容 API，负责查找、写入及删除 publisher PAT。模块仍使用 `vscode-vsce` 服务名。

`@napi-rs/keyring` 的 require 位于 `KeytarStore.open` 默认参数，函数被调用时才执行。`out/main.js` 会加载 store 模块，但源码没有在普通模块初始化或 `packageCommand` 中调用 `openDefaultStore`。打包路径静态检查没有发现凭据读取调用。该判断限定于已核对源码路径，候选 CLI 未执行。

`out/store.js:163–189` 保留 file-store 选择和系统凭据失败后明文 file-store fallback，并增加 legacy migration wrapper。`out/keytarMigration.js:51–53,95–118` 的旧凭据迁移面向 Windows/Linux 交互凭据路径；`out/legacyCredentials.js:86–111` 可能使用 Windows PowerShell 或 Linux `secret-tool` 读取旧记录。实际兼容性、迁移结果和凭据安全边界 **UNVERIFIED**，不纳入普通打包通过结论，也不以本次审批前检查实际触发。

## Node 和仓库接口

官方 manifest 的 Node engines 为 `>=22`。仓库 `.github/workflows/vscode-ci.yml:34–37` 及 release workflow 固定 Node 24.20.0，满足该版本范围。候选副本已有全节点 engines 静态检查 0 不兼容；没有实际运行固定 Node。

官方 manifest 没有 `exports` 限制，bin 仍为 `vsce`，发布包仍有 `vsce` 入口和 `yauzl` 直接依赖。仓库 `check-package-files.mjs:8–9,56` 的 package.json/CLI/ZIP-reader 解析方式在静态结构上仍匹配。候选 clean install 与 `require.resolve` 实际结果 **PREPARED_NOT_RUN**。

`tinyglobby` 替换 glob 实现；发布包额外处理符号链接和大小写匹配兼容。审批后必须保留 13 源文件、15 VSIX 条目的实际允许清单检查。解析条目从 327 降至 169、新增 19/删除 177/改变 7 属于 major 闭包变化，不能按原两个 lock 节点批准范围直接实施。

## Findings (fixed)

无产品修复。独立审查只新增本报告与来源证据。

## Findings (not fixed)

- 当前产品审计仍为 6 high / exit 1。候选副本审计 0 不能关闭实际产品安全验收。
- 候选 major manifest/lock 改动在本次审批前源码复核时未应用；后续用户已批准，由所属实施者在主会话源快照结束后处理，本审查没有产品写权限。
- 新 Secretlint API 的实际 secret/`.env` 负例、目标 Node clean install、类型/测试/coverage、最终 VSIX 包边界均尚未运行。建议列入新增批准后的强制门槛。
- 凭据后端及旧记录迁移改动存在实际行为变化；本次只读核对，不触发真实凭据操作。普通打包验收不能替代凭据迁移验收。

## Verification

- 官方包 SHA-512 / SHA-256：PASS，独立复算。
- secret 扫描与 CLI/ZIP 入口：静态检查 PASS；运行验证 PREPARED_NOT_RUN。
- Node 24.20.0 engines：静态兼容；实际运行 UNVERIFIED。
- 候选 lint/type-check/tests/package：PREPARED_NOT_RUN。
- credentials/native activation/Marketplace/hosted：UNVERIFIED。
