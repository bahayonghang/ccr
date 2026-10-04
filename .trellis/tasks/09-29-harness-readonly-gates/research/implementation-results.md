# T06 实施交付记录

日期：2026-09-30 UTC / 2026-09-29 America/Chicago。状态：实施与窄验证完成；等待独立审查和主会话完整 `just ci` 验收。`task.json` 保持 `in_progress`。

T06 集成前置项的本地已验证代码。T03 AC3 实际 Dependabot 运行仍为 UNVERIFIED；T01 的原生加载与已提交新检出边界保留。前置任务没有因本次集成而关闭所有 AC。

## 变更

本项修改 7 个源码或配置文件。`scripts/ci/check_workflow_governance.py` 与对应测试中已有 T03 改动继续保留。没有修改 `release.yml`、架构测试、生产 Rust/TypeScript 代码、测试并行策略、覆盖率门槛或 required job 名称。

| 文件 | 本项改动 |
| --- | --- |
| `justfile` | 三 OS 聚合去除 `version-sync`、`fmt`；加入 OMP、harness、Copilot 和 frontend audit；审计两个 Cargo lock；缺少 cargo-audit 时失败并打印前置要求；frontend-build 使用 recipe 级图标环境变量 |
| `.github/workflows/ci.yml` | workspace-quality 安装固定 Bun/Node 并调用三项工具契约门槛；security-audit 审计两个 lock |
| `.github/workflows/frontend-ci.yml` | frontend 检查显式传递 `CCR_SKIP_ICON_GENERATION=1` |
| `.github/workflows/vscode-ci.yml` | 说明既有 `vscode-ci` 调用链检查最终 VSIX |
| `scripts/ci/ci_surface_policy.py` | 新增工具入口、规则、共享 skill、OMP 源文件、独立 lock 及检查器的相关性路径 |
| `scripts/ci/check_workflow_governance.py` | 检查顺序、叶子命令、最终 VSIX 调用链、托管运行时、检查入口与相关性遗漏守卫 |
| `scripts/ci/test_check_workflow_governance.py` | 三 OS 命令 stub、首错传播、工具缺失、图标环境传播、遗漏和调用顺序负例 |

共同顺序共 16 项：

```text
version-check -> fmt-check -> omp-check -> harness-check -> copilot-check
-> lint-strict -> check-workspace -> test -> release -> audit
-> ci-governance-check -> tauri-ci -> frontend-check -> frontend-audit
-> frontend-coverage -> vscode-ci
```

`version-sync` 与 `fmt` 仍保留为显式修复命令。`ci` 的只读边界指不重写跟踪源码、版本、格式或图标；依赖安装、编译产物、覆盖率文件、VSIX 和 advisory 缓存仍有文件系统写入。

`vscode-ci` 已通过 T05 接通 `ccr-vscode/justfile` 的 `ci -> build -> npm run package`；`package` 在生成 `ccr-vscode.vsix` 后执行 `npm run check:vsix`。本项检查该链路，未增加重复打包步骤。

## 检查结果

| 命令 | 退出码 | 结果 / 原始证据 |
| --- | --- | --- |
| `python -m unittest scripts.ci.test_check_workflow_governance scripts.ci.test_architecture_contract_gates` | 0 | 48/48；`11-final-governance-tests.log` |
| `just workflow-governance-check` | 0 | 48/48，47 个 immutable action 引用，serial-only 计数 0；`12-final-workflow-governance.log` |
| `actionlint` | 0 | `04-actionlint.log` |
| `just omp-check` | 0 | 14/14，99 assertions；`05-omp-check.log` |
| `just harness-check` | 0 | 20/20；6 份共享文件、5 套工具契约；`06-harness-check.log` |
| `just copilot-check` | 0 | 34/34；12 个资产、4807 个跟踪文件；`07-copilot-check.log` |
| `just frontend-audit` | 0 | 0 advisory、0/0 active exceptions；`08-frontend-audit.log` |
| `just audit` | 0 | root 491、Tauri 678 个 crate dependency；Tauri 原有 9 allowed warnings 保留；`09-cargo-audit.log` |
| 定向 `git diff --check` | 0 | `13-final-diff-check.log` |

治理测试执行三种实际聚合 recipe 的 shell body；只将叶子 `just` 命令替换为受控 stub。每种 recipe 覆盖成功路径和 6 个失败位置，共 21 个场景。失败退出码固定为 37，并检查调用前缀准确终止。缺少/存在 cargo-audit 的 helper 覆盖 6 个场景。真实 `just frontend-build` 在临时 fixture 中执行，确认 frozen install 和 build 两次 Bun 调用都收到 `CCR_SKIP_ICON_GENERATION=1`。

Windows recipe 使用 PowerShell；Linux/macOS recipe body 在 Windows 主机的 WSL Bash 上执行。该证据证明命令顺序和控制流，不证明原生 macOS 产品测试通过。测试不需要真实 Rust/Bun/Node 产品工具链；图标传播 fixture 需要 `just` 和 Python。

`scoped-check-results.json` 保留首轮退出码、时间及输入 SHA。`final-scoped-check-results.json` 记录修正后的治理验证和 7 个交付文件的前后 SHA；漂移列表为空。完整仓库源码无漂移证明由主会话的最终 `just ci` 负责。

## 保留的失败记录

1. Windows legacy `bash.exe -c` 对脚本文本产生第二次 shell 展开，14 个 Bash 聚合场景失败。原始记录为 `01-initial-bash-command-transport-failure.log/.json`。
2. 改为标准输入后，Python 文本流在 Windows 写出 CRLF，使 Bash helper 和聚合体解析失败。首轮组合套件报告 18 个子场景失败，见 `02-governance-tests.log`、`03-workflow-governance.log`。
3. 测试传输改为 UTF-8 bytes 标准输入后，组合套件与真实治理 recipe 全部通过。保留实际聚合脚本的计时逻辑和执行策略。

## 环境与边界

`environment.json` 记录实际工具路径、版本、3 个 lockfile SHA 和 RustSec revision。实际本机版本为 just 1.58.0、Bun 1.4.2、Node v26.7.0、PowerShell 7.6.5、Bash 5.2.21、actionlint 1.7.12、cargo-audit 0.22.2。

托管配置继续固定 Bun 1.4.0 和 Node 24.20.0；本次本地成功不构成这些固定托管运行时的执行证据。未安装或切换用户全局工具。RustSec 数据库抓取成功，记录 revision `f23b768236fe2880e4cfa167da662cad8ca79240`；审计加载 1277 条 advisory。

四个 required context 仍为 `Root Workspace Required`、`Vue and Docs Required`、`Tauri Linux Required`、`VS Code Required`。Root/React/VS Code 70% 与 Root/Tauri gateway 85% 门槛保持不变。Rust/Tauri 默认并行和 `--skip export_bindings` 保持不变。

## AC 与后续

| AC | 本次证据 | 后续责任 |
| --- | --- | --- |
| AC1 | 去除聚合修复步骤；安装前置检查不安装；图标 env fixture 通过；7 个源码文件窄验证前后无漂移 | 主会话完整 `just ci` 的全仓库哈希检查 |
| AC2 | 21 个三 OS 聚合控制流场景、6 个 audit helper 场景、真实 just 图标传播和遗漏负例通过 | 独立强模型审查 |
| AC3 | 相关性遗漏守卫、47 个 immutable action 引用、稳定 required 名称和阈值检查通过 | 托管执行及远端保护设置保留 UNVERIFIED |
| AC4 | 本报告只认领实际执行项 | 主会话分别归集 coverage-rust、coverage-tauri、MSRV、hosted、原生运行时证据 |

共享知识回写由主会话负责，目标为 `.trellis/spec/ccr/backend/dependency-governance.md` 和五工具说明。以上命令与边界适用于 Claude Code、Codex、Grok Build、Kimi Code、OMP。没有提交、归档、推送、远端触发或发布。

## 2026-10-04 主会话聚合

迁移后 `just ci` 退出 0，TOTAL 15:06.415，16 步 OK。2509 个交付源文件前后哈希一致，`changed_source_paths` 为空，`source_bytes_preserved` 为 true。回执 `../09-29-evergreen-harness-audit/research/resume-2026-10-04-vsce4-full-ci.json`。AC4 只覆盖这次实际执行项。历史 FAIL 回执保留。hosted CI 仍为 UNVERIFIED。本段没有重跑 `just ci`。
