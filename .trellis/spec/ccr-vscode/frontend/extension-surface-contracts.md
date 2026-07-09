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
