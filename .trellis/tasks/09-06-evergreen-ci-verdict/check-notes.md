# Check notes (2.2)

Verdict: **PASS**

Applicable tools: Codex/Claude shell + source review for CI; this check ran in Cursor with shell (not Grok built-in plan/explore). Hosted workflow was not remotely triggered.

## Ledger

| id | severity | path_or_scope | issue | fix_required | test_required | status | evidence |
|---|---|---|---|---|---|---|---|
| F2 | P1 | `.github/workflows/vscode-ci.yml` coverage step | `just vscode-coverage \| tee` without pipefail; default Linux `bash -e` can swallow producer failure | yes | yes | fixed | Step is `run: just vscode-coverage \| tee vscode-coverage.txt` plus `shell: bash`. GitHub maps explicit `bash` to `bash --noprofile --norc -eo pipefail {0}` (unspecified remains `bash -e {0}`). Artifact upload stays `if: always()`. Thresholds unchanged (`ccr-vscode/justfile` lines 70). |
| F3 | P1 | `scripts/ci/ci_surface_policy.py` | `.cargo/tauri-ci.toml`, `.cargo/config.toml`, `.cargo/audit.toml` were `is_relevant` False for all surfaces | yes | yes | fixed | Exact paths only; no `.cargo/**`. Mapping: tauri-ci.toml → tauri; config.toml → root+tauri; audit.toml → root; frontend/vscode False for all three. `cargo audit` remains root `ci.yml` only. |
| F2a | P2 | `scripts/ci/test_check_workflow_governance.py` Windows WSL bash | First parallel `unittest` failed `true \| tee /dev/null` (return 1) because `shutil.which("bash")` is `C:\Windows\System32\bash.exe` (WSL) | no | no | wontfix | Isolated re-run OK (24 tests, then the single pipefail test 0.209s). Direct shell probes were 1/0 as expected. Hosted vscode-ci is ubuntu-24.04; this is local WSL contention, not the coverage step contract. |

No F3a.

## Product diff vs whitelist

Only the three implement.md files: `vscode-ci.yml` +1 (`shell: bash`), `ci_surface_policy.py` +4 (exact cargo paths), `test_check_workflow_governance.py` +114 (AC2 matrix + coverage step/shell + local pipe probes). Coverage thresholds not changed. `.cargo/**` not added.

## Commands (cwd `D:\Documents\Code\Github\ccr`)

| command | exit |
|---|---|
| `python -X utf8 -m unittest scripts.ci.test_check_workflow_governance` (first, parallel with other bash) | 1 (`true \| tee` return 1 under concurrent WSL) |
| `python -X utf8 -m unittest scripts.ci.test_check_workflow_governance` (isolated) | 0 (24 tests) |
| `python -X utf8 scripts/ci/check_workflow_governance.py` | 0 (45 immutable actions; serial 0) |
| `bash --noprofile --norc -eo pipefail -c "false \| tee /dev/null"` | 1 (expected) |
| `bash --noprofile --norc -eo pipefail -c "true \| tee /dev/null"` | 0 (expected) |
| `just vscode-coverage` | 0 (line 91.86%, functions 91.50%; threshold 70 unchanged) |

AC2 live `is_relevant`: `.cargo/tauri-ci.toml` → tauri; `.cargo/config.toml` → root+tauri; `.cargo/audit.toml` → root; `.cargo/**` absent.

## UNVERIFIED

- Current GitHub-hosted ubuntu-24.04 run of this branch (workflow not remotely triggered).
- Branch-protection required contexts on `main`/`develop`/`dev`.
- GitHub-hosted Windows `workflow-governance-check` using whatever `bash` is first on PATH.
