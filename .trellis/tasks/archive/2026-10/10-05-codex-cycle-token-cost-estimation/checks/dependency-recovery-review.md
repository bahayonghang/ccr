# 前端依赖恢复独立检查

日期：2026-10-06。角色：trellis-check。状态：SCOPED_REVIEW_PASS / FULL_CI_PASS（主会话执行）。

本轮检查范围为用户批准的两份锁文件、3 个现有条目。未发现缺陷，未修改产品文件。既有 Codex 功能审查由 [independent-contract-review.md](independent-contract-review.md) 留证，本轮没有重复该审查。

## 范围与一致性

`git diff --numstat` 为 `ccr-ui/bun.lock` 2 增 / 2 删、`docs/bun.lock` 1 增 / 1 删。逐文件读取 `HEAD` 与工作区、解析所有包条目并比较，确认包键集合未变，只有以下 3 个条目变化；完整文件等于 `HEAD` 加对应的行替换。

| 文件 | 包 | 版本 | SHA-512 integrity |
| --- | --- | --- | --- |
| `ccr-ui/bun.lock` | `postcss-selector-parser` | 7.1.4 → 7.1.6 | `sha512-7qASPzhKF2l2KLboRZux8CCTRMdGiV08vWmyKzPz22qZ7ZjQBOeY7rNzNoCLSUiftJ7HUq0GERHmxw/t0dCdMw==` |
| `ccr-ui/bun.lock` | `source-map-js` | 1.2.1 → 1.2.2 | `sha512-KGj/8Y43x35aZVDtt+J4mK1hoLGHULMYfSkODJNQjNDC3oW1PqPoxMwo0pLUsWM/UEGzON/NxeHywEfNXNP3Vw==` |
| `docs/bun.lock` | `source-map-js` | 1.2.1 → 1.2.2 | 同上 |

反向 `git apply --check --unidiff-zero` 退出 0，实际改动与 [批准补丁](dependency-recovery-draft.patch) 一致。`git diff --check -- ccr-ui/bun.lock docs/bun.lock` 退出 0。

`git diff --exit-code` 核对以下文件退出 0：两份 `package.json`、`ccr-ui/patches/braces@3.0.3.patch`、审计 allowlist、审计脚本和根 `justfile`。清单、已有补丁、审计例外及门槛未变。工作区已有 Codex 功能改动保持原值。

## 发布元数据与安装内容

独立从公开 npm registry 读取 [postcss-selector-parser 7.1.6](https://registry.npmjs.org/postcss-selector-parser/7.1.6) 和 [source-map-js 1.2.2](https://registry.npmjs.org/source-map-js/1.2.2)。核对发布的包名、版本、`dist.integrity` 和 runtime dependencies 与锁条目一致。前者依赖仍为 `cssesc ^3.0.0` 和 `util-deprecate ^1.0.2`；后者没有 runtime dependencies。

核对三处已安装的 `node_modules/<package>/package.json`，包名、版本及 runtime dependencies 与发布元数据相同。没有另行下载 tarball 计算独立哈希。

遍历两份锁文件的 dependencies、peerDependencies 和 optionalDependencies：

- UI 的 selector-parser 父范围为 `^7.1.1`、`^7.1.4`，均接受 7.1.6。
- UI 的 source-map-js 4 个父条目，以及 docs 的 3 个父条目，范围均为 `^1.2.1`，均接受 1.2.2。

上述独立静态核对脚本退出 0。没有新增包、父依赖或依赖范围。

## 验证结果与边界

读取 [实施记录](dependency-recovery.md) 及原始日志，确认以下限定检查均退出 0。本轮按派发指令复用成功记录，没有重复安装或审计。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| UI frozen install | PASS，复用实施记录 | [dependency-ui-frozen-install.log](dependency-ui-frozen-install.log) |
| docs frozen install | PASS，复用实施记录 | [dependency-docs-frozen-install.log](dependency-docs-frozen-install.log) |
| `just frontend-audit` | PASS，复用实施记录 | [dependency-frontend-audit.log](dependency-frontend-audit.log) |
| 独立锁文件 / registry / 安装元数据 / 父范围检查 | PASS，退出 0 | 本页 |
| Lint / TypeCheck / Tests | NOT_RUN，本轮限定范围未运行 | 主会话完整 CI 负责 |
| 完整 `just ci` | PASS，主会话执行，退出0 | [just-ci-06.log](just-ci-06.log)，16项步骤全部通过，12:30.169 |

审计原始输出为 `frontend dependency audit passed: 1 reported advisories, 1/1 active exceptions`；没有增加例外。第五轮 CI 的失败记录保留在 [just-ci-05.log](just-ci-05.log)。限定范围检查的 PASS 单独不完成 AC12；主会话后续取得完整 CI PASS 后完成 AC12。

本轮没有运行 Cargo、完整 CI、真实账户验证或其他系统验证，没有提交、归档、push、PR、merge、release 或安装用户工具。只写本检查记录；产品修复数量为 0。

## 主会话最终验收补记

第六轮完整 `just ci` 由主会话执行，退出 0，16 项步骤全部通过，用时 12:30.169。该结果覆盖 lint、type-check、tests、独立 binding gate、审计和前端覆盖率；检查代理本轮的执行范围保持原值。正式结果见 [verification.md](verification.md)。
