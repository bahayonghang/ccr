# T08 新返回公告后的兼容补丁

日期：2026-09-30 UTC / 2026-09-29 America/Chicago。状态：UI 四节点已修复，完整前端检查、覆盖率和最终 LF 文件的冻结安装、审计均通过。源码已冻结；主会话负责最终独立复核与完整 CI。

## 范围与历史证据

父任务完整 `just ci` 首次运行在 Frontend Audit 失败，前 13 项通过；2509 个源文件哈希未变化。原失败保存在父任务 `research/final-ci.log/.json`。旧 T08 四包修复与审计 0 的结果保留为此前查询时点的记录。

本次 GitHub API、npm registry、三锁审计和六节点方案位于父任务 `research/new-advisories-*`，入口为 `new-advisories-proposal.md`。公告 published/reviewed 时间早于旧 PASS；同锁 SHA 下审计输出转变的上游原因未查明。

主会话确认 UI 同文件内兼容安全更新属于 T08 原批准动态范围。本子代理只改 `ccr-ui/bun.lock`。扩展两节点由主会话等待追加确认；本子代理不改扩展、docs、manifest、生产源码或例外政策。

## 四节点更新

| 精确节点 | 版本变化 | 父范围 |
| --- | --- | --- |
| brace-expansion | 1.1.18 → 1.1.21 | minimatch 3.1.5：`^1.1.7` |
| fast-uri | 3.1.7 → 3.1.8 | table/ajv 8.20.0：`^3.0.1` |
| @typescript-eslint/typescript-estree/minimatch/brace-expansion | 5.0.9 → 5.0.12 | minimatch 10.2.5：`^5.0.5` |
| glob/minimatch/brace-expansion | 5.0.9 → 5.0.12 | minimatch 10.2.5：`^5.0.5` |

3 个唯一 tarball 的 SHA-512、包名、版本和依赖元数据已核对。新旧 dependencies、Node engines 相同；每个新版本均在父 semver 范围内并排除四项官方公告的所有匹配区间。UI 包数保持 743，四条目仅更新版本/integrity。前次 colord、js-yaml、undici 补丁保留。

原始完整字节为 `new-advisories-bun-before.lock`，743 节点快照为 `new-advisories-lock-before.json`。首次应用断言和六个 manifest/lock 的 SHA 前后表为 `new-advisories-apply-ui.json`。

## 换行审查发现

首次通过 `git apply --include=ccr-ui/bun.lock` 应用补丁时，当前 Git 自动换行设置把原 LF 文件写为 CRLF。独立 reviewer 发现额外的全文件换行变化。首次应用 SHA 为 `f4d25c2659543ffbc3c439a92813f0d67d9c44c2bc3d881d7e3710bb52adbf1e`，其 frozen install 和 audit 回执保留。

完整前端和覆盖率进程退出 0 后，从已保存的原 LF 字节构造只替换四行的目标，以 `write_bytes` 保存。反向替换已逐字节恢复原 SHA `87a80e33cdbdaf8ed72516419ece312de1134d94a3e0495d92139580ca65ae07`。最终文件 SHA 为 `c4301a8d1909ac7db2f92930e5d91172195d3ad51c84b4e163c16909a2528025`，CRLF 数为 0；包节点数保持 743。`new-advisories-lf-correction.json` 记录重建、反向验证及六个输入文件的前后哈希。

Git 配置未修改。最终冻结安装、审计和定向 `git diff --check` 均退出 0，所有受检输入哈希保持不变。Git 仍提示未来操作可能按现有 `core.autocrlf=true` 转为 CRLF；该提示保存在 stderr。完整前端和覆盖率回执对应首次 CRLF 文件，内容与最终文件仅换行不同；主会话最终完整 CI 将覆盖最终 LF 文件。

## 检查回执

新回执使用 `research/evidence/new-advisories-*`，不覆盖此前输出。每份回执包含命令、退出码、UTC、输入 SHA 和 stdout/stderr SHA。

| 检查 | 状态 | 证据 |
| --- | --- | --- |
| 首次 frozen install | PASS；4 packages installed，锁稳定 | `evidence/new-advisories-frozen-install.json` |
| 首次 frontend audit | PASS；0 advisory，0/0 exceptions | `evidence/new-advisories-frontend-audit.json` |
| audit policy 回归 | PASS；3/3 | `evidence/new-advisories-policy-regression.json` |
| `just frontend-check` | PASS；162.932 秒；type-check、lint、i18n 24/24、smoke 169 文件/904 测试、UI build、docs audit/build 通过 | `evidence/new-advisories-frontend-check.*` |
| `just frontend-coverage` | PASS；103.077 秒；169 文件/904 测试 | `evidence/new-advisories-frontend-coverage.*` |
| 首次定向 diff 检查 | PASS | `evidence/new-advisories-diff-check.*` |
| 最终 LF frozen install | PASS；654 installs / 723 packages；no changes，输入字节稳定 | `evidence/new-advisories-lf-frozen-install.*` |
| 最终 LF frontend audit | PASS；0 advisory，0/0 exceptions；输入字节稳定 | `evidence/new-advisories-lf-frontend-audit.*` |
| 最终 LF 定向 diff 检查 | PASS；保留 Git 自动换行提示 | `evidence/new-advisories-lf-diff-check.*` |
| 最终完整 `just ci` | 主会话负责 | 保留第一次失败，另建复验回执 |

最终审计开始于 `2026-09-30T03:48:31.740919+00:00`，耗时 4.311 秒。审计 0 是该时点和最终 LF SHA 的查询结果；新公告可能改变后续结果。

完整前端与覆盖率受检 lock SHA 均为 `f4d25c2659543ffbc3c439a92813f0d67d9c44c2bc3d881d7e3710bb52adbf1e`，执行前后输入稳定。覆盖率如下；现有门槛只要求 Lines 至少 70%，未修改配置或重复运行以改变百分比。

| 指标 | 已覆盖 / 总数 | 百分比 |
| --- | --- | --- |
| Lines | 13083 / 17395 | 75.21% |
| Statements | 14655 / 20092 | 72.93% |
| Branches | 8027 / 13500 | 59.45% |
| Functions | 4159 / 5785 | 71.89% |

Statements 72.93% 与此前 Lines 75.19% 属于不同指标，不能直接比较。前端与覆盖率日志中的 jsdom navigation、Vite config、CSS at-rule 和大 chunk 提示保留，未增加抑制设置。

本地 Bun 1.4.2、Node 26.7.0；托管固定版本 Bun 1.4.0、Node 24.20.0 未在本次本地执行。构建由 T06 recipe 传递 `CCR_SKIP_ICON_GENERATION=1`。窗口、native client、托管、MSRV 与 Rust/Tauri coverage 证据保持独立。

本轮未改扩展、docs、manifest、共享规范或生产源码；未提交、归档、推送、发布或安装全局工具。T08 保持 `in_progress`，本地 UI 补丁不代替主会话的最终验收，也不关闭扩展追加范围的审批与独立审计阻断。
