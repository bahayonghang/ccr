# Frozen release before visual implementation

Date: 2026-10-27. This is a source/visual/behavior reference, not a performance PASS.

Before any visual product edit, `just desktop-build` completed exit0 with the original sources and locked graph. The immutable executable is `outputs/desktop/airy-glass/2026-10-27/ccr-gpui-before.exe`, SHA256 `42e3aadb2fd6b676aafbe1d4882426c3bdf3733496c910645e1ddeb38612e1a8`. Environment, source and closed-fixture hashes are in `before-environment.json`; build identity is in `before-release-receipt.json` in that directory.

Main launched that copied release using a fresh closed-seed copy and explicit fixture inputs. Actual process PID68072 and main window handle14551756 matched the exact returned `CCR Desktop (GPUI)` window. The native launcher receipt at `outputs/desktop/profiles/native/before-release-20261027/receipt.json` records the matching executable hash, paired fixture paths, no HOME override and final native exit0.

Observed native behavior: two masked synthetic profiles loaded with `a-first` current; typing `a-first` filtered to one row; Settings opened; Light applied; returning to Configurations retained the query and filtered row. No size, zoom or DPI change was made. Captured screenshots are `outputs/desktop/airy-glass/2026-10-27/before-native/release-dark.jpg`, `release-filter.jpg` and `release-light.jpg`.

Presented frames, input-to-submit timing, sustained scrolling/logs, 120-second S1 CPU and S1/S2 memory distributions were NOT_RUN. The current application lacks the required insights/jobs workload surfaces and timing instrumentation; concurrent compilation also makes this an unsuitable full performance run. Basic native input/navigation observation cannot be substituted for those measurements. The frozen before executable permits future measurement by the same method when the workload is available. Tauri comparison remains separate and NOT_COMPARABLE until a valid complete baseline exists.
