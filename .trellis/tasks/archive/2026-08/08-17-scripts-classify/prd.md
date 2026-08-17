# 根 scripts 分类与调用对齐

## Goal

把根目录 `scripts/` 按职责分成可导航的目录结构。依赖漂移与文档漂移各只保留一份 Python 实现。仓库内 just、CI、spec、文档路径全部改到新位置。门禁语义不变。

## Background

根 `scripts/` 现有 20 个已跟踪文件，全部平铺。职责已经分成版本同步、漂移检查、CI 治理、质量守卫，但目录、命名和包装层没有按同一套规则维护。

用户已确认：

- 只整理根 `scripts/`。
- 采用方案 3：按职责搬家、Python 用 `snake_case`、删掉依赖漂移薄包装、把 `check-doc-drift` 收成单一 Python。
- 不保留根目录兼容包装。调用点在同一变更里改完。

## Confirmed Facts

根目录已跟踪文件与职责：

| 当前文件 | 职责 | 实现形态 | 主要调用 |
|---|---|---|---|
| `version-sync.sh` / `version-sync.ps1` | 以根 `Cargo.toml` 同步各表面版本 | Bash / PowerShell 双实现 | `just version-sync` / `just version-check` |
| `version-sync.bats` / `version-sync.Tests.ps1` | 版本同步测试 | 与脚本同目录 | `just test-scripts` |
| `check-doc-drift.sh` / `check-doc-drift.ps1` | `ccr-ui/README.md`、Bun lock、Tauri MSRV 事实漂移 | 行为等价的双实现，无测试 | `just version-check` |
| `check_dependency_drift.py` | Root / Tauri 依赖版本、allowlist、MSRV、禁止 umbrella `ccr` 依赖 | 规范实现 | `just version-check`、`just dependency-governance-check` |
| `check-dependency-drift.sh` / `.ps1` | 启动上述 Python | 薄包装，11 行 / 14 行 | 同上 |
| `dependency-drift-allowlist.json` | 依赖漂移豁免 | JSON 元数据 | `check_dependency_drift.py`、`check_json_format.py` 清单 |
| `check_workflow_governance.py` | workflow pin、触发器、just 配方、serial 计数 | Python | `just workflow-governance-check` |
| `ci_surface_policy.py` | PR 路径是否命中 root/frontend/tauri/vscode | Python；被治理脚本 import | 四个 hosted workflow 的 change detection |
| `check_json_format.py` | 人工维护 JSON 两空格规范化 | Python | `just json-format` / `just json-format-check` |
| `check-secret-writes.py` | 凭据模块禁止直接 async write，AtomicWriter 必须 `.secret(true)` | Python，kebab-case 文件名 | `just lint-strict`、`just secret-write-check` |
| `check_coverage_thresholds.py` | llvm-cov JSON 总体 / gateway 行覆盖率 | Python | `just coverage-rust` / `just coverage-tauri` |
| `check-copilot-assets.mjs` | Copilot 工作区资产与命名 | Node | `just copilot-check` |
| `test_check_*.py` | 上述 Python 的 unittest | `from scripts.<mod> import` | just 配方在检查前先跑 unittest |
| `README.md` | 目录说明 | 未覆盖治理、覆盖率、敏感写入、Copilot；`SYNC_TARGETS` 缺 `ccr-db`、`ccr-vscode` | 人工阅读 |

调用面：根 `justfile`（约 40 处路径）、`.github/workflows/{ci,frontend-ci,tauri-rust-ci,vscode-ci}.yml`、Python 模块导入、治理测试写死的 `scripts/ci_surface_policy.py`、`.trellis/spec` 中的路径契约、`docs/guide/github-copilot-workspace.md` 及其英文页、`version-sync.bats` / `version-sync.Tests.ps1` 的夹具路径。

结构问题：扁平堆叠；kebab / snake 混用；包装层不对称；README 落后；`Path(__file__).parents[1]` 假设脚本位于 `scripts/` 下一层；本地 `__pycache__` 残留已删除的 `check_release_security.py`。

`ccr-ui/scripts/`、`docs/scripts/`、`.trellis/scripts/` 不在本任务范围。

## Requirements

- R1. 根 `scripts/` 按职责分成 `version/`、`drift/`、`ci/`、`quality/`。根层只留 `README.md` 与 Python 包入口（`__init__.py`、共享 `common.py`）。
- R2. Python 文件用 `snake_case`。shell / PowerShell / Node 文件保持 kebab-case。`check-secret-writes.py` 改为 `check_secret_writes.py`。
- R3. 删除 `check-dependency-drift.sh` 与 `check-dependency-drift.ps1`。just 与文档直接调用 `check_dependency_drift.py`。
- R4. 删除 `check-doc-drift.sh` 与 `check-doc-drift.ps1`。用一份 `check_doc_drift.py` 复现现有 sh/ps1 的检查项，并补 unittest。
- R5. 不保留根目录兼容包装或重导出。仓库内旧路径引用全部改到新路径。
- R6. just 配方名与 hosted workflow 行为保持不变。改变的是脚本路径，不是门禁语义。
- R7. `ci_surface_policy.py` 搬家后，改该文件仍使四个 surface 全部 relevant。frontend / vscode 继续只精确匹配该策略文件，不把整个 `scripts/**` 扩进这两个 surface。
- R8. 分类后的 Python 测试仍可从仓库根用 `python -m unittest` 运行。实现通过 `scripts.common.REPO_ROOT` 定位仓库根，不再使用 `parents[1]`。
- R9. `scripts/README.md` 成为当前分类的完整目录，并与 `version-sync` 的 `SYNC_TARGETS` 对齐。
- R10. 所有仓库内调用路径一并更新：`justfile`、四个 hosted workflow、Python import、脚本互引、测试夹具、相关 docs、相关 `.trellis/spec`。

## Acceptance Criteria

- [ ] AC1. 根 `scripts/` 顶层不再放置职责脚本或测试。职责脚本位于 `version/`、`drift/`、`ci/`、`quality/`。对应 R1。
- [ ] AC2. 仓库内不存在 `check-dependency-drift.sh`、`check-dependency-drift.ps1`、`check-doc-drift.sh`、`check-doc-drift.ps1`、`check-secret-writes.py`。对应 R2、R3、R4、R5。
- [ ] AC3. `just version-check`、`just dependency-governance-check`、`just workflow-governance-check`、`just json-format-check`、`just secret-write-check` 在新路径下通过。对应 R3、R4、R6、R10。
- [ ] AC4. `python -m unittest scripts.drift.test_check_doc_drift` 覆盖：缺文件、存在 `package-lock.json`、README 缺事实、README 含过期描述、正常通过。对应 R4、R8。
- [ ] AC5. 四个 hosted workflow 的 change-detection 指向 `scripts/ci/ci_surface_policy.py`。`is_relevant(surface, ["scripts/ci/ci_surface_policy.py"])` 对四个 surface 均为 true。对应 R7。
- [ ] AC6. 对旧脚本路径的检索（`scripts/check-doc-drift`、`scripts/check-dependency-drift.`、`scripts/check-secret-writes.py`、`scripts/check_json_format.py`、`scripts/ci_surface_policy.py`、`scripts/version-sync.`）在仓库内无存活引用。历史审计报告 `docs/reports/ccr_code_audit_canvas.md` 除外。对应 R5、R10。
- [ ] AC7. `scripts/README.md` 按四个分类列出入口命令，并写明 `SYNC_TARGETS` 含 `ccr-db` 与 `ccr-vscode`。对应 R9。

## Out of Scope

- 整理或并入 `ccr-ui/scripts/`、`docs/scripts/`。
- 修改 `.trellis/scripts/`。
- 把 `version-sync` 收成单一 Python 或 Rust CLI 子命令。
- 改变覆盖率阈值、allowlist 内容、workflow pin、secret-write 扫描文件列表。
- 改写历史审计报告 `docs/reports/ccr_code_audit_canvas.md`。
- 提交本地 `__pycache__`。

## Key Decisions

- 范围：只做根 `scripts/`。
- 深度：方案 3。搬家 + Python `snake_case` + 删除依赖漂移薄包装 + 文档漂移收成 Python。
- 兼容：不留根目录包装。本仓库调用点一次性改完。
- 分类名：`version/`、`drift/`、`ci/`、`quality/`。

## Notes

- 规格层已把当前扁平路径写成契约。搬家时必须改 spec。
- 详细目录、数据流与回归命令见 `design.md`、`implement.md`。
