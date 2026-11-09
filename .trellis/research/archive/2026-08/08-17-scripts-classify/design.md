# 根 scripts 分类设计

## 目标树

```
scripts/
  README.md
  __init__.py
  common.py
  version/
    version-sync.sh
    version-sync.ps1
    version-sync.bats
    version-sync.Tests.ps1
  drift/
    __init__.py
    check_dependency_drift.py
    test_check_dependency_drift.py
    check_doc_drift.py
    test_check_doc_drift.py
    dependency-drift-allowlist.json
  ci/
    __init__.py
    ci_surface_policy.py
    check_workflow_governance.py
    test_check_workflow_governance.py
  quality/
    __init__.py
    check_json_format.py
    test_check_json_format.py
    check_secret_writes.py
    check_coverage_thresholds.py
    check-copilot-assets.mjs
```

根层 `scripts/__init__.py` 为空包标记，供 `from scripts.quality.check_json_format import ...` 使用。`version/` 只有 shell / PowerShell，不需要 Python 包。

## 命名

- Python 模块与测试：`snake_case`。`check-secret-writes.py` → `quality/check_secret_writes.py`。
- shell / PowerShell / Node：保持 kebab-case。`version-sync.sh`、`check-copilot-assets.mjs` 不改文件名，只改目录。
- 测试与实现对齐：`test_<module>.py` 与实现同目录。

## 共享仓库根

所有现有 Python 检查器用 `Path(__file__).resolve().parents[1]`。文件进入子目录后该表达式会指到 `scripts/`，不是仓库根。

`scripts/common.py` 提供唯一入口：

```python
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
```

Python 检查器与测试改为 `from scripts.common import REPO_ROOT`。禁止在子目录脚本里再写 `parents[1]` / `parents[2]`。

## 文档漂移收口

`check-doc-drift.sh` 与 `check-doc-drift.ps1` 检查项相同，解析方式不同（jq/sed vs `ConvertFrom-Json`）。新 `scripts/drift/check_doc_drift.py` 用标准库复现同一契约：

1. 下列文件必须存在：`ccr-ui/README.md`、`ccr-ui/package.json`、`ccr-ui/bun.lock`、`ccr-ui/src-tauri/Cargo.toml`。
2. `ccr-ui/package-lock.json` 存在则失败。
3. `package.json` 的 `version` 非空；`packageManager` 匹配 `^bun@[0-9]`。
4. Tauri Cargo.toml 含 `rust-version` 与 `edition`。
5. README 必须包含现有 9 条事实针（版本 badge、Bun-only、`bun.lock`、packageManager 单元格、Rust MSRV / edition、`Tauri invoke APIs`、`Web runtime`、`bun run lint:fix`）。
6. README 不得包含现有 10 条过期描述。
7. 支持 `--verbose` / `-v`。退出码 0 / 1 与现脚本一致。

`test_check_doc_drift.py` 用临时目录注入假根，不读真实 `ccr-ui/`。

## 删除的包装

- `check-dependency-drift.sh` / `.ps1`：just 各 OS 配方改为与 `json-format` 相同的 `python` / `python3` 直接调用。
- `check-doc-drift.sh` / `.ps1`：同上，改为 `scripts/drift/check_doc_drift.py`。

根目录不留转发脚本。

## 仓库根定位（非 Python）

- `version-sync.sh`：`dirname` 后再上两级（`scripts/version/` → 仓库根）。
- `version-sync.ps1`：`Split-Path` 三次，或与 bash 一样从脚本目录上两级。
- `version-sync.bats`：从测试文件定位仓库根改为上两级；复制路径改为 `scripts/version/version-sync.sh`；`sed` 替换式与新的 `ROOT_DIR` 赋值一致。
- `version-sync.Tests.ps1`：复制到夹具的 `scripts/version/`，执行路径同步。

## Python 导入

| 旧 | 新 |
|---|---|
| `from scripts.check_json_format import ...` | `from scripts.quality.check_json_format import ...` |
| `from scripts.check_dependency_drift import ...` | `from scripts.drift.check_dependency_drift import ...` |
| `from scripts.ci_surface_policy import ...` | `from scripts.ci.ci_surface_policy import ...` |
| `from scripts.check_workflow_governance import ...` | `from scripts.ci.check_workflow_governance import ...` |

`check_workflow_governance.py` 保留「包导入失败则同目录导入」回退，回退目标改为同目录的 `ci_surface_policy`。

just 中的 unittest 入口改为模块路径，例如 `python -m unittest scripts.quality.test_check_json_format`。

## CI 表面匹配

`SURFACE_PATHS`：

- `root` / `tauri` 继续用 `scripts/**`。
- `frontend` / `vscode` 的精确路径改为 `scripts/ci/ci_surface_policy.py`。不要改成 `scripts/**`，否则任意根脚本改动都会拉起 Frontend / VS Code CI。
- 治理测试 `is_relevant(surface, ["scripts/ci/ci_surface_policy.py"])` 对四个 surface 必须为 true。

四个 workflow 的 change-detection 命令改为：

`python3 scripts/ci/ci_surface_policy.py --surface <name> --base "$BASE_SHA" --head "$HEAD_SHA"`

## JSON 清单与自检路径

- `JSON_CONFIG_PATHS` 中的 allowlist 改为 `scripts/drift/dependency-drift-allowlist.json`。
- `check_dependency_drift.py` 的 `ALLOWLIST` 指向同一新路径。
- `check-copilot-assets.mjs` 跳过自身的相对路径改为 `scripts/quality/check-copilot-assets.mjs`。

## 调用面清单

必须改路径的存活引用：

- 根 `justfile`：`json-format*`、`lint-strict`、`secret-write-check`、`copilot-check`、`version-sync`、`version-check`、`workflow-governance-check`、`dependency-governance-check`、`coverage-*`、`test-scripts`
- `.github/workflows/{ci,frontend-ci,tauri-rust-ci,vscode-ci}.yml`
- `scripts/README.md`、`docs/guide/github-copilot-workspace.md`、`docs/en/guide/github-copilot-workspace.md`
- `.trellis/spec/ccr/backend/dependency-governance.md`
- `.trellis/spec/ccr/backend/repository-json-formatting.md`
- `.trellis/spec/ccr/backend/test-fixtures.md`
- `.trellis/spec/ccr-core/backend/atomic-writer.md`
- `.trellis/spec/ccr-codex/backend/codex-provider-bearer-runtime.md`
- `.trellis/spec/ccr-vscode/frontend/extension-surface-contracts.md`

`code_map.md` / `AGENTS.md` 只写 `scripts/` 目录，不写具体文件，可只在 README 写分类。

`docs/reports/ccr_code_audit_canvas.md` 是历史审计快照，不改。

## 回滚

一次提交内完成搬家与调用改写。回滚该提交即可回到扁平布局。不要做「先留包装再删」的两阶段发布。

## 风险

- `version-sync` 双实现的仓库根计算若少上一级，同步会写到错误目录。bats / Pester 必须先改再跑。
- 漏改 `from scripts.<mod>` 会让 `just json-format-check` 或治理检查在 import 阶段失败。
- frontend / vscode 若误扩成 `scripts/**`，会改变 hosted CI 触发面。
