# Transactional encrypted WebDAV sync design

## Layered boundary

The implementation is split into four independently tested layers:

1. `RemoteEntryName` validates one decoded remote path component.
2. `SyncBudget` accounts for depth, entries, bytes, and deadline while data is
   streamed.
3. `PullTransaction` stages, validates, durably swaps, and restores on failure.
4. `EncryptedEnvelopeV2` protects sensitive asset bytes before WebDAV sees
   them.

No layer trusts normalization performed by `reqwest_dav`.

## Remote name and containment

The final href segment is percent-decoded exactly once. It must be non-empty,
within the length limit, and parse as exactly one `Component::Normal`; `.`,
`..`, separators, drive prefixes, UNC forms, NUL/control characters, and nested
segments are rejected. The joined destination is checked lexically before
creation and canonically after its parent exists.

Cycles and duplicate normalized hrefs are tracked in a visited set. Rejections
are coded and counted without exposing credentials.

## Pull transaction

For each asset, the backend creates sibling staging and backup paths under the
same parent/filesystem. It streams into staging while enforcing `SyncBudget`,
validates the manifest and asset format, fsyncs staged files/directories, then
performs the lock-held transition:

`active -> backup`, `staging -> active`, `fsync(parent)`, remove backup.

Any failure before commit leaves active bytes untouched. Any failure after the
first rename restores the backup before returning. The transaction reports
whether rollback ran and whether it succeeded.

The sync truth table is explicit: absent/remote-only pulls, local-only pushes,
both-present without force returns a typed conflict, and an explicit
prefer-local/force action overwrites remote data. Boolean fall-through is
removed.

## Configuration ownership

The folder-manager representation becomes the canonical WebDAV configuration.
The legacy manager is read-only migration input. The first successful read
migrates through a single lock-held guarded write; subsequent saves write only
the canonical representation. This removes dual-commit compensation debt.

## Transport policy and budgets

- URL validation runs at save and connect. HTTPS is required except loopback
  HTTP when an explicit development setting is enabled.
- Responses are streamed to files; no `.bytes()` whole-body path remains for
  directory pull.
- Defaults are conservative and centrally declared: maximum file bytes, total
  bytes, depth, entry count, component length, redirects, and operation
  deadline. Exceeding any limit aborts and rolls back.

## Sensitive asset envelope

Sensitive assets use a versioned AES-256-GCM envelope with Argon2id-derived key,
random salt/nonce, authenticated asset kind/schema metadata, and explicit KDF
parameters. The crypto implementation reuses workspace-vetted primitives.

Key acquisition is separated from WebDAV credentials. The renderer requests an
independent passphrase for each sync operation and submits it through a typed
secret-bearing IPC input. The backend derives the envelope key, retains the
passphrase/key only for the operation lifetime, and never writes either to
config JSON, the local secret store, logs, events, audit fields, or remote
metadata. Plaintext v1 assets are readable only for an explicit migration flow;
all new sensitive writes are v2. Cross-device restore prompts for the same
passphrase and derives the key from the envelope KDF parameters.

## Compatibility and rollback

- Existing plaintext remote data remains exportable/readable through explicit
  v1 migration; it is never silently rewritten without a local backup.
- v2 writes can be disabled per asset if rollout fails, but plaintext writes of
  sensitive assets are not re-enabled by default.
- Transaction rollback always prioritizes preservation of the pre-operation
  active state.
