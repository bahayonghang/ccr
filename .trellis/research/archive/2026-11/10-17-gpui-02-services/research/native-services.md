# 02 native service verification

Date: 2026-10-18. Windows host, real GPUI window, main-session Computer Use only.
This report is in progress. It is not a performance or full native matrix pass.

## Data and launch

The domain API-generated fixture and hashes are recorded in
`outputs/desktop/services/native-seed-receipt.json`. Profiles use the logical
profiles.toml input; ConfigManager reads its actual profiles.db. Both launch
arguments are absolute fixture paths. No HOME/USERPROFILE override, real account,
network authentication or user configuration mutation was used.

Initial launches invoked the built debug exe with `--fixture-profiles` and
`--fixture-registry`; the second added `--fixture-read-delay-ms 30000`.
Their PowerShell wrapper did not explicitly propagate the native exit code.
Consequently its exit 0 is **not** accepted as a native exit receipt. This
collection mistake was corrected with the process-owning launcher below.

## Actual observations

Screenshots: `outputs/desktop/services/native/windows/`.

| Observation | Screenshot / result |
| --- | --- |
| Real catalog rows in domain order, z-last disabled, a-first current through registry fallback | config-list.png |
| Pointer selects current row and shows masked credential | config-current-masked.png |
| Up changes selection to z-last; model and masked credential change accordingly | config-keyboard-masked.png |
| Filter a-first yields one matching row and clears the disappeared selection | config-filtered.png |
| Controlled delay visibly labeled Demo data / controlled slow read with reading state | config-slow-loading.png |
| Filter text changes while background read remains pending | config-slow-filter-responsive.png |
| Ctrl+A visibly selects entered filter text while reading state remains | config-slow-selection.png |

First literal input attempt did not appear; after reobserving and explicitly
activating the exact target window, retry succeeded. This is recorded rather
than inferred to be an application timing failure. The helper groups the child
under the host app identifier, so selection used a unique exact CCR window title
and the captured image before any input. No host UI was operated.

All captures explicitly requesting both screenshot and accessibility text still
returned `accessibility: null`. This is **not** proof of working roles/states or
screen-reader support. Kit disabled-state semantics remain an explicit 03 gap.

These are qualitative interaction observations, not p95 response measurements.
Captured decorated bounds were 1282x831; effective target DPI was not measured.
No 100/150/200% or four-size matrix, light/dark matrix, new IME verification,
macOS/Linux native result or performance result is claimed here.

## Close failure and exact exit collection

During normal Alt+F4 closure of the second fixture window, captured command output
contained:

```text
thread '<unnamed>' (35952) panicked at accesskit_windows-0.34.0/src/util.rs:380:52:
called `Result::unwrap()` on an `Err` value:
Error { code: HRESULT(0x80070578), message: "Invalid window handle." }
panic in a function that cannot unwind
```

The tool output only contained the beginning of the stack; no complete stack is
invented. A process query afterwards found no ccr-gpui processes. Because the
launch wrapper did not own the actual native exit code, this run is a failed
close observation with native exit **unknown**, not a successful close.

The corrected launcher is retained verbatim at
`outputs/desktop/services/launch-native-fixture.ps1`. It uses ProcessStartInfo
ArgumentList, child-only RUST_BACKTRACE, redirected stdout/stderr, WaitForExit,
and the actual Process.ExitCode; it records the executable hash before launch.
CreateNoWindow suppresses a console only; the requested native GUI remains visible.
It refuses to overwrite existing run receipts and does not override HOME.

`launch-native-fixture.ps1 -RunName close-repro -DelayMs 30000` was actually run:
native exit **0**, empty stderr, no remaining process. Receipt and logs are in
`outputs/desktop/services/native/close-repro/`. SHA256 was
`daae9c7a5d037835951a12a79fd808207a55a560057797776a344d4eefbf48ed`.
The run lasted about 45 seconds, so its close does not prove cancellation during
the initial delay. Its one successful close does not erase the earlier race.

Exact upstream-source diagnosis is in [native-close-race.md](native-close-race.md).
A narrow independently owned AccessKit patch and deterministic invalid-HWND
regression are in progress. Native close acceptance remains open until the
patched artifact and repeated real UIA/close verification are recorded.

## Patched artifact and interrupted native continuation

The implementation owner produced a repository-owned narrow AccessKit Windows
patch. Its deterministic stock regression failed with the real invalid-window
error; the fixed three-test set passed, including four provider coordinate paths
while Context remains retained. Source, provenance, lock scope and actual
commands are recorded in [accesskit-close-fix.md](accesskit-close-fix.md).
This does not substitute for patched native exit verification.

Patched debug artifact SHA256:
`dd53feb570608970a1ce7cced1996399f541244b912d87afa8042988a38bf185`.
The process-owning launcher independently recorded this actual hash on both
following attempts:

- `patched-normal`: Computer Use reported the physical Escape stop before the
  first target capture. Main stopped UI input, verified the exact exe path,
  fixture command line and fresh PID identity, then force-stopped only that
  owned fixture process. Actual native exit -1, launcher exit 1, **interrupted**.
- User explicitly requested resuming Computer Use and focusing on core functions.
  `patched-core-normal` was launched, but the very first helper call again
  immediately returned the physical Escape stop state. No further Computer Use
  call was made after that response. The same scoped non-UI cleanup removed the
  owned fixture process. Actual native exit -1, launcher exit 1, **interrupted**.

Receipts/logs are retained in the respective
`outputs/desktop/services/native/{patched-normal,patched-core-normal}/` folders;
separate `*-interrupted.json` files record why forced cleanup occurred. These
are neither product crash failures nor successful normal-close receipts. The
patched normal close, error/retry and close-during-read scenarios remain unrun.

Latest user steering: verify core functional flows only; do not spend this
session on resizing/scaling/DPI matrices. Those matrices are user-deferred,
not passed. Core reads/filtering, error/retry, cancellation/close and later
edit/CAS/save/apply lifecycle remain the continuation priority. Product scope
and previously approved implementation authorization are unchanged.

Final command-layer integration subsequently rebuilt the normal debug artifact:
SHA256 `94acabe375bc032da8cf868934eae6fb2de4392ab55cd7440849e3e9b84fc741`.
It supersedes DD53 above. Default-PATH desktop check/test/lint/debug build passed,
with the actual toolchain paired in the child and target remaining local.
This final artifact has not been launched for a new native pass; the interrupted
DD53 receipts remain accurate historical records, not proof for the new binary.

## Core native acceptance after explicit continuation

The user explicitly resumed Computer Use and limited verification to core flows.
A fresh JS kernel restored the helper; no stop boundary was bypassed. During the
first run, concurrent user input/minimization triggered the input guard. Main
stopped affected actions, asked only for keyboard/mouse coordination, and resumed
after the user explicitly restored the CCR window and paused input. An unrelated
foreground capture was discarded; no action was taken on it or image retained.

All following receipts independently identify final artifact **94acabe3...fc741**,
and all three native process exits are **0**, stderr trimmed empty, no remaining
ccr-gpui process. Summary: `outputs/desktop/services/native/core-resume-summary.json`.

| Run | Actual core result |
| --- | --- |
| core-resume-normal | Two real domain rows/current registry fallback; zz-no-match yields explicit empty; replacing with a-first yields one row; selecting it shows masked credential. Normal Alt+F4 exits 0. |
| core-resume-error | Missing catalog shows a real error, not empty success; query does not create the missing directory. Main copies only the previously domain-generated synthetic DB into the explicitly owned missing fixture path. Clicking Refresh recovers two rows/current state without restarting; normal Alt+F4 exits 0. |
| core-resume-cancel | Explicit 30000ms fixture delay shows pending read. Alt+F4 is sent from that observed reading state; process exits 0 after 28.52s total lifetime, before the delay could naturally finish. This exercises read cancellation on view/window close plus shutdown join; it does not claim a separate job Cancel button or latency-budget measurement. |

Screenshots in `outputs/desktop/services/native/windows-core/`: read-list.png,
filter-empty.png, filter-match.png, read-masked-detail.png,
error-missing-catalog.png, error-retry-recovered.png, cancel-pending-read.png.
The error run also retains recovery.json identifying the copied synthetic DB.
No real configuration/account, global HOME or user file was changed.

These new runs supersede the pending patched **core close** result above; the
earlier interrupted and crash observations remain historical evidence. Repeated
native UIA requests did not reproduce the provider panic in these three runs,
and the deterministic invalid-HWND tests cover its identified mechanism. Native
accessibility text still returns null, so screen-reader semantics remain a 03
item rather than a claimed pass. Scaling/DPI tests remain user-deferred, and no
performance, packaging or other-OS acceptance is inferred.
