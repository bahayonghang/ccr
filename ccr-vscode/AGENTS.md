# ccr-vscode Agent Notes

These notes apply to everything under `ccr-vscode/` and supplement the repository-level `AGENTS.md`.

## Structure And Ownership
- `src/extension.ts` is the VS Code extension entry point.
- `src/providers/` owns tree views, status bar presentation, and webview/profile editor UI logic.
- `src/services/` owns CCR CLI integration, config path discovery, watchers, TOML parsing, Codex auth/quota/runtime readers, and command argument construction.
- `src/models/` owns shared TypeScript types and platform capability metadata.
- Keep tests beside the implementation as `*.test.ts`; current suites use Node's built-in test runner through `tsx`.

## Build, Test, And Package Commands
- From `ccr-vscode/`, run `npm run build` for the esbuild bundle.
- Run `npm test` for extension tests; it runs `npm run build` first via `pretest`.
- Run `npm run lint` for TypeScript checks across runtime and test configs.
- Run `npm run build && npm test` as the default local verification before handing off VS Code extension changes.
- Use `just ci` from `ccr-vscode/` or `just vscode-ci` from the repository root for the full extension gate.
- Use `npm run package` or `just build` only when a `.vsix` artifact is required.

## Style And Safety
- Use 2-space indentation, single quotes, and no semicolons.
- Do not read or write real user secrets in tests; use fixtures, temporary directories, and mocked environment/path inputs.
- Preserve configuration masking and backward-compatible settings semantics when editing `package.json` contributions.
- Do not commit generated extension artifacts such as `dist/`, `node_modules/`, or `ccr-vscode.vsix` unless a release task explicitly requires them.
