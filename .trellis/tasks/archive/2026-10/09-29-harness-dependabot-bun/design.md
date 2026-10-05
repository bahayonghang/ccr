# T03 设计

## 责任与方案
仅将 /ccr-ui 和 /docs 的生态映射改为 Bun；/ccr-vscode 继续 npm，两个 Rust lockfile 继续 cargo。在现有治理脚本加入 packageManager、lockfile 和 Dependabot 目录的对照，不增加独立 CI 引擎。保留原调度、标签和安全策略。

## 文件白名单
- `.github/dependabot.yml`
- `scripts/ci/check_workflow_governance.py`
- `scripts/ci/test_check_workflow_governance.py`
- `.trellis/spec/ccr/backend/dependency-governance.md`

## 兼容与回滚
保持既有命令、共享接口和数据安全规则；所列新增脚本均为检查器或测试。提交实施前重新核对当前文件和基线。回滚只撤销本任务差异，不 reset/clean 或覆盖他人工作。生成/ignored 文件的交付边界必须由父任务强模型复核。

## 依赖
无前置子任务。父子层级不代替执行顺序。
