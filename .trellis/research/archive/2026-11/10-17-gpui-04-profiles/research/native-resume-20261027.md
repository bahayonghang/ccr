# 04 native acceptance resumed, 2026-10-27

The user explicitly resumed the existing GPUI task. Historical pause and evidence remain intact. Main owns native automation and task/inventory integration; the visual implementer owns theme/shell, and a separate Trellis checker owns the newly observed current-profile defect. No real user configuration/account, global HOME, commit, push, publication or archive operation is used.

## Source identity and launch recovery

Before any product edits, all 15 entries in `outputs/desktop/profiles/native-check-freeze.json` and the independent desktop lock matched their frozen hashes. Debug executable SHA256 was `2d7b4271c07dbf0e14fd301ee3df7b6f8ba73996ac570eacf8b947645a606e26`.

The first new private fixture launch exited101 before a window: `DirectXRenderer` recovery reported a missing DirectWrite shader path. The exact locked GPUI registry source directory was absent. Normal locked compilation restored that source. The same debug executable then opened successfully; this is evidence for missing development shader resources, not proof of a GPU-driver defect. The source/API chain is in03 `research/airy-glass-capabilities.md`. Original failure remains in `outputs/desktop/profiles/native/resume-20261027-core/{receipt.json,stderr.log}`.

The recovered run uses a fresh copy of the closed domain-generated seed and paired explicit profiles/registry arguments. Its directory is `outputs/desktop/profiles/native/resume-20261027-core-recovered/`. Native PID31156 and `MainWindowHandle=20061654` matched the exact returned `CCR Desktop (GPUI)` window at selection. The automation tool attributed its app identifier to the host app; native process/window-handle evidence confirmed the actual target. One subsequent capture showed another surface, so main discarded that observation and rebound/activated the verified window before further input. That capture is not accepted as GPUI evidence.

## Actual observations so far

- Native list showed two synthetic profiles, `a-first` Current and `z-last` Disabled. Selected detail masked the credential and showed the expected nonsecret description/endpoint. Screenshots01/02 in the recovered run capture this state.
- Edit/rename opened the selected `a-first` profile. Main focused Description, selected its contents, typed `native-resume-20261027`, scrolled the editor and clicked Save. Typed input, editor scrolling and Save genuinely ran through the native UI.
- Save reported `Profile updated · warnings: []`; the description reread correctly, but Current unexpectedly moved to `z-last`. Explicit Refresh retained that marker. Screenshot03 preserves the unexpected result. **This flow is not accepted as PASS.**
- Read-only checker inspection found the seed's catalog marker `ghost` and default `z-last`, while registry current remained `a-first`. Metadata save changed the catalog marker to `z-last`. A checker is reproducing and fixing the domain owner behavior; its independent report owns the final root cause and tests.
- Main reopened `a-first` and entered a `-dirty` suffix for a real second-client CAS scenario. The old precompiled helper path no longer exists; a fresh helper artifact must be built/hashed before that scenario can proceed. No live DB copy or hand-written SQL mutation is used.
- Main closed that old debug fixture normally; the launcher recorded native exit0. The pending draft was not saved and the second-client scenario remains NOT_RUN for this run. A fresh test helper was subsequently compiled under `outputs/desktop/profiles/resume-current-check/target/`; final CAS acceptance will use the rebuilt candidate and a new fixture.

## Pending

Current-marker fix/rebuild/native rerun; real second-client conflict and retained draft; apply/current result; missing-catalog retry; close-before-delay cancellation; light/dark visual evidence after03's new palette. This document records old product behavior only. It cannot establish new airy-glass acceptance, OS accessibility/IME, S1/S2 performance, full42-command verification or completed04. Size/DPI matrices remain user-deferred.
