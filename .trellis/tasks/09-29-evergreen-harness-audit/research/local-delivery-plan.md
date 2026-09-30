# 本地提交与部分归档计划

用户已授权：请提交并归档。范围为已批准的 32 个交付路径及九任务证据。两个追加补丁仍待批准。

所有提交使用本地 git commit -F。标题无 [AI]；fix 提交包含 Why；提交均附 Agent-Task、Agent-Model、Generated-By。

## harness

~~~text
fix(工具契约): 🐛 补齐 OMP 上下文与五工具说明

Why: 统一五工具的任务上下文、权限边界和契约检查。
补齐 OMP 文档注入、缓存和仓库源文件测试夹具。
对齐 AGENTS、CLAUDE、双语说明和门禁恢复技能。

Tested: OMP 14 tests / 99 assertions；harness 21 tests；独立审查通过
Agent-Task: evergreen-harness-audit
Agent-Model: gpt-6
Generated-By: agent
~~~

文件：

- .codex/skills/ccr-gate-recovery/SKILL.md
- .gitignore
- .omp/extensions/trellis/index.ts
- AGENTS.md
- CLAUDE.md
- docs/agents/harnesses.md
- docs/en/agents/harnesses.md
- scripts/quality/check_harness_contracts.py
- scripts/quality/test_check_harness_contracts.py
- scripts/trellis/omp-context.test.ts

## gates

~~~text
ci(验证门禁): 👷 统一只读 CI 与跨平台检查

将三平台 CI 对齐为同序 16 项检查，校正 Bun Dependabot 配置。
兼容 Copilot 资产的 LF、CRLF 和 BOM，并回写依赖及测试夹具规范。
保留完整 CI 失败：最近一次在 Rust managers 测试出现 FileLockError；后续门槛未运行。

Tested: 治理 48 tests；Copilot 34 tests；完整 just ci FAILED，未宣称全量验收通过
Agent-Task: evergreen-harness-audit
Agent-Model: gpt-6
Generated-By: agent
~~~

文件：

- .github/dependabot.yml
- .github/workflows/ci.yml
- .github/workflows/frontend-ci.yml
- .github/workflows/vscode-ci.yml
- .trellis/spec/ccr/backend/dependency-governance.md
- .trellis/spec/ccr/backend/test-fixtures.md
- docs/en/guide/github-copilot-workspace.md
- docs/guide/github-copilot-workspace.md
- justfile
- scripts/ci/check_workflow_governance.py
- scripts/ci/ci_surface_policy.py
- scripts/ci/test_check_workflow_governance.py
- scripts/quality/check-copilot-assets.mjs
- scripts/quality/check-copilot-assets.test.mjs

## extension

~~~text
fix(扩展打包): 🐛 限制 VSIX 内容并更新已批准依赖

Why: 防止测试和开发文件进入 VSIX，并修复已批准的传递依赖版本。
增加包内容允许范围及 8 项回归测试。
保留 fast-uri 3.1.7、undici 7.29.1；后续新增公告的候选补丁仍待批准。

Tested: 扩展 CI、覆盖率及 15 ZIP 条目检查已通过；最新 npm audit 仍有两个受影响包
Agent-Task: evergreen-harness-audit
Agent-Model: gpt-6
Generated-By: agent
~~~

文件：

- .trellis/spec/ccr-vscode/frontend/extension-surface-contracts.md
- ccr-vscode/.vscodeignore
- ccr-vscode/justfile
- ccr-vscode/package-lock.json
- ccr-vscode/package.json
- ccr-vscode/scripts/check-package-files.mjs
- ccr-vscode/scripts/check-package-files.test.mjs

## ui-lock

~~~text
fix(前端依赖): 🐛 应用七项兼容锁文件补丁

Why: 关闭 UI 审计中已批准处理的安全公告并保持依赖接口兼容。
最终 LF 锁文件通过 frozen install 和 audit，当前漏洞及例外均为零。
完整前端检查和覆盖率基于等价 CRLF 内容；最终 LF 版本未执行完整前端检查。

Tested: 最终 LF frozen install / audit PASS；CRLF 完整前端 904 tests，lines 75.21%
Agent-Task: evergreen-harness-audit
Agent-Model: gpt-6
Generated-By: agent
~~~

文件：

- ccr-ui/bun.lock

## 后续提交

- 任务证据提交：保留两次完整 CI 失败、原始日志、独立审查和批准边界。大型原始 JSON 的提交方式等待用户答复。
- 归档提交：T01 通过新提交源文件夹具后，与 T02、T04 归档；父任务及 T03、T05、T06、T07、T08 保持进行中。
- Journal 提交：记录工作提交 SHA、归档结果与未关闭验收。

完整 just ci 保持 FAILED。不执行未批准修复，不触发远端操作。
