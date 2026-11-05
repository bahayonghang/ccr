# Command workbench checkpoint — 2026-10-29

Status remains in_progress, current active task08. Five inventory entries have an integrated source checkpoint;63 remain unimplemented natively. Automatic re-subscription, foreground/help, install/sync/checkin and actual trusted-sidecar/native process acceptance remain open.

Common policy, retained CommandJobs and process gateway now have GPUI and Tauri consumers. Tauri owns its wire/events and compile-time sidecar context. The preserved red test exposed free-text assignment secret leakage; ccr-core owner repair and the later four-byte UTF8 regression passed. Actual final app suite56 library+15 integration, core redaction5, native desktop44+4+2, Tauri command6/process1/prompt5/settings4 and strict/shared/native/Tauri gates passed. Transactional bindings and registry inventory match. No package versions changed.

Main-run receipts are under `outputs/desktop/continuation-20261028/resume29-*`; exact results/limitations and candidate8a0149 source identity are in the parent [continuation report](../10-17-gpui-desktop/research/continuation-20261029.md). Independent source review: [check report](research/check-20261029.md). Native jobs actions were not performed after minimized-window/user-input protection; app-owner fake-process tests are not labeled native GUI acceptance. No real CCR/domain command, install/network/account action, commit or archive.

## Route slice — 2026-11-05

PRD 8.1–8.6 stay open. This slice only connects three existing sidebar routes to local reads. The paper stays the solid sheet. The sidebar stays 13 routes. No real CCR command, provider call, WebDAV request, account write, password write, or user-home database open was run.

### `/commands` (`page-commands`)

The command catalog lens lists every entry from the existing local `command_catalog`, with its title and whether it can run. The rendered summary count matches that catalog. `status` is labeled “status · Status overview · Can run”. `delete` is labeled unavailable and cannot be selected or started. An empty catalog says “The local command catalog is empty.” and a start attempt is rejected before a job id exists. Invalid JSON arguments still do not start a job. Favorite and recent commands stay on the same route and say the list does not run a command. Visible copy is one language; the page labels do not contain “Home insights”, “Apply / refresh”, or “ / ”.

### `/checkin` (`page-checkin`)

The page reads saved accounts from the local SQLite database when that file already exists. A missing fixture database shows “No local check-in database was found. This page does not create one.” and does not create `checkin.db`, a WAL file, or a shared-memory file. It does not create `~/.ccr-ui`, import accounts, contact a provider, or turn a missing balance into 0. A seeded fixture shows “Unmeasured account” with “No stored balance.” and “Desk account” with the stored remaining `7.5 USD`, total `10`, used `2.5`, and the stored timestamps. Cookie text, API user, extra config, and provider URL containing `checkin-secret-sentinel` are not rendered. An existing database is opened read-only: the reader does not attach to the process pool, and a unit test keeps the file bytes and `journal_mode=delete` unchanged.

### `/sync` (`page-sync`)

The page reads only the canonical `sync_folders.toml`. A missing fixture file shows “No canonical sync config file was found. Legacy sync config is not read. Remote access was not tested.” and does not create the file, read a sibling `sync.toml`, or invent a Jianguoyun host. A present fixture shows that a username and password are stored, the folder `claude-fixture`, and “Remote access was not tested.” The username, password, and userinfo URL are not rendered. Password presence uses `Secret::is_empty` and does not call `expose()`. Both the documented `[[folder]]` table and the saved `[[folders]]` key are read. An unreadable file shows “The canonical sync config could not be read. Remote access was not tested.” without echoing the file’s secret. No sync, password write, or WebDAV request runs. Local mode resolves the canonical path through the existing folder manager; fixture mode uses the fixture directory and ignores that default path.

### Commands and results

| Command | Exit |
| --- | --- |
| `just desktop-fmt-check` | 0 |
| `just desktop-lint` | 0 |
| `just desktop-test` | 0 |

Independent check on 2026-11-05 re-ran those three commands after the read-only check-in open. Each exited 0. `just desktop-test` passed 12 Python tests, 3 AccessKit tests, 89 library tests, 8 shell tests, and 2 UI tests. The usage SQLite byte comparison did not fail, so it was not rerun. `ccr-desktop/Cargo.lock` records direct edges to the already locked `ccr-sync` 7.5.0, `dirs` 6.0.0, and `rusqlite` 0.40.2. No package version changed.

PRD 8.1–8.6 stay open. Native pixels, real sidecar execution, per-origin check-in throttling, live balance refresh, and sync transport remain unrun.
