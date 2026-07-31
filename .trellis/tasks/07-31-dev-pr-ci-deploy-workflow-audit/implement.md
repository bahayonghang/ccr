# 执行计划：开发、PR、CI、合并与部署流程审计

## 前置条件

- 任务保持 `planning`，直到用户在看到最终规划摘要后显式批准。
- 只有批准后的后续消息才允许运行：

  ```powershell
  python ./.trellis/scripts/task.py start 07-31-dev-pr-ci-deploy-workflow-audit
  ```

- 全程只读访问仓库和 GitHub；唯一允许写入的路径是本任务目录。
- 不运行会写文件的 `just ci`、`just fmt`、`just version-sync`、release/tag/deploy 命令。
- 不触碰 `07-31-profile-open-command` 的文件、状态或工作区改动。

## Step 1 - 刷新易漂移事实

重新采集并记录 `as of` 时间：

```powershell
git status --short
git rev-list --left-right --count origin/main...origin/dev
git branch -a --list '*develop*'
gh pr list --state open --limit 100 --json number,author,baseRefName,mergeStateStatus
gh run list --limit 200 --json databaseId,workflowName,status,conclusion,createdAt,updatedAt,event,headBranch
gh release list --limit 20
gh api repos/bahayonghang/ccr/environments
gh api repos/bahayonghang/ccr/deployments?per_page=100
```

读取 branch protection 时清除权限不足的进程级 token，仅让 `gh` 使用已有 keyring 凭据；不得打印 token 值。若仍不可读，保留 `protected: true` 的公开事实，并把 required-context 细节标为未刷新。

验收：所有漂移事实都有时间戳；失败查询成为证据缺口，不用旧值冒充当前值。

## Step 2 - 还原端到端现状

按 `design.md` 的流程模型检查：

- 本地：`code_map.md`、根/UI/VS Code `justfile`、package scripts、版本/格式/治理脚本、hooks/docs。
- PR/CI：四个 PR workflows、surface policy、required aggregators、branch protection、PR 历史与 run/job/step 数据。
- 发布：`release.yml`、版本源、CHANGELOG、Tauri bundle/signing/updater 配置、GitHub Release/environment/deployment。
- 依赖自动化：Dependabot ecosystem、target、group、limit、schedule 和 PR 状态。

执行期只在 research 文件发现事实错误时更新 `research/repository-workflow-baseline.md`；不要把推断写进事实段。

验收：五个阶段都有入口、触发、门禁、耗时/重复、失败反馈和交接关系。

## Step 3 - 计算指标并拆分等待与执行

使用最近可比较的 completed runs：

- workflow run：count、success/failure、P50/P90 end-to-end。
- job：`startedAt - run.createdAt` 的 queue time，`completedAt - startedAt` 的 execution time。
- step：工具安装、依赖安装、测试/coverage/build 的代表性耗时。
- PR：push 次数、每次 workflow 重跑、从创建到 merge、review 数、变更规模。

样本必须记录起止时间、数量、统计定义和 Dependabot burst 等偏差。只有样本足够时才给百分比收益；否则使用“预计减少固定安装成本”等方向性表述。

## Step 4 - 与行业基线交叉分析

逐项对照 `research/industry-practices.md`：

1. 判断做法是否适用于单维护者/低人类 PR 流量的 Rust + Vue + Tauri + VS Code monorepo。
2. 识别已有良好实践，避免报告只列问题。
3. 将差距归类为体验、速度、可靠性或治理。
4. 对 merge queue、自托管 runner、签名、updater、Marketplace 等写明启用条件。

验收：每个行业结论都有官方来源和适用条件，没有使用二手博客作为关键证据。

## Step 5 - 写 `audit-report.md`

按 `design.md` 第 7 节生成完整报告。每个主要发现填写证据、影响、根因、建议、收益、成本、风险、前置条件和指标。

建议矩阵至少覆盖：

- 恢复 `dev` 的完整反馈与明确管理员 bypass 合同。
- 为 PR 加 `concurrency`/`cancel-in-progress`，为 jobs 加合理 timeout。
- 将 `just ci` 拆成 verification-only full gate 与显式 repair 命令。
- 去重前端安装，减少现场编译 CI 工具的固定成本。
- 对 Dependabot grouping/limit/stagger/target branch 做最小改造方案。
- 将 Release 改为 build artifacts -> verify -> environment-controlled publish。
- job-level permissions、release preflight、provenance/signing/SBOM 的分期路径。
- PR/CI/release 指标基线与回归验证。

## Step 6 - 一致性复核

执行以下检查：

```powershell
rg -n "TODO|TBD|待确认|可能|推断|证据缺口" .trellis/tasks/07-31-dev-pr-ci-deploy-workflow-audit/audit-report.md
rg -n "https://" .trellis/tasks/07-31-dev-pr-ci-deploy-workflow-audit/audit-report.md
git diff --check -- .trellis/tasks/07-31-dev-pr-ci-deploy-workflow-audit
git status --short
python ./.trellis/scripts/task.py validate 07-31-dev-pr-ci-deploy-workflow-audit
```

人工复核：

- 逐项抽查所有 P0/P1 证据链接。
- 确认事实、推断、缺口和建议没有混写。
- 确认 P0/P1/P2 与收益/成本/风险一致。
- 确认任务目录外没有本任务造成的改动。
- 确认没有运行远端写操作、发布、部署或 GitHub 设置修改。

## 完成定义

- `audit-report.md` 满足 PRD AC1-AC8。
- 两份 research 文件与报告口径一致。
- `task.py validate` 通过，但仅作为结构校验，不替代报告证据复核。
- 将报告交付用户审阅；本任务不自动实施其中任何建议。

