# Extension Surface Contracts

> Manifest, activation, and platform exposure rules for `ccr-vscode`.

---

## Scenario: Lazy activation with platform-capability exposure

### 1. Scope / Trigger
- Trigger: editing `ccr-vscode/package.json`, `src/extension.ts`, `src/models/platformCapabilities.ts`, or tree/status presentation helpers that decide what the user can mutate.
- Applies to command contributions, tree-view context values, platform labels, and status bar targets.

### 2. Signatures
- Extension entry: `activate(context)` in `src/extension.ts`
- Contributed commands:
  - `ccr.refreshProfiles`
  - `ccr.switchProfile`
  - `ccr.switchProfileForPlatform`
  - `ccr.addProfile`
  - `ccr.addProfileForPlatform`
  - `ccr.editProfileVisual`
  - `ccr.editProfileField`
  - `ccr.toggleProfileEnabled`
  - `ccr.deleteProfile`
  - `ccr.switchCodexAuth`
  - `ccr.editCodexAuth`
  - `ccr.deleteCodexAuth`
  - `ccr.openProfilesFile`
  - `ccr.selectStatusBarPlatform`
- Platform metadata: `SUPPORTED_PLATFORMS`, `PLATFORM_CAPABILITIES`
- Tree context helpers: `getPlatformNodeContextValue`, `getSectionNodeContextValue`, `getProfileNodeContextValue`

### 3. Contracts
- `package.json` should rely on VS Code's implicit activation for contributed commands/views; do not add `onStartupFinished` just to wake the extension early.
- `ccr.switchProfileForPlatform` must stay contributed in `package.json` and registered in `src/extension.ts`.
- Writable profile actions remain limited to `claude` and `codex`.
- `gemini`, `qwen`, and `droid` may be surfaced as registry/tree browse entries, but they must stay read-only in the VS Code extension.
- Status bar targets remain Claude/Codex only.
- Manifest copy may mention platform metadata, but it must not imply mutation support for unsupported platforms.

### 4. Validation & Error Matrix
- Added eager startup activation -> extension wakes too early and regresses lazy load behavior.
- Registered a command but forgot to contribute it -> command palette/menu surface drifts from runtime behavior.
- Exposed a read-only platform with writable context values -> menus show unsupported mutation actions.
- Expanded the status bar to read-only platforms -> the status bar boundary no longer matches CLI capabilities.

### 5. Good/Base/Bad Cases
- Good: `gemini` appears in the registry tree as `Antigravity CLI` with browse-only labels.
- Good: Claude/Codex keep switch, edit, enable, disable, and delete flows.
- Base: a command contribution and its `registerCommand()` handler share the same ID.
- Bad: add `onStartupFinished` after contributed commands already provide lazy activation.
- Bad: assign `platform-create-supported` to `gemini`, `qwen`, or `droid`.

### 6. Tests Required
- `src/packageManifest.test.ts` should verify no eager activation event is present and the platform-scoped command is contributed.
- `src/providers/profileTreeVisibility.test.ts` should verify writable vs read-only context values.
- `src/providers/profileTreePresentation.test.ts` should verify browse-only platform labels.
- `cd ccr-vscode && npm run lint`
- `cd ccr-vscode && npm test`

### 7. Wrong vs Correct
#### Wrong
```json
{
  "activationEvents": ["onStartupFinished"],
  "contributes": {
    "commands": [
      { "command": "ccr.switchProfile", "title": "CCR: Switch Profile" }
    ]
  }
}
```

#### Correct
```json
{
  "contributes": {
    "commands": [
      { "command": "ccr.switchProfile", "title": "CCR: Switch Profile" },
      { "command": "ccr.switchProfileForPlatform", "title": "CCR: Switch Profile For Platform" }
    ],
    "views": {
      "ccr": [
        { "id": "ccr-profiles", "name": "Profiles" }
      ]
    }
  }
}
```

## Scenario: Clean PR CI and line coverage

### 1. Scope / Trigger
- Trigger: changing `ccr-vscode/**`, its hosted workflow, package lock, test runner, or packaging recipe.
- Applies because extension code previously built only during tag release and lacked a PR check.

### 2. Signatures
- Local required gate: root `just vscode-ci` -> `ccr-vscode/justfile` recipe `ci`.
- Local coverage gate: root `just vscode-coverage` -> Node `--experimental-test-coverage --test-coverage-lines=70 --test-coverage-functions=70`.
- Hosted entry: `.github/workflows/vscode-ci.yml`; heavy job `VS Code Validation`, stable branch-protection aggregator `VS Code Required`.
- Relevance signature: `python scripts/ci/ci_surface_policy.py --surface vscode --base <sha> --head <sha>`.

### 3. Contracts
- Hosted and local CI both run clean `npm ci`, TypeScript build checks, tests, `build:package`, VSIX creation, and artifact collection.
- Node is pinned to 24.20.0 and third-party actions use immutable commit SHAs.
- The coverage gate enforces at least 70% line coverage; the current function threshold is also 70%.
- Every pull request to `main`, `develop`, or `dev` creates `VS Code Required`. Changes to `ccr-vscode/**`, the root justfile, the workflow, or `scripts/ci/ci_surface_policy.py` set `relevant=true` and must run validation/coverage; other changes skip the heavy job and let only the aggregator pass.
- Workflow presence does not prove required branch protection; remote protection evidence is separate.

### 4. Validation & Error Matrix
- Lockfile and package manifest disagree -> `npm ci` fails.
- TypeScript source/test compile error -> `build-check` fails before packaging.
- Tests or line/function coverage below 70 -> Node test gate fails.
- VSIX cannot be created or collected -> `build:package`/artifact step fails.
- Required check not visible in branch protection -> repository-setting acceptance remains `UNVERIFIED`.
- Relevance detection fails, or a relevant validation is skipped/cancelled/failed -> `VS Code Required` fails closed.

### 5. Good/Base/Bad Cases
- Good: change a provider helper, add its `*.test.ts`, then run `just vscode-ci` and `just vscode-coverage` locally.
- Base: documentation-only changes outside the extension create the lightweight `VS Code Required` context but skip install, test, coverage, and package work.
- Bad: using a PR-level `paths` filter with `VS Code Required`; branch protection waits forever when the workflow is absent.
- Bad: replacing `npm ci` with mutable install behavior or testing only during tag release.

### 6. Tests Required
- `just vscode-ci` -> clean install, compile, package-file tests, 51 extension tests, package, and VSIX collection pass.
- `just vscode-coverage` -> line and function coverage at least 70%; record measured results with the test runtime version.
- `python -m unittest scripts.ci.test_check_workflow_governance` and `python scripts/ci/check_workflow_governance.py` -> path policy, stable context, and pinned actions pass.
- Inspect an actual PR check run and protected-branch required-check list when remote permission is available.

### 7. Wrong vs Correct
#### Wrong
```yaml
on:
  push:
    tags: ['v*']
```

#### Correct
```yaml
on:
  pull_request:
    branches: [main, develop, dev]
# scripts/ci/ci_surface_policy.py owns the heavy-job path policy; the stable
# required aggregator is created for every pull request.
```

## Scenario: Runtime package allowlist

### 1. Scope / Trigger
- Applies to Claude Code, Codex, Grok Build, Kimi Code, and OMP when changing extension packaging or release assets.
- Trigger: editing `.vscodeignore`, packaging scripts, runtime files, icons, or package metadata.

### 2. Signatures
- `.vscodeignore` excludes all files except the exact runtime allowlist.
- `scripts/check-package-files.mjs` validates both `vsce ls --no-dependencies` output and final VSIX entry names.
- `npm run check:package` checks source paths; `node scripts/check-package-files.mjs --vsix <path>` checks any final archive.
- `npm run package` builds, packages, and checks the final archive. The local `just build` and PR `just vscode-ci` use that command before artifact collection.

### 3. Contracts
- Keep the exact allowlists in `.vscodeignore` and the checker aligned. The package requires `package.json`, `dist/extension.js`, `icon.png`, seven named resource icons, `LICENSE`, `README.md`, and `CHANGELOG.md`.
- VSCE changes `README.md` to `extension/readme.md`, `CHANGELOG.md` to `extension/changelog.md`, and `LICENSE` to `extension/LICENSE.txt`. The archive also requires `[Content_Types].xml` and `extension.vsixmanifest`.
- Reject unexpected, duplicate, missing, and noncanonical entries. Reject local tool state, `*.local.*`, source maps, internal agent instructions, and unapproved runtime files.
- Read archive entry names only. Resolve the existing ZIP reader from the declared `@vscode/vsce` dependency. A reader or archive error fails the check.
- Use synthetic temporary directories for local-configuration negative tests. Do not inspect real local configuration contents.
- `vscode:prepublish` runs the source-list check for direct `vsce package` calls, including the existing release workflow. The current direct release workflow has no final-archive check; a passing prepublish check does not provide that evidence.
- Changes to the release workflow or publication behavior require the matching approved task scope. Do not publish as part of package verification.

### 4. Tests Required
- `node --test ccr-vscode/scripts/check-package-files.test.mjs` covers synthetic local files, missing assets, duplicate and noncanonical paths, final archive contamination, and invalid archives.
- `just vscode-ci` runs package-file tests, the existing extension tests, and both package checks.
- `cd ccr-vscode && npx --no-install vsce ls` exposes the selected source inventory for review.
- `cd ccr-vscode && npm run check:vsix` validates the actual local package.
- Package success does not prove native extension activation, Marketplace acceptance, or hosted release validation. Record those boundaries separately.

## Scenario: Approved packaging dependency patches

- Applies to Claude Code, Codex, Grok Build, Kimi Code, and OMP when updating the extension lockfile.
- Verify the approved target versions against official advisory ranges and registry metadata. Hash downloaded registry tarballs and compare the result with the published integrity before changing lock nodes.
- Keep the existing parent dependency ranges. For a scoped patch, change only the approved lock nodes and record the exact changed fields. Do not replace that patch with a bulk update or an audit exception.
- Preserve the initial failing audit receipt. Save the corrected audit result as separate evidence. Package success and dependency audit success remain separate checks.
- Run clean `npm ci`, explicit `npm audit`, `just vscode-ci`, `just vscode-coverage`, and the final VSIX file check. Keep the existing 70% line and function thresholds.
- Record the actual Node version and local versus hosted evidence. A local pass does not prove hosted or native extension behavior.
