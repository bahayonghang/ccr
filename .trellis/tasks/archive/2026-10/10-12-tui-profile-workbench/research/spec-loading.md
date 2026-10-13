# Implementation context loading

The following authoritative spec files exceed the current 32 KiB per-file native context injection cap. Their small indexes are in the manifests instead. Before implementing/checking the relevant platform, read the full file in bounded chunks so native truncation cannot silently omit constraints:

- `.trellis/spec/ccr-tui/backend/backend-guidelines.md`: required for all four tasks; includes state/render separation, localization, startup, background usage, per-tab selection, runtime/secret and async Auth contracts.
- `.trellis/spec/ccr-codex/backend/backend-guidelines.md`: required for the Codex child; includes auth/runtime/masking/test boundaries.

Use filesystem reads (for example PowerShell Get-Content slices of 150-200 lines) and inspect each chunk; no need to modify or split the authoritative specs for this planning task. `trellis-before-dev` remains the required entry before code changes.

Root AGENTS.md takes precedence over older serial-test examples: default test parallelism and `--skip export_bindings`. Native task context contains curated specs/research, not product source; read affected source at execution time and verify line anchors against the checkout.

Parent design.md and implement.md resolve choices among research recommendations. In particular: keep Profile navigation responsive while a Profile mutation is in flight, gate entry to Auth tabs, drain q/Esc/Ctrl+C only after the outcome, and preserve Enter/Space remaining in TUI. Research lists alternatives, not extra authorization or scope.
