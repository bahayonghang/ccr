
## 2026-10-28 populated native checkpoint

Selected real native populated read/filter/page/detail/virtual-scroll actions and exit0 are recorded in research/native-populated-20261028.md, tied to immutable baseline843f04. New archive refresh and policy consumers require their own current-source gates/new candidate. Full49-command and parent acceptance remain open; no task completion.

## 2026-11-05 local read slice

PRD 7.1–7.6 are not fully passed. This slice only connects the three sidebar routes to existing local reads. No performance budget, three-OS matrix, Tauri parity, native screenshot, or archive refresh was run. The GPUI window was not launched. `crates/ccr-usage` was not edited and `cargo test -p ccr-usage` was not required.

Read-only sources on this machine:

- Usage database: `C:\Users\lyh\.llmusage\llmusage.db` (schema_version 24, 59,043,840 bytes). Opened with `ccr_usage::open_dashboard`. File length was unchanged after the read.
- Session archive: `C:\Users\lyh\.ccr\analytics\usage.db` (417 `usage_session_archive` rows). Counted with SQLite `mode=ro`. The desktop archive pool was not opened, so its WAL pragma did not run.

`/` default 7-day window, local dates 2026-10-30 through 2026-11-05:

- Requests 2,957. Tokens 472,285,568. Stored cost 264.9791182, shown as $264.98.
- Claude 2,084 requests / 379,897,158 tokens. Codex 873 / 92,388,410. Antigravity and OpenCode have no rows in this window.
- Insights bounds for the session card are `2026-10-30 04:00:00 UTC` .. `2026-11-06 05:00:00 UTC`. That archive range contains 139 sessions. The archive is indexed, so the card is not an unindexed dash.
- 30-day and 90-day windows cover the whole database (earliest bucket 2026-10-19). All-history totals are 16,256 requests, 2,882,413,684 tokens, and cost 1,021.42082559. Stored `antigravity_ide` (2,132 requests) and `omp` (55) fold onto the Antigravity and Pi cards. Their daily bars stay empty and the caption counts those requests as not drawn.

`/usage` with empty date filters reads the same database with no date bound. Source rows keep the stored counts and show canonical labels: claude, codex, antigravity, grok, pi. A detail line keeps the stored id when it differs.

`/agent-sessions` reads the existing archive. 417 indexed rows: grok 119, antigravity 111, claude 104, codex 46, omp 33, pi 4. The first page requests 100 rows and says more follow. It does not start archive refresh. Provider availability is discovered from the user home when the page loads; that scan was not executed here. A missing archive stays "not indexed" and does not render a zero-session list.

Budget and pricing are not mounted on these three routes. The solid paper theme was not changed.

Checker re-run on 2026-11-05, after making `/`, `/usage`, and `/agent-sessions` status text use `locales::is_chinese` (missing language stays English). PRD 7.1–7.6 stay open. No production sync and no archive refresh were run.

Final commands, each exit 0:

- `just desktop-fmt-check` — exit 0
- `just desktop-lint` — exit 0
- `just desktop-test` — exit 0
  - Python unittest: 12 passed
  - AccessKit harness: 3 passed
  - `ccr-gpui` lib tests: 60 passed (`running 60 tests`, `60 passed; 0 failed`)
  - `native_shell`: 6 passed
  - `native_ui`: 2 passed

`archive_candidate_rows_are_virtualized_and_keyboard_selection_uses_opaque_ids` and `rendered_archive_drill_in_pages_real_provider_messages_and_clears_plaintext` both passed in that lib run. An earlier note that this suite was 60 tests was not tied to the 58-test green run; the count above is this re-run only.
