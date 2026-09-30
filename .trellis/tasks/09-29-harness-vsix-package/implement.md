# T05 实施计划

## 启动条件
- [x] 用户在父任务最终总结后明确批准本项。
- [x] 重新核对 git status、关联来源与当前依赖版本。
- [x] 前置项 无前置子任务 满足其验收条件。
- [x] 主会话已校验上下文并启动 T05；task.json 为 in_progress。

## 步骤
- [x] 复核 F07 与设计文件白名单；release.yml 保持未改，扩展锁文件在用户后续批准后只更新两个节点。
- [x] 按 design.md 实施最小修改，并保留原失败证据。
- [x] 逐项满足 prd.md 的 4 条 AC：15 条目、合成负例、lint、51 项既有测试、0 公告和 70% 覆盖率门禁通过。
- [x] 强模型独立复核代码、边界、负例与证据；见[独立审查](../09-29-evergreen-harness-audit/research/independent-review.md)。
- [x] 回写 extension-surface-contracts.md 并注明五套适用工具；未提交归档。

## 必须通过的检查
- [x] `node --test ccr-vscode/scripts/check-package-files.test.mjs`：8/8。
- [x] `just vscode-ci`：lint、8/8 新增测试、51/51 既有测试、完整构建打包通过。
- [x] `cd ccr-vscode && npx --no-install vsce ls`：13 个文件。
- [x] `读取最终 VSIX ZIP 清单并运行新增允许范围检查`：15 条目。
- [x] `git diff --check`：退出 0。

## 证据
记录命令、退出码、日期、OS、版本、输入 SHA 和输出摘要。没有运行的正式环境检查保留 UNVERIFIED。不要把 shell 退出 0、jsdom、原生测试或托管结果互相替代。

实施、包清单和独立 npm audit 后续项见 [验证记录](research/implementation-results.md)。原始审查证据仍保存在父任务。

## 后续批准的扩展依赖补丁

- [x] 用户批准两个兼容补丁，详见父任务 approval.json。
- [x] 核验 12 条官方公告及两个 registry tarball 的 SHA-512/SHA-1；只改两个节点的 version/resolved/integrity。
- [x] npm ci 与 npm audit 通过：0 vulnerabilities。
- [x] just vscode-ci 与 just vscode-coverage 通过：8+51 项测试；line 91.86%、function 91.50%，均超过原 70% 门槛。
- [x] 重新验证最终 15 条目 VSIX 清单及 diff；初轮失败原样保留，新证据单独保存。

后续批准补丁的完整结果见 [依赖补丁验收](research/dependency-patch-results.md)。

## 当前追加公告状态

上方两个补丁的 PASS 是历史快照。2026-09-30 UTC 的后续 npm audit 报告 brace-expansion 与 fast-uri 两个受影响包。新的 5.0.12 / 3.1.8 兼容目标已由强模型复核，正在等待补充批准；未应用新增扩展节点。详见父任务 [追加方案](../09-29-evergreen-harness-audit/research/new-advisories-proposal.md) 与 [独立复核](../09-29-evergreen-harness-audit/research/new-advisories-independent-review.md)。

- [ ] 用户批准两个新目标。
- [ ] 批准后只改上述两个解析节点，保留 undici 7.29.1。
- [ ] 完成 npm ci、npm audit、扩展 CI、覆盖率、最终 VSIX 与完整聚合检查。
