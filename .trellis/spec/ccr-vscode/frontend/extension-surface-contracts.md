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
- Relevance signature: `python scripts/ci_surface_policy.py --surface vscode --base <sha> --head <sha>`.

### 3. Contracts
- Hosted and local CI both run clean `npm ci`, TypeScript build checks, tests, `build:package`, VSIX creation, and artifact collection.
- Node is pinned to 24.18.0 and third-party actions use immutable commit SHAs.
- The coverage gate enforces at least 70% line coverage; the current function threshold is also 70%.
- Every pull request to `main`, `develop`, or `dev` creates `VS Code Required`. Changes to `ccr-vscode/**`, the root justfile, the workflow, or `scripts/ci_surface_policy.py` set `relevant=true` and must run validation/coverage; other changes skip the heavy job and let only the aggregator pass.
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
- `just vscode-ci` -> clean install, compile, 50 tests, package, and VSIX collection pass.
- `just vscode-coverage` -> line coverage at least 70% (current observed 91.79%).
- `python -m unittest scripts/test_check_workflow_governance.py` and `python scripts/check_workflow_governance.py` -> path policy, stable context, and pinned actions pass.
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
# scripts/ci_surface_policy.py owns the heavy-job path policy; the stable
# required aggregator is created for every pull request.
```

## Scenario: Signed VSIX release boundary

### 1. Scope / Trigger
- Trigger: changing VSIX packaging, signature generation/verification,
  Marketplace publication, publisher identity, or the release signing runner.

### 2. Signatures
- Package: `vsce package --no-dependencies --sign-tool <tool> -o <file>.vsix`.
- Verify: `vsce verify-signature -i <file>.vsix -m <manifest> -s <p7s>`.
- Publish: `vsce publish --packagePath <vsix> --manifestPath <manifest>
  --signaturePath <p7s>` with `VSCE_PAT` supplied through the environment.

### 3. Contracts
- Signing runs only on the protected self-hosted runner labels
  `[self-hosted, linux, vsix-signing]`.
- `VSIX_SIGN_TOOL_PATH` is a protected environment variable naming an
  executable publisher-managed sign-tool; it is not downloaded from a mutable
  URL during the release.
- VSIX, signature manifest, and `.p7s` are kept together through attestation,
  Marketplace publication, and GitHub Release publication.
- `VSCE_PAT` is never passed as a command-line argument, echoed, or included in
  an artifact. Missing publisher authorization fails the job.

### 4. Validation & Error Matrix
- Missing/non-executable sign-tool -> preflight failure before packaging.
- Manifest or `.p7s` missing -> package job fails before upload.
- `verify-signature` failure -> no Marketplace or GitHub publication.
- Marketplace publish failure -> GitHub Release job remains blocked.
- Fixture signature only -> local verifier behavior passes, but external
  acceptance remains incomplete until a real publisher-signed VSIX verifies.

### 5. Good/Base/Bad Cases
- Good: package, manifest, and `.p7s` verify locally, are attested, then the
  exact tuple is published to Marketplace.
- Base: ordinary `just vscode-ci` packages an unsigned development VSIX and
  never calls Marketplace or labels it an official release.
- Bad: publishing a checksum-only VSIX or using `--skip-duplicate` to hide a
  publisher/version mismatch.

### 6. Tests Required
- `python -m unittest scripts/test_check_release_security.py`.
- `python scripts/check_release_security.py check`.
- `just vscode-ci` and `just ci-governance-check`.
- External acceptance: run `vsce verify-signature` on the downloaded release
  tuple and confirm the Marketplace publisher/version from the real release.

### 7. Wrong vs Correct
#### Wrong
```bash
npx vsce package -o extension.vsix
sha256sum extension.vsix
```

#### Correct
```bash
npx vsce package --sign-tool "$VSIX_SIGN_TOOL_PATH" -o extension.vsix
npx vsce verify-signature -i extension.vsix -m extension.signature.manifest -s extension.signature.p7s
```
