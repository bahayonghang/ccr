# ccr-vscode/ code map

Navigation map for `ccr-vscode/**`. Behavioral rules stay in `./AGENTS.md`; use this file to route extension edits before broad grep.

## Start here

- Scoped agent rules: `./AGENTS.md`.
- Extension manifest and contribution points: `package.json`.
- Local just recipes: `justfile` (`install`, `lint`, `test`, `build`, `ci`).
- TypeScript configs: `tsconfig.json`, `tsconfig.test.json`.
- Runtime entry point: `src/extension.ts`.

## Source map

| Path | Purpose |
|---|---|
| `src/extension.ts` | Activates the extension, registers tree/status providers, watchers, and commands contributed by `package.json`. |
| `src/models/types.ts` | Shared extension model types, profile fields, platform/auth DTOs, and editable field contracts. |
| `src/models/platformCapabilities.ts` | Platform capability metadata that drives presentation and command availability. |
| `src/providers/profileTreeProvider.ts` | Tree data provider and node model for the CCR sidebar. |
| `src/providers/profileTreePresentation.ts` | Pure presentation helpers for tree labels, descriptions, icons, and context values. |
| `src/providers/profileEditorPanel.ts` | Webview/profile editor lifecycle and message handling. |
| `src/providers/profileEditorPanel.helpers.ts` | Pure helpers for profile editor normalization and field behavior. |
| `src/providers/statusBarProvider.ts` | Status bar item lifecycle and refresh behavior. |
| `src/providers/statusBarPresentation.ts`, `src/providers/statusBarTarget.ts` | Pure status bar display and target-selection helpers. |
| `src/services/ccrCli.ts` | CCR CLI availability and command execution wrappers. |
| `src/services/ccrCliArgs.ts` | Command argument construction; keep quoting/backward compatibility here. |
| `src/services/ccrPaths.ts` | CCR config/profile path discovery. |
| `src/services/ccrWatcher.ts` | File watchers that refresh tree and status bar state. |
| `src/services/tomlReader.ts` | TOML parsing for CCR registry/profile files. |
| `src/services/codexAuthReader.ts`, `src/services/codexQuotaReader.ts`, `src/services/codexRuntimeReader.ts` | Codex auth/quota/runtime readers with cache invalidation points used by `extension.ts`. |
| `resources/` | Icons and extension resources referenced by `package.json` and providers. |

## Test routing

- Tests live beside implementation as `*.test.ts`.
- Provider presentation changes: start with `src/providers/*Presentation.test.ts` or `profileEditorPanel.test.ts`.
- CLI/path argument changes: start with `src/services/ccrCli.test.ts` and `src/services/ccrPaths.test.ts`.
- Status bar target/presentation changes: run the matching `statusBar*.test.ts` files.
- Full local verification from this directory: `npm run lint` and `npm test`; `npm test` runs `npm run build` first via `pretest`.
- Full extension gate: `just ci` from `ccr-vscode/` or `just vscode-ci` from the repository root.

## Packaging and generated output

- `dist/` is esbuild output and `ccr-vscode.vsix` is the packaged artifact; do not edit either by hand.
- `npm run build` writes `dist/extension.js`.
- `npm run package` or `just build` produces a `.vsix`; use only when an artifact is required.
- `node_modules/`, `dist/`, and `.vsix` files are generated/local outputs unless a release task explicitly says otherwise.

## Safety and compatibility boundaries

- Do not read or write real user secrets in tests; mock paths, environment, CLI output, and TOML content.
- Preserve `package.json` contribution IDs, command IDs, and configuration keys unless the change intentionally migrates extension API.
- Keep `package.json` publisher metadata separate from desktop/Tauri publisher metadata in `ccr-ui`.
- When changing CCR CLI invocation, update both argument builders and the tests that lock command shape.
