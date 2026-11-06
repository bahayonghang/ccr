# Independent Trellis check

Scope: task-owned Codex model/service/runtime/platform changes, CLI auth-off
parser/default, save handler, binary regressions, and contract docs. Reviewer
was read-only while implementation ran Cargo checks; no competing builds.

## Finding resolved

P1: returning `AuthState.reason` from save's Invalid branch could expose a JSON
string-root credential through serde's map-deserialization error. Implementer
changed `load_auth_raw_map` to fixed text plus line/column and added a real CLI
string-root secret sentinel regression. Reviewer re-read and accepted the fix.

## Review result

- No remaining blocking code findings.
- All six production store-resolution callers preserve invalid TOML types.
- Explicit auto/keyring remains rejected by service and CLI even with auth.json.
- `save_current_content` still validates captured snapshot bytes.
- Errors propagate to the dispatcher once, with failure status.
- `git diff --check`: PASS.
- Narrowed new spec wording to raw-auth/save-preflight parsing; unrelated typed
  deserialization paths were not claimed to be fixed by this task.
- Cargo tests and lint are covered by implementer/main-session receipts, not an
  independent second execution by this reviewer.
