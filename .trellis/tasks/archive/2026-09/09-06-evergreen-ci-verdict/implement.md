# 执行计划

## 文件白名单

- .github/workflows/vscode-ci.yml
- scripts/ci/ci_surface_policy.py
- scripts/ci/test_check_workflow_governance.py

## 顺序

独立于 UI/OMP；修改完成后向规则子任务交付检查退出码和精确路径映射。

- [x] 获取本子项的明确实施批准，检查最新计划，再 task.py start。
- [x] 阅读 implement.jsonl/check.jsonl 的真实规范、父报告和 design.md。
- [x] 重现对应失败/缺口，实施最小变更，保留有意义的正负回归。
- [x] 逐条运行下列检查，记录命令/cwd/退出码；预期失败的负例以非零为通过。
- [x] 强模型独立审查，向父任务/规则子任务交付适用工具与证据；缺证据明确列出。

## 检查

~~~text
python -X utf8 -m unittest scripts.ci.test_check_workflow_governance
python -X utf8 scripts/ci/check_workflow_governance.py
bash --noprofile --norc -eo pipefail -c 'false | tee /dev/null'  # 预期 exit 1
bash --noprofile --norc -eo pipefail -c 'true | tee /dev/null'  # 预期 exit 0
just vscode-coverage
~~~

有生成/安装副作用的聚合命令先检查组成步骤；仅使用已有依赖。本地检查不证明当前 hosted 或真实客户端通过。必要检查缺条件时该项验收保持未完成，不能用其他命令的通过替代。

## 完成和权限

本项 AC 与父任务已批准集的交付约束均满足才算完成。用户 `/goal` 已批准本子任务在 check PASS 后提交并归档；仍不发布、不编辑用户全局文件。不新增 CI 引擎，不改依赖版本，不远程触发 workflow，不改变 coverage 阈值。

## 本地检查记录（implement）

cwd 均为仓库根 `D:\Documents\Code\Github\ccr`。hosted workflow 未远程触发。

- `python -X utf8 -m unittest scripts.ci.test_check_workflow_governance` → exit 0（24 tests OK）
- `python -X utf8 scripts/ci/check_workflow_governance.py` → exit 0
- `bash --noprofile --norc -eo pipefail -c "false | tee /dev/null"` → exit 1（预期失败，记为通过）
- `bash --noprofile --norc -eo pipefail -c "true | tee /dev/null"` → exit 0
- `just vscode-coverage` → exit 0（line 91.86%，阈值未改）

AC2 `is_relevant` 实证：`.cargo/tauri-ci.toml` 仅 tauri；`.cargo/config.toml` 为 root+tauri；`.cargo/audit.toml` 仅 root；frontend/vscode 对三文件均为 False。

check 2.2：强模型独立审查 PASS；见 `check-notes.md`。未证明当前 GitHub hosted 默认 shell / branch-protection（UNVERIFIED）。
