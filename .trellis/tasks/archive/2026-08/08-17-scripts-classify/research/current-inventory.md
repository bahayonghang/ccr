# 根 scripts 盘点（2026-08-17）

仓库证据。实现时以工作树为准。

## 根目录已跟踪文件

```
scripts/README.md
scripts/check-copilot-assets.mjs
scripts/check-dependency-drift.ps1
scripts/check-dependency-drift.sh
scripts/check-doc-drift.ps1
scripts/check-doc-drift.sh
scripts/check-secret-writes.py
scripts/check_coverage_thresholds.py
scripts/check_dependency_drift.py
scripts/check_json_format.py
scripts/check_workflow_governance.py
scripts/ci_surface_policy.py
scripts/dependency-drift-allowlist.json
scripts/test_check_dependency_drift.py
scripts/test_check_json_format.py
scripts/test_check_workflow_governance.py
scripts/version-sync.Tests.ps1
scripts/version-sync.bats
scripts/version-sync.ps1
scripts/version-sync.sh
```

## 已批准分类

| 分类 | 成员 | 备注 |
|---|---|---|
| `version/` | `version-sync.{sh,ps1,bats,Tests.ps1}` | 双实现保留 |
| `drift/` | `check_dependency_drift.py`、`check_doc_drift.py`、allowlist、对应测试 | 删除 sh/ps1 包装与文档漂移双实现 |
| `ci/` | `ci_surface_policy.py`、`check_workflow_governance.py`、对应测试 | frontend/vscode 仍精确匹配策略文件 |
| `quality/` | `check_json_format.py`、`check_secret_writes.py`、`check_coverage_thresholds.py`、`check-copilot-assets.mjs`、对应测试 | Python 改为 snake_case |

根层只留 `README.md`、`__init__.py`、`common.py`。不留兼容包装。

## 调用锚点

- `justfile`：`json-format`、`json-format-check`、`lint-strict`、`secret-write-check`、`copilot-check`、`version-sync`、`version-check`、`workflow-governance-check`、`dependency-governance-check`、`coverage-rust`、`coverage-tauri`、`test-scripts`
- `.github/workflows/ci.yml:29`
- `.github/workflows/frontend-ci.yml:25`
- `.github/workflows/tauri-rust-ci.yml:25`
- `.github/workflows/vscode-ci.yml:25`
- Python import：`scripts.check_json_format`、`scripts.check_dependency_drift`、`scripts.ci_surface_policy`、`scripts.check_workflow_governance`
- `scripts/ci_surface_policy.py` 把 `scripts/ci_surface_policy.py` 列为 frontend/vscode 精确路径；root/tauri 用 `scripts/**`
- `scripts/check_json_format.py` 清单含 `scripts/dependency-drift-allowlist.json`
- `version-sync.bats:84-85` 从脚本目录 `cd ..` 再取 `scripts/version-sync.sh`
- `check-dependency-drift.sh:11` 写死 `$ROOT_DIR/scripts/check_dependency_drift.py`

## 规格路径（搬家必须同步）

- `.trellis/spec/ccr/backend/dependency-governance.md`
- `.trellis/spec/ccr/backend/repository-json-formatting.md`
- `.trellis/spec/ccr-core/backend/atomic-writer.md`
- `.trellis/spec/ccr-vscode/frontend/extension-surface-contracts.md`
- `.trellis/spec/ccr-codex/backend/codex-provider-bearer-runtime.md`
- `.trellis/spec/ccr/backend/test-fixtures.md`

## 范围决定

只整理根 `scripts/`。下列目录不改。

## 未纳入本任务的脚本树

- `ccr-ui/scripts/` 24 文件，由 `ccr-ui/package.json` 与 UI justfile 拥有
- `docs/scripts/audit-docs.mjs`
- `.trellis/scripts/` Trellis 运行时
