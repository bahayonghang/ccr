# T06 设计

## 责任与方案
沿用 just 和现有工作流。将 just ci 的 version-sync、fmt 移至显式维护命令；ci 仅检查，保留 version-check、fmt-check。聚合检查接入 frontend-audit、OMP、harness 和 Copilot 契约及最终包清单，并对 root/Tauri 两个 lockfile 进行审计。UI 构建检查使用已提交图标，图标生成保留独立命令。缺少审计工具时给出前置要求，不在只读聚合命令中安装全局工具。维持默认测试并行、现有 coverage 阈值和 required context 名称。

## 文件白名单
- `justfile`
- `.github/workflows/ci.yml`
- `.github/workflows/frontend-ci.yml`
- `.github/workflows/vscode-ci.yml`
- `scripts/ci/ci_surface_policy.py`
- `scripts/ci/check_workflow_governance.py`
- `scripts/ci/test_check_workflow_governance.py`
- `.trellis/spec/ccr/backend/dependency-governance.md`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
T01、T02、T03、T04、T05、T08。父子层级不代替执行顺序。
