# Raw Config Editor Contracts

> Cross-layer contract for editing local configuration, prompt, and profile source files from ccr-ui.

## Scenario: Local plaintext source editing with versioned saves

### 1. Scope / Trigger

- Trigger: adding a UI that reads and writes an entire local config-like file rather than a managed form subset.
- Applies to Tauri commands, domain API wrappers, `CodeSourceEditor`, and source-mode consumers.
- These files may contain credentials; the backend is the enforcement boundary.

### 2. Signatures

- Read command: `get_*_raw_text(state) -> Result<Value, String>`.
- Save command: `save_*_raw_text(state, content: String, token: String) -> Result<Value, String>`.
- Profile save command: `*_save_profiles_raw(state, content: String, token: String, force: bool) -> Result<Value, String>`.
- Read result: `{ status: 'ok', content, token, path, exists } | { status: 'unsupported_environment', envType }`.
- Save result: `{ status: 'saved', token } | { status: 'conflict' } | { status: 'invalid', kind, message, line?, column? } | unsupported`.
- Profile save result additionally supports `{ status: 'saved', token, profiles_count } | { status: 'activation_conflict', current }`.
- Shared editor: `CodeSourceEditor(value, language: 'json' | 'toml' | 'markdown', readOnly?, errorMarker?)`.
- Production CSP integration: `EditorView.cspNonce.of(pageNonce)`, where `pageNonce` comes from the current document's `style[nonce]` or `script[nonce]`.

### 3. Contracts

- Raw commands are Local-only. Check the active `ExecutionEnvironment` in every read, save, and layer-list command; frontend disablement is UX, not authorization.
- Read directly from disk and return a content-version token. Do not place content in stores, logs, monitoring fields, local storage, or route state.
- Before entering source mode, use `uiStore.requestConfirm` with a plaintext warning.
- Validate syntax and semantic shape before saving, but write the user's original bytes verbatim so comments, formatting, and key order survive.
- Save through `write_guarded_versioned` with `secret: true` and a dedicated backup policy.
- Expected failures are structured statuses. Reserve command `Err` for unexpected I/O, lock, or task failures.
- On `conflict`, offer reload/cancel only. Never silently refresh the token and overwrite external changes.
- Profile source saves reject an empty parsed profile collection as `invalid/semantic`. If the active profile is absent and `force` is false, return `activation_conflict`; only an explicit danger confirmation may retry the same content and token with `force: true`.
- After a successful source save, clear dirty state, leave source mode, then reload the managed form. This order avoids remounting the editor and repeating plaintext confirmation.
- After a successful profile source save, close the editor and perform the owning Profiles view's full refresh so cards, current-profile state, quick switching, and distribution data cannot remain stale.
- Frontend wrappers live in `src/api/domains/*`; shared discriminated unions live in a domain-adjacent type module.
- CodeMirror injects its base layout and theme through runtime `<style>` elements. In Tauri production builds, pass the page CSP nonce through `EditorView.cspNonce`; otherwise WebView2 rejects the entire stylesheet and can lay out editor content below the gutter even though the DOM text and computed foreground color are present.

### 4. Validation & Error Matrix

- Non-Local active environment -> `unsupported_environment`; no filesystem access.
- Syntax error -> `invalid/syntax` with parser-derived line/column when available; no backup/write.
- Known-field type mismatch -> `invalid/semantic` with line/column when available; no backup/write.
- Parsed profile collection is empty -> `invalid/semantic`; no backup/write.
- Active profile absent from parsed content + `force == false` -> `activation_conflict`; no backup/write.
- Active profile absent from parsed content + confirmed `force == true` -> continue to the same token check and guarded write.
- Stale token -> `conflict`; preserve external bytes and create no backup.
- Matching token -> `saved` with the new token; backup and atomic replacement complete before response.
- Unexpected filesystem/lock/join failure -> command `Err` containing path/category only, never source content.
- CodeMirror runtime `<style>` has no page nonce in a Tauri production WebView -> `style.sheet === null`; editor acceptance fails even if Vite/browser development mode renders correctly.

### 5. Good/Base/Bad Cases

- Good: edit TOML comments, save verbatim, then reload the form from disk.
- Good: remove the active profile only after a danger confirmation, then retry with `force: true` and refresh the full profile view.
- Base: a missing file reads as empty content with token `""` and can be created once.
- Good: the generated CodeMirror runtime `<style>` has the same nonce as the Tauri bootstrap style and exposes its CSS rules through `style.sheet`.
- Bad: log the payload or parser input while reporting validation errors.
- Bad: return every validation/conflict case as a rejected Tauri invoke promise.
- Bad: keep source content in Pinia so it survives component teardown.
- Bad: validate only with Vite and assume CodeMirror runtime styles will pass the stricter production CSP.

### 6. Tests Required

- Backend: syntax and semantic rejection, empty-profile rejection, activation-conflict force protocol, line/column, probe-content non-leakage, stale-token preservation, first creation, backup generation, secret permissions, and Local environment guard.
- Frontend: editor v-model/save/error marker; panel read-token/save-token flow; activation-conflict confirmation and force retry; conflict reload behavior; successful close plus full refresh; API facade guard.
- Frontend editor CSP guard: seed a bootstrap `<style nonce="test-nonce">`, mount `CodeSourceEditor`, then assert the generated CodeMirror runtime style has `nonce === 'test-nonce'`.
- Run `bun run type-check`, `bun run lint`, `bun run test:i18n`, focused smoke tests, Tauri command tests, and `cargo check`.
- Use the ccr-ui web preview for ordinary visual evidence, but verify runtime-style or CSP changes against a production Tauri build (`just tbuild` / `just tdev`). Record that the generated style has a nonce, `style.sheet` is readable, `.cm-scroller` is flex, and gutter/content top coordinates align.
- Before diagnosing a production-only editor failure, rebuild the desktop binary from the current checkout and record the binary timestamp relative to the suspected fix commit. A pre-fix release EXE is not evidence about current source behavior.

### 7. Wrong vs Correct

#### Wrong

```typescript
localStorage.setItem('raw-config', content)
await saveRaw(content, await getLatestToken())
```

#### Correct

```typescript
const loaded = await getRaw()
const result = await saveRaw(editedContent, loaded.token)
if (result.status === 'conflict') showReloadOnlyState()
if (result.status === 'activation_conflict') confirmThenRetryWithForce()
```

#### Wrong

```typescript
const extensions = [lineNumbers(), editorTheme]
```

This renders in Vite but leaves CodeMirror's runtime stylesheet without a Tauri CSP nonce.

#### Correct

```typescript
const pageNonce = document.querySelector<HTMLStyleElement>('style[nonce]')?.nonce
  || document.querySelector<HTMLScriptElement>('script[nonce]')?.nonce
const extensions = [
  ...(pageNonce ? [EditorView.cspNonce.of(pageNonce)] : []),
  lineNumbers(),
  editorTheme,
]
```
