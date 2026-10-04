# T05 固定补丁实施结果

日期：2026-10-04（America/Chicago）。基线：`dev / c50716b5`。状态：固定补丁及功能检查完成；当前安全验收未完成。

## 实际差异

仅修改 `ccr-vscode/package-lock.json` 的 `node_modules/brace-expansion` 5.0.9 → 5.0.12 和 `node_modules/fast-uri` 3.1.7 → 3.1.8。每个节点只改 `version/resolved/integrity`。327 个解析条目保持不变，undici 保持 7.29.1，manifest 和另外 50 个已跟踪扩展文件字节保持不变。原混合换行计数保持不变。详细节点、输入 SHA 和最终包清单见 [最终核对](resume-2026-10-04-vsix-final-verification.json)。

保存的官方 registry、公告原文和独立 tarball 回执哈希已核对。两个目标重新请求官方 registry，重新下载 tarball，SHA-512 与 registry integrity 一致；没有执行 tarball 代码。具体来源、下载字节与哈希见 [实施核对](resume-2026-10-04-vsix-prepare.json)。

## 新检查回执

运行环境：Windows、Node v26.7.0、npm 12.2.0、just 1.58.0。原 Node 24.20.0 托管运行边界未验证。每个命令的 JSON 回执记录日期、argv、OS、HEAD、输入 SHA、退出码、stdout/stderr 字节及 SHA；同名前缀的 `.stdout.log/.stderr.log` 保留原始输出。

| 命令 | 结果 | 证据 |
| --- | --- | --- |
| `npm audit --json`，补丁前 | FAIL，7 high + 1 moderate，共 8 受影响包 | [回执](resume-2026-10-04-vsix-audit-before.json) |
| `npm ci` | PASS；安装脚本沿用现有阻止策略 | [回执](resume-2026-10-04-vsix-npm-ci.json) |
| `npm audit --json`，补丁后 | FAIL，6 high 受影响包，均来自 braces 传递链 | [回执](resume-2026-10-04-vsix-audit-after.json) |
| `just vscode-ci` | PASS；包检查 8/8，扩展测试 51/51，源码 13 文件，VSIX 15 条目 | [回执](resume-2026-10-04-vsix-ci.json) |
| `just vscode-coverage` | PASS；行 91.86%、函数 91.50%，保留原 70% 门槛和默认并行 | [回执](resume-2026-10-04-vsix-coverage.json) |
| `npx --no-install vsce ls --no-dependencies` | PASS；13 源文件 | [回执](resume-2026-10-04-vsix-source-list.json) |
| 最终 `check-package-files.mjs --vsix` | PASS；15 条目，未包含本地工具状态、配置、source map 或内部说明 | [回执](resume-2026-10-04-vsix-final-vsix.json) |
| 扩展锁 `git diff --check` | PASS | [回执](resume-2026-10-04-vsix-diff-check.json) |

新增未关闭公告为 [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm)。受影响链为 `@vscode/vsce → secretlint → globby → fast-glob → micromatch → braces 3.0.3`。这些依赖用于开发打包；本轮没有证明发行扩展的可利用路径。没有应用 major 候选、增加例外或降低门槛。候选详情见 [VSCE 4 方案](resume-2026-10-04-vsix-braces-proposal.md)。历史 zero 审计回执仍保留；当前 FAIL 单独保存。

## 执行辅助问题与边界

首次准备过程使用系统默认 GBK 读取历史 registry，发生 UnicodeDecodeError，尚未写实际锁。改为明确 UTF-8 后通过；[失败记录](resume-2026-10-04-vsix-prepare-first-failure.json)保留。扩展 CI 退出 0 后，辅助脚本打印摘要时发生 GBK UnicodeEncodeError；正式命令退出码、完整 stdout/stderr 和 JSON 回执已在打印前保存。[辅助失败记录](resume-2026-10-04-vsix-ci-runner-print-failure.json)单列；成功门禁未重跑。首次准备的原始 stdout/stderr 未独立捕获，该限制已记录。

`web-access` 前置检查发现无浏览器远程调试。该脚本自动创建 `C:/Users/lyh/.skillsmanage/skills/web-access/config.env`；未读取其内容，未更改浏览器或其他全局设置。后续只使用公开官方 registry/API 和已批准 npm 命令。未使用 CDP。

根完整 `just ci` 由主会话负责；本子代理未运行。hosted、Marketplace、原生扩展激活和客户加载/trust 保持 UNVERIFIED。没有提交、归档、推送或发布。
