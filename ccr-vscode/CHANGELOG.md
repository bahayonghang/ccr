# Changelog

## [5.0.9]

### Security
- Use crypto-safe nonce generation (`crypto.randomBytes`) instead of `Math.random()`
- Validate WebView message fields before processing (guard `typeof msg.field === "string"`)
- Mask auth tokens before sending to WebView; only write back on explicit user edit

### Changed
- Deduplicate field mapping: derive from `EDITABLE_FIELDS` single source of truth
- Convert `writeProfileField` and `toggleProfileEnabled` to async (`fs.promises.writeFile`)
- Remove `retainContextWhenHidden`; use `vscode.setState/getState` for WebView persistence
- Show disabled platforms in TreeView (grayed out with `eye-closed` icon, non-expandable)
- Add `ccr.confirmBeforeSwitch` setting to control confirmation dialog
- Extract `refreshAll()` helper to reduce duplication in `extension.ts`
- Move `activePanels` to static class field with `disposeAll()` lifecycle method
- Fix `err.code` type assertion in CLI service to use `typeof` guard
- Add `extensionKind: ["workspace"]` and `"Configuration"` category to `package.json`

### Added
- Enter key saves current field in WebView editor
- Loading state indicator in WebView editor
