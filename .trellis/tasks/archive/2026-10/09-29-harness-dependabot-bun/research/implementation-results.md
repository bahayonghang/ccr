# T03 实施与验证记录

执行日期：2026-09-30 UTC；America/Chicago 本地日期为 2026-09-29。
基线：`fe9d65972376dbb4c332fde9860c1e42ff2f2005`。环境为 Windows 11、Python 3.14.7、Bun 1.4.2、Node 26.7.0、just 1.58.0。详见 [environment.json](evidence/environment.json)。

## 实施结果

`.github/dependabot.yml` 仅修改两行：`/ccr-ui`、`/docs` 的 `package-ecosystem` 从 `npm` 改为 `bun`。`/ccr-vscode` 保持 `npm`；GitHub Actions、两个 Cargo 项、目录、周调度、labels 均保持原值。对 HEAD 文件作精确替换比对，结果记录在 [mapping-scope.json](evidence/mapping-scope.json)。

| 目录 | 生态 | 锁文件 / 包管理器依据 |
| --- | --- | --- |
| `/ccr-ui` | `bun` | `bun.lock`；`packageManager` 为 Bun 精确版本的权威来源 |
| `/docs` | `bun` | `bun.lock`；`packageManager` 与 UI 的 Bun pin 一致 |
| `/ccr-vscode` | `npm` | `package-lock.json`；当前 manifest 没有声明 `packageManager`，维持 npm 默认；若显式声明须使用 npm |
| `/` | `cargo` | 根 `Cargo.lock` |
| `/ccr-ui/src-tauri` | `cargo` | 独立 Tauri `Cargo.lock` |
| `/` | `github-actions` | GitHub Actions 更新项，与 Cargo 根目录项分别保留 |

在现有 `scripts/ci/check_workflow_governance.py` 中增加 `dependabot_failures`，接入原有 `main`。解析器沿用仓库约定的 YAML 缩进布局，不引入外部解析依赖或新 CI 引擎。检查覆盖：

- 必要生态/目录映射缺失或不匹配。
- 相同生态/目录重复，以及 JavaScript 目录配置多个生态。根目录的 GitHub Actions 与 Cargo 项允许并存。
- 重复 YAML mapping key、更新项缺少生态或目录。
- 应用包/文档包的权威锁文件缺失，Bun/npm 锁文件同时存在。
- Bun manifest 与规范 pin 不一致；扩展显式声明不同包管理器。
- 非法 JSON 或非对象 manifest，返回治理错误。

`scripts/ci/test_check_workflow_governance.py` 新增 11 个测试方法，使用独立临时目录。正例包括引号、行内注释、字段顺序和嵌套调度；负例包括上述映射、重复、锁文件和 manifest 情况。

## 历史失败与当前证据

父审查 F05 记录的真实失败：Dependabot run `36370961840`，SHA `7ab923ecfb302ba041128de852d90d6a5aca8c86`；日志明确报告 npm 更新器不能处理 `/docs/bun.lock`。原始证据留在父任务，见 [审查报告 F05](../../09-29-evergreen-harness-audit/research/audit-report.md)。本轮没有覆盖历史失败记录。

当前本地检查全部通过。下一次真实 Dependabot 运行仍为 **UNVERIFIED**。本轮没有推送、dispatch、rerun 或修改账号配置。静态配置、fixture 和冻结安装通过不证明 GitHub 已执行成功。

## 检查结果

每份 JSON 回执记录命令、工作目录、开始时间、耗时、退出码、输入文件前后 SHA-256，以及 stdout/stderr 日志 SHA-256。摘要见 [check-results.json](evidence/check-results.json)。

| 命令 | 退出码 | 结果 / 回执 |
| --- | --- | --- |
| `python -m unittest scripts.ci.test_check_workflow_governance` | 0 | 35/35；`evidence/unit.json` |
| `just workflow-governance-check` | 0 | 38/38 组合测试；45 个 immutable action refs；serial annotations 0；`evidence/workflow-governance.json` |
| `just version-check` | 0 | 版本、Bun pin/lock、Rust 依赖漂移检查；`evidence/version-check.json` |
| `cd docs && bun install --frozen-lockfile` | 0 | 130 installs / 180 packages，无变更；`evidence/docs-frozen.json` |
| `cd docs && bun run audit` | 0 | 文档审计；`evidence/docs-audit.json` |
| `cd docs && bun run build` | 0 | VitePress 构建；`evidence/docs-build.json` |
| `cd ccr-ui && bun install --frozen-lockfile` | 0 | 654 installs / 723 packages，无变更；`evidence/ui-frozen.json` |
| `actionlint` | 0 | 工作流静态检查；`evidence/actionlint.json` |
| `git diff --check` | 0 | 无空白错误；存在 autocrlf 转换提示；`evidence/diff-check.json` |

## 边界与交接

- 本地 Bun 1.4.2、Node 26.7.0 与固定 hosted 工具链不同。没有安装或切换全局工具。
- T03 未修改 root justfile、工作流文件、依赖 manifest 或 lockfile。既有并行策略、覆盖率门槛、required context 名称保持原值。
- 共享 `.trellis/spec/ccr/backend/dependency-governance.md` 由主会话回写：说明三个 JS 目录的生态/包管理器/锁文件一致性、缺失/重复负例和 hosted 结果的证据边界，并注明五套工具共用。
- T03-AC1、T03-AC2 已完成本地验收；T03-AC3 的真实托管运行仍待后续已授权或自然触发。主会话负责最终独立审查和任务状态。
- 未提交、归档、推送或发布。
