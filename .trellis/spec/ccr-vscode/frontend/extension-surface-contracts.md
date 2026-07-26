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
- Hosted entry: `.github/workflows/vscode-ci.yml`, job name `VS Code Required`.

### 3. Contracts
- Hosted and local CI both run clean `npm ci`, TypeScript build checks, tests, `build:package`, VSIX creation, and artifact collection.
- Node is pinned to 24.18.0 and third-party actions use immutable commit SHAs.
- The coverage gate enforces at least 70% line coverage; the current function threshold is also 70%.
- Pull requests to `main`, `develop`, or `dev` that touch `ccr-vscode/**`, the root justfile, or the workflow must trigger the job.
- Workflow presence does not prove required branch protection; remote protection evidence is separate.

### 4. Validation & Error Matrix
- Lockfile and package manifest disagree -> `npm ci` fails.
- TypeScript source/test compile error -> `build-check` fails before packaging.
- Tests or line/function coverage below 70 -> Node test gate fails.
- VSIX cannot be created or collected -> `build:package`/artifact step fails.
- Required check not visible in branch protection -> repository-setting acceptance remains `UNVERIFIED`.

### 5. Good/Base/Bad Cases
- Good: change a provider helper, add its `*.test.ts`, then run `just vscode-ci` and `just vscode-coverage` locally.
- Base: documentation-only changes outside the extension do not trigger the extension workflow.
- Bad: replacing `npm ci` with mutable install behavior or testing only during tag release.

### 6. Tests Required
- `just vscode-ci` -> clean install, compile, 50 tests, package, and VSIX collection pass.
- `just vscode-coverage` -> line coverage at least 70% (current observed 91.79%).
- `python scripts/check_workflow_governance.py` -> pinned actions and branch/path policy pass.
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
    paths: ['ccr-vscode/**', 'justfile', '.github/workflows/vscode-ci.yml']
```
