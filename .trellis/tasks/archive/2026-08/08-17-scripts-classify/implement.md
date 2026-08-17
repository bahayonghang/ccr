# 根 scripts 分类实施清单

## 顺序

1. 新增 `scripts/__init__.py` 与 `scripts/common.py`（`REPO_ROOT`）。
2. 新建 `version/`、`drift/`、`ci/`、`quality/`，放入对应 `__init__.py`。
3. `git mv` 现有文件到目标目录；`check-secret-writes.py` 改为 `quality/check_secret_writes.py`。
4. 全部 Python 检查器与测试改为 `from scripts.common import REPO_ROOT`，并更新包导入。
5. 新增 `drift/check_doc_drift.py` 与 `drift/test_check_doc_drift.py`，契约见 `design.md`。
6. 修正 `version-sync.sh` / `.ps1` / `.bats` / `.Tests.ps1` 的仓库根与复制路径。
7. 更新 `check-copilot-assets.mjs` 的自路径跳过、`JSON_CONFIG_PATHS`、allowlist 路径、`SURFACE_PATHS` 精确路径。
8. 更新根 `justfile` 全部脚本路径；漂移检查改为直接调 Python。
9. 更新四个 hosted workflow 的 `ci_surface_policy.py` 路径。
10. 更新 `scripts/README.md` 与设计中列出的 spec / docs。
11. 删除四个包装/双实现文件：`check-dependency-drift.sh`、`check-dependency-drift.ps1`、`check-doc-drift.sh`、`check-doc-drift.ps1`。
12. 检索旧路径，清掉存活引用（历史审计报告除外）。

## 验证

按依赖从窄到宽：

```text
python -m unittest scripts.quality.test_check_json_format
python -m unittest scripts.drift.test_check_dependency_drift
python -m unittest scripts.drift.test_check_doc_drift
python -m unittest scripts.ci.test_check_workflow_governance
just json-format-check
just secret-write-check
just version-check
just workflow-governance-check
just dependency-governance-check
just copilot-check
```

`just test-scripts` 在本机有 Pester / bats 时再跑。最终接受用 `just ci`。

Windows 配方走 `python`，Linux / macOS 走 `python3`，与现有 justfile 一致。

## 风险文件

- `justfile`：路径漏改会在 CI 第一批治理步骤失败。
- `scripts/ci/ci_surface_policy.py` 与四个 workflow：精确路径错误会改变 surface 触发。
- `scripts/version/version-sync.sh` 与 `.ps1`：仓库根多一级或少一级会写错文件。
- `.trellis/spec/ccr/backend/dependency-governance.md`：签名与验证命令必须与新路径一致，否则后续任务按旧契约改脚本。

## 回滚点

- 步骤 3 之后、步骤 8 之前：工作树半搬家，不要提交。
- 步骤 11 之后：包装已删，必须同时提交 just / workflow / spec，不能拆成「先删后改」。

## `task.py start` 前确认

- `prd.md` 无未决开放问题。
- `design.md` 与 `implement.md` 已存在。
- 用户已批准本轮规划摘要。
