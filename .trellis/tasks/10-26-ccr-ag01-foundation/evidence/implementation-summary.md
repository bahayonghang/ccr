# AG01 implementation evidence

- Baseline: `before/` captured before all product edits; 20 screenshots, 12 route timings, 3 real scroll samples. Baseline HEAD is `ab17ec92`; full hash is in metadata. Product diff was empty and fingerprint preserved. Preview was the existing task-owned Node/Vite wrapper (original PID 82260), Chrome headless and reduced-motion.
- Initial after: `after/` preserves the first AG01 pass (20 screenshots/12 route samples/3 scroll samples). Later semantic text/boundary calibrations and AG02/AG03 work occurred after this snapshot. Do not call it final integration acceptance. The same capture script accepts `after final-after` to preserve both snapshots.
- Runtime: `material-runtime/results.json` contains 28 passing samples. Four light/dark × neutral/clay combinations, 1200/959/960/1023/1024/390 widths and CDP reduced-transparency. Counter includes generated pseudo-elements. Exactly one app-sheet blur on desktop; zero below 960 and under reduced transparency, with solid fallback. Actual media match is awaited before inspection.
- Governance: `token-governance/summary.json` + before/after/added/removed name sets; unique definitions under src/styles/**/*.css are 478 → 486 (+8, -0). Before reconstructed from baseline HEAD only after checking the captured product diff SHA256 is the empty hash. No Tailwind namespace/provider addition.
- New eight tokens: field start/end, sheet bg/solid/blur/border/shadow/radius. Existing shared roles and modal/floating material remain opaque and no-blur. One config-card legacy blur consumer was migrated then passed to AG02 ownership.
- Unique paper uses `.layout-shell::before`; putting filter on a pseudo-element avoids a new containing block for fixed navigation descendants. Existing content-scroll-area, route key, ref, caching and interaction wiring are unchanged.
- Colors preserve existing preference keys and font tracks. Accent clay key now maps to light #156f4e/dark #85d5b1. Status text roles were calibrated against every reading surface and matching tint; control strong borders pass 3:1 across base/elevated/surface/overlay.

## Checks run

- Initial focused theme/shell/overlay/confirm smoke: PASS 17 files / 114 tests.
- TypeScript check using installed Node underlying package binary: PASS.
- Production `bun run build` using existing Bun1.4.4 with process-local PATH: PASS,1472 modules. Existing unknown `@tailwind` warning; icons:ensure used committed assets because uv unavailable.
- Initial full smoke using Node: 168 files passed,2 failed;902 passed,4 failed. One #fff shorthand consistency failure was fixed in flavorPreview; three failures were child processes finding a broken Bun shim. Existing installed Bun executable was subsequently located and no install was performed. Final canonical full suite belongs to main integration check.
- Canonical theme suite after semantic calibration: PASS13 files/110 tests.
- Final strengthened contrast suite: PASS1 file/34 tests.
- `bun run lint`: PASS,0 errors and2 pre-existing ESLint warnings. Final `bun run lint:style`: PASS after border change.

## Limits

- Native Windows/WebView, actual DPI and macOS window APIs remain NOT_RUN; native appearance changes are visual constants plus narrow mock expectation only.
- No-support fallback has static source assertions only; installed Chrome supports backdrop-filter. No unsupported-browser result was invented.
- Profiles/Usage browser-unavailable states do not prove populated IPC/chart/virtual-list performance. Existing before/after captures explicitly state this.
- Final four-combo/status screenshots, final performance comparison and broad route/keyboard/portal acceptance belong to AG04. Final preview needs restart after last source edits because wrapper deliberately has watch:null. Latest task-owned server PID34988; no product Vite config changed.
